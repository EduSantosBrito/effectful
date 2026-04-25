# effect_test — The Test Harness Boundary

Effectful tests should compose effects in the test body and let the harness execute them. This keeps lazy execution semantics intact and keeps direct `Effect::run(&mut ...)` calls inside runtime/test harness internals.

## Preferred Usage

```rust,ignore
use effectful::{Effect, effect_test};

#[effect_test]
fn simple_effect_succeeds() -> Effect<(), &'static str, ()> {
    Effect::new(|_| Ok(()))
}
```

`#[effect_test]` creates an async Tokio test and runs the returned effect. `Ok(_)` passes. `Err(E)` panics with `Debug` output, so the error type must implement `Debug`.

The macro uses effectful's internal current-thread Tokio test re-export; downstream crates do not need to call `Effect::run(&mut ...)` in the test body.

## Provided ServiceContext

```rust,ignore
use effectful::{Effect, MissingService, Service, ServiceContext, effect_test};

#[derive(Clone, Service)]
struct Config {
    port: u16,
}

fn test_env() -> ServiceContext {
    Config { port: 8080 }.to_context()
}

#[effect_test(env = "test_env")]
fn reads_config() -> Effect<(), MissingService, ServiceContext> {
    Effect::<Config, MissingService, ServiceContext>::service::<Config>()
        .map(|config| assert_eq!(config.port, 8080))
}
```

The fixture runs once per test and returns the environment consumed by the effect.

## Layer-Based Setup

```rust,ignore
use effectful::{Effect, Layer, MissingService, Service, ServiceContext, effect_test};

#[derive(Clone, Service)]
struct Config {
    port: u16,
}

fn test_layer() -> Layer<Config, MissingService, ()> {
    Layer::succeed(Config { port: 8080 })
}

#[effect_test(layer = "test_layer")]
fn reads_layer_config() -> Effect<(), MissingService, ServiceContext> {
    Effect::<Config, MissingService, ServiceContext>::service::<Config>()
        .map(|config| assert_eq!(config.port, 8080))
}
```

This is the Rust equivalent of Effect's `it.layer(...)` boundary: the test body remains an effect, and the adapter builds/provides the services.

## Helper API

Use helper functions when an attribute macro is not appropriate.

```rust,ignore
use effectful::testing::expect_effect_test;

#[tokio::test]
async fn create_user_inserts_into_db() {
    expect_effect_test(create_user(NewUser { name: "Alice".into(), age: 30 })).await;
}
```

Available helpers:

| Function | Notes |
|----------|-------|
| `expect_effect_test(effect).await` | Run with `R: Default`, panic on `Err(E: Debug)` |
| `expect_effect_test_with_env(effect, env).await` | Run with explicit environment, panic on failure |
| `expect_effect_test_with_layer(effect, layer).await` | Build a layer for `ServiceContext`, panic on failure |
| `run_effect_test(effect).await` | Run with `R: Default`, return `Result<A, E>` |
| `run_effect_test_with_env(effect, env).await` | Run with explicit environment, return `Result<A, E>` |
| `TestRuntime::with_env(fixture)` | Reusable adapter for explicit fixture functions |

## Asserting on Exit

`run_test` remains available when you need the older synchronous `Exit<A, E>` shape.

```rust,ignore
use effectful::{Exit, run_test};

#[test]
fn division_by_zero_fails() {
    let exit = run_test(divide(10, 0), ());
    assert!(matches!(exit, Exit::Failure(Cause::Fail(DivError::DivisionByZero))));
}
```

Pass `()` for effects with no environment. `run_test(effect, env)` resets the test leak counters, runs the effect with `run_blocking`, then checks the leak counters.

Common `Exit` shapes:

| Exit | Meaning |
|------|---------|
| `Exit::Success(a)` | Effect succeeded |
| `Exit::Failure(Cause::Fail(e))` | Typed failure |
| `Exit::Failure(Cause::Die(message))` | Defect message |
| `Exit::Failure(Cause::Interrupt(id))` | Fiber interrupt |

## run_test_with_clock

```rust,ignore
use std::time::Instant;
use effectful::{TestClock, run_test_with_clock};

let clock = TestClock::new(Instant::now());
let exit = run_test_with_clock(effect, env, clock);
```

`run_test_with_clock` currently delegates to `run_test` after accepting the explicit clock argument. Use explicit-clock scheduling helpers (`retry_with_clock`, `repeat_with_clock`) when the effect itself must use that clock.

## Leak Assertions

The testing module exposes assertion effects and test hooks:

```rust,ignore
use effectful::{assert_no_leaked_fibers, assert_no_unclosed_scopes};

run_blocking(assert_no_leaked_fibers(), ())?;
run_blocking(assert_no_unclosed_scopes(), ())?;
```

The effect-test helpers and `run_test` call both assertions after the effect run. If a hook recorded a leak, the assertion panics.
