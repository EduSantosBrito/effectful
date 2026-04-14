//! Effect.rs Dylint rules. See `.cursor/skills/effect.rs-fundamentals/SKILL.md` and
//! `crates/effect-rs/SPEC.md` (strata / DI / boundaries).
//!
//! Lints default to **`deny`** so `cargo dylint` / Moon `:lint` enforces project rules.
//!
//! - **`EffectStyleLate`**: style rules for compositional `Effect` code. Crates whose names start with
//!   `forge_` are skipped so `forge-*` packages need no crate-level `#![allow]`.
//! - **`EffectInteropLate`**: dependency-injection and boundary rules that apply to all application
//!   crates; skips only the `effect` runtime / macro crates that implement the
//!   system (see `effect_interop_skip_crate`).
//!
//! `Effect`-returning functions are expected to use a **single** top-level `effect!(…)` body
//! (source-snippet heuristic: the substring `effect!(`); there is no `succeed`/`pipe!`-only escape.

#![feature(rustc_private)]
#![warn(unused_extern_crates)]
// Nested `if let` is often clearer than let-chains in rustc internals traversals.
#![allow(clippy::collapsible_if)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use rustc_ast as ast;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, LOCAL_CRATE};
use rustc_hir::intravisit::FnKind;
use rustc_hir::{self as hir, Expr, ExprKind, GenericParamKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LateContext, LateLintPass, LintContext};
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;
use rustc_span::{FileName, Span, Symbol};

dylint_linting::dylint_library!();

// --- Lints (all Deny) -------------------------------------------------------

declare_lint! {
    pub EFFECT_SUCCESS_SHOULD_BE_FN_TYPE_PARAM,
    Deny,
    "Effect success type A must be a function type parameter (see Effect.rs skill)"
}

declare_lint! {
    pub EFFECT_NO_EFFECT_SUFFIX_ON_GRAPH_BUILDER,
    Deny,
    "Do not suffix graph builders with `_effect`; name the operation (Effect is in the return type)"
}

declare_lint! {
    pub EFFECT_RUN_BLOCKING_OUTSIDE_BOUNDARY,
    Deny,
    "Call `run_blocking` only at program boundary (main), tests, examples, or benches — not in library code"
}

declare_lint! {
    pub EFFECT_RUN_ASYNC_OUTSIDE_BOUNDARY,
    Deny,
    "Call `run_async` only at program boundary (main), tests, examples, or benches — not in library code"
}

declare_lint! {
    pub EFFECT_PREFER_FROM_ASYNC_OVER_NEW_ASYNC,
    Deny,
    "Prefer `from_async` for `'static` third-party `.await` + `map_err`; use `Effect::new_async` only when borrowing `&mut R` across `.await`"
}

declare_lint! {
    pub EFFECT_FROM_ASYNC_SINGLE_AWAIT,
    Deny,
    "The future passed to `from_async` should contain at most one `.await` (one I/O boundary per step)"
}

declare_lint! {
    pub EFFECT_EFFECT_GENERICS_NEED_BOUNDS,
    Deny,
    "Functions returning `Effect<A, E, R>` must give every type parameter at least one bound (in `<>` or `where`)"
}

declare_lint! {
    pub EFFECT_MULTIPLE_TOP_LEVEL_EFFECT_MACROS,
    Deny,
    "Use a single top-level `effect!(…)` per function; put `if`/`match` inside the macro body"
}

declare_lint! {
    pub EFFECT_RETURNING_EFFECT_SHOULD_USE_EFFECT_MACRO,
    Deny,
    "Functions returning `Effect` must use a single top-level `effect!(…)` for the function body"
}

declare_lint! {
    pub EFFECT_NO_ASYNC_FN_APPLICATION,
    Deny,
    "No `async fn` for application logic; return `Effect<A, E, R>` and run at the boundary (integration crates may use `#[allow]`)"
}

declare_lint! {
    pub EFFECT_NO_RAW_TRACING,
    Deny,
    "Do not use `tracing::…` macros; use `effect_logger` via `~EffectLogger` inside `effect!`"
}

declare_lint! {
    pub EFFECT_NO_EFFECT_NEW_OUTSIDE_EFFECT_CRATE,
    Deny,
    "Do not call `Effect::new` outside the `effect` crate; build graphs with `succeed`, `fail`, `from_async`, and combinators"
}

