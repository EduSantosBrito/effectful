# Spawning and Joining — run_fork and fiber_all

The current API uses `run_fork` to spawn effects and `FiberHandle` methods to join, inspect, or interrupt them.

## run_fork: Spawn One Fiber

```rust,ignore
use effectful::{ThreadSleepRuntime, run_fork};

let runtime = ThreadSleepRuntime;
let handle = run_fork(&runtime, || (compute_expensive_result(), env));

let local_result = local_computation();
let remote_result = handle.join().await?;

(local_result, remote_result)
```

The factory returns `(effect, env)` so the worker owns both the one-shot effect and its environment.

## await_exit vs join

```rust,ignore
let exit: Exit<A, E> = run_async(handle.await_exit(), ()).await?;
let result: Result<A, Cause<E>> = handle.join().await;
```

Use `await_exit` when you want the exact `Exit`; use `join` for a `Result` with `Cause<E>` on failure.

## fiber_all

`fiber_all` joins already-created handles.

```rust,ignore
use effectful::{fiber_all, run_async};

let handles = user_ids
    .into_iter()
    .map(|id| run_fork(&runtime, move || (fetch_user(id), ())))
    .collect::<Vec<_>>();

let users: Vec<User> = run_async(fiber_all(handles), ()).await?;
```

If any handle fails, `fiber_all` returns the first `Cause<E>` it observes.

## Racing

There is no `fiber_race` or `fiber_any` free function in the current API. `FiberHandle::or_else` races two handles and completes with whichever handle resolves first.

```rust,ignore
let primary = run_fork(&runtime, || (fetch_from_primary(), ()));
let backup = run_fork(&runtime, || (fetch_from_backup(), ()));

let raced = primary.or_else(backup);
let data = raced.join().await?;
```

The slower fiber is not automatically cancelled by `or_else`; keep its handle if you need to interrupt it.

## Error Behavior

| Operation | Failure shape |
|-----------|---------------|
| `handle.join().await` | `Result<A, Cause<E>>` |
| `handle.await_exit()` | `Exit<A, E>` inside an infallible effect |
| `fiber_all(handles)` | `Effect<Vec<A>, Cause<E>, ()>` |
| `handle.or_else(other)` | First handle completion wins |
