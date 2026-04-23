# Stacking Layers — Composition Patterns

Layer composition is explicit. Use `Stack` / `and_then` for HList-style layers, and `merge` / `provide` / `provide_merge` for `ServiceContext` layers.

## Independent HList Layers

```rust,ignore
use effectful::{LayerBuild, Stack};

let stack = Stack(config_layer, logger_layer);
let env = stack.build()?; // Cons<ConfigOutput, Cons<LoggerOutput, Nil>>
```

The two layers must have the same error type.

## Dependent HList Layers

Use `LayerExt::and_then` or `StackThen` when the second layer needs the first layer's output.

```rust,ignore
use effectful::{LayerBuild, LayerExt, LayerFn, LayerFnFrom};

let config = LayerFn(|| Ok(Config::from_env()?));
let db = LayerFnFrom(|config: &Config| Ok(Database::connect(config.db_url())?));

let env = config.and_then(db).build()?;
```

## ServiceContext merge

For derive-service layers, `merge` builds independent layers and combines their `ServiceContext`s.

```rust,ignore
let app_layer = config_layer.merge(logger_layer);
let context = run_blocking(app_layer.build(), ())?;
```

## ServiceContext provide

Use `provide` when one layer needs services from another layer and you want to hide provider output.

```rust,ignore
let db_with_config = db_layer.provide(config_layer);
let context = run_blocking(db_with_config.build(), ())?;
```

Use `provide_merge` when you want both provider and dependent services in the final context.

```rust,ignore
let app_layer = db_layer.provide_merge(config_layer);
```

## Providing to Effects

`Effect::provide(layer)` exists for effects whose environment is `ServiceContext`.

```rust,ignore
let result = run_blocking(my_application().provide(app_layer), ())?;
```

For typed `Context` / HList environments, build the context and pass it to `run_blocking(effect, env)`.