declare_lint! {
    pub EFFECT_WHERE_USES_RAW_GET_TRAIT,
    Deny,
    "Prefer a project `Needs…` supertrait over `effect::context::Get` / `GetMut` in public `where` clauses (call sites stay readable)"
}

declare_lint! {
    pub EFFECT_NO_INSTANT_NOW_OUTSIDE_BOUNDARY,
    Deny,
    "Avoid `Instant::now` in library code; inject `Clock` / `TestClock` (see `effect::scheduling`) for deterministic time"
}

declare_lint! {
    pub EFFECT_LEGACY_POSTFIX_TILDE,
    Deny,
    "Obsolete Effect.rs syntax: use `let x = ~expr;` (prefix `~`) instead of postfix `x ~ expr`"
}

declare_lint_pass!(EffectStyleLate => [
    EFFECT_SUCCESS_SHOULD_BE_FN_TYPE_PARAM,
    EFFECT_NO_EFFECT_SUFFIX_ON_GRAPH_BUILDER,
    EFFECT_RUN_BLOCKING_OUTSIDE_BOUNDARY,
    EFFECT_RUN_ASYNC_OUTSIDE_BOUNDARY,
    EFFECT_PREFER_FROM_ASYNC_OVER_NEW_ASYNC,
    EFFECT_FROM_ASYNC_SINGLE_AWAIT,
    EFFECT_EFFECT_GENERICS_NEED_BOUNDS,
    EFFECT_MULTIPLE_TOP_LEVEL_EFFECT_MACROS,
    EFFECT_RETURNING_EFFECT_SHOULD_USE_EFFECT_MACRO,
    EFFECT_NO_ASYNC_FN_APPLICATION,
]);

declare_lint_pass!(EffectInteropLate => [
    EFFECT_NO_EFFECT_NEW_OUTSIDE_EFFECT_CRATE,
    EFFECT_WHERE_USES_RAW_GET_TRAIT,
    EFFECT_NO_INSTANT_NOW_OUTSIDE_BOUNDARY,
    EFFECT_LEGACY_POSTFIX_TILDE,
]);

declare_lint_pass!(EffectTracingEarly => [EFFECT_NO_RAW_TRACING]);

// --- Crate / path helpers ---------------------------------------------------

fn file_path_string(cx: &LateContext<'_>, span: Span) -> Option<String> {
  let sm = cx.tcx.sess.source_map();
  match sm.span_to_filename(span) {
    FileName::Real(name) => Some(name.local_path()?.display().to_string()),
    _ => None,
  }
}

fn path_is_tests_examples_or_benches(path: &str) -> bool {
  path.contains("/tests/")
    || path.contains("/examples/")
    || path.contains("/benches/")
    || path.contains("/src/bin/")
    || path.contains("/src/main.rs")
    || path.contains("\\tests\\")
    || path.contains("\\examples\\")
    || path.contains("\\benches\\")
    || path.contains("\\src\\bin\\")
    || path.contains("\\src\\main.rs")
}

fn attr_path_string(attr: &hir::Attribute) -> String {
  attr
    .path()
    .iter()
    .map(|s| s.as_str())
    .collect::<Vec<_>>()
    .join("::")
}

fn fn_has_test_like_attr(cx: &LateContext<'_>, def_id: rustc_hir::def_id::LocalDefId) -> bool {
  let hir_id = cx.tcx.local_def_id_to_hir_id(def_id);
  cx.tcx.hir_attrs(hir_id).iter().any(|attr| {
    let p = attr_path_string(attr);
    p == "test" || p.ends_with("::test") || p.contains("tokio::test") || p.contains("rstest")
  })
}

fn boundary_ok_for_run(
  cx: &LateContext<'_>,
  def_id: rustc_hir::def_id::LocalDefId,
  span: Span,
) -> bool {
  let did = def_id.to_def_id();
  // `item_name` panics on anonymous items (e.g. nested closures inside `async` blocks).
  if matches!(
    cx.tcx.def_kind(did),
    DefKind::Closure | DefKind::InlineConst
  ) {
    return true;
  }
  let name = cx.tcx.item_name(did);
  let name_str = name.as_str();
  if name_str == "main" || name_str.ends_with("_blocking") {
    return true;
  }
  if fn_has_test_like_attr(cx, def_id) {
    return true;
  }
  if let Some(p) = file_path_string(cx, span) {
    if path_is_tests_examples_or_benches(&p) {
      return true;
    }
  }
  false
}

