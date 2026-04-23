# acquire_release — Acquire Then Release

`acquire_release(acquire, release)` runs an acquire effect, then runs the release effect, then returns the acquired value.

```rust,ignore
use effectful::acquire_release;

let effect = acquire_release(
    open_connection(),
    |conn| conn.close(),
);
```

Current behavior is simple and immediate:

1. Run `open_connection()`.
2. If acquisition succeeds, clone the acquired value.
3. Run `conn.close()` in a default release environment.
4. Return the cloned acquired value.

This is not a scoped bracket that keeps the resource open for a user block. For block-scoped resource use, use `scope_with` and `Scope::add_finalizer`, or a `Pool` checkout that returns resources when the caller's scope closes.

## When to Use It

Use `acquire_release` for acquire/release pairs where returning the acquired value after release is still meaningful, or as a low-level primitive while building stronger resource helpers.

```rust,ignore
let effect = effect! {
    bind* acquire_release(load_temp_value(), |value| cleanup_temp_value(value))
};
```

## Scoped Resource Pattern

For resources that must remain open while work runs, register a finalizer in a scope.

```rust,ignore
let program = scope_with(|scope| {
    effect! {
        let conn = bind* open_connection();
        let conn_for_close = conn.clone();
        scope.add_finalizer(Box::new(move |_| conn_for_close.close()));

        bind* run_query(&conn, "SELECT 1")
    }
});
```

That pattern keeps acquisition, use, and cleanup in the same visible block.
