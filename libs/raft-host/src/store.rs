//! Durable storage for a raft node's hard state.
//!
//! Persists [`PersistedState`] (term, votedFor, log, snapshot) to a single file
//! under a data dir, written atomically (temp + rename) and fsynced per
//! [`FsyncPolicy`]. The host calls [`RaftStore::save`] *before* flushing the
//! node's outbox, so no vote or ack is sent before the decision that produced it
//! is durable. (Lifted from lumen/relay's identical `raft_store`.)

use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::path::PathBuf;

use raft_core::{NodeId, PersistedState};

/// How aggressively hard state is flushed to disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FsyncPolicy {
    /// `sync_all` after every write (strongest, default — hard state is tiny).
    #[default]
    Always,
    /// Flush the writer but defer the OS fsync.
    Interval,
    /// Leave flushing to the OS page cache (fastest, weakest).
    Os,
}

/// File-backed persistence for one raft node.
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