fn enclosing_fn_for_expr(
  cx: &LateContext<'_>,
  hir_id: rustc_hir::HirId,
) -> rustc_hir::def_id::LocalDefId {
  cx.tcx.hir_enclosing_body_owner(hir_id)
}

// --- Effect / def helpers --------------------------------------------------

fn is_effect_adt(cx: &LateContext<'_>, did: DefId) -> bool {
  cx.tcx.item_name(did).as_str() == "Effect" && cx.tcx.crate_name(did.krate).as_str() == "effect"
}

/// Proc-macro and logger crates implement Effect.rs infrastructure. `forge_*` packages
/// opt out of style lints at the driver (no `#![allow]` in those crates); other compositional code is still checked.
fn effect_style_late_skip_crate(cx: &LateContext<'_>) -> bool {
  let sym = cx.tcx.crate_name(LOCAL_CRATE);
  let name = sym.as_str();
  name.starts_with("forge_")
    || matches!(name, "effect_logger" | "effect_macro" | "effect_proc_macro")
}

/// Object-safe ports use `fn … -> Effect<ConcreteSuccess, …>` in the trait; impls cannot use
/// generic `A` without breaking `dyn Trait`. Skip `check_fn` style rules for trait items and
/// their impls; inherent `fn` returning `Effect` are still linted.
fn effect_style_skip_trait_assoc_fn(
  cx: &LateContext<'_>,
  def_id: rustc_hir::def_id::LocalDefId,
) -> bool {
  let did = def_id.to_def_id();
  let Some(parent) = cx.tcx.opt_parent(did) else {
    return false;
  };
  match cx.tcx.def_kind(parent) {
    DefKind::Trait => true,
    DefKind::Impl { .. } => cx.tcx.impl_trait_ref(parent).is_some(),
    _ => false,
  }
}

fn effect_success_ty<'tcx>(cx: &LateContext<'tcx>, output: ty::Ty<'tcx>) -> Option<ty::Ty<'tcx>> {
  let output = output.peel_refs();
  if let ty::Adt(adt_def, substs) = output.kind() {
    if is_effect_adt(cx, adt_def.did()) && !substs.is_empty() {
      return Some(substs.type_at(0));
    }
  }
  None
}

fn success_ty_is_fn_type_param<'tcx>(
  cx: &LateContext<'tcx>,
  fn_def_id: DefId,
  success: ty::Ty<'tcx>,
) -> bool {
  let &ty::Param(p) = success.kind() else {
    return false;
  };
  let mut current = Some(cx.tcx.generics_of(fn_def_id));
  while let Some(gens) = current {
    if gens.own_params.iter().any(|gp| {
      matches!(gp.kind, rustc_middle::ty::GenericParamDefKind::Type { .. }) && gp.index == p.index
    }) {
      return true;
    }
    current = gens.parent.map(|did| cx.tcx.generics_of(did));
  }
  false
}

fn type_param_has_bounds<'tcx>(
  generics: &'tcx hir::Generics<'tcx>,
  param: &'tcx hir::GenericParam<'tcx>,
) -> bool {
  generics.bounds_for_param(param.def_id).next().is_some()
    || generics.outlives_for_param(param.def_id).next().is_some()
}

fn snippet_ok(cx: &LateContext<'_>, span: Span) -> Option<String> {
  cx.tcx.sess.source_map().span_to_snippet(span).ok()
}

fn count_substr(hay: &str, needle: &str) -> usize {
  hay.match_indices(needle).count()
}

fn is_effect_from_async(cx: &LateContext<'_>, def_id: DefId) -> bool {
  cx.tcx.crate_name(def_id.krate).as_str() == "effect"
    && cx.tcx.item_name(def_id).as_str() == "from_async"
}

fn is_effect_run_blocking(cx: &LateContext<'_>, def_id: DefId) -> bool {
  cx.tcx.crate_name(def_id.krate).as_str() == "effect"
    && cx.tcx.item_name(def_id).as_str() == "run_blocking"
}

