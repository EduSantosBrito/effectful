//! Runtime service context for the v1 dependency injection API.
//!
//! This is the Rust equivalent of Effect's `Context`: a typed service table keyed by
//! the service type itself. Values are still type checked at lookup sites, but callers
//! no longer need to expose HList paths such as `There<Here>` in application code.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::context::{Cons, Nil, Tagged};
use crate::runtime::Never;

/// Service marker implemented by `#[derive(Service)]`.
///
/// A service type is both the key and the value stored in [`ServiceContext`].
/// This intentionally mirrors Effect v4's `Context.Service` class style while
/// staying Rust-native.
pub trait Service: Clone + 'static {
  /// Human-readable service name used in diagnostics.
  const NAME: &'static str;

  /// Wrap this service in a fresh [`ServiceContext`].
  #[inline]
  fn to_context(self) -> ServiceContext
  where
    Self: Sized,
  {
    ServiceContext::empty().add(self)
  }

  /// Build a layer that provides this concrete service value.
  #[inline]
  fn layer(self) -> crate::layer::Layer<Self, Never, ()>
  where
    Self: Sized,
  {
    crate::layer::Layer::succeed(self)
  }

  /// Access this service and pass it to an effectful callback.
  #[inline]
  fn use_<A, E, R, F>(f: F) -> crate::Effect<A, E, R>
  where
    Self: Sized,
    A: 'static,
    E: From<MissingService> + 'static,
    R: ServiceLookup<Self> + 'static,
    F: FnOnce(Self) -> crate::Effect<A, E, R> + 'static,
  {
    crate::Effect::<Self, E, R>::service::<Self>().flat_map(f)
  }

  /// Access this service and pass it to a synchronous callback.
  #[inline]
  fn use_sync<A, E, R, F>(f: F) -> crate::Effect<A, E, R>
  where
    Self: Sized,
    A: 'static,
    E: From<MissingService> + 'static,
    R: ServiceLookup<Self> + 'static,
    F: FnOnce(Self) -> A + 'static,
  {
    crate::Effect::<Self, E, R>::service::<Self>().map(f)
  }
}

/// A typed service table keyed by service type.
#[derive(Default)]
pub struct ServiceContext {
  entries: HashMap<TypeId, ServiceEntry>,
}

struct ServiceEntry {
  name: &'static str,
  value: Box<dyn Any>,
  clone_value: fn(&dyn Any) -> Box<dyn Any>,
}

impl Clone for ServiceEntry {
  fn clone(&self) -> Self {
    Self {
      name: self.name,
      value: (self.clone_value)(self.value.as_ref()),
      clone_value: self.clone_value,
    }
  }
}

impl Clone for ServiceContext {
  fn clone(&self) -> Self {
    Self {
      entries: self.entries.clone(),
    }
  }
}

impl fmt::Debug for ServiceContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut names = self
      .entries
      .values()
      .map(|entry| entry.name)
      .collect::<Vec<_>>();
    names.sort_unstable();
    f.debug_struct("ServiceContext")
      .field("services", &names)
      .finish()
  }
}

impl ServiceContext {
  /// An empty service context.
  #[inline]
  pub fn empty() -> Self {
    Self {
      entries: HashMap::new(),
    }
  }

  /// Add or replace a service in this context.
  #[inline]
  pub fn add<S>(mut self, service: S) -> Self
  where
    S: Service,
  {
    self.insert(service);
    self
  }

  /// Mutably add or replace a service.
  #[inline]
  pub fn insert<S>(&mut self, service: S)
  where
    S: Service,
  {
    self.entries.insert(
      TypeId::of::<S>(),
      ServiceEntry {
        name: S::NAME,
        value: Box::new(service),
        clone_value: |value| {
          Box::new(
            value
              .downcast_ref::<S>()
              .expect("service context entry stored under the wrong type")
              .clone(),
          )
        },
      },
    );
  }

  /// Merge services from `other`, replacing duplicate service keys with `other`.
  #[inline]
  pub fn merge(mut self, other: ServiceContext) -> Self {
    self.entries.extend(other.entries);
    self
  }

  /// Borrow a service by type.
  #[inline]
  pub fn get<S>(&self) -> Option<&S>
  where
    S: Service,
  {
    self
      .entries
      .get(&TypeId::of::<S>())
      .and_then(|entry| entry.value.downcast_ref::<S>())
  }

  /// Clone a service by type.
  #[inline]
  pub fn get_cloned<S>(&self) -> Option<S>
  where
    S: Service,
  {
    self.get::<S>().cloned()
  }

  /// Return true if the service is present.
  #[inline]
  pub fn contains<S>(&self) -> bool
  where
    S: Service,
  {
    self.entries.contains_key(&TypeId::of::<S>())
  }

  /// Human-readable service names currently present.
  pub fn service_names(&self) -> Vec<&'static str> {
    let mut names = self
      .entries
      .values()
      .map(|entry| entry.name)
      .collect::<Vec<_>>();
    names.sort_unstable();
    names
  }
}

