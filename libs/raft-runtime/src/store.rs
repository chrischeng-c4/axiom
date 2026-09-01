// CODEGEN-BEGIN
//! Durable storage for a raft node's hard state.
//!
//! Persists [`PersistedState`] (term, votedFor, log, snapshot) to a single file
//! under a data dir, written atomically (temp + rename) and fsynced per
//! [`FsyncPolicy`]. The host calls [`RaftStore::save`] *before* flushing the
//! node's outbox, so no vote or ack is sent before the decision that produced it
//! is durable. (Lifted from lumen/relay's identical `raft_store`.)

use std::fs::{create_dir_all, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

use raft_core::{
    ConfState, EntryKind, Index, NodeId, PersistedState, PersistedStateRef, RaftEntry, Term,
};
use sha2::{Digest, Sha256};
pub use storage_durable::FsyncPolicy;

use crate::group::{GroupId, LEGACY_GROUP_ID};

const MAGIC_V1: &[u8; 8] = b"RAFTST01";
const MAGIC_V2: &[u8; 8] = b"RAFTST02";
const MAGIC_V3: &[u8; 8] = b"RAFTST03";
const MAGIC_V4: &[u8; 8] = b"RAFTST04";
const LOG_MAGIC_V1: &[u8; 8] = b"RAFTLG01";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LogLayout {
    generation: [u8; 32],
    byte_len: u64,
    entry_count: u64,
    first_index: u64,
    last_index: u64,
    last_term: u64,
    last_entry_digest: [u8; 32],
}

#[derive(Default)]
struct StoreCache {
    hard_digest: Option<[u8; 32]>,
    log: Option<LogLayout>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PersistenceStats {
    pub log_bytes_appended: u64,
    pub log_bytes_rewritten: u64,
}

/// File-backed persistence for one raft node.
pub struct RaftStore {
    path: PathBuf,
    fsync: FsyncPolicy,
    cache: Mutex<StoreCache>,
    log_bytes_appended: AtomicU64,
    log_bytes_rewritten: AtomicU64,
    injected_save_failure: Mutex<Option<io::ErrorKind>>,
    injected_after_artifact_failure: Mutex<Option<io::ErrorKind>>,
    injected_after_publish_failure: Mutex<Option<io::ErrorKind>>,
}

fn migration_file_digest(path: &Path) -> io::Result<[u8; 32]> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

fn copy_migration_artifact(source: &Path, target: &Path, fsync: FsyncPolicy) -> io::Result<()> {
    if target.exists() {
        if source.metadata()?.len() == target.metadata()?.len()
            && migration_file_digest(source)? == migration_file_digest(target)?
        {
            return Ok(());
        }
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "target artifact file already exists with different content: {}",
                target.display()
            ),
        ));
    }

    let target_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid target artifact"))?;
    let temporary = target.with_file_name(format!(".{target_name}.migration.tmp"));
    match std::fs::remove_file(&temporary) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let copy_result = (|| {
        let mut input = OpenOptions::new().read(true).open(source)?;
        let mut output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        io::copy(&mut input, &mut output)?;
        output.flush()?;
        if fsync != FsyncPolicy::Os {
            output.sync_all()?;
        }
        drop(output);
        std::fs::rename(&temporary, target)?;
        if fsync != FsyncPolicy::Os {
            storage_durable::sync_parent_dir(target).map_err(io::Error::other)?;
        }
        Ok(())
    })();
    if copy_result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    copy_result
}

