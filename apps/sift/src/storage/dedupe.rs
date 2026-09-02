//! Bounded, rebuildable event-id index.
//!
//! Sift guarantees exact idempotency for six hours after acknowledgement.
//! The canonical WAL and committed segments own telemetry durability. This
//! index keeps only the active acknowledgement generations and can be rebuilt.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::StoredEvent;

const SHARD_COUNT: usize = 4096;
const ENTRY_BYTES: usize = 48;
const GENERATION_SECONDS: i64 = 60 * 60;
pub const IDEMPOTENCY_WINDOW_SECONDS: i64 = 6 * 60 * 60;
const MIN_BLOOM_BYTES: usize = 1024 * 1024;
const MAX_BLOOM_BYTES: usize = 64 * 1024 * 1024;
const BLOOM_BITS_PER_ENTRY: u64 = 12;
const META_FILE: &str = "meta.json";
const RECEIPT_LOG_FILE: &str = "receipts.framed";
const FORMAT_VERSION: u32 = 6;
const REPLACE_FORMAT_VERSION: u32 = 1;
const REPLACE_MARKER_FILE: &str = ".dedupe-replace.json";
const REPLACE_STAGE_DIR: &str = ".dedupe-replace-stage";
const REPLACE_BACKUP_DIR: &str = ".dedupe-replace-backup";
const BACKGROUND_FLUSH_ENTRIES: usize = 100_000;
const MAX_PENDING_ENTRIES: usize = BACKGROUND_FLUSH_ENTRIES * 2;

type DedupeRecord = ([u8; 32], u64, i64);
type GroupedDedupeRecords = BTreeMap<i64, BTreeMap<usize, Vec<DedupeRecord>>>;

pub struct DedupeIndex {
    root: PathBuf,
    state: RwLock<DedupeState>,
    maintenance_gate: Mutex<()>,
}

#[derive(Default)]
struct DedupeState {
    generations: BTreeMap<i64, GenerationState>,
    indexed_through_cursor: u64,
    content_digest: [u8; 32],
    rebuild_required: bool,
    applied_time_unix_nano: Option<i64>,
}

struct GenerationState {
    sealed: bool,
    blooms: Vec<BloomLayer>,
    entry_count: u64,
    newest_cursor: u64,
    newest_acknowledged_at_unix_nano: Option<i64>,
    content_digest: [u8; 32],
    receipt_writer: Option<storage_durable::FramedLogWriter>,
    pending: BTreeMap<[u8; 32], (u64, i64)>,
}

