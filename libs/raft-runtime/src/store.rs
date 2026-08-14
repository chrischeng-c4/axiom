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
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use raft_core::{Index, NodeId, PersistedState, RaftEntry, Term};
use sha2::{Digest, Sha256};
pub use storage_durable::FsyncPolicy;

use crate::group::{GroupId, LEGACY_GROUP_ID};

const MAGIC_V1: &[u8; 8] = b"RAFTST01";
const MAGIC_V2: &[u8; 8] = b"RAFTST02";

/// File-backed persistence for one raft node.
/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-store-rs.md#source
pub struct RaftStore {
    path: PathBuf,
    fsync: FsyncPolicy,
    last_saved: Mutex<Option<[u8; 32]>>,
    injected_save_failure: Mutex<Option<io::ErrorKind>>,
    injected_after_artifact_failure: Mutex<Option<io::ErrorKind>>,
    injected_after_publish_failure: Mutex<Option<io::ErrorKind>>,
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
        if group_id.0 != LEGACY_GROUP_ID {
            let legacy_file = dir.join(format!("raft-{node_id}.state"));
            if legacy_file.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "cannot open named group {:?} for node {node_id}: legacy state file raft-{node_id}.state exists and must be explicitly migrated",
                        group_id.0
                    ),
                ));
            }
        }
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
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
        })
    }

    /// Explicitly migrate a node's legacy single-group state file and its snapshot
    /// artifacts to a named group.
    ///
    /// Validates that the legacy state file exists and successfully decodes, and that
    /// the target named group state file does not already exist, before moving any files.
    pub fn migrate_legacy_to_group(
        dir: &str,
        node_id: NodeId,
        target_group: GroupId,
        fsync: FsyncPolicy,
    ) -> io::Result<RaftStore> {
        if target_group.0 == LEGACY_GROUP_ID {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot migrate legacy state to legacy group ID",
            ));
        }

        let dir_path = PathBuf::from(dir);
        let legacy_filename = format!("raft-{node_id}.state");
        let legacy_path = dir_path.join(&legacy_filename);
        if !legacy_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("legacy state file not found: {}", legacy_path.display()),
            ));
        }

        let mut s = String::new();
        for b in target_group.0.as_bytes() {
            use std::fmt::Write;
            write!(&mut s, "{:02x}", b).unwrap();
        }
        let target_filename = format!("raft-{node_id}-{s}.state");
        let target_path = dir_path.join(&target_filename);
        if target_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "target state file already exists: {}",
                    target_path.display()
                ),
            ));
        }

        let legacy_store = RaftStore {
            path: legacy_path.clone(),
            fsync,
            last_saved: Mutex::new(None),
            injected_save_failure: Mutex::new(None),
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
        };

        // Read and decode to ensure valid format before any filesystem mutations
        let legacy_bytes = std::fs::read(&legacy_path)?;
        let _state = legacy_store.decode_persisted_state(&legacy_bytes)?;

        // Discover all associated snapshot artifact files to rename
        let legacy_stem = format!("raft-{node_id}");
        let legacy_prefix = format!("{legacy_stem}-snap-");
        let target_stem = format!("raft-{node_id}-{s}");

        let mut artifacts_to_rename = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(fname) = p.file_name().and_then(|n| n.to_str()) {
                    if fname.starts_with(&legacy_prefix) && fname.ends_with(".artifact") {
                        let suffix = &fname[legacy_prefix.len()..];
                        let target_art_name = format!("{target_stem}-snap-{suffix}");
                        let target_art_path = dir_path.join(target_art_name);
                        if target_art_path.exists() {
                            return Err(io::Error::new(
                                io::ErrorKind::AlreadyExists,
                                format!(
                                    "target artifact file already exists: {}",
                                    target_art_path.display()
                                ),
                            ));
                        }
                        artifacts_to_rename.push((p, target_art_path));
                    }
                }
            }
        }

        // Perform renames
        for (src, dst) in &artifacts_to_rename {
            std::fs::rename(src, dst)?;
        }
        std::fs::rename(&legacy_path, &target_path)?;

        Ok(RaftStore {
            path: target_path,
            fsync,
            last_saved: Mutex::new(None),
            injected_save_failure: Mutex::new(None),
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
        })
    }

    /// Fault-injection seam for testing durable persistence failures before save.
    pub fn inject_next_save_failure_with_kind(&self, kind: io::ErrorKind) {
        *self
            .injected_save_failure
            .lock()
            .expect("injected_save_failure mutex poisoned") = Some(kind);
    }

    /// Fault-injection seam armed after the snapshot artifact is written and before the hard-state reference is published.
    pub fn inject_next_after_artifact_failure_with_kind(&self, kind: io::ErrorKind) {
        *self
            .injected_after_artifact_failure
            .lock()
            .expect("injected_after_artifact_failure mutex poisoned") = Some(kind);
    }

    /// Fault-injection seam armed after the hard-state reference is published and before superseded artifact collection.
    pub fn inject_next_after_publish_failure_with_kind(&self, kind: io::ErrorKind) {
        *self
            .injected_after_publish_failure
            .lock()
            .expect("injected_after_publish_failure mutex poisoned") = Some(kind);
    }

    /// Access the file path of this store.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Returns the snapshot artifact path for a given generation.
    pub fn artifact_path(&self, snapshot_index: Index, snapshot_term: Term) -> PathBuf {
        let stem = self
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("raft");
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        dir.join(format!(
            "{stem}-snap-{snapshot_index}-{snapshot_term}.artifact"
        ))
    }

    fn collect_superseded_artifacts(&self, current_artifact: Option<&Path>) -> io::Result<()> {
        let stem = self
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("raft");
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let prefix = format!("{stem}-snap-");
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with(&prefix) && name.ends_with(".artifact") {
                        if current_artifact.map_or(true, |cur| cur != path.as_path()) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }
        Ok(())
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
        let snapshot_digest: [u8; 32] = Sha256::digest(&state.snapshot).into();
        let bytes = encode_persisted_state_v2(state, &snapshot_digest);
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let digest: [u8; 32] = hasher.finalize().into();

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

        let current_artifact = if !state.snapshot.is_empty() {
            let art_path = self.artifact_path(state.snapshot_index, state.snapshot_term);
            if !art_path.exists() {
                storage_durable::atomic_write(&art_path, &state.snapshot, self.fsync)
                    .map_err(io::Error::other)?;
            }
            Some(art_path)
        } else {
            None
        };

        if let Some(kind) = self
            .injected_after_artifact_failure
            .lock()
            .expect("injected_after_artifact_failure mutex poisoned")
            .take()
        {
            return Err(io::Error::new(
                kind,
                "injected after-artifact failure (fault-injection seam)",
            ));
        }

        storage_durable::atomic_write(&self.path, &bytes, self.fsync).map_err(io::Error::other)?;

        if let Some(kind) = self
            .injected_after_publish_failure
            .lock()
            .expect("injected_after_publish_failure mutex poisoned")
            .take()
        {
            return Err(io::Error::new(
                kind,
                "injected after-publish failure (fault-injection seam)",
            ));
        }

        let _ = self.collect_superseded_artifacts(current_artifact.as_deref());
        *last_saved = Some(digest);
        Ok(())
    }

    /// Load the persisted hard state, or `None` if this node has none yet.
    pub fn load(&self) -> io::Result<Option<PersistedState>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => {
                let state = self.decode_persisted_state(&bytes)?;
                let snapshot_digest: [u8; 32] = Sha256::digest(&state.snapshot).into();
                let hard_bytes = encode_persisted_state_v2(&state, &snapshot_digest);
                let mut hasher = Sha256::new();
                hasher.update(&hard_bytes);
                let digest: [u8; 32] = hasher.finalize().into();
                *self.last_saved.lock().expect("raft store cache poisoned") = Some(digest);
                Ok(Some(state))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn decode_persisted_state(&self, bytes: &[u8]) -> io::Result<PersistedState> {
        if bytes.starts_with(MAGIC_V2) {
            let mut r = CursorReader(&bytes[MAGIC_V2.len()..]);
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
            let snapshot_digest = r.read_bytes(32)?;

            let snapshot = if snapshot_len == 0 {
                Vec::new()
            } else {
                let art_path = self.artifact_path(snapshot_index, snapshot_term);
                if !art_path.exists() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "missing snapshot artifact for index {snapshot_index} term {snapshot_term}"
                        ),
                    ));
                }
                let art_bytes = std::fs::read(&art_path)?;
                if art_bytes.len() != snapshot_len {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "snapshot artifact truncated: expected {snapshot_len} bytes, found {}",
                            art_bytes.len()
                        ),
                    ));
                }
                let actual_digest: [u8; 32] = Sha256::digest(&art_bytes).into();
                if actual_digest.as_slice() != snapshot_digest.as_slice() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "snapshot artifact digest mismatch (content corrupted)",
                    ));
                }
                art_bytes
            };

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
        } else if bytes.starts_with(MAGIC_V1) {
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
                serde_json::from_slice(bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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

fn encode_persisted_state_v2(state: &PersistedState, snapshot_digest: &[u8; 32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(97 + state.log.len() * 32);
    buf.extend_from_slice(MAGIC_V2);
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
    buf.extend_from_slice(snapshot_digest);
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
// CODEGEN-END
