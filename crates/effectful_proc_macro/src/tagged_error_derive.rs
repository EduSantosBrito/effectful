//! `#[derive(TaggedError)]` macro for tagged error pattern matching.
//!
//! Generates `TaggedError` impl and associated constants for variant tags.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Lit, Meta, MetaNameValue};

/// Extract the tag string from `#[tag("...")]` attribute.
fn extract_tag(attrs: &[Attribute], variant_name: &str) -> String {
  for attr in attrs {
    if attr.path().is_ident("tag") {
      // Try parsing as a simple string literal: #[tag("validation")]
      if let Ok(lit) = attr.parse_args::<Lit>() {
        if let Lit::Str(lit_str) = lit {
          return lit_str.value();
        }
      }
      // Fallback: try parsing as name-value: #[tag = "validation"]
      if let Ok(meta) = attr.parse_args::<Meta>() {
        if let Meta::NameValue(MetaNameValue { value, .. }) = meta {
          if let syn::Expr::Lit(expr_lit) = value {
            if let Lit::Str(lit_str) = &expr_lit.lit {
              return lit_str.value();
            }
          }
        }
      }
    }
  }
  // Default to snake_case variant name
  to_snake_case(variant_name)
}

/// Convert CamelCase to snake_case.
fn to_snake_case(s: &str) -> String {
  let mut result = String::new();
  for (i, ch) in s.chars().enumerate() {
    if ch.is_uppercase() && i > 0 {
      result.push('_');
    }
    result.push(ch.to_ascii_lowercase());
  }
  result
}

/// Convert variant name to SCREAMING_SNAKE_CASE for const name.
fn to_screaming_snake_case(s: &str) -> String {
  let mut result = String::new();
  for (i, ch) in s.chars().enumerate() {
    if ch.is_uppercase() && i > 0 {
      result.push('_');
    }
    result.push(ch.to_ascii_uppercase());
  }
  result
}

pub fn derive_tagged_error(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;

  let data = match &input.data {
    Data::Enum(data) => data,
    _ => {
      return syn::Error::new(
        Span::call_site(),
        "TaggedError can only be derived for enums",
      )
      .to_compile_error()
      .into();
    }
  };

  // Build tag match arms and const declarations
  let mut tag_arms = Vec::new();
  let mut const_decls = Vec::new();

  for variant in &data.variants {
    let variant_name = &variant.ident;
    let tag = extract_tag(&variant.attrs, &variant_name.to_string());
    let const_name = to_screaming_snake_case(&variant_name.to_string());
    let const_ident = Ident::new(&const_name, Span::call_site());

    // Match arm for tag()
    match &variant.fields {
      Fields::Named(_) => {
        tag_arms.push(quote! {
          #name::#variant_name { .. } => #tag,
        });
      }
      Fields::Unnamed(_) => {
        tag_arms.push(quote! {
          #name::#variant_name(..) => #tag,
        });
      }
      Fields::Unit => {
        tag_arms.push(quote! {
          #name::#variant_name => #tag,
        });
      }
    }

    // Const declaration
    const_decls.push(quote! {
      /// Tag constant for this error variant.
      pub const #const_ident: &'static str = #tag;
    });
  }

  let expanded = quote! {
    #[automatically_derived]
    impl ::effectful::failure::TaggedError for #name {
      fn tag(&self) -> &'static str {
        match self {
          #(#tag_arms)*
        }
      }
    }

    #[automatically_derived]
    impl #name {
      #(#const_decls)*
    }
  };

  TokenStream::from(expanded)
}