struct BloomLayer {
    bits: Vec<u8>,
    entries: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DedupeStats {
    pub entry_count: u64,
    pub newest_cursor: u64,
    pub indexed_through_cursor: u64,
    pub oldest_generation: i64,
    pub newest_generation: i64,
    pub window_seconds: u64,
    pub rebuild_required: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DedupeReceipt {
    pub project: String,
    pub event_id: String,
    pub cursor: u64,
    pub acknowledged_at_unix_nano: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct DedupeMeta {
    format_version: u32,
    indexed_through_cursor: u64,
    content_sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReplaceIntent {
    format_version: u32,
    indexed_through_cursor: u64,
    content_sha256: String,
}

impl DedupeIndex {
    pub fn open(root: impl AsRef<Path>) -> Result<(Self, DedupeStats)> {
        Self::open_at(root, Utc::now())
    }

    #[doc(hidden)]
    pub fn open_at(root: impl AsRef<Path>, now: DateTime<Utc>) -> Result<(Self, DedupeStats)> {
        let root = root.as_ref().join("indexes").join("dedupe");
        reconcile_replace(&root, now)?;
        fs::create_dir_all(&root)
            .with_context(|| format!("create dedupe index root {}", root.display()))?;
        set_private_dir(&root)?;
        let state = match load_state(&root, generation_for(now)) {
            Ok(state) => state,
            Err(error) => {
                tracing::warn!(%error, "rebuilding damaged Sift dedupe index");
                clear_root(&root)?;
                DedupeState {
                    rebuild_required: true,
                    ..DedupeState::default()
                }
            }
        };
        let index = Self {
            root,
            state: RwLock::new(state),
            maintenance_gate: Mutex::new(()),
        };
        // Do not prune from the process wall clock while opening. A voter can
        // restart hours after it last applied Raft. It must first replay every
        // missed command in log order, using the decision time carried by that
        // command. `append_batch_at` advances the window after each apply.
        // Pruning here could remove the row needed to reproduce the leader's
        // earlier duplicate decision.
        let stats = index.stats_at(now)?;
        Ok((index, stats))
    }

    pub fn lookup(&self, project: &str, event_id: &str) -> Result<Option<u64>> {
        self.lookup_at(project, event_id, Utc::now())
    }

    #[doc(hidden)]
    pub fn lookup_at(
        &self,
        project: &str,
        event_id: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<u64>> {
        Ok(self
            .lookup_record_at(project, event_id, now)?
            .map(|(cursor, _)| cursor))
    }

    pub(crate) fn lookup_record_at(
        &self,
        project: &str,
        event_id: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<(u64, i64)>> {
        let digest = event_digest(project, event_id);
        self.lookup_digest_record_at(&digest, now)
    }

    fn lookup_digest_record_at(
        &self,
        digest: &[u8; 32],
        now: DateTime<Utc>,
    ) -> Result<Option<(u64, i64)>> {
        let oldest = oldest_generation(now);
        let cutoff_nanos = (now - chrono::Duration::seconds(IDEMPOTENCY_WINDOW_SECONDS))
            .timestamp_nanos_opt()
            .context("dedupe lookup time is outside the nanosecond range")?;
        let state = self.state.read().expect("dedupe state lock poisoned");
        for (generation, index) in state.generations.range(oldest..).rev() {
            if !index
                .blooms
                .iter()
                .any(|layer| bloom_contains(&layer.bits, digest))
            {
                continue;
            }
            if let Some((cursor, acknowledged_at)) = index.pending.get(digest).copied() {
                if acknowledged_at >= cutoff_nanos {
                    return Ok(Some((cursor, acknowledged_at)));
                }
            }
            let path = shard_path(&self.root, *generation, digest);
            if let Some((cursor, acknowledged_at)) = lookup_file(&path, digest, index.sealed)? {
                if acknowledged_at >= cutoff_nanos {
                    return Ok(Some((cursor, acknowledged_at)));
                }
            }
        }
        Ok(None)
    }

    pub fn append_batch(&self, events: &[StoredEvent]) -> Result<()> {
        self.append_batch_at(events, Utc::now())
    }

    /// Check the rebuildable receipt projection before the canonical WAL is
    /// mutated. This check does not fsync. The canonical signal WAL remains
    /// the only acknowledgement durability boundary.
    pub(crate) fn preflight_append_at(
        &self,
        now: DateTime<Utc>,
        incoming_entries: usize,
    ) -> Result<()> {
        let state = self.state.read().expect("dedupe state lock poisoned");
        if state.rebuild_required {
            bail!("dedupe index requires a rebuild before ingest can continue");
        }
        let pending = pending_entry_count(&state);
        if pending.saturating_add(incoming_entries) > MAX_PENDING_ENTRIES {
            bail!(
                "dedupe shard projection is behind ({pending} pending entries); retry after maintenance"
            );
        }
        drop(state);
        let root = fs::symlink_metadata(&self.root)
            .with_context(|| format!("inspect dedupe index root {}", self.root.display()))?;
        if root.file_type().is_symlink() || !root.is_dir() {
            bail!(
                "dedupe index root {} is not a real directory",
                self.root.display()
            );
        }
        let directory = generation_path(&self.root, generation_for(now));
        match fs::symlink_metadata(&directory) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                bail!(
                    "dedupe generation {} is not a real directory",
                    directory.display()
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&directory)
                    .with_context(|| format!("create dedupe generation {}", directory.display()))?;
                set_private_dir(&directory)?;
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("inspect dedupe generation {}", directory.display()));
            }
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn append_batch_at(&self, events: &[StoredEvent], now: DateTime<Utc>) -> Result<()> {
        self.advance_window_at(now)?;
        let mut page = BTreeSet::new();
        let mut entries = GroupedDedupeRecords::new();
        for event in events {
            let generation = acknowledged_generation(event)?;
            if !event_is_active_at(event, now)? {
                continue;
            }
            let digest = event_digest(&event.event.project, &event.event.event_id);
            if !page.insert(digest)
                || self
                    .lookup_at(&event.event.project, &event.event.event_id, now)?
                    .is_some()
            {
                bail!(
                    "active idempotency window contains duplicate event_id {}",
                    event.event.event_id
                );
            }
            entries
                .entry(generation)
                .or_default()
                .entry(shard_for(&digest))
                .or_default()
                .push((digest, event.cursor, acknowledged_nanos(event)?));
        }
        let indexed_through_cursor = events
            .iter()
            .map(|event| event.cursor)
            .max()
            .unwrap_or_default();
        self.append_grouped_records(entries, indexed_through_cursor, now)
    }

    pub(crate) fn append_receipts_at(
        &self,
        receipts: &[DedupeReceipt],
        indexed_through_cursor: u64,
        now: DateTime<Utc>,
    ) -> Result<()> {
        self.advance_window_at(now)?;
        let cutoff_nanos = (now - chrono::Duration::seconds(IDEMPOTENCY_WINDOW_SECONDS))
            .timestamp_nanos_opt()
            .context("dedupe receipt cutoff is outside the nanosecond range")?;
        let mut page = BTreeSet::new();
        let mut entries = GroupedDedupeRecords::new();
        for receipt in receipts {
            if receipt.project.is_empty()
                || receipt.event_id.is_empty()
                || receipt.cursor == 0
                || receipt.acknowledged_at_unix_nano < cutoff_nanos
            {
                continue;
            }
            let digest = event_digest(&receipt.project, &receipt.event_id);
            if !page.insert(digest) {
                continue;
            }
            match self.lookup_digest_record_at(&digest, now)? {
                Some((cursor, acknowledged_at))
                    if cursor == receipt.cursor
                        && acknowledged_at == receipt.acknowledged_at_unix_nano =>
                {
                    continue;
                }
                Some((cursor, _)) => bail!(
                    "active idempotency receipt {} conflicts at cursors {cursor} and {}",
                    receipt.event_id,
                    receipt.cursor
                ),
                None => {}
            }
            let generation = receipt
                .acknowledged_at_unix_nano
                .div_euclid(1_000_000_000)
                .div_euclid(GENERATION_SECONDS);
            entries
                .entry(generation)
                .or_default()
                .entry(shard_for(&digest))
                .or_default()
                .push((digest, receipt.cursor, receipt.acknowledged_at_unix_nano));
        }
        if entries.is_empty() {
            return Ok(());
        }
        self.append_grouped_records(entries, indexed_through_cursor, now)
    }

    fn append_grouped_records(
        &self,
        entries: GroupedDedupeRecords,
        indexed_through_cursor: u64,
        now: DateTime<Utc>,
    ) -> Result<()> {
        let current = generation_for(now);
        let mut state = self.state.write().expect("dedupe state lock poisoned");
        let mut batch_content_digest = [0_u8; 32];
        for (generation, shards) in entries {
            let directory = generation_path(&self.root, generation);
            fs::create_dir_all(&directory)?;
            set_private_dir(&directory)?;
            let index = state
                .generations
                .entry(generation)
                .or_insert_with(|| GenerationState::empty(generation < current));
            if index.receipt_writer.is_none() {
                index.receipt_writer = Some(storage_durable::FramedLogWriter::open(
                    directory.join(RECEIPT_LOG_FILE),
                    storage_durable::FsyncPolicy::Os,
                )?);
                set_private_file(&directory.join(RECEIPT_LOG_FILE))?;
            }
            let receipt_records = shards
                .values()
                .flat_map(|entries| entries.iter().copied())
                .collect::<Vec<_>>();
            let receipt_cursor = receipt_records
                .iter()
                .map(|(_, cursor, _)| *cursor)
                .max()
                .context("dedupe receipt batch has no records")?;
            let receipt_payload = encode_receipt_records(&receipt_records);
            index
                .receipt_writer
                .as_mut()
                .expect("receipt writer initialized")
                .append(receipt_cursor, &receipt_payload)
                .context("append one rebuildable dedupe receipt batch")?;
            index.sealed = false;
            for (shard, shard_entries) in shards {
                for entry @ (digest, cursor, acknowledged_at) in shard_entries {
                    if index
                        .pending
                        .insert(digest, (cursor, acknowledged_at))
                        .is_some()
                    {
                        bail!("dedupe receipt batch repeated one event digest");
                    }
                    bloom_insert_scalable(&mut index.blooms, &digest);
                    index.entry_count = index.entry_count.saturating_add(1);
                    index.newest_cursor = index.newest_cursor.max(cursor);
                    index.newest_acknowledged_at_unix_nano = Some(
                        index
                            .newest_acknowledged_at_unix_nano
                            .unwrap_or(i64::MIN)
                            .max(acknowledged_at),
                    );
                    let digest = dedupe_record_digest(generation, shard, &entry);
                    xor_digest_in_place(&mut index.content_digest, digest);
                    xor_digest_in_place(&mut batch_content_digest, digest);
                }
            }
        }
        xor_digest_in_place(&mut state.content_digest, batch_content_digest);
        state.indexed_through_cursor = state.indexed_through_cursor.max(indexed_through_cursor);
        state.applied_time_unix_nano = Some(
            state.applied_time_unix_nano.unwrap_or(i64::MIN).max(
                now.timestamp_nanos_opt()
                    .context("dedupe applied time is outside the nanosecond range")?,
            ),
        );
        persist_meta_with_policy(
            &self.root,
            state.indexed_through_cursor,
            state.content_digest,
            storage_durable::FsyncPolicy::Os,
        )?;
        Ok(())
    }

    pub(crate) fn covers(&self, event: &StoredEvent, now: DateTime<Utc>) -> Result<bool> {
        event_is_active_at(event, now)
    }

    pub(crate) fn stats(&self) -> Result<DedupeStats> {
        self.stats_at(Utc::now())
    }

    #[doc(hidden)]
    pub fn stats_at(&self, now: DateTime<Utc>) -> Result<DedupeStats> {
        let oldest = oldest_generation(now);
        let state = self.state.read().expect("dedupe state lock poisoned");
        Ok(stats_from_state(&state, oldest))
    }

    pub fn reset(&self) -> Result<()> {
        let mut state = self.state.write().expect("dedupe state lock poisoned");
        clear_root(&self.root)?;
        *state = DedupeState {
            rebuild_required: true,
            ..DedupeState::default()
        };
        Ok(())
    }

    pub(crate) fn mark_rebuilt_through(&self, cursor: u64) -> Result<()> {
        let mut state = self.state.write().expect("dedupe state lock poisoned");
        state.indexed_through_cursor = cursor;
        state.rebuild_required = false;
        persist_meta(&self.root, cursor, state.content_digest)
    }

    #[doc(hidden)]
    pub fn advance_window_at(&self, _now: DateTime<Utc>) -> Result<()> {
        // Expiration and shard sealing can touch thousands of files. The
        // projection worker owns that rebuildable work. The ingest path only
        // appends one receipt frame after the canonical signal WAL is durable.
        Ok(())
    }

    pub(crate) fn maintain_at(&self, now: DateTime<Utc>, force: bool) -> Result<usize> {
        let _maintenance = self
            .maintenance_gate
            .lock()
            .expect("dedupe maintenance lock poisoned");
        let current = generation_for(now);
        let oldest = oldest_generation(now);

        let expired = {
            let mut state = self.state.write().expect("dedupe state lock poisoned");
            let generations = state
                .generations
                .keys()
                .copied()
                .filter(|generation| *generation < oldest)
                .collect::<Vec<_>>();
            let mut expired = Vec::with_capacity(generations.len());
            for generation in generations {
                if let Some(index) = state.generations.remove(&generation) {
                    xor_digest_in_place(&mut state.content_digest, index.content_digest);
                    expired.push(generation);
                }
            }
            expired
        };
        for generation in &expired {
            remove_generation_dir(&generation_path(&self.root, *generation))?;
        }

        let total_pending = {
            let state = self.state.read().expect("dedupe state lock poisoned");
            pending_entry_count(&state)
        };
        let flush_current = force || total_pending >= BACKGROUND_FLUSH_ENTRIES;
        let generations = {
            let state = self.state.read().expect("dedupe state lock poisoned");
            state
                .generations
                .iter()
                .filter_map(|(generation, index)| {
                    (!index.pending.is_empty() && (*generation < current || flush_current))
                        .then_some(*generation)
                })
                .collect::<Vec<_>>()
        };

        let mut flushed = 0_usize;
        for generation in generations {
            let pending = {
                let state = self.state.read().expect("dedupe state lock poisoned");
                state
                    .generations
                    .get(&generation)
                    .map(|index| index.pending.clone())
                    .unwrap_or_default()
            };
            let mut shards = BTreeMap::<usize, Vec<DedupeRecord>>::new();
            for (digest, (cursor, acknowledged_at)) in &pending {
                shards.entry(shard_for(digest)).or_default().push((
                    *digest,
                    *cursor,
                    *acknowledged_at,
                ));
            }
            for (shard, records) in shards {
                let mut state = self.state.write().expect("dedupe state lock poisoned");
                let Some(index) = state.generations.get_mut(&generation) else {
                    continue;
                };
                if let Err(error) = append_entries(
                    &generation_path(&self.root, generation).join(format!("{shard:03x}.idx")),
                    &records,
                ) {
                    state.rebuild_required = true;
                    return Err(error).context("flush rebuildable dedupe shard projection");
                }
                for (digest, cursor, acknowledged_at) in records {
                    if index.pending.get(&digest) == Some(&(cursor, acknowledged_at)) {
                        index.pending.remove(&digest);
                        flushed = flushed.saturating_add(1);
                    }
                }
            }
        }

        let to_seal = {
            let mut state = self.state.write().expect("dedupe state lock poisoned");
            state
                .generations
                .iter_mut()
                .filter_map(|(generation, index)| {
                    if *generation < current && index.pending.is_empty() && !index.sealed {
                        index.receipt_writer.take();
                        Some(*generation)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        for generation in to_seal {
            if let Err(error) = seal_generation(&generation_path(&self.root, generation)) {
                self.state
                    .write()
                    .expect("dedupe state lock poisoned")
                    .rebuild_required = true;
                return Err(error).context("seal rebuildable dedupe generation");
            }
            let mut state = self.state.write().expect("dedupe state lock poisoned");
            if let Some(index) = state.generations.get_mut(&generation) {
                if index.pending.is_empty() {
                    index.sealed = true;
                }
            }
        }

        if !expired.is_empty() || flushed > 0 {
            let state = self.state.read().expect("dedupe state lock poisoned");
            if !state.rebuild_required {
                persist_meta(
                    &self.root,
                    state.indexed_through_cursor,
                    state.content_digest,
                )?;
            }
        }
        Ok(flushed)
    }

    pub(crate) fn maintain_applied(&self, force: bool) -> Result<usize> {
        let applied = self
            .state
            .read()
            .expect("dedupe state lock poisoned")
            .applied_time_unix_nano;
        let Some(applied) = applied else {
            return Ok(0);
        };
        self.maintain_at(DateTime::<Utc>::from_timestamp_nanos(applied), force)
    }

    #[doc(hidden)]
    pub fn maintain_for_applied_time_for_diagnostics(&self, force: bool) -> Result<usize> {
        self.maintain_applied(force)
    }

    #[doc(hidden)]
    pub fn flush_pending_at_for_diagnostics(&self, now: DateTime<Utc>) -> Result<usize> {
        self.maintain_at(now, true)
    }

    #[doc(hidden)]
    pub fn pending_entry_count_for_diagnostics(&self) -> usize {
        pending_entry_count(&self.state.read().expect("dedupe state lock poisoned"))
    }

    #[doc(hidden)]
    pub fn max_pending_entries_for_diagnostics(&self) -> usize {
        MAX_PENDING_ENTRIES
    }

    pub(crate) fn replace_from(&self, source: &DedupeIndex) -> Result<DedupeStats> {
        let _source = source.state.read().expect("source dedupe lock poisoned");
        let mut target = self.state.write().expect("dedupe state lock poisoned");
        let now = Utc::now();
        reconcile_replace(&self.root, now)?;
        let paths = replace_paths(&self.root)?;
        remove_real_dir_if_exists(&paths.stage)?;
        remove_real_dir_if_exists(&paths.backup)?;
        copy_dedupe_tree(&source.root, &paths.stage)?;
        let staged = load_state(&paths.stage, generation_for(now))?;
        if staged.rebuild_required {
            bail!("replacement dedupe index has no durable metadata");
        }
        let intent = ReplaceIntent {
            format_version: REPLACE_FORMAT_VERSION,
            indexed_through_cursor: staged.indexed_through_cursor,
            content_sha256: hex::encode(staged.content_digest),
        };
        storage_durable::atomic_write(
            &paths.marker,
            &serde_json::to_vec(&intent)?,
            storage_durable::FsyncPolicy::Always,
        )?;
        set_private_file(&paths.marker)?;
        fs::rename(&self.root, &paths.backup).context("stage prior dedupe index backup")?;
        storage_durable::sync_parent_dir(&self.root)?;
        fs::rename(&paths.stage, &self.root).context("publish staged dedupe index")?;
        storage_durable::sync_parent_dir(&self.root)?;
        let published = load_intended_state(&self.root, &intent, now)?;
        remove_real_dir_if_exists(&paths.backup)?;
        remove_file_if_exists(&paths.marker)?;
        *target = published;
        Ok(stats_from_state(&target, oldest_generation(now)))
    }

    pub(crate) fn preflight_rebuild(&self) -> Result<()> {
        let metadata = fs::symlink_metadata(&self.root)
            .with_context(|| format!("inspect dedupe index root {}", self.root.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            bail!(
                "dedupe index root {} is not a real directory",
                self.root.display()
            );
        }
        let probe = self.root.join(".rebuild-probe");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let file = options
            .open(&probe)
            .with_context(|| format!("write dedupe rebuild probe {}", probe.display()))?;
        file.sync_all()?;
        drop(file);
        fs::remove_file(&probe)?;
        storage_durable::sync_parent_dir(&probe)?;
        Ok(())
    }

    #[doc(hidden)]
    pub fn disk_bytes(&self) -> Result<u64> {
        let state = self.state.read().expect("dedupe state lock poisoned");
        let mut bytes = 0_u64;
        for generation in state.generations.keys() {
            for entry in fs::read_dir(generation_path(&self.root, *generation))? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    bytes = bytes.saturating_add(entry.metadata()?.len());
                }
            }
        }
        let meta = self.root.join(META_FILE);
        if meta.exists() {
            bytes = bytes.saturating_add(fs::metadata(meta)?.len());
        }
        Ok(bytes)
    }

    #[doc(hidden)]
    pub fn active_generation_count(&self) -> usize {
        self.state
            .read()
            .expect("dedupe state lock poisoned")
            .generations
            .len()
    }
}

struct ReplacePaths {
    marker: PathBuf,
    stage: PathBuf,
    backup: PathBuf,
}

fn replace_paths(root: &Path) -> Result<ReplacePaths> {
    let parent = root.parent().context("dedupe index root has no parent")?;
    Ok(ReplacePaths {
        marker: parent.join(REPLACE_MARKER_FILE),
        stage: parent.join(REPLACE_STAGE_DIR),
        backup: parent.join(REPLACE_BACKUP_DIR),
    })
}

fn reconcile_replace(root: &Path, now: DateTime<Utc>) -> Result<()> {
    let parent = root.parent().context("dedupe index root has no parent")?;
    fs::create_dir_all(parent)?;
    set_private_dir(parent)?;
    let paths = replace_paths(root)?;
    if !paths.marker.exists() {
        if !root.exists() && paths.backup.exists() {
            fs::rename(&paths.backup, root).context("restore interrupted dedupe backup")?;
            storage_durable::sync_parent_dir(root)?;
        } else if root.exists() && paths.backup.exists() {
            load_state(root, generation_for(now))
                .context("validate published dedupe index before deleting backup")?;
            remove_real_dir_if_exists(&paths.backup)?;
        }
        remove_real_dir_if_exists(&paths.stage)?;
        return Ok(());
    }

    let marker_meta = fs::symlink_metadata(&paths.marker)?;
    if marker_meta.file_type().is_symlink() || !marker_meta.is_file() {
        bail!("dedupe replacement marker is not a regular file");
    }
    let intent: ReplaceIntent = serde_json::from_slice(&fs::read(&paths.marker)?)
        .context("decode dedupe replacement intent")?;
    validate_replace_intent(&intent)?;
    if root.exists() && load_intended_state(root, &intent, now).is_ok() {
        remove_real_dir_if_exists(&paths.stage)?;
        remove_real_dir_if_exists(&paths.backup)?;
        remove_file_if_exists(&paths.marker)?;
        return Ok(());
    }

    if paths.stage.exists() && load_intended_state(&paths.stage, &intent, now).is_ok() {
        if root.exists() {
            if paths.backup.exists() {
                remove_real_dir_if_exists(root)?;
            } else {
                fs::rename(root, &paths.backup)
                    .context("preserve prior dedupe index during recovery")?;
                storage_durable::sync_parent_dir(root)?;
            }
        }
        fs::rename(&paths.stage, root).context("finish staged dedupe index publication")?;
        storage_durable::sync_parent_dir(root)?;
        load_intended_state(root, &intent, now)?;
        remove_real_dir_if_exists(&paths.backup)?;
        remove_file_if_exists(&paths.marker)?;
        return Ok(());
    }

    if paths.backup.exists() {
        if root.exists() {
            remove_real_dir_if_exists(root)?;
        }
        fs::rename(&paths.backup, root).context("roll back interrupted dedupe replacement")?;
        storage_durable::sync_parent_dir(root)?;
        load_state(root, generation_for(now)).context("validate rolled-back dedupe index")?;
        remove_real_dir_if_exists(&paths.stage)?;
        remove_file_if_exists(&paths.marker)?;
        return Ok(());
    }

    bail!("dedupe replacement has no valid published, staged, or backup index")
}

fn validate_replace_intent(intent: &ReplaceIntent) -> Result<[u8; 32]> {
    if intent.format_version != REPLACE_FORMAT_VERSION {
        bail!(
            "unsupported dedupe replacement format {}",
            intent.format_version
        );
    }
    hex::decode(&intent.content_sha256)
        .context("decode dedupe replacement content digest")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("dedupe replacement content digest must be 32 bytes"))
}

fn load_intended_state(
    root: &Path,
    intent: &ReplaceIntent,
    now: DateTime<Utc>,
) -> Result<DedupeState> {
    let expected = validate_replace_intent(intent)?;
    let state = load_state(root, generation_for(now))?;
    if state.rebuild_required
        || state.indexed_through_cursor != intent.indexed_through_cursor
        || state.content_digest != expected
    {
        bail!("dedupe replacement does not match its intent");
    }
    Ok(state)
}

fn copy_dedupe_tree(source: &Path, target: &Path) -> Result<()> {
    let source_meta = fs::symlink_metadata(source)?;
    if source_meta.file_type().is_symlink() || !source_meta.is_dir() {
        bail!("source dedupe index is not a real directory");
    }
    fs::create_dir(target)?;
    set_private_dir(target)?;
    let mut generations = Vec::new();
    let mut meta = None::<PathBuf>;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            bail!("source dedupe index contains a symlink");
        }
        if entry.file_name() == META_FILE && file_type.is_file() {
            meta = Some(entry.path());
        } else if file_type.is_dir() {
            generations.push(entry.path());
        } else {
            bail!("source dedupe index contains an unexpected entry");
        }
    }
    generations.sort();
    for generation in generations {
        let name = generation
            .file_name()
            .context("source dedupe generation has no name")?;
        let destination = target.join(name);
        fs::create_dir(&destination)?;
        set_private_dir(&destination)?;
        let mut shards = fs::read_dir(&generation)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        shards.sort();
        for shard in shards {
            let metadata = fs::symlink_metadata(&shard)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("source dedupe generation contains a special file");
            }
            let target_file = destination.join(
                shard
                    .file_name()
                    .context("source dedupe shard has no file name")?,
            );
            fs::copy(&shard, &target_file)?;
            set_private_file(&target_file)?;
            File::open(&target_file)?.sync_all()?;
            storage_durable::sync_parent_dir(&target_file)?;
        }
    }
    let meta = meta.context("source dedupe index has no durable metadata")?;
    let target_meta = target.join(META_FILE);
    fs::copy(meta, &target_meta)?;
    set_private_file(&target_meta)?;
    File::open(&target_meta)?.sync_all()?;
    storage_durable::sync_parent_dir(&target_meta)?;
    storage_durable::sync_parent_dir(target)
}

fn remove_real_dir_if_exists(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            bail!(
                "dedupe replacement path {} is not a real directory",
                path.display()
            )
        }
        Ok(_) => fs::remove_dir_all(path)
            .with_context(|| format!("remove dedupe replacement path {}", path.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    storage_durable::sync_parent_dir(path)
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            bail!("dedupe replacement marker is not a regular file")
        }
        Ok(_) => fs::remove_file(path)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    storage_durable::sync_parent_dir(path)
}

impl GenerationState {
    fn empty(sealed: bool) -> Self {
        Self {
            sealed,
            blooms: vec![BloomLayer::new(MIN_BLOOM_BYTES)],
            entry_count: 0,
            newest_cursor: 0,
            newest_acknowledged_at_unix_nano: None,
            content_digest: [0; 32],
            receipt_writer: None,
            pending: BTreeMap::new(),
        }
    }
}

impl BloomLayer {
    fn new(bytes: usize) -> Self {
        Self {
            bits: vec![0; bytes],
            entries: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.entries >= bloom_entries_for_bytes(self.bits.len())
    }
}

fn generation_for(now: DateTime<Utc>) -> i64 {
    now.timestamp().div_euclid(GENERATION_SECONDS)
}

fn oldest_generation(now: DateTime<Utc>) -> i64 {
    now.timestamp()
        .saturating_sub(IDEMPOTENCY_WINDOW_SECONDS)
        .div_euclid(GENERATION_SECONDS)
}

fn acknowledged_generation(event: &StoredEvent) -> Result<i64> {
    Ok(DateTime::parse_from_rfc3339(&event.acknowledged_at)
        .context("acknowledged_at must be RFC3339")?
        .timestamp()
        .div_euclid(GENERATION_SECONDS))
}

fn acknowledged_nanos(event: &StoredEvent) -> Result<i64> {
    DateTime::parse_from_rfc3339(&event.acknowledged_at)
        .context("acknowledged_at must be RFC3339")?
        .timestamp_nanos_opt()
        .context("acknowledged_at is outside the nanosecond range")
}

fn generation_path(root: &Path, generation: i64) -> PathBuf {
    root.join(format!("g-{generation}"))
}

fn parse_generation(name: &str) -> Result<i64> {
    name.strip_prefix("g-")
        .context("dedupe generation directory has an unknown name")?
        .parse::<i64>()
        .context("dedupe generation directory has an invalid number")
}

fn shard_path(root: &Path, generation: i64, digest: &[u8; 32]) -> PathBuf {
    generation_path(root, generation).join(format!("{:03x}.idx", shard_for(digest)))
}

fn load_state(root: &Path, current: i64) -> Result<DedupeState> {
    let mut state = DedupeState::default();
    let mut found_meta = false;
    let mut expected_content_digest = None::<[u8; 32]>;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || (!file_type.is_dir() && !file_type.is_file()) {
            bail!("dedupe root contains a special file");
        }
        if file_type.is_file() {
            if entry.file_name() != META_FILE {
                bail!("legacy or damaged dedupe index requires a rebuild");
            }
            let meta: DedupeMeta = serde_json::from_slice(&fs::read(entry.path())?)
                .context("decode dedupe index metadata")?;
            if meta.format_version != FORMAT_VERSION {
                bail!("unsupported dedupe index format {}", meta.format_version);
            }
            state.indexed_through_cursor = meta.indexed_through_cursor;
            expected_content_digest = Some(
                hex::decode(&meta.content_sha256)
                    .context("decode dedupe index content digest")?
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("dedupe index content digest must be 32 bytes"))?,
            );
            found_meta = true;
            continue;
        }
        let name = entry.file_name();
        let generation = parse_generation(&name.to_string_lossy())?;
        set_private_dir(&entry.path())?;
        let sealed = generation < current;
        let loaded = load_generation(&entry.path(), sealed)?;
        state.applied_time_unix_nano = match (
            state.applied_time_unix_nano,
            loaded.newest_acknowledged_at_unix_nano,
        ) {
            (Some(current), Some(loaded)) => Some(current.max(loaded)),
            (None, loaded) => loaded,
            (current, None) => current,
        };
        xor_digest_in_place(&mut state.content_digest, loaded.content_digest);
        state.indexed_through_cursor = state.indexed_through_cursor.max(loaded.newest_cursor);
        state.generations.insert(generation, loaded);
    }
    if expected_content_digest.is_some_and(|expected| expected != state.content_digest) {
        bail!("dedupe index content digest mismatch");
    }
    state.rebuild_required = !found_meta;
    Ok(state)
}

fn load_generation(directory: &Path, sealed: bool) -> Result<GenerationState> {
    let receipt_path = directory.join(RECEIPT_LOG_FILE);
    let mut receipts = load_generation_receipts(directory, sealed)?;
    let (shard_count, shard_digest) = generation_shard_identity(directory)?;
    if shard_count != receipts.entry_count || shard_digest != receipts.content_digest {
        rebuild_generation_shards(directory, sealed)?;
    } else if sealed {
        seal_generation(directory)?;
    }
    if !sealed {
        receipts.receipt_writer = Some(storage_durable::FramedLogWriter::open(
            receipt_path,
            storage_durable::FsyncPolicy::Os,
        )?);
    }
    Ok(receipts)
}

fn stats_from_state(state: &DedupeState, oldest: i64) -> DedupeStats {
    let mut stats = DedupeStats {
        indexed_through_cursor: state.indexed_through_cursor,
        window_seconds: IDEMPOTENCY_WINDOW_SECONDS as u64,
        rebuild_required: state.rebuild_required,
        ..DedupeStats::default()
    };
    for (generation, index) in state.generations.range(oldest..) {
        if stats.entry_count == 0 {
            stats.oldest_generation = *generation;
        }
        stats.newest_generation = *generation;
        stats.entry_count = stats.entry_count.saturating_add(index.entry_count);
        stats.newest_cursor = stats.newest_cursor.max(index.newest_cursor);
    }
    stats
}

fn pending_entry_count(state: &DedupeState) -> usize {
    state.generations.values().fold(0_usize, |count, index| {
        count.saturating_add(index.pending.len())
    })
}

fn persist_meta(root: &Path, indexed_through_cursor: u64, content_digest: [u8; 32]) -> Result<()> {
    persist_meta_with_policy(
        root,
        indexed_through_cursor,
        content_digest,
        storage_durable::FsyncPolicy::Always,
    )
}

fn persist_meta_with_policy(
    root: &Path,
    indexed_through_cursor: u64,
    content_digest: [u8; 32],
    policy: storage_durable::FsyncPolicy,
) -> Result<()> {
    let bytes = serde_json::to_vec(&DedupeMeta {
        format_version: FORMAT_VERSION,
        indexed_through_cursor,
        content_sha256: hex::encode(content_digest),
    })?;
    let path = root.join(META_FILE);
    storage_durable::atomic_write(&path, &bytes, policy)?;
    set_private_file(&path)
}

fn clear_root(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            bail!("dedupe root contains a symlink");
        }
        if file_type.is_dir() {
            remove_generation_dir(&path)?;
        } else if file_type.is_file() {
            fs::remove_file(&path)?;
        } else {
            bail!("dedupe root contains a special file");
        }
    }
    storage_durable::sync_parent_dir(root)
}

fn remove_generation_dir(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            bail!(
                "dedupe generation {} is not a real directory",
                path.display()
            )
        }
        Ok(_) => fs::remove_dir_all(path)
            .with_context(|| format!("remove expired dedupe generation {}", path.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    storage_durable::sync_parent_dir(path)
}

fn append_entries(path: &Path, entries: &[([u8; 32], u64, i64)]) -> Result<()> {
    let created = !path.exists();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("append dedupe index shard {}", path.display()))?;
    if created {
        set_private_file(path)?;
    }
    let mut bytes = Vec::with_capacity(entries.len() * ENTRY_BYTES);
    for (digest, cursor, acknowledged_at) in entries {
        bytes.extend_from_slice(digest);
        bytes.extend_from_slice(&cursor.to_le_bytes());
        bytes.extend_from_slice(&acknowledged_at.to_le_bytes());
    }
    file.write_all(&bytes)?;
    // The generation receipt WAL is the single durable batch boundary. Shard
    // files are a rebuildable lookup projection and intentionally do not fsync
    // on the ingest acknowledgement path.
    Ok(())
}

fn seal_generation(directory: &Path) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    let mut paths = generation_shard_paths(directory)?;
    paths.sort();
    for path in paths {
        let mut entries = read_entries(&path)?;
        entries.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then(left.2.cmp(&right.2))
                .then(left.1.cmp(&right.1))
        });
        write_entries_with_policy(&path, &entries, storage_durable::FsyncPolicy::Os)?;
    }
    Ok(())
}