fn is_effect_run_async(cx: &LateContext<'_>, def_id: DefId) -> bool {
  cx.tcx.crate_name(def_id.krate).as_str() == "effect"
    && cx.tcx.item_name(def_id).as_str() == "run_async"
}

fn is_effect_new_async_method(cx: &LateContext<'_>, def_id: DefId) -> bool {
  if cx.tcx.item_name(def_id).as_str() != "new_async" {
    return false;
  }
  let parent = cx.tcx.parent(def_id);
  if !matches!(cx.tcx.def_kind(parent), DefKind::Impl { .. }) {
    return false;
  }
  let ty = cx.tcx.type_of(parent).instantiate_identity();
  if let ty::Adt(adt_def, _) = ty.kind() {
    return is_effect_adt(cx, adt_def.did());
  }
  false
}

/// Inherent `Effect::new` (not `new_async`).
fn is_effect_new_method(cx: &LateContext<'_>, def_id: DefId) -> bool {
  if cx.tcx.item_name(def_id).as_str() != "new" {
    return false;
  }
  let parent = cx.tcx.parent(def_id);
  if !matches!(cx.tcx.def_kind(parent), DefKind::Impl { .. }) {
    return false;
  }
  let ty = cx.tcx.type_of(parent).instantiate_identity();
  if let ty::Adt(adt_def, _) = ty.kind() {
    return is_effect_adt(cx, adt_def.did());
  }
  false
}

/// Skip crates that *implement* the effect system and DI helpers (`effect_config`, `effect_logger`);
/// application crates are checked.
fn effect_interop_skip_crate(cx: &LateContext<'_>) -> bool {
  let sym = cx.tcx.crate_name(LOCAL_CRATE);
  let name = sym.as_str();
  matches!(
    name,
    "effect"
      | "effect_config"
      | "effect_logger"
      | "effect_macro"
      | "effect_proc_macro"
      | "effect_dylint_rules"
  )
}

fn trait_is_effect_get_or_getmut(cx: &LateContext<'_>, trait_def_id: DefId) -> bool {
  if cx.tcx.crate_name(trait_def_id.krate).as_str() != "effect" {
    return false;
  }
  matches!(cx.tcx.item_name(trait_def_id).as_str(), "Get" | "GetMut")
}

fn lint_where_clauses_use_raw_get_trait(cx: &LateContext<'_>, def_id: DefId) {
  // User-written `where` / `<>` bounds only (avoids repeating inherited impl predicates on every method).
  let preds = cx.tcx.explicit_predicates_of(def_id);
  for (pred, sp) in preds.predicates {
    let kind = pred.kind().skip_binder();
    if let ty::ClauseKind::Trait(trait_pred) = kind {
      if trait_is_effect_get_or_getmut(cx, trait_pred.trait_ref.def_id) {
        cx.span_lint(EFFECT_WHERE_USES_RAW_GET_TRAIT, *sp, |d| {
          d.primary_message(
            "prefer a `Needs…` supertrait (wrapping `Get`/`GetMut`) for environment bounds; keep `Get` only inside the `effect` crate",
          );
        });
      }
    }
  }
}

/// `Needs…` supertraits (and their blanket `impl<R: Get<…>> Needs… for R`) intentionally name `Get`/`GetMut`.
fn impl_block_implements_needs_trait(cx: &LateContext<'_>, impl_def_id: DefId) -> bool {
  cx.tcx.impl_trait_ref(impl_def_id).is_some_and(|tr| {
    cx.tcx
      .item_name(tr.skip_binder().def_id)
      .as_str()
      .starts_with("Needs")
  })
}

fn is_std_time_instant_now(cx: &LateContext<'_>, def_id: DefId) -> bool {
  let path = cx.tcx.def_path_str(def_id);
  path.ends_with("time::Instant::now")
}

/// Detect obsolete postfix `ident ~ expr` lines inside a function body snippet (not `let x = ~…`).
fn snippet_has_legacy_postfix_tilde(sn: &str) -> bool {
  for line in sn.lines() {
    let t = line.trim_start();
    if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
      continue;
    }
    if let Some(pos) = t.find(" ~ ") {
      let before = t[..pos].trim_end();
      if before.contains('=') {
        continue;
      }
      if before.starts_with("let ") {
        continue;
      }
      let mut tokens = before.split_whitespace();
      let Some(first) = tokens.next() else {
        continue;
      };
      if first.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !first.is_empty() {
        return true;
      }
    }
  }
  false
}

