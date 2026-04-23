# TQueue, TMap, TSemaphore — Transactional Collections

effectful provides STM-aware collection types. Their constructors and operations return `Stm` values, so they compose with `TRef` reads/writes and commit atomically.

## TQueue

```rust,ignore
use effectful::{Stm, TQueue};

let queue_stm: Stm<TQueue<Job>, ()> = TQueue::bounded(100);
let unbounded_stm: Stm<TQueue<Job>, ()> = TQueue::unbounded();

let offer: Stm<bool, ()> = queue.offer(job); // false when bounded and full
let take: Stm<Job, ()> = queue.take();       // retries while empty
```

`TQueue::offer` does not block when a bounded queue is full; it returns `false`. `TQueue::take` retries when the queue is empty.

### Producer-Consumer Pattern

```rust,ignore
fn producer(queue: TQueue<Job>, jobs: Vec<Job>) -> Effect<(), (), ()> {
    effect! {
        for job in jobs {
            let accepted = bind* commit(queue.offer(job));
            if !accepted {
                return Err(());
            }
        }
    }
}

fn consumer(queue: TQueue<Job>) -> Effect<(), JobError, ()> {
    effect! {
        loop {
            let job = bind* commit(queue.take());
            bind* process_job(job);
        }
    }
}
```

## TMap

```rust,ignore
use effectful::{Stm, TMap};

let map_stm: Stm<TMap<String, User>, ()> = TMap::make();

let get: Stm<Option<User>, ()> = map.get(&"alice".to_string());
let set: Stm<(), ()> = map.set("alice".to_string(), alice_user);
let delete: Stm<(), ()> = map.delete(&"alice".to_string());
```

`TMap` is a transactional hash map. Reading from a `TMap` and updating a `TRef` in the same transaction is atomic.

```rust,ignore
let transaction = user_map.get(&"alice".to_string()).flat_map(move |user| {
    access_counter
        .update_stm(|count| count + 1)
        .map(move |_| user)
});

let effect = commit(transaction);
```

## TSemaphore

```rust,ignore
use effectful::{Stm, TSemaphore};

let sem_stm: Stm<TSemaphore, ()> = TSemaphore::make(10);

let acquire: Stm<(), ()> = sem.acquire(); // retries while zero
let release: Stm<(), ()> = sem.release();
```

`TSemaphore` is a transactional permit counter. `acquire` decrements by one or retries when no permits are available; `release` increments by one.

```rust,ignore
let guarded = sem.acquire().flat_map(move |_| {
    update_shared_state().flat_map(move |result| {
        sem.release().map(move |_| result)
    })
});

let effect = commit(guarded);
```

## Summary

| Type | Purpose |
|------|---------|
| `TRef<T>` | Single transactional cell |
| `TQueue<T>` | Transactional FIFO queue |
| `TMap<K, V>` | Transactional hash map |
| `TSemaphore` | Transactional permit counter |

All compose as `Stm` values and commit atomically with other STM operations.