fn generation_shard_paths(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("dedupe generation contains a special file");
        }
        if entry.file_name() == RECEIPT_LOG_FILE {
            set_private_file(&path)?;
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("idx") {
            bail!(
                "dedupe generation contains an unknown file {}",
                path.display()
            );
        }
        if metadata.len() % ENTRY_BYTES as u64 != 0 {
            bail!("dedupe index shard {} has a torn record", path.display());
        }
        set_private_file(&path)?;
        paths.push(path);
    }
    paths.sort();
    Ok(paths)
}

fn encode_receipt_records(entries: &[DedupeRecord]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(entries.len() * ENTRY_BYTES);
    for (digest, cursor, acknowledged_at) in entries {
        bytes.extend_from_slice(digest);
        bytes.extend_from_slice(&cursor.to_le_bytes());
        bytes.extend_from_slice(&acknowledged_at.to_le_bytes());
    }
    bytes
}

fn decode_receipt_records(bytes: &[u8]) -> Result<Vec<DedupeRecord>> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(ENTRY_BYTES) {
        bail!("dedupe receipt frame has an invalid record boundary");
    }
    Ok(bytes
        .chunks_exact(ENTRY_BYTES)
        .map(|entry| {
            (
                entry[..32].try_into().expect("fixed digest bytes"),
                u64::from_le_bytes(entry[32..40].try_into().expect("fixed cursor bytes")),
                i64::from_le_bytes(
                    entry[40..48]
                        .try_into()
                        .expect("fixed acknowledgement bytes"),
                ),
            )
        })
        .collect())
}

