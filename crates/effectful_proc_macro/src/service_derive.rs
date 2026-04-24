//! `#[derive(Service)]` macro for v1 service declaration.
//!
//! This derive makes a struct self-describing: the struct is both the service key
//! and the service value, matching Effect's `Context.Service` model in a
//! Rust-native way.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, LitStr, parse_macro_input};

/// Extract the service name from `#[service(name = "...")]` attributes.
fn extract_service_name(attrs: &[Attribute], struct_name: &str) -> String {
  for attr in attrs {
    if attr.path().is_ident("service") {
      let mut service_name = None;
      let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
          let value = meta.value()?;
          let lit: LitStr = value.parse()?;
          service_name = Some(lit.value());
        }
        Ok(())
      });
      if let Some(service_name) = service_name {
        return service_name;
      }
    }
  }
  // Default to struct name if no attribute provided
  struct_name.to_string()
}

/// Implementation of `#[derive(Service)]`.
///
/// Called from `lib.rs` where the proc_macro attribute must reside.
pub fn derive_service(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;
  let name_str = name.to_string();

  // Parse #[service(name = "...")] attribute
  let service_name = extract_service_name(&input.attrs, &name_str);

  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let expanded = quote! {
    #[automatically_derived]
    impl #impl_generics ::effectful::context::Service for #name #ty_generics #where_clause {
      const NAME: &'static str = #service_name;
    }

    #[automatically_derived]
    impl #impl_generics #name #ty_generics #where_clause {
      /// Human-readable name for this service type.
      ///
      /// Defaults to the struct name unless overridden via `#[service(name = "...")]`.
      pub const NAME: &'static str = #service_name;

      /// Wrap this service value in a service context.
      #[inline]
      pub fn to_context(self) -> ::effectful::ServiceContext {
        <Self as ::effectful::context::Service>::to_context(self)
      }

      /// Build a layer that provides this concrete service value.
      #[inline]
      pub fn layer(self) -> ::effectful::Layer<Self, ::effectful::Never, ()> {
        <Self as ::effectful::context::Service>::layer(self)
      }
    }
  };

  TokenStream::from(expanded)
}
