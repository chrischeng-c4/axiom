// HANDWRITE-BEGIN gap="sift-sharded-storage-module" tracker="1659" reason="Compose blob, routing, segment, and archive ownership as Sift's canonical raw storage plane."
//! Canonical raw storage plane: content-addressed blobs plus epoch-routed,
//! CRC-framed per-signal segments.

pub mod archive;
mod blob;
mod capacity;
mod dedupe;
mod head;
mod layout;
mod segment;
mod shard;
mod wal;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::{ContentBlobRef, OperationalEventV2, SignalKind, StoredEvent};

pub use blob::BlobStore;
pub use capacity::{CapacityLevel, LocalCapacity, LocalCapacityError};
pub(crate) use dedupe::DedupeReceipt;
pub use dedupe::{DedupeIndex, IDEMPOTENCY_WINDOW_SECONDS};
pub use head::JournalHead;
pub use layout::{DataLayout, LayoutManifest, StorageRole, DEFAULT_DATA_DIR};
pub use segment::{AppendLocation, SegmentManifest, SegmentState};
pub use shard::{EpochMap, Route, VIRTUAL_BUCKETS};
pub use wal::{SignalWal, SignalWalReader};

pub(crate) trait BlobHashSet {
    fn insert_hash(&mut self, hash: &str) -> anyhow::Result<()>;
    fn contains_hash(&self, hash: &str) -> anyhow::Result<bool>;
}

use segment::{SegmentEventReader, SegmentStore};
use shard::ShardRouter;

#[derive(Clone, Debug)]
pub struct StorageConfig {
    pub initial_logical_shards: u16,
    pub max_segment_events: usize,
    pub max_segment_bytes: usize,
    pub blob_externalize_bytes: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            initial_logical_shards: 1,
            max_segment_events: 100_000,
            max_segment_bytes: 256 * 1024 * 1024,
            blob_externalize_bytes: 65_536,
        }
    }
}

pub struct RawStorage {
    root: PathBuf,
    blobs: BlobStore,
    router: ShardRouter,
    segments: SignalSegmentStores,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RetainedPrefixReconcileStats {
    pub max_buffered_events: usize,
}

struct SignalSegmentStores {
    logs: SegmentStore,
    metrics: SegmentStore,
    traces: SegmentStore,
}

pub(crate) struct RawStorageReader {
    streams: Vec<(SignalKind, SegmentEventReader)>,
    peeked: Vec<Option<StoredEvent>>,
}

impl RawStorage {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_config(root, StorageConfig::default())
    }