fn load_generation_receipts(directory: &Path, sealed: bool) -> Result<GenerationState> {
    let generation = parse_generation(
        directory
            .file_name()
            .and_then(|name| name.to_str())
            .context("dedupe generation path has no file name")?,
    )?;
    let receipt_path = directory.join(RECEIPT_LOG_FILE);
    if !receipt_path.exists() {
        bail!("dedupe generation is missing its receipt WAL");
    }
    let mut state = GenerationState::empty(sealed);
    storage_durable::FramedLogReader::visit_frames(&receipt_path, 0, |frame| {
        let records = decode_receipt_records(&frame.payload)?;
        let frame_cursor = records
            .iter()
            .map(|(_, cursor, _)| *cursor)
            .max()
            .context("dedupe receipt frame has no records")?;
        if frame.seq != frame_cursor {
            bail!("dedupe receipt frame sequence does not match its cursor");
        }
        for record @ (digest, cursor, acknowledged_at) in records {
            let shard = shard_for(&digest);
            bloom_insert_scalable(&mut state.blooms, &digest);
            state.entry_count = state.entry_count.saturating_add(1);
            state.newest_cursor = state.newest_cursor.max(cursor);
            state.newest_acknowledged_at_unix_nano = Some(
                state
                    .newest_acknowledged_at_unix_nano
                    .unwrap_or(i64::MIN)
                    .max(acknowledged_at),
            );
            xor_digest_in_place(
                &mut state.content_digest,
                dedupe_record_digest(generation, shard, &record),
            );
        }
        Ok(())
    })?;
    Ok(state)
}

