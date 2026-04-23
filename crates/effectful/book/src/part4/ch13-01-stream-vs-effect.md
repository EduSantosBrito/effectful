# Stream vs Effect — When to Use Each

The choice is about cardinality.

```text
Effect<A, E, R>  -> produces exactly one A or fails
Stream<A, E, R>  -> produces zero or more A values or fails
```

## Concrete Examples

```rust,ignore
fn get_user(id: u64) -> Effect<User, DbError, Db>
fn all_users() -> Stream<User, DbError, Db>

fn count_orders() -> Effect<u64, DbError, Db>
fn export_orders() -> Stream<Order, DbError, Db>
```

If you fetch 10 million rows into a `Vec` and return it as an `Effect`, you can run out of memory. A `Stream` lets consumers process chunks incrementally.

## Stream Transformations

```rust,ignore
all_users()
    .filter(Box::new(|u: &User| u.is_active()))
    .map(UserSummary::from)
    .take(100)
```

Current element operators include `map`, `filter`, `take`, `take_while`, `drop_while`, `map_effect`, and `map_par_n`.

## Collecting a Stream into an Effect

When all results fit in memory, use `run_collect`.

```rust,ignore
let users: Effect<Vec<User>, DbError, Db> = all_users().run_collect();
```

For large results, prefer a fold or a sink.

```rust,ignore
let count: Effect<usize, DbError, Db> = all_users().run_fold(0, |acc, _| acc + 1);
```

## Converting Effect to Stream

`Stream::from_effect` expects an effect that produces a `Vec<A>`.

```rust,ignore
use effectful::Stream;

let one_user: Effect<Vec<User>, DbError, Db> = get_user(1).map(|user| vec![user]);
let stream: Stream<User, DbError, Db> = Stream::from_effect(one_user);
```

For pure finite streams, use `Stream::from_iterable`.

```rust,ignore
let numbers = Stream::from_iterable([1, 2, 3]);
```

## The Rule

| Need | Use |
|------|-----|
| One result | `Effect` |
| Many results | `Stream` |
| All stream results in memory | `stream.run_collect()` |
| Aggregated result | `stream.run_fold(init, f)` or `Sink::fold_left` |
| Custom consumer | `sink.run(stream)` |
