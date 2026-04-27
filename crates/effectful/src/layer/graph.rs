//! Layer graph planner primitives and topological sorting (Stratum 5 — dependency resolution).
//!
//! The core topological planner composes from Strata 0–4.  The STM-backed
//! [`LayerGraph::plan_topological_from_tref`] additionally uses [`crate::stm::TRef`] (Stratum 12)
//! as an optional extension point for concurrent node updates.

use crate::runtime::run_blocking;
use crate::stm::{Outcome, Stm, TRef, commit};
use std::collections::{BTreeMap, BTreeSet};

/// One node in a dependency layer graph: unique id, required services, and provided services.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerNode {
  /// Stable identifier for this layer (must be unique in the graph).
  pub id: String,
  /// Service names this layer depends on (must be provided by some other node).
  pub requires: Vec<String>,
  /// Service names this layer supplies to dependents.
  pub provides: Vec<String>,
}

impl LayerNode {
  /// Builds a node with the given id, requirement keys, and provided service keys.
  pub fn new(
    id: impl Into<String>,
    requires: impl IntoIterator<Item = impl Into<String>>,
    provides: impl IntoIterator<Item = impl Into<String>>,
  ) -> Self {
    Self {
      id: id.into(),
      requires: requires.into_iter().map(Into::into).collect(),
      provides: provides.into_iter().map(Into::into).collect(),
    }
  }
}

/// Directed graph of [`LayerNode`] values used to plan a valid build order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerGraph {
  /// All nodes participating in planning.
  pub nodes: Vec<LayerNode>,
}

impl LayerGraph {
  /// Graph containing the given nodes (order is preserved only in the stored `nodes` slice).
  #[inline]
  pub fn new(nodes: impl IntoIterator<Item = LayerNode>) -> Self {
    Self {
      nodes: nodes.into_iter().collect(),
    }
  }

  /// Computes a topological order over nodes from `requires` / `provides` edges, or the first planner error.
  pub fn plan_topological(&self) -> Result<LayerPlan, LayerPlannerError> {
    let mut ids = BTreeSet::new();
    for node in &self.nodes {
      if !ids.insert(node.id.clone()) {
        return Err(LayerPlannerError::DuplicateNodeId {
          id: node.id.clone(),
        });
      }
    }

    let mut provider_by_service = BTreeMap::<String, String>::new();
    for node in &self.nodes {
      for service in &node.provides {
        if let Some(existing) = provider_by_service.get(service) {
          return Err(LayerPlannerError::ConflictingProvider {
            service: service.clone(),
            first: existing.clone(),
            second: node.id.clone(),
          });
        }
        provider_by_service.insert(service.clone(), node.id.clone());
      }
    }

    let mut missing = Vec::new();
    for node in &self.nodes {
      for required in &node.requires {
        if !provider_by_service.contains_key(required) {
          missing.push(LayerMissingProvider::new(node.id.clone(), required.clone()));
        }
      }
    }
    if !missing.is_empty() {
      missing.sort();
      missing.dedup();
      return Err(LayerPlannerError::MissingProviders { missing });
    }

    let mut indegree = BTreeMap::<String, usize>::new();
    let mut edges = BTreeMap::<String, Vec<String>>::new();
    for node in &self.nodes {
      indegree.insert(node.id.clone(), 0);
      edges.insert(node.id.clone(), Vec::new());
    }

    for node in &self.nodes {
      for required in &node.requires {
        let Some(provider) = provider_by_service.get(required) else {
          continue;
        };
        let provider = provider.clone();
        if provider == node.id {
          continue;
        }
        if let Some(dependents) = edges.get_mut(&provider) {
          dependents.push(node.id.clone());
        }
        if let Some(degree) = indegree.get_mut(&node.id) {
          *degree += 1;
        }
      }
    }

    let mut queue = BTreeSet::<String>::new();
    for (id, deg) in &indegree {
      if *deg == 0 {
        queue.insert(id.clone());
      }
    }
    let mut order = Vec::new();
    while let Some(next) = queue.pop_first() {
      order.push(next.clone());
      let dependents = edges.get(&next).cloned().unwrap_or_default();
      for dependent in dependents {
        if let Some(degree) = indegree.get_mut(&dependent) {
          *degree = degree.saturating_sub(1);
        }
        if indegree.get(&dependent) == Some(&0) {
          queue.insert(dependent);
        }
      }
    }

    if order.len() != self.nodes.len() {
      let cycle_nodes = indegree
        .iter()
        .filter_map(|(id, &deg)| if deg > 0 { Some(id.clone()) } else { None })
        .collect::<Vec<_>>();
      return Err(LayerPlannerError::CycleDetected { nodes: cycle_nodes });
    }

    Ok(LayerPlan { build_order: order })
  }

