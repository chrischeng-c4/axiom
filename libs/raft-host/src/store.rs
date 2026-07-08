// SPEC-MANAGED: libs/raft-host/tech-design/semantic/source/libs-raft-host-src-store-rs.md#rust-source-unit
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

use raft_core::{NodeId, PersistedState};
pub use service_durability::FsyncPolicy;

/// File-backed persistence for one raft node.
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-store-rs.md#source
pub struct RaftStore {
    path: PathBuf,
    fsync: FsyncPolicy,
}

/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-store-rs.md#source
impl RaftStore {
    /// Open (creating the dir if needed) the state file `raft-<node_id>.state`.
    pub fn open(dir: &str, node_id: NodeId, fsync: FsyncPolicy) -> io::Result<RaftStore> {
        let dir = PathBuf::from(dir);
        create_dir_all(&dir)?;
        Ok(RaftStore {
            path: dir.join(format!("raft-{node_id}.state")),
            fsync,
        })
    }

    /// Durably persist the hard state (atomic temp-write + rename, fsync unless
    /// [`FsyncPolicy::Os`]).
    pub fn save(&self, state: &PersistedState) -> io::Result<()> {
        let bytes =
            serde_json::to_vec(state).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        service_durability::atomic_write(&self.path, &bytes, self.fsync)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    /// Load the persisted hard state, or `None` if this node has none yet.
    pub fn load(&self) -> io::Result<Option<PersistedState>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => {
                Ok(Some(serde_json::from_slice(&bytes).map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}
// CODEGEN-END
