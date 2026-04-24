use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
  Expr, FnArg, Ident, ItemFn, LitStr, Pat, Result, ReturnType, Token, Type, parenthesized,
};

use crate::expand::crate_path;

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
  let original_item = TokenStream2::from(item.clone());
  let args = match syn::parse::<SpanArgs>(attr) {
    Ok(args) => args,
    Err(err) => return err.to_compile_error().into(),
  };
  let input = match syn::parse::<ItemFn>(item) {
    Ok(input) => input,
    Err(err) => return err.to_compile_error().into(),
  };
  match expand_item_fn(args, input) {
    Ok(tokens) => tokens.into(),
    Err(err) => {
      let compile_error = err.to_compile_error();
      quote! { #compile_error #original_item }.into()
    }
  }
}

#[derive(Default)]
struct SpanArgs {
  name: Option<LitStr>,
  level: SpanLevelArg,
  skip_all: bool,
  skip: Vec<Ident>,
  fields: Vec<FieldArg>,
  sample: Option<syn::LitFloat>,
}

impl Parse for SpanArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut out = SpanArgs::default();
    let args = Punctuated::<SpanArg, Token![,]>::parse_terminated(input)?;
    for arg in args {
      match arg {
        SpanArg::Name(name) => out.name = Some(name),
        SpanArg::Level(level) => out.level = level,
        SpanArg::SkipAll => out.skip_all = true,
        SpanArg::Skip(skip) => out.skip.extend(skip),
        SpanArg::Fields(fields) => out.fields.extend(fields),
        SpanArg::Sample(sample) => out.sample = Some(sample),
      }
    }
    Ok(out)
  }
}

enum SpanArg {
  Name(LitStr),
  Level(SpanLevelArg),
  SkipAll,
  Skip(Vec<Ident>),
  Fields(Vec<FieldArg>),
  Sample(syn::LitFloat),
}

impl Parse for SpanArg {
  fn parse(input: ParseStream) -> Result<Self> {
    let key = Ident::parse(input)?;
    let key_text = key.to_string();
    match key_text.as_str() {
      "name" => {
        <Token![=]>::parse(input)?;
        Ok(SpanArg::Name(input.parse()?))
      }
      "level" => {
        <Token![=]>::parse(input)?;
        Ok(SpanArg::Level(SpanLevelArg::parse(input)?))
      }
      "skip_all" => Ok(SpanArg::SkipAll),
      "sample" => {
        <Token![=]>::parse(input)?;
        Ok(SpanArg::Sample(input.parse()?))
      }
      "skip" => {
        let content;
        parenthesized!(content in input);
        let values = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
        Ok(SpanArg::Skip(values.into_iter().collect()))
      }
      "fields" => {
        let content;
        parenthesized!(content in input);
        let values = Punctuated::<FieldArg, Token![,]>::parse_terminated(&content)?;
        Ok(SpanArg::Fields(values.into_iter().collect()))
      }
      _ => Err(syn::Error::new_spanned(
        key,
        "effectful::span: expected name, level, skip, skip_all, sample, or fields",
      )),
    }
  }
}

enum SpanLevelArg {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
}

impl Default for SpanLevelArg {
  fn default() -> Self {
    Self::Info
  }
}

impl Parse for SpanLevelArg {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(LitStr) {
      let lit: LitStr = input.parse()?;
      return parse_level_name(&lit.value())
        .ok_or_else(|| syn::Error::new_spanned(lit, "effectful::span: invalid level"));
    }
    let ident = Ident::parse(input)?;
    parse_level_name(&ident.to_string())
      .ok_or_else(|| syn::Error::new_spanned(ident, "effectful::span: invalid level"))
  }
}

fn parse_level_name(name: &str) -> Option<SpanLevelArg> {
  match name {
    "trace" | "TRACE" => Some(SpanLevelArg::Trace),
    "debug" | "DEBUG" => Some(SpanLevelArg::Debug),
    "info" | "INFO" => Some(SpanLevelArg::Info),
    "warn" | "WARN" => Some(SpanLevelArg::Warn),
    "error" | "ERROR" => Some(SpanLevelArg::Error),
    _ => None,
  }
}

