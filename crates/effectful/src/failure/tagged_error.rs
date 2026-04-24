//! **Tagged Error Handling** — pattern matching on error variants by tag.
//!
//! Provides `TaggedError` trait for error enums and `catch_tag`/`catch_tags`
//! combinators for handling specific error variants.
//!
//! ## Example
//!
//! ```ignore
//! #[derive(Debug, Clone)]
//! enum AppError {
//!     ValidationError { field: String },
//!     NetworkError { status: u16 },
//! }
//!
//! impl TaggedError for AppError {
//!     fn tag(&self) -> &'static str {
//!         match self {
//!             AppError::ValidationError { .. } => "validation",
//!             AppError::NetworkError { .. } => "network",
//!         }
//!     }
//! }
//!
//! let result = catch_tags(effect, |tags| {
//!     tags.on("validation", |e| succeed(fallback))
//!         .on("network", |e| retry(schedule))
//! });
//! ```

use crate::kernel::Effect;
use core::fmt::Debug;
use core::marker::PhantomData;

/// An error type that can be tagged for pattern matching.
///
/// Implement this trait on error enums to enable `catch_tag`/`catch_tags`.
pub trait TaggedError: Debug + Clone + 'static {
  /// Returns a string tag identifying the error variant.
  ///
  /// Tags should be stable and unique per variant.
  fn tag(&self) -> &'static str;
}

/// Catch a specific error variant by tag.
///
/// If the effect fails with an error whose `tag()` matches the handler's expected tag,
/// the handler is invoked. Otherwise, the original error is propagated.
///
/// # Type Parameters
///
/// - `A`: Success type
/// - `E`: Error type (must implement `TaggedError`)
/// - `R`: Environment type
/// - `F`: Handler function type
/// - `E2`: Handler's error type
pub fn catch_tag<A, E, R, F, E2>(effect: Effect<A, E, R>, handler: F) -> Effect<A, E2, R>
where
  E: TaggedError,
  R: 'static,
  F: FnOnce(E) -> Effect<A, E2, R> + 'static,
  A: 'static,
  E2: 'static,
{
  effect.catch(handler)
}

/// Builder for handling multiple error tags.
///
/// Created by [`catch_tags`]. Use `.on(tag, handler)` to register handlers.
pub struct TagHandler<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  handlers: Vec<(
    &'static str,
    Box<dyn FnOnce(E) -> Effect<A, E, R> + 'static>,
  )>,
  _pd: PhantomData<(A, E, R)>,
}

impl<A, E, R> TagHandler<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  /// Register a handler for errors with the given tag.
  #[inline]
  pub fn on<F>(mut self, tag: &'static str, handler: F) -> Self
  where
    F: FnOnce(E) -> Effect<A, E, R> + 'static,
  {
    self.handlers.push((tag, Box::new(handler)));
    self
  }
}

/// Catch multiple error variants using a builder pattern.
///
/// # Example
///
/// ```ignore
/// let result = catch_tags(effect, |tags| {
///     tags.on("validation", |e| Effect::succeed(fallback))
///         .on("network", |e| Effect::retry(schedule))
/// });
/// ```
pub fn catch_tags<A, E, R, F>(effect: Effect<A, E, R>, builder: F) -> Effect<A, E, R>
where
  E: TaggedError,
  R: Clone + 'static,
  F: FnOnce(TagHandler<A, E, R>) -> TagHandler<A, E, R> + 'static,
  A: 'static,
{
  let handler = builder(TagHandler {
    handlers: Vec::new(),
    _pd: PhantomData,
  });

  effect.catch(move |e: E| {
    let tag = e.tag();
    for (handler_tag, handler_fn) in handler.handlers {
      if handler_tag == tag {
        return handler_fn(e);
      }
    }
    crate::kernel::fail(e)
  })
}

/// Convert recoverable errors into panics (defects).
///
/// Use when an error represents a programming mistake that should never happen.
/// The error must implement `Debug` for the panic message.
///
/// # Example
///
/// ```ignore
/// let result = or_die(effect); // panics if effect fails
/// ```
pub fn or_die<A, E, R>(effect: Effect<A, E, R>) -> Effect<A, (), R>
where
  E: Debug + 'static,
  R: 'static,
  A: 'static,
{
  effect.map_error(|e| {
    panic!("Effect failed: {:?}", e);
  })
}
