# The Unknown Type — Unvalidated Wire Data

`Unknown` is effectful's dynamic input tree for schema decoding. It represents JSON-like data without trusting its shape.

## Creating Unknown Values

```rust,ignore
use std::collections::BTreeMap;
use effectful::schema::Unknown;

let s = Unknown::String("hello".to_string());
let n = Unknown::I64(42);
let b = Unknown::Bool(true);
let null = Unknown::Null;
let arr = Unknown::Array(vec![Unknown::I64(1), Unknown::I64(2)]);

let mut fields = BTreeMap::new();
fields.insert("name".to_string(), Unknown::String("Alice".to_string()));
fields.insert("age".to_string(), Unknown::I64(30));
let obj = Unknown::Object(fields);
```

With the `schema-serde` feature, convert from `serde_json::Value` using `unknown_from_serde_json`.

```rust,ignore
use effectful::schema::unknown_from_serde_json;

let value: serde_json::Value = serde_json::from_str(input)?;
let unknown = unknown_from_serde_json(value);
```

## Why Not serde_json::Value Directly?

`serde_json::Value` is useful at the edge, but schemas need a stable internal representation with effectful's parse errors and combinators. `Unknown` gives schema decoders one input model independent of where the data came from.

## Inspecting Unknown Values

Most code should not inspect `Unknown` directly. Decode it with a `Schema`. For debugging or custom decoders, match on the enum variants.

```rust,ignore
match &unknown {
    Unknown::Object(fields) => { /* inspect fields */ }
    Unknown::Array(items) => { /* inspect items */ }
    Unknown::String(value) => { /* inspect string */ }
    Unknown::I64(value) => { /* inspect integer */ }
    Unknown::F64(value) => { /* inspect float */ }
    Unknown::Bool(value) => { /* inspect bool */ }
    Unknown::Null => { /* inspect null */ }
}
```

## The Parse Boundary

Use `Unknown` at I/O boundaries, then decode once into trusted domain types.

```rust,ignore
use effectful::schema::{Unknown, string};

let raw = Unknown::String("alice@example.com".to_string());
let email = string::<()>().decode_unknown(&raw)?;
```

Nothing beyond the parse boundary should accept `Unknown`; domain functions should accept validated types.
