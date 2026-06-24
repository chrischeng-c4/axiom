//! Completed-DAG garbage collection (#106, in-scope).
//!
//! A controller background task periodically reaps terminal (succeeded/failed)
//! runs from the [`RunStore`] once they have been terminal for longer than a
//! retention window — so the store does not grow without bound. The selection
//! is a pure function over a "first seen terminal" map (best-effort: the map is
//! in-memory, so a controller restart simply re-observes and waits the window
//! again, which is fine for GC).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use crate::model::{RunStatus, WorkflowRunId};
use crate::store::RunStore;

/// Decide which terminal runs to delete. Records newly-terminal ids at `now`
/// (unix seconds), prunes entries that are no longer terminal/present, and
/// returns (and forgets) ids whose terminal age has reached `retention_secs`.
pub fn gc_select(
    terminal: &[WorkflowRunId],
    seen: &mut BTreeMap<WorkflowRunId, u64>,
    now: u64,
    retention_secs: u64,
) -> Vec<WorkflowRunId> {
    let present: BTreeSet<&WorkflowRunId> = terminal.iter().collect();
    seen.retain(|id, _| present.contains(id));
    let mut to_delete = Vec::new();
    for id in terminal {
        let first = *seen.entry(id.clone()).or_insert(now);
        if now.saturating_sub(first) >= retention_secs {
            to_delete.push(id.clone());
        }
    }
    for id in &to_delete {
        seen.remove(id);
    }
    to_delete
}

fn now_secs() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}

/// Background reaper: every `interval`, find terminal runs and delete those past
/// the retention window. Runs until the controller exits.
pub async fn gc_loop(store: Arc<dyn RunStore>, retention_secs: u64) {
    let interval = Duration::from_secs((retention_secs / 4).clamp(5, 300));
    let mut seen: BTreeMap<WorkflowRunId, u64> = BTreeMap::new();
    eprintln!("loom: completed-DAG GC on (retention {retention_secs}s, sweep {}s)", interval.as_secs());
    loop {
        tokio::time::sleep(interval).await;
        let ids = match store.list().await {
            Ok(ids) => ids,
            Err(_) => continue,
        };
        let mut terminal = Vec::new();
        for id in ids {
            if let Ok(Some(run)) = store.get(&id).await {
                if matches!(run.status, RunStatus::Succeeded | RunStatus::Failed) {
                    terminal.push(id);
                }
            }
        }
        for id in gc_select(&terminal, &mut seen, now_secs(), retention_secs) {
            let _ = store.delete(&id).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> WorkflowRunId {
        WorkflowRunId::new(s)
    }

    #[test]
    fn reaps_only_after_retention_and_forgets_deleted() {
        let mut seen = BTreeMap::new();
        let terminal = vec![id("r1"), id("r2")];
        // first sweep at t=100: both newly seen, none old enough (retention 60)
        assert!(gc_select(&terminal, &mut seen, 100, 60).is_empty());
        assert_eq!(seen.len(), 2);
        // t=150: still under 60s for both
        assert!(gc_select(&terminal, &mut seen, 150, 60).is_empty());
        // t=161: r1/r2 first seen at 100 → age 61 ≥ 60 → both reaped + forgotten
        let mut reaped = gc_select(&terminal, &mut seen, 161, 60);
        reaped.sort();
        assert_eq!(reaped, vec![id("r1"), id("r2")]);
        assert!(seen.is_empty());
    }

    #[test]
    fn drops_runs_no_longer_terminal_or_present() {
        let mut seen = BTreeMap::new();
        gc_select(&[id("r1")], &mut seen, 10, 60);
        assert_eq!(seen.len(), 1);
        // r1 gone from the terminal set (e.g. already deleted) → pruned, not reaped
        let reaped = gc_select(&[], &mut seen, 200, 60);
        assert!(reaped.is_empty());
        assert!(seen.is_empty());
    }
}
