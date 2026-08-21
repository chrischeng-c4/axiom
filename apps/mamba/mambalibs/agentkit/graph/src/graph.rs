//! `Graph<State>` — directed graph of named [`Node`]s with a typed shared
//! state. First slice (#2038) supports linear paths only — every node has at
//! most one outgoing edge. Branching / cycles land in #2039.

// HANDWRITE-BEGIN reason: same gap as the parent lib — no async graph-engine
// generator yet (see lib.rs).

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use agent::{NovaError, NovaResult};
use thiserror::Error;
use tracing::{info_span, Instrument};

use crate::events::{GraphEvent, GraphEventBus};
use crate::node::{FnNode, Node, NodeRef};

/// Closure that decides which node to visit next given the running state.
/// Return `Some(name)` to jump to that node, `None` to terminate the walk.
/// Used by [`Graph::add_conditional_edge`] (#2039 — conditional edges +
/// cycles).
pub type EdgePicker<State> = Arc<dyn Fn(&State) -> Option<String> + Send + Sync + 'static>;

/// Directed edge in a `Graph`. Stored separately from nodes so the same node
/// can be the source or target of many edges (constrained to at most one
/// outgoing edge per node in this slice).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

/// Graph wiring + execution errors. Surfaced via `NovaError::ConfigError`
/// when leaving the graph crate.
#[derive(Debug, Error)]
pub enum GraphError {
    #[error("graph: duplicate node `{0}`")]
    DuplicateNode(String),
    #[error("graph: edge references unknown node `{0}`")]
    UnknownNode(String),
    #[error("graph: node `{0}` already has an outgoing edge — each node may have at most one (regular or conditional)")]
    MultipleOutgoing(String),
    #[error("graph: conditional edge from `{from}` picked unknown target `{target}`")]
    ConditionalPickerUnknown { from: String, target: String },
    #[error("graph: walk exceeded max_steps={0} — likely an unintended infinite cycle")]
    MaxStepsExceeded(u32),
    #[error("graph: no entry node set — call `set_entry` before `run`")]
    NoEntry,
}

impl From<GraphError> for NovaError {
    fn from(e: GraphError) -> Self {
        NovaError::ConfigError(e.to_string())
    }
}

/// Typed state graph. Build by adding nodes + edges, then `run(initial)` to
/// walk from the entry node to the terminal node (the unique node with no
/// outgoing edge).
/// Out-edge attached to a node. Either an unconditional jump to a named
/// target, or a closure that picks the next target from the running state
/// (`None` = terminate).
enum OutEdge<State> {
    Unconditional(String),
    Conditional(EdgePicker<State>),
}

impl<State> Clone for OutEdge<State> {
    fn clone(&self) -> Self {
        match self {
            Self::Unconditional(target) => Self::Unconditional(target.clone()),
            Self::Conditional(picker) => Self::Conditional(Arc::clone(picker)),
        }
    }
}

pub struct Graph<State>
where
    State: Send + 'static,
{
    nodes: HashMap<String, NodeRef<State>>,
    edges: Vec<Edge>,
    out: HashMap<String, OutEdge<State>>,
    entry: Option<String>,
    max_steps: u32,
    events: Option<Arc<GraphEventBus>>,
}

impl<State> Clone for Graph<State>
where
    State: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            out: self.out.clone(),
            entry: self.entry.clone(),
            max_steps: self.max_steps,
            events: self.events.clone(),
        }
    }
}

impl<State> Default for Graph<State>
where
    State: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Graph<State>
