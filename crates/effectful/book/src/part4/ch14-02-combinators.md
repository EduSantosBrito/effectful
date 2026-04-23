# Schema Combinators — Describing Data Shapes

A schema describes how to decode wire data into a typed value and encode it back. In the current API the full shape is `Schema<A, I, E>`: semantic value `A`, wire/intermediate value `I`, and schema marker `E`.

## Primitive Schemas

```rust,ignore
use effectful::schema::{bool_, f64, i64, string};

let name = string::<()>(); // Schema<String, String, ()>
let age = i64::<()>();     // Schema<i64, i64, ()>
let price = f64::<()>();   // Schema<f64, f64, ()>
let active = bool_::<()>(); // Schema<bool, bool, ()>
```

Each primitive can decode its typed wire value with `decode`, and can decode an `Unknown` with `decode_unknown`.

## Struct-Like Schemas

`struct_`, `struct3`, and `struct4` decode named object fields into tuples. Use `transform` to map the tuple into a domain struct.

```rust,ignore
use effectful::schema::{ParseError, Schema, i64, string, struct_, transform};

#[derive(Clone)]
struct User {
    name: String,
    age: i64,
}

let tuple_schema = struct_("name", string::<()>(), "age", i64::<()>());

let user_schema = transform(
    tuple_schema,
    |(name, age)| Ok(User { name, age }),
    |user: User| (user.name, user.age),
);
```

If a field is missing or has the wrong type, `decode_unknown` returns a `ParseError` with the field path.

## Optional and Array Schemas

```rust,ignore
use effectful::schema::{array, optional, string};

let maybe_name = optional(string::<()>());
let tags = array(string::<()>());
```

`optional(schema)` accepts `Unknown::Null` as `None`. `array(schema)` decodes each element and prefixes parse errors with the failing index.

## Validation and Transformation

Use free combinators, not schema methods.

```rust,ignore
use effectful::schema::{ParseError, filter, string, transform};

let non_empty = filter(string::<()>(), |s| !s.is_empty(), "must not be empty");

let email_schema = transform(
    non_empty,
    |s| Email::parse(s).map_err(|e| ParseError::new("", format!("invalid email: {e}"))),
    |email: Email| email.into_string(),
);
```

`filter` keeps values satisfying a predicate. `transform` performs bidirectional conversion and may fail while decoding.

## Unions

`union_` tries a primary schema, then a fallback schema. For more than two branches, use `union_chain` from `schema::extra`.

```rust,ignore
use effectful::schema::{Schema, Unknown, union_};

let primary: Schema<UserId, Unknown, ()> = user_id_from_number();
let fallback: Schema<UserId, Unknown, ()> = user_id_from_string();
let user_id_schema = union_(primary, fallback);
```

## Running a Schema

Run schemas directly through their methods.

```rust,ignore
use effectful::schema::{Unknown, i64};

let raw = Unknown::I64(30);
let age = i64::<()>().decode_unknown(&raw)?;
```

`decode` works on the schema's typed wire value `I`. `decode_unknown` works on `Unknown` trees at I/O boundaries.
