//! [`Get`] / [`GetMut`] — type-level lookup (`Stratum 3.5`).

use super::hlist::{Cons, Nil};
use super::path::{Here, There};
use super::tagged::Tagged;

/// Immutable borrow of the value at `Path` for tag `K`.
pub trait Get<K: ?Sized, Path = Here> {
  /// Type of the resolved service at `Path`.
  type Target: ?Sized;
  /// Borrow the value registered for `K` along `Path`.
  fn get(&self) -> &Self::Target;
}

/// Mutable borrow of the value at `Path` for tag `K`.
pub trait GetMut<K: ?Sized, Path = Here> {
  /// Type of the resolved service at `Path`.
  type Target: ?Sized;
  /// Mutably borrow the value registered for `K` along `Path`.
  fn get_mut(&mut self) -> &mut Self::Target;
}

impl<K: ?Sized, V, Tail> Get<K, Here> for Cons<Tagged<K, V>, Tail> {
  type Target = V;
  #[inline]
  fn get(&self) -> &V {
    &self.0.value
  }
}

impl<K: ?Sized, V, Tail> GetMut<K, Here> for Cons<Tagged<K, V>, Tail> {
  type Target = V;
  #[inline]
  fn get_mut(&mut self) -> &mut V {
    &mut self.0.value
  }
}

impl<Head, Tail, K: ?Sized, P> Get<K, There<P>> for Cons<Head, Tail>
where
  Tail: Get<K, P>,
{
  type Target = <Tail as Get<K, P>>::Target;
  #[inline]
  fn get(&self) -> &Self::Target {
    self.1.get()
  }
}

impl<Head, Tail, K: ?Sized, P> GetMut<K, There<P>> for Cons<Head, Tail>
where
  Tail: GetMut<K, P>,
{
  type Target = <Tail as GetMut<K, P>>::Target;
  #[inline]
  fn get_mut(&mut self) -> &mut Self::Target {
    self.1.get_mut()
  }
}

// ── Optional Head Lookup (Thesis 3) ───────────────────────────────────────

/// Optionally get a service if it's at the head of the list.
///
/// Returns [`Some`] when `K` matches the head tag, [`None`] otherwise.
/// This trait intentionally only checks the head to avoid overlap issues
/// in stable Rust. Use with [`provide_service`](crate::layer::provide_service)
/// which places the service at the head.
pub trait GetOption<K: ?Sized> {
  /// Type of the resolved service.
  type Target: ?Sized;
  /// Optionally get the value at the head for `K`.
  fn get_option(&self) -> Option<&Self::Target>;
}

impl<K: ?Sized, V, Tail> GetOption<K> for Cons<Tagged<K, V>, Tail> {
  type Target = V;
  #[inline]
  fn get_option(&self) -> Option<&V> {
    Some(&self.0.value)
  }
}

impl<K: ?Sized> GetOption<K> for Nil {
  type Target = K;
  #[inline]
  fn get_option(&self) -> Option<&K> {
    None
  }
}
