// CODEGEN-BEGIN
//! Durable storage for a raft node's hard state.
//!
//! Persists [`PersistedState`] (term, votedFor, log, snapshot) to a single file
//! under a data dir, written atomically (temp + rename) and fsynced per
//! [`FsyncPolicy`]. The host calls [`RaftStore::save`] *before* flushing the
//! node's outbox, so no vote or ack is sent before the decision that produced it
//! is durable. (Lifted from lumen/relay's identical `raft_store`.)

use std::collections::BTreeMap;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use memmap2::{Mmap, MmapOptions};
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

#[derive(Clone, Copy, Debug)]
struct CommandFrame {
    index: Index,
    term: Term,
    kind: EntryKind,
    payload_offset: u64,
    payload_len: u64,
    payload_crc: u32,
    command_offset: u64,
    command_len: u64,
}

#[derive(Clone)]
struct PublishedLog {
    layout: LogLayout,
    commands: BTreeMap<(Index, Term), CommandFrame>,
}

#[derive(Default)]
struct StoreCache {
    hard_digest: Option<[u8; 32]>,
    log: Option<PublishedLog>,
    commit_index: Index,
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
    io_serial: Mutex<()>,
    cache: Mutex<StoreCache>,
    pinned_log_generations: Arc<Mutex<BTreeMap<[u8; 32], usize>>>,
    log_bytes_appended: AtomicU64,
    log_bytes_rewritten: AtomicU64,
    injected_save_failure: Mutex<Option<io::ErrorKind>>,
    injected_after_artifact_failure: Mutex<Option<io::ErrorKind>>,
    injected_after_publish_failure: Mutex<Option<io::ErrorKind>>,
    #[cfg(test)]
    load_after_state_read_hook: Mutex<Option<LoadAfterStateReadHook>>,
}

#[cfg(test)]
struct LoadAfterStateReadHook {
    entered: std::sync::mpsc::Sender<()>,
    release: std::sync::mpsc::Receiver<()>,
}

/// A generation-pinned identity for one committed command. The pin is created
/// while a host still holds its Raft-node mutex; a later implementation maps
/// and validates the exact durable frame after that mutex is released.
pub struct CommittedCommandPin {
    generation: GenerationPin,
    frame: CommandFrame,
}

/// Read-only bytes for one durable Raft command.
pub struct CommittedCommandLease {
    map: Mmap,
    command_start: usize,
    command_end: usize,
    _generation: GenerationPin,
}

struct GenerationPin {
    generation: [u8; 32],
    path: PathBuf,
    byte_len: u64,
    pins: Arc<Mutex<BTreeMap<[u8; 32], usize>>>,
}

impl CommittedCommandLease {
    /// The exact command payload from the pinned durable frame.
    pub fn command(&self) -> &[u8] {
        &self.map[self.command_start..self.command_end]
    }
}

impl CommittedCommandPin {
    /// Map and validate the pinned artifact outside the host node mutex.
    pub fn map(self) -> io::Result<CommittedCommandLease> {
        let file = OpenOptions::new().read(true).open(&self.generation.path)?;
        let actual_len = file.metadata()?.len();
        if actual_len < self.generation.byte_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "pinned Raft log artifact is truncated",
            ));
        }
        let mapped_len = usize::try_from(self.generation.byte_len).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Raft log artifact exceeds address space",
            )
        })?;
        let map =
            unsafe { MmapOptions::new().len(mapped_len).map(&file) }.map_err(io::Error::other)?;
        validate_pinned_frame(&map, self.frame)?;
        let command_start = usize::try_from(self.frame.command_offset).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft command offset overflow")
        })?;
        let command_len = usize::try_from(self.frame.command_len).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft command length overflow")
        })?;
        let command_end = command_start.checked_add(command_len).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft command range overflow")
        })?;
        if command_end > map.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "pinned Raft command is truncated",
            ));
        }
        Ok(CommittedCommandLease {
            map,
            command_start,
            command_end,
            _generation: self.generation,
        })
    }
}

