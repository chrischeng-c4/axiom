//! Workflow data model (#112) — the sharded, strongly-consistent DAG state
//! loom owns.
//!
//! Adapted from `projects/queue/queue/src/workflow` (Celery-style canvas) for
//! loom's server-side model: payloads are **claim-check refs into keep**, never
//! inline bytes, and each task carries its [`RunnerClass`] (#164). loom keeps
//! this state; relay carries dispatch, keep carries payloads.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::runner::RunnerClass;

macro_rules! string_id {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
        }
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

/// Claim-check reference to a payload stored in keep. loom passes refs, never
/// bytes — input/result data moves client↔keep and worker↔keep directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeepRef(pub String);

/// The task a node runs. Small args inline; large inputs by keep ref.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Logical task name a worker dispatches on.
    pub task_name: String,
    /// Small inline arguments (JSON). Large inputs go in `input_refs`.
    #[serde(default)]
    pub args: serde_json::Value,
    /// Claim-check inputs fetched from keep by the runner.
    #[serde(default)]
    pub input_refs: Vec<KeepRef>,
    /// Execution environment for this task (#164).
    #[serde(default)]
    pub runner: RunnerClass,
    /// Maximum attempts before the node fails terminally.
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
}

fn default_max_attempts() -> u32 {
    3
}

impl TaskSpec {
    pub fn new(task_name: impl Into<String>) -> Self {
        Self {
            task_name: task_name.into(),
            args: serde_json::Value::Null,
            input_refs: Vec::new(),
            runner: RunnerClass::default(),
            max_attempts: default_max_attempts(),
        }
    }
}

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

/// One task instance in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub stage: StageId,
    pub task: TaskSpec,
    /// Upstream nodes that must reach [`NodeState::Done`] before this is `Ready`.
    pub deps: BTreeSet<NodeId>,
    pub state: NodeState,
    pub attempt: u32,
    /// keep ref to the result payload, set when the node completes (claim-check).
    pub result_ref: Option<KeepRef>,
    /// Inline result bytes for small payloads (skips keep). Worker-local, never
    /// serialized; lets a downstream node read this result without a keep hop.
    #[serde(default, skip)]
    pub result_inline: Option<Vec<u8>>,
}

impl Node {
    pub fn new(id: NodeId, stage: StageId, task: TaskSpec, deps: BTreeSet<NodeId>) -> Self {
        let state = if deps.is_empty() {
            NodeState::Ready
        } else {
            NodeState::Pending
        };
        Self { id, stage, task, deps, state, attempt: 0, result_ref: None, result_inline: None }
    }
}

/// A fan-out group of sibling nodes; the fan-in barrier is satisfied when every
/// member reaches [`NodeState::Done`] (counted by node id, not attempt).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub id: StageId,
    pub members: BTreeSet<NodeId>,
}

/// Terminal-or-running status of a whole run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

/// The persistent DAG state loom owns, sharded by [`WorkflowRunId`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: WorkflowRunId,
    pub nodes: BTreeMap<NodeId, Node>,
    pub stages: BTreeMap<StageId, Stage>,
    pub status: RunStatus,
}

impl WorkflowRun {
    pub fn new(id: WorkflowRunId) -> Self {
        Self { id, nodes: BTreeMap::new(), stages: BTreeMap::new(), status: RunStatus::Pending }
    }

    /// Insert a node and register it in its stage.
    pub fn add_node(&mut self, node: Node) {
        self.stages
            .entry(node.stage.clone())
            .or_insert_with(|| Stage { id: node.stage.clone(), members: BTreeSet::new() })
            .members
            .insert(node.id.clone());
        self.nodes.insert(node.id.clone(), node);
    }

