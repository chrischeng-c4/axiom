//! Run-state store (#110 / #123) — where loom persists its sharded DAG state.
//!
//! The control plane reads and writes [`WorkflowRun`]s through this trait. The
//! in-memory [`MemStore`] is the dev/test backend and the reference semantics;
//! the durable, crash-recoverable, per-shard raft-backed store (#110 ADR, #123)
//! plugs in behind the same trait without touching the scheduler or API.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::model::{WorkflowRun, WorkflowRunId};

/// Persistence boundary for loom's workflow state. Implementations own
/// durability and (eventually) per-shard consensus; callers see only get/put/list.
pub trait RunStore: Send + Sync {
    /// Insert or replace a run.
    fn put(&self, run: WorkflowRun) -> anyhow::Result<()>;
    /// Fetch a run by id, if present.
    fn get(&self, id: &WorkflowRunId) -> anyhow::Result<Option<WorkflowRun>>;
    /// List all run ids (ordered).
    fn list(&self) -> anyhow::Result<Vec<WorkflowRunId>>;
}

/// In-memory reference store: a `Mutex<BTreeMap>`. Not durable — replaced by the
/// raft-backed store for production (#110 / #123).
#[derive(Default)]
pub struct MemStore {
    runs: Mutex<BTreeMap<WorkflowRunId, WorkflowRun>>,
}

impl MemStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RunStore for MemStore {
    fn put(&self, run: WorkflowRun) -> anyhow::Result<()> {
        self.runs
            .lock()
            .map_err(|_| anyhow::anyhow!("run store poisoned"))?
            .insert(run.id.clone(), run);
        Ok(())
    }

    fn get(&self, id: &WorkflowRunId) -> anyhow::Result<Option<WorkflowRun>> {
        Ok(self
            .runs
            .lock()
            .map_err(|_| anyhow::anyhow!("run store poisoned"))?
            .get(id)
            .cloned())
    }

    fn list(&self) -> anyhow::Result<Vec<WorkflowRunId>> {
        Ok(self
            .runs
            .lock()
            .map_err(|_| anyhow::anyhow!("run store poisoned"))?
            .keys()
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_list_roundtrip() {
        let store = MemStore::new();
        assert!(store.list().unwrap().is_empty());

        let id = WorkflowRunId::new("run-1");
        store.put(WorkflowRun::new(id.clone())).unwrap();

        assert_eq!(store.get(&id).unwrap().unwrap().id, id);
        assert_eq!(store.list().unwrap(), vec![id.clone()]);
        assert!(store.get(&WorkflowRunId::new("missing")).unwrap().is_none());
    }

    #[test]
    fn put_replaces_existing() {
        let store = MemStore::new();
        let id = WorkflowRunId::new("run-1");
        let mut run = WorkflowRun::new(id.clone());
        run.status = crate::model::RunStatus::Running;
        store.put(run).unwrap();
        assert_eq!(store.get(&id).unwrap().unwrap().status, crate::model::RunStatus::Running);
        assert_eq!(store.list().unwrap().len(), 1);
    }
}