impl Drop for GenerationPin {
    fn drop(&mut self) {
        let mut pins = self
            .pins
            .lock()
            .expect("pinned log generations mutex poisoned");
        let count = pins
            .get_mut(&self.generation)
            .expect("generation pin missing from registry");
        *count -= 1;
        if *count == 0 {
            pins.remove(&self.generation);
        }
    }
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
            io_serial: Mutex::new(()),
            cache: Mutex::new(StoreCache::default()),
            pinned_log_generations: Arc::new(Mutex::new(BTreeMap::new())),
            log_bytes_appended: AtomicU64::new(0),
            log_bytes_rewritten: AtomicU64::new(0),
            injected_save_failure: Mutex::new(None),
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
            #[cfg(test)]
            load_after_state_read_hook: Mutex::new(None),
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
            io_serial: Mutex::new(()),
            cache: Mutex::new(StoreCache::default()),
            pinned_log_generations: Arc::new(Mutex::new(BTreeMap::new())),
            log_bytes_appended: AtomicU64::new(0),
            log_bytes_rewritten: AtomicU64::new(0),
            injected_save_failure: Mutex::new(None),
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
            #[cfg(test)]
            load_after_state_read_hook: Mutex::new(None),
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
            io_serial: Mutex::new(()),
            cache: Mutex::new(StoreCache::default()),
            pinned_log_generations: Arc::new(Mutex::new(BTreeMap::new())),
            log_bytes_appended: AtomicU64::new(0),
            log_bytes_rewritten: AtomicU64::new(0),
            injected_save_failure: Mutex::new(None),
            injected_after_artifact_failure: Mutex::new(None),
            injected_after_publish_failure: Mutex::new(None),
            #[cfg(test)]
            load_after_state_read_hook: Mutex::new(None),
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

    #[cfg(test)]
    fn pause_next_load_after_state_read(
        &self,
        entered: std::sync::mpsc::Sender<()>,
        release: std::sync::mpsc::Receiver<()>,
    ) {
        *self
            .load_after_state_read_hook
            .lock()
            .expect("load_after_state_read_hook mutex poisoned") =
            Some(LoadAfterStateReadHook { entered, release });
    }

    #[cfg(test)]
    fn wait_after_load_state_read(&self) {
        let hook = self
            .load_after_state_read_hook
            .lock()
            .expect("load_after_state_read_hook mutex poisoned")
            .take();
        if let Some(hook) = hook {
            let _ = hook.entered.send(());
            let _ = hook.release.recv();
        }
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
                        && !self.log_artifact_is_pinned(&path)
                    {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
        Ok(())
    }

    fn log_artifact_is_pinned(&self, path: &Path) -> bool {
        self.pinned_log_generations
            .lock()
            .expect("pinned log generations mutex poisoned")
            .keys()
            .any(|generation| self.log_artifact_path(generation) == path)
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

    /// Pin the published durable generation for exactly one committed command.
    ///
    /// This copies only frame metadata while the cache lock is held. The file
    /// is opened and mapped later by [`CommittedCommandPin::map`].
    pub fn pin_committed_command(
        &self,
        index: Index,
        term: Term,
    ) -> io::Result<CommittedCommandPin> {
        let cache = self.cache.lock().expect("raft store cache poisoned");
        if index > cache.commit_index {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "requested Raft command is not committed",
            ));
        }
        let published = cache.log.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "no published Raft log artifact",
            )
        })?;
        let frame = *published.commands.get(&(index, term)).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "requested Raft command identity is not published",
            )
        })?;
        if frame.kind != EntryKind::Command {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "requested Raft identity is not a command",
            ));
        }
        let generation = published.layout.generation;
        let mut pins = self
            .pinned_log_generations
            .lock()
            .expect("pinned log generations mutex poisoned");
        *pins.entry(generation).or_default() += 1;
        Ok(CommittedCommandPin {
            generation: GenerationPin {
                generation,
                path: self.log_artifact_path(&generation),
                byte_len: published.layout.byte_len,
                pins: Arc::clone(&self.pinned_log_generations),
            },
            frame,
        })
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
        let _io_serial = self
            .io_serial
            .lock()
            .expect("raft store I/O mutex poisoned");
        let mut cache = self.cache.lock().expect("raft store cache poisoned");
        let log_plan = plan_log_write(state, cache.log.as_ref().map(|log| log.layout));
        let next_log = log_plan.layout();
        let frame_update = command_frame_update(&log_plan)?;
        validate_command_frame_update(&cache, next_log, &frame_update)?;
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

        apply_command_frame_update(&mut cache, next_log, frame_update);
        cache.hard_digest = Some(digest);
        cache.commit_index = state.commit_index;

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
        let _io_serial = self
            .io_serial
            .lock()
            .expect("raft store I/O mutex poisoned");
        match std::fs::read(&self.path) {
            Ok(bytes) => {
                #[cfg(test)]
                self.wait_after_load_state_read();
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

    fn read_log_artifact(
        &self,
        layout: LogLayout,
    ) -> io::Result<(Vec<RaftEntry>, BTreeMap<(Index, Term), CommandFrame>)> {
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
        let log = decode_log_artifact(&bytes, layout)?;
        let commands = command_frames_from_artifact(&bytes, layout)?;
        Ok((log, commands))
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
            let (log, commands) = match layout {
                Some(layout) => self.read_log_artifact(layout)?,
                None => (Vec::new(), BTreeMap::new()),
            };
            let mut cache = self.cache.lock().expect("raft store cache poisoned");
            cache.log = layout.map(|layout| PublishedLog { layout, commands });
            cache.commit_index = commit_index;
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

enum CommandFrameUpdate {
    Unchanged,
    Append(Vec<CommandFrame>),
    Rewrite(BTreeMap<(Index, Term), CommandFrame>),
    Clear,
}

fn command_frame_update(plan: &LogWritePlan) -> io::Result<CommandFrameUpdate> {
    match plan {
        LogWritePlan::Unchanged(_) => Ok(CommandFrameUpdate::Unchanged),
        LogWritePlan::Append {
            previous,
            next,
            bytes,
        } => Ok(CommandFrameUpdate::Append(
            command_frames_from_encoded(
                bytes,
                next.entry_count
                    .checked_sub(previous.entry_count)
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Raft append entry count underflow",
                        )
                    })?,
                previous.byte_len,
                false,
            )?
            .into_values()
            .collect(),
        )),
        LogWritePlan::Rewrite {
            next: Some(next),
            bytes,
        } => Ok(CommandFrameUpdate::Rewrite(command_frames_from_encoded(
            bytes,
            next.entry_count,
            LOG_MAGIC_V1.len() as u64,
            true,
        )?)),
        LogWritePlan::Rewrite { next: None, .. } => Ok(CommandFrameUpdate::Clear),
    }
}

