# Schedule — The Retry/Repeat Policy Type

A `Schedule` is a policy used by the free `retry` and `repeat` functions. It receives `ScheduleInput { attempt }` and either returns a `ScheduleDecision { delay }` or stops.

## The Core Concept

```rust,ignore
use effectful::{Schedule, ScheduleInput};

let mut schedule = Schedule::recurs(3);

while let Some(decision) = schedule.next(ScheduleInput { attempt }) {
    sleep(decision.delay);
    attempt += 1;
}
```

The current schedule model tracks the attempt number. It does not inspect elapsed time, the last success value, or the last error.

## Creating Schedules

```rust,ignore
use std::time::Duration;
use effectful::{Schedule, ScheduleInput};

let max_three = Schedule::recurs(3);
let fixed = Schedule::spaced(Duration::from_secs(1));
let exponential = Schedule::exponential(Duration::from_millis(100));
let until_attempt_ten = Schedule::recurs_until(Box::new(|input: &ScheduleInput| input.attempt >= 10));
let while_first_ten = Schedule::recurs_while(Box::new(|input: &ScheduleInput| input.attempt < 10));
```

## Combining Schedules

Use `compose` to require both schedules to continue. The produced delay is the maximum of the two decisions.

```rust,ignore
use std::time::Duration;
use effectful::Schedule;

let retry_policy = Schedule::exponential(Duration::from_millis(100))
    .compose(Schedule::recurs(5));
```

Use `jittered` to apply deterministic jitter to delays.

```rust,ignore
let jittered = Schedule::spaced(Duration::from_secs(1)).jittered();
```

## Schedule as a Value

Schedules are values you pass to `retry` or `repeat`. Effects are one-shot, so these functions take factories.

```rust,ignore
use std::time::Duration;
use effectful::{Effect, Schedule, retry};

fn call_external_api() -> Effect<Response, ApiError, HttpClient> {
    retry(
        || make_request(),
        Schedule::exponential(Duration::from_millis(100)).compose(Schedule::recurs(5)),
    )
}
```