fn generation_shard_identity(directory: &Path) -> Result<(u64, [u8; 32])> {
    let generation = parse_generation(
        directory
            .file_name()
            .and_then(|name| name.to_str())
            .context("dedupe generation path has no file name")?,
    )?;
    let mut count = 0_u64;
    let mut digest = [0_u8; 32];
    for path in generation_shard_paths(directory)? {
        let shard = usize::from_str_radix(
            path.file_stem()
                .and_then(|name| name.to_str())
                .context("dedupe shard path has no file name")?,
            16,
        )
        .context("dedupe shard file has an invalid number")?;
        for record in read_entries(&path)? {
            count = count.saturating_add(1);
            xor_digest_in_place(
                &mut digest,
                dedupe_record_digest(generation, shard, &record),
            );
        }
    }
    Ok((count, digest))
}

fn rebuild_generation_shards(directory: &Path, sealed: bool) -> Result<()> {
    for path in generation_shard_paths(directory)? {
        fs::remove_file(&path)
            .with_context(|| format!("remove rebuildable dedupe shard {}", path.display()))?;
    }
    storage_durable::sync_parent_dir(directory)?;
    let receipt_path = directory.join(RECEIPT_LOG_FILE);
    storage_durable::FramedLogReader::visit_frames(&receipt_path, 0, |frame| {
        let mut shards = BTreeMap::<usize, Vec<([u8; 32], u64, i64)>>::new();
        for record @ (digest, _, _) in decode_receipt_records(&frame.payload)? {
            shards.entry(shard_for(&digest)).or_default().push(record);
        }
        for (shard, records) in shards {
            append_entries(&directory.join(format!("{shard:03x}.idx")), &records)?;
        }
        Ok(())
    })?;
    if sealed {
        seal_generation(directory)?;
    }
    Ok(())
}