  /// Plan from a single STM snapshot of `nodes` (consistent under concurrent [`TRef::write_stm`]).
  pub fn plan_topological_from_tref(
    nodes_tref: &TRef<Vec<LayerNode>>,
  ) -> Result<LayerPlan, LayerPlannerError> {
    let tr = nodes_tref.clone();
    run_blocking(
      commit(Stm::from_fn(move |txn| {
        let nodes = match tr.read_stm::<LayerPlannerError>().run_on(txn) {
          Outcome::Done(n) => n,
          Outcome::Fail(e) => return Outcome::Fail(e),
          Outcome::Retry => return Outcome::Retry,
        };
        match LayerGraph::new(nodes).plan_topological() {
          Ok(p) => Outcome::Done(p),
          Err(e) => Outcome::Fail(e),
        }
      })),
      (),
    )
  }
}

/// Successful planner output: layer node ids in an order that respects dependencies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerPlan {
  /// Node ids from roots to leaves (each requirement appears before its dependents).
  pub build_order: Vec<String>,
}

/// Human-readable diagnostic for a [`LayerPlannerError`] (code, message, suggestion).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerDiagnostic {
  /// Short stable error code (e.g. for logs or UI).
  pub code: &'static str,
  /// What went wrong.
  pub message: String,
  /// Actionable hint for fixing the graph.
  pub suggestion: String,
}

/// A missing dependency edge from a layer node to a required service key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerMissingProvider {
  /// Dependent node id.
  pub node: String,
  /// Missing service key.
  pub service: String,
}

impl LayerMissingProvider {
  /// Builds a missing dependency record.
  pub fn new(node: impl Into<String>, service: impl Into<String>) -> Self {
    Self {
      node: node.into(),
      service: service.into(),
    }
  }
}

/// Failure returned by [`LayerGraph::plan_topological`] and related APIs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerPlannerError {
  /// Two nodes share the same `id`.
  DuplicateNodeId {
    /// The duplicated node id.
    id: String,
  },
  /// More than one node lists the same service in `provides`.
  ConflictingProvider {
    /// Conflicting service key.
    service: String,
    /// First provider’s node id.
    first: String,
    /// Second provider’s node id.
    second: String,
  },
  /// A node `requires` a service that no node `provides`.
  MissingProvider {
    /// Dependent node id.
    node: String,
    /// Missing service key.
    service: String,
  },
  /// One or more nodes require services that no node provides.
  MissingProviders {
    /// Missing dependency edges sorted by node id then service key.
    missing: Vec<LayerMissingProvider>,
  },
  /// The dependency graph contains a cycle (subset of involved node ids).
  CycleDetected {
    /// Nodes that still had positive indegree when planning stalled.
    nodes: Vec<String>,
  },
}

impl LayerPlannerError {
  /// Maps this error to a [`LayerDiagnostic`] for display or tooling.
  pub fn to_diagnostic(&self) -> LayerDiagnostic {
    match self {
      LayerPlannerError::DuplicateNodeId { id } => LayerDiagnostic {
        code: "duplicate-node-id",
        message: format!("Layer graph contains duplicate node id `{id}`."),
        suggestion: String::from("Ensure each layer node has a unique `id`."),
      },
      LayerPlannerError::ConflictingProvider {
        service,
        first,
        second,
      } => LayerDiagnostic {
        code: "conflicting-provider",
        message: format!(
          "Multiple providers found for service `{service}` (`{first}`, `{second}`)."
        ),
        suggestion: String::from(
          "Split service keys or compose a single canonical provider layer for this service.",
        ),
      },
      LayerPlannerError::MissingProvider { node, service } => LayerDiagnostic {
        code: "missing-provider",
        message: format!("Layer `{node}` requires service `{service}` but no provider exists."),
        suggestion: String::from(
          "Add a provider layer for the missing service or remove the dependency edge.",
        ),
      },
      LayerPlannerError::MissingProviders { missing } => {
        if let [dependency] = missing.as_slice() {
          return LayerDiagnostic {
            code: "missing-provider",
            message: format!(
              "Layer `{}` requires service `{}` but no provider exists.",
              dependency.node, dependency.service
            ),
            suggestion: String::from(
              "Add a provider layer for the missing service or remove the dependency edge.",
            ),
          };
        }
        let services = missing
          .iter()
          .map(|dependency| format!("`{}`", dependency.service))
          .collect::<BTreeSet<_>>()
          .into_iter()
          .collect::<Vec<_>>()
          .join(", ");
        LayerDiagnostic {
          code: "missing-provider",
          message: format!("Layer graph requires services {services} but no providers exist."),
          suggestion: String::from(
            "Add provider layers for the missing services or remove the dependency edges.",
          ),
        }
      }
      LayerPlannerError::CycleDetected { nodes } => LayerDiagnostic {
        code: "cycle-detected",
        message: format!(
          "Layer dependency cycle detected across nodes: {}.",
          nodes.join(" -> ")
        ),
        suggestion: String::from(
          "Break the cycle by extracting shared requirements into an upstream layer.",
        ),
      },
    }
  }
}

