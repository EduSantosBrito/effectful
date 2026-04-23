# TestClock — Deterministic Time in Tests

`TestClock` is a manual `Clock` implementation. It is useful with `retry_with_clock`, `repeat_with_clock`, or code that accepts a `Clock` explicitly.

## The Problem with Real Time in Tests

```rust,ignore
let eff = retry(
    || failing_call(),
    Schedule::exponential(Duration::from_secs(1)).compose(Schedule::recurs(3)),
);
```

With the default live clock, this waits in real time. Use `TestClock` to avoid real sleeps.

## TestClock API

```rust,ignore
use std::time::{Duration, Instant};
use effectful::{Clock, TestClock};

let start = Instant::now();
let clock = TestClock::new(start);

let now: Instant = clock.now();
clock.advance(Duration::from_millis(500));
clock.set_time(start + Duration::from_secs(60));

let pending: Vec<Instant> = clock.pending_sleeps();
```

`pending_sleeps()` returns registered sleep deadlines. This is useful for asserting that code scheduled a delay.

## run_test_with_clock

`run_test_with_clock(effect, env, clock)` runs an already-built effect and returns `Exit<A, E>`. It does not create a closure-based test harness.

```rust,ignore
use std::time::Instant;
use effectful::{TestClock, run_test_with_clock, succeed};

let clock = TestClock::new(Instant::now());
let exit = run_test_with_clock(succeed::<_, (), ()>(42), (), clock);
assert_eq!(exit, Exit::succeed(42));
```

## Testing Scheduled Work

```rust,ignore
use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use std::time::{Duration, Instant};
use effectful::{Schedule, TestClock, repeat_with_clock, run_blocking, succeed};

let clock = TestClock::new(Instant::now());
let counter = Arc::new(AtomicU32::new(0));

let effect = repeat_with_clock(
    {
        let counter = counter.clone();
        move || {
            counter.fetch_add(1, Ordering::Relaxed);
            succeed::<u32, (), ()>(counter.load(Ordering::Relaxed))
        }
    },
    Schedule::spaced(Duration::from_secs(60)).compose(Schedule::recurs(3)),
    clock.clone(),
    None,
);

let value = run_blocking(effect, ())?;
assert_eq!(value, 4); // initial run + 3 repeats
```

`TestClock::sleep` records sleeps instead of blocking, so scheduled tests complete quickly.

## Fake Services

When business logic needs time, pass a `Clock` as part of your service/environment design. In production provide `LiveClock`; in tests provide `TestClock`.
