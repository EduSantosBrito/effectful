# Stm and commit — Building Transactions

`Stm<A, E>` describes a transactional computation. It is lazy and pure with respect to the outside world: it can read/write transactional cells, fail with `E`, or retry.

## commit: Lift Stm into Effect

```rust,ignore
use effectful::{Effect, Stm, TRef, commit, run_blocking};

let ref_a: TRef<i32> = run_blocking(commit(TRef::make(1)), ())?;
let ref_b: TRef<i32> = run_blocking(commit(TRef::make(2)), ())?;

let transaction: Stm<i32, ()> = ref_a.read_stm().flat_map(move |a| {
    ref_b.read_stm().map(move |b| a + b)
});

let effect: Effect<i32, (), ()> = commit(transaction);
let result = run_blocking(effect, ())?;
```

`commit(stm)` returns `Effect<A, E, R>`. The transaction is executed when the effect runs. On commit conflicts or `Stm::retry()`, it retries.

## atomically

`atomically(stm)` is an alias for `commit(stm)`. It still returns an `Effect`; run it with `run_blocking`, `run_async`, or compose it with other effects.

```rust,ignore
use effectful::{atomically, run_blocking};

let effect = atomically(counter.modify_stm(|n| (n + 1, n + 1)));
let value = run_blocking(effect, ())?;
```

## Stm::fail

Transactions can fail with typed errors:

```rust,ignore
use effectful::{Stm, TRef};

fn withdraw(account: TRef<u64>, amount: u64) -> Stm<u64, InsufficientFunds> {
    account.read_stm().flat_map(move |balance| {
        if balance < amount {
            Stm::fail(InsufficientFunds)
        } else {
            account
                .write_stm(balance - amount)
                .map(move |_| balance - amount)
        }
    })
}
```

`Stm::fail(e)` aborts the current transaction immediately. `commit` propagates that `E` through the returned effect.

## Stm::retry

Sometimes a transaction should wait until a condition is true rather than fail:

```rust,ignore
use effectful::{Stm, TRef};

fn dequeue(queue: TRef<Vec<Item>>) -> Stm<Item, ()> {
    queue.read_stm().flat_map(move |items| {
        if items.is_empty() {
            Stm::retry()
        } else {
            let item = items[0].clone();
            queue.write_stm(items[1..].to_vec()).map(move |_| item)
        }
    })
}
```

`Stm::retry()` asks the commit loop to restart later. `TQueue::take` uses this behavior to wait while empty.

## Composing Transactions

Transactions compose with `flat_map` and `map`.

```rust,ignore
let big_transaction: Stm<(), AppError> = transfer_funds(from, to, amount)
    .flat_map(move |_| record_audit_log(from, to, amount));

let effect = commit(big_transaction);
```

The composed transaction commits as a unit. If any part fails, retries, or observes a conflict, the whole transaction does not partially commit.