fn apply_command_frame_update(
    cache: &mut StoreCache,
    next_log: Option<LogLayout>,
    update: CommandFrameUpdate,
) {
    match update {
        CommandFrameUpdate::Unchanged => {
            debug_assert_eq!(cache.log.as_ref().map(|log| log.layout), next_log);
        }
        CommandFrameUpdate::Append(frames) => {
            let published = cache
                .log
                .as_mut()
                .expect("validated append has published Raft command metadata");
            for frame in frames {
                let previous = published
                    .commands
                    .insert((frame.index, frame.term), frame)
                    .is_some();
                debug_assert!(
                    !previous,
                    "validated append has distinct command identities"
                );
            }
            published.layout = next_log.expect("validated append has next Raft log layout");
        }
        CommandFrameUpdate::Rewrite(commands) => {
            cache.log = Some(PublishedLog {
                layout: next_log.expect("validated rewrite has next Raft log layout"),
                commands,
            });
        }
        CommandFrameUpdate::Clear => cache.log = None,
    }
}

fn validate_command_frame_update(
    cache: &StoreCache,
    next_log: Option<LogLayout>,
    update: &CommandFrameUpdate,
) -> io::Result<()> {
    match update {
        CommandFrameUpdate::Unchanged => {
            if cache.log.as_ref().map(|log| log.layout) != next_log {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unchanged Raft log metadata disagrees with cache",
                ));
            }
        }
        CommandFrameUpdate::Append(frames) => {
            let published = cache.log.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "append has no published Raft command metadata",
                )
            })?;
            if next_log.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "append has no next Raft log layout",
                ));
            }
            if frames
                .iter()
                .any(|frame| published.commands.contains_key(&(frame.index, frame.term)))
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "duplicate Raft command identity in log",
                ));
            }
        }
        CommandFrameUpdate::Rewrite(_) if next_log.is_none() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "rewrite has no next Raft log layout",
            ));
        }
        CommandFrameUpdate::Rewrite(_) | CommandFrameUpdate::Clear => {}
    }
    Ok(())
}

