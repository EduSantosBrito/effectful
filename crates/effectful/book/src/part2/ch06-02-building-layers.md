# Building Layers — From Simple to Complex

effectful currently has two layer surfaces:

- `LayerFn` / `LayerBuild` for typed HList-style values.
- `Layer<ROut, E, RIn>` for `ServiceContext` services.

## LayerFn

`LayerFn` wraps a zero-argument function returning `Result<Output, Error>`.

```rust,ignore
use effectful::{LayerBuild, LayerFn, tagged};

let config_layer = LayerFn(|| Ok(tagged::<ConfigKey, _>(Config::from_env()?)));
let config = config_layer.build()?;
```

Use `LayerEffect::new(effect)` when the output comes from an `Effect` and should be cached after first build.

```rust,ignore
use effectful::LayerEffect;

let db_layer = LayerEffect::new(connect_pool(config.db_url()).map(|pool| tagged::<DatabaseKey, _>(pool)));
let db = db_layer.build()?;
```

## LayerFnFrom

Use `LayerFnFrom` when a layer depends on the output of a previous layer.

```rust,ignore
use effectful::{LayerFn, LayerFnFrom, LayerExt};

let config = LayerFn(|| Ok(Config::from_env()?));
let db = LayerFnFrom(|config: &Config| Ok(Database::connect(config.db_url())?));

let stack = config.and_then(db);
let env = stack.build()?;
```

## ServiceContext Layers

For derive-based services, use `Layer::succeed` or `Layer::effect`.

```rust,ignore
#[derive(Clone, Service)]
struct Config { /* ... */ }

let config_layer = Layer::succeed(Config::from_env());

let db_layer = Layer::effect("Database", || {
    Config::use_(|config| Database::connect(config.database_url))
});
```

`Layer::effect` receives dependencies through `ServiceContext`, so it can use `Effect::service::<S>()` or `S::use_`.

## Memoization

`Layer<ROut, E, RIn>` exposes `.memoized()`.

```rust,ignore
let shared_config = config_layer.memoized();
```

`LayerEffect` in the HList surface caches by design after the first build.
