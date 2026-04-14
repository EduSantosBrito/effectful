//! `err!` macro.

/// Type-level error sum ("union") using nested [`Or`](::effect::Or).
///
/// ```ignore
/// type E = effect::err!(IoErr | DecodeErr);
/// ```
#[macro_export]
macro_rules! err {
  () => { () };
  ($e:ty) => { $e };
  ($e:ty | $($rest:tt)+) => {
    ::effect::Or<$e, $crate::err!($($rest)+)>
  };
}
