//! `req!` macro.

/// Type-level required service stack (Rust equivalent of `R` union requirements).
///
/// ```ignore
/// type R = effect::req!(DbKey: DbClient | LogKey: Logger);
/// ```
#[macro_export]
macro_rules! req {
  (@tail $k:ty : $v:ty) => {
    ::effect::Cons<::effect::Service<$k, $v>, ::effect::Nil>
  };
  (@tail $k:ty : $v:ty | $($rest:tt)+) => {
    ::effect::Cons<::effect::Service<$k, $v>, $crate::req!(@tail $($rest)+)>
  };
  () => {
    ::effect::Context<::effect::Nil>
  };
  ($($pairs:tt)+) => {
    ::effect::Context<$crate::req!(@tail $($pairs)+)>
  };
}
