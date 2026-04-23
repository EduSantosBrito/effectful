//! `req!` macro.

/// Type-level required service stack (Rust equivalent of `R` union requirements).
///
/// ```ignore
/// type R = effectful::req!(DbKey: DbClient | LogKey: Logger);
/// ```
#[macro_export]
macro_rules! req {
  (@tail $k:ty : $v:ty) => {
    ::effectful::Cons<::effectful::layer::service::Service<$k, $v>, ::effectful::Nil>
  };
  (@tail $k:ty : $v:ty | $($rest:tt)+) => {
    ::effectful::Cons<::effectful::layer::service::Service<$k, $v>, $crate::req!(@tail $($rest)+)>
  };
  () => {
    ::effectful::Context<::effectful::Nil>
  };
  ($($pairs:tt)+) => {
    ::effectful::Context<$crate::req!(@tail $($pairs)+)>
  };
}
