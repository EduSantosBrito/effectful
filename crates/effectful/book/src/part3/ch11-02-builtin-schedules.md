# Built-in Schedules

effectful's current `Schedule` API is intentionally small. It covers fixed delays, exponential backoff, attempt limits, predicates, composition, and deterministic jitter.

## recurs

```rust,ignore
Schedule::recurs(5)
```

Allows five additional schedule steps with zero delay. Compose it with a delay schedule to cap retries or repeats.

## spaced

```rust,ignore
Schedule::spaced(Duration::from_secs(5))
```

Produces the same delay for every step. Use it for polling, heartbeats, and fixed-interval repeat loops.

## exponential

```rust,ignore
Schedule::exponential(Duration::from_millis(100))
// Delays: 100ms, 200ms, 400ms, 800ms, ...
```

The standard retry backoff. Compose with `Schedule::recurs(n)` to cap attempts.

```rust,ignore
let bounded = Schedule::exponential(Duration::from_millis(100))
    .compose(Schedule::recurs(5));
```

## recurs_while / recurs_until

```rust,ignore
use effectful::{Schedule, ScheduleInput};

let first_ten = Schedule::recurs_while(Box::new(|input: &ScheduleInput| input.attempt < 10));
let until_ten = Schedule::recurs_until(Box::new(|input: &ScheduleInput| input.attempt >= 10));
```

These schedules decide from the attempt counter.

## compose

```rust,ignore
let policy = Schedule::spaced(Duration::from_secs(1))
    .compose(Schedule::recurs(10));
```

`compose` continues only while both schedules continue. When both produce a delay, the larger delay wins.

## jittered

```rust,ignore
let policy = Schedule::exponential(Duration::from_millis(100))
    .jittered()
    .compose(Schedule::recurs(5));
```

`jittered` currently applies deterministic jitter. It is useful for exercising jitter paths without adding random behavior to tests.

## Not Present Yet

The current API does not include Fibonacci schedules, max-delay caps, total-duration caps, method-style `.retry()`, or method-style `.repeat()`. Use the free `retry` / `repeat` functions with factories.
