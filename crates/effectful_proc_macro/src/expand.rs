use proc_macro_crate::{FoundCrate, crate_name};
use quote::quote;

use crate::parse::EffectKind;
use crate::transform::{
  effect_body_contains_await, effect_body_contains_bind, expand_bare_body, expand_closure_body,
};

pub fn expand(kind: EffectKind) -> proc_macro2::TokenStream {
  let path = crate_path();
  match kind {
    EffectKind::DoNotation {
      param,
      env_ty,
      body,
    } => {
      let r = &param;
      let env_ty = *env_ty;
      let has_bind = effect_body_contains_bind(body.clone());
      let has_await = effect_body_contains_await(body.clone());
      let expanded = expand_closure_body(body, r, &path);
      if has_bind {
        // For R = (), the async block is 'static; avoid the AsyncBorrow closure box
        let is_unit_env = is_unit_type(&env_ty);
        if is_unit_env {
          quote! {
            #path::Effect::new_inline_async(async move {
              let #r = &mut ();
              #expanded
            })
          }
        } else {
          quote! {
            #path::Effect::new_async(move | #r: &mut #env_ty | {
              #path::box_future(async move { #expanded })
            })
          }
        }
      } else if has_await {
        // Await without binds: no need for env parameter, use new_inline_async
        quote! {
          #path::Effect::new_inline_async(async move { #expanded })
        }
      } else {
        quote! {
          #path::Effect::new(move | #r: &mut #env_ty | {
            #expanded
          })
        }
      }
    }
    EffectKind::Bare { body } => {
      let has_bind = effect_body_contains_bind(body.clone());
      let has_await = effect_body_contains_await(body.clone());
      let expanded = expand_bare_body(body, &path);
      if has_bind {
        // Bare effects have R = (). For bind operands that don't need the env
        // (like YieldNow, Result, or Effect<(), E, ()>), we can create &mut ()
        // inside the async block and use new_inline_async, avoiding the closure
        // allocation in new_async.
        quote! {
          #path::Effect::new_inline_async(async move {
            let __effect_r = &mut ();
            #expanded
          })
        }
      } else if has_await {
        quote! {
          #path::Effect::new_inline_async(async move { #expanded })
        }
      } else {
        quote! {
          #path::Effect::new(move |__effect_r: &mut ()| {
            #expanded
          })
        }
      }
    }
  }
}

/// Detect if a type is the unit type `()`.
fn is_unit_type(ty: &syn::Type) -> bool {
  matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

/// We always use `::effectful::…` so the generated code resolves in the caller's crate.
pub fn crate_path() -> proc_macro2::TokenStream {
  match crate_name("effectful") {
    Ok(FoundCrate::Itself) => quote!(::effectful),
    Ok(FoundCrate::Name(name)) => {
      let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
      quote!(::#ident)
    }
    Err(_) => quote!(::effectful),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parse::parse_effect_input;
  use quote::quote;

  fn expanded_contains_new_async(ts: &proc_macro2::TokenStream) -> bool {
    ts.to_string().contains("new_async")
  }

  fn expanded_contains_new_inline_async(ts: &proc_macro2::TokenStream) -> bool {
    ts.to_string().contains("new_inline_async")
  }

  fn expanded_uses_box_future(ts: &proc_macro2::TokenStream) -> bool {
    ts.to_string().contains("box_future")
  }

  #[test]
  fn bind_free_do_notation_uses_effect_new_not_async() {
    let input = quote! { |_r: &mut ()| { let x = 1; let y = 2; x + y } };
    let kind = parse_effect_input(input).expect("parse");
    let out = expand(kind);
    assert!(
      !expanded_contains_new_async(&out) && !expanded_contains_new_inline_async(&out),
      "expected Effect::new, got: {out}"
    );
    assert!(
      !expanded_uses_box_future(&out),
      "bind-free body should not use box_future async block: {out}"
    );
  }

  #[test]
  fn bind_in_bare_effect_uses_new_inline_async() {
    let input = quote! { bind* fail::<(), (), ()>(()) };
    let kind = parse_effect_input(input).expect("parse");
    let out = expand(kind);
    assert!(
      expanded_contains_new_inline_async(&out),
      "expected new_inline_async for bare effect with bind: {out}"
    );
    assert!(
      !expanded_contains_new_async(&out),
      "should not use new_async for bare effect: {out}"
    );
    assert!(
      !expanded_uses_box_future(&out),
      "should not use box_future for bare effect with bind: {out}"
    );
  }

  #[test]
  fn bind_free_bare_effect_uses_effect_new() {
    let input = quote! { 41 };
    let kind = parse_effect_input(input).expect("parse");
    let out = expand(kind);
    assert!(
      !expanded_contains_new_async(&out) && !expanded_contains_new_inline_async(&out),
      "expected Effect::new: {out}"
    );
  }

  #[test]
  fn await_without_bind_uses_new_inline_async() {
    let input = quote! { |_r: &mut ()| { foo().await } };
    let kind = parse_effect_input(input).expect("parse");
    let out = expand(kind);
    assert!(
      expanded_contains_new_inline_async(&out),
      "expected new_inline_async for await without bind: {out}"
    );
    assert!(
      !expanded_contains_new_async(&out),
      "should not use new_async when no binds: {out}"
    );
    assert!(
      !expanded_uses_box_future(&out),
      "new_inline_async should not use box_future: {out}"
    );
  }

  #[test]
  fn await_inside_async_move_closure_does_not_force_new_async() {
    use crate::transform::effect_body_contains_await;
    let body = quote! {
      let s = f(|x| async move { x.foo().await });
    };
    assert!(!effect_body_contains_await(body));
  }

  #[test]
  fn top_level_await_forces_async_detection() {
    use crate::transform::effect_body_contains_await;
    let body = quote! {
      foo().await
    };
    assert!(effect_body_contains_await(body));
  }

  #[test]
  fn bind_in_do_notation_with_unit_env_uses_new_inline_async() {
    // Do-notation with R = () should use new_inline_async, avoiding the closure box
    let input = quote! { |_r: &mut ()| { bind* fail::<(), (), ()>(()) } };
    let kind = parse_effect_input(input).expect("parse");
    let out = expand(kind);
    assert!(
      expanded_contains_new_inline_async(&out),
      "expected new_inline_async for do-notation with unit env: {out}"
    );
    assert!(
      !expanded_contains_new_async(&out),
      "should not use new_async when env is unit: {out}"
    );
    assert!(
      !expanded_uses_box_future(&out),
      "new_inline_async should not use box_future: {out}"
    );
  }

  #[test]
  fn bind_in_do_notation_with_non_unit_env_uses_new_async() {
    // Do-notation with R != () should still use new_async
    let input = quote! { |r: &mut String| { bind* fail::<(), (), String>(()) } };
    let kind = parse_effect_input(input).expect("parse");
    let out = expand(kind);
    assert!(
      expanded_contains_new_async(&out),
      "expected new_async for do-notation with non-unit env: {out}"
    );
    assert!(
      !expanded_contains_new_inline_async(&out),
      "should not use new_inline_async when env is non-unit: {out}"
    );
    assert!(
      expanded_uses_box_future(&out),
      "new_async should use box_future: {out}"
    );
  }
}
