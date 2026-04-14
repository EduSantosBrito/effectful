---
name: rust-production-engineering
description: Use for Rust build pipelines, cross-compilation, CI/CD, tooling, and validation (Miri, deny)—consult the Rust Engineering Practices book in the submodule
---

# Rust production engineering (RustTraining)

## Material

Use the **Rust Engineering Practices** book in the vendored submodule:

- **Root:** `.cursor/microsoft-rust-training/engineering-book/`
- **TOC:** `.cursor/microsoft-rust-training/engineering-book/src/SUMMARY.md`

Upstream: [microsoft/RustTraining](https://github.com/microsoft/RustTraining) (engineering-book). Prefer this skill together with **rust-microsoft-training** when the task spans both idioms and tooling.

## When to use

- Setting up or changing CI for Rust (checks, caching, multi-target builds).
- Cross-compilation, target triples, linker/toolchain issues.
- Build scripts (`build.rs`), feature flags, workspace layout affecting releases.
- Running or interpreting **Miri**, **cargo-deny**, or other “production readiness” checks discussed in the book.
- Documenting release or contributor setup for a Rust workspace.

## Workflow

1. Open `engineering-book/src/SUMMARY.md` and jump to the chapter that matches the failure or goal.
2. Align project scripts and CI with the practices there, adapted to this repository (e.g. moon/devenv if present).
3. Verify locally with the same commands you expect in CI.

## If the submodule is missing

```bash
git submodule update --init --recursive
```