/// True when `Effect::new_async`'s first argument (the closure) likely polls a nested effect with
/// `.run(…).await` — the case `from_async` cannot express.
fn new_async_snippet_suggests_run_env_across_await(cx: &LateContext<'_>, expr: &Expr<'_>) -> bool {
  if let ExprKind::Call(_, args) = expr.kind {
    if let Some(arg0) = args.first() {
      if let Some(sn) = snippet_ok(cx, arg0.span) {
        return sn.contains(".run(") && sn.contains(".await");
      }
    }
  }
  false
}

fn call_callee_def_id(cx: &LateContext<'_>, expr: &Expr<'_>) -> Option<DefId> {
  match expr.kind {
    ExprKind::Call(f, _) => {
      if let Some(did) = cx.typeck_results().type_dependent_def_id(expr.hir_id) {
        return Some(did);
      }
      if let ExprKind::Path(qpath) = &f.kind {
        if let Res::Def(_, did) = cx.typeck_results().qpath_res(qpath, f.hir_id) {
          return Some(did);
        }
      }
      None
    }
    _ => None,
  }
}

// --- Late pass --------------------------------------------------------------

impl<'tcx> LateLintPass<'tcx> for EffectStyleLate {
  fn check_fn(
    &mut self,
    cx: &LateContext<'tcx>,
    kind: FnKind<'tcx>,
    _decl: &'tcx hir::FnDecl<'tcx>,
    body: &'tcx hir::Body<'tcx>,
    span: Span,
    def_id: rustc_hir::def_id::LocalDefId,
  ) {
    if effect_style_late_skip_crate(cx) {
      return;
    }
    if matches!(kind, FnKind::Closure) {
      return;
    }
    if effect_style_skip_trait_assoc_fn(cx, def_id) {
      return;
    }

    let fn_def_id = def_id.to_def_id();
    let sig = cx.tcx.fn_sig(fn_def_id).instantiate_identity();
    let output = sig.output().skip_binder();
    let returns_effect = effect_success_ty(cx, output).is_some();

    // async fn (application)
    if kind.asyncness().is_async() && !fn_has_test_like_attr(cx, def_id) {
      if let Some(path) = file_path_string(cx, span) {
        if !path_is_tests_examples_or_benches(&path) {
          cx.span_lint(EFFECT_NO_ASYNC_FN_APPLICATION, span, |d| {
            d.primary_message(
              "replace `async fn` with a function returning `Effect<A, E, R>`; run at the boundary",
            );
          });
        }
      }
    }

    // _effect suffix
    if returns_effect {
      let name = cx.tcx.item_name(fn_def_id);
      let s = name.as_str();
      if s.ends_with("_effect") && !s.ends_with("_blocking") {
        cx.span_lint(EFFECT_NO_EFFECT_SUFFIX_ON_GRAPH_BUILDER, span, |d| {
          d.primary_message(format!(
            "drop the `_effect` suffix from `{s}`; `Effect<…>` in the signature is enough"
          ));
        });
      }
    }

    // Effect success type = type param (skip async fn here)
    if returns_effect && !kind.asyncness().is_async() {
      if let Some(success) = effect_success_ty(cx, output) {
        if !success_ty_is_fn_type_param(cx, fn_def_id, success) {
          cx.span_lint(EFFECT_SUCCESS_SHOULD_BE_FN_TYPE_PARAM, span, |d| {
                        d.primary_message(
                            "return type `Effect<…>` uses a concrete success type; use `fn foo<A, E, R>(…) -> Effect<A, E, R>` with bounds on `A`",
                        );
                    });
        }
      }
    }

    // Generics: every type param has a bound (<> or where)
    if returns_effect {
      if let Some(generics) = cx.tcx.hir_get_generics(def_id) {
        for p in generics.params {
          if let GenericParamKind::Type {
            synthetic: true, ..
          } = p.kind
          {
            continue;
          }
          if let GenericParamKind::Type { .. } = p.kind {
            if !type_param_has_bounds(generics, p) {
              cx.span_lint(EFFECT_EFFECT_GENERICS_NEED_BOUNDS, p.span, |d| {
                                d.primary_message(
                                    "add a trait bound on this type parameter or mention it in a `where` clause (document the `Effect` contract)",
                                );
                            });
            }
          }
        }
      }
    }

    // Source-based effect! rules (macro-expanded HIR does not preserve invocations)
    if returns_effect && !kind.asyncness().is_async() {
      if let Some(sn) = snippet_ok(cx, body.value.span) {
        let n_effect = count_substr(&sn, "effect!(");
        if n_effect > 1 {
          cx.span_lint(EFFECT_MULTIPLE_TOP_LEVEL_EFFECT_MACROS, body.value.span, |d| {
                        d.primary_message(
                            "multiple `effect!(…)` in one function; use a single `effect!(|r| { … })` and branch inside",
                        );
                    });
        }
        if n_effect == 0 {
          cx.span_lint(EFFECT_RETURNING_EFFECT_SHOULD_USE_EFFECT_MACRO, body.value.span, |d| {
            d.primary_message(
              "wrap the entire function body in one top-level `effect!(|r| { … })` (and put `succeed`/`fail`/`pipe!` inside that block if needed)",
            );
          });
        }
      }
    }
  }

  fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
    if effect_style_late_skip_crate(cx) {
      return;
    }
    let Some(callee) = call_callee_def_id(cx, expr) else {
      return;
    };

    // run_blocking / run_async
    if is_effect_run_blocking(cx, callee) || is_effect_run_async(cx, callee) {
      let lint = if is_effect_run_blocking(cx, callee) {
        EFFECT_RUN_BLOCKING_OUTSIDE_BOUNDARY
      } else {
        EFFECT_RUN_ASYNC_OUTSIDE_BOUNDARY
      };
      let encl = enclosing_fn_for_expr(cx, expr.hir_id);
      if !boundary_ok_for_run(cx, encl, expr.span) {
        cx.span_lint(lint, expr.span, |d| {
          d.primary_message(
            "move `run_blocking` / `run_async` to `main`, tests, examples, or benches — keep library code as pure `Effect` values",
          );
        });
      }
    }

    // Effect::new_async — only suggest `from_async` when the closure is not clearly driving a
    // nested [`Effect::run`] with the same `&mut R` across `.await` (heuristic: `.run(` + `.await`
    // in the closure / async snippet).
    if is_effect_new_async_method(cx, callee)
      && !new_async_snippet_suggests_run_env_across_await(cx, expr)
    {
      cx.span_lint(EFFECT_PREFER_FROM_ASYNC_OVER_NEW_ASYNC, expr.span, |d| {
        d.primary_message(
          "prefer `from_async` when the future is `'static` and only needs `.await` + `map_err`; keep `new_async` for `&mut R` across `.await`",
        );
      });
    }

    // from_async: at most one .await in first arg (closure/async block span)
    if is_effect_from_async(cx, callee) {
      if let ExprKind::Call(_, args) = expr.kind {
        if let Some(first) = args.first() {
          if let Some(arg_sn) = snippet_ok(cx, first.span) {
            // Count `.await` — heuristic for "single I/O step"
            let awaits = count_substr(&arg_sn, ".await");
            if awaits > 1 {
              cx.span_lint(EFFECT_FROM_ASYNC_SINGLE_AWAIT, first.span, |d| {
                                d.primary_message(
                                    "`from_async` should wrap a minimal future (typically one `.await`); split additional steps into separate `~` binds in `effect!`",
                                );
                            });
            }
          }
        }
      }
    }
  }
}

impl<'tcx> LateLintPass<'tcx> for EffectInteropLate {
  fn check_fn(
    &mut self,
    cx: &LateContext<'tcx>,
    kind: FnKind<'tcx>,
    _decl: &'tcx hir::FnDecl<'tcx>,
    body: &'tcx hir::Body<'tcx>,
    _span: Span,
    def_id: rustc_hir::def_id::LocalDefId,
  ) {
    if effect_interop_skip_crate(cx) {
      return;
    }
    if matches!(kind, FnKind::Closure) {
      return;
    }
    if effect_style_skip_trait_assoc_fn(cx, def_id) {
      return;
    }

    lint_where_clauses_use_raw_get_trait(cx, def_id.to_def_id());

    let fn_def_id = def_id.to_def_id();
    let sig = cx.tcx.fn_sig(fn_def_id).instantiate_identity();
    let output = sig.output().skip_binder();
    let returns_effect = effect_success_ty(cx, output).is_some();

    if returns_effect && !kind.asyncness().is_async() {
      if let Some(sn) = snippet_ok(cx, body.value.span) {
        if snippet_has_legacy_postfix_tilde(&sn) {
          cx.span_lint(EFFECT_LEGACY_POSTFIX_TILDE, body.value.span, |d| {
            d.primary_message(
              "obsolete postfix `x ~ expr`; use `let x = ~expr;` (prefix `~`) inside `effect!`",
            );
          });
        }
      }
    }
  }

  fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
    if effect_interop_skip_crate(cx) {
      return;
    }
    let Some(callee) = call_callee_def_id(cx, expr) else {
      return;
    };

    if is_effect_new_method(cx, callee) {
      let encl = enclosing_fn_for_expr(cx, expr.hir_id);
      if !boundary_ok_for_run(cx, encl, expr.span) {
        cx.span_lint(EFFECT_NO_EFFECT_NEW_OUTSIDE_EFFECT_CRATE, expr.span, |d| {
          d.primary_message(
            "do not use `Effect::new` here; use `succeed`, `fail`, `from_async`, or `Effect` combinators",
          );
        });
      }
    }

    if is_std_time_instant_now(cx, callee) {
      let encl = enclosing_fn_for_expr(cx, expr.hir_id);
      if !boundary_ok_for_run(cx, encl, expr.span) {
        cx.span_lint(EFFECT_NO_INSTANT_NOW_OUTSIDE_BOUNDARY, expr.span, |d| {
          d.primary_message(
            "avoid `Instant::now` in library code; use `Clock` / `TestClock` from `effect::scheduling`",
          );
        });
      }
    }
  }

  fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
    if effect_interop_skip_crate(cx) {
      return;
    }
    let def_id = item.owner_id.to_def_id();
    match item.kind {
      hir::ItemKind::Trait(..) => {
        if cx.tcx.item_name(def_id).as_str().starts_with("Needs") {
          return;
        }
        lint_where_clauses_use_raw_get_trait(cx, def_id);
      }
      hir::ItemKind::Impl(..) => {
        if impl_block_implements_needs_trait(cx, def_id) {
          return;
        }
        lint_where_clauses_use_raw_get_trait(cx, def_id);
      }
      hir::ItemKind::Struct(..)
      | hir::ItemKind::Enum(..)
      | hir::ItemKind::Union(..)
      | hir::ItemKind::TyAlias(..) => {
        lint_where_clauses_use_raw_get_trait(cx, def_id);
      }
      _ => {}
    }
  }
}

