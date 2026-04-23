# TRef — Transactional References

`TRef<T>` is the fundamental mutable cell in effectful's STM system. A `TRef` is read and written through `Stm` operations, then committed atomically with `commit` / `atomically`.

## Creating a TRef

```rust,ignore
use effectful::{TRef, commit, run_blocking};

let counter: TRef<i32> = run_blocking(commit(TRef::make(0)), ())?;
let balance: TRef<f64> = run_blocking(commit(TRef::make(1000.0)), ())?;
```

`TRef::make(value)` returns `Stm<TRef<T>, ()>`, not a `TRef` directly. That keeps allocation inside the same transactional model as reads and writes.

## Transactional Operations

All operations return `Stm<_, E>` descriptions. Nothing changes until the transaction is committed.

```rust,ignore
use effectful::{Stm, TRef};

let counter: TRef<i32> = /* built with TRef::make */;

let read_op: Stm<i32, ()> = counter.read_stm();
let write_op: Stm<(), ()> = counter.write_stm(42);
let update_op: Stm<(), ()> = counter.update_stm(|n| n + 1);
let modify_op: Stm<i32, ()> = counter.modify_stm(|n| (n, n + 1));
```

Use `update_stm` when you only need to write a new value. Use `modify_stm` when the transaction should return one value and store another.

## Composing Transactions

There is currently no `stm!` macro. Compose `Stm` values with `flat_map` and `map`.

```rust,ignore
use effectful::{Stm, TRef};

let counter: TRef<i32> = /* ... */;
let total: TRef<i32> = /* ... */;

let transaction: Stm<(), ()> = counter.read_stm().flat_map(move |count| {
    let total = total.clone();
    counter
        .write_stm(count + 1)
        .flat_map(move |_| total.update_stm(move |sum| sum + count))
});
```

## Sharing TRefs

`TRef` is cloneable and internally shared. Wrap it in `Arc` only when your surrounding ownership model needs an `Arc`.

```rust,ignore
use std::sync::Arc;
use effectful::TRef;

let shared: Arc<TRef<i32>> = Arc::new(run_blocking(commit(TRef::make(0)), ())?);
```

## TRef vs. `Mutex<T>`

| Property | TRef | Mutex |
|----------|------|-------|
| Composable across multiple cells | Yes | Manual lock ordering |
| Commit is atomic | Yes | Only while locks are held |
| Transaction body can do I/O | No | Technically yes, but risky |
| Retry on conflict | Yes | No |

Use `TRef` for short, composable state mutations. Use `Mutex` for non-transactional shared state, especially when you cannot model the operation as a short pure transaction.
