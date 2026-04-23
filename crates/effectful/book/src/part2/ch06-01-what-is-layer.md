# What Is a Layer?

An `Effect` describes a computation that needs an environment. A layer describes how to build values or services that can become that environment.

## Two Layer Surfaces

effectful currently exposes two related layer APIs.

```text
LayerFn / LayerBuild
  Builds typed values, often `Tagged<K, V>` cells for HList `Context`s.

Layer<ROut, E, RIn>
  Builds services into `ServiceContext` for derive-service applications.
```

## HList-Style Layer

```rust,ignore
use effectful::{LayerBuild, LayerFn, tagged};

let db_layer = LayerFn(|| {
    let pool = connect_pool_blocking(database_url)?;
    Ok(tagged::<DatabaseKey, _>(pool))
});

let db_cell = db_layer.build()?;
```

`LayerFn` is lazy: construction does nothing; `build()` runs the constructor.

## ServiceContext Layer

```rust,ignore
use effectful::{Layer, Service};

#[derive(Clone, Service)]
struct Database { /* ... */ }

let db_layer = Layer::succeed(Database::new());
let context = run_blocking(db_layer.build(), ())?;
```

For dependencies, use `Layer::effect` and read upstream services from `ServiceContext`.

```rust,ignore
let db_layer = Layer::effect("Database", || {
    Config::use_(|config| Database::connect(config.database_url))
});
```

## Providing to Effects

`Effect::provide(layer)` exists for `Effect<_, _, ServiceContext>`.

```rust,ignore
let result = run_blocking(my_app().provide(app_layer), ())?;
```

For typed `Context` environments, build the context and pass it directly to `run_blocking(effect, env)`.

## Lifecycle

Resource lifecycles are handled by `Scope`, `Pool`, and explicit finalizers. Layers are constructors; if a layer creates resources that require cleanup, make that cleanup part of the service design or the surrounding scope.