fn cleanup_migrated_source(
    legacy_path: &Path,
    artifacts: &[(PathBuf, PathBuf)],
    fsync: FsyncPolicy,
) -> io::Result<()> {
    for (source, _) in artifacts {
        match std::fs::remove_file(source) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    match std::fs::remove_file(legacy_path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    if fsync != FsyncPolicy::Os {
        storage_durable::sync_parent_dir(legacy_path).map_err(io::Error::other)?;
    }
    Ok(())
}

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
            cache: Mutex::new(StoreCache::default()),
            log_bytes_appended: AtomicU64::new(0),
            log_bytes_rewritten: AtomicU64::new(0),
            injected_save_failure: Mutex::new(None),
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
        })
    }

    /// Explicitly migrate a node's legacy single-group state file and its snapshot
    /// artifacts to a named group.
    ///
    /// Copies artifacts before publishing the target hard state. The legacy
    /// files remain authoritative until the target store has loaded
    /// successfully, so an interrupted migration can be retried safely.
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
        let legacy_store = RaftStore {
            path: legacy_path.clone(),
            fsync,
            cache: Mutex::new(StoreCache::default()),
            log_bytes_appended: AtomicU64::new(0),
            log_bytes_rewritten: AtomicU64::new(0),
            injected_save_failure: Mutex::new(None),
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
        };

        // Read the hard state before inspecting a possible target. A target
        // with identical bytes is an idempotent retry only when it also loads
        // through the target artifact namespace.
        let legacy_bytes = std::fs::read(&legacy_path)?;

        // Discover all associated snapshot and split-log artifact files. A V4
        // hard state resolves its log artifact from the state-file stem.
        let legacy_stem = format!("raft-{node_id}");
        let target_stem = format!("raft-{node_id}-{s}");
        let artifact_prefixes = [
            (
                format!("{legacy_stem}-snap-"),
                format!("{target_stem}-snap-"),
            ),
            (format!("{legacy_stem}-log-"), format!("{target_stem}-log-")),
        ];

        let mut artifacts = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(fname) = p.file_name().and_then(|n| n.to_str()) {
                    for (legacy_prefix, target_prefix) in &artifact_prefixes {
                        if fname.starts_with(legacy_prefix) && fname.ends_with(".artifact") {
                            let suffix = &fname[legacy_prefix.len()..];
                            let target_art_path = dir_path.join(format!("{target_prefix}{suffix}"));
                            artifacts.push((p.clone(), target_art_path));
                            break;
                        }
                    }
                }
            }
        }

        let target_store = RaftStore {
            path: target_path.clone(),
            fsync,
            cache: Mutex::new(StoreCache::default()),
            log_bytes_appended: AtomicU64::new(0),
            log_bytes_rewritten: AtomicU64::new(0),
            injected_save_failure: Mutex::new(None),
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
        };

        if target_path.exists() {
            if std::fs::read(&target_path)? != legacy_bytes {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "target state file already exists: {}",
                        target_path.display()
                    ),
                ));
            }
            target_store.load()?.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "published Raft migration target has no hard state",
                )
            })?;
            cleanup_migrated_source(&legacy_path, &artifacts, fsync)?;
            return Ok(target_store);
        }

        // Validate the source while every legacy artifact is still present.
        legacy_store.decode_persisted_state(&legacy_bytes)?;
        for (source, target) in &artifacts {
            copy_migration_artifact(source, target, fsync)?;
        }
        storage_durable::atomic_write(&target_path, &legacy_bytes, fsync)
            .map_err(io::Error::other)?;
        target_store.load()?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "published Raft migration target has no hard state",
            )
        })?;
        cleanup_migrated_source(&legacy_path, &artifacts, fsync)?;
        Ok(target_store)
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

    fn log_artifact_path(&self, generation: &[u8; 32]) -> PathBuf {
        let stem = self
            .path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("raft");
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        dir.join(format!("{stem}-log-{}.artifact", hex_digest(generation)))
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

    fn collect_superseded_log_artifacts(&self, current_artifact: Option<&Path>) -> io::Result<()> {
        let stem = self
            .path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("raft");
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let prefix = format!("{stem}-log-");
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
                    if name.starts_with(&prefix)
                        && name.ends_with(".artifact")
                        && current_artifact.map_or(true, |current| current != path.as_path())
                    {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
        Ok(())
    }

    fn read_snapshot_artifact(
        &self,
        snapshot_index: Index,
        snapshot_term: Term,
        snapshot_len: usize,
        snapshot_digest: &[u8],
    ) -> io::Result<Vec<u8>> {
        if snapshot_len == 0 {
            return Ok(Vec::new());
        }
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
        if actual_digest.as_slice() != snapshot_digest {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "snapshot artifact digest mismatch (content corrupted)",
            ));
        }
        Ok(art_bytes)
    }

    /// Returns the number of heap/retained bytes in the save-dedup cache.
    pub fn cache_footprint(&self) -> usize {
        if self
            .cache
            .lock()
            .expect("raft store cache poisoned")
            .hard_digest
            .is_some()
        {
            std::mem::size_of::<[u8; 32]>()
        } else {
            0
        }
    }

    pub fn persistence_stats(&self) -> PersistenceStats {
        PersistenceStats {
            log_bytes_appended: self.log_bytes_appended.load(Ordering::Relaxed),
            log_bytes_rewritten: self.log_bytes_rewritten.load(Ordering::Relaxed),
        }
    }

    /// Durably persist the hard state (atomic temp-write + rename, fsync unless
    /// [`FsyncPolicy::Os`]).
    pub fn save(&self, state: &PersistedState) -> io::Result<()> {
        self.save_ref(&PersistedStateRef {
            term: state.term,
            voted_for: state.voted_for,
            log: &state.log,
            commit_index: state.commit_index,
            snapshot_index: state.snapshot_index,
            snapshot_term: state.snapshot_term,
            snapshot: &state.snapshot,
            conf: state.conf.as_ref(),
        })
    }

    /// Persist borrowed state without cloning the resident Raft log.
    pub fn save_ref(&self, state: &PersistedStateRef<'_>) -> io::Result<()> {
        let mut cache = self.cache.lock().expect("raft store cache poisoned");
        let log_plan = plan_log_write(state, cache.log);
        let next_log = log_plan.layout();
        let snapshot_digest: [u8; 32] = Sha256::digest(state.snapshot).into();
        let bytes = encode_persisted_state_v4(state, &snapshot_digest, next_log);
        let digest: [u8; 32] = Sha256::digest(&bytes).into();

        if cache.hard_digest == Some(digest) {
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
                storage_durable::atomic_write(&art_path, state.snapshot, self.fsync)
                    .map_err(io::Error::other)?;
            }
            Some(art_path)
        } else {
            None
        };

        self.persist_log_plan(&log_plan)?;

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

        cache.hard_digest = Some(digest);
        cache.log = next_log;

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
        let current_log = next_log.map(|layout| self.log_artifact_path(&layout.generation));
        let _ = self.collect_superseded_log_artifacts(current_log.as_deref());
        Ok(())
    }

    fn persist_log_plan(&self, plan: &LogWritePlan) -> io::Result<()> {
        match plan {
            LogWritePlan::Unchanged(_) => Ok(()),
            LogWritePlan::Append {
                previous, bytes, ..
            } => {
                let path = self.log_artifact_path(&previous.generation);
                let file = OpenOptions::new().read(true).write(true).open(&path)?;
                let actual_len = file.metadata()?.len();
                if actual_len < previous.byte_len {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Raft log artifact is shorter than committed hard state",
                    ));
                }
                file.set_len(previous.byte_len)?;
                let mut file = file;
                file.seek(SeekFrom::Start(previous.byte_len))?;
                file.write_all(bytes)?;
                file.flush()?;
                if self.fsync != FsyncPolicy::Os {
                    file.sync_all()?;
                }
                self.log_bytes_appended
                    .fetch_add(bytes.len() as u64, Ordering::Relaxed);
                Ok(())
            }
            LogWritePlan::Rewrite {
                next: Some(next),
                bytes,
            } => {
                let path = self.log_artifact_path(&next.generation);
                storage_durable::atomic_write(&path, bytes, self.fsync)
                    .map_err(io::Error::other)?;
                self.log_bytes_rewritten
                    .fetch_add(bytes.len() as u64, Ordering::Relaxed);
                Ok(())
            }
            LogWritePlan::Rewrite { next: None, .. } => Ok(()),
        }
    }

    /// Load the persisted hard state, or `None` if this node has none yet.
    pub fn load(&self) -> io::Result<Option<PersistedState>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => {
                let state = self.decode_persisted_state(&bytes)?;
                let mut cache = self.cache.lock().expect("raft store cache poisoned");
                if bytes.starts_with(MAGIC_V4) {
                    cache.hard_digest = Some(Sha256::digest(&bytes).into());
                } else {
                    cache.hard_digest = None;
                    cache.log = None;
                }
                Ok(Some(state))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn read_log_artifact(&self, layout: LogLayout) -> io::Result<Vec<RaftEntry>> {
        if layout.entry_count == 0 || layout.byte_len < LOG_MAGIC_V1.len() as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Raft log artifact layout",
            ));
        }
        let path = self.log_artifact_path(&layout.generation);
        let mut file = OpenOptions::new().read(true).write(true).open(&path)?;
        let actual_len = file.metadata()?.len();
        if actual_len < layout.byte_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Raft log artifact is truncated",
            ));
        }
        let declared_len = usize::try_from(layout.byte_len).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Raft log artifact exceeds addressable memory",
            )
        })?;
        let mut bytes = vec![0_u8; declared_len];
        file.read_exact(&mut bytes)?;
        if actual_len > layout.byte_len {
            file.set_len(layout.byte_len)?;
            if self.fsync != FsyncPolicy::Os {
                file.sync_all()?;
            }
        }
        decode_log_artifact(&bytes, layout)
    }

    fn decode_persisted_state(&self, bytes: &[u8]) -> io::Result<PersistedState> {
        if bytes.starts_with(MAGIC_V4) {
            let mut r = CursorReader(&bytes[MAGIC_V4.len()..]);
            let term = r.read_u64()?;
            let voted_for = match r.read_u8()? {
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
            let snapshot = self.read_snapshot_artifact(
                snapshot_index,
                snapshot_term,
                snapshot_len,
                &snapshot_digest,
            )?;
            let conf = match r.read_u8()? {
                0 => None,
                1 => {
                    let (conf, consumed) = ConfState::decode_with_len(r.0).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "invalid conf state")
                    })?;
                    r.0 = &r.0[consumed..];
                    Some(conf)
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid conf tag",
                    ))
                }
            };
            let layout = match r.read_u8()? {
                0 => None,
                1 => Some(LogLayout {
                    generation: r.read_bytes(32)?.try_into().unwrap(),
                    byte_len: r.read_u64()?,
                    entry_count: r.read_u64()?,
                    first_index: r.read_u64()?,
                    last_index: r.read_u64()?,
                    last_term: r.read_u64()?,
                    last_entry_digest: r.read_bytes(32)?.try_into().unwrap(),
                }),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid Raft log artifact tag",
                    ))
                }
            };
            if !r.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "trailing bytes after persisted state",
                ));
            }
            let log = match layout {
                Some(layout) => self.read_log_artifact(layout)?,
                None => Vec::new(),
            };
            self.cache.lock().expect("raft store cache poisoned").log = layout;
            Ok(PersistedState {
                term,
                voted_for,
                log,
                commit_index,
                snapshot_index,
                snapshot_term,
                snapshot,
                conf,
            })
        } else if bytes.starts_with(MAGIC_V3) {
            let mut r = CursorReader(&bytes[MAGIC_V3.len()..]);
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

            let snapshot = self.read_snapshot_artifact(
                snapshot_index,
                snapshot_term,
                snapshot_len,
                &snapshot_digest,
            )?;

            let has_conf = r.read_u8()?;
            let conf = match has_conf {
                0 => None,
                1 => {
                    let (conf, consumed) = ConfState::decode_with_len(r.0).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "invalid conf state")
                    })?;
                    r.0 = &r.0[consumed..];
                    Some(conf)
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid conf tag",
                    ))
                }
            };

            let log_len = r.read_u64()? as usize;
            let mut log = Vec::with_capacity(log_len.min(100_000));
            for _ in 0..log_len {
                let entry_term = r.read_u64()?;
                let entry_index = r.read_u64()?;
                let kind_tag = r.read_u8()?;
                let kind = match kind_tag {
                    0 => EntryKind::Command,
                    1 => EntryKind::Config,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "invalid entry kind tag",
                        ))
                    }
                };
                let command_len = r.read_u64()? as usize;
                let command = r.read_bytes(command_len)?;
                if kind == EntryKind::Config && ConfState::decode(&command).is_none() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid conf state",
                    ));
                }
                log.push(RaftEntry {
                    term: entry_term,
                    index: entry_index,
                    command,
                    kind,
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
                conf,
            })
        } else if bytes.starts_with(MAGIC_V2) {
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

            let snapshot = self.read_snapshot_artifact(
                snapshot_index,
                snapshot_term,
                snapshot_len,
                &snapshot_digest,
            )?;

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
                    kind: EntryKind::Command,
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
                conf: None,
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
                    kind: EntryKind::Command,
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
                conf: None,
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
            conf: None,
        })
    }
}

