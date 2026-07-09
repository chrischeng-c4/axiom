//! Orchestration core (#106, #116): drive a [`WorkflowRun`]'s DAG by
//! dispatching ready nodes and folding in completions.
//!
//! Backend-agnostic by design: dispatch goes through an async [`Dispatcher`]
//! (loom → relay publish) and completions are fed in (from relay acks). The
//! real relay client ([`crate::relay_client::RelayDispatcher`]) implements
//! `Dispatcher`; tests and the in-process controller use [`MemDispatcher`].

use std::sync::Mutex;

use async_trait::async_trait;
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
    /// Inline input bytes for small payloads (#127): when set, the worker uses
    /// these directly instead of a keep fetch. Set by the dispatcher when an
    /// upstream node produced a small inline result. Omitted when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_inline: Option<Vec<u8>>,
    pub runner: RunnerClass,
}

impl TaskMessage {
    /// Idempotency key for the relay publish: stable per (run, node, attempt).
    pub fn message_id(&self) -> String {
        format!("{}:{}:{}", self.run_id, self.node_id, self.attempt)
    }
}

/// loom → relay publish. Shared (`&self`) so the control plane can hold one
/// behind an `Arc`; async because the production impl is an HTTP/2 relay client.
#[async_trait]
pub trait Dispatcher: Send + Sync {
    async fn dispatch(&self, route: &str, msg: TaskMessage) -> anyhow::Result<()>;
}

/// In-memory dispatcher: records every published message. The dev/test backend
/// and the default for `loom controller` until the relay client is wired.
#[derive(Default)]
pub struct MemDispatcher {
    sent: Mutex<Vec<(String, TaskMessage)>>,
}

impl MemDispatcher {
    pub fn new() -> Self {
        Self::default()
    }
    /// Snapshot of everything dispatched so far (route, message).
    pub fn sent(&self) -> Vec<(String, TaskMessage)> {
        self.sent.lock().unwrap().clone()
    }
}

#[async_trait]
impl Dispatcher for MemDispatcher {
    async fn dispatch(&self, route: &str, msg: TaskMessage) -> anyhow::Result<()> {
        self.sent.lock().unwrap().push((route.to_string(), msg));
        Ok(())
    }
}

/// A worker completion observed via a relay ack.
#[derive(Debug, Clone)]
pub enum Completion {
    Ok { node: NodeId, result: Option<KeepRef> },
    Failed { node: NodeId },
}

/// A runtime-discovered fan-out child (#116): a task a completing node splices
/// into the DAG (e.g. a CSV reader emitting one node per chunk).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FanOutSpec {
    pub id: String,
    pub task_name: String,
    #[serde(default)]
    pub input_refs: Vec<KeepRef>,
    /// Inline child input the worker writes to keep before reporting, then
    /// replaces with an `input_refs` entry — so chunk bytes never enter the
    /// control plane (claim-check). Worker-local; never serialized.
    #[serde(skip)]
    pub input_data: Option<Vec<u8>>,
}

/// Wire form of a worker completion: published to the loom completions subject
/// by a worker and consumed by the controller to advance the DAG.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionMsg {
    pub run_id: String,
    pub node_id: String,
    pub attempt: u32,
    #[serde(default)]
    pub result_ref: Option<String>,
    /// Inline result bytes for small payloads (#127): the worker reports the
    /// result inline instead of writing it to keep. Omitted when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_inline: Option<Vec<u8>>,
    #[serde(default)]
    pub failed: bool,
    /// Runtime fan-out children to splice in after this node (#116).
    #[serde(default)]
    pub fan_out: Vec<FanOutSpec>,
}

