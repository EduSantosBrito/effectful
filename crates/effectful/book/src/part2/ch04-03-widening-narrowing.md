# Widening and Narrowing — Environment Transformations

Sometimes your effect needs *part* of an environment, but the caller has the whole thing. This is where `zoom_env` comes in.

## The Mismatch Problem

Imagine your application has a large environment type:

```rust,ignore
struct AppEnv {
    db: Database,
    logger: Logger,
    config: Config,
    metrics: MetricsClient,
}
```

You have a utility function that only needs a `Logger`:

```rust,ignore
fn log_event(msg: &str) -> Effect<(), LogError, Logger> { ... }
```

You can't call this inside an `effect!` block that has `AppEnv` in scope — the types don't match. You need to *narrow* the environment down.

## zoom_env: Narrow the Environment

`zoom_env` adapts an effect to work with a *larger* environment by providing a lens from the larger type to the smaller one:

```rust,ignore
// Adapt log_event to work with AppEnv.
// The projection returns the smaller environment by value.
let app_log = log_event("hello").zoom_env(|env: &mut AppEnv| env.logger.clone());
```

Now `app_log` has type `Effect<(), LogError, AppEnv>`. The function extracts the `Logger` from `AppEnv` and feeds it to the original effect.

Inside `effect!`, the pattern looks like:

```rust,ignore
fn process(data: Data) -> Effect<(), AppError, AppEnv> {
    effect! {
        bind* log_event("start").zoom_env(|e: &mut AppEnv| e.logger.clone()).map_error(AppError::Log);
        bind* db_query(data).zoom_env(|e: &mut AppEnv| e.db.clone()).map_error(AppError::Db);
    }
}
```

## Transforming the Environment

The current API uses `zoom_env` for both narrowing and transformation. It applies a function to convert the caller's environment into what the effect needs:

```rust,ignore
// Effect needs a raw string URL
fn connect_raw() -> Effect<Database, DbError, String> { ... }

// You have a Config that contains the URL
let with_config = connect_raw().zoom_env(|cfg: &mut Config| cfg.db_url.clone());
// Now type is Effect<Database, DbError, Config>
```

There is no separate `contramap_env` method in the current API.

## R as Documentation Revisited

These combinators highlight why `R` is valuable as documentation. When you see:

```rust,ignore
fn log_event(msg: &str) -> Effect<(), LogError, Logger>
```

You know *exactly* what this function needs. You don't need to read its body to see if it also touches the database. The `zoom_env` call at the use site makes the adaptation explicit — it's not hidden.

Compare to the pre-effect alternative:

```rust,ignore
// Traditional: you'd need to read the body to know what `env` is used for
fn log_event(env: &AppEnv, msg: &str) -> Result<(), LogError> { ... }
```

With `R`, the function declares what it needs. With `zoom_env`, the caller declares how to satisfy it.

## When to Use These

In practice, `zoom_env` appears most often in library code — when writing reusable utilities that should work with any environment containing the right piece. Application code often uses `ServiceContext`, `Context`, and Layers instead.

Think of `zoom_env` as the manual fallback when the automatic layer-based wiring isn't the right fit.
