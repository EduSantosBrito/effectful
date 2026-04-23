# Sinks — Consuming Streams

A `Sink<Out, In, E, R>` reduces a `Stream<In, E, R>` into an `Out`. It is a struct with a driver, not a trait you implement directly.

## Built-in Sinks

```rust,ignore
use effectful::Sink;

let collect = Sink::<Vec<User>, User, DbError, Db>::collect();
let total = Sink::fold_left(0u64, |acc, order: Order| acc + order.amount);
let drain = Sink::<(), Event, EventError, EventEnv>::drain();
```

Run a sink with `sink.run(stream)`.

```rust,ignore
let users: Effect<Vec<User>, DbError, Db> = Sink::collect().run(all_users());
let total: Effect<u64, DbError, Db> = total.run(orders());
```

## Stream Consumer Methods

For common cases, stream methods are often simpler.

```rust,ignore
let users: Effect<Vec<User>, DbError, Db> = all_users().run_collect();

let total: Effect<u64, DbError, Db> = orders()
    .run_fold(0u64, |acc, order| acc + order.amount);

let logged: Effect<(), EventError, EventEnv> = events()
    .run_for_each_effect(|event| log_event(event));
```

## Sink Composition

`Sink::zip` combines two fold-based sinks into one pass.

```rust,ignore
let count = Sink::fold_left(0usize, |n, _order: Order| n + 1);
let total = Sink::fold_left(0u64, |sum, order: Order| sum + order.amount);

let both = count.zip(total);
let effect: Effect<(usize, u64), DbError, Db> = both.run(orders());
```

`zip` panics if either sink was not created with `fold_left` / `from_fold`.

## Other Built-ins

| Sink | Purpose |
|------|---------|
| `Sink::collect()` | Collect elements into `Vec<In>` |
| `Sink::collect_all_while(pred)` | Collect until predicate first fails |
| `Sink::collect_all_until(pred)` | Collect until predicate first succeeds |
| `Sink::fold_left(init, f)` | Left fold |
| `Sink::drain()` | Discard all elements |
| `Sink::to_queue(queue)` | Offer each element to a queue |
| `Sink::collect_to_map()` | Collect `(K, V)` pairs into `EffectHashMap` |

## Custom Sinks

The current API does not expose a public trait for custom chunk callbacks. Build custom consumers with `Stream::run_fold`, `run_fold_effect`, `run_for_each_effect`, or compose existing `Sink` constructors.
