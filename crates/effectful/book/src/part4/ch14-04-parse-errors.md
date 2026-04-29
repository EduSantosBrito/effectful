# ParseErrors — Structured Parse Failures

`ParseError` represents one schema decoding failure. `ParseErrors` is a small aggregate wrapper used when APIs want to return more than one issue.

## ParseError vs ParseErrors

```rust,ignore
use effectful::schema::{ParseError, ParseErrors};

let e = ParseError::new("age", "age must be positive");
let one = ParseErrors::one(e.clone());
let many = ParseErrors::new(vec![e]);
```

`ParseError` has two public fields: `path` and `message`.

```rust,ignore
let err = ParseError::new("users.0.age", "expected i64");
assert_eq!(err.path, "users.0.age");
assert_eq!(err.message, "expected i64");
```

## Path Tracking

Schema combinators prefix paths as they decode nested data.

```rust,ignore
use effectful::schema::{Unknown, array, i64};

let schema = array(i64::<()>());
let raw = Unknown::Array(vec![Unknown::I64(1), Unknown::String("oops".to_string())]);

let err = schema.decode_unknown(&raw).unwrap_err();
assert_eq!(err.path, "1");
```

`struct_`, `struct3`, and `struct4` prefix field names. `array` prefixes element indexes.

## Accumulation Status

The current schema decoders generally short-circuit on the first failure for `decode_unknown`, but `decode_unknown_all` accumulates nested field errors for object and tuple schemas (including `struct_`/`struct3`/`struct4`, `tuple`/`tuple3`/`tuple4`), as well as array elements and union arm diagnostics. `ParseErrors` exists as the aggregate type for boundaries or custom validators that collect multiple `ParseError` values themselves.

```rust,ignore
fn validate_user(raw: &Unknown) -> Result<User, ParseErrors> {
    let mut issues = Vec::new();

    if let Err(err) = name_schema().decode_unknown(raw) {
        issues.push(err);
    }
    if let Err(err) = age_schema().decode_unknown(raw) {
        issues.push(err);
    }

    if issues.is_empty() {
        build_user(raw).map_err(ParseErrors::one)
    } else {
        Err(ParseErrors::new(issues))
    }
}
```

## API Boundary Conversion

Convert parse issues into your API error type at the boundary.

```rust,ignore
#[derive(Debug)]
enum ApiError {
    Validation(Vec<FieldError>),
}

#[derive(Debug)]
struct FieldError {
    field: String,
    message: String,
}

fn to_api_errors(errs: ParseErrors) -> ApiError {
    ApiError::Validation(
        errs.issues
            .into_iter()
            .map(|e| FieldError { field: e.path, message: e.message })
            .collect(),
    )
}
```

## Parse Errors in Effects

Schema decoding returns `Result`. Lift it into an `Effect` by mapping the error into your effect error channel.

```rust,ignore
effect! {
    let req = create_user_schema()
        .decode_unknown(&raw)
        .map_err(|err| ApiError::Validation(ParseErrors::one(err)))?;

    bind* create_user(req)
}
```

## Display

`ParseErrors` implements `Display` by printing each issue on its own line. Empty paths omit the `path: ` prefix.

```text
name: must not be empty
age: age must be positive
```

## Summary

| Type | Meaning |
|------|---------|
| `ParseError` | One failure with `path` and `message` |
| `ParseErrors` | Aggregate `{ issues: Vec<ParseError> }` |
| `ParseErrors::one(err)` | Build a single-issue aggregate |
| `ParseErrors::new(vec)` | Build an aggregate from collected issues |
