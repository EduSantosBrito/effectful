# Transforming Success — map and its Friends

You have an effect. It produces some value. But you want a different value — or a different error. That's what `map` and `map_error` are for.

## map

`map` transforms the success value without running any new effects:

```rust,ignore
use effectful::{succeed, Effect};

let number: Effect<i32, String, ()> = succeed(21);
let doubled: Effect<i32, String, ()> = number.map(|n| n * 2);
let text: Effect<String, String, ()> = doubled.map(|n| n.to_string());
```

None of these `.map()` calls executes anything. Each one wraps the previous description in a new layer: "and then transform the result with this function." The chain of transformations only runs when you call `run_blocking` or similar.

The type of the effect changes with each `map`. The `A` parameter shifts:

```rust,ignore
// Effect<i32, String, ()>
//   .map(|n: i32| n.to_string())
// → Effect<String, String, ()>
```

The `E` (error type) and `R` (requirements) stay the same. `.map` touches only the success path.

## map_error

`map_error` transforms the failure type, leaving the success path untouched:

```rust,ignore
use effectful::fail;

#[derive(Debug)]
struct AppError(String);

let db_err: Effect<String, String, ()> = fail("db connection failed".to_string());
let app_err: Effect<String, AppError, ()> = db_err.map_error(|s| AppError(s));
```

This is typically used at module boundaries when you need to unify error types. A database layer might return `DbError`, but your application layer needs `AppError`. `map_error` does the conversion without touching anything else.

## Why These Don't Execute Anything

It's worth repeating: neither `map` nor `map_error` runs any computation.

```rust,ignore
let effect = succeed(42)
    .map(|n| { println!("mapping!"); n + 1 })
    .map(|n| n * 2);

// At this point: nothing has printed, nothing has computed.
// We have a description of three steps.

let result = run_blocking(effect, ());
// NOW the effect runs. "mapping!" prints once. Result is 86.
```

This is the promise of laziness: you can build pipelines of transformations without triggering side effects until the moment you choose.

## Combining map and map_error

A common pattern is calling both to normalise an effect into your domain's types:

```rust,ignore
fn fetch_user_record(id: u64) -> Effect<User, AppError, ()> {
    raw_db_fetch(id)
        .map(|row| User::from_row(row))
        .map_error(|e| AppError::Database(e))
}
```

The effect goes in with raw DB types; it comes out with domain types. The transformation chain documents the conversion at a glance.

## and_then / and_then_discard

Two more helpers are worth knowing:

```rust,ignore
// and_then: sequence two effects, keep the second result
let validated: Effect<i32, String, ()> = succeed(42)
    .and_then(succeed(100));

// and_then_discard: sequence two effects, keep the first result
let kept_left: Effect<i32, String, ()> = succeed(42)
    .and_then_discard(succeed(()));
```

Use `flat_map` when the second effect depends on the first value.

## Summary

| Method | Changes | Does not change |
|--------|---------|-----------------|
| `.map(f)` | `A` (success type) | `E`, `R` |
| `.map_error(f)` | `E` (error type) | `A`, `R` |
| `.tap(f)` | nothing | `A`, `E`, `R` |

None of them execute the effect. They all return new, larger descriptions.
