# R Revisited — More Than Just a Type Parameter

`R` is the environment type required to run an effect.

```rust,ignore
fn get_user(id: u64) -> Effect<User, DbError, Database>
```

This says: to run `get_user`, supply a `Database` environment.

## R as a Contract

```rust,ignore
let effect = get_user(1);

// Missing environment: does not match current runner API.
// run_blocking(effect);

let user = run_blocking(effect, my_database)?;
```

You can also capture the environment first.

```rust,ignore
let ready = get_user(1).provide_env(my_database);
let user = run_blocking(ready, ())?;
```

## Composition Requires One Environment Type

Inside one `effect!` block, bound effects must agree on the same `R` type after any adaptations.

```rust,ignore
fn get_user(id: u64) -> Effect<User, DbError, Database> { /* ... */ }
fn get_posts(user_id: u64) -> Effect<Vec<Post>, DbError, Database> { /* ... */ }

fn get_user_with_posts(id: u64) -> Effect<(User, Vec<Post>), DbError, Database> {
    effect! {
        let user = bind* get_user(id);
        let posts = bind* get_posts(user.id);
        (user, posts)
    }
}
```

When effects need different environment types, adapt them with `zoom_env`, use a common `ServiceContext`, or build an HList `Context` that exposes both services.

```rust,ignore
fn log(msg: &str) -> Effect<(), LogError, Logger> { /* ... */ }
fn get_user(id: u64) -> Effect<User, DbError, Database> { /* ... */ }

fn get_user_logged(id: u64) -> Effect<User, AppError, AppEnv> {
    effect! {
        bind* log(&format!("Fetching user {id}"))
            .zoom_env(|env: &mut AppEnv| env.logger.clone())
            .map_error(AppError::Log);

        bind* get_user(id)
            .zoom_env(|env: &mut AppEnv| env.database.clone())
            .map_error(AppError::Db)
    }
}
```

## R as Documentation

`Effect<Receipt, AppError, ServiceContext>` says the computation reads services from a runtime service table. `Effect<Receipt, AppError, Context<...>>` says exactly which tagged HList cells are required. A concrete environment type like `AppEnv` documents which aggregate application environment must be supplied.

## Why R Instead of Parameters?

Traditional Rust threads dependencies as function parameters. `R` moves that dependency requirement into the returned effect type, which makes it composable and delayable until the program edge.

The key rule: library functions return honest `Effect<A, E, R>` values; application edges decide which concrete environment to pass.
