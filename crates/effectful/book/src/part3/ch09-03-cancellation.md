# Cancellation — Interrupting Gracefully

Cancellation is explicit and cooperative. `CancellationToken` is a shared flag; `FiberHandle::interrupt()` marks a fiber handle as interrupted.

## CancellationToken

```rust,ignore
use effectful::{CancellationToken, check_interrupt};

let token = CancellationToken::new();
let child = token.child_token();

assert!(!token.is_cancelled());
token.cancel();
assert!(child.is_cancelled());
```

Cancelling a parent token cancels child tokens. Cancelling a child does not cancel its parent.

## Checking for Cancellation

`check_interrupt(&token)` snapshots whether the token is cancelled.

```rust,ignore
fn process_large_dataset(token: CancellationToken) -> Effect<(), Never, ()> {
    effect! {
        for chunk in large_dataset.chunks(1000) {
            let cancelled = bind* check_interrupt(&token);
            if cancelled {
                break;
            }
            process_chunk(chunk);
        }
    }
}
```

Use `token.cancelled()` when an effect should wait until cancellation happens.

```rust,ignore
let wait_for_shutdown = token.cancelled(); // Effect<(), Never, ()>
```

## Interrupting a FiberHandle

```rust,ignore
let handle = run_fork(&runtime, || (background_work(), ()));

handle.interrupt();

let result = handle.join().await;
assert!(matches!(result, Err(Cause::Interrupt(_))));
```

`interrupt()` completes the handle with `Cause::Interrupt(id)` if it was still pending. It returns `false` if the handle had already completed.

## Graceful Shutdown

The basic shutdown pattern is:

1. Signal shared cancellation tokens.
2. Interrupt top-level handles that should stop.
3. Await handles with whatever timeout policy your runtime uses.

```rust,ignore
token.cancel();

for handle in &handles {
    handle.interrupt();
}

for handle in handles {
    let _ = handle.join().await;
}
```

## Not Present Yet

The current API does not include method-style `with_cancellation` or an `uninterruptible` helper. Keep cancellation explicit by passing `CancellationToken` to long-running effects and checking it at safe points.