    /// Nodes eligible for dispatch right now: `Ready` state.
    pub fn ready_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .values()
            .filter(|n| n.state == NodeState::Ready)
            .map(|n| n.id.clone())
            .collect()
    }

    pub fn mark_dispatched(&mut self, id: &NodeId) {
        if let Some(n) = self.nodes.get_mut(id) {
            n.state = NodeState::Dispatched;
            n.attempt += 1;
        }
        self.status = RunStatus::Running;
    }

    pub fn mark_running(&mut self, id: &NodeId) {
        if let Some(n) = self.nodes.get_mut(id) {
            n.state = NodeState::Running;
        }
    }

    /// Record success, store the result ref, and promote any downstream node
    /// whose dependencies are now all `Done` to `Ready`.
    /// Whether a completion for `id` at `attempt` should be applied now: the node
    /// must be currently in flight (Dispatched/Running) for exactly this attempt.
    /// Rejects duplicates (already terminal) and stale attempts (a redelivered
    /// completion from a prior attempt) — the at-least-once idempotency guard
    /// (#437), so a replayed completion never double-splices fan-out or re-runs.
    pub fn completion_is_current(&self, id: &NodeId, attempt: u32) -> bool {
        self.nodes.get(id).is_some_and(|n| {
            matches!(n.state, NodeState::Dispatched | NodeState::Running) && n.attempt == attempt
        })
    }

    pub fn mark_done(&mut self, id: &NodeId, result_ref: Option<KeepRef>) {
        self.mark_done_inline(id, result_ref, None);
    }

    /// Like [`mark_done`](Self::mark_done) but also records an inline result so a
    /// downstream node can read it without a keep round-trip (#127 inline).
    pub fn mark_done_inline(
        &mut self,
        id: &NodeId,
        result_ref: Option<KeepRef>,
        result_inline: Option<Vec<u8>>,
    ) {
        if let Some(n) = self.nodes.get_mut(id) {
            n.state = NodeState::Done;
            n.result_ref = result_ref;
            n.result_inline = result_inline;
        }
        self.recompute_ready();
        self.recompute_status();
    }

    /// A node failed an attempt. Retry while attempts remain; otherwise fail
    /// terminally (which fails the run).
    pub fn mark_failed(&mut self, id: &NodeId) {
        if let Some(n) = self.nodes.get_mut(id) {
            if n.attempt < n.task.max_attempts {
                // eligible for redelivery / a fresh attempt
                n.state = NodeState::Ready;
            } else {
                n.state = NodeState::Failed;
            }
        }
        self.recompute_status();
    }

    /// Fan-in: every member of `stage` has reached `Done`.
    pub fn stage_complete(&self, stage: &StageId) -> bool {
        match self.stages.get(stage) {
            Some(s) => s.members.iter().all(|id| {
                self.nodes.get(id).map(|n| n.state == NodeState::Done).unwrap_or(false)
            }),
            None => false,
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, RunStatus::Succeeded | RunStatus::Failed)
    }

    /// Dynamic fan-out (#116, runtime stage-expand): after `parent` has
    /// completed, splice in `children` (each gated on `parent`) and make every
    /// node that depended on `parent` also wait for the new children — the
    /// fan-in barrier grows at runtime. This turns a static `split → merge`
    /// into `split → {c1..cN} → merge` once the split task knows N (e.g. a CSV
    /// reader that discovers the chunk count). No-op if `parent` is unknown.
    pub fn expand(&mut self, parent: &NodeId, mut children: Vec<Node>) {
        if !self.nodes.contains_key(parent) || children.is_empty() {
            return;
        }
        let child_ids: Vec<NodeId> = children.iter().map(|c| c.id.clone()).collect();
        for c in &mut children {
            c.deps.insert(parent.clone());
            c.state = NodeState::Pending;
        }
        // Downstream dependents of `parent` now also gate on the children.
        for node in self.nodes.values_mut() {
            if node.deps.contains(parent) {
                for cid in &child_ids {
                    node.deps.insert(cid.clone());
                }
                if node.state == NodeState::Ready {
                    node.state = NodeState::Pending;
                }
            }
        }
        for child in children {
            self.add_node(child);
        }
        self.recompute_ready();
        // Adding not-yet-done children un-finishes a run that mark_done had just
        // marked Succeeded.
        if self.nodes.values().any(|n| n.state == NodeState::Failed) {
            self.status = RunStatus::Failed;
        } else if !self.nodes.values().all(|n| n.state == NodeState::Done) {
            self.status = RunStatus::Running;
        }
    }

    /// Promote `Pending` nodes whose deps are all `Done` to `Ready`.
    fn recompute_ready(&mut self) {
        let done: BTreeSet<NodeId> = self
            .nodes
            .values()
            .filter(|n| n.state == NodeState::Done)
            .map(|n| n.id.clone())
            .collect();
        for node in self.nodes.values_mut() {
            if node.state == NodeState::Pending && node.deps.iter().all(|d| done.contains(d)) {
                node.state = NodeState::Ready;
            }
        }
    }

    fn recompute_status(&mut self) {
        if self.nodes.values().any(|n| n.state == NodeState::Failed) {
            self.status = RunStatus::Failed;
        } else if self.nodes.values().all(|n| n.state == NodeState::Done) && !self.nodes.is_empty() {
            self.status = RunStatus::Succeeded;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, stage: &str, deps: &[&str]) -> Node {
        Node::new(
            NodeId::new(id),
            StageId::new(stage),
            TaskSpec::new(format!("task-{id}")),
            deps.iter().map(|d| NodeId::new(*d)).collect(),
        )
    }

    /// DAG: A → B, A → C, {B,C} → D (a fan-out then fan-in).
    fn diamond() -> WorkflowRun {
        let mut run = WorkflowRun::new(WorkflowRunId::new("run-1"));
        run.add_node(node("A", "s0", &[]));
        run.add_node(node("B", "s1", &["A"]));
        run.add_node(node("C", "s1", &["A"]));
        run.add_node(node("D", "s2", &["B", "C"]));
        run
    }

    #[test]
    fn only_root_is_ready_initially() {
        let run = diamond();
        assert_eq!(run.ready_nodes(), vec![NodeId::new("A")]);
        assert_eq!(run.status, RunStatus::Pending);
    }

    #[test]
    fn completing_root_readies_the_fan_out() {
        let mut run = diamond();
        run.mark_dispatched(&NodeId::new("A"));
        assert_eq!(run.status, RunStatus::Running);
        run.mark_done(&NodeId::new("A"), Some(KeepRef("k/A".into())));
        let mut ready = run.ready_nodes();
        ready.sort();
        assert_eq!(ready, vec![NodeId::new("B"), NodeId::new("C")]);
    }

    #[test]
    fn fan_in_waits_for_all_siblings() {
        let mut run = diamond();
        run.mark_done(&NodeId::new("A"), None);
        run.mark_done(&NodeId::new("B"), None);
        // C not done yet: D must stay Pending and the stage incomplete.
        assert!(!run.stage_complete(&StageId::new("s1")));
        assert_eq!(run.ready_nodes(), vec![NodeId::new("C")]);
        run.mark_done(&NodeId::new("C"), None);
        assert!(run.stage_complete(&StageId::new("s1")));
        assert_eq!(run.ready_nodes(), vec![NodeId::new("D")]);
    }

    #[test]
    fn run_succeeds_when_all_nodes_done() {
        let mut run = diamond();
        for id in ["A", "B", "C", "D"] {
            run.mark_done(&NodeId::new(id), None);
        }
        assert_eq!(run.status, RunStatus::Succeeded);
        assert!(run.is_complete());
    }

    #[test]
    fn failed_node_retries_then_fails_the_run() {
        let mut run = diamond();
        let a = NodeId::new("A");
        // exhaust attempts: max_attempts defaults to 3.
        for _ in 0..3 {
            run.mark_dispatched(&a);
            run.mark_failed(&a);
        }
        assert_eq!(run.nodes[&a].state, NodeState::Failed);
        assert_eq!(run.status, RunStatus::Failed);
    }

    #[test]
    fn dynamic_fan_out_expands_at_runtime() {
        // static split → merge; split discovers 3 chunks at runtime.
        let mut run = WorkflowRun::new(WorkflowRunId::new("dyn"));
        run.add_node(node("split", "s0", &[]));
        run.add_node(node("merge", "s2", &["split"]));

        run.mark_done(&NodeId::new("split"), None);
        // before expand, merge would be ready; we expand into 3 children first.
        run.expand(
            &NodeId::new("split"),
            vec![
                node("c0", "s1", &[]),
                node("c1", "s1", &[]),
                node("c2", "s1", &[]),
            ],
        );

        // run is back to Running; the 3 children are ready, merge waits for them.
        assert_eq!(run.status, RunStatus::Running);
        let mut ready = run.ready_nodes();
        ready.sort();
        assert_eq!(ready, vec![NodeId::new("c0"), NodeId::new("c1"), NodeId::new("c2")]);
        assert_eq!(run.nodes[&NodeId::new("merge")].state, NodeState::Pending);

        // completing all children releases the merge barrier.
        for c in ["c0", "c1", "c2"] {
            run.mark_done(&NodeId::new(c), None);
        }
        assert_eq!(run.ready_nodes(), vec![NodeId::new("merge")]);
        run.mark_done(&NodeId::new("merge"), None);
        assert_eq!(run.status, RunStatus::Succeeded);
    }
}
