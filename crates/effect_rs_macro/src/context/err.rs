//! `err!` macro.

/// Type-level error sum ("union") using nested [`Or`](::effect_rs::Or).
///
/// ```ignore
/// type E = effect_rs::err!(IoErr | DecodeErr);
/// ```
#[macro_export]
macro_rules! err {
  () => { () };
  ($e:ty) => { $e };
  ($e:ty | $($rest:tt)+) => {
    ::effect_rs::Or<$e, $crate::err!($($rest)+)>
  };
}
