//! `#[derive(Service)]` macro for Thesis 1: Service Declaration & Identity.
//!
//! This derive makes a struct self-describing — the struct IS both the key AND the value.
//! Enables ergonomic service access without separate key declarations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Implementation of `#[derive(Service)]`.
///
/// Called from `lib.rs` where the proc_macro attribute must reside.
pub fn derive_service(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;

  // Generate impl block with helper methods
  let expanded = quote! {
    // The type itself is usable as a service key via Tagged<Self, Self>
    // No additional marker traits needed — the type itself serves as identity

    #[automatically_derived]
    impl #name {
      /// Wrap this service value in a context as the head cell.
      ///
      /// Shorthand for `Context::new(Cons(Tagged::<Self, _>::new(self), Nil))`.
      #[inline]
      pub fn to_context(self) -> ::effectful::Context<
        ::effectful::Cons<
          ::effectful::Tagged<Self, Self>,
          ::effectful::Nil
        >
      > {
        ::effectful::Context::new(
          ::effectful::Cons(
            ::effectful::Tagged::<Self, _>::new(self),
            ::effectful::Nil
          )
        )
      }

      /// Wrap this service value in a Cons cell for context composition.
      #[inline]
      pub fn as_tagged(self) -> ::effectful::Tagged<Self, Self> {
        ::effectful::Tagged::<Self, _>::new(self)
      }
    }
  };

  TokenStream::from(expanded)
}
