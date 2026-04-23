# R as Documentation — Self-Describing Functions

The `R` parameter is often described as "the environment type." That's true, but it undersells the practical benefit. `R` is *living documentation* that the compiler enforces.

## The Signature Tells the Story

Consider two versions of the same function:

```rust,ignore
// Version A: traditional async
async fn process_order(order: Order) -> Result<Receipt, Error> {
    // What does this use? Read the body to find out.
    // Database? PaymentGateway? Email? Metrics?
    // You'll have to trace through 200 lines to know.
}

// Version B: effect-based
fn process_order(order: Order) -> Effect<Receipt, OrderError, (Database, PaymentGateway, EmailService, Logger)> {
    // What does this use? Look at the signature.
    // Database ✓, PaymentGateway ✓, EmailService ✓, Logger ✓
    // Done.
}
```

Version B's type is self-describing. You don't need to read the implementation to understand its dependency surface.

## Code Review Benefits

In a pull request, `R` changes are visible in the diff. If someone adds a call to `send_metrics()` inside `process_order` and the `MetricsClient` wasn't previously in `R`, the function signature must change:

```diff
- fn process_order(order: Order) -> Effect<Receipt, OrderError, (Database, PaymentGateway, EmailService, Logger)>
+ fn process_order(order: Order) -> Effect<Receipt, OrderError, (Database, PaymentGateway, EmailService, Logger, MetricsClient)>
```

This diff is in the function signature — impossible to miss. With traditional parameters or singletons, new dependencies can silently appear in implementation bodies.

## Refactoring Safety

When you refactor and remove a dependency, the `R` type shrinks. Callers that construct a concrete environment may need to simplify that environment too.

```rust,ignore
// After removing Logger from process_order:

// Before: process_order required AppEnv { db, logger }
let result = run_blocking(process_order(order), AppEnv { db, logger })?;

// After: process_order only requires Database
let result = run_blocking(process_order(order), db)?;
```

The compiler guides the cleanup when the environment type changes. Traditional singleton-style code can leave stale dependencies silently lingering.

## Testing Clarity

When writing a test, `R` tells you exactly what you need to mock:

```rust,ignore
#[test]
fn test_process_order() {
    // R = (Database, PaymentGateway, EmailService, Logger)
    // So the test needs these four — no more, no less
    let result = run_test(
        process_order(test_order()),
        (mock_db(), mock_payment(), mock_email(), test_logger()),
    );
    assert!(matches!(result, Exit::Success(_)));
}
```

There's no "I wonder if this also touches the metrics service" uncertainty. The type says it doesn't. If you're missing a mock, the code won't compile.

## R is Not Magic

It's important to understand that `R` is just a type parameter. The "compile-time DI" property comes from:

1. Functions declaring what they need in `R`
2. Runners requiring an actual environment value of type `R`
3. Composition preserving environment requirements in the resulting effect type

There's no reflection, no registration, no framework. Just types.

The next chapter shows how `Tags` and `Context` make this scale beyond simple tuples — handling large, complex dependency graphs without positional ambiguity.