fn command_frames_from_artifact(
    bytes: &[u8],
    layout: LogLayout,
) -> io::Result<BTreeMap<(Index, Term), CommandFrame>> {
    command_frames_from_encoded(bytes, layout.entry_count, LOG_MAGIC_V1.len() as u64, true)
}

fn command_frames_from_encoded(
    bytes: &[u8],
    entry_count: u64,
    mut frame_offset: u64,
    include_magic: bool,
) -> io::Result<BTreeMap<(Index, Term), CommandFrame>> {
    let encoded = if include_magic {
        if !bytes.starts_with(LOG_MAGIC_V1) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Raft log artifact marker",
            ));
        }
        &bytes[LOG_MAGIC_V1.len()..]
    } else {
        bytes
    };
    if include_magic && frame_offset != LOG_MAGIC_V1.len() as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid Raft log frame base offset",
        ));
    }
    let mut reader = CursorReader(encoded);
    let mut commands = BTreeMap::new();
    for _ in 0..entry_count {
        let payload_len = usize::try_from(reader.read_u64()?).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft log frame length overflow")
        })?;
        let payload_crc = reader.read_u32()?;
        let payload = reader.read_slice(payload_len)?;
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
        let command_len = entry.read_u64()?;
        let command_len_usize = usize::try_from(command_len).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft command length overflow")
        })?;
        entry.skip_bytes(command_len_usize)?;
        if !entry.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Raft log entry payload",
            ));
        }
        let payload_len_u64 = payload_len as u64;
        let payload_offset = frame_offset.checked_add(12).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft frame offset overflow")
        })?;
        let command_offset = payload_offset.checked_add(25).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Raft command offset overflow")
        })?;
        let frame = CommandFrame {
            index,
            term,
            kind,
            payload_offset,
            payload_len: payload_len_u64,
            payload_crc,
            command_offset,
            command_len,
        };
        if commands.insert((index, term), frame).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "duplicate Raft command identity in log",
            ));
        }
        frame_offset = frame_offset
            .checked_add(12)
            .and_then(|offset| offset.checked_add(payload_len_u64))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Raft frame offset overflow")
            })?;
    }
    if !reader.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Raft log artifact disagrees with hard state",
        ));
    }
    Ok(commands)
}