/// Publish every currently-ready node of `run` to `dispatcher` (routed by its
/// runner class) and mark it dispatched. Returns the number dispatched.
pub async fn dispatch_ready(
    run: &mut WorkflowRun,
    dispatcher: &dyn Dispatcher,
) -> anyhow::Result<usize> {
    let ready = run.ready_nodes();
    // #127 inline: a node's first input ref may name an upstream node's result;
    // if that result was produced inline (small), pass it as input_inline so the
    // worker skips the keep fetch entirely. Build the lookup once per sweep.
    let inline_results: std::collections::HashMap<String, Vec<u8>> = run
        .nodes
        .values()
        .filter_map(|n| {
            n.result_inline
                .as_ref()
                .map(|r| (format!("{}:{}:result", run.id.0, n.id.0), r.clone()))
        })
        .collect();
    for id in &ready {
        let node = &run.nodes[id];
        let mut input_refs = node.task.input_refs.clone();
        let mut input_inline = None;
        if let Some(first) = input_refs.first() {
            if let Some(bytes) = inline_results.get(&first.0) {
                input_inline = Some(bytes.clone());
                input_refs.clear(); // satisfied inline; no keep ref needed
            }
        }
        let msg = TaskMessage {
            run_id: run.id.0.clone(),
            node_id: node.id.0.clone(),
            attempt: node.attempt + 1,
            task_name: node.task.task_name.clone(),
            args: node.task.args.clone(),
            input_refs,
            input_inline,
            runner: node.task.runner,
        };
        dispatcher.dispatch(node.task.runner.relay_route(), msg).await?;
        run.mark_dispatched(id);
    }
    Ok(ready.len())
}

/// Fold a completion into `run`, then dispatch any newly-ready nodes.
pub async fn apply_completion(
    run: &mut WorkflowRun,
    dispatcher: &dyn Dispatcher,
    completion: Completion,
) -> anyhow::Result<usize> {
    match completion {
        Completion::Ok { node, result } => run.mark_done(&node, result),
        Completion::Failed { node } => run.mark_failed(&node),
    }
    dispatch_ready(run, dispatcher).await
}

/// Owns a run + a dispatcher and drives it forward (a standalone driver; the
/// controller uses the free functions over store-backed runs instead).
pub struct Scheduler<D: Dispatcher> {
    pub run: WorkflowRun,
    dispatcher: D,
}

impl<D: Dispatcher + 'static> Scheduler<D> {
    pub fn new(run: WorkflowRun, dispatcher: D) -> Self {
        Self { run, dispatcher }
    }
    pub async fn tick(&mut self) -> anyhow::Result<usize> {
        dispatch_ready(&mut self.run, &self.dispatcher).await
    }
    pub async fn on_completion(&mut self, completion: Completion) -> anyhow::Result<usize> {
        apply_completion(&mut self.run, &self.dispatcher, completion).await
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

    #[tokio::test]
    async fn drives_diamond_dag_to_completion() {
        let mut sched = Scheduler::new(diamond(), MemDispatcher::new());
        assert_eq!(sched.tick().await.unwrap(), 1);
        assert_eq!(
            sched.on_completion(Completion::Ok { node: NodeId::new("A"), result: Some(KeepRef("k/A".into())) }).await.unwrap(),
            2
        );
        assert_eq!(sched.on_completion(Completion::Ok { node: NodeId::new("B"), result: None }).await.unwrap(), 0);
        assert_eq!(sched.on_completion(Completion::Ok { node: NodeId::new("C"), result: None }).await.unwrap(), 1);
        assert_eq!(sched.on_completion(Completion::Ok { node: NodeId::new("D"), result: None }).await.unwrap(), 0);
        assert_eq!(sched.status(), RunStatus::Succeeded);
        assert!(sched.is_complete());
    }

    #[tokio::test]
    async fn dispatch_carries_route_and_identity() {
        let mut sched = Scheduler::new(diamond(), MemDispatcher::new());
        sched.tick().await.unwrap();
        let sent = sched.dispatcher.sent();
        let (route, msg) = &sent[0];
        assert_eq!(route, "resident");
        assert_eq!(msg.run_id, "run-1");
        assert_eq!(msg.node_id, "A");
        assert_eq!(msg.attempt, 1);
        assert_eq!(msg.message_id(), "run-1:A:1");
    }

    #[tokio::test]
    async fn failed_node_is_redispatched_within_attempts() {
        let mut sched = Scheduler::new(diamond(), MemDispatcher::new());
        sched.tick().await.unwrap();
        assert_eq!(sched.on_completion(Completion::Failed { node: NodeId::new("A") }).await.unwrap(), 1);
        assert_eq!(sched.status(), RunStatus::Running);
        assert_eq!(sched.dispatcher.sent().last().unwrap().1.attempt, 2);
    }
}
