// HANDWRITE-BEGIN gap="sift-sharded-storage-module" tracker="1659" reason="Compose blob, routing, segment, and archive ownership as Sift's canonical raw storage plane."
//! Canonical raw storage plane: content-addressed blobs plus epoch-routed,
//! CRC-framed segments. The legacy flat journal remains a compatibility copy.

pub mod archive;
mod blob;
mod segment;
mod shard;

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{ContentBlobRef, OperationalEventV2, StoredEvent};

pub use blob::BlobStore;
pub use segment::{AppendLocation, SegmentManifest, SegmentState};
pub use shard::{EpochMap, Route, VIRTUAL_BUCKETS};

use segment::SegmentStore;
use shard::ShardRouter;

#[derive(Clone, Debug)]
pub struct StorageConfig {
    pub initial_logical_shards: u16,
    pub max_segment_events: usize,
    pub blob_externalize_bytes: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            initial_logical_shards: 16,
            max_segment_events: 1_000,
            blob_externalize_bytes: 65_536,
        }
    }
}

pub struct RawStorage {
    root: PathBuf,
    blobs: BlobStore,
    router: ShardRouter,
    segments: SegmentStore,
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
            segments: SegmentStore::open(&root, config.max_segment_events)?,
            root,
        })
    }

    pub fn externalize_event(&self, event: &mut OperationalEventV2) -> Result<()> {
        self.blobs.externalize_event(event)?;
        self.blobs.validate_references(&event.blob_refs)
    }

    pub fn append(&self, stored: &StoredEvent) -> Result<AppendLocation> {
        let route = self.route(&stored.event.event_id, stored.cursor);
        self.segments.append(route, stored)
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
        self.segments.recovered_events()
    }

    pub fn seal_all(&self) -> Result<Vec<SegmentManifest>> {
        self.segments.seal_all()
    }

    pub fn manifests(&self) -> Result<Vec<SegmentManifest>> {
        self.segments.manifests()
    }

    pub fn active_segment_paths(&self) -> Vec<PathBuf> {
        self.segments.active_paths()
    }

    pub fn move_segment(
        &self,
        segment_id: &str,
        destination: impl AsRef<Path>,
    ) -> Result<SegmentManifest> {
        self.segments.move_segment(segment_id, destination.as_ref())
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

    pub fn root(&self) -> &Path {
        &self.root
    }
}
// HANDWRITE-END
