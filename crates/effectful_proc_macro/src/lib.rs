//! Procedural macros for the workspace `effect` crate.
//!
//! Doc links cannot use the `effectful::…` prefix here: this crate defines an `effect` function, which
//! shadows the `effect` crate name in rustdoc link resolution.
#![allow(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

mod effect_data;
mod effect_tagged;
mod expand;
mod parse;
mod service_derive;
mod tagged_error_derive;
mod transform;

use proc_macro::TokenStream;

/// Derive macro: structural [`PartialEq`], [`Eq`], and [`Hash`] for Effect.ts-style data types.
///
/// Types implementing these impls automatically satisfy [`effectful::data::EffectData`] via the
/// blanket implementation in the `effect` crate.
#[proc_macro_derive(EffectData)]
pub fn derive_effect_data(input: TokenStream) -> TokenStream {
  effect_data::derive_effect_data(input)
}

/// Injects `pub _tag: &'static str`, an [`effectful::match_::HasTag`] impl, and
/// `EFFECT_TAGGED_TAG` on the struct (see generated inherent associated const).
///
/// Only supports structs with **named fields**. Place **above** `#[derive(EffectData, …)]`.
#[proc_macro_attribute]
pub fn effect_tagged(attr: TokenStream, item: TokenStream) -> TokenStream {
  effect_tagged::expand(attr, item)
}

/// Derive macro: makes a struct act as its own service key.
///
/// Enables self-describing services where the type IS both key and value.
/// The struct can be used directly with `Effect::service::<Self>()` without
/// a separate key declaration.
///
/// # Example
///
/// ```ignore
/// use effectful::Service;
///
/// #[derive(Service, Clone)]
/// struct Database {
///     url: String,
/// }
///
/// let env = Database { url: "...".into() }.to_context();
/// let db: &Database = env.get::<Database>();
/// ```
#[proc_macro_derive(Service, attributes(service))]
pub fn derive_service(input: TokenStream) -> TokenStream {
  service_derive::derive_service(input)
}

/// Derive macro: implements [`TaggedError`] and generates tag constants.
///
/// Use `#[tag("...")]` on each variant to specify the tag string. Defaults to
/// lowercase variant name.
///
/// Generates associated constants in SCREAMING_SNAKE_CASE for IDE autocomplete.
///
/// # Example
///
/// ```ignore
/// #[derive(TaggedError, Debug, Clone)]
/// enum AppError {
///     #[tag("validation")]
///     ValidationError { field: String },
///     #[tag("network")]
///     NetworkError { status: u16 },
/// }
///
/// // Use with catch_tags:
/// tags.on(AppError::VALIDATION, |e| { ... })
///     .on(AppError::NETWORK, |e| { ... })
/// ```
#[proc_macro_derive(TaggedError, attributes(tag))]
pub fn derive_tagged_error(input: TokenStream) -> TokenStream {
  tagged_error_derive::derive_tagged_error(input)
}

/// Procedural do-notation macro for [`effectful::Effect`].
///
/// See the `effect` crate documentation for usage.
#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
  let input = proc_macro2::TokenStream::from(input);
  let kind = match parse::parse_effect_input(input) {
    Ok(k) => k,
    Err(e) => return e.to_compile_error().into(),
  };
  expand::expand(kind).into()
}
mod test_yield;
mod test_bind_star;
