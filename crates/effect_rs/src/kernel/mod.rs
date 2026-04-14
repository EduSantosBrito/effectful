//! Stratum 2 — **Core Effect**: [`Effect`], [`Thunk`](thunk), [`Result`](result), [`Reader`](reader).
//!
//! The [`effect`] submodule holds [`Effect`](effect_rs::Effect), [`BoxFuture`](effect_rs::BoxFuture), and
//! [`IntoBind`](effect_rs::IntoBind). [`thunk`], [`result`], and [`reader`] are the supporting kernel
//! types and combinators described in `SPEC.md`.

pub mod effect;
pub mod reader;
pub mod result;
pub mod thunk;

pub use self::effect::{
  BoxFuture, Effect, IntoBind, acquire_release, box_future, fail, from_async, into_bind, pure,
  scope_with, scoped, succeed, unwrap_infallible,
};
