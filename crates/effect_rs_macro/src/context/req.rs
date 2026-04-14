//! `req!` macro.

/// Type-level required service stack (Rust equivalent of `R` union requirements).
///
/// ```ignore
/// type R = effect_rs::req!(DbKey: DbClient | LogKey: Logger);
/// ```
#[macro_export]
macro_rules! req {
  (@tail $k:ty : $v:ty) => {
    ::effect_rs::Cons<::effect_rs::Service<$k, $v>, ::effect_rs::Nil>
  };
  (@tail $k:ty : $v:ty | $($rest:tt)+) => {
    ::effect_rs::Cons<::effect_rs::Service<$k, $v>, $crate::req!(@tail $($rest)+)>
  };
  () => {
    ::effect_rs::Context<::effect_rs::Nil>
  };
  ($($pairs:tt)+) => {
    ::effect_rs::Context<$crate::req!(@tail $($pairs)+)>
  };
}
