//! Declarative macros (`pipe!`, `ctx!`, …).
//!
//! They are implemented in the **`effect-macro`** crate and re-exported at the `effect` crate
//! root. Procedural **`effect!`** is implemented in **`effect-proc-macro`** (Rust cannot combine
//! `macro_rules!` and `#[proc_macro]` in one crate).
//!
//! Submodule **`effect`** keeps the stable path `macros::effect::effect` for the procedural macro.

/// Re-exports the procedural `effect!` macro (`effect_proc_macro::effect`) at `macros::effect::effect`.
pub mod effect {
  pub use effect_proc_macro::effect;
}

pub use effect_macro::{ctx, err, layer_graph, layer_node, pipe, req, service_def, service_key};
