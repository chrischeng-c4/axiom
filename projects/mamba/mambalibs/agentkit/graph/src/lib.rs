//! agentkit-graph — stateful, typed graph runtime modelled on LangGraph.
//!
//! This crate is the foundation for Epic 3 (#2017). Each issue under the
//! epic layers one capability on top of the [`Graph`] core:
//!
//! | Issue | Capability |
//! |-------|------------|
//! | #2038 | `Graph<State>` + `Node` + `Edge` core (this issue — linear path) |
//! | #2039 | Conditional edges + cycles |
//! | #2040 | `Checkpoint` trait + memory/file/SQL impls |
//! | #2041 | Interrupt + human-in-the-loop resumption |
//! | #2042 | Parallel node execution (fan-out / fan-in) |
//! | #2043 | Multi-agent / subgraph composition |
//! | #2044 | Graph event streaming (state-delta + node events) |
//!
//! The runtime is deliberately tiny in this first slice: a typed `State`,
//! a directed graph of `Node`s, and a `run` loop that walks the configured
//! edges linearly. Branching and cycles are out of scope (deferred to
//! #2039); the loop refuses to run on a graph that has more than one
//! outgoing edge from any node.

// HANDWRITE-BEGIN reason: no Rust-runtime generator for an async graph
// engine yet. Once a `graph-engine` section type lands that can emit
// `Graph` / `Node` / `Edge` from the lifecycle/dependency YAML, swap to
// CODEGEN markers.

pub mod checkpoint;
pub mod events;
pub mod graph;
pub mod node;

pub use checkpoint::{Checkpoint, FileCheckpoint, MemoryCheckpoint};
pub use events::{GraphEvent, GraphEventBus};
pub use graph::{Edge, EdgePicker, Graph, GraphError};
pub use node::{FnNode, Node, NodeRef};

// HANDWRITE-END
