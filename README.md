# Learn you an effect for great good!

> **effect-rs** — `Effect<A, E, R>` in Rust: structured effects, typed errors, and composable services. Heavily inspired by [Effect-TS](https://effect.website), gently explained so your brain does not melt.

So you’ve met `async fn`, `Pin<Box<dyn Future<...>>>`, and that one enum with *seventeen* error variants. You’ve passed the same `Arc<Config>` through fourteen layers of call stack because “that’s how we do it.” You’re fine. Everything is fine.

This repository is a **playful guide’s** older sibling: the actual library and friends. The book (*A Playful Guide to Typed Effects in Rust*) lives in the same spirit as *Learn You a Haskell* — friendly, a little silly, serious about correctness. The code here is the part you `cargo add`.

<!-- Badge row 1: CI & quality -->
[![CI](https://github.com/Industrial/effect-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Industrial/effect-rs/actions/workflows/ci.yml)
[![Docs & Pages](https://github.com/Industrial/effect-rs/actions/workflows/docs.yml/badge.svg)](https://github.com/Industrial/effect-rs/actions/workflows/docs.yml)
[![Security Audit](https://github.com/Industrial/effect-rs/actions/workflows/audit.yml/badge.svg)](https://github.com/Industrial/effect-rs/actions/workflows/audit.yml)
[![codecov](https://codecov.io/gh/Industrial/effect-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/Industrial/effect-rs)

<!-- Badge row 2: crates.io -->
[![crates.io](https://img.shields.io/crates/v/effect-rs.svg)](https://crates.io/crates/effect-rs)
[![docs.rs](https://docs.rs/effect-rs/badge.svg)](https://docs.rs/effect-rs)
[![downloads](https://img.shields.io/crates/d/effect-rs.svg)](https://crates.io/crates/effect-rs)

<!-- Badge row 3: repository health -->
[![License](https://img.shields.io/badge/license-CC--BY--SA--4.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![GitHub Stars](https://img.shields.io/github/stars/Industrial/effect-rs?style=social)](https://github.com/Industrial/effect-rs/stargazers)

---

## What even is this?

**effect-rs** is the Rusty cousin of the Effect-TS programming model. Instead of “run this I/O *right now*,” you mostly build **descriptions** of work — lazy `Effect` values — and only *later* hand them to a runtime. That sounds like extra ceremony until you notice:

- **`Effect<A, E, R>`** — success `A`, failure `E`, environment `R`. Three type parameters, one honest story about your program.
- **Context / layers** — dependencies as a typed map: wire once, stop threading `Arc<Everything>` through every function signature.
- **`pipe!` and `effect!`** — do-notation-shaped composition without pretending you’re writing Haskell (you’re not; that’s okay).
- **Streams, STM, schema** — pull-based streams, software transactional memory, and structural validation when you’re ready to get fancy.
- **No bundled executor** — bring your own async runtime (Tokio and friends live in integration crates below). The core stays portable and interpreter-flavored.

If you like your errors typed, your resources scoped, and your refactors caught at compile time, you’re in the right barn.

---

## Crates in this workspace

| Crate | Version | Description |
|-------|---------|-------------|
| [`effect-rs`](crates/effect-rs) | [![crates.io](https://img.shields.io/crates/v/effect-rs.svg)](https://crates.io/crates/effect-rs) | Core: `Effect`, `pipe!`, `effect!`, context, schema, STM, … |
| [`effect-rs-macro`](crates/effect-rs-macro) | [![crates.io](https://img.shields.io/crates/v/effect-rs-macro.svg)](https://crates.io/crates/effect-rs-macro) | Declarative macros (`ctx!`, `pipe!`, …) |
| [`effect-rs-proc-macro`](crates/effect-rs-proc-macro) | [![crates.io](https://img.shields.io/crates/v/effect-rs-proc-macro.svg)](https://crates.io/crates/effect-rs-proc-macro) | Procedural `effect!` macro |
| [`effect-rs-tokio`](crates/effect-rs-tokio) | [![crates.io](https://img.shields.io/crates/v/effect-rs-tokio.svg)](https://crates.io/crates/effect-rs-tokio) | Tokio runtime adapter |
| [`effect-rs-axum`](crates/effect-rs-axum) | [![crates.io](https://img.shields.io/crates/v/effect-rs-axum.svg)](https://crates.io/crates/effect-rs-axum) | Axum integration |
| [`effect-rs-logger`](crates/effect-rs-logger) | [![crates.io](https://img.shields.io/crates/v/effect-rs-logger.svg)](https://crates.io/crates/effect-rs-logger) | Logging service (tracing backend) |
| [`effect-rs-config`](crates/effect-rs-config) | [![crates.io](https://img.shields.io/crates/v/effect-rs-config.svg)](https://crates.io/crates/effect-rs-config) | `ConfigProvider` + Figment/serde layers |
| [`effect-rs-reqwest`](crates/effect-rs-reqwest) | [![crates.io](https://img.shields.io/crates/v/effect-rs-reqwest.svg)](https://crates.io/crates/effect-rs-reqwest) | HTTP via reqwest |
| [`effect-rs-tower`](crates/effect-rs-tower) | [![crates.io](https://img.shields.io/crates/v/effect-rs-tower.svg)](https://crates.io/crates/effect-rs-tower) | Tower `Service` bridge |

---

## Your first effect (it’s allowed to be tiny)

Add the crate:

```toml
[dependencies]
effect-rs = "0.1"
```

Then greet the world like a civilized program:

```rust
use effect::Effect;

fn greet(name: &str) -> Effect<String, (), ()> {
    Effect::succeed(format!("Hello, {name}!"))
}

fn main() {
    let result = greet("world").run_sync(());
    println!("{result:?}");
}
```

That’s it. No monad sermon yet. For a guided tour in order, poke the numbered examples under [`crates/effect-rs/examples/`](crates/effect-rs/examples/). For prose, diagrams, and the occasional joke, open the [mdBook](https://industrial.github.io/effect-rs/).

---

## Where to read things

| Resource | Link |
|----------|------|
| API reference | [docs.rs/effect-rs](https://docs.rs/effect-rs) |
| mdBook (playful guide) | [industrial.github.io/effect-rs](https://industrial.github.io/effect-rs/) |
| Examples | [`crates/effect-rs/examples/`](crates/effect-rs/examples/) |

---

## Hacking on this repo

We use [devenv](https://devenv.sh) (Nix-flavored). Wrap commands so the elves find your compilers:

```bash
devenv shell -- <command>
```

Common [Moon](https://moonrepo.dev) tasks:

```bash
# Format
devenv shell -- moon run :format

# Check + clippy
devenv shell -- moon run :clippy

# Tests (nextest)
devenv shell -- moon run :test

# Coverage (95% threshold — yes, we’re a little intense)
devenv shell -- moon run :coverage

# Build
devenv shell -- moon run :build

# Run examples for a crate
devenv shell -- moon run effect-rs-lib:examples

# Security audit
devenv shell -- moon run :audit

# API docs + mdBook
devenv shell -- moon run :docs :book

# Pre-push “did I break the universe?” bundle
devenv shell -- moon run :format :check :build :test :coverage :audit :check-docs
```

### Continuous integration

| Workflow | Triggers | What it does |
|----------|----------|--------------|
| [CI](https://github.com/Industrial/effect-rs/actions/workflows/ci.yml) | push/PR → `main` | Format, check, clippy, test, build, coverage, doc-check, matrix (stable+beta × linux+mac+win) |
| [Docs & Pages](https://github.com/Industrial/effect-rs/actions/workflows/docs.yml) | push/PR → `main` | API docs + mdBook → GitHub Pages |
| [Security Audit](https://github.com/Industrial/effect-rs/actions/workflows/audit.yml) | daily + `Cargo.lock` changes | `cargo audit` |
| [Publish](https://github.com/Industrial/effect-rs/actions/workflows/publish.yml) | `v*.*.*` tag | Test, then publish crates in dependency order |

### Shipping a release

```bash
# 1. Bump versions in the workspace Cargo.toml files
# 2. Commit and push
# 3. Tag and push — automation does the rest
git tag v0.2.0
git push --tags
```

> **Secrets:** `CARGO_REGISTRY_TOKEN` for crates.io; `CODECOV_TOKEN` optional for coverage uploads. Publishing uses a `crates-io` GitHub environment if you want human gates on the shiny red button.

---

## Coverage

[![codecov](https://codecov.io/gh/Industrial/effect-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/Industrial/effect-rs)

CI expects **≥ 95%** lines, regions, and functions via `cargo llvm-cov nextest`. Green is happy; red is a teaching moment.

![Coverage sunburst](https://codecov.io/gh/Industrial/effect-rs/branch/main/graphs/sunburst.svg)

---

## Star history

[![Star History Chart](https://api.star-history.com/svg?repos=Industrial/effect-rs&type=Date)](https://star-history.com/#Industrial/effect-rs&Date)

---

## Contributors

Huge thanks to everyone who’s sent patches, opened issues, or stared at a type error until it confessed.

[![Contributors](https://contrib.rocks/image?repo=Industrial/effect-rs)](https://github.com/Industrial/effect-rs/graphs/contributors)

---

## License

This project is licensed under [CC-BY-SA-4.0](LICENSE).
