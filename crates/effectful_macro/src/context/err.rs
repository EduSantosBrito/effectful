//! `err!` macro.

/// Type-level error sum ("union") using nested [`Or`](::effectful::Or).
///
/// ```ignore
/// type E = effectful::err!(IoErr | DecodeErr);
/// ```
#[macro_export]
macro_rules! err {
  () => { () };
  ($e:ty) => { $e };
  ($e:ty | $($rest:tt)+) => {
    ::effectful::Or<$e, $crate::err!($($rest)+)>
  };
}
