//! `layer_node!` and `layer_graph!` macros for compact layer-planner DSL.

/// Build a single [`LayerNode`](crate::LayerNode).
///
/// ```ignore
/// let node = effect_rs::layer_node!(
///   "repo",
///   requires = ["Db", "Cache"],
///   provides = ["Repo"]
/// );
/// ```
#[macro_export]
macro_rules! layer_node {
  ($id:expr, requires = [$($req:expr),* $(,)?], provides = [$($prov:expr),* $(,)?]) => {
    ::effect_rs::LayerNode::new($id, [$($req),*], [$($prov),*])
  };
}

/// Build a [`LayerGraph`](crate::LayerGraph) from a compact declaration block.
///
/// ```ignore
/// let graph = effect_rs::layer_graph! {
///   db    => [Db];
///   cache => [Cache];
///   repo  : [Db, Cache] => [Repo];
///   api   : [Repo] => [Api];
/// };
/// ```
#[macro_export]
macro_rules! layer_graph {
  (
    $(
      $id:ident $( : [$($req:ident),* $(,)?] )? => [$($prov:ident),* $(,)?]
    );+ $(;)?
  ) => {
    ::effect_rs::LayerGraph::new([
      $(
        ::effect_rs::LayerNode::new(
          stringify!($id),
          $crate::layer_graph!(@reqs $( [$($req),*] )?),
          $crate::layer_graph!(@provs [$( $prov ),*]),
        )
      ),+
    ])
  };

  (@reqs) => {
    ::std::vec::Vec::<&'static str>::new()
  };

  (@reqs [$($req:ident),*]) => {
    ::std::vec![$( stringify!($req) ),*]
  };

  (@provs [$($prov:ident),*]) => {
    ::std::vec![$( stringify!($prov) ),*]
  };
}
