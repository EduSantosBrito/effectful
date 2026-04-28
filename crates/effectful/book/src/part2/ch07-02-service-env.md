# ServiceEnv and service_env

`ServiceEnv<K, V>` is a one-cell tagged `Context`. `service_env::<K, V>(value)` constructs that context; it is not an accessor effect.

## Constructing a Tagged Service Environment

```rust,ignore
use effectful::{ServiceEnv, service_env, service_key};

service_key!(pub struct UserRepositoryKey);

let env: ServiceEnv<UserRepositoryKey, UserRepository> =
    service_env::<UserRepositoryKey, _>(repo);
```

The alias is:

```rust,ignore
type ServiceEnv<K, V> = Context<Cons<Service<K, V>, Nil>>;
```

## Accessing Tagged Services

Use `Context::get::<K>()` or `Get<K>` bounds.

```rust,ignore
fn get_user(id: u64) -> Effect<User, DbError, ServiceEnv<UserRepositoryKey, UserRepository>> {
    Effect::new(move |env| {
        let repo = env.get::<UserRepositoryKey>();
        repo.get_user_blocking(id)
    })
}
```

Inside `effect!`, there is no `bind* UserRepositoryKey` shorthand in the current API.

## Derive-Service Alternative

For most service code, prefer the `#[derive(Service)]` / `ServiceContext` API.

```rust,ignore
#[derive(Clone, Service)]
struct UserRepository { /* ... */ }

fn get_user(id: u64) -> Effect<User, AppError, ServiceContext> {
    UserRepository::use_(move |repo| repo.get_user(id))
}

let env = UserRepository::new().to_context();
let user = run_blocking(get_user(42), env)?;
```

This avoids exposing HList paths in most application signatures.

## Interop: compile-time `Context` → `ServiceContext`

When you have a statically-built `Context` and need to feed it into an effect that requires `ServiceContext`, convert at the edge:

```rust,ignore
use effectful::{ctx, Effect, IntoServiceContext, MissingService, Service, ServiceContext};

#[derive(Clone, Service)]
struct Config { port: u16 }

let static_ctx = ctx!(Config => Config { port: 8080 });
let env: ServiceContext = static_ctx.into_service_context();

let program: Effect<bool, MissingService, ServiceContext> =
    Config::use_sync(|config| config.port == 8080);

let ok = run_blocking(program, env)?;
```
