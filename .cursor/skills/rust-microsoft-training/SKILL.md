---
name: rust-microsoft-training
description: Use when writing, designing, or reviewing Rust code, or when learning Rust idioms—consult Microsoft RustTraining books in the repo before guessing patterns
---

# Rust via Microsoft RustTraining (curriculum skill)

## Source material (local)

This repository vendors **[microsoft/RustTraining](https://github.com/microsoft/RustTraining)** as a git submodule:

**Root:** `.cursor/microsoft-rust-training/`

Upstream describes it as training material, not an authoritative spec. **Cross-check** important details with [The Rust Book](https://doc.rust-lang.org/book/) and [The Rust Reference](https://doc.rust-lang.org/reference/).

## When to use this skill

- Implementing or refactoring non-trivial Rust (ownership, traits, error handling, FFI, generics).
- You are about to use `unsafe`, custom `Drop`, interior mutability, or advanced generics—**read the relevant book chapters first** in the submodule, then apply.
- Reviewing Rust for idiomatic style and soundness.
- Onboarding: mapping concepts from another language to Rust.

## Pick a book (route the task)

All books expose a table of contents at `*/src/SUMMARY.md` under the submodule root.

| Book (path under `.cursor/microsoft-rust-training/`) | Level | Use when |
| --- | --- | --- |
| `c-cpp-book/` | Bridge | Coming from C/C++; move semantics, RAII, FFI, `no_std`, embedded |
| `csharp-book/` | Bridge | Coming from C#/Java/Swift-style OOP → ownership and the type system |
| `python-book/` | Bridge | Dynamic → static typing; concurrency without a GIL mental model |
| `async-book/` | Deep dive | `async`/`.await`, Tokio, streams, cancellation |
| `rust-patterns-book/` | Advanced | `Pin`, allocators, lock-free ideas, `unsafe` patterns |
| `type-driven-correctness-book/` | Expert | Type-state, phantom types, encoding invariants in types |
| `engineering-book/` | Practices | Builds, cross-compilation, CI, Miri—see also **rust-production-engineering** skill |

## Workflow (do this in order)

1. **Classify** the problem: language bridge vs async vs patterns vs types vs tooling.
2. **Open** the matching book’s `src/SUMMARY.md` and identify 1–3 chapters that cover the topic.
3. **Read** those chapters (markdown under `src/`—follow links from `SUMMARY.md`).
4. **Apply** the guidance to the codebase: prefer patterns and APIs the book recommends; note any version drift (check your `Cargo.toml` vs book examples).
5. **Verify** with `cargo check`, `cargo test`, and clippy; for `unsafe` or concurrency, consider Miri when appropriate (engineering book).

## Anti-patterns

- Skipping the material and “winging” ownership, lifetimes, or `unsafe`.
- Copy-pasting snippets without matching them to your crate’s edition and dependency versions.
- Treating training prose as a substitute for `rustc` errors—use both.

## Submodule not present?

If `.cursor/microsoft-rust-training/` is missing, from the repo root run:

```bash
git submodule update --init --recursive
```
