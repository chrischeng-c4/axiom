// SPEC-MANAGED: projects/relay/tech-design/logic/raft-hard-state-persistence-fsyncpolicy-crash-safe-single-voter.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:29c02a4d" tracker="pending-tracker" reason="File-backed RaftStore: open(dir, node_id, FsyncPolicy), save(&PersistedState) writing term/votedFor + the log under data_dir and fsyncing per policy (hard state always durable), and load() -> Option<PersistedState> (None for an empty dir). No external dependency."
//! Durable storage for a Raft node's hard state.
//!
//! Persists [`PersistedState`] (term, votedFor, log) to a single file under a
//! data dir, written atomically (temp + rename) and fsynced per [`FsyncPolicy`]
//! — hard state is tiny and safety-critical, so it is made durable unless fsync
//! is explicitly disabled. The driver calls [`RaftStore::save`] *before*
//! flushing a node's outbox, so no vote or ack is ever sent before the decision
//! that produced it is durable.

use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config::FsyncPolicy;
use crate::raft::{NodeId, PersistedState};

/// File-backed persistence for one Raft node.
///
/// @spec projects/relay/tech-design/logic/raft-hard-state-persistence-fsyncpolicy-crash-safe-single-voter.md#logic
pub struct RaftStore {
    path: PathBuf,
    fsync: FsyncPolicy,
}

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
        let tmp = self.path.with_extension("tmp");
        {
            let mut f = File::create(&tmp)?;
            f.write_all(&bytes)?;
            if self.fsync != FsyncPolicy::Os {
                f.sync_all()?;
            }
        }
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
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
// HANDWRITE-END
