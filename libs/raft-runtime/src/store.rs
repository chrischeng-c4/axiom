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

use raft_core::{Index, NodeId, PersistedState, RaftEntry, Term};
use sha2::{Digest, Sha256};
pub use storage_durable::FsyncPolicy;

use crate::group::{GroupId, LEGACY_GROUP_ID};

const MAGIC_V1: &[u8; 8] = b"RAFTST01";

/// File-backed persistence for one raft node.
/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-store-rs.md#source
pub struct RaftStore {
    path: PathBuf,
    fsync: FsyncPolicy,
    last_saved: Mutex<Option<[u8; 32]>>,
    injected_save_failure: Mutex<Option<io::ErrorKind>>,
}

/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-store-rs.md#source
impl RaftStore {
    /// Open (creating the dir if needed) the state file `raft-<node_id>.state`.
    pub fn open(dir: &str, node_id: NodeId, fsync: FsyncPolicy) -> io::Result<RaftStore> {
        Self::open_group(dir, node_id, GroupId(LEGACY_GROUP_ID.to_string()), fsync)
    }

    pub fn open_group(
        dir: &str,
        node_id: NodeId,
        group_id: GroupId,
        fsync: FsyncPolicy,
    ) -> io::Result<RaftStore> {
        let dir = PathBuf::from(dir);
        create_dir_all(&dir)?;
        let filename = if group_id.0 == LEGACY_GROUP_ID {
            format!("raft-{node_id}.state")
        } else {
            let mut s = String::new();
            for b in group_id.0.as_bytes() {
                use std::fmt::Write;
                write!(&mut s, "{:02x}", b).unwrap();
            }
            format!("raft-{node_id}-{s}.state")
        };
        Ok(RaftStore {
            path: dir.join(filename),
            fsync,
            last_saved: Mutex::new(None),
            injected_save_failure: Mutex::new(None),
        })
    }

    /// Fault-injection seam for testing durable persistence failures.
    pub fn inject_next_save_failure_with_kind(&self, kind: io::ErrorKind) {
        *self
            .injected_save_failure
            .lock()
            .expect("injected_save_failure mutex poisoned") = Some(kind);
    }

    /// Access the file path of this store.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Returns the number of heap/retained bytes in the save-dedup cache.
    pub fn cache_footprint(&self) -> usize {
        if self
            .last_saved
            .lock()
            .expect("last_saved mutex poisoned")
            .is_some()
        {
            std::mem::size_of::<[u8; 32]>()
        } else {
            0
        }
    }

    /// Durably persist the hard state (atomic temp-write + rename, fsync unless
    /// [`FsyncPolicy::Os`]).
    pub fn save(&self, state: &PersistedState) -> io::Result<()> {
        let bytes = encode_persisted_state(state);
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        let mut last_saved = self.last_saved.lock().expect("raft store cache poisoned");
        if *last_saved == Some(digest) {
            return Ok(());
        }
        if let Some(kind) = self
            .injected_save_failure
            .lock()
            .expect("injected_save_failure mutex poisoned")
            .take()
        {
            return Err(io::Error::new(
                kind,
                "injected save failure (fault-injection seam)",
            ));
        }
        storage_durable::atomic_write(&self.path, &bytes, self.fsync).map_err(io::Error::other)?;
        *last_saved = Some(digest);
        Ok(())
    }

    /// Load the persisted hard state, or `None` if this node has none yet.
    pub fn load(&self) -> io::Result<Option<PersistedState>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => {
                let state = decode_persisted_state(&bytes)?;
                let digest: [u8; 32] = Sha256::digest(&encode_persisted_state(&state)).into();
                *self.last_saved.lock().expect("raft store cache poisoned") = Some(digest);
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

fn encode_persisted_state(state: &PersistedState) -> Vec<u8> {
    let mut buf = Vec::with_capacity(65 + state.snapshot.len() + state.log.len() * 32);
    buf.extend_from_slice(MAGIC_V1);
    buf.extend_from_slice(&state.term.to_le_bytes());
    match state.voted_for {
        Some(node_id) => {
            buf.push(1);
            buf.extend_from_slice(&node_id.to_le_bytes());
        }
        None => {
            buf.push(0);
        }
    }
    buf.extend_from_slice(&state.commit_index.to_le_bytes());
    buf.extend_from_slice(&state.snapshot_index.to_le_bytes());
    buf.extend_from_slice(&state.snapshot_term.to_le_bytes());
    buf.extend_from_slice(&(state.snapshot.len() as u64).to_le_bytes());
    buf.extend_from_slice(&state.snapshot);
    buf.extend_from_slice(&(state.log.len() as u64).to_le_bytes());
    for entry in &state.log {
        buf.extend_from_slice(&entry.term.to_le_bytes());
        buf.extend_from_slice(&entry.index.to_le_bytes());
        buf.extend_from_slice(&(entry.command.len() as u64).to_le_bytes());
        buf.extend_from_slice(&entry.command);
    }
    buf
}

struct CursorReader<'a>(&'a [u8]);

impl<'a> CursorReader<'a> {
    fn read_u8(&mut self) -> io::Result<u8> {
        if self.0.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected eof reading u8",
            ));
        }
        let b = self.0[0];
        self.0 = &self.0[1..];
        Ok(b)
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        if self.0.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected eof reading u64",
            ));
        }
        let (num_bytes, rest) = self.0.split_at(8);
        self.0 = rest;
        Ok(u64::from_le_bytes(num_bytes.try_into().unwrap()))
    }

    fn read_bytes(&mut self, len: usize) -> io::Result<Vec<u8>> {
        if self.0.len() < len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected eof reading byte payload",
            ));
        }
        let (data, rest) = self.0.split_at(len);
        self.0 = rest;
        Ok(data.to_vec())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn decode_persisted_state(bytes: &[u8]) -> io::Result<PersistedState> {
    if bytes.starts_with(MAGIC_V1) {
        let mut r = CursorReader(&bytes[MAGIC_V1.len()..]);
        let term = r.read_u64()?;
        let has_voted_for = r.read_u8()?;
        let voted_for = match has_voted_for {
            0 => None,
            1 => Some(r.read_u64()?),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid voted_for tag",
                ))
            }
        };
        let commit_index = r.read_u64()?;
        let snapshot_index = r.read_u64()?;
        let snapshot_term = r.read_u64()?;
        let snapshot_len = r.read_u64()? as usize;
        let snapshot = r.read_bytes(snapshot_len)?;
        let log_len = r.read_u64()? as usize;
        let mut log = Vec::with_capacity(log_len.min(100_000));
        for _ in 0..log_len {
            let entry_term = r.read_u64()?;
            let entry_index = r.read_u64()?;
            let command_len = r.read_u64()? as usize;
            let command = r.read_bytes(command_len)?;
            log.push(RaftEntry {
                term: entry_term,
                index: entry_index,
                command,
            });
        }
        if !r.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing bytes after persisted state",
            ));
        }
        Ok(PersistedState {
            term,
            voted_for,
            log,
            commit_index,
            snapshot_index,
            snapshot_term,
            snapshot,
        })
    } else if let Some(&first_non_ws) = bytes.iter().find(|&&b| !b.is_ascii_whitespace()) {
        if first_non_ws == b'{' {
            serde_json::from_slice(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unrecognised durable state format marker",
            ))
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "empty durable state file",
        ))
    }
}
// CODEGEN-END
