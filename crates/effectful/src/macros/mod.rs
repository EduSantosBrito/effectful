//! Declarative macros (`pipe!`, `ctx!`, …).
//!
//! They are implemented in the **`effectful_macro`** crate and re-exported at the `effectful` crate
//! root. Procedural **`effect!`** is implemented in **`effectful_proc_macro`** (Rust cannot combine
//! `macro_rules!` and `#[proc_macro]` in one crate).
//!
//! Submodule **`effect`** keeps the stable path `macros::effect::effect` for the procedural macro.

/// Re-exports the procedural `effect!` macro (`effectful_proc_macro::effect`) at `macros::effect::effect`.
pub mod effect {
  pub use effectful_proc_macro::effect;
}

pub use effectful_macro::{ctx, err, layer_graph, layer_node, pipe, req, service_def, service_key};
