//! `ctx!` macro.

/// Runtime context builder with service cells.
///
/// ```ignore
/// let r = effectful::ctx!(
///   LogKey => Logger::default(),
///   DbKey => db,
/// );
/// ```
#[macro_export]
macro_rules! ctx {
  () => {
    ::effectful::Context::new(::effectful::Nil)
  };
  ($k:ty => $v:expr $(, $rk:ty => $rv:expr )* $(,)?) => {
    ::effectful::Context::new($crate::ctx!(@list $k => $v $(, $rk => $rv )*))
  };
  (@list $k:ty => $v:expr) => {
    ::effectful::Cons(::effectful::layer::service::service::<$k, _>($v), ::effectful::Nil)
  };
  (@list $k:ty => $v:expr, $($rest:tt)+) => {
    ::effectful::Cons(::effectful::layer::service::service::<$k, _>($v), $crate::ctx!(@list $($rest)+))
  };
}
