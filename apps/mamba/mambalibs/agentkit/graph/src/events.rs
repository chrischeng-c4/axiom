//! Structured event log for the graph runtime (#2058 — graph half).
//!
//! Mirrors the agent-side `AgentEvent` shape from `agent::events`. Each
//! state change of `Graph::run` is published to an attached
//! `EventBus<GraphEvent>` so the local UI, an OTel exporter, or an
//! eval harness can observe the walk without re-reading the state.

// HANDWRITE-BEGIN reason: same gap as the rest of the graph crate —
// no rust-runtime generator for an event-stream module yet.

use agent::EventBus;
use serde::{Deserialize, Serialize};

/// Typed event emitted from `Graph::run`. Variant names mirror the
/// agent-side `AgentEvent` so a unified UI/exporter can switch on a
/// flat tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GraphEvent {
    WalkStarted {
        entry: String,
    },
    NodeStarted {
        node: String,
        step: u32,
    },
    NodeCompleted {
        node: String,
        step: u32,
    },
    NodeFailed {
        node: String,
        step: u32,
        error: String,
    },
    WalkCompleted {
        steps: u32,
    },
    WalkFailed {
        steps: u32,
        error: String,
    },
}

/// Re-exported helper so callers can `use agentkit_graph::GraphEventBus`
/// without reaching into the agent crate.
pub type GraphEventBus = EventBus<GraphEvent>;

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_event_serde_round_trip() {
        let e = GraphEvent::NodeCompleted {
            node: "writer".into(),
            step: 3,
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"event\":\"node_completed\""));
        let back: GraphEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e);
    }
}
