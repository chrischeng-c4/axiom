//! Workflow data model (#112) — foundational identifiers and node lifecycle.
//!
//! P0 scaffold: the stable ID newtypes + node-state enum every other module
//! refs. The full WorkflowRun / Node / Stage model (DAG edges, fan-in barriers,
//! claim-check input/result refs), seeded from `projects/queue/queue/src/workflow`,
//! lands in P1 via the aw td lifecycle.

use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(/// Identifies one workflow run (the shard key for loom's DAG state).
    WorkflowRunId);
string_id!(/// Identifies one node (task instance) within a run.
    NodeId);
string_id!(/// Identifies one stage (a fan-out group of sibling nodes) within a run.
    StageId);

/// Lifecycle of a single node, owned by loom's persistent DAG state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeState {
    /// Dependencies not yet satisfied.
    Pending,
    /// Dependencies satisfied; eligible for dispatch.
    Ready,
    /// Published to relay; awaiting lease/completion.
    Dispatched,
    /// A worker has leased and is running it (heartbeating).
    Running,
    /// Completed successfully; result-ref recorded in keep.
    Done,
    /// Failed terminally (retries exhausted).
    Failed,
}
