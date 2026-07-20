// SPEC-MANAGED: libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-store-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Durable storage for a raft node's hard state.
//!
//! Persists [`PersistedState`] (term, votedFor, log, snapshot) to a single file
//! under a data dir, written atomically (temp + rename) and fsynced per
//! [`FsyncPolicy`]. The host calls [`RaftStore::save`] *before* flushing the
//! node's outbox, so no vote or ack is sent before the decision that produced it
//! is durable. (Lifted from lumen/relay's identical `raft_store`.)

use std::fs::create_dir_all;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use raft_core::{Index, NodeId, PersistedState, Term};
pub use storage_durable::FsyncPolicy;

/// File-backed persistence for one raft node.
/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-store-rs.md#source
pub struct RaftStore {
    path: PathBuf,
    fsync: FsyncPolicy,
    last_saved: Mutex<Option<Vec<u8>>>,
}

/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-store-rs.md#source
impl RaftStore {
    /// Open (creating the dir if needed) the state file `raft-<node_id>.state`.
    pub fn open(dir: &str, node_id: NodeId, fsync: FsyncPolicy) -> io::Result<RaftStore> {
        let dir = PathBuf::from(dir);
        create_dir_all(&dir)?;
        Ok(RaftStore {
            path: dir.join(format!("raft-{node_id}.state")),
            fsync,
            last_saved: Mutex::new(None),
        })
    }

    /// Durably persist the hard state (atomic temp-write + rename, fsync unless
    /// [`FsyncPolicy::Os`]).
    pub fn save(&self, state: &PersistedState) -> io::Result<()> {
        let bytes =
            serde_json::to_vec(state).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut last_saved = self.last_saved.lock().expect("raft store cache poisoned");
        if last_saved.as_deref() == Some(bytes.as_slice()) {
            return Ok(());
        }
        storage_durable::atomic_write(&self.path, &bytes, self.fsync).map_err(io::Error::other)?;
        *last_saved = Some(bytes);
        Ok(())
    }

    /// Load the persisted hard state, or `None` if this node has none yet.
    pub fn load(&self) -> io::Result<Option<PersistedState>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => {
                let state = serde_json::from_slice(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                *self.last_saved.lock().expect("raft store cache poisoned") = Some(bytes);
                Ok(Some(state))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Seed a brand-new node with an externally supplied state-machine
    /// snapshot. The snapshot becomes the node's committed compaction point,
    /// so the next command starts at `snapshot_index + 1`. Existing hard state
    /// is never overwritten.
    pub fn seed_snapshot(
        &self,
        snapshot_index: Index,
        snapshot_term: Term,
        snapshot: Vec<u8>,
    ) -> io::Result<()> {
        if self.path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("raft state already exists at {}", self.path.display()),
            ));
        }
        self.save(&PersistedState {
            term: snapshot_term,
            voted_for: None,
            log: Vec::new(),
            commit_index: snapshot_index,
            snapshot_index,
            snapshot_term,
            snapshot,
        })
    }
}
// CODEGEN-END