    pub fn open_with_config(root: impl AsRef<Path>, config: StorageConfig) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root)?;
        Ok(Self {
            blobs: BlobStore::open(&root, config.blob_externalize_bytes)?,
            router: ShardRouter::open(&root, config.initial_logical_shards)?,
            segments: SignalSegmentStores {
                logs: SegmentStore::open_at(
                    root.join("segments").join("logs"),
                    config.max_segment_events,
                    config.max_segment_bytes,
                )?,
                metrics: SegmentStore::open_at(
                    root.join("segments").join("metrics"),
                    config.max_segment_events,
                    config.max_segment_bytes,
                )?,
                traces: SegmentStore::open_at(
                    root.join("segments").join("traces"),
                    config.max_segment_events,
                    config.max_segment_bytes,
                )?,
            },
            root,
        })
    }

    pub fn externalize_event(&self, event: &mut OperationalEventV2) -> Result<()> {
        self.blobs.externalize_event(event)?;
        self.blobs.validate_references(&event.blob_refs)
    }

    pub fn append(&self, stored: &StoredEvent) -> Result<AppendLocation> {
        let route = self.route(&stored.event.event_id, stored.cursor);
        self.segments
            .for_signal(stored.event.signal)?
            .append(route, stored)
    }

    pub fn append_batch(&self, events: &[StoredEvent]) -> Result<Vec<AppendLocation>> {
        let locations = events
            .iter()
            .map(|event| self.append(event))
            .collect::<Result<Vec<_>>>()?;
        for signal in SignalKind::ALL {
            if events.iter().any(|event| event.event.signal == signal) {
                self.segments.for_signal(signal)?.flush_active()?;
            }
        }
        Ok(locations)
    }

    pub fn seal_ready(&self) -> Result<Vec<SegmentManifest>> {
        let mut manifests = Vec::new();
        for segments in self.segments.all() {
            manifests.extend(segments.seal_ready()?);
        }
        manifests.sort_by_key(|manifest| manifest.first_cursor);
        Ok(manifests)
    }

    pub fn route(&self, event_id: &str, cursor: u64) -> Route {
        self.router.route(event_id, cursor)
    }

    pub fn activate_epoch(&self, activated_at_cursor: u64, buckets: Vec<u16>) -> Result<EpochMap> {
        self.router.activate(activated_at_cursor, buckets)
    }

    pub fn epoch_maps(&self) -> Vec<EpochMap> {
        self.router.epochs()
    }

    pub fn recovered_events(&self) -> Result<Vec<StoredEvent>> {
        let mut events = Vec::new();
        for segments in self.segments.all() {
            events.extend(segments.recovered_events()?);
        }
        events.sort_by_key(|event| event.cursor);
        Ok(events)
    }

    pub fn query_events(
        &self,
        signal: Option<SignalKind>,
        after: u64,
        limit: usize,
    ) -> Result<Vec<StoredEvent>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut events = Vec::new();
        match signal {
            Some(signal) => {
                events.extend(
                    self.segments
                        .for_signal(signal)?
                        .query_events(after, limit)?,
                );
            }
            None => {
                for segments in self.segments.all() {
                    events.extend(segments.query_events(after, limit)?);
                }
            }
        }
        events.sort_by_key(|event| event.cursor);
        let mut canonical = Vec::<StoredEvent>::with_capacity(events.len());
        for event in events {
            if let Some(previous) = canonical.last() {
                if previous.cursor == event.cursor {
                    if previous != &event {
                        anyhow::bail!("raw signal segments disagree at cursor {}", event.cursor);
                    }
                    continue;
                }
            }
            canonical.push(event);
        }
        let mut events = canonical;
        events.truncate(limit);
        Ok(events)
    }

    pub(crate) fn reader(&self, after: u64) -> Result<RawStorageReader> {
        let streams = SignalKind::ALL
            .into_iter()
            .map(|signal| Ok((signal, self.segments.for_signal(signal)?.reader(after)?)))
            .collect::<Result<Vec<_>>>()?;
        Ok(RawStorageReader {
            peeked: vec![None; streams.len()],
            streams,
        })
    }

    pub fn seal_all(&self) -> Result<Vec<SegmentManifest>> {
        Ok(self
            .seal_all_with_signal()?
            .into_iter()
            .map(|(_, manifest)| manifest)
            .collect())
    }

    pub fn seal_all_with_signal(&self) -> Result<Vec<(SignalKind, SegmentManifest)>> {
        let mut manifests = Vec::new();
        for signal in SignalKind::ALL {
            manifests.extend(
                self.segments
                    .for_signal(signal)?
                    .seal_all()?
                    .into_iter()
                    .map(|manifest| (signal, manifest)),
            );
        }
        manifests.sort_by_key(|(_, manifest)| manifest.first_cursor);
        Ok(manifests)
    }

    /// Replace the local archived prefix with the exact retained rows from a
    /// hash-verified restore. Rows after `snapshot_index` are a Raft suffix and
    /// remain untouched.
    #[doc(hidden)]
    pub fn reconcile_retained_prefix(
        &self,
        retained: &RawStorage,
        snapshot_index: u64,
    ) -> Result<RetainedPrefixReconcileStats> {
        let mut stats = RetainedPrefixReconcileStats::default();

        // First remove every local row in the authoritative checkpoint
        // prefix. Keep only the post-checkpoint Raft suffix. One immutable
        // segment is materialized at a time, so memory does not grow with the
        // total retained row count.
        for (signal, manifest) in self.seal_all_with_signal()? {
            let events = self.read_segment_events(signal, &manifest)?;
            stats.max_buffered_events = stats.max_buffered_events.max(events.len());
            let replacement = events
                .iter()
                .filter(|event| event.cursor > snapshot_index)
                .cloned()
                .collect::<Vec<_>>();
            if replacement == events {
                continue;
            }
            if !replacement.is_empty() {
                self.segments
                    .for_signal(signal)?
                    .write_reconciled_segment(&manifest.segment_id, &replacement)?;
            }
            self.evict_segment(signal, &manifest.segment_id)?;
        }

        // Rebuild the exact retained prefix from the verified restore source.
        // The source reader owns one frame at a time. The append batch is also
        // bounded by item count and encoded bytes.
        for signal in SignalKind::ALL {
            let mut reader = retained.segments.for_signal(signal)?.reader(0)?;
            let mut batch = Vec::with_capacity(1_000);
            let mut batch_bytes = 0_usize;
            while let Some(event) = reader.next_event()? {
                if event.event.signal != signal {
                    anyhow::bail!("retained archive contains the wrong signal");
                }
                if event.cursor > snapshot_index {
                    break;
                }
                let encoded = serde_json::to_vec(&event)?.len();
                if !batch.is_empty()
                    && (batch.len() == 1_000
                        || batch_bytes.saturating_add(encoded) > 16 * 1024 * 1024)
                {
                    stats.max_buffered_events = stats.max_buffered_events.max(batch.len());
                    self.append_batch(&batch)?;
                    batch.clear();
                    batch_bytes = 0;
                }
                batch_bytes = batch_bytes.saturating_add(encoded);
                batch.push(event);
            }
            if !batch.is_empty() {
                stats.max_buffered_events = stats.max_buffered_events.max(batch.len());
                self.append_batch(&batch)?;
            }
        }

        verify_local_retained_exact(self, retained, snapshot_index)?;
        Ok(stats)
    }

    pub fn read_segment_events(
        &self,
        signal: SignalKind,
        manifest: &SegmentManifest,
    ) -> Result<Vec<StoredEvent>> {
        let events = self
            .segments
            .for_signal(signal)?
            .read_manifest_events(manifest)?;
        if events.iter().any(|event| event.event.signal != signal) {
            anyhow::bail!("segment {} contains the wrong signal", manifest.segment_id);
        }
        Ok(events)
    }

    /// Remove only local cache rows older than a committed retention cutoff.
    /// Rows after the archived Raft prefix stay intact. The remote manifest is
    /// still the authority for every cold row while a bounded scan continues.
    pub(crate) fn evict_expired_before(
        &self,
        cutoff: DateTime<Utc>,
        snapshot_index: u64,
    ) -> Result<u64> {
        let cutoff_nanos = cutoff
            .timestamp_nanos_opt()
            .context("local retention cutoff is outside the nanosecond range")?;
        let mut removed = 0_u64;
        for (signal, manifest) in self.seal_all_with_signal()? {
            if manifest.first_cursor > snapshot_index
                || manifest.min_event_time_unix_nano >= cutoff_nanos
            {
                continue;
            }
            let events = self.read_segment_events(signal, &manifest)?;
            let mut retained = Vec::with_capacity(events.len());
            for event in events {
                let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
                    .context("local retained event occurred_at must be RFC3339")?
                    .with_timezone(&Utc);
                if event.cursor <= snapshot_index && occurred < cutoff {
                    removed = removed.saturating_add(1);
                } else {
                    retained.push(event);
                }
            }
            if retained.len() as u64 == manifest.event_count {
                continue;
            }
            if !retained.is_empty() {
                self.segments
                    .for_signal(signal)?
                    .write_reconciled_segment(&manifest.segment_id, &retained)?;
            }
            self.evict_segment(signal, &manifest.segment_id)?;
        }
        Ok(removed)
    }

    pub(crate) fn evict_segment(
        &self,
        signal: SignalKind,
        segment_id: &str,
    ) -> Result<Option<SegmentManifest>> {
        let receipt_root = self
            .root
            .join("archive-cache")
            .join("evicted")
            .join(match signal {
                SignalKind::Log => "logs",
                SignalKind::Metric => "metrics",
                SignalKind::Span => "traces",
            });
        self.segments
            .for_signal(signal)?
            .evict_segment(segment_id, &receipt_root)
    }

    pub fn manifests(&self) -> Result<Vec<SegmentManifest>> {
        let mut manifests = Vec::new();
        for segments in self.segments.all() {
            manifests.extend(segments.manifests()?);
        }
        manifests.sort_by_key(|manifest| manifest.first_cursor);
        Ok(manifests)
    }

    pub fn active_segment_paths(&self) -> Vec<PathBuf> {
        self.segments
            .all()
            .into_iter()
            .flat_map(SegmentStore::active_paths)
            .collect()
    }

    pub fn move_segment(
        &self,
        segment_id: &str,
        destination: impl AsRef<Path>,
    ) -> Result<SegmentManifest> {
        for segments in self.segments.all() {
            if segments
                .manifests()?
                .iter()
                .any(|manifest| manifest.segment_id == segment_id)
            {
                return segments.move_segment(segment_id, destination.as_ref());
            }
        }
        anyhow::bail!("segment `{segment_id}` is not sealed")
    }

    pub fn read_blob(&self, hash: &str) -> Result<Vec<u8>> {
        self.blobs.read(hash)
    }

    pub fn validate_blob_refs(&self, references: &[ContentBlobRef]) -> Result<()> {
        self.blobs.validate_references(references)
    }

    pub fn blob_paths(&self) -> Result<Vec<PathBuf>> {
        self.blobs.blob_paths()
    }

    pub(crate) fn remove_blob(&self, hash: &str) -> Result<bool> {
        self.blobs.remove(hash)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl RawStorageReader {
    pub(crate) fn next_event(&mut self) -> Result<Option<StoredEvent>> {
        for index in 0..self.streams.len() {
            if self.peeked[index].is_none() {
                let (signal, stream) = &mut self.streams[index];
                let event = stream.next_event()?;
                if event
                    .as_ref()
                    .is_some_and(|event| event.event.signal != *signal)
                {
                    anyhow::bail!("raw segment contains the wrong signal");
                }
                self.peeked[index] = event;
            }
        }
        let Some((selected, cursor)) = self
            .peeked
            .iter()
            .enumerate()
            .filter_map(|(index, event)| event.as_ref().map(|event| (index, event.cursor)))
            .min_by_key(|(_, cursor)| *cursor)
        else {
            return Ok(None);
        };
        let event = self.peeked[selected]
            .take()
            .expect("selected raw segment event exists");
        for (index, duplicate) in self.peeked.iter_mut().enumerate() {
            if index == selected || duplicate.as_ref().map(|row| row.cursor) != Some(cursor) {
                continue;
            }
            let duplicate = duplicate.take().expect("matched raw segment event exists");
            if duplicate != event {
                anyhow::bail!("raw signal segments disagree at cursor {cursor}");
            }
        }
        Ok(Some(event))
    }
}

fn verify_local_retained_exact(
    local: &RawStorage,
    retained: &RawStorage,
    snapshot_index: u64,
) -> Result<()> {
    for signal in SignalKind::ALL {
        let mut local_reader = local.segments.for_signal(signal)?.reader(0)?;
        let mut retained_reader = retained.segments.for_signal(signal)?.reader(0)?;
        loop {
            let local_event = next_prefix_event(&mut local_reader, signal, snapshot_index)?;
            let retained_event = next_prefix_event(&mut retained_reader, signal, snapshot_index)?;
            if local_event != retained_event {
                anyhow::bail!(
                    "local {} prefix does not equal the retained archive through cursor {snapshot_index}",
                    signal
                );
            }
            if local_event.is_none() {
                break;
            }
        }
    }
    Ok(())
}

fn next_prefix_event(
    reader: &mut SegmentEventReader,
    signal: SignalKind,
    snapshot_index: u64,
) -> Result<Option<StoredEvent>> {
    let Some(event) = reader.next_event()? else {
        return Ok(None);
    };
    if event.event.signal != signal {
        anyhow::bail!("retained-prefix verification found the wrong signal");
    }
    Ok((event.cursor <= snapshot_index).then_some(event))
}

impl SignalSegmentStores {
    fn for_signal(&self, signal: SignalKind) -> Result<&SegmentStore> {
        match signal {
            SignalKind::Log => Ok(&self.logs),
            SignalKind::Metric => Ok(&self.metrics),
            SignalKind::Span => Ok(&self.traces),
        }
    }

    fn all(&self) -> [&SegmentStore; 3] {
        [&self.logs, &self.metrics, &self.traces]
    }
}
// HANDWRITE-END
