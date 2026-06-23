//! Run-state store (#110 / #123) — where loom persists its sharded DAG state.
//!
//! The control plane reads and writes [`WorkflowRun`]s through this trait. The
//! in-memory [`MemStore`] is the dev/test backend and the reference semantics;
//! the durable, crash-recoverable, per-shard raft-backed store (#110 ADR, #123)
//! plugs in behind the same trait without touching the scheduler or API.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
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

/// File-backed store with crash recovery (#123): runs are persisted to disk and
/// reloaded on open, so a controller restart resumes its in-flight DAGs. Writes
/// go through an atomic temp-file rename. A read cache fronts the file; the
/// durable, multi-node raft-backed store (#110) layers on the same trait later.
pub struct FileStore {
    path: PathBuf,
    cache: Mutex<BTreeMap<WorkflowRunId, WorkflowRun>>,
}

impl FileStore {
    /// Open (or create) a store under `dir`, loading any persisted runs.
    pub fn open(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let path = dir.join("runs.json");
        let cache = if path.exists() {
            let bytes = std::fs::read(&path)?;
            let runs: Vec<WorkflowRun> = serde_json::from_slice(&bytes).unwrap_or_default();
            runs.into_iter().map(|r| (r.id.clone(), r)).collect()
        } else {
            BTreeMap::new()
        };
        Ok(Self { path, cache: Mutex::new(cache) })
    }

    fn persist(&self, map: &BTreeMap<WorkflowRunId, WorkflowRun>) -> anyhow::Result<()> {
        let runs: Vec<&WorkflowRun> = map.values().collect();
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec(&runs)?)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

impl RunStore for FileStore {
    fn put(&self, run: WorkflowRun) -> anyhow::Result<()> {
        let mut g = self.cache.lock().map_err(|_| anyhow::anyhow!("run store poisoned"))?;
        g.insert(run.id.clone(), run);
        self.persist(&g)
    }

    fn get(&self, id: &WorkflowRunId) -> anyhow::Result<Option<WorkflowRun>> {
        Ok(self
            .cache
            .lock()
            .map_err(|_| anyhow::anyhow!("run store poisoned"))?
            .get(id)
            .cloned())
    }

    fn list(&self) -> anyhow::Result<Vec<WorkflowRunId>> {
        Ok(self
            .cache
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

    #[test]
    fn file_store_survives_reopen() {
        let dir = std::env::temp_dir().join(format!("loom-fs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let id = WorkflowRunId::new("persisted");

        {
            let store = FileStore::open(&dir).unwrap();
            let mut run = WorkflowRun::new(id.clone());
            run.status = crate::model::RunStatus::Running;
            store.put(run).unwrap();
        } // drop: only the on-disk file remains

        // A fresh FileStore on the same dir recovers the run (crash recovery).
        let recovered = FileStore::open(&dir).unwrap();
        assert_eq!(recovered.list().unwrap(), vec![id.clone()]);
        assert_eq!(
            recovered.get(&id).unwrap().unwrap().status,
            crate::model::RunStatus::Running
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
