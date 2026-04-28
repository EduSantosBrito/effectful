//! Effect-style service layers for [`ServiceContext`](crate::ServiceContext).
//!
//! This is the v1 dependency-injection surface: a [`Layer`] is a lazy recipe that
//! builds one or more services, may fail with `E`, and may require upstream
//! services represented by `RIn`.

use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::context::{Service, ServiceContext};
use crate::kernel::{BoxFuture, Effect, box_future};
use crate::layer::graph::{LayerDiagnostic, LayerGraph, LayerNode};

type LayerBuildFn<E> =
  dyn Fn(ServiceContext) -> BoxFuture<'static, Result<ServiceContext, E>> + 'static;

/// Trait for types that declare layer requirements as stable service keys.
///
/// Implemented for `()` (no requirements), single [`Service`] types, and common
/// tuples of services so that `Layer::effect` and `Layer::succeed` can produce
/// canonical planner metadata automatically.
pub trait LayerRequirements {
  /// Returns the stable service keys this type requires from upstream layers.
  fn layer_requirements() -> Vec<&'static str>;
}

impl LayerRequirements for () {
  fn layer_requirements() -> Vec<&'static str> {
    Vec::new()
  }
}

impl<S: Service> LayerRequirements for S {
  fn layer_requirements() -> Vec<&'static str> {
    vec![S::NAME]
  }
}

impl<A: Service, B: Service> LayerRequirements for (A, B) {
  fn layer_requirements() -> Vec<&'static str> {
    vec![A::NAME, B::NAME]
  }
}

impl<A: Service, B: Service, C: Service> LayerRequirements for (A, B, C) {
  fn layer_requirements() -> Vec<&'static str> {
    vec![A::NAME, B::NAME, C::NAME]
  }
}

/// A lazy service graph node.
///
/// `ROut` is the service output type carried in signatures, `E` is the layer
/// error channel, and `RIn` documents the services this layer expects to find in
/// its input [`ServiceContext`]. The runtime representation is a typed service
/// table so application code does not need HList paths.
pub struct Layer<ROut, E, RIn = ()>
where
  ROut: 'static,
  E: 'static,
  RIn: 'static,
{
  name: &'static str,
  build: Rc<LayerBuildFn<E>>,
  nodes: Vec<LayerNode>,
  _pd: PhantomData<fn(RIn) -> ROut>,
}

impl<ROut, E, RIn> Clone for Layer<ROut, E, RIn>
where
  ROut: 'static,
  E: 'static,
  RIn: 'static,
{
  fn clone(&self) -> Self {
    Self {
      name: self.name,
      build: Rc::clone(&self.build),
      nodes: self.nodes.clone(),
      _pd: PhantomData,
    }
  }
}

impl<ROut, E, RIn> fmt::Debug for Layer<ROut, E, RIn>
where
  ROut: 'static,
  E: 'static,
  RIn: 'static,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Layer").field("name", &self.name).finish()
  }
}

