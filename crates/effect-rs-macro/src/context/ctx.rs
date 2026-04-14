//! `ctx!` macro.

/// Runtime context builder with service cells.
///
/// ```ignore
/// let r = effect::ctx!(
///   LogKey => Logger::default(),
///   DbKey => db,
/// );
/// ```
#[macro_export]
macro_rules! ctx {
  () => {
    ::effect::Context::new(::effect::Nil)
  };
  ($k:ty => $v:expr $(, $rk:ty => $rv:expr )* $(,)?) => {
    ::effect::Context::new($crate::ctx!(@list $k => $v $(, $rk => $rv )*))
  };
  (@list $k:ty => $v:expr) => {
    ::effect::Cons(::effect::service::<$k, _>($v), ::effect::Nil)
  };
  (@list $k:ty => $v:expr, $($rest:tt)+) => {
    ::effect::Cons(::effect::service::<$k, _>($v), $crate::ctx!(@list $($rest)+))
  };
}
