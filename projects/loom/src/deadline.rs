//! Dispatch deadline (#438) — re-dispatch a node that was dispatched but whose
//! completion never arrived (a worker that acked but never reported, or crashed
//! after ack before report). Relay redelivery only covers *un-acked* entries, so
//! loom needs its own backstop for the acked-but-silent case.
//!
//! Opt-in via `LOOM_DISPATCH_DEADLINE_SECS`: there is no worker heartbeat yet, so
//! a too-short deadline would re-dispatch a still-running task — set it above your
//! longest task. An overdue node is treated as a failed attempt (retry-or-fail),
//! and the stale attempt's late completion is rejected by the #437 idempotency
//! guard, so re-dispatch never double-runs.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use crate::model::{NodeId, NodeState, WorkflowRun};
use crate::scheduler::{dispatch_ready, Dispatcher};
use crate::store::RunStore;

/// One in-flight attempt across sweeps: (run, node, attempt).
type Key = (String, String, u32);

fn now_secs() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}

/// Record this run's in-flight (Dispatched/Running) nodes by attempt, prune
/// keys that are no longer in flight, and mark-failed (retry-or-fail) any node
/// first seen in flight ≥ `deadline` ago. Returns the overdue node ids so the
/// caller can re-dispatch + persist. Pure + deterministic in `now`.
pub fn sweep_run(
    run: &mut WorkflowRun,
    seen: &mut BTreeMap<Key, u64>,
    now: u64,
    deadline: u64,
) -> Vec<NodeId> {
    let rid = run.id.0.clone();
    let inflight: Vec<(NodeId, u32)> = run
        .nodes
        .values()
        .filter(|n| matches!(n.state, NodeState::Dispatched | NodeState::Running))
        .map(|n| (n.id.clone(), n.attempt))
        .collect();
    let live: BTreeSet<Key> =
        inflight.iter().map(|(id, a)| (rid.clone(), id.0.clone(), *a)).collect();
    // forget keys for this run that are no longer in flight (completed/retried)
    seen.retain(|k, _| k.0 != rid || live.contains(k));

    let mut overdue = Vec::new();
    for (id, attempt) in inflight {
        let key = (rid.clone(), id.0.clone(), attempt);
        let first = *seen.entry(key.clone()).or_insert(now);
        if now.saturating_sub(first) >= deadline {
            overdue.push(id.clone());
            seen.remove(&key); // the retry gets a fresh window
        }
    }
    for id in &overdue {
        run.mark_failed(id); // retry-or-fail; #437 rejects the old attempt's late completion
    }
    overdue
}

/// Background backstop: every sweep, re-dispatch overdue nodes across all runs.
pub async fn deadline_loop(
    store: Arc<dyn RunStore>,
    dispatcher: Arc<dyn Dispatcher>,
    deadline_secs: u64,
) {
    let interval = Duration::from_secs((deadline_secs / 4).clamp(5, 300));
    let mut seen: BTreeMap<Key, u64> = BTreeMap::new();
    eprintln!("loom: dispatch deadline on ({deadline_secs}s, sweep {}s)", interval.as_secs());
    loop {
        tokio::time::sleep(interval).await;
        let now = now_secs();
        let ids = store.list().await.unwrap_or_default();
        for id in ids {
            let Ok(Some(mut run)) = store.get(&id).await else { continue };
            let overdue = sweep_run(&mut run, &mut seen, now, deadline_secs);
            if !overdue.is_empty() {
                eprintln!("loom: re-dispatching {} overdue node(s) in {}", overdue.len(), id.0);
                let _ = dispatch_ready(&mut run, dispatcher.as_ref()).await;
                let _ = store.put(run).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Node, NodeId, StageId, TaskSpec};
    use std::collections::BTreeSet as Set;

    fn dispatched_run() -> WorkflowRun {
        let mut run = WorkflowRun::new(crate::model::WorkflowRunId::new("r"));
        run.add_node(Node::new(NodeId::new("a"), StageId::new("a"), TaskSpec::new("t"), Set::new()));
        run.mark_dispatched(&NodeId::new("a")); // Dispatched, attempt = 1
        run
    }

    #[test]
    fn overdue_only_after_deadline_then_retries() {
        let mut run = dispatched_run();
        let mut seen = BTreeMap::new();
        // first sighting: records, not overdue
        assert!(sweep_run(&mut run, &mut seen, 100, 60).is_empty());
        assert_eq!(run.nodes[&NodeId::new("a")].state, NodeState::Dispatched);
        // still under the deadline
        assert!(sweep_run(&mut run, &mut seen, 150, 60).is_empty());
        // past the deadline (first seen 100, now 161) → overdue → marked failed→ready (retry)
        assert_eq!(sweep_run(&mut run, &mut seen, 161, 60), vec![NodeId::new("a")]);
        assert_eq!(run.nodes[&NodeId::new("a")].state, NodeState::Ready);
        assert!(seen.is_empty(), "the overdue attempt is forgotten so the retry gets a fresh window");
    }

    #[test]
    fn completed_node_is_pruned_not_overdue() {
        let mut run = dispatched_run();
        let mut seen = BTreeMap::new();
        sweep_run(&mut run, &mut seen, 10, 60);
        assert_eq!(seen.len(), 1);
        // node completes → no longer in flight
        run.mark_done(&NodeId::new("a"), None);
        let overdue = sweep_run(&mut run, &mut seen, 200, 60);
        assert!(overdue.is_empty());
        assert!(seen.is_empty(), "completed node's key pruned");
    }
}