enum LogWritePlan {
    Unchanged(Option<LogLayout>),
    Append {
        previous: LogLayout,
        next: LogLayout,
        bytes: Vec<u8>,
    },
    Rewrite {
        next: Option<LogLayout>,
        bytes: Vec<u8>,
    },
}

impl LogWritePlan {
    fn layout(&self) -> Option<LogLayout> {
        match self {
            Self::Unchanged(layout) => *layout,
            Self::Append { next, .. } => Some(*next),
            Self::Rewrite { next, .. } => *next,
        }
    }
}

fn plan_log_write(state: &PersistedStateRef<'_>, previous: Option<LogLayout>) -> LogWritePlan {
    let Some(first) = state.log.first() else {
        return if previous.is_some() {
            LogWritePlan::Rewrite {
                next: None,
                bytes: Vec::new(),
            }
        } else {
            LogWritePlan::Unchanged(None)
        };
    };
    let last = state.log.last().expect("non-empty Raft log has a tail");
    let last_digest = entry_digest(last);
    if let Some(previous) = previous {
        if previous.entry_count == state.log.len() as u64
            && previous.first_index == first.index
            && previous.last_index == last.index
            && previous.last_term == last.term
            && previous.last_entry_digest == last_digest
        {
            return LogWritePlan::Unchanged(Some(previous));
        }
        if state.log.len() as u64 > previous.entry_count
            && previous.first_index == first.index
            && previous.entry_count > 0
        {
            let prior_tail = &state.log[previous.entry_count as usize - 1];
            if prior_tail.index == previous.last_index
                && prior_tail.term == previous.last_term
                && entry_digest(prior_tail) == previous.last_entry_digest
            {
                let bytes = encode_log_entries(&state.log[previous.entry_count as usize..], false);
                let next = LogLayout {
                    generation: previous.generation,
                    byte_len: previous.byte_len.saturating_add(bytes.len() as u64),
                    entry_count: state.log.len() as u64,
                    first_index: first.index,
                    last_index: last.index,
                    last_term: last.term,
                    last_entry_digest: last_digest,
                };
                return LogWritePlan::Append {
                    previous,
                    next,
                    bytes,
                };
            }
        }
    }

    let bytes = encode_log_entries(state.log, true);
    let mut generation = Sha256::new();
    generation.update(b"raft-log-generation-v1");
    generation.update(
        previous
            .map(|layout| layout.generation)
            .unwrap_or([0_u8; 32]),
    );
    generation.update(state.snapshot_index.to_le_bytes());
    generation.update(state.snapshot_term.to_le_bytes());
    generation.update(&bytes);
    let next = LogLayout {
        generation: generation.finalize().into(),
        byte_len: bytes.len() as u64,
        entry_count: state.log.len() as u64,
        first_index: first.index,
        last_index: last.index,
        last_term: last.term,
        last_entry_digest: last_digest,
    };
    LogWritePlan::Rewrite {
        next: Some(next),
        bytes,
    }
}

