//! `ctx!` macro.

/// Runtime context builder with service cells.
///
/// ```ignore
/// let r = effect_rs::ctx!(
///   LogKey => Logger::default(),
///   DbKey => db,
/// );
/// ```
#[macro_export]
macro_rules! ctx {
  () => {
    ::effect_rs::Context::new(::effect_rs::Nil)
  };
  ($k:ty => $v:expr $(, $rk:ty => $rv:expr )* $(,)?) => {
    ::effect_rs::Context::new($crate::ctx!(@list $k => $v $(, $rk => $rv )*))
  };
  (@list $k:ty => $v:expr) => {
    ::effect_rs::Cons(::effect_rs::service::<$k, _>($v), ::effect_rs::Nil)
  };
  (@list $k:ty => $v:expr, $($rest:tt)+) => {
    ::effect_rs::Cons(::effect_rs::service::<$k, _>($v), $crate::ctx!(@list $($rest)+))
  };
}
