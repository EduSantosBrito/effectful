# Context and HLists — The Heterogeneous Stack

`Context<L>` wraps a heterogeneous list of typed cells. In the tagged API, each cell is usually `Tagged<K, V>`.

## The Structure: Cons / Nil

```rust,ignore
use effectful::{Cons, Nil, Tagged};

type Empty = Nil;
type WithDb = Cons<Tagged<DatabaseKey, Pool>, Nil>;
type WithDbAndLogger = Cons<Tagged<DatabaseKey, Pool>, Cons<Tagged<LoggerKey, Logger>, Nil>>;
```

`Cons<Head, Tail>` prepends one item to a list. `Nil` is the empty list.

## Building Context Values

```rust,ignore
use effectful::{Cons, Context, Nil, tagged};

let env = Context::new(Cons(
    tagged::<DatabaseKey, _>(my_pool),
    Cons(tagged::<LoggerKey, _>(my_logger), Nil),
));
```

You can also prepend to an existing context:

```rust,ignore
let base = Context::new(Cons(tagged::<LoggerKey, _>(my_logger), Nil));
let full = base.prepend(tagged::<DatabaseKey, _>(my_pool));
```

## Access

`Context::get::<K>()` reads the head cell when it has key `K`.

```rust,ignore
let pool: &Pool = full.get::<DatabaseKey>();
```

For non-head cells, use `get_path::<K, P>()` with an explicit type-level path such as `ThereHere` / `Skip1`.

```rust,ignore
let logger: &Logger = full.get_path::<LoggerKey, ThereHere>();
```

This explicit path is why application code often wraps bounds in `NeedsX` traits or uses `ServiceContext` instead.

## Why HLists and Not HashMap?

An HList preserves each value's type in the environment type. That gives compile-time lookup and no runtime downcast. The cost is verbose types like `Cons<Tagged<A, V>, Cons<Tagged<B, W>, Nil>>`.

Use HList `Context` when you want maximum static structure. Use `ServiceContext` when you want a simpler runtime service table keyed by service type.