fn encode_log_entries(entries: &[RaftEntry], include_magic: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    if include_magic {
        bytes.extend_from_slice(LOG_MAGIC_V1);
    }
    for entry in entries {
        let payload = encode_log_entry(entry);
        bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&crc32fast::hash(&payload).to_le_bytes());
        bytes.extend_from_slice(&payload);
    }
    bytes
}

fn encode_log_entry(entry: &RaftEntry) -> Vec<u8> {
    let mut payload = Vec::with_capacity(25 + entry.command.len());
    payload.extend_from_slice(&entry.term.to_le_bytes());
    payload.extend_from_slice(&entry.index.to_le_bytes());
    payload.push(match entry.kind {
        EntryKind::Command => 0,
        EntryKind::Config => 1,
    });
    payload.extend_from_slice(&(entry.command.len() as u64).to_le_bytes());
    payload.extend_from_slice(&entry.command);
    payload
}

fn entry_digest(entry: &RaftEntry) -> [u8; 32] {
    Sha256::digest(encode_log_entry(entry)).into()
}

fn decode_log_artifact(bytes: &[u8], layout: LogLayout) -> io::Result<Vec<RaftEntry>> {
    if !bytes.starts_with(LOG_MAGIC_V1) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid Raft log artifact marker",
        ));
    }
    let mut reader = CursorReader(&bytes[LOG_MAGIC_V1.len()..]);
    let mut log = Vec::with_capacity((layout.entry_count as usize).min(100_000));
    for _ in 0..layout.entry_count {
        let payload_len = usize::try_from(reader.read_u64()?).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft log frame length overflow")
        })?;
        let expected_crc = reader.read_u32()?;
        let payload = reader.read_bytes(payload_len)?;
        if crc32fast::hash(&payload) != expected_crc {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Raft log frame checksum mismatch",
            ));
        }
        let mut entry = CursorReader(&payload);
        let term = entry.read_u64()?;
        let index = entry.read_u64()?;
        let kind = match entry.read_u8()? {
            0 => EntryKind::Command,
            1 => EntryKind::Config,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid Raft log entry kind",
                ))
            }
        };
        let command_len = usize::try_from(entry.read_u64()?).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft command length overflow")
        })?;
        let command = entry.read_bytes(command_len)?;
        if !entry.is_empty() || (kind == EntryKind::Config && ConfState::decode(&command).is_none())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Raft log entry payload",
            ));
        }
        log.push(RaftEntry {
            term,
            index,
            command,
            kind,
        });
    }
    if !reader.is_empty()
        || log.first().map(|entry| entry.index) != Some(layout.first_index)
        || log.last().map(|entry| entry.index) != Some(layout.last_index)
        || log.last().map(|entry| entry.term) != Some(layout.last_term)
        || log.last().map(entry_digest) != Some(layout.last_entry_digest)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Raft log artifact disagrees with hard state",
        ));
    }
    Ok(log)
}

