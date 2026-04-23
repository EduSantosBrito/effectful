# Exit — Terminal Outcomes

`Exit<A, E>` records whether an effect-like computation succeeded or failed with a structured `Cause<E>`.

## The Exit Type

```rust,ignore
use effectful::{Cause, Exit};

enum Exit<A, E> {
    Success(A),
    Failure(Cause<E>),
}
```

The public constructors are `Exit::succeed(value)`, `Exit::fail(error)`, `Exit::die(message)`, and `Exit::interrupt(fiber_id)`.

## Getting an Exit

`run_blocking(effect, env)` and `run_async(effect, env)` return `Result<A, E>`. The test harness returns `Exit<A, E>`.

```rust,ignore
use effectful::{Exit, run_test};

let exit: Exit<User, DbError> = run_test(get_user(1), env);

match exit {
    Exit::Success(user) => println!("Got user: {}", user.name),
    Exit::Failure(Cause::Fail(DbError::NotFound)) => println!("User not found"),
    Exit::Failure(Cause::Die(message)) => eprintln!("Defect: {message}"),
    Exit::Failure(Cause::Interrupt(id)) => println!("Interrupted fiber {id:?}"),
    Exit::Failure(Cause::Both(_, _) | Cause::Then(_, _)) => println!("Composite failure"),
}
```

There is no `run_to_exit` helper in the current API.

## Converting Exit to Result

Use `into_result()` to get `Result<A, Cause<E>>`.

```rust,ignore
let result: Result<User, Cause<DbError>> = exit.into_result();
```

If you only want typed failures, pattern match on the cause.

```rust,ignore
let result: Result<User, AppError> = match exit.into_result() {
    Ok(user) => Ok(user),
    Err(Cause::Fail(e)) => Err(AppError::Expected(e)),
    Err(Cause::Die(message)) => Err(AppError::Defect(message)),
    Err(Cause::Interrupt(id)) => Err(AppError::Interrupted(id)),
    Err(cause) => Err(AppError::Composite(cause.pretty())),
};
```

There is no `into_result_or_panic` helper in the current API.

## Exit in Fibers

`FiberHandle::await_exit()` returns `Effect<Exit<A, E>, Never, ()>`. `FiberHandle::join().await` returns `Result<A, Cause<E>>`.

```rust,ignore
let exit: Exit<A, E> = run_async(handle.await_exit(), ()).await?;
let result: Result<A, Cause<E>> = handle.join().await;
```

Use `Exit` when you need to preserve all failure detail. Use `Result` when typed failures are enough.