struct FieldArg {
  key: Ident,
  format: FieldFormat,
  expr: Expr,
}

impl Parse for FieldArg {
  fn parse(input: ParseStream) -> Result<Self> {
    let key = Ident::parse(input)?;
    <Token![=]>::parse(input)?;
    let format = if input.peek(Token![?]) {
      <Token![?]>::parse(input)?;
      FieldFormat::Debug
    } else if input.peek(Token![%]) {
      <Token![%]>::parse(input)?;
      FieldFormat::Display
    } else {
      FieldFormat::Typed
    };
    let expr = Expr::parse(input)?;
    Ok(Self { key, format, expr })
  }
}

enum FieldFormat {
  Typed,
  Debug,
  Display,
}

fn expand_item_fn(args: SpanArgs, input: ItemFn) -> Result<TokenStream2> {
  let path = crate_path();
  let effect_ty = effect_return_type(&input.sig.output)?;
  let vis = input.vis;
  let attrs = input.attrs;
  let sig = input.sig;
  let block = input.block;
  let options = span_options_tokens(&args, &sig, &path)?;
  let sample_val = match &args.sample {
    Some(v) => quote! { ::core::option::Option::Some(#v) },
    None => quote! { ::core::option::Option::None },
  };
  if sig.asyncness.is_some() {
    Ok(quote! {
      #(#attrs)*
      #vis #sig {
        let __effectful_span_options = #options;
        let __effectful_span_effect: #effect_ty = (async move #block).await;
        #path::Effect::new_async(move |__effectful_span_env| {
          #path::box_future(async move {
            #path::with_span_options(__effectful_span_effect, __effectful_span_options)
              .run(__effectful_span_env)
              .await
          })
        })
      }
    })
  } else {
    Ok(quote! {
      #(#attrs)*
      #vis #sig {
        #path::__effectful_span_lazy_scoped(
          move |__effectful_span_instrument| -> (::core::option::Option<#path::SpanOptions>, #effect_ty) {
            if __effectful_span_instrument {
              let __effectful_span_options = #options;
              let __effectful_span_effect: #effect_ty = (|| #block)();
              (::core::option::Option::Some(__effectful_span_options), __effectful_span_effect)
            } else {
              let __effectful_span_effect: #effect_ty = (|| #block)();
              (::core::option::Option::None, __effectful_span_effect)
            }
          },
          #sample_val,
        )
      }
    })
  }
}

fn effect_return_type(output: &ReturnType) -> Result<Type> {
  let ReturnType::Type(_, ty) = output else {
    return Err(syn::Error::new_spanned(
      output,
      "effectful::span: function must return Effect",
    ));
  };
  let Type::Path(path) = ty.as_ref() else {
    return Err(syn::Error::new_spanned(
      ty,
      "effectful::span: function must return Effect",
    ));
  };
  let Some(segment) = path.path.segments.last() else {
    return Err(syn::Error::new_spanned(
      ty,
      "effectful::span: function must return Effect",
    ));
  };
  if segment.ident != "Effect" {
    return Err(syn::Error::new_spanned(
      ty,
      "effectful::span: function must return Effect",
    ));
  }
  Ok(*ty.clone())
}

