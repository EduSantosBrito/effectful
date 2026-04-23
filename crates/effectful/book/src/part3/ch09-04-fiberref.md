# FiberRef — Fiber-Local State

`FiberRef<A>` stores values keyed by `(FiberRef id, FiberId)`. It is useful for trace ids, request context, and other fiber-local data.

## Creating a FiberRef

```rust,ignore
use effectful::{FiberRef, run_blocking};

let trace_id: FiberRef<String> = run_blocking(
    FiberRef::make(|| "none".to_string()),
    (),
)?;
```

`FiberRef::make` returns an effect because allocation is part of the effect runtime model.

## Reading and Writing

```rust,ignore
let program = effect! {
    bind* trace_id.set("req-abc-123".to_string());

    let id = bind* trace_id.get();
    bind* log(&format!("[{id}] processing request"));

    bind* process_request()
};
```

Available operations include `get`, `set`, `update`, `modify`, `reset`, `locally`, and `locally_with`.

## Fork and Join Hooks

When you manage logical child fibers yourself, use `on_fork` and `on_join` to seed and merge fiber-local values.

```rust,ignore
let parent = FiberId::ROOT;
let child = FiberId::fresh();

run_blocking(trace_id.on_fork(parent, child), ())?;

with_fiber_id(child, || {
    run_blocking(trace_id.set("child-trace".to_string()), ())
})?;

run_blocking(trace_id.on_join(parent, child), ())?;
```

The default fork behavior clones the parent value into the child. The default join behavior keeps the child value.

## Local Overrides

`locally(value, effect)` overrides the current fiber's value while the inner effect runs, then restores the previous value.

```rust,ignore
let inner = trace_id.locally(
    "override".to_string(),
    trace_id.get(),
);

let value = run_blocking(inner, ())?;
assert_eq!(value, "override");
```

## Current Limitations

Fiber identity is stored in a thread-local cell. This matches single-threaded `run_blocking` and current-thread Tokio runtimes. Multi-threaded task migration is not tracked yet.
