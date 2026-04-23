# Error Handling Inside effect!

The `bind*` operator short-circuits on failure — if a bound effect fails, the whole `effect!` block fails with that error. But you can also handle errors *within* the block.

## The Default: Short-Circuit

```rust,ignore
effect! {
    let a = bind* step_a();    // if this fails → whole block fails
    let b = bind* step_b(a);   // if this fails → whole block fails
    b
}
```

This matches `?` in `Result`. You get clean sequencing at the cost of aborting early. For most code, that's exactly what you want.

## Catching Errors Mid-block

To handle an error inline and continue, use `.catch` before the `bind*`:

```rust,ignore
effect! {
    let user = bind* fetch_user(id).catch(|_| succeed(User::anonymous()));
    // If fetch_user fails, we get User::anonymous() and continue
    render_user(user)
}
```

`.catch` converts a failure into a success (or a different effect). The `bind*` then sees a successful effect.

## Converting Errors with map_error

Often you have multiple effect types with different `E` parameters and need to unify them:

```rust,ignore
#[derive(Debug)]
enum AppError {
    Db(DbError),
    Network(HttpError),
}

effect! {
    let user = bind* fetch_user(id).map_error(AppError::Db);
    let data = bind* fetch_external_data(user.id).map_error(AppError::Network);
    process(user, data)
}
```

Both effects are converted to the same `AppError` before binding. The block's `E` parameter is `AppError` throughout.

## Handling Errors as Values

The current `Effect` API does not expose a `fold` method. Use `.catch` / `.catch_all`, or run the effect and pattern match on `Result` at the boundary.

```rust,ignore
effect! {
    let outcome = bind* risky_operation()
        .map(|val| format!("Success: {val}"))
        .catch_all(|err| format!("Error: {err}"));
    log_outcome(outcome)
}
```

`catch_all` turns a typed failure into a fallback success value, so the resulting effect is infallible through `E`.

## Re-raising Errors

Inside a `.catch` handler, you can inspect the error and decide whether to recover or re-fail:

```rust,ignore
effect! {
    let result = bind* db_operation().catch(|error| {
        if error.is_transient() {
            // Transient: retry once with a fallback
            fallback_db_operation()
        } else {
            // Permanent: re-raise
            fail(error)
        }
    });
    result
}
```

`fail(error)` inside a handler produces a failing effect — the outer `bind*` then propagates it.

## Accumulating Multiple Errors

Short-circuit stops at the first error. The current root API does not include `validate_all`; collect independent validation errors manually at the boundary:

```rust,ignore
let mut errors = Vec::new();

if let Err(error) = run_blocking(validate_name(&input.name), ()) {
    errors.push(error);
}
if let Err(error) = run_blocking(validate_email(&input.email), ()) {
    errors.push(error);
}
if let Err(error) = run_blocking(validate_age(input.age), ()) {
    errors.push(error);
}
```

Chapter 8 covers accumulation patterns in detail.

## The Rule of Thumb

| Want | Do |
|------|----|
| Stop at first failure | plain `bind* effect` |
| Provide a fallback | `bind* effect.catch(|e| fallback)` |
| Unify error types | `bind* effect.map_error(Into::into)` |
| Turn failure into value | `bind* effect.catch_all(|e| fallback)` |
| Collect all failures | Manual accumulation outside the macro |