/// Read access to a service from an environment.
pub trait ServiceLookup<S: Service> {
  /// Borrow a service.
  fn service(&self) -> Option<&S>;
}

impl<S> ServiceLookup<S> for ServiceContext
where
  S: Service,
{
  #[inline]
  fn service(&self) -> Option<&S> {
    self.get::<S>()
  }
}

impl<S> ServiceLookup<S> for &ServiceContext
where
  S: Service,
{
  #[inline]
  fn service(&self) -> Option<&S> {
    (*self).get::<S>()
  }
}

/// Error returned when a required service is missing from [`ServiceContext`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingService {
  /// Name of the requested service.
  pub name: &'static str,
}

impl fmt::Display for MissingService {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "missing service `{}`", self.name)
  }
}

impl std::error::Error for MissingService {}

/// Convert a compile-time HList [`Context`] into a runtime [`ServiceContext`].
///
/// Only self-keyed service cells (`Tagged<S, S>` where `S: `[`Service`]) are
/// extracted; arbitrary tagged values cannot silently enter runtime service
/// lookup.
///
/// ## Example
///
/// ```ignore
/// use effectful::{ctx, Effect, IntoServiceContext, MissingService, Service, ServiceContext,
///   run_blocking};
///
/// #[derive(Clone, Service)]
/// struct Config { port: u16 }
///
/// let static_ctx = ctx!(Config => Config { port: 8080 });
/// let env: ServiceContext = static_ctx.into_service_context();
///
/// let program: Effect<u16, MissingService, ServiceContext> =
///   Config::use_sync(|config| config.port);
///
/// assert_eq!(run_blocking(program, env), Ok(8080));
/// ```
pub trait IntoServiceContext {
  /// Consume `self` and produce a [`ServiceContext`].
  fn into_service_context(self) -> ServiceContext;
}

impl IntoServiceContext for ServiceContext {
  #[inline]
  fn into_service_context(self) -> ServiceContext {
    self
  }
}

impl IntoServiceContext for Nil {
  #[inline]
  fn into_service_context(self) -> ServiceContext {
    ServiceContext::empty()
  }
}

impl<S, Tail> IntoServiceContext for Cons<Tagged<S, S>, Tail>
where
  S: Service,
  Tail: IntoServiceContext,
{
  #[inline]
  fn into_service_context(self) -> ServiceContext {
    let tail = self.1.into_service_context();
    tail.add(self.0.value)
  }
}

impl<L> IntoServiceContext for crate::context::Context<L>
where
  L: IntoServiceContext,
{
  #[inline]
  fn into_service_context(self) -> ServiceContext {
    self.0.into_service_context()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, crate::Service)]
  struct Config {
    port: u16,
  }

  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, crate::Service)]
  struct Db {
    id: u8,
  }

  mod into_service_context {
    use super::*;

    #[test]
    fn nil_produces_empty_context() {
      let ctx = Nil.into_service_context();
      assert!(ctx.service_names().is_empty());
    }

    #[test]
    fn single_service_cell_converts() {
      let ctx = Cons(Tagged::<Config, _>::new(Config { port: 8080 }), Nil).into_service_context();
      assert_eq!(ctx.get_cloned::<Config>(), Some(Config { port: 8080 }));
    }

    #[test]
    fn multiple_service_cells_convert() {
      let ctx = Cons(
        Tagged::<Config, _>::new(Config { port: 8080 }),
        Cons(Tagged::<Db, _>::new(Db { id: 1 }), Nil),
      )
      .into_service_context();
      assert_eq!(ctx.get_cloned::<Config>(), Some(Config { port: 8080 }));
      assert_eq!(ctx.get_cloned::<Db>(), Some(Db { id: 1 }));
    }

    #[test]
    fn context_wrapper_converts() {
      let ctx =
        crate::context::Context::new(Cons(Tagged::<Config, _>::new(Config { port: 9090 }), Nil));
      let svc_ctx = ctx.into_service_context();
      assert_eq!(svc_ctx.get_cloned::<Config>(), Some(Config { port: 9090 }));
    }

    #[test]
    fn service_context_identity() {
      let ctx = ServiceContext::empty().add(Config { port: 3000 });
      let converted = ctx.into_service_context();
      assert_eq!(
        converted.get_cloned::<Config>(),
        Some(Config { port: 3000 })
      );
    }

    #[test]
    fn duplicate_service_types_head_wins() {
      let ctx = Cons(
        Tagged::<Config, _>::new(Config { port: 1 }),
        Cons(Tagged::<Config, _>::new(Config { port: 2 }), Nil),
      )
      .into_service_context();
      assert_eq!(ctx.get_cloned::<Config>(), Some(Config { port: 1 }));
    }

    #[test]
    fn service_lookup_trait_still_works_after_conversion() {
      let ctx = Cons(Tagged::<Config, _>::new(Config { port: 8080 }), Nil).into_service_context();
      let looked_up: Option<&Config> = ServiceLookup::<Config>::service(&ctx);
      assert_eq!(looked_up, Some(&Config { port: 8080 }));
    }
  }
}
