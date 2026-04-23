# Tags — Branding Values with Identity

A tag is a compile-time key. `Tagged<K, V>` stores a value `V` under key type `K`, so two values with the same runtime type can remain distinct in a typed context.

## Tag and Tagged

```rust,ignore
use effectful::{Tagged, tagged};

struct DatabaseTag;
struct CacheTag;

let db: Tagged<DatabaseTag, Pool> = tagged::<DatabaseTag, _>(connect_database());
let cache: Tagged<CacheTag, Pool> = tagged::<CacheTag, _>(connect_cache());

let pool_ref: &Pool = &db.value;
```

`Tag<K>` itself is a zero-sized phantom identity. Most code works with `Tagged<K, V>` values rather than constructing `Tag<K>` directly.

## Why Tags Help

`Tagged<DatabaseTag, Pool>` and `Tagged<CacheTag, Pool>` are different types even though both contain `Pool`. That prevents positional swaps.

```rust,ignore
fn needs_database(db: Tagged<DatabaseTag, Pool>) { /* ... */ }

let cache: Tagged<CacheTag, Pool> = tagged::<CacheTag, _>(pool);
needs_database(cache); // type error
```

## service_key!

The legacy `service_key!` macro declares a nominal key type.

```rust,ignore
use effectful::service_key;

service_key!(pub struct DatabaseKey);
service_key!(pub struct CacheKey);
```

Pair the key with a value using `Service<K, V>` / `Tagged<K, V>`.

```rust,ignore
type DatabaseService = effectful::Service<DatabaseKey, Pool>;
let db = effectful::service::<DatabaseKey, _>(pool);
```

For new service-style code, prefer `#[derive(Service)]` on the service struct and `ServiceContext`.

## NeedsX Supertraits

For HList contexts, a `NeedsX` trait can name a `Get` bound.

```rust,ignore
pub trait NeedsDatabase: Get<DatabaseKey, Target = Pool> {}
impl<R: Get<DatabaseKey, Target = Pool>> NeedsDatabase for R {}

pub fn get_user<R: NeedsDatabase>(id: u64) -> Effect<User, DbError, R> { /* ... */ }
```

This is a readability pattern, not a separate runtime feature.

## Summary

| Concept | Purpose |
|---------|---------|
| `Tag<K>` | Zero-sized key identity |
| `Tagged<K, V>` | Value `V` stored under key `K` |
| `tagged::<K, _>(v)` | Construct a tagged cell |
| `service_key!(pub struct K);` | Declare a nominal key type |
| `Service<K, V>` | Alias for `Tagged<K, V>` |
