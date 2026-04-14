# effect_rs

**effect_rs** brings [`Effect<A, E, R>`](https://docs.rs/effect_rs): structured effects, typed errors, and composable services in Rust. The design is heavily inspired by [Effect](https://effect.website) (Effect-TS): programs describe work as lazy values, wire dependencies through the type system, and run under an explicit runtime.

If you have written async Rust—`Future`s, `Pin`, long `.await?` chains—and want clearer dependency boundaries, recoverable errors, and tests that do not rely on global mocks, this workspace is for you.

<!-- Badge row 1: CI & quality -->
[![CI](https://github.com/Industrial/effect_rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Industrial/effect_rs/actions/workflows/ci.yml)
[![Docs & Pages](https://github.com/Industrial/effect_rs/actions/workflows/docs.yml/badge.svg)](https://github.com/Industrial/effect_rs/actions/workflows/docs.yml)
[![Security Audit](https://github.com/Industrial/effect_rs/actions/workflows/audit.yml/badge.svg)](https://github.com/Industrial/effect_rs/actions/workflows/audit.yml)
[![codecov](https://codecov.io/gh/Industrial/effect_rs/branch/main/graph/badge.svg)](https://codecov.io/gh/Industrial/effect_rs)

<!-- Badge row 2: crates.io -->
[![crates.io](https://img.shields.io/crates/v/effect_rs.svg)](https://crates.io/crates/effect_rs)
[![docs.rs](https://docs.rs/effect_rs/badge.svg)](https://docs.rs/effect_rs)
[![downloads](https://img.shields.io/crates/d/effect_rs.svg)](https://crates.io/crates/effect_rs)

<!-- Badge row 3: repository health -->
[![License](https://img.shields.io/badge/license-CC--BY--SA--4.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![GitHub Stars](https://img.shields.io/github/stars/Industrial/effect_rs?style=social)](https://github.com/Industrial/effect_rs/stargazers)

---

## Overview

**effect_rs** models programs as **descriptions** of work (`Effect` values) rather than immediate side effects. You compose them, attach requirements to the type signature, and run them when you choose. That buys:

- **`Effect<A, E, R>`** — success type `A`, error type `E`, environment/requirements `R`.
- **Context and layers** — typed dependency injection: declare what an effect needs, provide it once at the edge.
- **`pipe!` and `effect!`** — ergonomic composition and do-notation-style blocks without hiding what the types mean.
- **Streams, STM, schema** — pull-based streams, software transactional memory, and structural validation for larger systems.
- **No bundled async executor** — the core stays portable; Tokio and other runtimes live in separate integration crates (see below).

The long-form guide is the mdBook [**Typed Effects in Rust**](https://industrial.github.io/effect_rs/) in [`crates/effect_rs/book/`](crates/effect_rs/book/). It follows the same arc as the library: foundations, environment and dependencies, production concerns (errors, concurrency, resources, scheduling), then advanced topics (STM, streams, schema, testing).

---

## Crates in this workspace

| Crate | Version | Description |
|-------|---------|-------------|
| [`effect_rs`](crates/effect_rs) | [![crates.io](https://img.shields.io/crates/v/effect_rs.svg)](https://crates.io/crates/effect_rs) | Core: `Effect`, `pipe!`, `effect!`, context, schema, STM, … |
| [`effect_rs_macro`](crates/effect_rs_macro) | [![crates.io](https://img.shields.io/crates/v/effect_rs_macro.svg)](https://crates.io/crates/effect_rs_macro) | Declarative macros (`ctx!`, `pipe!`, …) |
| [`effect_rs_proc_macro`](crates/effect_rs_proc_macro) | [![crates.io](https://img.shields.io/crates/v/effect_rs_proc_macro.svg)](https://crates.io/crates/effect_rs_proc_macro) | Procedural `effect!` macro |
| [`effect_rs_tokio`](crates/effect_rs_tokio) | [![crates.io](https://img.shields.io/crates/v/effect_rs_tokio.svg)](https://crates.io/crates/effect_rs_tokio) | Tokio runtime adapter |
| [`effect_rs_axum`](crates/effect_rs_axum) | [![crates.io](https://img.shields.io/crates/v/effect_rs_axum.svg)](https://crates.io/crates/effect_rs_axum) | Axum integration |
| [`effect_rs_logger`](crates/effect_rs_logger) | [![crates.io](https://img.shields.io/crates/v/effect_rs_logger.svg)](https://crates.io/crates/effect_rs_logger) | Logging service (tracing backend) |
| [`effect_rs_config`](crates/effect_rs_config) | [![crates.io](https://img.shields.io/crates/v/effect_rs_config.svg)](https://crates.io/crates/effect_rs_config) | `ConfigProvider` + Figment/serde layers |
| [`effect_rs_reqwest`](crates/effect_rs_reqwest) | [![crates.io](https://img.shields.io/crates/v/effect_rs_reqwest.svg)](https://crates.io/crates/effect_rs_reqwest) | HTTP via reqwest |
| [`effect_rs_tower`](crates/effect_rs_tower) | [![crates.io](https://img.shields.io/crates/v/effect_rs_tower.svg)](https://crates.io/crates/effect_rs_tower) | Tower `Service` bridge |

---

## Minimal example

Add the crate:

```toml
[dependencies]
effect_rs = "0.1"
```

```rust
use effect_rs::Effect;

fn greet(name: &str) -> Effect<String, (), ()> {
    Effect::succeed(format!("Hello, {name}!"))
}

fn main() {
    let result = greet("world").run_sync(());
    println!("{result:?}");
}
```

For a guided path through the API, use the numbered examples under [`crates/effect_rs/examples/`](crates/effect_rs/examples/) and the mdBook linked below.

---

## Documentation

| Resource | Link |
|----------|------|
| API reference | [docs.rs/effect_rs](https://docs.rs/effect_rs) |
| mdBook (*Typed Effects in Rust*) | [industrial.github.io/effect_rs](https://industrial.github.io/effect_rs/) |
| Examples | [`crates/effect_rs/examples/`](crates/effect_rs/examples/) |

---

## Development

This repository uses [devenv](https://devenv.sh). Run commands inside the dev shell:

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

# Coverage (95% threshold)
devenv shell -- moon run :coverage

# Build
devenv shell -- moon run :build

# Run examples for a crate
devenv shell -- moon run effect_rs_lib:examples

# Security audit
devenv shell -- moon run :audit

# API docs + mdBook
devenv shell -- moon run :docs :book

# Pre-push checks
devenv shell -- moon run :format :check :build :test :coverage :audit :check-docs
```

### Continuous integration

| Workflow | Triggers | What it does |
|----------|----------|--------------|
| [CI](https://github.com/Industrial/effect_rs/actions/workflows/ci.yml) | push/PR → `main` | Format, check, clippy, test, build, coverage, doc-check, matrix (stable+beta × linux+mac+win) |
| [Docs & Pages](https://github.com/Industrial/effect_rs/actions/workflows/docs.yml) | push/PR → `main` | API docs + mdBook → GitHub Pages |
| [Security Audit](https://github.com/Industrial/effect_rs/actions/workflows/audit.yml) | daily + `Cargo.lock` changes | `cargo audit` |
| [Publish](https://github.com/Industrial/effect_rs/actions/workflows/publish.yml) | `v*.*.*` tag | Test, then publish crates in dependency order |

### Releases

```bash
# 1. Bump versions in the workspace Cargo.toml files
# 2. Commit and push
# 3. Tag and push — CI publishes
git tag v0.2.0
git push --tags
```

Publishing expects `CARGO_REGISTRY_TOKEN` on crates.io; `CODECOV_TOKEN` is optional for coverage uploads. The publish workflow can use a `crates-io` GitHub environment for approval gates.

---

## Coverage

[![codecov](https://codecov.io/gh/Industrial/effect_rs/branch/main/graph/badge.svg)](https://codecov.io/gh/Industrial/effect_rs)

CI enforces **≥ 95%** lines, regions, and functions via `cargo llvm-cov nextest`.

![Coverage sunburst](https://codecov.io/gh/Industrial/effect_rs/branch/main/graphs/sunburst.svg)

---

## Star history

[![Star History Chart](https://api.star-history.com/svg?repos=Industrial/effect_rs&type=Date)](https://star-history.com/#Industrial/effect_rs&Date)

---

## Contributors

Thanks to everyone who has contributed patches, reported issues, or improved the types.

[![Contributors](https://contrib.rocks/image?repo=Industrial/effect_rs)](https://github.com/Industrial/effect_rs/graphs/contributors)

---

## License

This project is licensed under [CC-BY-SA-4.0](LICENSE).