fn lookup_file(path: &Path, digest: &[u8; 32], sorted: bool) -> Result<Option<(u64, i64)>> {
    if !path.exists() {
        return Ok(None);
    }
    let metadata = fs::metadata(path)?;
    if metadata.len() % ENTRY_BYTES as u64 != 0 {
        bail!("dedupe index shard {} has a torn record", path.display());
    }
    let entries = metadata.len() / ENTRY_BYTES as u64;
    let mut file = File::open(path)?;
    if !sorted {
        let mut entry = [0u8; ENTRY_BYTES];
        for _ in 0..entries {
            file.read_exact(&mut entry)?;
            if entry[..32] == digest[..] {
                return Ok(Some((
                    u64::from_le_bytes(entry[32..40].try_into().unwrap()),
                    i64::from_le_bytes(entry[40..48].try_into().unwrap()),
                )));
            }
        }
        return Ok(None);
    }
    let mut low = 0_u64;
    let mut high = entries;
    let mut entry = [0u8; ENTRY_BYTES];
    while low < high {
        let middle = low + (high - low) / 2;
        file.seek(SeekFrom::Start(middle * ENTRY_BYTES as u64))?;
        file.read_exact(&mut entry)?;
        match entry[..32].cmp(&digest[..]) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => {
                return Ok(Some((
                    u64::from_le_bytes(entry[32..40].try_into().unwrap()),
                    i64::from_le_bytes(entry[40..48].try_into().unwrap()),
                )))
            }
        }
    }
    Ok(None)
}