fn encode_persisted_state_v4(
    state: &PersistedStateRef<'_>,
    snapshot_digest: &[u8; 32],
    log: Option<LogLayout>,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(256);
    bytes.extend_from_slice(MAGIC_V4);
    bytes.extend_from_slice(&state.term.to_le_bytes());
    match state.voted_for {
        Some(node_id) => {
            bytes.push(1);
            bytes.extend_from_slice(&node_id.to_le_bytes());
        }
        None => bytes.push(0),
    }
    bytes.extend_from_slice(&state.commit_index.to_le_bytes());
    bytes.extend_from_slice(&state.snapshot_index.to_le_bytes());
    bytes.extend_from_slice(&state.snapshot_term.to_le_bytes());
    bytes.extend_from_slice(&(state.snapshot.len() as u64).to_le_bytes());
    bytes.extend_from_slice(snapshot_digest);
    match state.conf {
        Some(conf) => {
            bytes.push(1);
            bytes.extend_from_slice(&ConfState::encode(conf));
        }
        None => bytes.push(0),
    }
    match log {
        Some(log) => {
            bytes.push(1);
            bytes.extend_from_slice(&log.generation);
            bytes.extend_from_slice(&log.byte_len.to_le_bytes());
            bytes.extend_from_slice(&log.entry_count.to_le_bytes());
            bytes.extend_from_slice(&log.first_index.to_le_bytes());
            bytes.extend_from_slice(&log.last_index.to_le_bytes());
            bytes.extend_from_slice(&log.last_term.to_le_bytes());
            bytes.extend_from_slice(&log.last_entry_digest);
        }
        None => bytes.push(0),
    }
    bytes
}

fn hex_digest(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    let mut value = String::with_capacity(64);
    for byte in bytes {
        write!(&mut value, "{byte:02x}").expect("write digest into String");
    }
    value
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

    fn read_u32(&mut self) -> io::Result<u32> {
        if self.0.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected eof reading u32",
            ));
        }
        let (num_bytes, rest) = self.0.split_at(4);
        self.0 = rest;
        Ok(u32::from_le_bytes(num_bytes.try_into().unwrap()))
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