fn span_options_tokens(
  args: &SpanArgs,
  sig: &syn::Signature,
  path: &TokenStream2,
) -> Result<TokenStream2> {
  let name = match &args.name {
    Some(name) => quote! { #name },
    None => {
      let fn_name = &sig.ident;
      quote! { ::std::format!("{}::{}", ::core::module_path!(), ::core::stringify!(#fn_name)) }
    }
  };
  let level = level_tokens(&args.level, path);
  let captured_args = argument_attribute_tokens(args, sig, path)?;
  let fields = field_attribute_tokens(args, path);
  let sample = sample_tokens(&args.sample);
  Ok(quote! {{
    let mut __effectful_span_options = #path::SpanOptions::new(#name).with_level(#level);
    #(#captured_args)*
    #(#fields)*
    #sample
    __effectful_span_options
  }})
}

fn sample_tokens(sample: &Option<syn::LitFloat>) -> TokenStream2 {
  match sample {
    Some(val) => {
      quote! { __effectful_span_options.sample_rate = ::core::option::Option::Some(#val); }
    }
    None => TokenStream2::new(),
  }
}

fn level_tokens(level: &SpanLevelArg, path: &TokenStream2) -> TokenStream2 {
  match level {
    SpanLevelArg::Trace => quote! { #path::SpanLevel::Trace },
    SpanLevelArg::Debug => quote! { #path::SpanLevel::Debug },
    SpanLevelArg::Info => quote! { #path::SpanLevel::Info },
    SpanLevelArg::Warn => quote! { #path::SpanLevel::Warn },
    SpanLevelArg::Error => quote! { #path::SpanLevel::Error },
  }
}

fn argument_attribute_tokens(
  args: &SpanArgs,
  sig: &syn::Signature,
  _path: &TokenStream2,
) -> Result<Vec<TokenStream2>> {
  if args.skip_all {
    return Ok(Vec::new());
  }
  let mut tokens = Vec::new();
  for input in &sig.inputs {
    let FnArg::Typed(pat_ty) = input else {
      continue;
    };
    let Pat::Ident(pat) = pat_ty.pat.as_ref() else {
      return Err(syn::Error::new_spanned(
        &pat_ty.pat,
        "effectful::span: non-ident arguments must be skipped",
      ));
    };
    let ident = &pat.ident;
    if args.skip.iter().any(|skip| skip == ident) {
      continue;
    }
    let key = ident.to_string();
    tokens.push(quote! {
      __effectful_span_options = __effectful_span_options
        .with_attribute(#key, ::std::format!("{:?}", &#ident));
    });
  }
  Ok(tokens)
}

fn field_attribute_tokens(args: &SpanArgs, _path: &TokenStream2) -> Vec<TokenStream2> {
  args
    .fields
    .iter()
    .map(|field| {
      let key = field.key.to_string();
      let expr = &field.expr;
      match field.format {
        FieldFormat::Typed => quote! {
          __effectful_span_options = __effectful_span_options.with_attribute(#key, (#expr));
        },
        FieldFormat::Debug => quote! {
          __effectful_span_options = __effectful_span_options
            .with_attribute(#key, ::std::format!("{:?}", &(#expr)));
        },
        FieldFormat::Display => quote! {
          __effectful_span_options = __effectful_span_options
            .with_attribute(#key, ::std::format!("{}", &(#expr)));
        },
      }
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use quote::quote;

  #[test]
  fn default_args_are_debug_captured_lazily_inside_effect() {
    let args = syn::parse2::<SpanArgs>(quote! {}).expect("args");
    let item = syn::parse2::<ItemFn>(quote! {
      fn load(id: u32) -> ::effectful::Effect<u32, (), ()> {
        ::effectful::succeed(id)
      }
    })
    .expect("item");
    let out = expand_item_fn(args, item).expect("expand").to_string();
    assert!(out.contains("format !"));
    assert!(out.contains("id"));
    assert!(out.contains("__effectful_span_lazy"));
  }

  #[test]
  fn skip_all_omits_argument_capture() {
    let args = syn::parse2::<SpanArgs>(quote! { skip_all }).expect("args");
    let item = syn::parse2::<ItemFn>(quote! {
      fn load(secret: String) -> Effect<(), (), ()> { ::effectful::succeed(()) }
    })
    .expect("item");
    let out = expand_item_fn(args, item).expect("expand").to_string();
    assert!(!out.contains("with_attribute (\"secret\""));
  }

  #[test]
  fn rejects_non_effect_return() {
    let args = syn::parse2::<SpanArgs>(quote! {}).expect("args");
    let item = syn::parse2::<ItemFn>(quote! { fn load() -> u32 { 1 } }).expect("item");
    let err = expand_item_fn(args, item).expect_err("error");
    assert!(err.to_string().contains("must return Effect"));
  }
}
