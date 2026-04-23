# Scopes and Finalizers — Guaranteed Cleanup

`Scope` is a finalizer registry. Finalizers are plain boxed closures that receive an `Exit<(), Never>` and return `Effect<(), Never, ()>`.

## Creating a Scope

```rust,ignore
use effectful::{Effect, Exit, Never, Scope, scope_with};

let result = scope_with(|scope| {
    effect! {
        let conn = bind* open_connection();

        let conn_for_close = conn.clone();
        let added = scope.add_finalizer(Box::new(move |_exit: Exit<(), Never>| {
            conn_for_close.close()
        }));

        if !added {
            return Err(AppError::ScopeClosed);
        }

        let data = bind* fetch_data(&conn);
        process(data)
    }
});
```

`scope_with` creates a fresh scope, runs the returned effect, then closes the scope. Closing runs registered finalizers.

## Finalizers Always Run on Close

```rust,ignore
let scope = Scope::make();
let added = scope.add_finalizer(Box::new(|_exit| cleanup_temp_file()));
assert!(added);

scope.close();
```

Finalizers run when `close` / `close_with_exit` is called. `close` is idempotent and returns `true` only for the first close.

## Multiple Finalizers

Finalizers run in reverse registration order.

```rust,ignore
scope.add_finalizer(Box::new(|_| close_connection(conn)));
scope.add_finalizer(Box::new(|_| rollback_transaction(txn)));
scope.add_finalizer(Box::new(|_| close_cursor(cursor)));

scope.close();
// Runs: close_cursor, rollback_transaction, close_connection
```

Register parent resources first and child resources last.

## Scope Inheritance

Scopes can be nested manually.

```rust,ignore
let outer = Scope::make();
let inner = outer.fork();

inner.add_finalizer(Box::new(|_| cleanup_inner()));
outer.add_finalizer(Box::new(|_| cleanup_outer()));

outer.close(); // closes children before outer finalizers
```

Use `Scope::fork` for child scopes and `Scope::extend` to reparent an existing open scope.
