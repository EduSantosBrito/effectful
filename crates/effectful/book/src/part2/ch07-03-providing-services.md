# Providing Services via Layers

The derive-service API makes a service type both key and value. Provide concrete services with `Layer::succeed` or construct them with `Layer::effect`.

## Minimal Service Layer

```rust,ignore
use effectful::{Layer, Service};

#[derive(Clone, Service)]
struct UserRepository {
    pool: Pool,
}

let repo_layer = Layer::succeed(UserRepository { pool });
```

An effect can read the service from `ServiceContext`.

```rust,ignore
fn get_user(id: u64) -> Effect<User, AppError, ServiceContext> {
    UserRepository::use_(move |repo| repo.get_user(id))
}
```

Run by providing the layer at the edge.

```rust,ignore
let user = run_blocking(get_user(42).provide(repo_layer), ())?;
```

## Layer with Dependencies

`Layer::effect` builds a service by running an effect in `ServiceContext`.

```rust,ignore
#[derive(Clone, Service)]
struct Config { database_url: String }

#[derive(Clone, Service)]
struct Database { /* ... */ }

let config_layer = Layer::succeed(Config::from_env());

let db_layer = Layer::effect("Database", || {
    Config::use_(|config| Database::connect(config.database_url))
});

let app_layer = db_layer.provide_merge(config_layer);
```

`provide` hides provider services in the final output. `provide_merge` keeps them.

## Test Layer with Mock

Tests provide the same service type with a fake implementation.

```rust,ignore
#[derive(Clone, Service)]
struct UserRepository {
    users: Arc<HashMap<u64, User>>,
}

let test_layer = Layer::succeed(UserRepository::from_users([alice(), bob()]));
let exit = run_test(get_user(1).provide(test_layer), ());
```

Application code does not change; only the layer at the edge changes.