impl<ROut, E, RIn> Layer<ROut, E, RIn>
where
  ROut: 'static,
  E: 'static,
  RIn: 'static,
{
  /// Create a layer from a low-level build function.
  #[inline]
  pub fn from_context<F>(name: &'static str, build: F) -> Self
  where
    F: Fn(ServiceContext) -> BoxFuture<'static, Result<ServiceContext, E>> + 'static,
  {
    Self {
      name,
      build: Rc::new(build),
      nodes: Vec::new(),
      _pd: PhantomData,
    }
  }

  /// Private constructor that carries planner metadata.
  #[inline]
  fn from_context_with_nodes<F>(name: &'static str, build: F, nodes: Vec<LayerNode>) -> Self
  where
    F: Fn(ServiceContext) -> BoxFuture<'static, Result<ServiceContext, E>> + 'static,
  {
    Self {
      name,
      build: Rc::new(build),
      nodes,
      _pd: PhantomData,
    }
  }

  /// Human-readable name used in diagnostics.
  #[inline]
  pub fn name(&self) -> &'static str {
    self.name
  }

  /// Build this layer from an existing service context.
  #[inline]
  pub fn build_from(&self, context: ServiceContext) -> Effect<ServiceContext, E, ()> {
    let layer = self.clone();
    Effect::new_async(move |_env: &mut ()| layer.build_with(context))
  }

  /// Build this layer from an empty service context.
  #[inline]
  pub fn build(&self) -> Effect<ServiceContext, E, ()> {
    self.build_from(ServiceContext::empty())
  }

  /// Convert this layer into an effect that reads its input context from `R`.
  #[inline]
  pub fn to_effect(&self) -> Effect<ServiceContext, E, ServiceContext> {
    let layer = self.clone();
    Effect::new_async(move |context: &mut ServiceContext| layer.build_with(context.clone()))
  }

  /// Map the error channel.
  #[inline]
  pub fn map_error<E2, F>(self, f: F) -> Layer<ROut, E2, RIn>
  where
    E2: 'static,
    F: Fn(E) -> E2 + Clone + 'static,
  {
    let name = self.name;
    let nodes = self.nodes.clone();
    Layer::from_context_with_nodes(
      name,
      move |context| {
        let layer = self.clone();
        let f = f.clone();
        box_future(async move { layer.build_with(context).await.map_err(f) })
      },
      nodes,
    )
  }

  /// Memoize this layer's first result for subsequent builds of the same value.
  ///
  /// Use this for expensive layers that do not depend on changing input
  /// contexts. The cached value is cloned into each caller.
  #[inline]
  pub fn memoized(self) -> Self
  where
    E: Clone,
  {
    let cache: Rc<RefCell<Option<Result<ServiceContext, E>>>> = Rc::new(RefCell::new(None));
    let name = self.name;
    let nodes = self.nodes.clone();
    Self::from_context_with_nodes(
      name,
      move |context| {
        let layer = self.clone();
        let cache = Rc::clone(&cache);
        if let Some(cached) = cache.borrow().clone() {
          return box_future(async move { cached });
        }
        box_future(async move {
          let result = layer.build_with(context).await;
          *cache.borrow_mut() = Some(result.clone());
          result
        })
      },
      nodes,
    )
  }

  /// Merge this layer with another layer that uses the same input context.
  #[inline]
  pub fn merge<ROut2>(self, that: Layer<ROut2, E, RIn>) -> Layer<(ROut, ROut2), E, RIn>
  where
    ROut2: 'static,
  {
    let mut nodes = self.nodes.clone();
    nodes.extend(that.nodes.clone());
    Layer::from_context_with_nodes(
      "Layer.merge",
      move |context| {
        let left = self.clone();
        let right = that.clone();
        box_future(async move {
          let left_context = left.build_with(context.clone()).await?;
          let right_context = right.build_with(context).await?;
          Ok(left_context.merge(right_context))
        })
      },
      nodes,
    )
  }

  /// Provide this layer's requirements with another layer.
  ///
  /// The output contains only this layer's services; provider services are used
  /// to build `self` and are then hidden, matching Effect's `Layer.provide`.
  #[inline]
  pub fn provide<RProvider>(self, provider: Layer<RProvider, E, ()>) -> Layer<ROut, E, ()>
  where
    RProvider: 'static,
  {
    let mut nodes = self.nodes.clone();
    nodes.extend(provider.nodes.clone());
    Layer::from_context_with_nodes(
      "Layer.provide",
      move |context| {
        let layer = self.clone();
        let provider = provider.clone();
        box_future(async move {
          let provided = provider.build_with(context.clone()).await?;
          let full_context = context.merge(provided);
          layer.build_with(full_context).await
        })
      },
      nodes,
    )
  }

  /// Provide this layer's requirements and keep provider services in the output.
  #[inline]
  pub fn provide_merge<RProvider>(
    self,
    provider: Layer<RProvider, E, ()>,
  ) -> Layer<(ROut, RProvider), E, ()>
  where
    RProvider: 'static,
  {
    let mut nodes = self.nodes.clone();
    nodes.extend(provider.nodes.clone());
    Layer::from_context_with_nodes(
      "Layer.provide_merge",
      move |context| {
        let layer = self.clone();
        let provider = provider.clone();
        box_future(async move {
          let provided = provider.build_with(context.clone()).await?;
          let output = layer.build_with(context.merge(provided.clone())).await?;
          Ok(provided.merge(output))
        })
      },
      nodes,
    )
  }

  /// Returns planner diagnostics for this layer's dependency graph.
  ///
  /// Diagnostics are empty when the layer can be built without additional
  /// providers; otherwise they contain a single [`LayerDiagnostic`] from the
  /// canonical planner with stable service keys.
  #[inline]
  pub fn diagnostics(&self) -> Vec<LayerDiagnostic> {
    self.diagnostics_with_context(&ServiceContext::empty())
  }

  /// Returns planner diagnostics, treating services in `context` as available.
  #[inline]
  pub fn diagnostics_with_context(&self, context: &ServiceContext) -> Vec<LayerDiagnostic> {
    let mut nodes = self.nodes.clone();
    for name in context.service_names() {
      nodes.push(LayerNode::new(name, Vec::<&str>::new(), [name]));
    }
    LayerGraph::new(nodes).diagnostics()
  }

  pub(crate) fn build_with(
    &self,
    context: ServiceContext,
  ) -> BoxFuture<'static, Result<ServiceContext, E>> {
    (self.build)(context)
  }
}

