# effect-rs

> `Effect<A, E, R>` for Rust — structured effects, typed errors, and composable services inspired by [Effect-TS](https://effect.website).

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

## What is effect-rs?

**effect-rs** is a Rust library that brings the [Effect-TS](https://effect.website) programming model to Rust. It provides:

- **`Effect<A, E, R>`** — a lazy, composable value that describes a computation returning `A`, failing with `E`, and requiring services from environment `R`.
- **Context / Layers** — dependency injection via typed service maps; wire once, run anywhere.
- **`pipe!` / `effect!`** macros — ergonomic, do-notation-style effect composition.
- **Streams** — async pull-based streams as effects.
- **STM** — software transactional memory via effect-native transactions.
- **Schema** — structural type-safe value descriptions (with optional `serde_json`).
- **Zero bundled executor** — bring Tokio, async-std, or a custom executor via integration crates.

---

## Crates in this workspace

| Crate | Version | Description |
|-------|---------|-------------|
| [`effect-rs`](crates/effect-rs) | [![crates.io](https://img.shields.io/crates/v/effect-rs.svg)](https://crates.io/crates/effect-rs) | Core library: `Effect`, `pipe!`, `effect!`, Context, Schema, STM |
| [`effect-rs-macro`](crates/effect-rs-macro) | [![crates.io](https://img.shields.io/crates/v/effect-rs-macro.svg)](https://crates.io/crates/effect-rs-macro) | Declarative macros (`ctx!`, `pipe!`, …) |
| [`effect-rs-proc-macro`](crates/effect-rs-proc-macro) | [![crates.io](https://img.shields.io/crates/v/effect-rs-proc-macro.svg)](https://crates.io/crates/effect-rs-proc-macro) | Procedural `effect!` macro |
| [`effect-rs-tokio`](crates/effect-rs-tokio) | [![crates.io](https://img.shields.io/crates/v/effect-rs-tokio.svg)](https://crates.io/crates/effect-rs-tokio) | Tokio runtime adapter |
| [`effect-rs-axum`](crates/effect-rs-axum) | [![crates.io](https://img.shields.io/crates/v/effect-rs-axum.svg)](https://crates.io/crates/effect-rs-axum) | Axum web framework integration |
| [`effect-rs-logger`](crates/effect-rs-logger) | [![crates.io](https://img.shields.io/crates/v/effect-rs-logger.svg)](https://crates.io/crates/effect-rs-logger) | Effect logging service (tracing backend) |
| [`effect-rs-config`](crates/effect-rs-config) | [![crates.io](https://img.shields.io/crates/v/effect-rs-config.svg)](https://crates.io/crates/effect-rs-config) | ConfigProvider + Figment/serde layers |
| [`effect-rs-reqwest`](crates/effect-rs-reqwest) | [![crates.io](https://img.shields.io/crates/v/effect-rs-reqwest.svg)](https://crates.io/crates/effect-rs-reqwest) | HTTP client integration (reqwest) |
| [`effect-rs-tower`](crates/effect-rs-tower) | [![crates.io](https://img.shields.io/crates/v/effect-rs-tower.svg)](https://crates.io/crates/effect-rs-tower) | Tower `Service` bridge |

---

## Quick start

Add `effect-rs` to your `Cargo.toml`:

```toml
[dependencies]
effect-rs = "0.1"
```

A minimal program:

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

See the numbered examples under [`crates/effect-rs/examples/`](crates/effect-rs/examples/) for a guided curriculum, and the [mdBook](https://industrial.github.io/effect-rs/) for full documentation.

---

## Documentation

| Resource | Link |
|----------|------|
| API reference | [docs.rs/effect-rs](https://docs.rs/effect-rs) |
| mdBook guide | [industrial.github.io/effect-rs](https://industrial.github.io/effect-rs/) |
| Examples | [`crates/effect-rs/examples/`](crates/effect-rs/examples/) |

---

## Development

This project uses [devenv](https://devenv.sh) (Nix-based). All shell commands should be run inside devenv:

```bash
devenv shell -- <command>
```

Common tasks (via [Moon](https://moonrepo.dev)):

```bash
# Format
devenv shell -- moon run :format

# Check + clippy
devenv shell -- moon run :clippy

# Run tests (nextest)
devenv shell -- moon run :test

# Coverage (95% threshold)
devenv shell -- moon run :coverage

# Build
devenv shell -- moon run :build

# Run examples for a crate
devenv shell -- moon run effect-rs-lib:examples

# Security audit
devenv shell -- moon run :audit

# Build docs + book
devenv shell -- moon run :docs :book

# Full pre-push gate
devenv shell -- moon run :format :check :build :test :coverage :audit :check-docs
```

### Continuous Integration

| Workflow | Triggers | What it does |
|----------|----------|--------------|
| [CI](https://github.com/Industrial/effect-rs/actions/workflows/ci.yml) | push/PR → `main` | Format, check, clippy, test, build, coverage, doc-check, matrix (stable+beta × linux+mac+win) |
| [Docs & Pages](https://github.com/Industrial/effect-rs/actions/workflows/docs.yml) | push/PR → `main` | Build API docs + mdBook, deploy to GitHub Pages |
| [Security Audit](https://github.com/Industrial/effect-rs/actions/workflows/audit.yml) | daily + Cargo.lock changes | `cargo audit` |
| [Publish](https://github.com/Industrial/effect-rs/actions/workflows/publish.yml) | `v*.*.*` tag push | Test then publish all crates to crates.io in dependency order |

### Publishing a release

```bash
# 1. Bump all crate versions in Cargo.toml files
# 2. Commit and push
# 3. Tag and push
git tag v0.2.0
git push --tags
# The Publish workflow fires automatically.
```

> **Required secrets:** `CARGO_REGISTRY_TOKEN` (crates.io API token), `CODECOV_TOKEN` (optional, for coverage uploads).  
> **Required environment:** create a `crates-io` GitHub environment with the publish secret to enforce approvals.

---

## Coverage

[![codecov](https://codecov.io/gh/Industrial/effect-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/Industrial/effect-rs)

The CI enforces **≥ 95%** lines, regions, and functions coverage via `cargo llvm-cov nextest`.

![Coverage sunburst](https://codecov.io/gh/Industrial/effect-rs/branch/main/graphs/sunburst.svg)

---

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Industrial/effect-rs&type=Date)](https://star-history.com/#Industrial/effect-rs&Date)

---

## Contributors

Thanks to everyone who has contributed to effect-rs!

[![Contributors](https://contrib.rocks/image?repo=Industrial/effect-rs)](https://github.com/Industrial/effect-rs/graphs/contributors)

---

## License

This project is licensed under the [CC-BY-SA-4.0](LICENSE) license.
