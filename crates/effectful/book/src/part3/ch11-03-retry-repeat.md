# retry and repeat — Applying Policies

`retry` and `repeat` apply a `Schedule` to effect factories. They are free functions because `Effect` values are one-shot.

## retry

```rust,ignore
use std::time::Duration;
use effectful::{Schedule, retry};

let result = retry(
    || flaky_api_call(),
    Schedule::exponential(Duration::from_millis(100)).compose(Schedule::recurs(3)),
);
```

`retry` runs the effect returned by the factory. If it fails and the schedule continues, it waits and calls the factory again. It returns the first success, or the last error when the schedule stops.

## repeat

```rust,ignore
use std::time::Duration;
use effectful::{Schedule, repeat};

let polling = repeat(
    || check_job_status(),
    Schedule::spaced(Duration::from_secs(5)).compose(Schedule::recurs(12)),
);
```

`repeat` runs once, then continues while the schedule continues. It returns the last success value.

Use cases:

- Poll for job completion every few seconds
- Send a bounded number of heartbeats
- Refresh a cache on a fixed interval

## Explicit Clock Variants

Use `retry_with_clock` and `repeat_with_clock` when tests need a deterministic clock.

```rust,ignore
use effectful::{Schedule, TestClock, retry_with_clock};
use std::time::{Duration, Instant};

let clock = TestClock::new(Instant::now());
let effect = retry_with_clock(
    || flaky_api_call(),
    Schedule::exponential(Duration::from_millis(100)).compose(Schedule::recurs(3)),
    clock,
    None,
);
```

The `_and_interrupt` variants also accept a `CancellationToken`.

## Conditional Retry

The current API does not include `retry_while` or an error predicate parameter. `retry` retries every failure until the schedule stops. If you need error-sensitive retry, write a small custom loop around `Effect::run` or add that policy at the call site before using `retry`.

## Composition

`retry` and `repeat` return ordinary effects.

```rust,ignore
let batch = retry(
    || process_single_item(item.clone()),
    Schedule::exponential(Duration::from_millis(100)).compose(Schedule::recurs(3)),
);

let continuous = repeat(
    move || batch_factory(),
    Schedule::spaced(Duration::from_secs(60)).compose(Schedule::recurs(10)),
);
```