fn validate_pinned_frame(map: &[u8], frame: CommandFrame) -> io::Result<()> {
    if !map.starts_with(LOG_MAGIC_V1) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid Raft log artifact marker",
        ));
    }
    let payload_start = usize::try_from(frame.payload_offset)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Raft payload offset overflow"))?;
    let payload_len = usize::try_from(frame.payload_len)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Raft payload length overflow"))?;
    let payload_end = payload_start
        .checked_add(payload_len)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Raft payload range overflow"))?;
    let payload = map.get(payload_start..payload_end).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "pinned Raft frame is truncated")
    })?;
    let frame_start = payload_start.checked_sub(12).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Raft frame header offset underflow",
        )
    })?;
    let header = map.get(frame_start..payload_start).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "pinned Raft frame header is truncated",
        )
    })?;
    let encoded_len = u64::from_le_bytes(header[..8].try_into().unwrap());
    let encoded_crc = u32::from_le_bytes(header[8..].try_into().unwrap());
    if encoded_len != frame.payload_len || encoded_crc != frame.payload_crc {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pinned Raft frame header disagrees with metadata",
        ));
    }
    if crc32fast::hash(payload) != frame.payload_crc {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pinned Raft frame checksum mismatch",
        ));
    }
    let mut entry = CursorReader(payload);
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
    let command_len = entry.read_u64()?;
    let command_len_usize = usize::try_from(command_len)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Raft command length overflow"))?;
    if index != frame.index
        || term != frame.term
        || kind != frame.kind
        || command_len != frame.command_len
        || entry.0.len() != command_len_usize
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pinned Raft frame identity disagrees with metadata",
        ));
    }
    let expected_command_offset = frame.payload_offset.checked_add(25).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Raft command offset overflow")
    })?;
    if frame.command_offset != expected_command_offset {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pinned Raft command range disagrees with metadata",
        ));
    }
    Ok(())
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
    fn read_slice(&mut self, len: usize) -> io::Result<&'a [u8]> {
        if self.0.len() < len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected eof reading byte payload",
            ));
        }
        let (data, rest) = self.0.split_at(len);
        self.0 = rest;
        Ok(data)
    }

    fn skip_bytes(&mut self, len: usize) -> io::Result<()> {
        self.read_slice(len).map(|_| ())
    }

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
        Ok(self.read_slice(len)?.to_vec())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod command_lease_tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    fn state(entries: Vec<(Index, Term, Vec<u8>)>) -> PersistedState {
        PersistedState {
            term: entries.last().map(|(_, term, _)| *term).unwrap_or(0),
            voted_for: None,
            commit_index: entries.last().map(|(index, _, _)| *index).unwrap_or(0),
            snapshot_index: 0,
            snapshot_term: 0,
            snapshot: Vec::new(),
            conf: None,
            log: entries
                .into_iter()
                .map(|(index, term, command)| RaftEntry {
                    index,
                    term,
                    command,
                    kind: EntryKind::Command,
                })
                .collect(),
        }
    }

    fn store(dir: &tempfile::TempDir) -> RaftStore {
        RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap()
    }

    #[test]
    fn command_pin_maps_published_frame_after_append() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        store.save(&state(vec![(1, 1, b"first".to_vec())])).unwrap();
        let lease = store.pin_committed_command(1, 1).unwrap().map().unwrap();
        store
            .save(&state(vec![
                (1, 1, b"first".to_vec()),
                (2, 1, b"second".to_vec()),
            ]))
            .unwrap();

        assert_eq!(lease.command(), b"first");
        assert_eq!(
            store
                .pin_committed_command(2, 1)
                .unwrap()
                .map()
                .unwrap()
                .command(),
            b"second"
        );
    }

    #[test]
    fn command_pin_survives_rewrite_and_superseded_generation_collection() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        store.save(&state(vec![(1, 1, b"old".to_vec())])).unwrap();
        let old_generation = store
            .cache
            .lock()
            .unwrap()
            .log
            .as_ref()
            .unwrap()
            .layout
            .generation;
        let pin = store.pin_committed_command(1, 1).unwrap();

        store.save(&state(vec![(2, 2, b"new".to_vec())])).unwrap();

        let lease = pin.map().unwrap();
        assert_eq!(lease.command(), b"old");
        assert!(store.log_artifact_path(&old_generation).exists());
        drop(lease);
        store.save(&state(vec![(3, 3, b"newer".to_vec())])).unwrap();
        assert!(!store.log_artifact_path(&old_generation).exists());
    }

    #[test]
    fn reopened_store_rebuilds_published_command_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let first = store(&dir);
        first
            .save(&state(vec![(1, 7, b"reopen".to_vec())]))
            .unwrap();
        drop(first);

        let reopened = store(&dir);
        reopened.load().unwrap();
        let lease = reopened.pin_committed_command(1, 7).unwrap().map().unwrap();
        assert_eq!(lease.command(), b"reopen");
    }

    #[test]
    fn command_pin_refuses_wrong_identity_and_unpublished_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        store
            .save(&state(vec![(1, 3, b"published".to_vec())]))
            .unwrap();
        let published = store.pin_committed_command(1, 3).unwrap();
        store.inject_next_save_failure_with_kind(io::ErrorKind::Other);
        assert!(store
            .save(&state(vec![(2, 4, b"unpublished".to_vec())]))
            .is_err());

        assert_eq!(published.map().unwrap().command(), b"published");

        let wrong_term = match store.pin_committed_command(1, 4) {
            Ok(_) => panic!("wrong term must not pin a published command"),
            Err(error) => error,
        };
        assert_eq!(wrong_term.kind(), io::ErrorKind::InvalidInput);
        let unpublished = match store.pin_committed_command(2, 4) {
            Ok(_) => panic!("unpublished command must not pin"),
            Err(error) => error,
        };
        assert_eq!(unpublished.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn command_pin_refuses_published_but_uncommitted_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        let mut durable = state(vec![(1, 3, b"uncommitted".to_vec())]);
        durable.commit_index = 0;
        store.save(&durable).unwrap();

        let error = match store.pin_committed_command(1, 3) {
            Ok(_) => panic!("uncommitted suffix must not pin"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn load_cannot_truncate_concurrently_appended_published_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(store(&dir));
        store.save(&state(vec![(1, 1, b"first".to_vec())])).unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        store.pause_next_load_after_state_read(entered_tx, release_rx);
        let loading = {
            let store = Arc::clone(&store);
            thread::spawn(move || store.load())
        };
        let entered = entered_rx.recv_timeout(Duration::from_secs(1));
        if entered.is_err() {
            drop(release_tx);
            let _ = loading.join();
            panic!("load did not reach the post-read pause");
        }

        let (saved_tx, saved_rx) = mpsc::channel();
        let saving = {
            let store = Arc::clone(&store);
            thread::spawn(move || {
                let result = store.save(&state(vec![
                    (1, 1, b"first".to_vec()),
                    (2, 1, b"second".to_vec()),
                ]));
                let _ = saved_tx.send(result);
            })
        };

        // A correct I/O mutex may keep this pending. Always release before join.
        let saved_before_release = saved_rx.recv_timeout(Duration::from_secs(1)).ok();
        let _ = release_tx.send(());
        let loaded = loading.join();
        let saving_join = saving.join();
        let saved =
            saved_before_release.or_else(|| saved_rx.recv_timeout(Duration::from_secs(1)).ok());
        assert!(loaded.is_ok(), "load worker panicked");
        assert!(saving_join.is_ok(), "save worker panicked");
        assert_eq!(loaded.unwrap().unwrap().unwrap().log[0].command, b"first");
        saved.expect("save did not finish after release").unwrap();

        let reopened =
            RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap();
        assert_eq!(reopened.load().unwrap().unwrap().log[1].command, b"second");
        assert_eq!(
            reopened
                .pin_committed_command(2, 1)
                .unwrap()
                .map()
                .unwrap()
                .command(),
            b"second"
        );
    }

    #[test]
    fn duplicate_append_identity_refuses_before_hard_state_publication() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        store.save(&state(vec![(1, 1, b"first".to_vec())])).unwrap();
        let hard_before = std::fs::read(store.path()).unwrap();
        let pin = store.pin_committed_command(1, 1).unwrap();

        let error = store
            .save(&state(vec![
                (1, 1, b"first".to_vec()),
                (1, 1, b"repeated".to_vec()),
            ]))
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(store.path()).unwrap(), hard_before);
        assert_eq!(pin.map().unwrap().command(), b"first");
    }

    #[test]
    fn command_pin_map_refuses_missing_published_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        store.save(&state(vec![(1, 1, b"frame".to_vec())])).unwrap();
        let pin = store.pin_committed_command(1, 1).unwrap();
        let generation = store
            .cache
            .lock()
            .unwrap()
            .log
            .as_ref()
            .unwrap()
            .layout
            .generation;
        std::fs::remove_file(store.log_artifact_path(&generation)).unwrap();

        let error = match pin.map() {
            Ok(_) => panic!("missing artifact must not map"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn command_pin_map_refuses_crc_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        store.save(&state(vec![(1, 1, b"frame".to_vec())])).unwrap();
        let pin = store.pin_committed_command(1, 1).unwrap();
        let generation = store
            .cache
            .lock()
            .unwrap()
            .log
            .as_ref()
            .unwrap()
            .layout
            .generation;
        let path = store.log_artifact_path(&generation);
        let mut artifact = OpenOptions::new().write(true).open(path).unwrap();
        artifact.seek(SeekFrom::Start(8 + 12 + 25)).unwrap();
        artifact.write_all(b"X").unwrap();
        artifact.flush().unwrap();

        let error = match pin.map() {
            Ok(_) => panic!("corrupt artifact must not map"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn command_pin_map_refuses_truncated_published_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        store.save(&state(vec![(1, 1, b"frame".to_vec())])).unwrap();
        let pin = store.pin_committed_command(1, 1).unwrap();
        let generation = store
            .cache
            .lock()
            .unwrap()
            .log
            .as_ref()
            .unwrap()
            .layout
            .generation;
        let artifact = OpenOptions::new()
            .write(true)
            .open(store.log_artifact_path(&generation))
            .unwrap();
        artifact.set_len(8).unwrap();

        let error = match pin.map() {
            Ok(_) => panic!("truncated artifact must not map"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
// CODEGEN-END
