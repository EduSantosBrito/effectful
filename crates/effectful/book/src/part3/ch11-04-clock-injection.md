# Clock Injection — Testable Time

`retry` and `repeat` use a live clock by default. For deterministic tests, use the explicit-clock variants: `retry_with_clock`, `repeat_with_clock`, and their interruption-aware forms.

## The Clock Trait

```rust,ignore
use std::time::{Duration, Instant};
use effectful::{Effect, Never};

trait Clock {
    fn now(&self) -> Instant;
    fn sleep(&self, duration: Duration) -> Effect<(), Never, ()>;
    fn sleep_until(&self, deadline: Instant) -> Effect<(), Never, ()>;
}
```

`Clock` is monotonic-time oriented. Calendar time for logging is exposed separately through `LiveClock::now_utc()`.

## Production: LiveClock

```rust,ignore
use effectful::{LiveClock, ThreadSleepRuntime};

let live_clock = LiveClock::new(ThreadSleepRuntime);
```

`LiveClock` delegates sleeping and `now()` to a runtime.

## Testing: TestClock

```rust,ignore
use std::time::{Duration, Instant};
use effectful::{Clock, TestClock};

let start = Instant::now();
let clock = TestClock::new(start);

assert_eq!(clock.now(), start);

clock.advance(Duration::from_secs(60));
assert_eq!(clock.now(), start + Duration::from_secs(60));
```

`TestClock` records pending sleeps. Advancing or setting time drops pending sleeps whose deadlines have elapsed.

## Test Example

```rust,ignore
use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use std::time::{Duration, Instant};
use effectful::{Schedule, TestClock, retry_with_clock, run_blocking};

let clock = TestClock::new(Instant::now());
let attempts = Arc::new(AtomicU32::new(0));

let effect = retry_with_clock(
    {
        let attempts = attempts.clone();
        move || failing_operation(attempts.clone())
    },
    Schedule::exponential(Duration::from_secs(1)).compose(Schedule::recurs(3)),
    clock.clone(),
    None,
);

let result = run_blocking(effect, ());
assert!(result.is_err());
assert_eq!(attempts.load(Ordering::Relaxed), 4); // initial + 3 retries
```

This test runs without sleeping in real time because `TestClock::sleep` only records deadlines.

## Clock as a Service

For application logic that needs time directly, model the clock as a service in your environment. The scheduling helpers accept a clock value explicitly; your own services can do the same.
