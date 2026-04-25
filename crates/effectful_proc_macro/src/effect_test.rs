use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, ItemFn, LitStr, Result, Token};

use crate::expand::crate_path;

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
  let original_item = TokenStream2::from(item.clone());
  let args = match syn::parse::<EffectTestArgs>(attr) {
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
struct EffectTestArgs {
  runtime: Option<LitStr>,
  env: Option<syn::Path>,
  layer: Option<syn::Path>,
}

impl Parse for EffectTestArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut out = EffectTestArgs::default();
    let args = Punctuated::<EffectTestArg, Token![,]>::parse_terminated(input)?;
    for arg in args {
      match arg {
        EffectTestArg::Runtime(runtime) => out.runtime = Some(runtime),
        EffectTestArg::Env(env) => out.env = Some(env),
        EffectTestArg::Layer(layer) => out.layer = Some(layer),
      }
    }
    Ok(out)
  }
}

enum EffectTestArg {
  Runtime(LitStr),
  Env(syn::Path),
  Layer(syn::Path),
}

impl Parse for EffectTestArg {
  fn parse(input: ParseStream) -> Result<Self> {
    let key = Ident::parse(input)?;
    let key_text = key.to_string();
    <Token![=]>::parse(input)?;
    match key_text.as_str() {
      "runtime" => Ok(Self::Runtime(input.parse()?)),
      "env" => parse_path_lit(input).map(Self::Env),
      "layer" => parse_path_lit(input).map(Self::Layer),
      _ => Err(syn::Error::new_spanned(
        key,
        "effectful::effect_test: expected runtime, env, or layer",
      )),
    }
  }
}

fn parse_path_lit(input: ParseStream) -> Result<syn::Path> {
  let lit: LitStr = input.parse()?;
  lit.parse()
}

fn expand_item_fn(args: EffectTestArgs, input: ItemFn) -> Result<TokenStream2> {
  validate_args(&args)?;
  validate_signature(&input)?;

  let path = crate_path();
  let attrs = input.attrs;
  let vis = input.vis;
  let sig = input.sig;
  let name = sig.ident.clone();
  let body_name = format_ident!("__effectful_test_body_{name}");
  let output = sig.output;
  let block = input.block;
  let run_body = run_body_tokens(&path, &args, &body_name);

  Ok(quote! {
    #(#attrs)*
    #[#path::testing::__tokio::test(flavor = "current_thread")]
    #vis async fn #name() {
      fn #body_name() #output #block
      #run_body
    }
  })
}

fn validate_args(args: &EffectTestArgs) -> Result<()> {
  if let Some(runtime) = &args.runtime {
    if runtime.value() != "tokio" {
      return Err(syn::Error::new_spanned(
        runtime,
        "effectful::effect_test: only runtime = \"tokio\" is supported",
      ));
    }
  }
  if args.env.is_some() && args.layer.is_some() {
    return Err(syn::Error::new(
      proc_macro2::Span::call_site(),
      "effectful::effect_test: use env or layer, not both",
    ));
  }
  Ok(())
}

fn validate_signature(input: &ItemFn) -> Result<()> {
  if input.sig.asyncness.is_some() {
    return Err(syn::Error::new_spanned(
      &input.sig.asyncness,
      "effectful::effect_test: test function must return Effect, not be async",
    ));
  }
  if !input.sig.inputs.is_empty() {
    return Err(syn::Error::new_spanned(
      &input.sig.inputs,
      "effectful::effect_test: test function must not take arguments",
    ));
  }
  Ok(())
}

fn run_body_tokens(
  path: &TokenStream2,
  args: &EffectTestArgs,
  body_name: &syn::Ident,
) -> TokenStream2 {
  if let Some(env) = &args.env {
    return quote! {
      #path::testing::expect_effect_test_with_env(#body_name(), #env()).await;
    };
  }
  if let Some(layer) = &args.layer {
    return quote! {
      #path::testing::expect_effect_test_with_layer(#body_name(), #layer()).await;
    };
  }
  quote! {
    #path::testing::expect_effect_test(#body_name()).await;
  }
}
