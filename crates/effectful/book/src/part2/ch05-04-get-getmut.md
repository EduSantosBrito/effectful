# Get and GetMut — Extracting from Context

`Get` and `GetMut` are type-level lookup traits for HList contexts.

## Get: Read-Only Access

```rust,ignore
use effectful::{Get, Here};

fn use_database<R>(env: &R) -> &Pool
where
    R: Get<DatabaseKey, Here, Target = Pool>,
{
    env.get()
}
```

The path parameter tells the compiler where the cell lives. `Here` means the head of the list.

## GetMut: Mutable Access

```rust,ignore
use effectful::{GetMut, Here};

fn increment_counter<R>(env: &mut R)
where
    R: GetMut<CounterKey, Here, Target = Counter>,
{
    let counter = env.get_mut();
    counter.increment();
}
```

Use `GetMut` sparingly. For shared concurrent state, prefer `TRef` or a service that owns its mutation rules.

## Explicit Paths

For values not at the head, use a path such as `ThereHere` or `Skip1`.

```rust,ignore
let logger: &Logger = env.get_path::<LoggerKey, ThereHere>();
```

Lookup is type-safe, but not magical: the path is part of the bound.

## No bind* Tag Shorthand

The current `effect!` macro only binds expressions whose type implements the bind protocol, usually `Effect<_, _, _>`.

```rust,ignore
effect! {
    let user = bind* fetch_user(id);
    user
}
```

It does not support `bind* DatabaseKey` service lookup shorthand. Access tagged contexts with `Context::get` / `Get`, or use derive-service helpers such as `UserRepository::use_`.

## NeedsX Supertraits

You can hide verbose `Get` bounds behind a trait.

```rust,ignore
pub trait NeedsDatabase: Get<DatabaseKey, Here, Target = Pool> {}
impl<R: Get<DatabaseKey, Here, Target = Pool>> NeedsDatabase for R {}
```

This is only a naming pattern; the compiler still verifies the underlying `Get` bound.

## Compile-Time Guarantees

If a function requires `NeedsDatabase`, callers must supply an environment type that satisfies that trait. Missing tagged cells are compile-time errors for HList `Context`s.