impl EarlyLintPass for EffectTracingEarly {
  fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &ast::Expr) {
    let Some(name) = cx.sess().opts.crate_name.as_deref() else {
      return;
    };
    if name.starts_with("forge_") {
      return;
    }
    if matches!(
      name,
      "effect" | "effect_logger" | "effect_macro" | "effect_proc_macro" | "effect_dylint_rules"
    ) {
      return;
    }

    let ast::ExprKind::MacCall(mac) = &expr.kind else {
      return;
    };
    let segs = &mac.path.segments;
    if segs.len() < 2 {
      return;
    }
    if segs[0].ident.name != Symbol::intern("tracing") {
      return;
    }
    cx.span_lint(EFFECT_NO_RAW_TRACING, expr.span, |d| {
            d.primary_message(
                "do not use `tracing::…` macros here; use `effect_logger` and `~EffectLogger` inside `effect!`",
            );
        });
  }
}

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
  dylint_linting::init_config(sess);
  lint_store.register_lints(&[
    EFFECT_SUCCESS_SHOULD_BE_FN_TYPE_PARAM,
    EFFECT_NO_EFFECT_SUFFIX_ON_GRAPH_BUILDER,
    EFFECT_RUN_BLOCKING_OUTSIDE_BOUNDARY,
    EFFECT_RUN_ASYNC_OUTSIDE_BOUNDARY,
    EFFECT_PREFER_FROM_ASYNC_OVER_NEW_ASYNC,
    EFFECT_FROM_ASYNC_SINGLE_AWAIT,
    EFFECT_EFFECT_GENERICS_NEED_BOUNDS,
    EFFECT_MULTIPLE_TOP_LEVEL_EFFECT_MACROS,
    EFFECT_RETURNING_EFFECT_SHOULD_USE_EFFECT_MACRO,
    EFFECT_NO_ASYNC_FN_APPLICATION,
    EFFECT_NO_RAW_TRACING,
    EFFECT_NO_EFFECT_NEW_OUTSIDE_EFFECT_CRATE,
    EFFECT_WHERE_USES_RAW_GET_TRAIT,
    EFFECT_NO_INSTANT_NOW_OUTSIDE_BOUNDARY,
    EFFECT_LEGACY_POSTFIX_TILDE,
  ]);
  lint_store.register_late_pass(|_| Box::new(EffectStyleLate));
  lint_store.register_late_pass(|_| Box::new(EffectInteropLate));
  lint_store.register_early_pass(|| Box::new(EffectTracingEarly));
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  // ---------- path_is_tests_examples_or_benches ----------

  mod path_boundary {
    mod when_path_contains_harness_dirs {
      use super::super::*;

      #[rstest]
      #[case::unix_tests("/home/u/proj/my-crate/tests/integration.rs")]
      #[case::unix_examples("/home/u/proj/my-crate/examples/demo.rs")]
      #[case::unix_benches("/home/u/proj/my-crate/benches/throughput.rs")]
      #[case::unix_src_bin("/home/u/proj/my-crate/src/bin/cli.rs")]
      #[case::unix_src_main("/home/u/proj/my-crate/src/main.rs")]
      #[case::windows_tests(r"C:\proj\my-crate\tests\foo.rs")]
      #[case::windows_examples(r"C:\proj\my-crate\examples\foo.rs")]
      #[case::windows_benches(r"C:\proj\my-crate\benches\foo.rs")]
      #[case::windows_src_bin(r"C:\proj\my-crate\src\bin\tool.rs")]
      #[case::windows_src_main(r"C:\proj\my-crate\src\main.rs")]
      fn returns_true(#[case] path: &str) {
        assert!(path_is_tests_examples_or_benches(path));
      }
    }

    mod when_path_is_normal_library_source {
      use super::super::*;

      #[rstest]
      #[case::lib("/home/u/proj/my-crate/src/lib.rs")]
      #[case::module("/home/u/proj/my-crate/src/thing/mod.rs")]
      #[case::empty("")]
      #[case::substring_but_not_segment("/home/proj-footests/src/lib.rs")]
      fn returns_false(#[case] path: &str) {
        assert!(!path_is_tests_examples_or_benches(path));
      }
    }
  }

  // ---------- count_substr ----------

  mod count_substr {
    use super::*;

    #[test]
    fn returns_zero_when_needle_missing() {
      assert_eq!(count_substr("hello", "x"), 0);
    }

    #[test]
    fn counts_non_overlapping_occurrences() {
      assert_eq!(count_substr("ababab", "ab"), 3);
    }

    #[test]
    fn counts_effect_macro_invocations_in_snippet() {
      let sn = "let a = effect!(|r| { 1 });\nlet b = effect!(|r| { 2 });";
      assert_eq!(count_substr(sn, "effect!("), 2);
    }

    #[rstest]
    #[case::empty_hay("", "a", 0)]
    #[case::empty_needle_no_panic("abc", "x", 0)]
    fn with_edge_inputs(#[case] hay: &str, #[case] needle: &str, #[case] expected: usize) {
      assert_eq!(count_substr(hay, needle), expected);
    }
  }

  // ---------- snippet_has_legacy_postfix_tilde ----------

  mod legacy_postfix_tilde {
    use super::super::snippet_has_legacy_postfix_tilde;

    #[test]
    fn detects_obsolete_postfix_form() {
      assert!(snippet_has_legacy_postfix_tilde(
        "effect!(|_r| {\n    x ~ foo();\n})\n"
      ));
    }

    #[test]
    fn allows_prefix_tilde_bind() {
      assert!(!snippet_has_legacy_postfix_tilde(
        "effect!(|_r| {\n    let x = ~foo();\n})\n"
      ));
    }

    #[test]
    fn ignores_comment_lines() {
      assert!(!snippet_has_legacy_postfix_tilde(
        "// x ~ foo();\nlet y = ~bar();\n"
      ));
    }
  }
}