where
    State: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            out: HashMap::new(),
            entry: None,
            max_steps: 1024,
            events: None,
        }
    }

    /// Attach a typed event bus (#2058). Every state change during a
    /// `Graph::run` walk is published to the bus; subscribers receive
    /// [`GraphEvent`]s through their own queues. No-op when no bus is
    /// attached.
    pub fn events(&mut self, bus: Arc<GraphEventBus>) -> &mut Self {
        self.events = Some(bus);
        self
    }

    fn emit(&self, event: GraphEvent) {
        if let Some(bus) = &self.events {
            let _ = bus.emit(event);
        }
    }

    /// Cap on the number of node invocations during a single `run` walk.
    /// Default 1024 — protects against unintended infinite cycles in
    /// conditional-edge graphs (#2039). Set higher for legitimate long
    /// walks.
    pub fn max_steps(&mut self, max_steps: u32) -> &mut Self {
        self.max_steps = max_steps;
        self
    }

    /// Register a node. Duplicate names are rejected.
    pub fn add_node<N>(&mut self, node: N) -> Result<(), GraphError>
    where
        N: Node<State> + 'static,
    {
        let name = node.name().to_string();
        if self.nodes.contains_key(&name) {
            return Err(GraphError::DuplicateNode(name));
        }
        self.nodes.insert(name, std::sync::Arc::new(node));
        Ok(())
    }

    /// Convenience: register a closure as a [`FnNode`].
    pub fn add_fn<F, Fut>(&mut self, name: impl Into<String>, handler: F) -> Result<(), GraphError>
    where
        F: Fn(State) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = NovaResult<State>> + Send + 'static,
    {
        self.add_node(FnNode::new(name, handler))
    }

    /// Connect two nodes. Both endpoints must already have been added.
    /// A node may have at most one outgoing edge (regular or conditional).
    pub fn add_edge(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<(), GraphError> {
        let from = from.into();
        let to = to.into();
        if !self.nodes.contains_key(&from) {
            return Err(GraphError::UnknownNode(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(GraphError::UnknownNode(to));
        }
        if self.out.contains_key(&from) {
            return Err(GraphError::MultipleOutgoing(from));
        }
        self.out
            .insert(from.clone(), OutEdge::Unconditional(to.clone()));
        self.edges.push(Edge { from, to });
        Ok(())
    }

    /// Attach a state-dependent picker to `from` (#2039). On each visit of
    /// `from`, `picker(&state)` is called — return `Some(name)` to jump to
    /// that node, `None` to terminate the walk. `from` must already exist;
    /// it must not already have an outgoing edge.
    pub fn add_conditional_edge<F>(
        &mut self,
        from: impl Into<String>,
        picker: F,
    ) -> Result<(), GraphError>
    where
        F: Fn(&State) -> Option<String> + Send + Sync + 'static,
    {
        let from = from.into();
        if !self.nodes.contains_key(&from) {
            return Err(GraphError::UnknownNode(from));
        }
        if self.out.contains_key(&from) {
            return Err(GraphError::MultipleOutgoing(from));
        }
        self.out
            .insert(from, OutEdge::Conditional(Arc::new(picker)));
        Ok(())
    }

    /// Mark which node `run` should start from. Required.
    pub fn set_entry(&mut self, name: impl Into<String>) -> Result<(), GraphError> {
        let name = name.into();
        if !self.nodes.contains_key(&name) {
            return Err(GraphError::UnknownNode(name));
        }
        self.entry = Some(name);
        Ok(())
    }

    /// Walk the graph from `entry`, invoking each node in sequence with the
    /// state threaded through. Returns the final state when the walk reaches
    /// a node with no outgoing edge OR a conditional edge whose picker
    /// returns `None`. Cycles are permitted but bounded by [`Self::max_steps`].
    pub async fn run(&self, mut state: State) -> NovaResult<State> {
        let walk_span = info_span!(
            "graph.run",
            graph.entry = tracing::field::Empty,
            graph.nodes = self.nodes.len(),
            graph.max_steps = self.max_steps,
            steps = tracing::field::Empty,
        );
        let entry = self.entry.clone().ok_or(GraphError::NoEntry)?;
        walk_span.record("graph.entry", entry.as_str());
        self.emit(GraphEvent::WalkStarted {
            entry: entry.clone(),
        });

        let mut current = entry;
        let mut steps: u32 = 0;
        let result: NovaResult<State> = async {
            for _ in 0..self.max_steps {
                steps += 1;
                let node = self
                    .nodes
                    .get(&current)
                    .ok_or_else(|| GraphError::UnknownNode(current.clone()))?;
                let node_name = node.name().to_string();
                self.emit(GraphEvent::NodeStarted {
                    node: node_name.clone(),
                    step: steps,
                });
                let node_span = info_span!("graph.node", node.name = %node_name);
                state = match node.call(state).instrument(node_span).await {
                    Ok(s) => {
                        self.emit(GraphEvent::NodeCompleted {
                            node: node_name.clone(),
                            step: steps,
                        });
                        s
                    }
                    Err(e) => {
                        self.emit(GraphEvent::NodeFailed {
                            node: node_name,
                            step: steps,
                            error: e.to_string(),
                        });
                        return Err(e);
                    }
                };

                current = match self.out.get(&current) {
                    None => return Ok(state),
                    Some(OutEdge::Unconditional(to)) => to.clone(),
                    Some(OutEdge::Conditional(picker)) => match picker(&state) {
                        None => return Ok(state),
                        Some(target) => {
                            if !self.nodes.contains_key(&target) {
                                return Err(GraphError::ConditionalPickerUnknown {
                                    from: current,
                                    target,
                                }
                                .into());
                            }
                            target
                        }
                    },
                };
            }
            Err(GraphError::MaxStepsExceeded(self.max_steps).into())
        }
        .instrument(walk_span.clone())
        .await;

        walk_span.record("steps", steps);
        match &result {
            Ok(_) => self.emit(GraphEvent::WalkCompleted { steps }),
            Err(e) => self.emit(GraphEvent::WalkFailed {
                steps,
                error: e.to_string(),
            }),
        }
        result
    }

    /// Synchronous facade over [`Self::run`].
    ///
    /// The graph is cloned and executed on a dedicated OS thread with a
    /// current-thread Tokio runtime. That keeps sync callers ergonomic while
    /// avoiding Tokio's nested-runtime panic when the caller already happens
    /// to be inside an async runtime.
    pub fn run_blocking(&self, state: State) -> NovaResult<State> {
        let graph = self.clone();
        run_graph_blocking(async move { graph.run(state).await })
    }

    /// Read-only view of the registered node names (for introspection /
    /// debugging tools). Order is unspecified.
    pub fn node_names(&self) -> impl Iterator<Item = &str> {
        self.nodes.keys().map(|s| s.as_str())
    }

    /// Read-only view of the configured edges.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
}

fn run_graph_blocking<Fut, T>(future: Fut) -> NovaResult<T>
where
    Fut: Future<Output = NovaResult<T>> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| {
                NovaError::ConfigError(format!("failed to create graph runtime: {err}"))
            })?;
        runtime.block_on(future)
    })
    .join()
    .map_err(|_| NovaError::ConfigError("graph blocking runner panicked".to_string()))?
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Counter {
        n: i32,
        trail: Vec<String>,
    }

    fn empty() -> Counter {
        Counter {
            n: 0,
            trail: Vec::new(),
        }
    }

    #[tokio::test]
    async fn linear_three_node_path_threads_state_in_order() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("inc", |mut s: Counter| async move {
            s.n += 1;
            s.trail.push("inc".into());
            Ok(s)
        })
        .unwrap();
        g.add_fn("double", |mut s: Counter| async move {
            s.n *= 2;
            s.trail.push("double".into());
            Ok(s)
        })
        .unwrap();
        g.add_fn("inc2", |mut s: Counter| async move {
            s.n += 1;
            s.trail.push("inc2".into());
            Ok(s)
        })
        .unwrap();
        g.add_edge("inc", "double").unwrap();
        g.add_edge("double", "inc2").unwrap();
        g.set_entry("inc").unwrap();

        let out = g.run(empty()).await.unwrap();
        assert_eq!(out.n, 3); // (0 + 1) * 2 + 1
        assert_eq!(out.trail, vec!["inc", "double", "inc2"]);
    }

    #[test]
    fn blocking_run_threads_state_in_order() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("inc", |mut s: Counter| async move {
            s.n += 1;
            s.trail.push("inc".into());
            Ok(s)
        })
        .unwrap();
        g.add_fn("double", |mut s: Counter| async move {
            s.n *= 2;
            s.trail.push("double".into());
            Ok(s)
        })
        .unwrap();
        g.add_edge("inc", "double").unwrap();
        g.set_entry("inc").unwrap();

        let out = g.run_blocking(empty()).unwrap();
        assert_eq!(out.n, 2);
        assert_eq!(out.trail, vec!["inc", "double"]);
    }

    #[tokio::test]
    async fn single_node_with_no_edge_runs_once_and_terminates() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("only", |mut s: Counter| async move {
            s.n = 42;
            Ok(s)
        })
        .unwrap();
        g.set_entry("only").unwrap();

        let out = g.run(empty()).await.unwrap();
        assert_eq!(out.n, 42);
    }

    #[tokio::test]
    async fn missing_entry_returns_typed_error() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("a", |s| async move { Ok(s) }).unwrap();
        let err = g.run(empty()).await.unwrap_err();
        assert!(format!("{err}").contains("no entry node"));
    }

    #[tokio::test]
    async fn add_edge_rejects_second_outgoing_from_same_node() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("a", |s| async move { Ok(s) }).unwrap();
        g.add_fn("b", |s| async move { Ok(s) }).unwrap();
        g.add_fn("c", |s| async move { Ok(s) }).unwrap();
        g.add_edge("a", "b").unwrap();
        let err = g.add_edge("a", "c").unwrap_err();
        assert!(matches!(err, GraphError::MultipleOutgoing(_)));
    }

    #[tokio::test]
    async fn unbounded_cycle_is_capped_by_max_steps() {
        // Two-node loop using unconditional edges. With max_steps=8, run
        // walks 8 nodes then surfaces MaxStepsExceeded — no infinite loop.
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("a", |mut s: Counter| async move {
            s.n += 1;
            Ok(s)
        })
        .unwrap();
        g.add_fn("b", |mut s: Counter| async move {
            s.n += 1;
            Ok(s)
        })
        .unwrap();
        g.add_edge("a", "b").unwrap();
        g.add_edge("b", "a").unwrap();
        g.set_entry("a").unwrap();
        g.max_steps(8);

        let err = g.run(empty()).await.unwrap_err();
        assert!(format!("{err}").contains("max_steps"));
    }

    #[tokio::test]
    async fn conditional_edge_branches_on_state() {
        // graph: start --cond--> { hot if n >= 5 else cold }
        // Each leaf tags `n` and returns; no further edges → terminate.
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("start", |mut s: Counter| async move {
            s.trail.push("start".into());
            Ok(s)
        })
        .unwrap();
        g.add_fn("hot", |mut s: Counter| async move {
            s.trail.push("hot".into());
            Ok(s)
        })
        .unwrap();
        g.add_fn("cold", |mut s: Counter| async move {
            s.trail.push("cold".into());
            Ok(s)
        })
        .unwrap();
        g.add_conditional_edge("start", |s: &Counter| {
            Some(if s.n >= 5 {
                "hot".to_string()
            } else {
                "cold".to_string()
            })
        })
        .unwrap();
        g.set_entry("start").unwrap();

        let cold = g
            .run(Counter {
                n: 0,
                trail: vec![],
            })
            .await
            .unwrap();
        assert_eq!(cold.trail, vec!["start", "cold"]);

        let hot = g
            .run(Counter {
                n: 10,
                trail: vec![],
            })
            .await
            .unwrap();
        assert_eq!(hot.trail, vec!["start", "hot"]);
    }

    #[tokio::test]
    async fn conditional_loop_until_picker_returns_none() {
        // Increment n in a tight loop, with the picker re-entering "inc"
        // until n >= 3, then returning None to terminate.
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("inc", |mut s: Counter| async move {
            s.n += 1;
            s.trail.push("inc".into());
            Ok(s)
        })
        .unwrap();
        g.add_conditional_edge("inc", |s: &Counter| {
            if s.n < 3 {
                Some("inc".to_string())
            } else {
                None
            }
        })
        .unwrap();
        g.set_entry("inc").unwrap();

        let out = g.run(empty()).await.unwrap();
        assert_eq!(out.n, 3);
        assert_eq!(out.trail.len(), 3);
    }

    #[tokio::test]
    async fn conditional_picker_unknown_target_surfaces_typed_error() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("start", |s| async move { Ok(s) }).unwrap();
        g.add_conditional_edge("start", |_| Some("nowhere".to_string()))
            .unwrap();
        g.set_entry("start").unwrap();

        let err = g.run(empty()).await.unwrap_err();
        assert!(
            format!("{err}").contains("ConditionalPickerUnknown")
                || format!("{err}").contains("picked unknown")
        );
    }

    #[tokio::test]
    async fn node_returning_err_aborts_walk() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("a", |s| async move { Ok(s) }).unwrap();
        g.add_fn("b", |_s| async move {
            Err(agent::NovaError::ToolError("boom".into()))
        })
        .unwrap();
        g.add_fn("c", |s| async move { Ok(s) }).unwrap();
        g.add_edge("a", "b").unwrap();
        g.add_edge("b", "c").unwrap();
        g.set_entry("a").unwrap();

        let err = g.run(empty()).await.unwrap_err();
        assert!(matches!(err, agent::NovaError::ToolError(_)));
    }

    #[test]
    fn duplicate_node_rejected() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("a", |s| async move { Ok(s) }).unwrap();
        let err = g.add_fn("a", |s| async move { Ok(s) }).unwrap_err();
        assert!(matches!(err, GraphError::DuplicateNode(_)));
    }

    #[test]
    fn edge_to_unknown_node_rejected() {
        let mut g: Graph<Counter> = Graph::new();
        g.add_fn("a", |s| async move { Ok(s) }).unwrap();
        let err = g.add_edge("a", "ghost").unwrap_err();
        assert!(matches!(err, GraphError::UnknownNode(_)));
    }
}
