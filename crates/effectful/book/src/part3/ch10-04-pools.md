# Pools — Reusing Expensive Resources

`Pool<A, E>` and `KeyedPool<K, A, E>` manage reusable values with a capacity gate. A checkout is tied to a `Scope`: when the scope closes, the value is returned to the idle list unless invalidated.

## Pool

```rust,ignore
use effectful::Pool;

let pool_effect = Pool::make(10, || open_connection("postgres://localhost/app"));
let pool: Pool<Connection, DbError> = run_blocking(pool_effect, ())?;
```

`Pool::make(capacity, factory)` returns `Effect<Pool<A, E>, Never, ()>`. The factory is an effect that creates a fresh value when no reusable idle value is available.

## Checking Out

```rust,ignore
use effectful::{Scope, run_blocking};

let scope = Scope::make();
let conn = run_blocking(pool.get(), scope.clone())?;

// Use conn while scope is open.

scope.close(); // returns conn to the pool's idle list
```

`pool.get()` returns `Effect<A, E, Scope>`. It acquires capacity, reuses an idle value or runs the factory, and registers a finalizer in the provided scope.

## Invalidating Values

```rust,ignore
run_blocking(pool.invalidate(conn.clone()), ())?;
```

Invalidated values are not reused when their checkout scope closes.

## KeyedPool

Use `KeyedPool` when resource creation depends on a key.

```rust,ignore
use effectful::KeyedPool;

let pools_effect = KeyedPool::make(20, |key: DbRole| open_connection_for(key));
let pools: KeyedPool<DbRole, Connection, DbError> = run_blocking(pools_effect, ())?;

let scope = Scope::make();
let primary = run_blocking(pools.get(DbRole::Primary), scope.clone())?;
let replica = run_blocking(pools.get(DbRole::Replica), scope.clone())?;

scope.close();
```

Capacity is global across all keys. Idle values are stored per key.

## TTL

Both pool types have `make_with_ttl`. Idle values older than the TTL are discarded on checkout.

```rust,ignore
let pool = run_blocking(
    Pool::make_with_ttl(10, Duration::from_secs(300), || open_connection(url.clone())),
    (),
)?;
```

## Pool as a Service

In applications, build the pool at startup and provide it as a service or context value. Business code should depend on the pool abstraction and checkout inside an explicit `Scope`.
