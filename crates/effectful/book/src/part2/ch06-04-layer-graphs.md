# Layer Graphs — Dependency Planning

`LayerGraph` is a planner for named layer nodes. It does not build layers itself; it computes a topological build order from `requires` and `provides` service names.

## Declaring a Layer Graph

```rust,ignore
use effectful::{LayerGraph, LayerNode};

let graph = LayerGraph::new([
    LayerNode::new("config", std::iter::empty::<&str>(), ["config"]),
    LayerNode::new("db", ["config"], ["db"]),
    LayerNode::new("cache", ["config"], ["cache"]),
    LayerNode::new("service", ["db", "cache"], ["service"]),
]);
```

Each node has:

- `id`: stable unique node id
- `requires`: service names it needs
- `provides`: service names it supplies

## Planning

```rust,ignore
let plan = graph.plan_topological()?;
assert_eq!(plan.build_order, vec!["config", "db", "cache", "service"]);
```

Sibling order is deterministic but should not be used as a semantic dependency. If one layer must precede another, express that with `requires` / `provides`.

## Planner Errors

`plan_topological` can fail with:

| Error | Meaning |
|-------|---------|
| `DuplicateNodeId` | Two nodes share an id |
| `ConflictingProvider` | More than one node provides the same service name |
| `MissingProvider` | A requirement has no provider |
| `CycleDetected` | Dependencies contain a cycle |

```rust,ignore
let bad_graph = LayerGraph::new([
    LayerNode::new("a", ["b"], ["a"]),
    LayerNode::new("b", ["a"], ["b"]),
]);

let err = bad_graph.plan_topological();
assert!(matches!(err, Err(LayerPlannerError::CycleDetected { .. })));
```

Use `error.to_diagnostic()` for user-facing messages and suggestions.

## Planning from STM

`LayerGraph::plan_topological_from_tref(&nodes_tref)` reads a `TRef<Vec<LayerNode>>` snapshot transactionally and plans from that snapshot.

## When to Use LayerGraph

Use `LayerGraph` when you need validation, diagnostics, or tool-visible dependency order. For a few layers in application code, direct layer composition is usually clearer.
