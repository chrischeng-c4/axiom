//! Orchestration core (#106, #116): drive a [`WorkflowRun`]'s DAG by
//! dispatching ready nodes and folding in completions.
//!
//! Backend-agnostic by design: the scheduler talks to a [`Dispatcher`]
//! (loom → relay publish) and is fed [`Completion`] events (from relay acks).
//! The real relay/keep HTTP/2 clients implement `Dispatcher` and feed
//! completions; tests use in-memory fakes. This keeps loom's core logic
//! testable with no broker, store, or cluster.

use serde::{Deserialize, Serialize};

use crate::model::{KeepRef, NodeId, RunStatus, WorkflowRun};
use crate::runner::RunnerClass;

/// What loom publishes to relay to dispatch one node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskMessage {
    pub run_id: String,
    pub node_id: String,
    pub attempt: u32,
    pub task_name: String,
    pub args: serde_json::Value,
    /// Claim-check inputs the runner fetches from keep.
    pub input_refs: Vec<KeepRef>,
    pub runner: RunnerClass,
}

/// loom → relay publish. The production impl is an HTTP/2 relay client; tests
/// use a recording fake.
pub trait Dispatcher {
    fn dispatch(&mut self, route: &str, msg: TaskMessage) -> anyhow::Result<()>;
}

/// A worker completion observed via a relay ack.
#[derive(Debug, Clone)]
pub enum Completion {
    Ok { node: NodeId, result: Option<KeepRef> },
    Failed { node: NodeId },
}

/// Drives one run's DAG forward over a [`Dispatcher`].
pub struct Scheduler<D: Dispatcher> {
    pub run: WorkflowRun,
    dispatcher: D,
}

impl<D: Dispatcher> Scheduler<D> {
    pub fn new(run: WorkflowRun, dispatcher: D) -> Self {
        Self { run, dispatcher }
    }

    /// Publish every currently-ready node to relay (routed by its runner class)
    /// and mark it dispatched. Returns the number dispatched.
    pub fn tick(&mut self) -> anyhow::Result<usize> {
        let ready = self.run.ready_nodes();
        for id in &ready {
            let node = &self.run.nodes[id];
            let msg = TaskMessage {
                run_id: self.run.id.0.clone(),
                node_id: node.id.0.clone(),
                attempt: node.attempt + 1,
                task_name: node.task.task_name.clone(),
                args: node.task.args.clone(),
                input_refs: node.task.input_refs.clone(),
                runner: node.task.runner,
            };
            self.dispatcher.dispatch(node.task.runner.relay_route(), msg)?;
            self.run.mark_dispatched(id);
        }
        Ok(ready.len())
    }

    /// Fold a completion into DAG state, then dispatch any newly-ready nodes.
    /// Returns the number dispatched as a result.
    pub fn on_completion(&mut self, completion: Completion) -> anyhow::Result<usize> {
        match completion {
            Completion::Ok { node, result } => self.run.mark_done(&node, result),
            Completion::Failed { node } => self.run.mark_failed(&node),
        }
        self.tick()
    }

    pub fn status(&self) -> RunStatus {
        self.run.status
    }

    pub fn is_complete(&self) -> bool {
        self.run.is_complete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Node, NodeId, StageId, TaskSpec, WorkflowRunId};
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct RecordingDispatcher {
        sent: Vec<(String, TaskMessage)>,
    }
    impl Dispatcher for RecordingDispatcher {
        fn dispatch(&mut self, route: &str, msg: TaskMessage) -> anyhow::Result<()> {
            self.sent.push((route.to_string(), msg));
            Ok(())
        }
    }

    fn node(id: &str, stage: &str, deps: &[&str]) -> Node {
        Node::new(
            NodeId::new(id),
            StageId::new(stage),
            TaskSpec::new(format!("task-{id}")),
            deps.iter().map(|d| NodeId::new(*d)).collect::<BTreeSet<_>>(),
        )
    }

    fn diamond() -> WorkflowRun {
        let mut run = WorkflowRun::new(WorkflowRunId::new("run-1"));
        run.add_node(node("A", "s0", &[]));
        run.add_node(node("B", "s1", &["A"]));
        run.add_node(node("C", "s1", &["A"]));
        run.add_node(node("D", "s2", &["B", "C"]));
        run
    }

    #[test]
    fn drives_diamond_dag_to_completion() {
        let mut sched = Scheduler::new(diamond(), RecordingDispatcher::default());

        // tick 1: only the root is ready.
        assert_eq!(sched.tick().unwrap(), 1);

        // completing the root fans out to B and C.
        assert_eq!(
            sched.on_completion(Completion::Ok { node: NodeId::new("A"), result: Some(KeepRef("k/A".into())) }).unwrap(),
            2
        );

        // fan-in: D only dispatches once BOTH B and C complete.
        assert_eq!(sched.on_completion(Completion::Ok { node: NodeId::new("B"), result: None }).unwrap(), 0);
        assert_eq!(sched.on_completion(Completion::Ok { node: NodeId::new("C"), result: None }).unwrap(), 1);

        // completing D finishes the run.
        assert_eq!(sched.on_completion(Completion::Ok { node: NodeId::new("D"), result: None }).unwrap(), 0);
        assert_eq!(sched.status(), RunStatus::Succeeded);
        assert!(sched.is_complete());
    }

    #[test]
    fn dispatch_carries_route_and_identity() {
        let mut sched = Scheduler::new(diamond(), RecordingDispatcher::default());
        sched.tick().unwrap();
        let (route, msg) = &sched.dispatcher.sent[0];
        assert_eq!(route, "resident"); // default runner class
        assert_eq!(msg.run_id, "run-1");
        assert_eq!(msg.node_id, "A");
        assert_eq!(msg.attempt, 1);
        assert_eq!(msg.task_name, "task-A");
    }

    #[test]
    fn failed_node_is_redispatched_within_attempts() {
        let mut sched = Scheduler::new(diamond(), RecordingDispatcher::default());
        sched.tick().unwrap(); // dispatch A (attempt 1)
        // A fails but has attempts left -> on_completion re-readies and re-dispatches it.
        assert_eq!(sched.on_completion(Completion::Failed { node: NodeId::new("A") }).unwrap(), 1);
        assert_eq!(sched.status(), RunStatus::Running);
        // the redispatch is attempt 2.
        assert_eq!(sched.dispatcher.sent.last().unwrap().1.attempt, 2);
    }
}
