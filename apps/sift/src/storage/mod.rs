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

use anyhow::Result;

use crate::{ContentBlobRef, OperationalEventV2, SignalKind, StoredEvent};

pub use blob::BlobStore;
pub use capacity::{CapacityLevel, LocalCapacity, LocalCapacityError};
pub use dedupe::DedupeIndex;
pub use head::JournalHead;
pub use layout::{DataLayout, LayoutManifest, StorageRole, DEFAULT_DATA_DIR};
pub use segment::{AppendLocation, SegmentManifest, SegmentState};
pub use shard::{EpochMap, Route, VIRTUAL_BUCKETS};
pub use wal::SignalWal;

use segment::SegmentStore;
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

struct SignalSegmentStores {
    logs: SegmentStore,
    metrics: SegmentStore,
    traces: SegmentStore,
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

    pub(crate) fn write_retained_segment(
        &self,
        signal: SignalKind,
        source_segment_id: &str,
        retained: &[StoredEvent],
    ) -> Result<Option<SegmentManifest>> {
        self.segments
            .for_signal(signal)?
            .write_retained_segment(source_segment_id, retained)
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

    pub(crate) fn prune_blobs_except(
        &self,
        retained_hashes: &std::collections::BTreeSet<String>,
    ) -> Result<usize> {
        self.blobs.prune_except(retained_hashes)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
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
