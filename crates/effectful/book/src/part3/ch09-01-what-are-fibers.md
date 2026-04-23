# What Are Fibers? — Lightweight Structured Tasks

A fiber is an effectful concurrent task represented by `FiberHandle<A, E>`. It gives you a stable id, status inspection, interruption, and typed completion.

## Fibers vs. Raw Tasks

```rust,ignore
// Raw tokio::spawn: lifecycle is owned by Tokio's JoinHandle.
tokio::spawn(async {
    do_something().await;
});

// effectful fiber: lifecycle is explicit through FiberHandle<A, E>.
let runtime = ThreadSleepRuntime;
let handle: FiberHandle<User, DbError> = run_fork(&runtime, || (get_user(1), env));
let result: Result<User, Cause<DbError>> = handle.join().await;
```

`join()` returns `Result<A, Cause<E>>`. Use `await_exit()` when you need `Exit<A, E>`.

## FiberId

Each handle has a `FiberId`.

```rust,ignore
let id = handle.id();
log::debug!("spawned fiber {id:?}");
```

For code that needs to run under a specific fiber id, use `with_fiber_id(id, || ...)`.

## FiberHandle and FiberStatus

```rust,ignore
let status: FiberStatus = handle.status();

match status {
    FiberStatus::Running => {}
    FiberStatus::Succeeded => {}
    FiberStatus::Failed => {}
    FiberStatus::Interrupted => {}
}

handle.interrupt();
```

`FiberHandle<A, E>` is cloneable. Status inspection does not consume the handle.

## Structured Cleanup

Fibers can be attached to a `Scope` with `handle.scoped()`. Closing the scope interrupts the fiber through a finalizer.

```rust,ignore
let scope = Scope::make();
let scoped_effect = handle.scoped(); // Effect<A, Cause<E>, Scope>

// Run scoped_effect with `scope`; when another owner closes `scope`,
// the registered finalizer interrupts the handle.
```

Use scopes when a parent computation must own child-fiber cleanup.
