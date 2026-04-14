# effect-rs-dylint-rules

[Dylint](https://github.com/trailofbits/dylint) dynamic library: custom rustc lints for **Effect.rs** conventions (see `.cursor/skills/effect.rs-fundamentals/SKILL.md`).

This crate is **not** a normal workspace member: it uses a **pinned nightly** and **`rustc-dev`**, which the main repo’s stable Nix/devenv toolchain does not provide. It lives in its own `[workspace]` under `crates/effect-rs-dylint-rules/` and is listed in the root `Cargo.toml` under `[workspace.exclude]`.

## In this repo (devenv / Nix, no rustup)

- **Fenix nightly** matching `rust-toolchain` is provided as **`EFFECT_DYLINT_TOOLCHAIN`** from `devenv.nix` (components: `rustc-dev`, `llvm-tools-preview`, `rust-src`, etc.).
- **`cargo-dylint`** and **`dylint-link`** are installed once into **`.devenv/state/dylint-cli`** on shell entry (`cargo install` from the upstream `v5.0.0` git tag; the crates.io tarball layout does not satisfy Dylint’s driver layout).
- **`moon … :lint`** and **`scripts/moon-dylint.sh`** build this crate with that toolchain, set **`DYLINT_LIBRARY_PATH`** and **`DYLINT_DRIVER_PATH`**, use **`cargo dylint --no-metadata`**, and prepend **`scripts/dylint-rustup-shim`** so Dylint’s internal `rustup which rustc` / `rustup +stable which cargo` calls work without installing rustup.

## Manual build (optional)

```bash
# From devenv shell so EFFECT_DYLINT_TOOLCHAIN is set:
export PATH="$EFFECT_DYLINT_TOOLCHAIN/bin:$PATH"
export RUSTUP_TOOLCHAIN="nightly-2025-09-18-$(rustc -vV | sed -n 's/^host: //p')"
cd crates/effect-rs-dylint-rules
cargo build --release
```

`rust-toolchain` pins the nightly used against rustc’s private API.

## Run lints from the repo root (manual)

Prefer **`moon run <crate>:lint`**. For ad-hoc use, mirror `scripts/moon-dylint.sh`: same `PATH`, `RUSTUP_TOOLCHAIN`, `DYLINT_LIBRARY_PATH`, shim, then e.g.:

```bash
cargo dylint --no-metadata --all -p effect-rs
```

Lints default to **`deny`** when this library is loaded via **`cargo dylint`** / Moon **`:lint`**. Normal `cargo check` / `cargo clippy` without Dylint does not load them. Use targeted **`#[allow(lint_name)]`** only where the Effect.rs skill documents an exception.

## Lints

Passes:

| Pass | Crates | Notes |
|------|--------|--------|
| `EffectStyleLate` | All **except** names starting with `forge_` (and macro/logger helper crates per code) | Compositional `Effect` style: `effect!`, generics, `run_*` boundaries, `async` policy. |
| `EffectInteropLate` | All application crates; skipped only for `effect`, `effect_config`, `effect_logger`, `effect_macro`, `effect_proc_macro`, `effect_dylint_rules` | DI and platform escape hatches: raw `Get`, `Effect::new`, `Instant::now`, legacy postfix `~`. |
| `EffectTracingEarly` | Same exclusions as style lint (non-`forge_`, etc.) | Forbids `tracing::…` macros. |

### `EffectStyleLate`

| Lint | Default | Meaning |
|------|---------|--------|
| `effect_success_should_be_fn_type_param` | deny | If a function returns `effect::Effect<A, E, R>`, `A` should be one of that function’s type parameters (not a fixed concrete success type). Skips `async fn` and closures. |
| `effect_no_effect_suffix_on_graph_builder` | deny | No `_effect` suffix on functions that return `Effect`; the return type already signals it. |
| `effect_run_blocking_outside_boundary` | deny | `run_blocking` only in `main`, tests, examples, benches (not arbitrary library code). |
| `effect_run_async_outside_boundary` | deny | Same for `run_async`. |
| `effect_prefer_from_async_over_new_async` | deny | Prefer `from_async` over `Effect::new_async` except when `&mut R` crosses `.await`. |
| `effect_from_async_single_await` | deny | Heuristic: the snippet passed to `from_async` should contain at most one `.await`. |
| `effect_effect_generics_need_bounds` | deny | Every type parameter on an `Effect`-returning function must have bounds (HIR predicates). |
| `effect_multiple_top_level_effect_macros` | deny | At most one top-level `effect!(…)` per function body (source heuristic). |
| `effect_returning_effect_should_use_effect_macro` | deny | `Effect`-returning functions must use a single top-level `effect!(…)` (source must contain `effect!(`); no `succeed`/`pipe!`-only bodies. |
| `effect_no_async_fn_application` | deny | No `async fn` for app logic; integration crates may `#[allow]`. |

### `EffectInteropLate`

| Lint | Default | Meaning |
|------|---------|--------|
| `effect_no_effect_new_outside_effect_crate` | deny | Do not call `Effect::new` outside the `effect` crate; use `succeed` / `fail` / `from_async` / combinators. Same boundary exceptions as `run_blocking` (harness / `main` / `*_blocking`). |
| `effect_where_uses_raw_get_trait` | deny | User-written bounds that use `effect::context::Get` or `GetMut` should be replaced with a project `Needs…` supertrait at public APIs (`explicit_predicates_of` only). |
| `effect_no_instant_now_outside_boundary` | deny | `std::time::Instant::now` only in the same boundary as `run_*` (inject `Clock` / `TestClock` for library code). |
| `effect_legacy_postfix_tilde` | deny | Obsolete `x ~ expr` inside `effect!` bodies; use `let x = ~expr;`. |

### `EffectTracingEarly`

| Lint | Default | Meaning |
|------|---------|--------|
| `effect_no_raw_tracing` | deny | No `tracing::…` macros; use `effect_logger` inside `effect!`. |

## Toolchain bumps

When you change `rust-toolchain`, update the Fenix `toolchainOf` date/hash in **`devenv.nix`** and the `DYLINT_CHANNEL_DATE` line in **`scripts/moon-dylint.sh`** so they stay aligned. Rebuild this crate. You may need a matching **`dylint_linting`** / **`cargo-dylint`** version; see the [Dylint changelog](https://github.com/trailofbits/dylint/blob/master/CHANGELOG.md).