fn read_entries(path: &Path) -> Result<Vec<DedupeRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path)?;
    if bytes.len() % ENTRY_BYTES != 0 {
        bail!("dedupe index shard {} has a torn record", path.display());
    }
    Ok(bytes
        .chunks_exact(ENTRY_BYTES)
        .map(|entry| {
            (
                entry[..32].try_into().expect("fixed digest bytes"),
                u64::from_le_bytes(entry[32..40].try_into().expect("fixed cursor bytes")),
                i64::from_le_bytes(
                    entry[40..48]
                        .try_into()
                        .expect("fixed acknowledgement bytes"),
                ),
            )
        })
        .collect())
}

fn write_entries_with_policy(
    path: &Path,
    entries: &[DedupeRecord],
    policy: storage_durable::FsyncPolicy,
) -> Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * ENTRY_BYTES);
    for (digest, cursor, acknowledged_at) in entries {
        bytes.extend_from_slice(digest);
        bytes.extend_from_slice(&cursor.to_le_bytes());
        bytes.extend_from_slice(&acknowledged_at.to_le_bytes());
    }
    storage_durable::atomic_write(path, &bytes, policy)?;
    set_private_file(path)
}

fn dedupe_record_digest(
    generation: i64,
    shard: usize,
    (digest, cursor, acknowledged_at): &([u8; 32], u64, i64),
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"sift-dedupe-record-v1\0");
    hasher.update(generation.to_le_bytes());
    hasher.update((shard as u64).to_le_bytes());
    hasher.update(digest);
    hasher.update(cursor.to_le_bytes());
    hasher.update(acknowledged_at.to_le_bytes());
    hasher.finalize().into()
}

