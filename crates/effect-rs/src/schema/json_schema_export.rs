//! Minimal JSON Schema (draft-07 style) fragments for **primitives** — not a full exporter for
//! composed [`crate::schema::parse::Schema`] values (those are closure-backed and not introspectable).
//!
//! Use for docs, OpenAPI-ish hints, and tests. Requires **`schema-serde`**.
//!
//! See [`TESTING.md`](../../../../TESTING.md).

use serde_json::{Value, json};

/// `"type": "string"`.
pub fn type_string() -> Value {
  json!({ "type": "string" })
}

/// `"type": "integer"` without `format`.
pub fn type_integer() -> Value {
  json!({ "type": "integer" })
}

/// `"type": "number"`.
pub fn type_number() -> Value {
  json!({ "type": "number" })
}

/// `"type": "boolean"`.
pub fn type_boolean() -> Value {
  json!({ "type": "boolean" })
}

/// `{ "type": "array", "items": items }`.
pub fn type_array(items: Value) -> Value {
  json!({ "type": "array", "items": items })
}

/// `{ "type": "object", "additionalProperties": value_schema }` for string-keyed records.
pub fn type_record(value_schema: Value) -> Value {
  json!({
    "type": "object",
    "additionalProperties": value_schema
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn string_fragment_is_stable() {
    assert_eq!(type_string(), json!({"type": "string"}));
  }
}