impl<S, E> Layer<S, E, ()>
where
  S: Service,
  E: 'static,
{
  /// Construct an infallible layer from a concrete service value.
  #[inline]
  pub fn succeed(service: S) -> Self {
    let nodes = vec![LayerNode::new(S::NAME, Vec::<&str>::new(), [S::NAME])];
    Self::from_context_with_nodes(
      S::NAME,
      move |_context| {
        let service = service.clone();
        box_future(async move { Ok(ServiceContext::empty().add(service)) })
      },
      nodes,
    )
  }
}

impl<S, E, RIn> Layer<S, E, RIn>
where
  S: Service,
  E: 'static,
  RIn: LayerRequirements + 'static,
{
  /// Construct a layer from an effectful service constructor.
  ///
  /// The constructor receives dependencies through [`ServiceContext`], so it can
  /// use [`Effect::service`](crate::Effect::service) or `MyService::use_sync`.
  #[inline]
  pub fn effect<F>(name: &'static str, make: F) -> Self
  where
    F: Fn() -> Effect<S, E, ServiceContext> + 'static,
  {
    let nodes = vec![LayerNode::new(name, RIn::layer_requirements(), [S::NAME])];
    Self::from_context_with_nodes(
      name,
      move |mut context| {
        let effect = make();
        box_future(async move {
          let service = effect.run(&mut context).await?;
          Ok(ServiceContext::empty().add(service))
        })
      },
      nodes,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{ContextService, MissingService, run_blocking};

  #[derive(Clone, Debug, PartialEq, effectful::Service)]
  struct Config {
    url: String,
  }

  #[derive(Clone, Debug, PartialEq, effectful::Service)]
  struct Database {
    url: String,
  }

  #[derive(Clone, Debug, PartialEq, effectful::Service)]
  struct Logger {
    level: String,
  }

  #[test]
  fn succeed_builds_service_context() {
    let layer = Layer::<Config, MissingService>::succeed(Config {
      url: "postgres://".to_string(),
    });

    let context = run_blocking(layer.build(), ()).expect("layer should build");
    assert_eq!(
      context.get::<Config>(),
      Some(&Config {
        url: "postgres://".to_string()
      })
    );
  }

  #[test]
  fn provide_satisfies_layer_dependencies() {
    let config = Layer::<Config, MissingService>::succeed(Config {
      url: "postgres://".to_string(),
    });
    let database = Layer::<Database, MissingService, Config>::effect("Database", || {
      Config::use_sync(|config| Database { url: config.url })
    });
    let program: Effect<String, MissingService, ServiceContext> =
      Database::use_sync(|database| database.url);

    let result = run_blocking(program.provide(database.provide(config)), ());

    assert_eq!(result, Ok("postgres://".to_string()));
  }

  #[test]
  fn merge_keeps_services_from_both_layers() {
    let config = Layer::<Config, MissingService>::succeed(Config {
      url: "postgres://".to_string(),
    });
    let logger = Layer::<Logger, MissingService>::succeed(Logger {
      level: "debug".to_string(),
    });

    let context = run_blocking(config.merge(logger).build(), ()).expect("layer should build");

    assert!(context.contains::<Config>());
    assert!(context.contains::<Logger>());
  }

  #[test]
  fn missing_dependency_fails_layer_build() {
    let database = Layer::<Database, MissingService, Config>::effect("Database", || {
      Config::use_sync(|config| Database { url: config.url })
    });

    let result = run_blocking(database.build(), ());

    assert!(matches!(
      result,
      Err(MissingService { name }) if name == Config::NAME
    ));
  }

  mod diagnostics {
    use super::*;

    #[test]
    fn diagnostics_when_effect_layer_missing_provider_returns_stable_service_key() {
      let database = Layer::<Database, MissingService, Config>::effect("Database", || {
        Config::use_sync(|config| Database { url: config.url })
      });

      let diagnostics = database.diagnostics();

      assert_eq!(diagnostics.len(), 1);
      assert_eq!(diagnostics[0].code, "missing-provider");
      assert!(diagnostics[0].message.contains(Config::NAME));
    }

    #[test]
    fn diagnostics_when_provider_supplies_requirement_returns_empty() {
      let config = Layer::<Config, MissingService>::succeed(Config {
        url: "postgres://".to_string(),
      });
      let database = Layer::<Database, MissingService, Config>::effect("Database", || {
        Config::use_sync(|config| Database { url: config.url })
      });

      let diagnostics = database.provide(config).diagnostics();

      assert!(diagnostics.is_empty());
    }

    #[test]
    fn diagnostics_with_context_when_context_supplies_requirement_returns_empty() {
      let database = Layer::<Database, MissingService, Config>::effect("Database", || {
        Config::use_sync(|config| Database { url: config.url })
      });
      let ctx = ServiceContext::empty().add(Config {
        url: "postgres://".to_string(),
      });

      let diagnostics = database.diagnostics_with_context(&ctx);

      assert!(diagnostics.is_empty());
    }

    #[test]
    fn diagnostics_merge_preserves_existing_build_behavior() {
      let config = Layer::<Config, MissingService>::succeed(Config {
        url: "postgres://".to_string(),
      });
      let logger = Layer::<Logger, MissingService>::succeed(Logger {
        level: "debug".to_string(),
      });

      let context = run_blocking(config.merge(logger).build(), ()).expect("layer should build");
      assert!(context.contains::<Config>());
      assert!(context.contains::<Logger>());
    }

    #[test]
    fn diagnostics_provide_preserves_existing_build_behavior() {
      let config = Layer::<Config, MissingService>::succeed(Config {
        url: "postgres://".to_string(),
      });
      let database = Layer::<Database, MissingService, Config>::effect("Database", || {
        Config::use_sync(|config| Database { url: config.url })
      });
      let program: Effect<String, MissingService, ServiceContext> =
        Database::use_sync(|database| database.url);

      let result = run_blocking(program.provide(database.provide(config)), ());
      assert_eq!(result, Ok("postgres://".to_string()));
    }

    #[test]
    fn diagnostics_provide_merge_preserves_existing_build_behavior() {
      let config = Layer::<Config, MissingService>::succeed(Config {
        url: "postgres://".to_string(),
      });
      let database = Layer::<Database, MissingService, Config>::effect("Database", || {
        Config::use_sync(|config| Database { url: config.url })
      });

      let merged = database.provide_merge(config);
      let diagnostics = merged.diagnostics();
      assert!(diagnostics.is_empty());
    }

    #[test]
    fn diagnostics_service_layer_preserves_existing_build_behavior() {
      let config = Config {
        url: "postgres://".to_string(),
      };
      let layer = config.layer();

      let context = run_blocking(layer.build(), ()).expect("layer should build");
      assert!(context.contains::<Config>());
    }

    #[test]
    fn diagnostics_succeed_preserves_existing_build_behavior() {
      let layer = Layer::<Config, MissingService>::succeed(Config {
        url: "postgres://".to_string(),
      });

      let context = run_blocking(layer.build(), ()).expect("layer should build");
      assert!(context.contains::<Config>());
    }
  }
}
