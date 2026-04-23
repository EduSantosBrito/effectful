# Providing Dependencies

An effect with a non-`()` `R` needs an environment before it can run. The current low-level API is explicit: pass the environment to the runner, or capture it with `provide_env`.

## Run with an Environment

```rust,ignore
fn get_user(id: u64) -> Effect<User, DbError, Database> { /* ... */ }

let effect = get_user(42);
let user = run_blocking(effect, my_database)?;
```

`run_blocking(effect, env)` and `run_async(effect, env)` consume both the effect and the environment.

## Capture an Environment

Use `provide_env` when you want to turn `Effect<A, E, R>` into `Effect<A, E, ()>` before running or composing at the edge.

```rust,ignore
let ready: Effect<User, DbError, ()> = get_user(42).provide_env(my_database);
let user = run_blocking(ready, ())?;
```

There is no raw `.provide(value)` / `.provide_some(value)` API in the current core effect surface.

## ServiceContext and Layers

For derive-based services, effects can require `ServiceContext` and be provided with a `Layer`.

```rust,ignore
#[derive(Clone, Service)]
struct Database { /* ... */ }

fn get_user(id: u64) -> Effect<User, AppError, ServiceContext> {
    Effect::<Database, AppError, ServiceContext>::service::<Database>()
        .flat_map(move |db| db.get_user(id))
}

let layer = Layer::succeed(Database::new());
let user = run_blocking(get_user(42).provide(layer), ())?;
```

`Effect::provide(layer)` exists for `Effect<_, _, ServiceContext>`.

## Tagged Contexts

For HList-style typed contexts, construct a `Context` and pass it as `R`.

```rust,ignore
service_key!(pub struct DbKey);

let env = service_env::<DbKey, _>(my_database);
let user = run_blocking(get_user(42), env)?;
```

Use `effect.provide_head(value)` when the effect's environment is `Context<Cons<Service<K, V>, Tail>>` and you want to provide the head cell.

## Program Edge Rule

Provide dependencies at the program edge: `main`, request adapters, integration tests, and top-level supervisors. Library functions should return effects that honestly describe their required `R`.
