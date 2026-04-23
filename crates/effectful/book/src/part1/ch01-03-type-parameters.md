# The Three Type Parameters

Every `Effect` carries three type parameters: `Effect<A, E, R>`. These aren't arbitrary — they answer the three fundamental questions every computation must address:

- **A** — What do I produce when I succeed?
- **E** — What do I produce when I fail?
- **R** — What do I need in order to run?

Let's examine each one.

## A: The Answer

The `A` parameter is the success type — what you get back when everything goes right.

```rust,ignore
use effectful::{Effect, succeed};

// This effect produces an i32 on success
let answer: Effect<i32, String, ()> = succeed(42);

// This effect produces a User on success
let user_effect: Effect<User, DbError, ()> = succeed(User::new("Alice"));
```

If you're familiar with `Result<T, E>`, think of `A` as the `T`. It's what you're hoping to get.

When you transform an effect with `.map()`, you're changing the `A`:

```rust,ignore
let numbers: Effect<i32, String, ()> = succeed(21);
let doubled: Effect<i32, String, ()> = numbers.map(|n| n * 2);
let stringified: Effect<String, String, ()> = doubled.map(|n| n.to_string());
```

Each `.map()` transforms the success value while preserving the error type and requirements.

## E: The Error

The `E` parameter is the failure type — what you get back when something goes wrong.

```rust,ignore
use effectful::{Effect, fail};

// This effect always fails with a String error
let failure: Effect<i32, String, ()> = fail("something went wrong".to_string());

// This effect can fail with a DbError
let user: Effect<User, DbError, ()> = fetch_user_from_db(42);
```

Again, if you know `Result<T, E>`, think of `E` as the `E`. It's what you're worried might happen.

You can transform error types with `.map_error()`:

```rust,ignore
let db_effect: Effect<User, DbError, ()> = fetch_user(42);

// Convert DbError to a more general AppError
let app_effect: Effect<User, AppError, ()> = db_effect.map_error(|e| AppError::Database(e));
```

Unlike traditional error handling where you sprinkle `.map_err()` everywhere, with effects you typically handle error transformation at specific boundaries — when composing larger effects from smaller ones, or when exposing an API.

## R: The Requirements

Here's where effects get interesting. The `R` parameter represents the **environment** — the dependencies this effect needs in order to run.

```rust,ignore
// This effect needs nothing to run — R is ()
let standalone: Effect<i32, String, ()> = succeed(42);

// This effect needs a Database to run
fn get_user(id: u64) -> Effect<User, DbError, Database> {
    // ... implementation that uses the database
}

// This effect needs both a Database AND a Logger
fn get_user_logged(id: u64) -> Effect<User, DbError, (Database, Logger)> {
    // ... implementation that uses both
}
```

The key insight: **you cannot run an effect unless you provide its requirements.**

```rust,ignore
let needs_db: Effect<User, DbError, Database> = get_user(42);

// This won't compile! We haven't satisfied the Database requirement.
// run_blocking(needs_db);  // ERROR: Database not provided

// We pass the required environment to the runner
let user = run_blocking(needs_db, my_database)?;
```

You can also capture an environment with `.provide_env(my_database)` to get an `Effect<User, DbError, ()>`.

## Why R Matters

The `R` parameter is why effectful can offer compile-time dependency injection.

Consider this function signature:

```rust,ignore
fn process_order(order: Order) -> Effect<Receipt, OrderError, (Database, PaymentGateway, EmailService, Logger)>
```

Just from the type, you know:
- This produces a `Receipt` on success
- It can fail with `OrderError`
- It requires four services to run

You don't need to read the implementation. You don't need to trace through function calls. The type tells you exactly what dependencies are involved.

And the compiler enforces it. If you try to run this effect without providing all four services, you get a compile error. No runtime "service not found" exceptions. No forgetting to initialize something.

## R Flows Through Composition

When you combine effects, the bound effects must agree on one environment type. Use a shared environment type when multiple services are needed:

```rust,ignore
fn get_user(id: u64) -> Effect<User, DbError, Database> { ... }
fn send_email(to: &str, body: &str) -> Effect<(), EmailError, EmailService> { ... }

fn notify_user(id: u64) -> Effect<(), AppError, AppEnv> {
    effect! {
        let user = bind* get_user(id)
            .zoom_env(|env: &mut AppEnv| env.database.clone())
            .map_error(AppError::Db);
        bind* send_email(&user.email, "Hello!")
            .zoom_env(|env: &mut AppEnv| env.email.clone())
            .map_error(AppError::Email);
    }
}
```

The `notify_user` function now documents that callers must supply `AppEnv`, and each inner effect explicitly projects the part it needs.

## The Unit Environment: ()

When `R = ()`, the effect is self-contained. It doesn't need anything from the outside world to run:

```rust,ignore
let standalone: Effect<i32, String, ()> = succeed(42);

// Can run immediately — no dependencies
let result = run_blocking(standalone, ());
```

Most effects start with requirements and gradually have them satisfied as you move toward the "edge" of your program:

```rust,ignore
// Deep in your code: many requirements
fn business_logic() -> Effect<Result, Error, (Db, Cache, Logger, Config)>

// At the edge: provide everything
fn main() {
    let db = connect_database();
    let cache = connect_cache();
    let logger = setup_logger();
    let config = load_config();

    let env = AppEnv { db, cache, logger, config };

    run_blocking(business_logic(), env);
}
```

## Reading Effect Signatures

Let's practice reading some signatures:

```rust,ignore
// Produces String, never fails, needs nothing
Effect<String, Never, ()>

// Produces i32, can fail with ParseError, needs nothing
Effect<i32, ParseError, ()>

// Produces User, can fail with DbError, needs Database
Effect<User, DbError, Database>

// Produces (), can fail with AppError, needs Database, Cache, and Logger
Effect<(), AppError, (Database, Cache, Logger)>
```

With practice, you'll read these as fluently as you read `Result<T, E>`. The extra `R` parameter becomes second nature.

## What's Next

We've seen that effects are descriptions, not actions. We've seen that `Effect<A, E, R>` encodes success type, error type, and requirements.

But we haven't answered the obvious question: why does this matter? Why is it better to describe computations than to just do them?

The answer is laziness. And laziness, it turns out, is a superpower.