fn xor_digest_in_place(target: &mut [u8; 32], value: [u8; 32]) {
    for (target, value) in target.iter_mut().zip(value) {
        *target ^= value;
    }
}

fn bloom_entries_for_bytes(bytes: usize) -> u64 {
    (bytes as u64)
        .saturating_mul(8)
        .checked_div(BLOOM_BITS_PER_ENTRY)
        .unwrap_or_default()
        .max(1)
}

fn bloom_insert_scalable(layers: &mut Vec<BloomLayer>, digest: &[u8; 32]) {
    if layers.last().is_none_or(BloomLayer::is_full) {
        let bytes = layers
            .last()
            .map(|layer| layer.bits.len().saturating_mul(2))
            .unwrap_or(MIN_BLOOM_BYTES)
            .min(MAX_BLOOM_BYTES);
        layers.push(BloomLayer::new(bytes));
    }
    let layer = layers.last_mut().expect("Bloom layer exists");
    bloom_insert(&mut layer.bits, digest);
    layer.entries = layer.entries.saturating_add(1);
}

fn event_digest(project: &str, event_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"sift-dedupe-v6\0");
    hasher.update((project.len() as u64).to_be_bytes());
    hasher.update(project.as_bytes());
    hasher.update((event_id.len() as u64).to_be_bytes());
    hasher.update(event_id.as_bytes());
    hasher.finalize().into()
}

fn event_is_active_at(event: &StoredEvent, now: DateTime<Utc>) -> Result<bool> {
    let cutoff = (now - chrono::Duration::seconds(IDEMPOTENCY_WINDOW_SECONDS))
        .timestamp_nanos_opt()
        .context("dedupe activity cutoff is outside the nanosecond range")?;
    Ok(acknowledged_nanos(event)? >= cutoff)
}

fn shard_for(digest: &[u8; 32]) -> usize {
    (((digest[0] as usize) << 4) | ((digest[1] as usize) >> 4)) % SHARD_COUNT
}

fn bloom_insert(bloom: &mut [u8], digest: &[u8; 32]) {
    for position in bloom_positions(digest, bloom.len() * 8) {
        bloom[position / 8] |= 1 << (position % 8);
    }
}

fn bloom_contains(bloom: &[u8], digest: &[u8; 32]) -> bool {
    bloom_positions(digest, bloom.len() * 8)
        .into_iter()
        .all(|position| bloom[position / 8] & (1 << (position % 8)) != 0)
}

fn bloom_positions(digest: &[u8; 32], bits: usize) -> [usize; 4] {
    let word = |offset| {
        u64::from_le_bytes(
            digest[offset..offset + 8]
                .try_into()
                .expect("fixed digest word"),
        ) as usize
            % bits
    };
    [word(0), word(8), word(16), word(24)]
}

fn set_private_dir(path: &Path) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn set_private_file(path: &Path) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
