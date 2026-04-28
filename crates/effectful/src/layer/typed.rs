//! **Typed Layer Composition** — runtime dependency tracking with typed API.
//!
//! Mirrors effect-smol's `Layer<ROut, E, RIn>` but uses runtime validation
//! instead of compile-time proofs (stable Rust limitation).
//!
//! ## Design
//!
//! - `TypedLayer` carries `provides: HashSet<String>` and `requires: HashSet<String>`
//! - `build()` returns `Result<O, E>` like standard `Layer`
//! - `merge_all()` combines layers, unioning provides/requires
//! - `memoized()` caches build output
//!
//! ## Future (nightly)
//!
//! When `generic_const_exprs` stabilizes, `provides`/`requires` can become
//! associated type-level sets with compile-time checking.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::ServiceContext;

/// A layer with named dependency tracking.
///
/// `TypedLayer` is a **recipe** for constructing a value, annotated with:
/// - **`provides`**: what service(s) this layer produces
/// - **`requires`**: what service(s) must exist before building
///
/// # Example
///
/// ```ignore
/// let db_layer = TypedLayer::from_fn(|| Ok(Database::new()))
///     .providing("Database")
///     .requiring("Config");
/// ```
pub struct TypedLayer<O, E> {
  build_fn: Box<dyn Fn() -> Result<O, E> + Send + Sync>,
  provides: HashSet<String>,
  requires: HashSet<String>,
}

impl<O, E> TypedLayer<O, E> {
  /// Create a layer from a fallible constructor.
  #[inline]
  pub fn from_fn<F>(f: F) -> Self
  where
    F: Fn() -> Result<O, E> + Send + Sync + 'static,
  {
    Self {
      build_fn: Box::new(f),
      provides: HashSet::new(),
      requires: HashSet::new(),
    }
  }

  /// Create a layer from an infallible constructor.
  #[inline]
  pub fn from_fn_infallible<F>(f: F) -> Self
  where
    F: Fn() -> O + Send + Sync + 'static,
    E: Default,
  {
    Self {
      build_fn: Box::new(move || Ok(f())),
      provides: HashSet::new(),
      requires: HashSet::new(),
    }
  }

  /// Declare what this layer provides.
  #[inline]
  pub fn providing(mut self, name: &str) -> Self {
    self.provides.insert(name.to_string());
    self
  }

  /// Declare what this layer requires.
  #[inline]
  pub fn requiring(mut self, name: &str) -> Self {
    self.requires.insert(name.to_string());
    self
  }

  /// Create with explicit requirements list.
  #[inline]
  pub fn with_requirements<F, I>(f: F, requires: I) -> Self
  where
    F: Fn() -> Result<O, E> + Send + Sync + 'static,
    I: IntoIterator,
    I::Item: Into<String>,
  {
    Self {
      build_fn: Box::new(f),
      provides: HashSet::new(),
      requires: requires.into_iter().map(Into::into).collect(),
    }
  }

  /// Set of services this layer provides.
  #[inline]
  pub fn provides(&self) -> &HashSet<String> {
    &self.provides
  }

  /// Set of services this layer requires.
  #[inline]
  pub fn requires(&self) -> &HashSet<String> {
    &self.requires
  }

  /// Build the layer, returning the output or error.
  #[inline]
  pub fn build(&self) -> Result<O, E> {
    (self.build_fn)()
  }

  /// Build with dependency validation against a [`ServiceContext`].
  ///
  /// Returns `Err` if any required service is missing from the context.
  #[inline]
  pub fn build_with_dependencies(&self, ctx: &ServiceContext) -> Result<O, E>
  where
    E: From<LayerError>,
  {
    let available: HashSet<String> = ctx
      .service_names()
      .into_iter()
      .map(|s| s.to_string())
      .collect();
    let mut missing: Vec<String> = self
      .requires
      .iter()
      .filter(|r| !available.contains(*r))
      .cloned()
      .collect();
    if !missing.is_empty() {
      missing.sort_unstable();
      return Err(LayerError::MissingDependencies { missing }.into());
    }
    self.build()
  }

  /// Merge multiple layers into one.
  ///
  /// - `provides` = union of all layer provides
  /// - `requires` = union of all layer requires minus union of provides
  /// - Build order = sequential (first to last)
  pub fn merge_all(layers: Vec<Self>) -> MergedLayer<O, E>
  where
    O: 'static,
    E: 'static,
  {
    let mut provides = HashSet::new();
    let mut requires = HashSet::new();

    for layer in &layers {
      provides.extend(layer.provides.iter().cloned());
      requires.extend(layer.requires.iter().cloned());
    }

    // Satisfied requirements = requires - provides
    requires.retain(|r| !provides.contains(r));

    MergedLayer {
      layers,
      provides,
      requires,
    }
  }
}

/// Error type for layer operations.
#[derive(Clone, Debug, PartialEq)]
pub enum LayerError {
  /// Required dependencies are missing.
  MissingDependencies {
    /// Names of missing services.
    missing: Vec<String>,
  },
}

impl std::fmt::Display for LayerError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      LayerError::MissingDependencies { missing } => {
        write!(f, "missing dependencies: {}", missing.join(", "))
      }
    }
  }
}

impl std::error::Error for LayerError {}

/// A merged layer combining multiple sub-layers.
pub struct MergedLayer<O, E> {
  #[allow(dead_code)]
  layers: Vec<TypedLayer<O, E>>,
  provides: HashSet<String>,
  requires: HashSet<String>,
}

impl<O, E> MergedLayer<O, E> {
  /// Set of services this merged layer provides.
  pub fn provides(&self) -> &HashSet<String> {
    &self.provides
  }

  /// Set of services this merged layer still requires.
  pub fn requires(&self) -> &HashSet<String> {
    &self.requires
  }
}

/// Extension combinators for `TypedLayer`.
pub trait TypedLayerExt<O, E> {
  /// Wrap this layer with memoization.
  ///
  /// The build function is called at most once; subsequent calls return the cached result.
  fn memoized(self) -> MemoizedLayer<O, E>;
}

impl<O, E> TypedLayerExt<O, E> for TypedLayer<O, E>
where
  O: Clone + Send + Sync + 'static,
  E: Clone + Send + Sync + 'static,
{
  fn memoized(self) -> MemoizedLayer<O, E> {
    MemoizedLayer {
      inner: self,
      cached: Arc::new(Mutex::new(None)),
    }
  }
}

/// A memoized layer that caches its build result.
pub struct MemoizedLayer<O, E> {
  inner: TypedLayer<O, E>,
  cached: Arc<Mutex<Option<Result<O, E>>>>,
}

impl<O, E> MemoizedLayer<O, E>
where
  O: Clone,
  E: Clone,
{
  /// Build the layer, using cached result if available.
  pub fn build(&self) -> Result<O, E> {
    if let Some(cached) = self.cached.lock().unwrap().clone() {
      return cached;
    }
    let result = self.inner.build();
    *self.cached.lock().unwrap() = Some(result.clone());
    result
  }

  /// Access the underlying layer's provides.
  pub fn provides(&self) -> &HashSet<String> {
    self.inner.provides()
  }

  /// Access the underlying layer's requires.
  pub fn requires(&self) -> &HashSet<String> {
    self.inner.requires()
  }
}
