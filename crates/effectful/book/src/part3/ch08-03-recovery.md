# Recovery Combinators — catch and Friends

The current `Effect` recovery surface works with typed errors `E`. `Cause<E>` appears in `Exit` and fibers, not in ordinary `catch` handlers.

## catch

`catch` handles a typed failure by returning another effect.

```rust,ignore
let resilient = risky_db_call().catch(|error: DbError| {
    match error {
        DbError::NotFound => succeed(User::anonymous()),
        other => fail(other),
    }
});
```

Use `catch` when recovery itself may still fail.

## catch_all

`catch_all` maps any typed error into a fallback success value. The returned effect is infallible through `E`.

```rust,ignore
let user = fetch_user(id).catch_all(|_error| User::anonymous());
```

Despite the name, this does not handle `Cause::Die` or `Cause::Interrupt`; it handles the effect's typed error channel.

## tap_error

`tap_error` runs an effect when the source fails, then re-emits the original error if the tap succeeds.

```rust,ignore
let observed = risky_call().tap_error(|message| log_error(message));
```

The handler receives a debug-formatted string because the original error is re-emitted without requiring `Clone`.

## map_error

Use `map_error` to translate errors into your application error type.

```rust,ignore
let effect = db_call().map_error(AppError::Database);
```

## Not Present Yet

The current API does not include `fold`, `or_else`, or `ignore_error` methods on `Effect`. Use `catch`, `catch_all`, `map`, and `map_error`, or pattern match on `run_blocking(effect, env)` at a boundary.
