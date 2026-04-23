# Validation and Refinement — Constrained Types

Schemas parse structure. Validation adds constraints: an age must be positive, an email must contain `@`, a price must have at most two decimal places.

## refine / filter

`refine` and `filter` attach a predicate to an existing schema. Parsing succeeds only when the base schema succeeds and the predicate returns `true`.

```rust,ignore
use effectful::schema::{i64, refine, string, filter};

let age_schema = refine(
    i64::<()>(),
    |n| (0..=150).contains(n),
    "age must be between 0 and 150",
);

let non_empty = filter(
    string::<()>(),
    |s: &String| !s.is_empty(),
    "must not be empty",
);
```

If the predicate fails, decoding returns `ParseError::new("", message)`.

## Fallible Transformation

Use `transform` when conversion can fail or when the semantic type differs from the wire type.

```rust,ignore
use effectful::schema::{ParseError, string, transform};

let url_schema = transform(
    string::<()>(),
    |s| url::Url::parse(&s).map_err(|e| ParseError::new("", format!("invalid URL: {e}"))),
    |url: url::Url| url.to_string(),
);
```

The decode closure returns `Result<B, ParseError>`. The encode closure maps the semantic value back to the base schema's semantic type.

## Brand

`Brand<A, B>` is a zero-cost nominal wrapper. Use `Brand::nominal` when the value was already validated, or `RefinedBrand` when construction should validate.

```rust,ignore
use effectful::schema::{Brand, RefinedBrand};

struct EmailMarker;
type Email = Brand<String, EmailMarker>;

let email = Brand::nominal("alice@example.com".to_string());

let make_email = RefinedBrand::<String, EmailMarker>::new(|s| {
    if s.contains('@') {
        Ok(())
    } else {
        Err("invalid email".to_string())
    }
});

let checked: Email = make_email.try_make("alice@example.com".to_string())?;
```

Now APIs can demand `Email` instead of a raw `String`.

```rust,ignore
fn send_welcome(to: Email) -> Effect<(), MailError, Mailer> { /* ... */ }
```

## HasSchema

`HasSchema` attaches a canonical schema to a type family. The trait exposes associated types for semantic value, wire value, and schema marker.

```rust,ignore
use effectful::schema::{HasSchema, Schema, i64};

struct UserIdSchema;

impl HasSchema for UserIdSchema {
    type A = i64;
    type I = i64;
    type E = ();

    fn schema() -> Schema<Self::A, Self::I, Self::E> {
        i64::<()>()
    }
}
```

Use `HasSchema` for generic tooling that needs to ask for a canonical schema without knowing how it is built.

## Summary

| Tool | When to use |
|------|-------------|
| `refine` / `filter` | Predicate on a parsed value |
| `transform` | Fallible conversion or semantic/wire conversion |
| `Brand::nominal` | Nominal wrapper after validation elsewhere |
| `RefinedBrand` | Validating branded constructor |
| `HasSchema` | Attach a canonical schema to a type-level provider |
