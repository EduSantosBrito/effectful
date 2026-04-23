# Error Accumulation — Collecting All Failures

`effect!` and `flat_map` are fail-fast. That is correct for dependent steps, but independent validation sometimes needs to collect multiple failures.

## Fail-Fast Behavior

```rust,ignore
effect! {
    let name = bind* validate_name(&input.name);     // if this fails, stop
    let email = bind* validate_email(&input.email);  // not run after name failure
    let age = bind* validate_age(input.age);
    User { name, email, age }
}
```

Use fail-fast composition when each step depends on earlier successful values.

## Manual Accumulation

The current root API does not include `validate_all` or `partition` helpers. For independent validations, run them as plain `Result`-returning checks or explicitly run each effect and collect errors.

```rust,ignore
let mut errors = Vec::new();

if let Err(error) = run_blocking(validate_name(&input.name), ()) {
    errors.push(error);
}
if let Err(error) = run_blocking(validate_email(&input.email), ()) {
    errors.push(error);
}
if let Err(error) = run_blocking(validate_age(input.age), ()) {
    errors.push(error);
}

if errors.is_empty() {
    Ok(())
} else {
    Err(errors)
}
```

Keep this pattern at validation boundaries. Inside business workflows, prefer typed fail-fast effects.

## Combining Error Types

When composing effects with different error types, use `map_error` or `flat_map_union` with `Or`.

```rust,ignore
use effectful::Or;

type BothErrors = Or<DbError, NetworkError>;

let combined: Effect<Data, BothErrors, ()> = db_fetch()
    .union_error::<NetworkError>()
    .flat_map(|db| network_fetch().map_error(Or::Right).map(move |net| merge(db, net)));
```

`Or<A, B>` lets you defer flattening into an application error until a boundary.

## ParseErrors

`ParseErrors` is an aggregate container for schema-style validation issues.

```rust,ignore
let errors = ParseErrors::new(vec![
    ParseError::new("name", "must not be empty"),
    ParseError::new("email", "invalid email"),
]);

for issue in errors.issues {
    eprintln!("{}: {}", issue.path, issue.message);
}
```

Current schema combinators usually return the first `ParseError`; build `ParseErrors` yourself when your boundary wants to report multiple issues.

## When to Accumulate vs. Short-Circuit

| Situation | Use |
|-----------|-----|
| Dependent steps | `effect!` / `flat_map` |
| Independent form validation | Manual accumulation |
| Batch import with partial success | Explicit loop collecting successes/failures |
| Schema boundary with many field issues | `ParseErrors::new(collected)` |