impl LayerGraph {
  /// Returns zero diagnostics if planning succeeds, otherwise a single diagnostic from the first error.
  pub fn diagnostics(&self) -> Vec<LayerDiagnostic> {
    match self.plan_topological() {
      Ok(_) => Vec::new(),
      Err(error) => vec![error.to_diagnostic()],
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  mod plan_topological {
    use super::*;

    #[test]
    fn plan_topological_with_acyclic_dependencies_orders_nodes_in_dependency_order() {
      let graph = LayerGraph::new([
        LayerNode::new("repo", ["Db", "Cache"], ["Repo"]),
        LayerNode::new("api", ["Repo"], ["Api"]),
        LayerNode::new("db", Vec::<&str>::new(), ["Db"]),
        LayerNode::new("cache", Vec::<&str>::new(), ["Cache"]),
      ]);

      let plan = graph.plan_topological().expect("plan should succeed");
      assert_eq!(plan.build_order, ["cache", "db", "repo", "api"]);
    }

    #[test]
    fn layer_graph_topo_sort_with_mutable_list_matches_original() {
      let graph = LayerGraph::new([
        LayerNode::new("db", Vec::<&str>::new(), ["Db"]),
        LayerNode::new("cache", Vec::<&str>::new(), ["Cache"]),
        LayerNode::new("repo", ["Db", "Cache"], ["Repo"]),
        LayerNode::new("api", ["Repo"], ["Api"]),
      ]);

      let plan = graph.plan_topological().expect("plan should succeed");

      let pos = |id: &str, order: &[String]| {
        order
          .iter()
          .position(|node| node == id)
          .expect("node must exist")
      };
      let o = plan.build_order.as_slice();
      assert!(pos("db", o) < pos("repo", o));
      assert!(pos("cache", o) < pos("repo", o));
      assert!(pos("repo", o) < pos("api", o));
    }

    #[test]
    fn plan_topological_with_no_nodes_returns_empty_build_order() {
      let graph = LayerGraph::new(Vec::<LayerNode>::new());
      let plan = graph
        .plan_topological()
        .expect("empty graph should succeed");
      assert!(plan.build_order.is_empty());
    }

    #[test]
    fn plan_topological_with_duplicate_node_ids_returns_duplicate_node_id_error() {
      let graph = LayerGraph::new([
        LayerNode::new("db", Vec::<&str>::new(), ["Db"]),
        LayerNode::new("db", Vec::<&str>::new(), ["DbShadow"]),
      ]);

      let err = graph
        .plan_topological()
        .expect_err("duplicate id should fail");
      assert_eq!(
        err,
        LayerPlannerError::DuplicateNodeId {
          id: String::from("db"),
        }
      );
    }

    #[test]
    fn plan_topological_with_conflicting_service_providers_returns_conflicting_provider_error() {
      let graph = LayerGraph::new([
        LayerNode::new("db_a", Vec::<&str>::new(), ["Db"]),
        LayerNode::new("db_b", Vec::<&str>::new(), ["Db"]),
      ]);

      let err = graph
        .plan_topological()
        .expect_err("conflicting provider should fail");
      assert_eq!(
        err,
        LayerPlannerError::ConflictingProvider {
          service: String::from("Db"),
          first: String::from("db_a"),
          second: String::from("db_b"),
        }
      );
    }

    #[test]
    fn plan_topological_with_missing_providers_returns_stable_missing_provider_error() {
      let graph = LayerGraph::new([
        LayerNode::new("repo", ["Queue", "Db"], ["Repo"]),
        LayerNode::new("api", ["Auth"], ["Api"]),
      ]);

      let err = graph
        .plan_topological()
        .expect_err("missing providers should fail");
      assert_eq!(
        err,
        LayerPlannerError::MissingProviders {
          missing: vec![
            LayerMissingProvider::new("api", "Auth"),
            LayerMissingProvider::new("repo", "Db"),
            LayerMissingProvider::new("repo", "Queue"),
          ],
        }
      );
    }

    #[test]
    fn plan_topological_with_dependency_cycle_returns_cycle_detected_error() {
      let graph = LayerGraph::new([
        LayerNode::new("a", ["B"], ["A"]),
        LayerNode::new("b", ["A"], ["B"]),
      ]);

      let err = graph.plan_topological().expect_err("cycle should fail");
      assert!(matches!(err, LayerPlannerError::CycleDetected { .. }));
    }

    #[test]
    fn plan_topological_with_self_required_service_does_not_create_self_edge() {
      let graph = LayerGraph::new([LayerNode::new("db", ["Db"], ["Db"])]);
      let plan = graph
        .plan_topological()
        .expect("self-provided requirement should succeed");
      assert_eq!(plan.build_order, vec![String::from("db")]);
    }
  }

  mod diagnostics {
    use super::*;

    #[test]
    fn diagnostics_with_valid_graph_returns_empty_diagnostics() {
      let graph = LayerGraph::new([LayerNode::new("db", Vec::<&str>::new(), ["Db"])]);
      assert!(graph.diagnostics().is_empty());
    }

    #[rstest]
    #[case::duplicate(
      LayerPlannerError::DuplicateNodeId { id: String::from("db") },
      "duplicate-node-id",
      "duplicate node id",
      "unique"
    )]
    #[case::conflicting(
      LayerPlannerError::ConflictingProvider {
        service: String::from("Db"),
        first: String::from("db_a"),
        second: String::from("db_b"),
      },
      "conflicting-provider",
      "Multiple providers found",
      "canonical provider"
    )]
    #[case::missing(
      LayerPlannerError::MissingProviders {
        missing: vec![LayerMissingProvider::new("repo", "Db")],
      },
      "missing-provider",
      "requires service `Db`",
      "provider layer"
    )]
    #[case::cycle(
      LayerPlannerError::CycleDetected { nodes: vec![String::from("a"), String::from("b")] },
      "cycle-detected",
      "dependency cycle",
      "Break the cycle"
    )]
    fn to_diagnostic_with_error_variant_returns_expected_code_and_actionable_text(
      #[case] error: LayerPlannerError,
      #[case] expected_code: &'static str,
      #[case] expected_message_fragment: &str,
      #[case] expected_suggestion_fragment: &str,
    ) {
      let diagnostic = error.to_diagnostic();
      assert_eq!(diagnostic.code, expected_code);
      assert!(diagnostic.message.contains(expected_message_fragment));
      assert!(diagnostic.suggestion.contains(expected_suggestion_fragment));
    }

    #[test]
    fn diagnostics_with_missing_providers_returns_stable_service_keys() {
      let graph = LayerGraph::new([
        LayerNode::new("repo", ["Queue", "Db"], ["Repo"]),
        LayerNode::new("api", ["Auth"], ["Api"]),
      ]);
      let diagnostics = graph.diagnostics();

      assert_eq!(diagnostics.len(), 1);
      assert_eq!(diagnostics[0].code, "missing-provider");
      assert!(diagnostics[0].message.contains("`Auth`, `Db`, `Queue`"));
      assert!(diagnostics[0].suggestion.contains("provider"));
    }
  }

  mod stm_layer_plan {
    use super::*;
    use crate::runtime::run_blocking;
    use crate::stm::{TRef, commit};

    #[test]
    fn layer_graph_stm_plan_consistent_under_concurrent_read() {
      let state_a = vec![
        LayerNode::new("db", Vec::<&str>::new(), ["Db"]),
        LayerNode::new("api", ["Db"], ["Api"]),
      ];
      let state_b = vec![LayerNode::new("x", Vec::<&str>::new(), ["X"])];
      let tref: TRef<Vec<LayerNode>> =
        run_blocking(commit(TRef::make(state_a.clone())), ()).expect("tref");
      let tr_w = tref.clone();
      let writer = std::thread::spawn(move || {
        for _ in 0..64 {
          let _ = run_blocking(commit(tr_w.write_stm::<()>(state_b.clone())), ());
          let _ = run_blocking(commit(tr_w.write_stm::<()>(state_a.clone())), ());
        }
      });
      let mut readers = vec![];
      for _ in 0..4 {
        let tr = tref.clone();
        readers.push(std::thread::spawn(move || {
          for _ in 0..128 {
            let _ = LayerGraph::plan_topological_from_tref(&tr);
          }
        }));
      }
      writer.join().expect("writer");
      for r in readers {
        r.join().expect("reader");
      }
    }
  }

  mod clone_independence {
    use super::*;

    #[test]
    fn layer_graph_plan_clone_is_independent() {
      let graph = LayerGraph::new([
        LayerNode::new("db", Vec::<&str>::new(), ["Db"]),
        LayerNode::new("api", ["Db"], ["Api"]),
      ]);
      let plan = graph.plan_topological().expect("plan");
      let mut clone = plan.clone();
      clone.build_order.push(String::from("tamper"));
      assert_eq!(plan.build_order.len(), 2);
      assert_eq!(clone.build_order.len(), 3);
      assert!(!plan.build_order.contains(&String::from("tamper")));
    }
  }
}
