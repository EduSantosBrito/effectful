# run_test — The Test Harness

`run_test` runs an effect with an explicit environment and returns `Exit<A, E>`.

## Basic Usage

```rust,ignore
use effectful::{Exit, run_test, succeed};

#[test]
fn simple_effect_succeeds() {
    let result = run_test(succeed::<_, (), ()>(42), ());
    assert_eq!(result, Exit::Success(42));
}
```

There is no one-argument shorthand. Pass `()` for effects with no environment.

## Asserting on Exit

```rust,ignore
#[test]
fn division_by_zero_fails() {
    let exit = run_test(divide(10, 0), ());
    assert!(matches!(exit, Exit::Failure(Cause::Fail(DivError::DivisionByZero))));
}
```

Common shapes:

| Exit | Meaning |
|------|---------|
| `Exit::Success(a)` | Effect succeeded |
| `Exit::Failure(Cause::Fail(e))` | Typed failure |
| `Exit::Failure(Cause::Die(message))` | Defect message |
| `Exit::Failure(Cause::Interrupt(id))` | Fiber interrupt |

## Running with an Environment

```rust,ignore
#[test]
fn create_user_inserts_into_db() {
    let fake_db = FakeDatabase::new();
    let env = fake_db.to_context();

    let eff = create_user(NewUser { name: "Alice".into(), age: 30 });
    let exit = run_test(eff, env);

    assert!(matches!(exit, Exit::Success(_)));
}
```

`run_test(effect, env)` is the full API. It resets the test leak counters, runs the effect with `run_blocking`, then checks the leak counters.

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

`run_test` calls both assertions after the effect run. If a hook recorded a leak, the assertion panics.
