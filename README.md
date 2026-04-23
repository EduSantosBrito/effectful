# effectful

**effectful** brings [`Effect<A, E, R>`](https://docs.rs/effectful): structured effects, typed errors, and composable services in Rust. The design is heavily inspired by [Effect](https://effect.website) (Effect-TS): programs describe work as lazy values, wire dependencies through the type system, and run under an explicit runtime.

If you have written async Rust—`Future`s, `Pin`, long `.await?` chains—and want clearer dependency boundaries, recoverable errors, and tests that do not rely on global mocks, this workspace is for you.

<!-- Badge row 1: CI & quality -->
[![CI](https://github.com/EduSantosBrito/effectful/actions/workflows/ci.yml/badge.svg)](https://github.com/EduSantosBrito/effectful/actions/workflows/ci.yml)
[![Docs & Pages](https://github.com/EduSantosBrito/effectful/actions/workflows/docs.yml/badge.svg)](https://github.com/EduSantosBrito/effectful/actions/workflows/docs.yml)
[![Security Audit](https://github.com/EduSantosBrito/effectful/actions/workflows/audit.yml/badge.svg)](https://github.com/EduSantosBrito/effectful/actions/workflows/audit.yml)
[![codecov](https://codecov.io/gh/Industrial/id_effect/branch/main/graph/badge.svg)](https://codecov.io/gh/Industrial/id_effect)

<!-- Badge row 2: crates.io -->
[![crates.io](https://img.shields.io/crates/v/effectful.svg)](https://crates.io/crates/effectful)
[![docs.rs](https://docs.rs/effectful/badge.svg)](https://docs.rs/effectful)
[![downloads](https://img.shields.io/crates/d/effectful.svg)](https://crates.io/crates/effectful)

<!-- Badge row 3: repository health -->
[![License](https://img.shields.io/badge/license-CC--BY--SA--4.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![GitHub Stars](https://img.shields.io/github/stars/Industrial/effectful?style=social)](https://github.com/EduSantosBrito/effectful/stargazers)

### Start with the book

**[Typed Effects in Rust](https://industrial.github.io/effectful/)** is the main way to learn **effectful**: it walks the same story as the library—from `Effect<A, E, R>` and the `effect!` macro, through context, layers, and services, to concurrency, resources, STM, streams, schema, and testing. The API docs answer “what does this type do?”; the book answers “how do I think in effects?” and ties the pieces together.

- **Read online:** [Typed Effects in Rust](https://industrial.github.io/effectful/) (GitHub Pages)
- **Terminology:** [Glossary](https://industrial.github.io/effectful/appendix-c-glossary.html) — quick definitions for `Effect`, `Cause`, `Layer`, `Fiber`, `Stm`, and the rest of the vocabulary
- **Source:** [`crates/effectful/book/`](crates/effectful/book/) (build locally with `moon run :book`)

---

## Overview

**effectful** models programs as **descriptions** of work (`Effect` values) rather than immediate side effects. You compose them, attach requirements to the type signature, and run them when you choose. That buys:

- **`Effect<A, E, R>`** — success type `A`, error type `E`, environment/requirements `R`.
- **Context and layers** — typed dependency injection: declare what an effect needs, provide it once at the edge.
- **`pipe!` and `effect!`** — ergonomic composition and do-notation-style blocks without hiding what the types mean.
- **Streams, STM, schema** — pull-based streams, software transactional memory, and structural validation for larger systems.
- **No bundled async executor** — the core stays portable; Tokio and other runtimes live in separate integration crates (see below).

For depth beyond this README, use the mdBook [**Typed Effects in Rust**](https://industrial.github.io/effectful/) (see **Start with the book** above). It follows the same arc as the library: foundations, environment and dependencies, production concerns (errors, concurrency, resources, scheduling), then advanced topics (STM, streams, schema, testing).

---

## Crates in this workspace

| Crate | Version | Description |
|-------|---------|-------------|
| [`effectful`](crates/effectful) | [![crates.io](https://img.shields.io/crates/v/effectful.svg)](https://crates.io/crates/effectful) | Core: `Effect`, `pipe!`, `effect!`, context, schema, STM, … |
| [`effectful_macro`](crates/effectful_macro) | [![crates.io](https://img.shields.io/crates/v/effectful_macro.svg)](https://crates.io/crates/effectful_macro) | Declarative macros (`ctx!`, `pipe!`, …) |
| [`effectful_proc_macro`](crates/effectful_proc_macro) | [![crates.io](https://img.shields.io/crates/v/effectful_proc_macro.svg)](https://crates.io/crates/effectful_proc_macro) | Procedural `effect!` macro |
| [`effectful_tokio`](crates/effectful_tokio) | [![crates.io](https://img.shields.io/crates/v/effectful_tokio.svg)](https://crates.io/crates/effectful_tokio) | Tokio runtime adapter |
| [`effectful_axum`](crates/effectful_axum) | [![crates.io](https://img.shields.io/crates/v/effectful_axum.svg)](https://crates.io/crates/effectful_axum) | Axum integration |
| [`effectful_logger`](crates/effectful_logger) | [![crates.io](https://img.shields.io/crates/v/effectful_logger.svg)](https://crates.io/crates/effectful_logger) | Logging service (tracing backend) |
| [`effectful_config`](crates/effectful_config) | [![crates.io](https://img.shields.io/crates/v/effectful_config.svg)](https://crates.io/crates/effectful_config) | `ConfigProvider` + Figment/serde layers |
| [`effectful_reqwest`](crates/effectful_reqwest) | [![crates.io](https://img.shields.io/crates/v/effectful_reqwest.svg)](https://crates.io/crates/effectful_reqwest) | HTTP via reqwest |
| [`effectful_tower`](crates/effectful_tower) | [![crates.io](https://img.shields.io/crates/v/effectful_tower.svg)](https://crates.io/crates/effectful_tower) | Tower `Service` bridge |
| [`effectful_lint`](crates/effectful_lint) | (nightly only) | Custom rustc lints for effectful code |

---

## Minimal example

Add the crate:

```toml
[dependencies]
effectful = "0.1"
```

```rust
use effectful::Effect;

fn greet(name: &str) -> Effect<String, (), ()> {
    Effect::succeed(format!("Hello, {name}!"))
}

fn main() {
    let result = greet("world").run_sync(());
    println!("{result:?}");
}
```

For a guided path through the API, read [**Typed Effects in Rust**](https://industrial.github.io/effectful/) first, then use the numbered examples under [`crates/effectful/examples/`](crates/effectful/examples/) and [docs.rs](https://docs.rs/effectful).

---

## Documentation

| Resource | Link |
|----------|------|
| **Book (primary learning path)** | [**Typed Effects in Rust**](https://industrial.github.io/effectful/) — [glossary](https://industrial.github.io/effectful/appendix-c-glossary.html) |
| API reference | [docs.rs/effectful](https://docs.rs/effectful) |
| Examples | [`crates/effectful/examples/`](crates/effectful/examples/) |

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
devenv shell -- moon run effectful_lib:examples

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
| [CI](https://github.com/EduSantosBrito/effectful/actions/workflows/ci.yml) | push/PR → `main` | Format, check, clippy, test, build, coverage, doc-check, matrix (stable+beta × linux+mac+win) |
| [Docs & Pages](https://github.com/EduSantosBrito/effectful/actions/workflows/docs.yml) | push/PR → `main` | API docs + mdBook → GitHub Pages |
| [Security Audit](https://github.com/EduSantosBrito/effectful/actions/workflows/audit.yml) | daily + `Cargo.lock` changes | `cargo audit` |
| [Publish](https://github.com/EduSantosBrito/effectful/actions/workflows/publish.yml) | `v*.*.*` tag | Test, then publish crates in dependency order |

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

[![codecov](https://codecov.io/gh/Industrial/id_effect/branch/main/graph/badge.svg)](https://codecov.io/gh/Industrial/id_effect)

CI enforces **≥ 95%** lines, regions, and functions via `cargo llvm-cov nextest`.

![Coverage sunburst](https://codecov.io/gh/Industrial/id_effect/branch/main/graphs/sunburst.svg)

---

## Star history

[![Star History Chart](https://api.star-history.com/svg?repos=Industrial/effectful&type=Date)](https://star-history.com/#EduSantosBrito/effectful&Date)

---

## Contributors

Thanks to everyone who has contributed patches, reported issues, or improved the types.

[![Contributors](https://contrib.rocks/image?repo=EduSantosBrito/effectful)](https://github.com/EduSantosBrito/effectful/graphs/contributors)

---

## License

This project is a fork and adaptation of [`Industrial/id_effect`](https://github.com/Industrial/id_effect), which is licensed under the [Creative Commons Attribution-ShareAlike 4.0 International License](https://creativecommons.org/licenses/by-sa/4.0/) (`CC-BY-SA-4.0`).

Changes have been made from the original project, including the rename to **effectful** and ongoing API, runtime, documentation, and developer-experience work. Unless otherwise noted, this repository is distributed under the same `CC-BY-SA-4.0` license. No endorsement by the original authors is implied.
