// HANDWRITE-BEGIN gap="sift-gcs-archive-manifest" tracker="1659" reason="Upload immutable Parquet segments and blobs before the commit manifest, then restore only hash-verified objects."
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{bail, Context, Result};
use arrow_array::{Array, ArrayRef, RecordBatch, RecordBatchReader, StringArray, UInt64Array};
use arrow_schema::{DataType, Field, Schema};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzipCompression};
use parquet::{
    arrow::{
        arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder},
        ArrowWriter,
    },
    basic::Compression,
    file::properties::WriterProperties,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ContentBlobRef, SignalKind, StoredEvent};

use super::{
    blob::BlobStore, dedupe::DedupeReceipt, shard, BlobHashSet, DataLayout, EpochMap, RawStorage,
    SegmentManifest, StorageRole,
};

const ARCHIVE_FORMAT_VERSION: u16 = 10;
const ARCHIVE_COMMIT_FORMAT_VERSION: u16 = 3;
const ARCHIVE_COMMIT_PATH: &str = "control/archive-commit.json";
const LOCAL_COMMIT_FORMAT_VERSION: u16 = 2;
const LOCAL_COMMIT_PATH: &str = "control/local-segment-commit.json";
const ARCHIVE_GC_FORMAT_VERSION: u16 = 3;
const ARCHIVE_GC_PATH: &str = "control/archive-gc-pending.json";
const ARCHIVE_GC_STAGED_PATH: &str = "control/archive-gc-staged.json";
const LOCAL_BLOB_GC_PATH: &str = "control/local-blob-gc.json";
const LOCAL_BLOB_GC_COMPLETE_PATH: &str = "control/local-blob-gc-complete.json";
const LOCAL_BLOB_GC_FORMAT_VERSION: u16 = 3;
const ARCHIVE_UPLOAD_INTENT_PATH: &str = "control/archive-upload-intent.json";
const ARCHIVE_UPLOAD_INTENT_FORMAT_VERSION: u16 = 1;
const ARCHIVE_CACHE_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const RESTORE_STATE_PATH: &str = ".sift-restore.json";
const RESTORE_STAGE_DIR: &str = ".sift-restore-stage";
const RESTORE_STATE_FORMAT_VERSION: u16 = 1;
const RETENTION_SCAN_BATCH_SEGMENTS: usize = 64;
const ARCHIVE_SPILL_PREFIXES: [&str; 8] = [
    "retention-obsolete-",
    "retention-live-",
    "archive-updates-",
    "archive-blob-counts-",
    "archive-obsolete-",
    "archive-live-",
    "restore-hot-blobs-",
    "restore-blob-counts-",
];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ArchiveBlob {
    pub reference: ContentBlobRef,
    pub object_uri: String,
    #[serde(default)]
    pub reference_count: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ArchiveSegment {
    pub signal: SignalKind,
    pub source: SegmentManifest,
    pub object_uri: String,
    pub parquet_bytes: u64,
    pub parquet_sha256: String,
    /// Exact acceptance-time bounds let a dedupe rebuild skip cold segments.
    pub min_acknowledged_at_unix_nano: i64,
    pub max_acknowledged_at_unix_nano: i64,
    pub dedupe_receipt: ArchiveDedupeReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveDedupeReceipt {
    pub object_uri: String,
    pub bytes: u64,
    pub sha256: String,
    pub entry_count: u64,
    pub first_cursor: u64,
    pub last_cursor: u64,
    pub min_acknowledged_at_unix_nano: i64,
    pub max_acknowledged_at_unix_nano: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ArchiveManifest {
    pub format_version: u16,
    pub generated_at: String,
    pub source_cluster_id: String,
    pub source_node_id: String,
    pub raft_snapshot_index: u64,
    pub event_count: u64,
    pub event_id_digest_algorithm: String,
    pub event_id_sha256: String,
    pub event_content_digest_algorithm: String,
    pub event_content_sha256: String,
    /// Monotonic epoch for each bounded retention scan commit. Replicas use it
    /// to apply one small source/target delta without downloading the full
    /// cumulative archive.
    pub retention_generation: u64,
    /// Highest cursor made durable in an archive commit. WAL compaction uses
    /// this monotonic coverage even after an event expires.
    pub watermarks: ArchiveWatermarks,
    /// Highest cursor that is still present in each retained signal set.
    pub retained_watermarks: ArchiveWatermarks,
    /// Minimum retained event time. The lifecycle worker uses this fixed root
    /// field instead of scanning the full catalog every minute.
    pub oldest_event_time_unix_nano: Option<i64>,
    pub epochs: Vec<EpochMap>,
    pub catalog_uri: String,
    pub catalog_root: storage_segment::CatalogRoot,
    pub segment_count: u64,
    pub blob_count: u64,
    pub dedupe_receipt_count: u64,
    pub gc_plan_uri: Option<String>,
    pub gc_plan_root: Option<storage_segment::CatalogRoot>,
    pub gc_object_count: u64,
    /// One bounded retention generation can be applied by a caught-up voter
    /// without downloading the cumulative archive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_delta: Option<ArchiveRetentionDelta>,
    /// Durable cursor for a bounded catalog scan. A non-empty value keeps the
    /// same cutoff until every segment entry has been visited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_scan: Option<ArchiveRetentionScan>,
    /// Runtime-only compatibility view. The remote V8 root never embeds the
    /// cumulative segment list.
    #[serde(skip, default)]
    pub segments: Vec<ArchiveSegment>,
    /// Runtime-only compatibility view. The remote V8 root never embeds the
    /// cumulative blob list.
    #[serde(skip, default)]
    pub blobs: Vec<ArchiveBlob>,
    /// Runtime-only compatibility view. The cleanup plan is a separate paged
    /// catalog and only a post-checkpoint leader may execute it.
    #[serde(skip, default)]
    pub gc_object_uris: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveRetentionDelta {
    pub source_manifest_uri: String,
    pub source_manifest_sha256: String,
    pub source_generation: u64,
    pub source_event_count: u64,
    pub source_event_content_sha256: String,
    pub cutoff_unix_nano: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveRetentionScan {
    pub cutoff_unix_nano: i64,
    pub after_catalog_key: String,
    pub oldest_retained_event_time_unix_nano: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ArchiveReceipt {
    pub manifest_uri: String,
    pub manifest_sha256: String,
    pub manifest: ArchiveManifest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ArchiveUploadIntent {
    format_version: u16,
    destination_uri: String,
    archive_prefix: String,
    manifest_uri: String,
    generated_at: String,
    captured_cursor: u64,
    source_cluster_id: String,
    source_manifest_uri: Option<String>,
    source_manifest_sha256: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArchiveWatermarks {
    pub logs: u64,
    pub metrics: u64,
    pub traces: u64,
}

impl ArchiveWatermarks {
    pub(crate) fn through(self, signal: SignalKind) -> u64 {
        match signal {
            SignalKind::Log => self.logs,
            SignalKind::Metric => self.metrics,
            SignalKind::Span => self.traces,
        }
    }

    pub(crate) fn covers(self, signal: SignalKind, cursor: u64) -> bool {
        cursor <= self.through(signal)
    }

    fn include(&mut self, signal: SignalKind, cursor: u64) {
        match signal {
            SignalKind::Log => self.logs = self.logs.max(cursor),
            SignalKind::Metric => self.metrics = self.metrics.max(cursor),
            SignalKind::Span => self.traces = self.traces.max(cursor),
        }
    }

    fn merge(self, other: Self) -> Self {
        Self {
            logs: self.logs.max(other.logs),
            metrics: self.metrics.max(other.metrics),
            traces: self.traces.max(other.traces),
        }
    }

    pub(crate) fn max_cursor(self) -> u64 {
        self.logs.max(self.metrics).max(self.traces)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LocalSegmentCommitState {
    format_version: u16,
    committed_at: String,
    snapshot_index: u64,
    watermarks: ArchiveWatermarks,
    segments: Vec<LocalCommittedSegment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LocalCommittedSegment {
    signal: SignalKind,
    manifest: SegmentManifest,
}

#[derive(Clone, Debug)]
pub struct LocalArchiveReceipt {
    pub committed_at: String,
    pub snapshot_index: u64,
    pub watermarks: ArchiveWatermarks,
    pub event_count: u64,
    pub segment_count: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HotEvictionReceipt {
    pub evicted_segments: usize,
    pub evicted_events: u64,
    pub evicted_through_cursor: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpirationReceipt {
    pub manifest_uri: String,
    pub retained_events: u64,
    pub retained_segments: usize,
    pub expired_events: u64,
    pub replaced_segments: usize,
    pub removed_segments: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ArchiveGcPending {
    format_version: u16,
    replacement_manifest_uri: String,
    replacement_manifest_sha256: String,
    gc_plan_uri: String,
    gc_plan_root: storage_segment::CatalogRoot,
    cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct LocalBlobGcPending {
    format_version: u16,
    replacement_manifest_uri: String,
    replacement_manifest_sha256: String,
    gc_plan_uri: String,
    gc_plan_root: storage_segment::CatalogRoot,
    plan_cursor: Option<String>,
    plan_exhausted: bool,
    candidates: Vec<String>,
    scan_start_cursor: u64,
    scanned_through_cursor: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct LocalBlobGcComplete {
    format_version: u16,
    replacement_manifest_uri: String,
    replacement_manifest_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArchiveCommitState {
    format_version: u16,
    manifest_uri: String,
    manifest_sha256: String,
    committed_at: String,
    watermarks: ArchiveWatermarks,
    manifest: ArchiveManifest,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RestorePhase {
    Building,
    Ready,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RestoreState {
    format_version: u16,
    manifest_uri: String,
    phase: RestorePhase,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SpillBlobReference {
    reference: ContentBlobRef,
    count: u64,
}

#[derive(Clone, Debug)]
pub struct ArchiveCommitStatus {
    pub manifest_uri: String,
    pub manifest_sha256: String,
    pub committed_at: String,
    pub snapshot_index: u64,
    pub watermarks: ArchiveWatermarks,
    pub retention_generation: u64,
    pub retention_scan_pending: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalArchiveStatus {
    pub snapshot_index: u64,
    pub watermarks: ArchiveWatermarks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RemoteRetainedState {
    pub watermarks: ArchiveWatermarks,
    pub event_count: u64,
    pub snapshot_index: u64,
    pub retention_generation: u64,
    pub retention_scan_pending: bool,
    pub event_content_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveReplay {
    pub watermark: u64,
    pub scanned: u64,
    pub replayed: u64,
}

/// Commit a remote archive locally before any canonical WAL bytes are removed.
pub fn archive_journal_gcs(
    journal: &crate::DurableJournal,
    destination_uri: &str,
) -> Result<ArchiveReceipt> {
    let (captured_cursor, segments) = journal.seal_archive_prefix()?;
    archive_journal_gcs_captured(journal, destination_uri, captured_cursor, segments)
}

#[doc(hidden)]
pub fn archive_journal_gcs_captured(
    journal: &crate::DurableJournal,
    destination_uri: &str,
    captured_cursor: u64,
    segments: Vec<(SignalKind, SegmentManifest)>,
) -> Result<ArchiveReceipt> {
    let receipt = archive_gcs_captured(
        journal.storage(),
        destination_uri,
        captured_cursor,
        segments,
    )?;
    stage_archive_gc_pending(journal.storage().root(), &receipt)?;
    let watermarks = record_archive_commit(journal.storage().root(), &receipt)?;
    promote_archive_gc_pending(journal.storage().root(), &receipt)?;
    retire_local_commit(journal.storage().root())?;
    journal.compact_archived_wal(watermarks)?;
    clear_archive_upload_intent(journal.storage().root(), &receipt)?;
    Ok(receipt)
}

/// Retry WAL removal that is already authorized by a durable local or remote
/// commit. This is safe to call after a crash between receipt commit and WAL
/// truncation.
pub(crate) fn reconcile_committed_wal(journal: &crate::DurableJournal) -> Result<()> {
    journal.compact_archived_wal(committed_watermarks(journal.storage().root())?)
}

/// Evict local copies whose complete event set is older than the 30-day hot
/// window and is present in the verified remote manifest.
pub fn evict_committed_cold_segments_at(
    journal: &crate::DurableJournal,
    now: DateTime<Utc>,
) -> Result<HotEvictionReceipt> {
    let manifest = fetch_verified_committed_root(journal.storage().root())?
        .context("cold segment eviction requires a committed remote manifest")?;
    let catalog = catalog_for_uri(&manifest.catalog_uri)?;
    let cutoff = now - chrono::Duration::days(30);
    let cutoff_nanos = cutoff
        .timestamp_nanos_opt()
        .context("30-day hot retention cutoff is outside the nanosecond range")?;
    let mut receipt = HotEvictionReceipt::default();
    for (signal, local) in journal.storage().seal_all_with_signal()? {
        if local.last_cursor > manifest.watermarks.through(signal) {
            // This immutable file also carries a newer Raft suffix. It cannot
            // be evicted until a later compaction splits or archives that
            // suffix.
            continue;
        }
        let events = journal.storage().read_segment_events(signal, &local)?;
        let entirely_cold = if local.max_event_time_unix_nano == 0 {
            events.iter().try_fold(true, |cold, event| {
                let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
                    .context("segment event occurred_at must be RFC3339")?
                    .with_timezone(&Utc);
                anyhow::Ok(cold && occurred < cutoff)
            })?
        } else {
            local.max_event_time_unix_nano < cutoff_nanos
        };
        if !entirely_cold {
            continue;
        }
        // Segment boundaries are local implementation details. A follower can
        // seal the same Raft rows at different points than the leader. Require
        // exact metadata when such a remote segment exists, otherwise rely on
        // the verified manifest coverage and the cold-time check above.
        let portable = portable_manifest(local.clone(), signal);
        let probe = ArchiveSegment {
            signal,
            source: portable.clone(),
            object_uri: String::new(),
            parquet_bytes: 0,
            parquet_sha256: String::new(),
            min_acknowledged_at_unix_nano: 0,
            max_acknowledged_at_unix_nano: 0,
            dedupe_receipt: empty_dedupe_receipt(),
        };
        let exact_remote = catalog
            .lookup(&manifest.catalog_root, &segment_catalog_key(&probe))?
            .map(decode_archive_catalog_entry)
            .transpose()?
            .is_some_and(|item| {
                matches!(item, ArchiveCatalogItem::Segment(segment) if segment.signal == signal && segment.source == portable)
            });
        if !exact_remote
            && events
                .iter()
                .any(|event| event.cursor > manifest.watermarks.through(signal))
        {
            bail!(
                "local segment {} extends beyond committed archive coverage",
                local.segment_id
            );
        }
        if journal
            .storage()
            .evict_segment(signal, &local.segment_id)?
            .is_some()
        {
            receipt.evicted_segments += 1;
            receipt.evicted_events = receipt.evicted_events.saturating_add(local.event_count);
            receipt.evicted_through_cursor = receipt.evicted_through_cursor.max(local.last_cursor);
        }
    }
    journal.evict_resident_before(cutoff)?;
    Ok(receipt)
}

/// Remove events older than the fixed 180-day boundary from the committed
/// manifest. A mixed Parquet segment is rewritten with only retained rows.
/// The new objects and manifest commit before the prior objects are deleted.
pub fn expire_committed_events_at(
    journal: &crate::DurableJournal,
    now: DateTime<Utc>,
) -> Result<ExpirationReceipt> {
    expire_committed_events_bounded_at(journal, now)
}

fn committed_has_expired_dedupe_receipt(
    manifest: &ArchiveManifest,
    cutoff_unix_nano: i64,
) -> Result<bool> {
    if manifest.dedupe_receipt_count == 0 {
        return Ok(false);
    }
    let catalog = catalog_for_uri(&manifest.catalog_uri)?;
    let Some(entry) = catalog
        .reader_after(&manifest.catalog_root, "receipt/")?
        .next()
        .transpose()?
    else {
        bail!("archive manifest counts dedupe receipts but its catalog has none");
    };
    if !entry.key.starts_with("receipt/") {
        bail!("archive receipt count disagrees with its catalog key range");
    }
    let ArchiveCatalogItem::Receipt(receipt) = decode_archive_catalog_entry(entry)? else {
        bail!("archive receipt key resolved to another catalog item");
    };
    validate_dedupe_receipt(&receipt)?;
    Ok(receipt.max_acknowledged_at_unix_nano < cutoff_unix_nano)
}

fn expire_committed_events_bounded_at(
    journal: &crate::DurableJournal,
    now: DateTime<Utc>,
) -> Result<ExpirationReceipt> {
    const BATCH_PARQUET_BYTES: u64 = 64 * 1024 * 1024;

    reconcile_live_committed_retention(journal)?;
    let root = journal.storage().root();
    let current_state = read_commit_state(root)?
        .context("180-day expiration requires a committed remote manifest")?;
    let current = fetch_verified_committed_root(root)?
        .context("180-day expiration requires a readable remote manifest")?;
    if journal.last_cursor() < current.raft_snapshot_index {
        bail!(
            "180-day expiration journal cursor {} is behind archive cursor {}",
            journal.last_cursor(),
            current.raft_snapshot_index
        );
    }
    let suffix_events = journal.last_cursor() - current.raft_snapshot_index;
    let expected_local_events = current
        .event_count
        .checked_add(suffix_events)
        .context("retained archive and suffix event count exhausted u64")?;
    if journal.total_event_count() != expected_local_events
        || journal.recovery_required()
        || journal.retention_generation() != current.retention_generation
    {
        bail!("Sift journal must reconcile its committed retention head before advancing it");
    }

    let requested_cutoff_nanos = (now - chrono::Duration::days(180))
        .timestamp_nanos_opt()
        .context("180-day retention cutoff is outside the nanosecond range")?;
    let receipt_cutoff_nanos = (now
        - chrono::Duration::seconds(super::dedupe::IDEMPOTENCY_WINDOW_SECONDS))
    .timestamp_nanos_opt()
    .context("dedupe receipt cleanup cutoff is outside the nanosecond range")?;
    if current.retention_scan.is_none()
        && current
            .oldest_event_time_unix_nano
            .is_none_or(|oldest| oldest >= requested_cutoff_nanos)
        && !committed_has_expired_dedupe_receipt(&current, receipt_cutoff_nanos)?
    {
        return Ok(ExpirationReceipt {
            manifest_uri: current_state.manifest_uri,
            retained_events: current.event_count,
            retained_segments: usize::try_from(current.segment_count).unwrap_or(usize::MAX),
            expired_events: 0,
            replaced_segments: 0,
            removed_segments: 0,
        });
    }

    let mut scan = current
        .retention_scan
        .clone()
        .unwrap_or(ArchiveRetentionScan {
            cutoff_unix_nano: requested_cutoff_nanos,
            after_catalog_key: "segment/".to_string(),
            oldest_retained_event_time_unix_nano: None,
        });
    let cutoff = DateTime::<Utc>::from_timestamp_nanos(scan.cutoff_unix_nano);
    let (bucket, current_manifest_key) = split_gcs_uri(&current_state.manifest_uri)?;
    let object_store: Arc<dyn storage_object::ObjectStore> =
        Arc::new(storage_object::GcsObjectStore::new(&bucket, "")?);
    let parent = current_manifest_key
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("sift");
    let target_generation = current.retention_generation.saturating_add(1);
    let rewrite_prefix = format!(
        "{parent}/retention-g{target_generation:020}-{}",
        &current_state.manifest_sha256[..16]
    );
    let manifest_key = format!("{rewrite_prefix}/manifest.json");
    match object_store.get(&manifest_key) {
        Ok(object) => {
            let manifest: ArchiveManifest = serde_json::from_slice(&object.bytes)
                .context("decode an uploaded bounded retention manifest")?;
            validate_archive_manifest(&manifest)?;
            let receipt = ArchiveReceipt {
                manifest_uri: gcs_uri(&bucket, &manifest_key),
                manifest_sha256: sha256(&object.bytes),
                manifest,
            };
            validate_retention_receipt_source(&receipt, &current_state)?;
            return finish_bounded_retention_commit(journal, &current, receipt);
        }
        Err(storage_object::ObjectStoreError::NotFound { .. }) => {}
        Err(error) => return Err(error.into()),
    }

    let current_catalog = catalog_for_uri(&current.catalog_uri)?;
    let mut catalog_root = current.catalog_root.clone();
    let mut reader =
        current_catalog.reader_after(&current.catalog_root, &scan.after_catalog_key)?;
    let spill_parent = root.join("tmp");
    let mut gc_candidates = SpillCatalog::new(&spill_parent, "retention-obsolete-")?;
    let mut live_objects = SpillCatalog::new(&spill_parent, "retention-live-")?;
    spill_pending_archive_gc(root, &mut gc_candidates)?;
    gc_candidates.insert_uri(&current_state.manifest_uri)?;
    if let (Some(gc_uri), Some(gc_root)) = (&current.gc_plan_uri, &current.gc_plan_root) {
        let (gc_bucket, _) = split_gcs_uri(gc_uri)?;
        let current_gc = catalog_for_uri(gc_uri)?;
        for key in current_gc.page_keys(gc_root)? {
            gc_candidates.insert_uri(&gcs_uri(&gc_bucket, &key?))?;
        }
    }

    let coordinator = storage_segment::ArchiveCoordinator::new(object_store.clone());
    let mut transaction = coordinator.begin();
    let mut event_count = current.event_count;
    let mut event_id_digest = decode_event_id_digest(&current.event_id_sha256)?;
    let mut event_content_digest = decode_event_content_digest(&current.event_content_sha256)?;
    let mut segment_count = current.segment_count;
    let mut blob_count = current.blob_count;
    let mut dedupe_receipt_count = current.dedupe_receipt_count;
    let mut blob_decrements = BTreeMap::<String, u64>::new();
    let mut expired_events = 0_u64;
    let mut replaced_segments = 0_usize;
    let mut removed_segments = 0_usize;
    let mut processed_segments = 0_usize;
    let mut fetched_parquet_bytes = 0_u64;
    let mut scan_complete = false;

    while processed_segments < RETENTION_SCAN_BATCH_SEGMENTS {
        let Some(entry) = reader.next() else {
            scan_complete = true;
            break;
        };
        let entry = entry?;
        let entry_key = entry.key.clone();
        let ArchiveCatalogItem::Segment(segment) = decode_archive_catalog_entry(entry)? else {
            scan.after_catalog_key = entry_key;
            continue;
        };
        let might_expire = segment.source.min_event_time_unix_nano < scan.cutoff_unix_nano;
        if might_expire
            && processed_segments > 0
            && fetched_parquet_bytes.saturating_add(segment.parquet_bytes) > BATCH_PARQUET_BYTES
        {
            break;
        }
        processed_segments += 1;
        scan.after_catalog_key = entry_key.clone();
        if !might_expire {
            include_retention_oldest(
                &mut scan.oldest_retained_event_time_unix_nano,
                segment.source.min_event_time_unix_nano,
            );
            continue;
        }

        fetched_parquet_bytes = fetched_parquet_bytes.saturating_add(segment.parquet_bytes);
        let bytes = cached_segment_bytes(root, &segment)?;
        let events = decode_parquet(&bytes)?;
        verify_archive_segment(&segment, &events)?;
        let mut retained = Vec::with_capacity(events.len());
        let mut segment_expired = 0_u64;
        let receipt_cutoff_nanos = (now
            - chrono::Duration::seconds(super::dedupe::IDEMPOTENCY_WINDOW_SECONDS))
        .timestamp_nanos_opt()
        .context("dedupe receipt retention cutoff is outside the nanosecond range")?;
        let mut has_active_expired_receipt = false;
        for event in events {
            let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
                .context("archive event occurred_at must be RFC3339")?
                .with_timezone(&Utc);
            if occurred < cutoff {
                segment_expired = segment_expired.saturating_add(1);
                has_active_expired_receipt |=
                    acknowledgement_time_unix_nano(&event)? >= receipt_cutoff_nanos;
                event_count = event_count
                    .checked_sub(1)
                    .context("expired archive event count underflow")?;
                include_event_id(&mut event_id_digest, &event.event.event_id);
                include_event_content(&mut event_content_digest, &event)?;
                for reference in &event.event.blob_refs {
                    *blob_decrements.entry(reference.hash.clone()).or_default() += 1;
                }
            } else {
                retained.push(event);
            }
        }
        if segment_expired == 0 {
            include_retention_oldest(
                &mut scan.oldest_retained_event_time_unix_nano,
                segment.source.min_event_time_unix_nano,
            );
            continue;
        }
        expired_events = expired_events.saturating_add(segment_expired);
        apply_catalog_remove(
            &current_catalog,
            &mut catalog_root,
            &entry_key,
            &bucket,
            &mut gc_candidates,
            &mut live_objects,
        )?;
        gc_candidates.insert_uri(&segment.object_uri)?;
        gc_candidates.insert_uri(&segment.dedupe_receipt.object_uri)?;
        segment_count = segment_count
            .checked_sub(1)
            .context("retained segment count underflow")?;
        if has_active_expired_receipt {
            fetch_dedupe_receipts(&segment.dedupe_receipt)
                .context("verify active dedupe receipt before retaining it")?;
            apply_catalog_upsert(
                &current_catalog,
                &mut catalog_root,
                storage_segment::CatalogEntry {
                    key: dedupe_receipt_catalog_key(&segment.dedupe_receipt),
                    value: serde_json::to_vec(&segment.dedupe_receipt)?,
                },
                &bucket,
                &mut gc_candidates,
                &mut live_objects,
            )?;
            live_objects.insert_uri(&segment.dedupe_receipt.object_uri)?;
            dedupe_receipt_count = dedupe_receipt_count.saturating_add(1);
        }
        if retained.is_empty() {
            removed_segments += 1;
            continue;
        }

        replaced_segments += 1;
        let parquet = encode_parquet(&retained)?;
        let parquet_sha256 = sha256(&parquet);
        let key = format!(
            "{rewrite_prefix}/segments/{}/{}.parquet",
            segment.signal, parquet_sha256
        );
        transaction.put(storage_segment::ArchiveObject::new(
            key.clone(),
            parquet.clone(),
            "application/vnd.apache.parquet",
        ))?;
        let mut source = segment.source.clone();
        source.segment_id = format!("retained-{}", &parquet_sha256[..32]);
        source.first_cursor = retained.first().expect("retained segment").cursor;
        source.last_cursor = retained.last().expect("retained segment").cursor;
        source.event_count = retained.len() as u64;
        source.bytes = parquet.len() as u64;
        source.sha256 = parquet_sha256.clone();
        source.local_path = PathBuf::from(format!(
            "segments/{}/{}.framed",
            segment.signal, source.segment_id
        ));
        source.object_uri = None;
        let mut event_times = retained.iter().map(event_time_unix_nano);
        let first_event_time = event_times
            .next()
            .transpose()?
            .expect("retained segment is non-empty");
        let (minimum, maximum) = event_times.try_fold(
            (first_event_time, first_event_time),
            |(minimum, maximum), value| {
                let value = value?;
                anyhow::Ok((minimum.min(value), maximum.max(value)))
            },
        )?;
        source.min_event_time_unix_nano = minimum;
        source.max_event_time_unix_nano = maximum;
        let (minimum_acknowledged_at, maximum_acknowledged_at) =
            acknowledgement_time_bounds(&retained)?;
        let dedupe_receipt =
            archive_dedupe_receipts(&mut transaction, &bucket, &rewrite_prefix, &retained)?;
        let replacement = ArchiveSegment {
            signal: segment.signal,
            source,
            object_uri: gcs_uri(&bucket, &key),
            parquet_bytes: parquet.len() as u64,
            parquet_sha256,
            min_acknowledged_at_unix_nano: minimum_acknowledged_at,
            max_acknowledged_at_unix_nano: maximum_acknowledged_at,
            dedupe_receipt,
        };
        apply_catalog_upsert(
            &current_catalog,
            &mut catalog_root,
            storage_segment::CatalogEntry {
                key: segment_catalog_key(&replacement),
                value: serde_json::to_vec(&replacement)?,
            },
            &bucket,
            &mut gc_candidates,
            &mut live_objects,
        )?;
        live_objects.insert_uri(&replacement.object_uri)?;
        live_objects.insert_uri(&replacement.dedupe_receipt.object_uri)?;
        segment_count = segment_count.saturating_add(1);
        include_retention_oldest(
            &mut scan.oldest_retained_event_time_unix_nano,
            replacement.source.min_event_time_unix_nano,
        );
    }

    for (hash, decrement) in blob_decrements {
        let key = format!("blob/{hash}");
        let entry = current_catalog
            .lookup(&catalog_root, &key)?
            .context("expired event references a blob absent from the archive catalog")?;
        let ArchiveCatalogItem::Blob(mut blob) = decode_archive_catalog_entry(entry)? else {
            bail!("archive blob key resolved to a segment");
        };
        if decrement > blob.reference_count {
            bail!("archive blob reference count underflow");
        }
        apply_catalog_remove(
            &current_catalog,
            &mut catalog_root,
            &key,
            &bucket,
            &mut gc_candidates,
            &mut live_objects,
        )?;
        blob.reference_count -= decrement;
        if blob.reference_count == 0 {
            blob_count = blob_count
                .checked_sub(1)
                .context("retained blob count underflow")?;
            gc_candidates.insert_uri(&blob.object_uri)?;
        } else {
            apply_catalog_upsert(
                &current_catalog,
                &mut catalog_root,
                storage_segment::CatalogEntry {
                    key: blob_catalog_key(&blob),
                    value: serde_json::to_vec(&blob)?,
                },
                &bucket,
                &mut gc_candidates,
                &mut live_objects,
            )?;
            live_objects.insert_uri(&blob.object_uri)?;
        }
    }

    let mut receipt_reader = current_catalog.reader_after(&catalog_root, "receipt/")?;
    let mut removed_receipts = 0_usize;
    while removed_receipts < RETENTION_SCAN_BATCH_SEGMENTS {
        let Some(entry) = receipt_reader.next() else {
            break;
        };
        let entry = entry?;
        if !entry.key.starts_with("receipt/") {
            break;
        }
        let ArchiveCatalogItem::Receipt(receipt) = decode_archive_catalog_entry(entry.clone())?
        else {
            bail!("archive receipt key resolved to another catalog item");
        };
        validate_dedupe_receipt(&receipt)?;
        if receipt.max_acknowledged_at_unix_nano >= receipt_cutoff_nanos {
            break;
        }
        apply_catalog_remove(
            &current_catalog,
            &mut catalog_root,
            &entry.key,
            &bucket,
            &mut gc_candidates,
            &mut live_objects,
        )?;
        gc_candidates.insert_uri(&receipt.object_uri)?;
        dedupe_receipt_count = dedupe_receipt_count
            .checked_sub(1)
            .context("archive dedupe receipt count underflow")?;
        removed_receipts += 1;
    }

    if catalog_root.entry_count
        != segment_count
            .saturating_add(blob_count)
            .saturating_add(dedupe_receipt_count)
    {
        bail!("bounded retention catalog count disagrees with its manifest totals");
    }
    let retained_watermarks = retained_watermarks_from_catalog(&current_catalog, &catalog_root)?;
    let retention_scan = if scan_complete {
        if scan
            .oldest_retained_event_time_unix_nano
            .is_some_and(|oldest| oldest < scan.cutoff_unix_nano)
        {
            bail!("bounded retention completed but retained an expired segment minimum");
        }
        None
    } else {
        Some(scan.clone())
    };
    let oldest_event_time_unix_nano = retention_scan
        .as_ref()
        .and(current.oldest_event_time_unix_nano)
        .or(scan.oldest_retained_event_time_unix_nano);

    let gc_prefix = format!("{rewrite_prefix}/gc-plan");
    let gc_plan = build_gc_catalog_from_spills(
        object_store,
        &gc_prefix,
        &gc_candidates,
        &live_objects,
        &spill_parent,
    )?;
    let gc_object_count = gc_plan
        .as_ref()
        .map(|plan| plan.root.entry_count)
        .unwrap_or_default();
    let manifest = ArchiveManifest {
        format_version: ARCHIVE_FORMAT_VERSION,
        generated_at: now.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        source_cluster_id: current.source_cluster_id.clone(),
        source_node_id: current.source_node_id.clone(),
        raft_snapshot_index: current.raft_snapshot_index,
        event_count,
        event_id_digest_algorithm: "xor-sha256-v1".to_string(),
        event_id_sha256: hex::encode(event_id_digest),
        event_content_digest_algorithm: "xor-sha256-v1".to_string(),
        event_content_sha256: hex::encode(event_content_digest),
        retention_generation: target_generation,
        watermarks: current.watermarks,
        retained_watermarks,
        oldest_event_time_unix_nano,
        epochs: current.epochs.clone(),
        catalog_uri: current.catalog_uri.clone(),
        catalog_root,
        segment_count,
        blob_count,
        dedupe_receipt_count,
        gc_plan_uri: gc_plan.as_ref().map(|_| gcs_uri(&bucket, &gc_prefix)),
        gc_plan_root: gc_plan.map(|plan| plan.root),
        gc_object_count,
        retention_delta: Some(ArchiveRetentionDelta {
            source_manifest_uri: current_state.manifest_uri.clone(),
            source_manifest_sha256: current_state.manifest_sha256.clone(),
            source_generation: current.retention_generation,
            source_event_count: current.event_count,
            source_event_content_sha256: current.event_content_sha256.clone(),
            cutoff_unix_nano: scan.cutoff_unix_nano,
        }),
        retention_scan,
        segments: Vec::new(),
        blobs: Vec::new(),
        gc_object_uris: Vec::new(),
    };
    validate_archive_manifest(&manifest)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let commit = transaction.commit(storage_segment::ArchiveObject::new(
        manifest_key.clone(),
        manifest_bytes,
        "application/json",
    ))?;
    let receipt = ArchiveReceipt {
        manifest_uri: gcs_uri(&bucket, &manifest_key),
        manifest_sha256: commit.manifest.sha256,
        manifest,
    };
    finish_bounded_retention_commit(journal, &current, receipt).map(|mut outcome| {
        outcome.expired_events = expired_events;
        outcome.replaced_segments = replaced_segments;
        outcome.removed_segments = removed_segments;
        outcome
    })
}

fn include_retention_oldest(oldest: &mut Option<i64>, candidate: i64) {
    *oldest = Some(
        oldest
            .map(|value| value.min(candidate))
            .unwrap_or(candidate),
    );
}

fn apply_catalog_remove(
    catalog: &storage_segment::PagedCatalog,
    root: &mut storage_segment::CatalogRoot,
    key: &str,
    bucket: &str,
    gc_candidates: &mut SpillCatalog,
    live_objects: &mut SpillCatalog,
) -> Result<()> {
    let mutation = catalog.remove(root, key)?;
    for obsolete in &mutation.obsolete_page_keys {
        let uri = gcs_uri(bucket, obsolete);
        gc_candidates.insert_uri(&uri)?;
        live_objects.remove_uri(&uri)?;
    }
    for written in &mutation.written_page_keys {
        live_objects.insert_uri(&gcs_uri(bucket, written))?;
    }
    *root = mutation.root;
    Ok(())
}

fn apply_catalog_upsert(
    catalog: &storage_segment::PagedCatalog,
    root: &mut storage_segment::CatalogRoot,
    entry: storage_segment::CatalogEntry,
    bucket: &str,
    gc_candidates: &mut SpillCatalog,
    live_objects: &mut SpillCatalog,
) -> Result<()> {
    let mutation = catalog.upsert(root, entry)?;
    for obsolete in &mutation.obsolete_page_keys {
        let uri = gcs_uri(bucket, obsolete);
        gc_candidates.insert_uri(&uri)?;
        live_objects.remove_uri(&uri)?;
    }
    for written in &mutation.written_page_keys {
        live_objects.insert_uri(&gcs_uri(bucket, written))?;
    }
    *root = mutation.root;
    Ok(())
}

fn retained_watermarks_from_catalog(
    catalog: &storage_segment::PagedCatalog,
    root: &storage_segment::CatalogRoot,
) -> Result<ArchiveWatermarks> {
    let mut watermarks = ArchiveWatermarks::default();
    for signal in SignalKind::ALL {
        let prefix = format!("segment/{signal}/");
        let Some(entry) = catalog.last_with_prefix(root, &prefix)? else {
            continue;
        };
        let ArchiveCatalogItem::Segment(segment) = decode_archive_catalog_entry(entry)? else {
            bail!("archive signal tail resolved to a blob");
        };
        if segment.signal != signal {
            bail!("archive signal tail resolved to another signal");
        }
        watermarks.include(signal, segment.source.last_cursor);
    }
    Ok(watermarks)
}

fn validate_retention_receipt_source(
    receipt: &ArchiveReceipt,
    source: &ArchiveCommitState,
) -> Result<()> {
    let delta = receipt
        .manifest
        .retention_delta
        .as_ref()
        .context("bounded retention manifest is missing its source delta")?;
    if delta.source_manifest_uri != source.manifest_uri
        || delta.source_manifest_sha256 != source.manifest_sha256
        || delta.source_generation != source.manifest.retention_generation
        || delta.source_event_count != source.manifest.event_count
        || delta.source_event_content_sha256 != source.manifest.event_content_sha256
        || receipt.manifest.retention_generation != delta.source_generation.saturating_add(1)
        || receipt.manifest.raft_snapshot_index != source.manifest.raft_snapshot_index
        || receipt.manifest.watermarks != source.manifest.watermarks
    {
        bail!("bounded retention manifest source identity changed");
    }
    Ok(())
}

fn finish_bounded_retention_commit(
    journal: &crate::DurableJournal,
    source: &ArchiveManifest,
    receipt: ArchiveReceipt,
) -> Result<ExpirationReceipt> {
    let delta = receipt
        .manifest
        .retention_delta
        .clone()
        .context("bounded retention receipt is missing its delta")?;
    stage_archive_gc_pending(journal.storage().root(), &receipt)?;
    journal.mark_recovery_required();
    record_archive_commit(journal.storage().root(), &receipt)?;
    promote_archive_gc_pending(journal.storage().root(), &receipt)?;
    let cutoff = DateTime::<Utc>::from_timestamp_nanos(delta.cutoff_unix_nano);
    journal
        .storage()
        .evict_expired_before(cutoff, receipt.manifest.raft_snapshot_index)?;
    journal.apply_expiration_head(
        cutoff,
        delta.source_event_count,
        receipt.manifest.event_count,
        decode_event_content_digest(&delta.source_event_content_sha256)?,
        decode_event_content_digest(&receipt.manifest.event_content_sha256)?,
        false,
    )?;
    resume_local_blob_gc_batch(journal, 128, 1_280_000)?;
    evict_committed_cold_segments_at(journal, cutoff + chrono::Duration::days(180))?;
    Ok(ExpirationReceipt {
        manifest_uri: receipt.manifest_uri,
        retained_events: receipt.manifest.event_count,
        retained_segments: usize::try_from(receipt.manifest.segment_count).unwrap_or(usize::MAX),
        expired_events: source
            .event_count
            .saturating_sub(receipt.manifest.event_count),
        replaced_segments: 0,
        removed_segments: 0,
    })
}

fn spill_pending_archive_gc(root: &Path, target: &mut SpillCatalog) -> Result<()> {
    reconcile_staged_archive_gc(root)?;
    let path = root.join(ARCHIVE_GC_PATH);
    if !path.exists() {
        return Ok(());
    }
    let pending = read_archive_gc_pending(&path)?;
    let committed = read_commit_state(root)?;
    if committed.as_ref().is_none_or(|committed| {
        committed.manifest_uri != pending.replacement_manifest_uri
            || committed.manifest_sha256 != pending.replacement_manifest_sha256
    }) {
        remove_archive_gc_pending(&path)?;
        return Ok(());
    }
    let catalog = catalog_for_uri(&pending.gc_plan_uri)?;
    let mut reader = match pending.cursor.as_deref() {
        Some(cursor) => catalog.reader_after(&pending.gc_plan_root, cursor)?,
        None => catalog.reader(&pending.gc_plan_root)?,
    };
    for entry in &mut reader {
        let entry = entry?;
        let uri = String::from_utf8(entry.value).context("decode pending archive GC URI")?;
        service_backup::GcsSink::from_exact_uri(&uri)
            .with_context(|| format!("validate pending archive GC object URI {uri}"))?;
        target.insert_uri(&uri)?;
    }
    Ok(())
}

struct FilteredGcEntries<'a> {
    candidates: storage_segment::CatalogReader,
    live: &'a SpillCatalog,
}

impl Iterator for FilteredGcEntries<'_> {
    type Item = storage_segment::Result<storage_segment::CatalogEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = self.candidates.next()?;
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => return Some(Err(error)),
            };
            let uri = match String::from_utf8(entry.value) {
                Ok(uri) => uri,
                Err(error) => {
                    return Some(Err(storage_segment::SegmentError::Serialization {
                        message: error.to_string(),
                    }))
                }
            };
            match self.live.contains_uri(&uri) {
                Ok(true) => continue,
                Ok(false) => {
                    return Some(Ok(storage_segment::CatalogEntry {
                        key: format!("gc/{}", sha256(uri.as_bytes())),
                        value: uri.into_bytes(),
                    }))
                }
                Err(error) => {
                    return Some(Err(storage_segment::SegmentError::Serialization {
                        message: error.to_string(),
                    }))
                }
            }
        }
    }
}

fn build_gc_catalog_from_spills(
    store: Arc<dyn storage_object::ObjectStore>,
    prefix: &str,
    candidates: &SpillCatalog,
    live: &SpillCatalog,
    spill_parent: &Path,
) -> Result<Option<storage_segment::StreamingCatalogBuild>> {
    let mut entries = FilteredGcEntries {
        candidates: candidates.reader()?,
        live,
    };
    let Some(first) = entries.next() else {
        return Ok(None);
    };
    let first = first?;
    let catalog = storage_segment::PagedCatalog::new(store.clone(), prefix)?;
    let mut written = SpillCatalog::new(spill_parent, "gc-catalog-written-")?;
    match catalog.build_sorted_observed(std::iter::once(Ok(first)).chain(entries), |reference| {
        record_catalog_page(&mut written, reference)
    }) {
        Ok(build) => Ok(Some(build)),
        Err(error) => {
            cleanup_observed_catalog_pages(store.as_ref(), &written)?;
            Err(error.into())
        }
    }
}

fn record_catalog_page(
    ledger: &mut SpillCatalog,
    reference: &storage_segment::CatalogPageRef,
) -> storage_segment::Result<()> {
    ledger
        .upsert(
            format!("page/{}", sha256(reference.key.as_bytes())),
            reference.key.as_bytes().to_vec(),
        )
        .map_err(|error| storage_segment::SegmentError::Serialization {
            message: format!("persist catalog cleanup ledger: {error:#}"),
        })
}

fn cleanup_observed_catalog_pages(
    store: &dyn storage_object::ObjectStore,
    ledger: &SpillCatalog,
) -> Result<()> {
    for entry in ledger.reader()? {
        let key = String::from_utf8(entry?.value).context("decode catalog cleanup ledger key")?;
        store
            .delete(&key)
            .with_context(|| format!("remove page from aborted catalog build {key}"))?;
    }
    Ok(())
}

#[doc(hidden)]
pub fn retention_due_at(root: &Path, now: DateTime<Utc>) -> Result<bool> {
    let Some(manifest) = fetch_verified_committed_root(root)? else {
        return Ok(false);
    };
    if manifest.retention_scan.is_some() {
        return Ok(true);
    }
    let cutoff_nanos = (now - chrono::Duration::days(180))
        .timestamp_nanos_opt()
        .context("180-day retention cutoff is outside the nanosecond range")?;
    if manifest
        .oldest_event_time_unix_nano
        .is_some_and(|oldest| oldest < cutoff_nanos)
    {
        return Ok(true);
    }
    let receipt_cutoff_nanos = (now
        - chrono::Duration::seconds(super::dedupe::IDEMPOTENCY_WINDOW_SECONDS))
    .timestamp_nanos_opt()
    .context("dedupe receipt cleanup cutoff is outside the nanosecond range")?;
    committed_has_expired_dedupe_receipt(&manifest, receipt_cutoff_nanos)
}

/// Seal local immutable segments and commit their exact manifest set before
/// compacting the corresponding WAL. This is the durable fallback for local
/// installations without GCS.
pub fn archive_journal_local(journal: &crate::DurableJournal) -> Result<LocalArchiveReceipt> {
    let (snapshot_index, segments) = journal.seal_archive_prefix()?;
    archive_journal_local_captured(journal, snapshot_index, segments)
}

#[doc(hidden)]
pub fn archive_journal_local_captured(
    journal: &crate::DurableJournal,
    snapshot_index: u64,
    segments: Vec<(SignalKind, SegmentManifest)>,
) -> Result<LocalArchiveReceipt> {
    let mut watermarks = ArchiveWatermarks::default();
    let mut event_count = 0_u64;
    let mut cursors = BTreeSet::new();
    let mut committed = Vec::with_capacity(segments.len());
    for (signal, manifest) in segments {
        let events = journal.storage().read_segment_events(signal, &manifest)?;
        event_count = event_count.saturating_add(events.len() as u64);
        for event in &events {
            if !cursors.insert(event.cursor) {
                bail!("local archive contains duplicate cursor {}", event.cursor);
            }
        }
        watermarks.include(signal, manifest.last_cursor);
        committed.push(LocalCommittedSegment { signal, manifest });
    }
    for (offset, cursor) in cursors.into_iter().enumerate() {
        let expected = offset as u64 + 1;
        if cursor != expected {
            bail!("local archive prefix expected cursor {expected}, found {cursor}");
        }
    }
    if event_count != snapshot_index {
        bail!(
            "local archive contains {event_count} events but captured cursor is {snapshot_index}"
        );
    }
    committed.sort_by_key(|segment| segment.manifest.first_cursor);
    let committed_at = Utc::now().to_rfc3339();
    let state = LocalSegmentCommitState {
        format_version: LOCAL_COMMIT_FORMAT_VERSION,
        committed_at: committed_at.clone(),
        snapshot_index,
        watermarks,
        segments: committed,
    };
    let path = journal.storage().root().join(LOCAL_COMMIT_PATH);
    storage_durable::atomic_write(
        &path,
        &serde_json::to_vec_pretty(&state)?,
        storage_durable::FsyncPolicy::Always,
    )?;
    set_private_file(&path)?;
    journal.compact_archived_wal(watermarks)?;
    Ok(LocalArchiveReceipt {
        committed_at,
        snapshot_index,
        watermarks,
        event_count,
        segment_count: state.segments.len(),
    })
}

/// Upload every immutable object first. The manifest upload is the commit point.
pub fn archive_gcs(storage: &RawStorage, destination_uri: &str) -> Result<ArchiveReceipt> {
    let segments = storage.seal_all_with_signal()?;
    let captured_cursor = segments
        .iter()
        .map(|(_, manifest)| manifest.last_cursor)
        .max()
        .unwrap_or_default();
    archive_gcs_captured(storage, destination_uri, captured_cursor, segments)
}

fn archive_gcs_captured(
    storage: &RawStorage,
    destination_uri: &str,
    captured_cursor: u64,
    captured_segments: Vec<(SignalKind, SegmentManifest)>,
) -> Result<ArchiveReceipt> {
    archive_gcs_captured_streaming(storage, destination_uri, captured_cursor, captured_segments)
}

fn verify_local_archive_prefix(
    storage: &RawStorage,
    previously_covered: u64,
    captured_cursor: u64,
) -> Result<()> {
    let mut after = previously_covered;
    while after < captured_cursor {
        let page = storage.query_events(None, after, 10_000)?;
        let mut progressed = false;
        for event in page {
            if event.cursor > captured_cursor {
                break;
            }
            let expected = after
                .checked_add(1)
                .context("archive cursor exhausted u64")?;
            if event.cursor != expected {
                bail!(
                    "archive prefix is not contiguous: expected cursor {expected}, found {}",
                    event.cursor
                );
            }
            after = event.cursor;
            progressed = true;
        }
        if !progressed {
            bail!(
                "archive prefix ended at cursor {after}, before captured cursor {captured_cursor}"
            );
        }
    }
    Ok(())
}

fn prepare_archive_upload_intent(
    root: &Path,
    destination_uri: &str,
    destination_prefix: &str,
    cluster_id: &str,
    captured_cursor: u64,
    previous: Option<&ArchiveCommitState>,
) -> Result<ArchiveUploadIntent> {
    let path = root.join(ARCHIVE_UPLOAD_INTENT_PATH);
    if path.exists() {
        let intent = read_archive_upload_intent(&path)?;
        if previous.is_some_and(|state| {
            state.manifest_uri == intent.manifest_uri
                && state.manifest.raft_snapshot_index == intent.captured_cursor
        }) {
            remove_archive_upload_intent(&path)?;
        } else {
            let previous_uri = previous.map(|state| state.manifest_uri.as_str());
            let previous_hash = previous.map(|state| state.manifest_sha256.as_str());
            if intent.destination_uri != destination_uri
                || intent.source_cluster_id != cluster_id
                || intent.source_manifest_uri.as_deref() != previous_uri
                || intent.source_manifest_sha256.as_deref() != previous_hash
                || captured_cursor < intent.captured_cursor
            {
                bail!("unfinished archive upload intent does not match the captured prefix");
            }
            return Ok(intent);
        }
    }

    let source_token = previous
        .map(|state| state.manifest_sha256[..16].to_string())
        .unwrap_or_else(|| "root".to_string());
    let archive_id = format!(
        "{}-c{captured_cursor:020}-s{source_token}",
        &sha256(cluster_id.as_bytes())[..16]
    );
    let archive_prefix = format!("{destination_prefix}/archives/{archive_id}");
    let intent = ArchiveUploadIntent {
        format_version: ARCHIVE_UPLOAD_INTENT_FORMAT_VERSION,
        destination_uri: destination_uri.to_string(),
        manifest_uri: format!("{destination_uri}/archives/{archive_id}/manifest.json"),
        archive_prefix,
        generated_at: Utc::now().to_rfc3339(),
        captured_cursor,
        source_cluster_id: cluster_id.to_string(),
        source_manifest_uri: previous.map(|state| state.manifest_uri.clone()),
        source_manifest_sha256: previous.map(|state| state.manifest_sha256.clone()),
    };
    persist_archive_upload_intent(&path, &intent)?;
    Ok(intent)
}

fn recover_uploaded_archive(
    store: &dyn storage_object::ObjectStore,
    intent: &ArchiveUploadIntent,
) -> Result<Option<ArchiveReceipt>> {
    let (_, manifest_key) = split_gcs_uri(&intent.manifest_uri)?;
    let object = match store.get(&manifest_key) {
        Ok(object) => object,
        Err(storage_object::ObjectStoreError::NotFound { .. }) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let manifest_sha256 = sha256(&object.bytes);
    let manifest: ArchiveManifest = serde_json::from_slice(&object.bytes)
        .context("decode manifest recovered from archive upload intent")?;
    validate_archive_manifest(&manifest)?;
    if manifest.source_cluster_id != intent.source_cluster_id
        || manifest.raft_snapshot_index != intent.captured_cursor
        || manifest.generated_at != intent.generated_at
    {
        bail!("uploaded archive manifest does not match its durable intent");
    }
    Ok(Some(ArchiveReceipt {
        manifest_uri: intent.manifest_uri.clone(),
        manifest_sha256,
        manifest,
    }))
}

fn read_archive_upload_intent(path: &Path) -> Result<ArchiveUploadIntent> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("archive upload intent is not a regular file");
    }
    let intent: ArchiveUploadIntent =
        serde_json::from_slice(&std::fs::read(path)?).context("decode archive upload intent")?;
    validate_archive_upload_intent(&intent)?;
    Ok(intent)
}

fn validate_archive_upload_intent(intent: &ArchiveUploadIntent) -> Result<()> {
    if intent.format_version != ARCHIVE_UPLOAD_INTENT_FORMAT_VERSION
        || !intent.destination_uri.starts_with("gs://")
        || !intent.manifest_uri.starts_with(&intent.destination_uri)
        || intent.archive_prefix.is_empty()
        || intent.generated_at.is_empty()
        || intent.captured_cursor == 0
        || intent.source_cluster_id.is_empty()
        || intent.source_manifest_uri.is_some() != intent.source_manifest_sha256.is_some()
        || intent.source_manifest_sha256.as_ref().is_some_and(|hash| {
            hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
    {
        bail!("archive upload intent has invalid identity fields");
    }
    Ok(())
}

fn persist_archive_upload_intent(path: &Path, intent: &ArchiveUploadIntent) -> Result<()> {
    validate_archive_upload_intent(intent)?;
    storage_durable::atomic_write(
        path,
        &serde_json::to_vec_pretty(intent)?,
        storage_durable::FsyncPolicy::Always,
    )?;
    set_private_file(path)
}

fn remove_archive_upload_intent(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => storage_durable::sync_parent_dir(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn clear_archive_upload_intent(root: &Path, receipt: &ArchiveReceipt) -> Result<()> {
    let path = root.join(ARCHIVE_UPLOAD_INTENT_PATH);
    if !path.exists() {
        return Ok(());
    }
    let intent = read_archive_upload_intent(&path)?;
    if intent.manifest_uri != receipt.manifest_uri
        || intent.captured_cursor != receipt.manifest.raft_snapshot_index
    {
        bail!("archive commit does not match its durable upload intent");
    }
    remove_archive_upload_intent(&path)
}

fn archive_gcs_captured_streaming(
    storage: &RawStorage,
    destination_uri: &str,
    mut captured_cursor: u64,
    mut captured_segments: Vec<(SignalKind, SegmentManifest)>,
) -> Result<ArchiveReceipt> {
    let destination = service_backup::BackupDestination::from_uri(destination_uri)?;
    let (bucket, prefix) = gcs_destination(&destination)?;
    reconcile_staged_archive_gc(storage.root())?;
    let layout: super::LayoutManifest = serde_json::from_slice(
        &std::fs::read(storage.root().join("layout.json"))
            .context("read Sift layout for archive identity")?,
    )
    .context("decode Sift layout for archive identity")?;
    let previous_state = read_commit_state(storage.root())?;
    let previous = previous_state.as_ref().map(|state| &state.manifest);
    if previous.is_some_and(|manifest| manifest.source_cluster_id != layout.cluster_id) {
        bail!("committed archive belongs to a different Sift cluster");
    }
    if previous.is_some_and(|manifest| manifest.retention_scan.is_some()) {
        bail!("bounded retention must complete before a newer archive suffix is committed");
    }
    let canonical_destination = format!("gs://{bucket}/{prefix}");
    let intent = prepare_archive_upload_intent(
        storage.root(),
        &canonical_destination,
        &prefix,
        &layout.cluster_id,
        captured_cursor,
        previous_state.as_ref(),
    )?;
    let object_store: Arc<dyn storage_object::ObjectStore> =
        Arc::new(storage_object::GcsObjectStore::new(&bucket, "")?);
    if let Some(receipt) = recover_uploaded_archive(object_store.as_ref(), &intent)? {
        return Ok(receipt);
    }
    if captured_cursor < intent.captured_cursor {
        bail!("captured archive cursor moved behind its durable upload intent");
    }
    captured_cursor = intent.captured_cursor;
    if captured_segments.iter().any(|(_, segment)| {
        segment.first_cursor <= captured_cursor && segment.last_cursor > captured_cursor
    }) {
        bail!("a local segment crosses the durable archive upload cursor");
    }
    captured_segments.retain(|(_, segment)| segment.last_cursor <= captured_cursor);
    let coordinator = storage_segment::ArchiveCoordinator::new(object_store.clone());
    let mut archive_transaction = coordinator.begin();
    let archive_prefix = intent.archive_prefix.clone();
    let previous_catalog = previous
        .map(|manifest| catalog_for_uri(&manifest.catalog_uri))
        .transpose()?;
    let previously_covered = previous
        .map(|manifest| manifest.raft_snapshot_index)
        .unwrap_or_default();
    if captured_cursor < previously_covered {
        bail!(
            "captured archive cursor {captured_cursor} is behind committed prefix {previously_covered}"
        );
    }

    let spill_parent = storage.root().join("tmp");
    let mut catalog_updates = SpillCatalog::new(&spill_parent, "archive-updates-")?;
    let mut referenced = SpillCatalog::new(&spill_parent, "archive-blob-counts-")?;
    let mut gc_candidates = SpillCatalog::new(&spill_parent, "archive-obsolete-")?;
    let mut live_objects = SpillCatalog::new(&spill_parent, "archive-live-")?;
    spill_pending_archive_gc(storage.root(), &mut gc_candidates)?;
    if let Some(state) = &previous_state {
        gc_candidates.insert_uri(&state.manifest_uri)?;
    }
    if let Some(previous) = previous {
        if let (Some(gc_uri), Some(gc_root)) = (&previous.gc_plan_uri, &previous.gc_plan_root) {
            let (gc_bucket, _) = split_gcs_uri(gc_uri)?;
            let gc_catalog = catalog_for_uri(gc_uri)?;
            for key in gc_catalog.page_keys(gc_root)? {
                gc_candidates.insert_uri(&gcs_uri(&gc_bucket, &key?))?;
            }
        }
    }

    let mut event_count = previous.map(|value| value.event_count).unwrap_or_default();
    let mut event_id_digest = previous
        .map(|value| decode_event_id_digest(&value.event_id_sha256))
        .transpose()?
        .unwrap_or([0; 32]);
    let mut event_content_digest = previous
        .map(|value| decode_event_content_digest(&value.event_content_sha256))
        .transpose()?
        .unwrap_or([0; 32]);
    let mut watermarks = previous.map(|value| value.watermarks).unwrap_or_default();
    let mut retained_watermarks = previous
        .map(|value| value.retained_watermarks)
        .unwrap_or_default();
    let retention_generation = previous
        .map(|value| value.retention_generation)
        .unwrap_or_default();
    let mut oldest_event_time_unix_nano =
        previous.and_then(|value| value.oldest_event_time_unix_nano);
    let mut new_segment_count = 0_u64;
    verify_local_archive_prefix(storage, previously_covered, captured_cursor)?;
    captured_segments.sort_by_key(|(_, segment)| segment.first_cursor);

    for (signal, source) in captured_segments {
        let portable_source = portable_manifest(source.clone(), signal);
        let covered_through = previous
            .map(|manifest| manifest.watermarks.through(signal))
            .unwrap_or_default();
        if source.last_cursor <= covered_through {
            if let (Some(previous), Some(catalog)) = (previous, &previous_catalog) {
                let probe = ArchiveSegment {
                    signal,
                    source: portable_source.clone(),
                    object_uri: String::new(),
                    parquet_bytes: 0,
                    parquet_sha256: String::new(),
                    min_acknowledged_at_unix_nano: 0,
                    max_acknowledged_at_unix_nano: 0,
                    dedupe_receipt: empty_dedupe_receipt(),
                };
                if let Some(entry) =
                    catalog.lookup(&previous.catalog_root, &segment_catalog_key(&probe))?
                {
                    let ArchiveCatalogItem::Segment(committed) =
                        decode_archive_catalog_entry(entry)?
                    else {
                        bail!("archive segment catalog key resolved to a blob");
                    };
                    if committed.signal != signal || committed.source != portable_source {
                        bail!(
                            "immutable segment {} changed after archive commit",
                            source.segment_id
                        );
                    }
                }
            }
            continue;
        }
        if source.first_cursor <= covered_through {
            bail!(
                "local segment {} crosses committed archive cursor {}",
                source.segment_id,
                covered_through
            );
        }
        let events = storage.read_segment_events(signal, &source)?;
        let partition = events
            .first()
            .map(|event| storage_segment::Partitioner::partition(&SiftSignalPartitioner, event))
            .transpose()?
            .context("immutable segment must contain at least one event")?;
        if partition != signal.to_string() {
            bail!("segment partition disagrees with its signal");
        }
        for event in &events {
            event_count = event_count.saturating_add(1);
            include_event_id(&mut event_id_digest, &event.event.event_id);
            include_event_content(&mut event_content_digest, event)?;
            watermarks.include(signal, event.cursor);
            retained_watermarks.include(signal, event.cursor);
            for reference in &event.event.blob_refs {
                add_spill_blob_reference(&mut referenced, reference)?;
            }
        }
        let parquet = encode_parquet(&events)?;
        let parquet_sha256 = sha256(&parquet);
        let key = format!(
            "{archive_prefix}/segments/{partition}/{}.parquet",
            source.segment_id
        );
        archive_transaction.put(storage_segment::ArchiveObject::new(
            key.clone(),
            parquet.clone(),
            "application/vnd.apache.parquet",
        ))?;
        let (minimum_acknowledged_at, maximum_acknowledged_at) =
            acknowledgement_time_bounds(&events)?;
        let dedupe_receipt =
            archive_dedupe_receipts(&mut archive_transaction, &bucket, &archive_prefix, &events)?;
        let segment = ArchiveSegment {
            signal,
            source: portable_source,
            object_uri: gcs_uri(&bucket, &key),
            parquet_bytes: parquet.len() as u64,
            parquet_sha256,
            min_acknowledged_at_unix_nano: minimum_acknowledged_at,
            max_acknowledged_at_unix_nano: maximum_acknowledged_at,
            dedupe_receipt,
        };
        oldest_event_time_unix_nano = Some(
            oldest_event_time_unix_nano
                .map(|oldest| oldest.min(segment.source.min_event_time_unix_nano))
                .unwrap_or(segment.source.min_event_time_unix_nano),
        );
        catalog_updates.upsert(segment_catalog_key(&segment), serde_json::to_vec(&segment)?)?;
        live_objects.insert_uri(&segment.object_uri)?;
        live_objects.insert_uri(&segment.dedupe_receipt.object_uri)?;
        new_segment_count = new_segment_count.saturating_add(1);
    }

    let mut new_blob_count = 0_u64;
    for entry in referenced.reader()? {
        let counted: SpillBlobReference =
            serde_json::from_slice(&entry?.value).context("decode archive spill blob reference")?;
        let catalog_key = format!("blob/{}", counted.reference.hash);
        let existing = match (previous, &previous_catalog) {
            (Some(previous), Some(catalog)) => catalog
                .lookup(&previous.catalog_root, &catalog_key)?
                .map(decode_archive_catalog_entry)
                .transpose()?,
            _ => None,
        };
        let blob = if let Some(ArchiveCatalogItem::Blob(mut blob)) = existing {
            if blob.reference != counted.reference || blob.reference_count == 0 {
                bail!(
                    "immutable blob {} changed after archive commit",
                    counted.reference.hash
                );
            }
            blob.reference_count = blob.reference_count.saturating_add(counted.count);
            blob
        } else if existing.is_some() {
            bail!("archive blob catalog key resolved to a segment");
        } else {
            let bytes = storage.read_blob(&counted.reference.hash)?;
            if bytes.len() as u64 != counted.reference.size {
                bail!(
                    "blob {} size changed before archive",
                    counted.reference.hash
                );
            }
            let digest = counted.reference.hash.trim_start_matches("sha256:");
            let key = format!("{archive_prefix}/blobs/{digest}.blob");
            archive_transaction.put(storage_segment::ArchiveObject::new(
                key.clone(),
                bytes,
                "application/octet-stream",
            ))?;
            new_blob_count = new_blob_count.saturating_add(1);
            ArchiveBlob {
                reference: counted.reference,
                object_uri: gcs_uri(&bucket, &key),
                reference_count: counted.count,
            }
        };
        catalog_updates.upsert(blob_catalog_key(&blob), serde_json::to_vec(&blob)?)?;
        live_objects.insert_uri(&blob.object_uri)?;
    }

    let catalog_prefix = format!("{archive_prefix}/catalog");
    let catalog_runtime =
        storage_segment::PagedCatalog::new(object_store.clone(), &catalog_prefix)?;
    let catalog_root = if let Some(previous) = previous {
        let (previous_bucket, _) = split_gcs_uri(&previous.catalog_uri)?;
        if previous_bucket != bucket {
            bail!("committed archive catalog moved to a different bucket");
        }
        let mut root = previous.catalog_root.clone();
        for entry in catalog_updates.reader()? {
            let mutation = catalog_runtime.upsert(&root, entry?)?;
            for key in &mutation.obsolete_page_keys {
                gc_candidates.insert_uri(&gcs_uri(&bucket, key))?;
                live_objects.remove_uri(&gcs_uri(&bucket, key))?;
            }
            for key in &mutation.written_page_keys {
                live_objects.insert_uri(&gcs_uri(&bucket, key))?;
            }
            root = mutation.root;
        }
        root
    } else {
        let mut written = SpillCatalog::new(&spill_parent, "archive-catalog-written-")?;
        match catalog_runtime.build_sorted_observed(catalog_updates.reader()?, |reference| {
            record_catalog_page(&mut written, reference)
        }) {
            Ok(build) => build.root,
            Err(error) => {
                cleanup_observed_catalog_pages(object_store.as_ref(), &written)?;
                return Err(error.into());
            }
        }
    };
    let segment_count = previous
        .map(|manifest| manifest.segment_count)
        .unwrap_or_default()
        .saturating_add(new_segment_count);
    let blob_count = previous
        .map(|manifest| manifest.blob_count)
        .unwrap_or_default()
        .saturating_add(new_blob_count);
    let dedupe_receipt_count = previous
        .map(|manifest| manifest.dedupe_receipt_count)
        .unwrap_or_default();
    if catalog_root.entry_count
        != segment_count
            .saturating_add(blob_count)
            .saturating_add(dedupe_receipt_count)
    {
        bail!("archive catalog root count does not match segment and blob counts");
    }
    let gc_prefix = format!("{archive_prefix}/gc-plan");
    let gc_plan = build_gc_catalog_from_spills(
        object_store,
        &gc_prefix,
        &gc_candidates,
        &live_objects,
        &spill_parent,
    )?;
    let gc_object_count = gc_plan
        .as_ref()
        .map(|plan| plan.root.entry_count)
        .unwrap_or_default();
    let manifest = ArchiveManifest {
        format_version: ARCHIVE_FORMAT_VERSION,
        generated_at: intent.generated_at,
        source_cluster_id: layout.cluster_id,
        source_node_id: layout.node_id,
        raft_snapshot_index: captured_cursor,
        event_count,
        event_id_digest_algorithm: "xor-sha256-v1".to_string(),
        event_id_sha256: hex::encode(event_id_digest),
        event_content_digest_algorithm: "xor-sha256-v1".to_string(),
        event_content_sha256: hex::encode(event_content_digest),
        retention_generation,
        watermarks,
        retained_watermarks,
        oldest_event_time_unix_nano,
        epochs: storage.epoch_maps(),
        catalog_uri: gcs_uri(&bucket, &catalog_prefix),
        catalog_root,
        segment_count,
        blob_count,
        dedupe_receipt_count,
        gc_plan_uri: gc_plan.as_ref().map(|_| gcs_uri(&bucket, &gc_prefix)),
        gc_plan_root: gc_plan.map(|plan| plan.root),
        gc_object_count,
        retention_delta: None,
        retention_scan: previous.and_then(|manifest| manifest.retention_scan.clone()),
        segments: Vec::new(),
        blobs: Vec::new(),
        gc_object_uris: Vec::new(),
    };
    validate_archive_manifest(&manifest)?;
    let manifest_key = format!("{archive_prefix}/manifest.json");
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let commit = archive_transaction.commit(storage_segment::ArchiveObject::new(
        manifest_key.clone(),
        manifest_bytes,
        "application/json",
    ))?;
    Ok(ArchiveReceipt {
        manifest_uri: gcs_uri(&bucket, &manifest_key),
        manifest_sha256: commit.manifest.sha256,
        manifest,
    })
}

fn add_spill_blob_reference(target: &mut SpillCatalog, reference: &ContentBlobRef) -> Result<()> {
    let key = format!("blob/{}", reference.hash);
    let mut counted = target
        .lookup(&key)?
        .map(|bytes| {
            serde_json::from_slice::<SpillBlobReference>(&bytes)
                .context("decode archive spill blob reference")
        })
        .transpose()?
        .unwrap_or_else(|| SpillBlobReference {
            reference: reference.clone(),
            count: 0,
        });
    if counted.reference != *reference {
        bail!("one blob hash has conflicting content references");
    }
    counted.count = counted.count.saturating_add(1);
    target.upsert(key, serde_json::to_vec(&counted)?)
}

fn archive_dedupe_receipts(
    transaction: &mut storage_segment::ArchiveTransaction,
    bucket: &str,
    archive_prefix: &str,
    events: &[StoredEvent],
) -> Result<ArchiveDedupeReceipt> {
    let mut receipts = Vec::with_capacity(events.len());
    for event in events {
        receipts.push(DedupeReceipt {
            project: event.event.project.clone(),
            event_id: event.event.event_id.clone(),
            cursor: event.cursor,
            acknowledged_at_unix_nano: acknowledgement_time_unix_nano(event)?,
        });
    }
    if receipts.is_empty() {
        bail!("archive dedupe receipt cannot be empty");
    }
    let mut encoder = GzEncoder::new(Vec::new(), GzipCompression::fast());
    encoder
        .write_all(&serde_json::to_vec(&receipts)?)
        .context("encode archive dedupe receipt payload")?;
    let bytes = encoder
        .finish()
        .context("finish archive dedupe receipt gzip")?;
    let digest = sha256(&bytes);
    let key = format!("{archive_prefix}/receipts/{digest}.json.gz");
    transaction.put(storage_segment::ArchiveObject::new(
        key.clone(),
        bytes.clone(),
        "application/gzip",
    ))?;
    Ok(ArchiveDedupeReceipt {
        object_uri: gcs_uri(bucket, &key),
        bytes: bytes.len() as u64,
        sha256: digest,
        entry_count: receipts.len() as u64,
        first_cursor: receipts.first().expect("non-empty receipt").cursor,
        last_cursor: receipts.last().expect("non-empty receipt").cursor,
        min_acknowledged_at_unix_nano: receipts
            .iter()
            .map(|receipt| receipt.acknowledged_at_unix_nano)
            .min()
            .expect("non-empty receipt"),
        max_acknowledged_at_unix_nano: receipts
            .iter()
            .map(|receipt| receipt.acknowledged_at_unix_nano)
            .max()
            .expect("non-empty receipt"),
    })
}

fn fetch_dedupe_receipts(receipt: &ArchiveDedupeReceipt) -> Result<Vec<DedupeReceipt>> {
    validate_dedupe_receipt(receipt)?;
    let bytes = service_backup::fetch_backup_object(&receipt.object_uri)?;
    verify_bytes(&receipt.sha256, receipt.bytes, &bytes, "dedupe receipt")?;
    let maximum = receipt.entry_count.saturating_mul(1024).saturating_add(1);
    let mut decoded = Vec::new();
    GzDecoder::new(bytes.as_slice())
        .take(maximum)
        .read_to_end(&mut decoded)
        .context("decompress archive dedupe receipt")?;
    if decoded.len() as u64 >= maximum {
        bail!("archive dedupe receipt exceeds its decoded size limit");
    }
    let rows: Vec<DedupeReceipt> =
        serde_json::from_slice(&decoded).context("decode archive dedupe receipt")?;
    if rows.len() as u64 != receipt.entry_count
        || rows.first().map(|row| row.cursor) != Some(receipt.first_cursor)
        || rows.last().map(|row| row.cursor) != Some(receipt.last_cursor)
        || rows.iter().map(|row| row.acknowledged_at_unix_nano).min()
            != Some(receipt.min_acknowledged_at_unix_nano)
        || rows.iter().map(|row| row.acknowledged_at_unix_nano).max()
            != Some(receipt.max_acknowledged_at_unix_nano)
        || rows
            .windows(2)
            .any(|window| window[0].cursor >= window[1].cursor)
        || rows
            .iter()
            .any(|row| row.project.is_empty() || row.event_id.is_empty() || row.cursor == 0)
    {
        bail!("archive dedupe receipt metadata does not match its rows");
    }
    Ok(rows)
}

fn validate_dedupe_receipt(receipt: &ArchiveDedupeReceipt) -> Result<()> {
    if !receipt.object_uri.starts_with("gs://")
        || receipt.bytes == 0
        || receipt.sha256.len() != 64
        || !receipt.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        || receipt.entry_count == 0
        || receipt.first_cursor == 0
        || receipt.first_cursor > receipt.last_cursor
        || receipt.min_acknowledged_at_unix_nano > receipt.max_acknowledged_at_unix_nano
    {
        bail!("archive dedupe receipt has invalid metadata");
    }
    Ok(())
}

fn gcs_destination(destination: &service_backup::BackupDestination) -> Result<(String, String)> {
    let service_backup::BackupDestination::Gcs {
        bucket,
        prefix,
        credentials_secret,
    } = destination
    else {
        bail!("{} is not a GCS backup destination", destination.identity());
    };
    if let Some(secret) = credentials_secret {
        bail!(
            "GCS credentials_secret `{secret}` is not supported; use ADC and GKE Workload Identity"
        );
    }
    Ok((
        bucket.clone(),
        if prefix.is_empty() {
            "backup".to_string()
        } else {
            prefix.trim_matches('/').to_string()
        },
    ))
}

fn split_gcs_uri(uri: &str) -> Result<(String, String)> {
    let value = uri
        .trim()
        .strip_prefix("gs://")
        .context("GCS object URI must start with gs://")?;
    let (bucket, key) = value
        .split_once('/')
        .context("GCS object URI must contain an object key")?;
    if bucket.is_empty() || key.is_empty() {
        bail!("GCS object URI must contain a bucket and object key");
    }
    Ok((bucket.to_string(), key.to_string()))
}

fn gcs_uri(bucket: &str, key: &str) -> String {
    format!("gs://{bucket}/{}", key.trim_start_matches('/'))
}

// Catalog rows are decoded and consumed one at a time. Keeping the concrete
// values here avoids an allocation on every segment in the query hot path.
#[allow(clippy::large_enum_variant)]
enum ArchiveCatalogItem {
    Segment(ArchiveSegment),
    Blob(ArchiveBlob),
    Receipt(ArchiveDedupeReceipt),
}

/// Non-durable file store for rebuildable archive scratch pages.
///
/// Archive scratch state is never an acknowledgement or recovery source. The
/// containing operation can restart from the committed manifest, so these
/// writes intentionally avoid one fsync per copy-on-write page.
struct EphemeralFileObjectStore {
    root: PathBuf,
    gate: Mutex<()>,
}

impl EphemeralFileObjectStore {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            gate: Mutex::new(()),
        }
    }

    fn path(&self, key: &str) -> storage_object::Result<PathBuf> {
        let key = key.trim_matches('/');
        if key.is_empty()
            || key.contains('\0')
            || key.contains('\\')
            || key
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err(storage_object::ObjectStoreError::InvalidKey {
                key: key.to_string(),
            });
        }
        Ok(self.root.join(key))
    }

    fn meta(&self, key: &str, bytes: &[u8], content_type: &str) -> storage_object::ObjectMeta {
        let version = storage_object::ObjectVersion::new(sha256(bytes));
        storage_object::ObjectMeta {
            key: key.to_string(),
            size: bytes.len() as u64,
            content_type: content_type.to_string(),
            version: version.clone(),
            etag: Some(version.as_str().to_string()),
            updated: None,
        }
    }
}

impl storage_object::ObjectStore for EphemeralFileObjectStore {
    fn put(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
        condition: storage_object::PutCondition,
    ) -> storage_object::Result<storage_object::ObjectMeta> {
        let _gate = self.gate.lock().expect("archive scratch lock poisoned");
        let path = self.path(key)?;
        if matches!(condition, storage_object::PutCondition::IfAbsent) && path.exists() {
            return Err(storage_object::ObjectStoreError::PreconditionFailed {
                key: key.to_string(),
            });
        }
        if let storage_object::PutCondition::IfVersion(expected) = condition {
            let current =
                std::fs::read(&path).map_err(|error| storage_object::ObjectStoreError::Io {
                    message: error.to_string(),
                })?;
            if sha256(&current) != expected.as_str() {
                return Err(storage_object::ObjectStoreError::PreconditionFailed {
                    key: key.to_string(),
                });
            }
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                storage_object::ObjectStoreError::Io {
                    message: error.to_string(),
                }
            })?;
        }
        std::fs::write(&path, bytes).map_err(|error| storage_object::ObjectStoreError::Io {
            message: error.to_string(),
        })?;
        Ok(self.meta(key, bytes, content_type))
    }

    fn get(&self, key: &str) -> storage_object::Result<storage_object::Object> {
        let _gate = self.gate.lock().expect("archive scratch lock poisoned");
        let path = self.path(key)?;
        let bytes = std::fs::read(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                storage_object::ObjectStoreError::NotFound {
                    key: key.to_string(),
                }
            } else {
                storage_object::ObjectStoreError::Io {
                    message: error.to_string(),
                }
            }
        })?;
        Ok(storage_object::Object {
            meta: self.meta(key, &bytes, "application/json"),
            bytes,
        })
    }

    fn head(&self, key: &str) -> storage_object::Result<storage_object::ObjectMeta> {
        self.get(key).map(|object| object.meta)
    }

    fn list(&self, prefix: &str) -> storage_object::Result<Vec<storage_object::ObjectMeta>> {
        let _gate = self.gate.lock().expect("archive scratch lock poisoned");
        let mut pending = vec![self.root.clone()];
        let mut objects = Vec::new();
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).map_err(|error| {
                storage_object::ObjectStoreError::Io {
                    message: error.to_string(),
                }
            })? {
                let entry = entry.map_err(|error| storage_object::ObjectStoreError::Io {
                    message: error.to_string(),
                })?;
                let file_type =
                    entry
                        .file_type()
                        .map_err(|error| storage_object::ObjectStoreError::Io {
                            message: error.to_string(),
                        })?;
                if file_type.is_symlink() {
                    return Err(storage_object::ObjectStoreError::UnsafePath {
                        path: entry.path().display().to_string(),
                    });
                }
                if file_type.is_dir() {
                    pending.push(entry.path());
                    continue;
                }
                let key = entry
                    .path()
                    .strip_prefix(&self.root)
                    .expect("scratch entry remains below root")
                    .to_string_lossy()
                    .replace('\\', "/");
                if key.starts_with(prefix) {
                    let bytes = std::fs::read(entry.path()).map_err(|error| {
                        storage_object::ObjectStoreError::Io {
                            message: error.to_string(),
                        }
                    })?;
                    objects.push(self.meta(&key, &bytes, "application/json"));
                }
            }
        }
        objects.sort_by(|left, right| left.key.cmp(&right.key));
        Ok(objects)
    }

    fn delete(&self, key: &str) -> storage_object::Result<()> {
        let _gate = self.gate.lock().expect("archive scratch lock poisoned");
        let path = self.path(key)?;
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(storage_object::ObjectStoreError::Io {
                message: error.to_string(),
            }),
        }
    }
}

/// A temporary, disk-backed ordered map used by archive maintenance.
///
/// Each mutation keeps only the paged-catalog search path in memory. Replaced
/// copy-on-write pages are deleted immediately. The whole temporary directory
/// disappears when the operation ends.
struct SpillCatalog {
    store: Arc<EphemeralFileObjectStore>,
    catalog: storage_segment::PagedCatalog,
    root: storage_segment::CatalogRoot,
    _directory: tempfile::TempDir,
}

impl SpillCatalog {
    fn new(parent: &Path, name: &str) -> Result<Self> {
        let directory = tempfile::Builder::new()
            .prefix(name)
            .tempdir_in(parent)
            .with_context(|| format!("create archive spill directory in {}", parent.display()))?;
        let store = Arc::new(EphemeralFileObjectStore::new(directory.path()));
        let catalog = storage_segment::PagedCatalog::new(store.clone(), "catalog")?;
        let root = catalog.build_sorted(std::iter::empty())?.root;
        Ok(Self {
            store,
            catalog,
            root,
            _directory: directory,
        })
    }

    fn upsert(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        let mutation = self
            .catalog
            .upsert(&self.root, storage_segment::CatalogEntry { key, value })?;
        let written = mutation
            .written_page_keys
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for obsolete in &mutation.obsolete_page_keys {
            if !written.contains(obsolete.as_str()) {
                storage_object::ObjectStore::delete(self.store.as_ref(), obsolete)?;
            }
        }
        self.root = mutation.root;
        Ok(())
    }

    fn lookup(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self
            .catalog
            .lookup(&self.root, key)?
            .map(|entry| entry.value))
    }

    fn remove(&mut self, key: &str) -> Result<()> {
        let mutation = self.catalog.remove(&self.root, key)?;
        let written = mutation
            .written_page_keys
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for obsolete in &mutation.obsolete_page_keys {
            if !written.contains(obsolete.as_str()) {
                storage_object::ObjectStore::delete(self.store.as_ref(), obsolete)?;
            }
        }
        self.root = mutation.root;
        Ok(())
    }

    fn add_u64(&mut self, key: String, amount: u64) -> Result<()> {
        let current = self
            .lookup(&key)?
            .map(|bytes| {
                bytes
                    .as_slice()
                    .try_into()
                    .map(u64::from_le_bytes)
                    .map_err(|_| anyhow::anyhow!("archive spill count is corrupt"))
            })
            .transpose()?
            .unwrap_or_default();
        self.upsert(key, current.saturating_add(amount).to_le_bytes().to_vec())
    }

    fn get_u64(&self, key: &str) -> Result<u64> {
        self.lookup(key)?
            .map(|bytes| {
                bytes
                    .as_slice()
                    .try_into()
                    .map(u64::from_le_bytes)
                    .map_err(|_| anyhow::anyhow!("archive spill count is corrupt"))
            })
            .transpose()
            .map(|value| value.unwrap_or_default())
    }

    fn insert_uri(&mut self, uri: &str) -> Result<()> {
        self.upsert(
            format!("uri/{}", sha256(uri.as_bytes())),
            uri.as_bytes().to_vec(),
        )
    }

    fn remove_uri(&mut self, uri: &str) -> Result<()> {
        self.remove(&format!("uri/{}", sha256(uri.as_bytes())))
    }

    fn contains_uri(&self, uri: &str) -> Result<bool> {
        Ok(self
            .lookup(&format!("uri/{}", sha256(uri.as_bytes())))?
            .is_some())
    }

    fn reader(&self) -> Result<storage_segment::CatalogReader> {
        self.catalog.reader(&self.root).map_err(Into::into)
    }

    fn len(&self) -> u64 {
        self.root.entry_count
    }
}

impl super::BlobHashSet for SpillCatalog {
    fn insert_hash(&mut self, hash: &str) -> Result<()> {
        self.upsert(format!("blob/{hash}"), Vec::new())
    }

    fn contains_hash(&self, hash: &str) -> Result<bool> {
        Ok(self.lookup(&format!("blob/{hash}"))?.is_some())
    }
}

fn segment_catalog_key(segment: &ArchiveSegment) -> String {
    format!(
        "segment/{}/{:020}/{}",
        segment.signal, segment.source.first_cursor, segment.source.segment_id
    )
}

fn empty_dedupe_receipt() -> ArchiveDedupeReceipt {
    ArchiveDedupeReceipt {
        object_uri: String::new(),
        bytes: 0,
        sha256: String::new(),
        entry_count: 0,
        first_cursor: 0,
        last_cursor: 0,
        min_acknowledged_at_unix_nano: 0,
        max_acknowledged_at_unix_nano: 0,
    }
}

fn dedupe_receipt_catalog_key(receipt: &ArchiveDedupeReceipt) -> String {
    format!(
        "receipt/{:020}/{:020}/{}",
        receipt.max_acknowledged_at_unix_nano, receipt.first_cursor, receipt.sha256
    )
}

fn blob_catalog_key(blob: &ArchiveBlob) -> String {
    format!("blob/{}", blob.reference.hash)
}

fn catalog_for_uri(uri: &str) -> Result<storage_segment::PagedCatalog> {
    let (bucket, prefix) = split_gcs_uri(uri)?;
    storage_segment::PagedCatalog::new(
        Arc::new(storage_object::GcsObjectStore::new(&bucket, "")?),
        prefix,
    )
    .map_err(Into::into)
}

fn decode_archive_catalog_entry(
    entry: storage_segment::CatalogEntry,
) -> Result<ArchiveCatalogItem> {
    if entry.key.starts_with("segment/") {
        return Ok(ArchiveCatalogItem::Segment(
            serde_json::from_slice(&entry.value).context("decode archive segment catalog entry")?,
        ));
    }
    if entry.key.starts_with("blob/") {
        return Ok(ArchiveCatalogItem::Blob(
            serde_json::from_slice(&entry.value).context("decode archive blob catalog entry")?,
        ));
    }
    if entry.key.starts_with("receipt/") {
        return Ok(ArchiveCatalogItem::Receipt(
            serde_json::from_slice(&entry.value)
                .context("decode archive dedupe receipt catalog entry")?,
        ));
    }
    bail!("archive catalog contains an unknown key {}", entry.key)
}

fn load_archive_catalog(
    manifest: &ArchiveManifest,
) -> Result<(Vec<ArchiveSegment>, Vec<ArchiveBlob>)> {
    let catalog = catalog_for_uri(&manifest.catalog_uri)?;
    let mut segments = Vec::new();
    let mut blobs = Vec::new();
    let mut receipt_count = 0_u64;
    for entry in catalog.reader(&manifest.catalog_root)? {
        match decode_archive_catalog_entry(entry?)? {
            ArchiveCatalogItem::Segment(segment) => segments.push(segment),
            ArchiveCatalogItem::Blob(blob) => blobs.push(blob),
            ArchiveCatalogItem::Receipt(_) => {
                receipt_count = receipt_count.saturating_add(1);
            }
        }
    }
    segments.sort_by_key(|segment| segment.source.first_cursor);
    blobs.sort_by(|left, right| left.reference.hash.cmp(&right.reference.hash));
    if segments.len() as u64 != manifest.segment_count
        || blobs.len() as u64 != manifest.blob_count
        || receipt_count != manifest.dedupe_receipt_count
    {
        bail!("archive catalog counts disagree with its manifest root");
    }
    Ok((segments, blobs))
}

#[doc(hidden)]
pub fn inspect_archive_catalog(
    manifest: &ArchiveManifest,
) -> Result<(Vec<ArchiveSegment>, Vec<ArchiveBlob>)> {
    load_archive_catalog(manifest)
}

fn load_gc_catalog(manifest: &ArchiveManifest) -> Result<Vec<String>> {
    let (Some(uri), Some(root)) = (&manifest.gc_plan_uri, &manifest.gc_plan_root) else {
        if manifest.gc_object_count != 0 {
            bail!("archive GC root is missing for a non-empty plan");
        }
        return Ok(Vec::new());
    };
    let catalog = catalog_for_uri(uri)?;
    let mut uris = Vec::new();
    for entry in catalog.reader(root)? {
        let entry = entry?;
        if !entry.key.starts_with("gc/") {
            bail!("archive GC catalog contains an unknown key {}", entry.key);
        }
        uris.push(String::from_utf8(entry.value).context("decode archive GC object URI")?);
    }
    uris.sort();
    uris.dedup();
    if uris.len() as u64 != manifest.gc_object_count {
        bail!("archive GC catalog count disagrees with its manifest root");
    }
    Ok(uris)
}

#[doc(hidden)]
pub fn inspect_archive_gc_plan(manifest: &ArchiveManifest) -> Result<Vec<String>> {
    load_gc_catalog(manifest)
}

pub(crate) fn committed_watermarks(root: &Path) -> Result<ArchiveWatermarks> {
    let remote = committed_status(root)?
        .map(|status| status.watermarks)
        .unwrap_or_default();
    Ok(remote.merge(local_committed_watermarks(root)?))
}

pub(crate) fn remote_retained_state(root: &Path) -> Result<Option<RemoteRetainedState>> {
    read_commit_state(root)?
        .map(|state| {
            Ok(RemoteRetainedState {
                watermarks: state.watermarks,
                event_count: state.manifest.event_count,
                snapshot_index: state.manifest.raft_snapshot_index,
                retention_generation: state.manifest.retention_generation,
                retention_scan_pending: state.manifest.retention_scan.is_some(),
                event_content_sha256: decode_event_content_digest(
                    &state.manifest.event_content_sha256,
                )?,
            })
        })
        .transpose()
}

pub(crate) fn local_committed_watermarks(root: &Path) -> Result<ArchiveWatermarks> {
    Ok(read_local_commit_state(root)?
        .map(|state| state.watermarks)
        .unwrap_or_default())
}

pub(crate) fn local_committed_status(root: &Path) -> Result<Option<LocalArchiveStatus>> {
    Ok(
        read_local_commit_state(root)?.map(|state| LocalArchiveStatus {
            snapshot_index: state.snapshot_index,
            watermarks: state.watermarks,
        }),
    )
}

pub fn committed_status(root: &Path) -> Result<Option<ArchiveCommitStatus>> {
    Ok(read_commit_state(root)?.map(|state| ArchiveCommitStatus {
        manifest_uri: state.manifest_uri,
        manifest_sha256: state.manifest_sha256,
        committed_at: state.committed_at,
        snapshot_index: state.manifest.raft_snapshot_index,
        watermarks: state.watermarks,
        retention_generation: state.manifest.retention_generation,
        retention_scan_pending: state.manifest.retention_scan.is_some(),
    }))
}

/// Finish a retention commit that reached the local archive receipt but was
/// interrupted before the local hot set and projection generation changed.
/// This runs only when the manifest generation is newer than the journal head.
pub(crate) fn reconcile_committed_retention(
    root: &Path,
    storage: &RawStorage,
    current_generation: u64,
) -> Result<Option<u64>> {
    let Some(status) = committed_status(root)? else {
        return Ok(None);
    };
    if status.retention_generation <= current_generation {
        return Ok(Some(status.retention_generation));
    }
    let committed = fetch_verified_committed_root(root)?
        .context("committed retention recovery requires its manifest")?;
    if let Some(delta) = &committed.retention_delta {
        if delta.source_generation == current_generation
            && committed.retention_generation == current_generation.saturating_add(1)
        {
            let cutoff = DateTime::<Utc>::from_timestamp_nanos(delta.cutoff_unix_nano);
            storage.evict_expired_before(cutoff, committed.raft_snapshot_index)?;
            return Ok(Some(committed.retention_generation));
        }
    }
    let restore_parent = root.join("tmp");
    let restored_root = tempfile::tempdir_in(&restore_parent).with_context(|| {
        format!(
            "create interrupted-retention recovery directory in {}",
            restore_parent.display()
        )
    })?;
    let manifest = restore_gcs(&status.manifest_uri, restored_root.path())
        .context("restore committed archive for interrupted retention recovery")?;
    if manifest.retention_generation != status.retention_generation
        || manifest.raft_snapshot_index != status.snapshot_index
    {
        bail!("committed retention manifest changed during recovery");
    }
    let retained = RawStorage::open(restored_root.path())?;
    storage.reconcile_retained_prefix(&retained, manifest.raft_snapshot_index)?;
    Ok(Some(manifest.retention_generation))
}

/// Finish a retention receipt inside the same process after a late local
/// failure. The remote manifest is already the durable authority. Reapply its
/// idempotent local delta and persist the journal head before serving again.
pub(crate) fn reconcile_live_committed_retention(journal: &crate::DurableJournal) -> Result<()> {
    let current_generation = journal.retention_generation();
    let Some(status) = committed_status(journal.data_dir())? else {
        return Ok(());
    };
    if status.retention_generation < current_generation {
        bail!("local retention generation is ahead of its committed manifest");
    }
    if status.retention_generation == current_generation && !journal.recovery_required() {
        return Ok(());
    }
    let committed = fetch_verified_committed_root(journal.data_dir())?
        .context("live retention recovery requires its committed manifest")?;
    let delta = committed
        .retention_delta
        .as_ref()
        .context("live retention recovery requires a committed retention delta")?;
    if delta.source_generation.saturating_add(1) != committed.retention_generation
        || (current_generation != delta.source_generation
            && current_generation != committed.retention_generation)
    {
        bail!("live retention recovery cannot bridge the committed generation gap");
    }
    reconcile_committed_retention(journal.data_dir(), journal.storage(), current_generation)?;
    journal.apply_expiration_head(
        DateTime::<Utc>::from_timestamp_nanos(delta.cutoff_unix_nano),
        delta.source_event_count,
        committed.event_count,
        decode_event_content_digest(&delta.source_event_content_sha256)?,
        decode_event_content_digest(&committed.event_content_sha256)?,
        true,
    )
}

/// Verify that the last locally committed archive manifest is still readable.
/// A cold query uses this check before it claims a complete answer. The local
/// commit receipt is not enough because GCS access can be removed after the
/// commit was written.
pub fn verify_committed_manifest_available(root: &Path) -> Result<bool> {
    Ok(fetch_verified_committed_root(root)?.is_some())
}

/// Replay matching events from the latest committed remote manifest.
///
/// Parquet objects are hash-checked before use. A good local cache is reused,
/// but Sift still verifies the remote commit manifest on each cold query. This
/// makes a GCS outage visible instead of returning a silent empty result.
pub fn replay_committed_events<F>(
    root: &Path,
    signal: SignalKind,
    project: &str,
    environment: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
    mut visitor: F,
) -> Result<Option<ArchiveReplay>>
where
    F: FnMut(StoredEvent) -> Result<()>,
{
    let Some(manifest) = fetch_verified_committed_root(root)? else {
        return Ok(None);
    };
    let start = parse_optional_archive_time("start", start)?;
    let end = parse_optional_archive_time("end", end)?;
    if start.zip(end).is_some_and(|(start, end)| start >= end) {
        bail!("archive query start must be earlier than end");
    }

    let mut scanned = 0_u64;
    let mut replayed = 0_u64;
    let catalog = catalog_for_uri(&manifest.catalog_uri)?;
    for entry in catalog.reader(&manifest.catalog_root)? {
        let ArchiveCatalogItem::Segment(segment) = decode_archive_catalog_entry(entry?)? else {
            continue;
        };
        if segment.signal != signal {
            continue;
        }
        let bytes = cached_segment_bytes(root, &segment)?;
        let events = decode_parquet(&bytes)?;
        verify_archive_segment(&segment, &events)?;
        for stored in events {
            scanned = scanned.saturating_add(1);
            let event = &stored.event;
            if event.project != project
                || environment.is_some_and(|environment| event.environment != environment)
            {
                continue;
            }
            let occurred = DateTime::parse_from_rfc3339(&event.occurred_at)
                .context("archive event occurred_at must be RFC3339")?
                .with_timezone(&Utc);
            if start.is_some_and(|start| occurred < start) || end.is_some_and(|end| occurred >= end)
            {
                continue;
            }
            visitor(stored)?;
            replayed = replayed.saturating_add(1);
        }
    }
    Ok(Some(ArchiveReplay {
        watermark: manifest.watermarks.through(signal),
        scanned,
        replayed,
    }))
}

/// Read one globally ordered page from the committed retained archive.
///
/// Projection rebuilds use this path after retention changes. It joins the
/// three signal streams by raw cursor and never treats an unavailable archive
/// as an empty source.
pub(crate) fn read_committed_events_after(
    root: &Path,
    after: u64,
    limit: usize,
) -> Result<Option<Vec<StoredEvent>>> {
    if limit == 0 {
        return Ok(Some(Vec::new()));
    }
    let Some(mut reader) = CommittedEventReader::open(root, after)? else {
        return Ok(None);
    };
    reader.read_next(limit).map(Some)
}

/// One linear, globally ordered scan of the committed retained archive.
///
/// The local commit receipt is checked before any remote request. A caller
/// whose cursor is already beyond the committed archive prefix gets `None` and
/// can continue from local WAL/segments during a GCS outage.
#[doc(hidden)]
pub struct CommittedEventReader {
    streams: [ArchiveSignalStream; 3],
    snapshot_index: u64,
}

impl CommittedEventReader {
    pub fn open(root: &Path, after: u64) -> Result<Option<Self>> {
        let Some(status) = committed_status(root)? else {
            return Ok(None);
        };
        if after >= status.snapshot_index {
            return Ok(None);
        }
        let manifest = fetch_verified_committed_root(root)?
            .context("committed archive receipt has no readable manifest")?;
        let catalog = catalog_for_uri(&manifest.catalog_uri)?;
        let stream = |signal| -> Result<ArchiveSignalStream> {
            let prefix = format!("segment/{signal}/");
            Ok(ArchiveSignalStream::new_after(
                root,
                catalog.reader_after(&manifest.catalog_root, &prefix)?,
                signal,
                after,
            ))
        };
        Ok(Some(Self {
            streams: [
                stream(SignalKind::Log)?,
                stream(SignalKind::Metric)?,
                stream(SignalKind::Span)?,
            ],
            snapshot_index: manifest.raft_snapshot_index,
        }))
    }

    pub fn snapshot_index(&self) -> u64 {
        self.snapshot_index
    }

    pub fn read_next(&mut self, limit: usize) -> Result<Vec<StoredEvent>> {
        let mut page = Vec::with_capacity(limit);
        while page.len() < limit {
            let mut next = None::<(usize, u64)>;
            for (index, stream) in self.streams.iter_mut().enumerate() {
                if let Some(cursor) = stream.peek_cursor()? {
                    if next.is_none_or(|(_, current)| cursor < current) {
                        next = Some((index, cursor));
                    }
                }
            }
            let Some((index, _)) = next else {
                break;
            };
            page.push(
                self.streams[index]
                    .pop_event()?
                    .context("archive projection stream head disappeared")?,
            );
        }
        Ok(page)
    }
}

/// Replay only archive rows that can still be inside the idempotency window.
/// Segment acceptance-time bounds avoid downloading the 180-day cold archive
/// when a rebuildable dedupe index is lost.
pub(crate) fn replay_recent_committed_events<F>(
    root: &Path,
    since: DateTime<Utc>,
    mut visitor: F,
) -> Result<Option<ArchiveReplay>>
where
    F: FnMut(StoredEvent) -> Result<()>,
{
    let Some(manifest) = fetch_verified_committed_root(root)? else {
        return Ok(None);
    };
    let cutoff_nanos = since
        .timestamp_nanos_opt()
        .context("dedupe rebuild cutoff is outside the nanosecond range")?;
    let catalog = catalog_for_uri(&manifest.catalog_uri)?;
    let mut scanned = 0_u64;
    let mut replayed = 0_u64;
    for entry in catalog.reader(&manifest.catalog_root)? {
        let ArchiveCatalogItem::Segment(segment) = decode_archive_catalog_entry(entry?)? else {
            continue;
        };
        if segment.min_acknowledged_at_unix_nano > segment.max_acknowledged_at_unix_nano {
            bail!("archive segment has invalid acknowledgement-time bounds");
        }
        if segment.max_acknowledged_at_unix_nano < cutoff_nanos {
            continue;
        }
        let bytes = cached_segment_bytes(root, &segment)?;
        let events = decode_parquet(&bytes)?;
        verify_archive_segment(&segment, &events)?;
        for event in events {
            scanned = scanned.saturating_add(1);
            if acknowledgement_time_unix_nano(&event)? < cutoff_nanos {
                continue;
            }
            visitor(event)?;
            replayed = replayed.saturating_add(1);
        }
    }
    Ok(Some(ArchiveReplay {
        watermark: manifest.raft_snapshot_index,
        scanned,
        replayed,
    }))
}

pub(crate) fn replay_recent_committed_receipts<F>(
    root: &Path,
    since: DateTime<Utc>,
    mut visitor: F,
) -> Result<Option<u64>>
where
    F: FnMut(DedupeReceipt) -> Result<()>,
{
    let Some(manifest) = fetch_verified_committed_root(root)? else {
        return Ok(None);
    };
    let cutoff_nanos = since
        .timestamp_nanos_opt()
        .context("dedupe receipt rebuild cutoff is outside the nanosecond range")?;
    let catalog = catalog_for_uri(&manifest.catalog_uri)?;
    let mut replayed = 0_u64;
    for entry in catalog.reader_after(&manifest.catalog_root, "receipt/")? {
        let entry = entry?;
        if !entry.key.starts_with("receipt/") {
            break;
        }
        let ArchiveCatalogItem::Receipt(receipt) = decode_archive_catalog_entry(entry)? else {
            bail!("archive receipt key resolved to another catalog item");
        };
        if receipt.max_acknowledged_at_unix_nano < cutoff_nanos {
            continue;
        }
        for row in fetch_dedupe_receipts(&receipt)? {
            if row.acknowledged_at_unix_nano < cutoff_nanos {
                continue;
            }
            visitor(row)?;
            replayed = replayed.saturating_add(1);
        }
    }
    Ok(Some(replayed))
}

fn fetch_verified_committed_root(root: &Path) -> Result<Option<ArchiveManifest>> {
    let Some(state) = read_commit_state(root)? else {
        return Ok(None);
    };
    let bytes = service_backup::fetch_backup_object(&state.manifest_uri)
        .context("fetch committed Sift archive manifest")?;
    if sha256(&bytes) != state.manifest_sha256 {
        bail!("committed Sift archive manifest failed its SHA-256 check");
    }
    let manifest: ArchiveManifest =
        serde_json::from_slice(&bytes).context("decode committed Sift archive manifest")?;
    if manifest != state.manifest || manifest.watermarks != state.watermarks {
        bail!("remote Sift archive manifest disagrees with its local commit receipt");
    }
    validate_archive_manifest(&manifest)?;
    let catalog = catalog_for_uri(&manifest.catalog_uri)?;
    let mut reader = catalog.reader(&manifest.catalog_root)?;
    let _ = reader.next().transpose()?;
    Ok(Some(manifest))
}

fn cached_segment_bytes(root: &Path, segment: &ArchiveSegment) -> Result<Vec<u8>> {
    let cache_path = cached_segment_path(root, segment)?;
    std::fs::read(&cache_path)
        .with_context(|| format!("read archive cache {}", cache_path.display()))
}

fn cached_segment_path(root: &Path, segment: &ArchiveSegment) -> Result<PathBuf> {
    if segment.parquet_sha256.len() != 64
        || !segment
            .parquet_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("archive segment has an invalid Parquet SHA-256 value");
    }
    let cache_path = root
        .join("archive-cache")
        .join(format!("{}.parquet", segment.parquet_sha256));
    if cache_path.exists() {
        if verify_file(
            &cache_path,
            &segment.parquet_sha256,
            segment.parquet_bytes,
            "cached Parquet segment",
        )
        .is_ok()
        {
            set_private_file(&cache_path)?;
            prune_archive_cache(root, &cache_path)?;
            return Ok(cache_path);
        }
    }

    let bytes = service_backup::fetch_backup_object(&segment.object_uri)
        .with_context(|| format!("fetch archive segment {}", segment.object_uri))?;
    verify_bytes(
        &segment.parquet_sha256,
        segment.parquet_bytes,
        &bytes,
        "Parquet segment",
    )?;
    storage_durable::atomic_write(&cache_path, &bytes, storage_durable::FsyncPolicy::Always)?;
    set_private_file(&cache_path)?;
    prune_archive_cache(root, &cache_path)?;
    Ok(cache_path)
}

fn prune_archive_cache(root: &Path, keep: &Path) -> Result<()> {
    let cache_root = root.join("archive-cache");
    let mut total = 0_u64;
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(&cache_root)
        .with_context(|| format!("read archive cache {}", cache_root.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("parquet") {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_file() {
            bail!(
                "archive cache entry {} is not a regular file",
                path.display()
            );
        }
        total = total.saturating_add(metadata.len());
        candidates.push((metadata.modified().ok(), path, metadata.len()));
    }
    candidates.sort_by_key(|(modified, path, _)| (*modified, path.clone()));
    for (_, path, bytes) in candidates {
        if total <= ARCHIVE_CACHE_MAX_BYTES {
            break;
        }
        if path == keep {
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => {
                total = total.saturating_sub(bytes);
                storage_durable::sync_parent_dir(&path)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                total = total.saturating_sub(bytes);
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn parse_optional_archive_time(name: &str, value: Option<&str>) -> Result<Option<DateTime<Utc>>> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(value)
                .with_context(|| format!("archive query {name} must be RFC3339"))
                .map(|value| value.with_timezone(&Utc))
        })
        .transpose()
}

pub(crate) fn validate_archive_manifest(manifest: &ArchiveManifest) -> Result<()> {
    if manifest.format_version != ARCHIVE_FORMAT_VERSION
        || manifest.source_cluster_id.trim().is_empty()
        || manifest.source_node_id.trim().is_empty()
        || manifest.event_id_digest_algorithm != "xor-sha256-v1"
        || manifest.event_id_sha256.len() != 64
        || !manifest
            .event_id_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || manifest.event_content_digest_algorithm != "xor-sha256-v1"
        || manifest.event_content_sha256.len() != 64
        || !manifest
            .event_content_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || !manifest.catalog_uri.starts_with("gs://")
        || manifest.catalog_root.entry_count
            != manifest
                .segment_count
                .saturating_add(manifest.blob_count)
                .saturating_add(manifest.dedupe_receipt_count)
        || (manifest.segment_count == 0) != manifest.oldest_event_time_unix_nano.is_none()
        || manifest.catalog_root.page_bytes_limit as usize
            != storage_segment::DEFAULT_CATALOG_PAGE_BYTES
        || manifest.gc_plan_uri.is_some() != manifest.gc_plan_root.is_some()
        || manifest
            .gc_plan_uri
            .as_ref()
            .is_some_and(|uri| !uri.starts_with("gs://"))
        || (manifest.gc_object_count == 0) != manifest.gc_plan_root.is_none()
        || manifest
            .gc_plan_root
            .as_ref()
            .is_some_and(|root| root.entry_count != manifest.gc_object_count)
    {
        bail!("Sift archive manifest has invalid identity or digest fields");
    }
    if serde_json::to_vec(manifest)?.len() >= 64 * 1024 {
        bail!("Sift archive root manifest exceeds 64 KiB");
    }
    if manifest.watermarks.max_cursor() > manifest.raft_snapshot_index
        || manifest.event_count > manifest.raft_snapshot_index
        || manifest.retained_watermarks.logs > manifest.watermarks.logs
        || manifest.retained_watermarks.metrics > manifest.watermarks.metrics
        || manifest.retained_watermarks.traces > manifest.watermarks.traces
    {
        bail!("Sift archive manifest has invalid coverage or retained watermarks");
    }
    if let Some(delta) = &manifest.retention_delta {
        if !delta.source_manifest_uri.starts_with("gs://")
            || delta.source_manifest_sha256.len() != 64
            || !delta
                .source_manifest_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || delta.source_event_content_sha256.len() != 64
            || !delta
                .source_event_content_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || manifest.retention_generation != delta.source_generation.saturating_add(1)
            || manifest.event_count > delta.source_event_count
        {
            bail!("Sift archive retention delta has invalid source metadata");
        }
    }
    if let Some(scan) = &manifest.retention_scan {
        if scan.after_catalog_key.as_str() <= "segment/"
            || !scan.after_catalog_key.starts_with("segment/")
        {
            bail!("Sift archive retention scan has an invalid catalog cursor");
        }
        if manifest
            .retention_delta
            .as_ref()
            .is_none_or(|delta| delta.cutoff_unix_nano != scan.cutoff_unix_nano)
        {
            bail!("Sift archive retention scan lacks its generation delta");
        }
    }
    let mut event_count = 0_u64;
    let mut retained_watermarks = ArchiveWatermarks::default();
    let mut segment_ids = BTreeSet::new();
    for segment in &manifest.segments {
        if !segment.object_uri.starts_with("gs://")
            || segment.parquet_bytes == 0
            || segment.parquet_sha256.len() != 64
            || !segment
                .parquet_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || segment.source.segment_id.trim().is_empty()
            || segment.source.event_count == 0
            || segment.source.first_cursor == 0
            || segment.source.first_cursor > segment.source.last_cursor
            || segment.source.last_cursor > manifest.watermarks.through(segment.signal)
            || segment.source.min_event_time_unix_nano > segment.source.max_event_time_unix_nano
            || segment.min_acknowledged_at_unix_nano > segment.max_acknowledged_at_unix_nano
            || validate_dedupe_receipt(&segment.dedupe_receipt).is_err()
            || segment.dedupe_receipt.entry_count != segment.source.event_count
            || segment.dedupe_receipt.first_cursor != segment.source.first_cursor
            || segment.dedupe_receipt.last_cursor != segment.source.last_cursor
            || segment.dedupe_receipt.min_acknowledged_at_unix_nano
                != segment.min_acknowledged_at_unix_nano
            || segment.dedupe_receipt.max_acknowledged_at_unix_nano
                != segment.max_acknowledged_at_unix_nano
            || !segment_ids.insert(segment.source.segment_id.clone())
        {
            bail!("Sift archive manifest has an invalid segment object");
        }
        event_count = event_count
            .checked_add(segment.source.event_count)
            .context("Sift archive manifest event count exhausted u64")?;
        retained_watermarks.include(segment.signal, segment.source.last_cursor);
    }
    if !manifest.segments.is_empty() || manifest.segment_count == 0 {
        let observed_oldest = manifest
            .segments
            .iter()
            .map(|segment| segment.source.min_event_time_unix_nano)
            .min();
        let oldest_matches = match manifest.retention_scan {
            Some(_) => match (manifest.oldest_event_time_unix_nano, observed_oldest) {
                (Some(lower_bound), Some(observed)) => lower_bound <= observed,
                (None, None) => true,
                _ => false,
            },
            None => observed_oldest == manifest.oldest_event_time_unix_nano,
        };
        if event_count != manifest.event_count
            || retained_watermarks != manifest.retained_watermarks
            || !oldest_matches
            || manifest.segments.len() as u64 != manifest.segment_count
        {
            bail!("Sift archive manifest segment totals do not match retained event metadata");
        }
    }
    let mut blob_hashes = BTreeSet::new();
    for blob in &manifest.blobs {
        if !blob.object_uri.starts_with("gs://")
            || blob.reference.size == 0
            || !blob_hashes.insert(blob.reference.hash.clone())
        {
            bail!("Sift archive manifest has an invalid blob object");
        }
    }
    if (!manifest.blobs.is_empty() || manifest.blob_count == 0)
        && manifest.blobs.len() as u64 != manifest.blob_count
    {
        bail!("Sift archive manifest blob count does not match its catalog root");
    }
    let mut prior_gc_uri: Option<&str> = None;
    for uri in &manifest.gc_object_uris {
        if !uri.starts_with("gs://") || prior_gc_uri.is_some_and(|prior| prior >= uri.as_str()) {
            bail!("Sift archive manifest has an invalid or unsorted GC object URI");
        }
        service_backup::GcsSink::from_exact_uri(uri)
            .with_context(|| format!("validate archive manifest GC object URI {uri}"))?;
        prior_gc_uri = Some(uri);
    }
    if (!manifest.gc_object_uris.is_empty() || manifest.gc_object_count == 0)
        && manifest.gc_object_uris.len() as u64 != manifest.gc_object_count
    {
        bail!("Sift archive manifest GC count does not match its catalog root");
    }
    Ok(())
}

fn event_time_unix_nano(event: &StoredEvent) -> Result<i64> {
    DateTime::parse_from_rfc3339(&event.event.occurred_at)
        .context("archive event occurred_at must be RFC3339")?
        .timestamp_nanos_opt()
        .context("archive event occurred_at is outside the nanosecond range")
}

fn acknowledgement_time_unix_nano(event: &StoredEvent) -> Result<i64> {
    DateTime::parse_from_rfc3339(&event.acknowledged_at)
        .context("archive event acknowledged_at must be RFC3339")?
        .timestamp_nanos_opt()
        .context("archive event acknowledged_at is outside the nanosecond range")
}

fn acknowledgement_time_bounds(events: &[StoredEvent]) -> Result<(i64, i64)> {
    let mut values = events.iter().map(acknowledgement_time_unix_nano);
    let first = values
        .next()
        .transpose()?
        .context("archive segment must contain at least one event")?;
    values.try_fold((first, first), |(minimum, maximum), value| {
        let value = value?;
        anyhow::Ok((minimum.min(value), maximum.max(value)))
    })
}

fn archive_gc_pending_for_receipt(receipt: &ArchiveReceipt) -> Option<ArchiveGcPending> {
    let (Some(gc_plan_uri), Some(gc_plan_root)) = (
        receipt.manifest.gc_plan_uri.clone(),
        receipt.manifest.gc_plan_root.clone(),
    ) else {
        return None;
    };
    Some(ArchiveGcPending {
        format_version: ARCHIVE_GC_FORMAT_VERSION,
        replacement_manifest_uri: receipt.manifest_uri.clone(),
        replacement_manifest_sha256: receipt.manifest_sha256.clone(),
        gc_plan_uri,
        gc_plan_root,
        cursor: None,
    })
}

fn write_archive_gc_pending(root: &Path, receipt: &ArchiveReceipt) -> Result<()> {
    let path = root.join(ARCHIVE_GC_PATH);
    match archive_gc_pending_for_receipt(receipt) {
        Some(pending) => persist_archive_gc_pending(&path, &pending),
        None => remove_archive_gc_pending(&path),
    }
}

fn stage_archive_gc_pending(root: &Path, receipt: &ArchiveReceipt) -> Result<()> {
    let path = root.join(ARCHIVE_GC_STAGED_PATH);
    match archive_gc_pending_for_receipt(receipt) {
        Some(pending) => persist_archive_gc_pending(&path, &pending),
        None => remove_archive_gc_pending(&path),
    }
}

fn promote_archive_gc_pending(root: &Path, receipt: &ArchiveReceipt) -> Result<()> {
    let staged_path = root.join(ARCHIVE_GC_STAGED_PATH);
    let pending_path = root.join(ARCHIVE_GC_PATH);
    let Some(expected) = archive_gc_pending_for_receipt(receipt) else {
        remove_archive_gc_pending(&staged_path)?;
        return remove_archive_gc_pending(&pending_path);
    };
    let staged = read_archive_gc_pending(&staged_path)
        .context("promote committed archive GC staged intent")?;
    if staged != expected {
        bail!("archive GC staged intent does not match the committed manifest");
    }
    std::fs::rename(&staged_path, &pending_path)
        .context("atomically promote committed archive GC intent")?;
    set_private_file(&pending_path)?;
    storage_durable::sync_parent_dir(&pending_path)
}

pub(crate) fn reconcile_staged_archive_gc(root: &Path) -> Result<()> {
    reconcile_completed_archive_upload_intent(root)?;
    let staged_path = root.join(ARCHIVE_GC_STAGED_PATH);
    if !staged_path.exists() {
        return Ok(());
    }
    let staged = read_archive_gc_pending(&staged_path)?;
    let Some(committed) = read_commit_state(root)? else {
        return remove_archive_gc_pending(&staged_path);
    };
    let receipt = ArchiveReceipt {
        manifest_uri: committed.manifest_uri,
        manifest_sha256: committed.manifest_sha256,
        manifest: committed.manifest,
    };
    if archive_gc_pending_for_receipt(&receipt).as_ref() == Some(&staged) {
        promote_archive_gc_pending(root, &receipt)
    } else {
        remove_archive_gc_pending(&staged_path)
    }
}

fn reconcile_completed_archive_upload_intent(root: &Path) -> Result<()> {
    let path = root.join(ARCHIVE_UPLOAD_INTENT_PATH);
    if !path.exists() {
        return Ok(());
    }
    let intent = read_archive_upload_intent(&path)?;
    let Some(committed) = read_commit_state(root)? else {
        return Ok(());
    };
    if committed.manifest_uri == intent.manifest_uri
        && committed.manifest.raft_snapshot_index == intent.captured_cursor
    {
        remove_archive_upload_intent(&path)?;
    } else if intent.source_manifest_uri.as_deref() != Some(committed.manifest_uri.as_str())
        || intent.source_manifest_sha256.as_deref() != Some(committed.manifest_sha256.as_str())
    {
        // The local commit advanced from a different path, such as retention,
        // before this manifest-last upload recorded its receipt. The upload is
        // now an unreachable orphan. Keeping its intent would block every
        // later archive because its source identity can never match again.
        remove_archive_upload_intent(&path)?;
    }
    Ok(())
}

pub(crate) fn install_archive_gc_plan(root: &Path, receipt: &ArchiveReceipt) -> Result<()> {
    write_archive_gc_pending(root, receipt)
}

/// Remove local archive-deletion authority when a quorum-only checkpoint is
/// installed. The immutable objects remain valid recovery sources until a
/// later all-voter checkpoint installs the exact plan again.
pub(crate) fn withhold_archive_gc_plan(root: &Path) -> Result<()> {
    let path = root.join(ARCHIVE_GC_PATH);
    match std::fs::remove_file(&path) {
        Ok(()) => storage_durable::sync_parent_dir(&path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn archive_gc_pending(root: &Path) -> bool {
    root.join(ARCHIVE_GC_PATH).exists()
}

pub(crate) fn cleanup_orphan_spills(root: &Path) -> Result<usize> {
    let tmp = root.join("tmp");
    if !tmp.exists() {
        return Ok(0);
    }
    storage_durable::reject_symlink(&tmp)?;
    let mut removed = 0_usize;
    for entry in std::fs::read_dir(&tmp)
        .with_context(|| format!("list Sift temporary directory {}", tmp.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !ARCHIVE_SPILL_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
        {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || !file_type.is_dir() {
            bail!("archive spill path is not a real directory");
        }
        std::fs::remove_dir_all(entry.path())
            .with_context(|| format!("remove orphan archive spill {}", entry.path().display()))?;
        removed = removed.saturating_add(1);
    }
    if removed > 0 {
        storage_durable::sync_parent_dir(tmp.join("spill-cleanup"))?;
    }
    Ok(removed)
}

fn local_blob_gc_pending_for_receipt(receipt: &ArchiveReceipt) -> Option<LocalBlobGcPending> {
    let (Some(gc_plan_uri), Some(gc_plan_root)) = (
        receipt.manifest.gc_plan_uri.clone(),
        receipt.manifest.gc_plan_root.clone(),
    ) else {
        return None;
    };
    Some(LocalBlobGcPending {
        format_version: LOCAL_BLOB_GC_FORMAT_VERSION,
        replacement_manifest_uri: receipt.manifest_uri.clone(),
        replacement_manifest_sha256: receipt.manifest_sha256.clone(),
        gc_plan_uri,
        gc_plan_root,
        plan_cursor: None,
        plan_exhausted: false,
        candidates: Vec::new(),
        scan_start_cursor: receipt.manifest.raft_snapshot_index,
        scanned_through_cursor: receipt.manifest.raft_snapshot_index,
    })
}

fn ensure_local_blob_gc_pending(root: &Path) -> Result<()> {
    let path = root.join(LOCAL_BLOB_GC_PATH);
    if path.exists() {
        // Finish the prior committed plan before a newer manifest installs
        // its plan. Replacing this cursor can leak the older local blobs.
        read_local_blob_gc_pending(&path)?;
        return Ok(());
    }
    let Some(committed) = read_commit_state(root)? else {
        return Ok(());
    };
    let complete_path = root.join(LOCAL_BLOB_GC_COMPLETE_PATH);
    if complete_path.exists() {
        let complete = read_local_blob_gc_complete(&complete_path)?;
        if complete.replacement_manifest_uri == committed.manifest_uri
            && complete.replacement_manifest_sha256 == committed.manifest_sha256
        {
            remove_local_blob_live_index(root, &complete.replacement_manifest_sha256)?;
            return Ok(());
        }
    }
    let receipt = ArchiveReceipt {
        manifest_uri: committed.manifest_uri,
        manifest_sha256: committed.manifest_sha256,
        manifest: committed.manifest,
    };
    match local_blob_gc_pending_for_receipt(&receipt) {
        Some(pending) => persist_local_blob_gc_pending(&path, &pending),
        None => remove_local_blob_gc_pending(&path),
    }
}

#[doc(hidden)]
pub fn resume_local_blob_gc_batch(
    journal: &crate::DurableJournal,
    max_plan_entries: usize,
    max_scan_events: usize,
) -> Result<(usize, bool)> {
    if max_plan_entries == 0 || max_scan_events == 0 {
        bail!("local blob GC limits must be greater than zero");
    }
    ensure_local_blob_gc_pending(journal.data_dir())?;
    let path = journal.data_dir().join(LOCAL_BLOB_GC_PATH);
    if !path.exists() {
        return Ok((0, true));
    }
    let mut pending = read_local_blob_gc_pending(&path)?;
    committed_status(journal.data_dir())?
        .context("local blob GC requires a committed archive receipt")?;

    let (references, scanned_through, _scan_exhausted) =
        journal.scan_blob_references_page(pending.scanned_through_cursor, max_scan_events)?;
    for hash in references {
        mark_local_blob_live(
            journal.data_dir(),
            &pending.replacement_manifest_sha256,
            &hash,
        )?;
    }
    pending.scanned_through_cursor = scanned_through;
    persist_local_blob_gc_pending(&path, &pending)?;

    if pending.candidates.is_empty() && !pending.plan_exhausted {
        let catalog = catalog_for_uri(&pending.gc_plan_uri)?;
        let mut reader = match pending.plan_cursor.as_deref() {
            Some(cursor) => catalog.reader_after(&pending.gc_plan_root, cursor)?,
            None => catalog.reader(&pending.gc_plan_root)?,
        };
        let mut inspected = 0_usize;
        while inspected < max_plan_entries {
            let Some(entry) = reader.next() else {
                pending.plan_exhausted = true;
                break;
            };
            let entry = entry?;
            pending.plan_cursor = Some(entry.key);
            inspected += 1;
            let uri = String::from_utf8(entry.value).context("decode local blob GC object URI")?;
            if let Some(hash) = blob_hash_from_archive_uri(&uri) {
                pending.candidates.push(hash);
            }
        }
        pending.candidates.sort();
        pending.candidates.dedup();
        if pending.candidates.is_empty() {
            if pending.plan_exhausted {
                complete_local_blob_gc_plan(journal.data_dir(), &path, &pending)?;
                ensure_local_blob_gc_pending(journal.data_dir())?;
                return Ok((0, !path.exists()));
            }
            persist_local_blob_gc_pending(&path, &pending)?;
            return Ok((0, false));
        }
        persist_local_blob_gc_pending(&path, &pending)?;
    }

    let (scanned, removed, complete_batch) = journal.finalize_blob_candidates_with_index(
        &pending.candidates,
        pending.scanned_through_cursor,
        10_000,
        |hash| {
            mark_local_blob_live(
                journal.data_dir(),
                &pending.replacement_manifest_sha256,
                hash,
            )
        },
        |hash| {
            local_blob_is_live(
                journal.data_dir(),
                &pending.replacement_manifest_sha256,
                hash,
            )
        },
    )?;
    pending.scanned_through_cursor = scanned;
    if complete_batch {
        pending.candidates.clear();
        if pending.plan_exhausted {
            complete_local_blob_gc_plan(journal.data_dir(), &path, &pending)?;
            ensure_local_blob_gc_pending(journal.data_dir())?;
            return Ok((removed, !path.exists()));
        }
    }
    persist_local_blob_gc_pending(&path, &pending)?;
    Ok((removed, false))
}

pub(crate) fn finish_local_blob_gc(journal: &crate::DurableJournal) -> Result<usize> {
    let mut removed = 0_usize;
    loop {
        let (batch_removed, complete) = resume_local_blob_gc_batch(journal, 128, 1_280_000)?;
        removed = removed.saturating_add(batch_removed);
        if complete {
            return Ok(removed);
        }
    }
}

fn complete_local_blob_gc_plan(
    root: &Path,
    pending_path: &Path,
    pending: &LocalBlobGcPending,
) -> Result<()> {
    let complete = LocalBlobGcComplete {
        format_version: LOCAL_BLOB_GC_FORMAT_VERSION,
        replacement_manifest_uri: pending.replacement_manifest_uri.clone(),
        replacement_manifest_sha256: pending.replacement_manifest_sha256.clone(),
    };
    persist_local_blob_gc_complete(&root.join(LOCAL_BLOB_GC_COMPLETE_PATH), &complete)?;
    remove_local_blob_gc_pending(pending_path)?;
    remove_local_blob_live_index(root, &pending.replacement_manifest_sha256)
}

fn blob_hash_from_archive_uri(uri: &str) -> Option<String> {
    let file = uri.rsplit('/').next()?;
    let digest = file.strip_suffix(".blob")?;
    (digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| format!("sha256:{digest}"))
}

fn local_blob_live_index_root(root: &Path, replacement_sha256: &str) -> Result<PathBuf> {
    if replacement_sha256.len() != 64
        || !replacement_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("local blob GC replacement digest is invalid");
    }
    Ok(root
        .join("indexes")
        .join("local-blob-gc")
        .join(replacement_sha256))
}

fn local_blob_marker_path(root: &Path, replacement_sha256: &str, hash: &str) -> Result<PathBuf> {
    let digest = hash
        .strip_prefix("sha256:")
        .context("local blob hash must use sha256")?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("local blob hash is invalid");
    }
    Ok(local_blob_live_index_root(root, replacement_sha256)?
        .join(&digest[..2])
        .join(&digest[2..]))
}

fn mark_local_blob_live(root: &Path, replacement_sha256: &str, hash: &str) -> Result<()> {
    let path = local_blob_marker_path(root, replacement_sha256, hash)?;
    if path.exists() {
        let metadata = std::fs::symlink_metadata(&path)
            .with_context(|| format!("inspect local blob live marker {}", path.display()))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            bail!("local blob live marker is not a regular file");
        }
        return Ok(());
    }
    let parent = path
        .parent()
        .context("local blob live marker has no parent")?;
    let index_root = local_blob_live_index_root(root, replacement_sha256)?;
    std::fs::create_dir_all(&index_root)
        .with_context(|| format!("create local blob live index {}", index_root.display()))?;
    storage_durable::reject_symlink(&index_root)?;
    storage_durable::set_private_directory_mode(&index_root)?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create local blob live shard {}", parent.display()))?;
    storage_durable::reject_symlink(parent)?;
    storage_durable::set_private_directory_mode(parent)?;
    storage_durable::atomic_write(&path, b"live\n", storage_durable::FsyncPolicy::Always)?;
    storage_durable::set_private_file_mode(&path)
}

fn local_blob_is_live(root: &Path, replacement_sha256: &str, hash: &str) -> Result<bool> {
    let path = local_blob_marker_path(root, replacement_sha256, hash)?;
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => bail!("local blob live marker is not a regular file"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => {
            Err(error).with_context(|| format!("inspect local blob live marker {}", path.display()))
        }
    }
}

fn remove_local_blob_live_index(root: &Path, replacement_sha256: &str) -> Result<()> {
    let path = local_blob_live_index_root(root, replacement_sha256)?;
    storage_durable::reject_symlink(&path)?;
    match std::fs::remove_dir_all(&path) {
        Ok(()) => storage_durable::sync_parent_dir(&path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error)
            .with_context(|| format!("remove completed local blob live index {}", path.display())),
    }
}

fn read_local_blob_gc_pending(path: &Path) -> Result<LocalBlobGcPending> {
    let pending: LocalBlobGcPending =
        serde_json::from_slice(&std::fs::read(path)?).context("decode local blob GC progress")?;
    validate_local_blob_gc_pending(&pending)?;
    Ok(pending)
}

fn validate_local_blob_gc_pending(pending: &LocalBlobGcPending) -> Result<()> {
    if pending.format_version != LOCAL_BLOB_GC_FORMAT_VERSION
        || !pending.replacement_manifest_uri.starts_with("gs://")
        || pending.replacement_manifest_sha256.len() != 64
        || !pending
            .replacement_manifest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || !pending.gc_plan_uri.starts_with("gs://")
        || pending.scanned_through_cursor < pending.scan_start_cursor
        || pending.candidates.len() > 128
        || pending.candidates.iter().any(|hash| {
            hash.strip_prefix("sha256:").is_none_or(|digest| {
                digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
        })
    {
        bail!("local blob GC progress has invalid fields");
    }
    Ok(())
}

fn read_local_blob_gc_complete(path: &Path) -> Result<LocalBlobGcComplete> {
    let complete: LocalBlobGcComplete = serde_json::from_slice(&std::fs::read(path)?)
        .context("decode completed local blob GC receipt")?;
    validate_local_blob_gc_complete(&complete)?;
    Ok(complete)
}

fn validate_local_blob_gc_complete(complete: &LocalBlobGcComplete) -> Result<()> {
    if complete.format_version != LOCAL_BLOB_GC_FORMAT_VERSION
        || !complete.replacement_manifest_uri.starts_with("gs://")
        || complete.replacement_manifest_sha256.len() != 64
        || !complete
            .replacement_manifest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("completed local blob GC receipt has invalid fields");
    }
    Ok(())
}

fn persist_local_blob_gc_complete(path: &Path, complete: &LocalBlobGcComplete) -> Result<()> {
    validate_local_blob_gc_complete(complete)?;
    storage_durable::atomic_write(
        path,
        &serde_json::to_vec_pretty(complete)?,
        storage_durable::FsyncPolicy::Always,
    )?;
    set_private_file(path)
}

fn persist_local_blob_gc_pending(path: &Path, pending: &LocalBlobGcPending) -> Result<()> {
    validate_local_blob_gc_pending(pending)?;
    let bytes = serde_json::to_vec_pretty(pending)?;
    if bytes.len() >= 64 * 1024 {
        bail!("local blob GC progress exceeds 64 KiB");
    }
    storage_durable::atomic_write(path, &bytes, storage_durable::FsyncPolicy::Always)?;
    set_private_file(path)
}

fn remove_local_blob_gc_pending(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => storage_durable::sync_parent_dir(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

/// Delete obsolete remote archive objects after the replacement checkpoint is
/// durable on every voter. A single-node caller may use its local checkpoint
/// as the same barrier.
#[doc(hidden)]
pub fn finalize_archive_gc_after_checkpoint(root: &Path) -> Result<usize> {
    let (deleted, complete) = finalize_archive_gc_batch_after_checkpoint(root, usize::MAX)?;
    if !complete {
        bail!("archive GC did not finish its unbounded finalization pass");
    }
    Ok(deleted)
}

/// Delete at most `max_objects` obsolete archive objects and persist progress
/// after every successful delete. A later process can resume from the saved
/// catalog cursor without keeping the full cleanup plan in memory.
#[doc(hidden)]
pub fn finalize_archive_gc_batch_after_checkpoint(
    root: &Path,
    max_objects: usize,
) -> Result<(usize, bool)> {
    if max_objects == 0 {
        bail!("archive GC batch size must be greater than zero");
    }
    let path = root.join(ARCHIVE_GC_PATH);
    if !path.exists() {
        return Ok((0, true));
    }
    let mut pending = read_archive_gc_pending(&path)?;
    let committed =
        committed_status(root)?.context("archive GC requires the replacement manifest receipt")?;
    if committed.manifest_uri != pending.replacement_manifest_uri
        || committed.manifest_sha256 != pending.replacement_manifest_sha256
    {
        bail!("archive GC replacement manifest is not the committed archive identity");
    }
    let mut deleted = 0_usize;
    let catalog = catalog_for_uri(&pending.gc_plan_uri)?;
    let mut reader = match pending.cursor.as_deref() {
        Some(cursor) => catalog.reader_after(&pending.gc_plan_root, cursor)?,
        None => catalog.reader(&pending.gc_plan_root)?,
    };
    for entry in &mut reader {
        let entry = entry?;
        if deleted == max_objects {
            return Ok((deleted, false));
        }
        let uri = String::from_utf8(entry.value).context("decode archive GC object URI")?;
        let (sink, key) = service_backup::GcsSink::from_exact_uri(&uri)
            .with_context(|| format!("validate archive GC object URI {uri}"))?;
        sink.delete_object(&key)
            .with_context(|| format!("delete obsolete archive object {uri}"))?;
        pending.cursor = Some(entry.key);
        deleted += 1;
        persist_archive_gc_pending(&path, &pending)?;
    }
    remove_archive_gc_pending(&path)?;
    Ok((deleted, true))
}

fn read_archive_gc_pending(path: &Path) -> Result<ArchiveGcPending> {
    let pending: ArchiveGcPending = serde_json::from_slice(
        &std::fs::read(path)
            .with_context(|| format!("read archive GC receipt {}", path.display()))?,
    )
    .with_context(|| format!("decode archive GC receipt {}", path.display()))?;
    validate_archive_gc_pending(&pending)?;
    Ok(pending)
}

fn validate_archive_gc_pending(pending: &ArchiveGcPending) -> Result<()> {
    if pending.format_version != ARCHIVE_GC_FORMAT_VERSION
        || !pending.replacement_manifest_uri.starts_with("gs://")
        || pending.replacement_manifest_sha256.len() != 64
        || !pending
            .replacement_manifest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || !pending.gc_plan_uri.starts_with("gs://")
    {
        bail!("archive GC pending root has invalid identity fields");
    }
    Ok(())
}

fn persist_archive_gc_pending(path: &Path, pending: &ArchiveGcPending) -> Result<()> {
    validate_archive_gc_pending(pending)?;
    let bytes = serde_json::to_vec_pretty(pending)?;
    if bytes.len() >= 64 * 1024 {
        bail!("archive GC pending root exceeds 64 KiB");
    }
    storage_durable::atomic_write(path, &bytes, storage_durable::FsyncPolicy::Always)?;
    set_private_file(path)
}

fn remove_archive_gc_pending(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => storage_durable::sync_parent_dir(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn read_commit_state(root: &Path) -> Result<Option<ArchiveCommitState>> {
    let path = root.join(ARCHIVE_COMMIT_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let state: ArchiveCommitState = serde_json::from_slice(
        &std::fs::read(&path)
            .with_context(|| format!("read archive commit receipt {}", path.display()))?,
    )
    .with_context(|| format!("decode archive commit receipt {}", path.display()))?;
    if state.format_version != ARCHIVE_COMMIT_FORMAT_VERSION {
        bail!(
            "unsupported archive commit format {}; expected {}",
            state.format_version,
            ARCHIVE_COMMIT_FORMAT_VERSION
        );
    }
    if state.manifest_uri.trim().is_empty()
        || state.manifest_sha256.len() != 64
        || !state
            .manifest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("archive commit receipt has invalid manifest identity");
    }
    if state.manifest.format_version != ARCHIVE_FORMAT_VERSION
        || state.manifest.watermarks != state.watermarks
    {
        bail!("archive commit receipt has invalid embedded manifest");
    }
    set_private_file(&path)?;
    Ok(Some(state))
}

fn read_local_commit_state(root: &Path) -> Result<Option<LocalSegmentCommitState>> {
    let path = root.join(LOCAL_COMMIT_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let state: LocalSegmentCommitState = serde_json::from_slice(
        &std::fs::read(&path)
            .with_context(|| format!("read local segment commit {}", path.display()))?,
    )
    .with_context(|| format!("decode local segment commit {}", path.display()))?;
    if state.format_version != LOCAL_COMMIT_FORMAT_VERSION {
        bail!(
            "unsupported local segment commit format {}; expected {}",
            state.format_version,
            LOCAL_COMMIT_FORMAT_VERSION
        );
    }
    let mut watermarks = ArchiveWatermarks::default();
    for segment in &state.segments {
        if segment.manifest.state != super::SegmentState::Sealed {
            bail!("local segment commit contains a non-sealed segment");
        }
        watermarks.include(segment.signal, segment.manifest.last_cursor);
        let manifest_path = root
            .join("segments")
            .join(signal_storage_dir(segment.signal))
            .join("manifests")
            .join(format!("{}.json", segment.manifest.segment_id));
        let current: SegmentManifest =
            serde_json::from_slice(&std::fs::read(&manifest_path).with_context(|| {
                format!(
                    "read committed local segment manifest {}",
                    manifest_path.display()
                )
            })?)
            .with_context(|| {
                format!(
                    "decode committed local segment manifest {}",
                    manifest_path.display()
                )
            })?;
        if current != segment.manifest {
            bail!(
                "committed local segment {} disagrees with its manifest",
                segment.manifest.segment_id
            );
        }
    }
    if watermarks != state.watermarks || watermarks.max_cursor() != state.snapshot_index {
        bail!("local segment commit watermarks do not match its manifest set");
    }
    set_private_file(&path)?;
    Ok(Some(state))
}

fn retire_local_commit(root: &Path) -> Result<()> {
    let path = root.join(LOCAL_COMMIT_PATH);
    match std::fs::remove_file(&path) {
        Ok(()) => storage_durable::sync_parent_dir(&path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| {
            format!(
                "retire local segment commit after remote archive {}",
                path.display()
            )
        }),
    }
}

fn signal_storage_dir(signal: SignalKind) -> &'static str {
    match signal {
        SignalKind::Log => "logs",
        SignalKind::Metric => "metrics",
        SignalKind::Span => "traces",
    }
}

fn record_archive_commit(root: &Path, receipt: &ArchiveReceipt) -> Result<ArchiveWatermarks> {
    let watermarks = receipt.manifest.watermarks;
    let state = ArchiveCommitState {
        format_version: ARCHIVE_COMMIT_FORMAT_VERSION,
        manifest_uri: receipt.manifest_uri.clone(),
        manifest_sha256: receipt.manifest_sha256.clone(),
        committed_at: Utc::now().to_rfc3339(),
        watermarks,
        manifest: receipt.manifest.clone(),
    };
    let path = root.join(ARCHIVE_COMMIT_PATH);
    let bytes = serde_json::to_vec_pretty(&state)?;
    if bytes.len() >= 64 * 1024 {
        bail!("archive commit root exceeds 64 KiB");
    }
    storage_durable::atomic_write(&path, &bytes, storage_durable::FsyncPolicy::Always)?;
    set_private_file(&path)?;
    Ok(watermarks)
}

pub(crate) fn adopt_verified_archive_receipt(
    root: &Path,
    receipt: &ArchiveReceipt,
) -> Result<ArchiveWatermarks> {
    validate_archive_manifest(&receipt.manifest)?;
    let catalog = catalog_for_uri(&receipt.manifest.catalog_uri)?;
    let mut reader = catalog.reader(&receipt.manifest.catalog_root)?;
    let _ = reader.next().transpose()?;
    record_archive_commit(root, receipt)
}

/// Restore through a resumable staging directory inside the target volume.
/// The live data-root names are published only after every remote object and
/// digest has been verified. A crash during download restarts the private
/// stage. A crash during publication resumes the remaining renames.
pub fn restore_gcs(manifest_uri: &str, target: impl AsRef<Path>) -> Result<ArchiveManifest> {
    let target = target.as_ref();
    std::fs::create_dir_all(target)
        .with_context(|| format!("create cold restore target {}", target.display()))?;
    set_private_dir(target)?;
    let state_path = target.join(RESTORE_STATE_PATH);
    let stage = target.join(RESTORE_STAGE_DIR);
    let existing = read_restore_state(&state_path)?;
    match existing {
        Some(state) => {
            if state.manifest_uri != manifest_uri {
                bail!(
                    "cold restore staging belongs to a different manifest: {}",
                    state.manifest_uri
                );
            }
            if state.phase == RestorePhase::Ready {
                publish_restore_stage(target)?;
                return fetch_verified_committed_root(target)?
                    .context("published cold restore has no committed archive receipt");
            }
            validate_building_restore_target(target)?;
            remove_restore_stage(&stage)?;
        }
        None => {
            require_empty_restore_target(target)?;
            persist_restore_state(
                &state_path,
                &RestoreState {
                    format_version: RESTORE_STATE_FORMAT_VERSION,
                    manifest_uri: manifest_uri.to_string(),
                    phase: RestorePhase::Building,
                },
            )?;
        }
    }
    std::fs::create_dir(&stage)
        .with_context(|| format!("create cold restore stage {}", stage.display()))?;
    set_private_dir(&stage)?;
    storage_durable::sync_parent_dir(&stage)?;

    let manifest = restore_gcs_into_empty(manifest_uri, &stage)?;
    persist_restore_state(
        &state_path,
        &RestoreState {
            format_version: RESTORE_STATE_FORMAT_VERSION,
            manifest_uri: manifest_uri.to_string(),
            phase: RestorePhase::Ready,
        },
    )?;
    publish_restore_stage(target)?;
    Ok(manifest)
}

fn restore_gcs_into_empty(manifest_uri: &str, target: &Path) -> Result<ArchiveManifest> {
    require_empty_restore_target(target)?;
    let manifest_bytes = service_backup::fetch_backup_object(manifest_uri)?;
    let manifest_sha256 = sha256(&manifest_bytes);
    let mut manifest: ArchiveManifest =
        serde_json::from_slice(&manifest_bytes).context("decode Sift archive manifest")?;
    validate_archive_manifest(&manifest)?;

    let layout = DataLayout::open(target, StorageRole::All)?;
    if layout.manifest().cluster_id == manifest.source_cluster_id {
        bail!("cold restore must create a new Sift cluster ID");
    }
    drop(layout);
    shard::write_epoch_maps(target, &manifest.epochs)?;
    let journal = crate::DurableJournal::open(target)?;
    let hot_cutoff_nanos = (Utc::now() - chrono::Duration::days(30))
        .timestamp_nanos_opt()
        .context("30-day hot restore cutoff is outside the nanosecond range")?;
    let catalog = catalog_for_uri(&manifest.catalog_uri)?;
    let stream = |signal| -> Result<ArchiveSignalStream> {
        let prefix = format!("segment/{signal}/");
        Ok(ArchiveSignalStream::new(
            target,
            catalog.reader_after(&manifest.catalog_root, &prefix)?,
            signal,
        ))
    };
    let mut streams = [
        stream(SignalKind::Log)?,
        stream(SignalKind::Metric)?,
        stream(SignalKind::Span)?,
    ];
    let mut page = Vec::with_capacity(10_000);
    let spill_parent = target.join("tmp");
    let mut hot_blob_hashes = SpillCatalog::new(&spill_parent, "restore-hot-blobs-")?;
    let mut blob_reference_counts = SpillCatalog::new(&spill_parent, "restore-blob-counts-")?;
    let mut event_count = 0_u64;
    let mut last_cursor = 0_u64;
    let mut event_id_digest = [0_u8; 32];
    let mut event_content_digest = [0_u8; 32];
    let mut restored_watermarks = ArchiveWatermarks::default();
    let mut oldest_event_time_unix_nano = None::<i64>;
    loop {
        let mut next = None::<(usize, u64)>;
        for (index, stream) in streams.iter_mut().enumerate() {
            if let Some(cursor) = stream.peek_cursor()? {
                if next.is_none_or(|(_, current)| cursor < current) {
                    next = Some((index, cursor));
                }
            }
        }
        let Some((index, cursor)) = next else {
            break;
        };
        if cursor <= last_cursor {
            bail!("archive cursor {cursor} is not globally strictly increasing");
        }
        let event = streams[index]
            .pop_event()?
            .context("archive stream head disappeared during restore")?;
        event.event.validate()?;
        last_cursor = cursor;
        event_count = event_count.saturating_add(1);
        include_event_id(&mut event_id_digest, &event.event.event_id);
        include_event_content(&mut event_content_digest, &event)?;
        let event_time = event_time_unix_nano(&event)?;
        oldest_event_time_unix_nano = Some(
            oldest_event_time_unix_nano
                .map(|oldest| oldest.min(event_time))
                .unwrap_or(event_time),
        );
        restored_watermarks.include(event.event.signal, event.cursor);
        for reference in &event.event.blob_refs {
            blob_reference_counts.add_u64(format!("blob/{}", reference.hash), 1)?;
        }
        page.push(event);
        if page.len() == 10_000 {
            restore_archive_page(
                &journal,
                std::mem::take(&mut page),
                hot_cutoff_nanos,
                &mut hot_blob_hashes,
            )?;
            page = Vec::with_capacity(10_000);
        }
    }
    if !page.is_empty() {
        restore_archive_page(&journal, page, hot_cutoff_nanos, &mut hot_blob_hashes)?;
    }
    let oldest_matches = match manifest.retention_scan {
        Some(_) => match (
            manifest.oldest_event_time_unix_nano,
            oldest_event_time_unix_nano,
        ) {
            (Some(lower_bound), Some(observed)) => lower_bound <= observed,
            (None, None) => true,
            _ => false,
        },
        None => oldest_event_time_unix_nano == manifest.oldest_event_time_unix_nano,
    };
    if event_count != manifest.event_count
        || hex::encode(event_id_digest) != manifest.event_id_sha256
        || hex::encode(event_content_digest) != manifest.event_content_sha256
        || !oldest_matches
        || last_cursor != manifest.retained_watermarks.max_cursor()
        || restored_watermarks != manifest.retained_watermarks
        || last_cursor > manifest.raft_snapshot_index
    {
        bail!("Sift archive manifest count, digest, watermark, or Raft snapshot index mismatch");
    }

    let blob_store = BlobStore::open(target, 65_536)?;
    let mut restored_blobs = 0_u64;
    for entry in catalog.reader(&manifest.catalog_root)? {
        let ArchiveCatalogItem::Blob(blob) = decode_archive_catalog_entry(entry?)? else {
            continue;
        };
        restored_blobs = restored_blobs.saturating_add(1);
        let actual_references =
            blob_reference_counts.get_u64(&format!("blob/{}", blob.reference.hash))?;
        if actual_references == 0
            || (blob.reference_count > 0 && blob.reference_count != actual_references)
        {
            bail!(
                "archive blob {} reference count disagrees with restored events",
                blob.reference.hash
            );
        }
        let bytes = service_backup::fetch_backup_object(&blob.object_uri)?;
        let actual_hash = format!("sha256:{}", sha256(&bytes));
        if actual_hash != blob.reference.hash || bytes.len() as u64 != blob.reference.size {
            bail!(
                "restored blob {} failed hash/size verification",
                blob.reference.hash
            );
        }
        if hot_blob_hashes.contains_hash(&blob.reference.hash)? {
            let restored = blob_store.put(&bytes, blob.reference.encoding.clone())?;
            if restored != blob.reference {
                bail!(
                    "restored hot blob {} changed its content reference",
                    blob.reference.hash
                );
            }
        }
    }
    if blob_reference_counts.len() != restored_blobs || restored_blobs != manifest.blob_count {
        bail!("restored events reference blobs missing from the archive manifest");
    }

    let mut restored_receipt_segments = 0_u64;
    for entry in catalog.reader(&manifest.catalog_root)? {
        let ArchiveCatalogItem::Receipt(receipt) = decode_archive_catalog_entry(entry?)? else {
            continue;
        };
        let rows = fetch_dedupe_receipts(&receipt)?;
        journal.restore_archive_receipts(&rows, manifest.raft_snapshot_index)?;
        restored_receipt_segments = restored_receipt_segments.saturating_add(1);
    }
    if restored_receipt_segments != manifest.dedupe_receipt_count {
        bail!("archive dedupe receipt count disagrees with its manifest");
    }

    journal.set_restored_archive_head(&manifest)?;
    let receipt = ArchiveReceipt {
        manifest_uri: manifest_uri.to_string(),
        manifest_sha256,
        manifest: manifest.clone(),
    };
    adopt_verified_archive_receipt(target, &receipt)?;
    drop(journal);
    let mut layout = DataLayout::open(target, StorageRole::All)?;
    layout.mark_restored_from(manifest_uri)?;
    manifest.segments.clear();
    manifest.blobs.clear();
    manifest.gc_object_uris.clear();
    Ok(manifest)
}

fn restore_archive_page(
    journal: &crate::DurableJournal,
    events: Vec<StoredEvent>,
    hot_cutoff_nanos: i64,
    hot_blob_hashes: &mut impl BlobHashSet,
) -> Result<()> {
    let mut hot = Vec::with_capacity(events.len());
    let mut cold = Vec::new();
    for event in events {
        if event_time_unix_nano(&event)? >= hot_cutoff_nanos {
            for reference in &event.event.blob_refs {
                hot_blob_hashes.insert_hash(&reference.hash)?;
            }
            hot.push(event);
        } else {
            cold.push(event);
        }
    }
    journal.restore_stored_page(hot)?;
    journal.restore_archive_dedupe_page(&cold)
}

struct ArchiveSignalStream {
    root: PathBuf,
    catalog: storage_segment::CatalogReader,
    prefix: String,
    signal: SignalKind,
    events: VecDeque<StoredEvent>,
    parquet: Option<ParquetRecordBatchReader>,
    last_cursor: Option<u64>,
    finished: bool,
}

impl ArchiveSignalStream {
    fn new(root: &Path, catalog: storage_segment::CatalogReader, signal: SignalKind) -> Self {
        Self::new_after(root, catalog, signal, 0)
    }

    fn new_after(
        root: &Path,
        catalog: storage_segment::CatalogReader,
        signal: SignalKind,
        after: u64,
    ) -> Self {
        Self {
            root: root.to_path_buf(),
            catalog,
            prefix: format!("segment/{signal}/"),
            signal,
            events: VecDeque::new(),
            parquet: None,
            last_cursor: Some(after),
            finished: false,
        }
    }

    fn peek_cursor(&mut self) -> Result<Option<u64>> {
        self.fill()?;
        Ok(self.events.front().map(|event| event.cursor))
    }

    fn pop_event(&mut self) -> Result<Option<StoredEvent>> {
        self.fill()?;
        let event = self.events.pop_front();
        if let Some(event) = &event {
            if self
                .last_cursor
                .is_some_and(|previous| event.cursor <= previous)
            {
                bail!("archive signal stream cursors are not strictly increasing");
            }
            self.last_cursor = Some(event.cursor);
        }
        Ok(event)
    }

    fn fill(&mut self) -> Result<()> {
        while self.events.is_empty() && !self.finished {
            if let Some(reader) = self.parquet.as_mut() {
                match reader.next() {
                    Some(batch) => {
                        let after = self.last_cursor.unwrap_or_default();
                        self.events = decode_parquet_batch(&batch?)?
                            .into_iter()
                            .filter(|event| event.cursor > after)
                            .collect();
                        continue;
                    }
                    None => {
                        self.parquet = None;
                        continue;
                    }
                }
            }
            let Some(entry) = self.catalog.next() else {
                self.finished = true;
                break;
            };
            let entry = entry?;
            if !entry.key.starts_with(&self.prefix) {
                self.finished = true;
                break;
            }
            let ArchiveCatalogItem::Segment(segment) = decode_archive_catalog_entry(entry)? else {
                bail!("archive segment prefix resolved to another catalog item");
            };
            if segment.signal != self.signal {
                bail!("archive segment prefix contains the wrong signal");
            }
            if segment.source.last_cursor <= self.last_cursor.unwrap_or_default() {
                continue;
            }
            let path = cached_segment_path(&self.root, &segment)?;
            verify_archive_segment_file(&segment, &path)?;
            self.parquet = Some(open_parquet_reader(&path)?);
        }
        Ok(())
    }
}

/// Bootstrap a fresh volume once. A normal pod restart sees the same
/// `restored_from` value and reuses the completed restore without rewriting it.
pub fn bootstrap_gcs_if_needed(
    manifest_uri: &str,
    target: impl AsRef<Path>,
) -> Result<Option<ArchiveManifest>> {
    let target = target.as_ref();
    if target.join(RESTORE_STATE_PATH).exists() {
        return restore_gcs(manifest_uri, target).map(Some);
    }
    let layout_path = target.join("layout.json");
    if layout_path.exists() {
        let layout: super::LayoutManifest = serde_json::from_slice(
            &std::fs::read(&layout_path)
                .with_context(|| format!("read bootstrap layout {}", layout_path.display()))?,
        )
        .with_context(|| format!("decode bootstrap layout {}", layout_path.display()))?;
        if layout.restored_from.as_deref() == Some(manifest_uri) {
            return Ok(None);
        }
        bail!(
            "bootstrap archive cannot overwrite an existing Sift data directory: {}",
            target.display()
        );
    }
    restore_gcs(manifest_uri, target).map(Some)
}

fn read_restore_state(path: &Path) -> Result<Option<RestoreState>> {
    if !path.exists() {
        return Ok(None);
    }
    let state: RestoreState = serde_json::from_slice(
        &std::fs::read(path)
            .with_context(|| format!("read cold restore state {}", path.display()))?,
    )
    .with_context(|| format!("decode cold restore state {}", path.display()))?;
    if state.format_version != RESTORE_STATE_FORMAT_VERSION
        || !state.manifest_uri.starts_with("gs://")
    {
        bail!("cold restore state has invalid identity fields");
    }
    set_private_file(path)?;
    Ok(Some(state))
}

fn persist_restore_state(path: &Path, state: &RestoreState) -> Result<()> {
    if state.format_version != RESTORE_STATE_FORMAT_VERSION
        || !state.manifest_uri.starts_with("gs://")
    {
        bail!("cold restore state has invalid identity fields");
    }
    let bytes = serde_json::to_vec_pretty(state)?;
    storage_durable::atomic_write(path, &bytes, storage_durable::FsyncPolicy::Always)?;
    set_private_file(path)
}

fn validate_building_restore_target(target: &Path) -> Result<()> {
    for entry in std::fs::read_dir(target)
        .with_context(|| format!("read interrupted cold restore target {}", target.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        if name == "lost+found" || name == RESTORE_STATE_PATH || name == RESTORE_STAGE_DIR {
            continue;
        }
        bail!(
            "interrupted cold restore target contains an unexpected entry: {}",
            entry.path().display()
        );
    }
    Ok(())
}

fn publish_restore_stage(target: &Path) -> Result<()> {
    let state_path = target.join(RESTORE_STATE_PATH);
    let state = read_restore_state(&state_path)?
        .context("cold restore publication requires a durable restore state")?;
    if state.phase != RestorePhase::Ready {
        bail!("cold restore stage is not ready for publication");
    }
    let stage = target.join(RESTORE_STAGE_DIR);
    if !stage.exists() {
        let layout: super::LayoutManifest = serde_json::from_slice(
            &std::fs::read(target.join("layout.json"))
                .context("resume cold restore publication without a layout")?,
        )
        .context("decode resumed cold restore layout")?;
        if layout.restored_from.as_deref() != Some(state.manifest_uri.as_str()) {
            bail!("resumed cold restore layout has the wrong source manifest");
        }
        remove_archive_gc_pending(&state_path)?;
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(&stage)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("cold restore stage is not a real directory");
    }
    let mut entries = std::fs::read_dir(&stage)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|path| path.file_name() == Some(std::ffi::OsStr::new("layout.json")));
    for source in entries {
        let name = source
            .file_name()
            .context("cold restore stage entry has no file name")?;
        let destination = target.join(name);
        if destination.exists() {
            bail!(
                "cold restore publication found an existing destination: {}",
                destination.display()
            );
        }
        std::fs::rename(&source, &destination).with_context(|| {
            format!(
                "publish cold restore entry {} to {}",
                source.display(),
                destination.display()
            )
        })?;
        storage_durable::sync_parent_dir(&destination)?;
    }
    std::fs::remove_dir(&stage)
        .with_context(|| format!("remove published cold restore stage {}", stage.display()))?;
    storage_durable::sync_parent_dir(&stage)?;
    remove_archive_gc_pending(&state_path)
}

fn remove_restore_stage(stage: &Path) -> Result<()> {
    match std::fs::symlink_metadata(stage) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            bail!("cold restore stage is not a real directory")
        }
        Ok(_) => std::fs::remove_dir_all(stage).with_context(|| {
            format!("remove interrupted cold restore stage {}", stage.display())
        })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    storage_durable::sync_parent_dir(stage)
}

fn require_empty_restore_target(target: &Path) -> Result<()> {
    if target.exists() {
        let entries = std::fs::read_dir(target)
            .with_context(|| format!("read cold restore target {}", target.display()))?;
        let mut has_owned_entry = false;
        for entry in entries {
            if entry?.file_name() != "lost+found" {
                has_owned_entry = true;
                break;
            }
        }
        if has_owned_entry {
            bail!(
                "cold restore requires an empty data directory: {}",
                target.display()
            );
        }
    }
    Ok(())
}

fn include_event_id(accumulator: &mut [u8; 32], event_id: &str) {
    let digest: [u8; 32] = Sha256::digest(event_id.as_bytes()).into();
    for (slot, byte) in accumulator.iter_mut().zip(digest) {
        *slot ^= byte;
    }
}

fn include_event_content(accumulator: &mut [u8; 32], event: &StoredEvent) -> Result<()> {
    let encoded =
        serde_json::to_vec(&event.event).context("encode archive event content digest")?;
    let digest: [u8; 32] = Sha256::digest(encoded).into();
    for (slot, byte) in accumulator.iter_mut().zip(digest) {
        *slot ^= byte;
    }
    Ok(())
}

fn decode_event_id_digest(encoded: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(encoded).context("decode committed archive event ID digest")?;
    bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("committed archive event ID digest must be 32 bytes"))
}

fn decode_event_content_digest(encoded: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(encoded).context("decode committed archive event content digest")?;
    bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("committed archive event content digest must be 32 bytes"))
}

fn portable_manifest(mut source: SegmentManifest, signal: SignalKind) -> SegmentManifest {
    source.local_path = PathBuf::from(format!("segments/{signal}/{}.framed", source.segment_id));
    source.object_uri = None;
    source
}

struct SiftParquetCodec;

impl storage_segment::RecordCodec<StoredEvent> for SiftParquetCodec {
    fn encode(&self, records: &[StoredEvent]) -> storage_segment::Result<Vec<u8>> {
        encode_parquet_inner(records).map_err(|error| storage_segment::SegmentError::Codec {
            message: error.to_string(),
        })
    }

    fn decode(&self, bytes: &[u8]) -> storage_segment::Result<Vec<StoredEvent>> {
        decode_parquet_inner(bytes).map_err(|error| storage_segment::SegmentError::Codec {
            message: error.to_string(),
        })
    }
}

struct SiftSignalPartitioner;

impl storage_segment::Partitioner<StoredEvent> for SiftSignalPartitioner {
    fn partition(&self, record: &StoredEvent) -> storage_segment::Result<String> {
        Ok(record.event.signal.to_string())
    }
}

fn parquet_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("cursor", DataType::UInt64, false),
        Field::new("acknowledged_at", DataType::Utf8, false),
        Field::new("event_id", DataType::Utf8, false),
        Field::new("project", DataType::Utf8, false),
        Field::new("environment", DataType::Utf8, false),
        Field::new("signal", DataType::Utf8, false),
        Field::new("occurred_at", DataType::Utf8, false),
        Field::new("event_json", DataType::Utf8, false),
    ]))
}

fn encode_parquet(events: &[StoredEvent]) -> Result<Vec<u8>> {
    storage_segment::RecordCodec::encode(&SiftParquetCodec, events).map_err(Into::into)
}

fn encode_parquet_inner(events: &[StoredEvent]) -> Result<Vec<u8>> {
    let event_json = events
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let columns: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(
            events.iter().map(|event| event.cursor).collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            events
                .iter()
                .map(|event| event.acknowledged_at.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            events
                .iter()
                .map(|event| event.event.event_id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            events
                .iter()
                .map(|event| event.event.project.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            events
                .iter()
                .map(|event| event.event.environment.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            events
                .iter()
                .map(|event| event.event.signal.to_string())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            events
                .iter()
                .map(|event| event.event.occurred_at.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(event_json)),
    ];
    let schema = parquet_schema();
    let batch = RecordBatch::try_new(schema.clone(), columns)?;
    let properties = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();
    let mut output = Vec::new();
    {
        let mut writer = ArrowWriter::try_new(&mut output, schema, Some(properties))?;
        writer.write(&batch)?;
        writer.close()?;
    }
    Ok(output)
}

fn decode_parquet(bytes: &[u8]) -> Result<Vec<StoredEvent>> {
    storage_segment::RecordCodec::decode(&SiftParquetCodec, bytes).map_err(Into::into)
}

fn decode_parquet_inner(bytes: &[u8]) -> Result<Vec<StoredEvent>> {
    let mut reader = ParquetRecordBatchReaderBuilder::try_new(Bytes::copy_from_slice(bytes))?
        .with_batch_size(1_024)
        .build()?;
    if reader.schema() != parquet_schema() {
        bail!("archive Parquet schema is not the Sift v2 segment schema");
    }
    let mut events = Vec::new();
    for batch in &mut reader {
        events.extend(decode_parquet_batch(&batch?)?);
    }
    Ok(events)
}

fn open_parquet_reader(path: &Path) -> Result<ParquetRecordBatchReader> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("open cached Parquet segment {}", path.display()))?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)?
        .with_batch_size(1_024)
        .build()?;
    if reader.schema() != parquet_schema() {
        bail!("archive Parquet schema is not the Sift v2 segment schema");
    }
    Ok(reader)
}

fn decode_parquet_batch(batch: &RecordBatch) -> Result<Vec<StoredEvent>> {
    if batch
        .columns()
        .iter()
        .any(|column| column.null_count() != 0)
    {
        bail!("archive Parquet segment contains a null required value");
    }
    let cursors = uint64_column(batch, 0, "cursor")?;
    let acknowledged = string_column(batch, 1, "acknowledged_at")?;
    let event_ids = string_column(batch, 2, "event_id")?;
    let projects = string_column(batch, 3, "project")?;
    let environments = string_column(batch, 4, "environment")?;
    let signals = string_column(batch, 5, "signal")?;
    let occurred = string_column(batch, 6, "occurred_at")?;
    let json = string_column(batch, 7, "event_json")?;
    let mut events = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let event: StoredEvent = serde_json::from_str(json.value(row))?;
        if event.cursor != cursors.value(row)
            || event.acknowledged_at != acknowledged.value(row)
            || event.event.event_id != event_ids.value(row)
            || event.event.project != projects.value(row)
            || event.event.environment != environments.value(row)
            || event.event.signal.to_string() != signals.value(row)
            || event.event.occurred_at != occurred.value(row)
        {
            bail!("archive Parquet columns disagree with event_json");
        }
        events.push(event);
    }
    Ok(events)
}

fn verify_archive_segment_file(segment: &ArchiveSegment, path: &Path) -> Result<()> {
    let mut reader = open_parquet_reader(path)?;
    let mut event_count = 0_u64;
    let mut first_cursor = None;
    let mut last_cursor = None;
    let mut minimum_acknowledged = None::<i64>;
    let mut maximum_acknowledged = None::<i64>;
    for batch in &mut reader {
        for event in decode_parquet_batch(&batch?)? {
            if last_cursor.is_some_and(|previous| event.cursor <= previous)
                || event.event.signal != segment.signal
            {
                bail!(
                    "archive segment {} does not match its committed metadata",
                    segment.source.segment_id
                );
            }
            let acknowledged = DateTime::parse_from_rfc3339(&event.acknowledged_at)
                .context("archive event acknowledged_at must be RFC3339")?
                .timestamp_nanos_opt()
                .context("archive acknowledgement time is outside the nanosecond range")?;
            first_cursor.get_or_insert(event.cursor);
            last_cursor = Some(event.cursor);
            minimum_acknowledged = Some(
                minimum_acknowledged
                    .map(|minimum| minimum.min(acknowledged))
                    .unwrap_or(acknowledged),
            );
            maximum_acknowledged = Some(
                maximum_acknowledged
                    .map(|maximum| maximum.max(acknowledged))
                    .unwrap_or(acknowledged),
            );
            event_count = event_count.saturating_add(1);
        }
    }
    if event_count != segment.source.event_count
        || first_cursor != Some(segment.source.first_cursor)
        || last_cursor != Some(segment.source.last_cursor)
        || (minimum_acknowledged, maximum_acknowledged)
            != (
                Some(segment.min_acknowledged_at_unix_nano),
                Some(segment.max_acknowledged_at_unix_nano),
            )
    {
        bail!(
            "archive segment {} does not match its committed metadata",
            segment.source.segment_id
        );
    }
    Ok(())
}

fn uint64_column<'a>(batch: &'a RecordBatch, index: usize, name: &str) -> Result<&'a UInt64Array> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .with_context(|| format!("archive Parquet column {name} has the wrong type"))
}

fn string_column<'a>(batch: &'a RecordBatch, index: usize, name: &str) -> Result<&'a StringArray> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .with_context(|| format!("archive Parquet column {name} has the wrong type"))
}

fn verify_archive_segment(segment: &ArchiveSegment, events: &[StoredEvent]) -> Result<()> {
    let acknowledgement_bounds = acknowledgement_time_bounds(events)?;
    if events.len() as u64 != segment.source.event_count
        || events.first().map(|event| event.cursor) != Some(segment.source.first_cursor)
        || events.last().map(|event| event.cursor) != Some(segment.source.last_cursor)
        || events
            .windows(2)
            .any(|pair| pair[0].cursor >= pair[1].cursor)
        || events
            .iter()
            .any(|event| event.event.signal != segment.signal)
        || acknowledgement_bounds
            != (
                segment.min_acknowledged_at_unix_nano,
                segment.max_acknowledged_at_unix_nano,
            )
    {
        bail!(
            "archive segment {} does not match its committed metadata",
            segment.source.segment_id
        );
    }
    Ok(())
}

fn verify_bytes(expected_hash: &str, expected_size: u64, bytes: &[u8], kind: &str) -> Result<()> {
    if bytes.len() as u64 != expected_size || sha256(bytes) != expected_hash {
        bail!("{kind} archive object failed hash/size verification");
    }
    Ok(())
}

fn verify_file(path: &Path, expected_hash: &str, expected_size: u64, kind: &str) -> Result<()> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("open {kind} {}", path.display()))?;
    if file.metadata()?.len() != expected_size {
        bail!("{kind} archive object failed hash/size verification");
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    if hex::encode(digest.finalize()) != expected_hash {
        bail!("{kind} archive object failed hash/size verification");
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[cfg(unix)]
fn set_private_dir(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_dir(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_file(_path: &Path) -> Result<()> {
    Ok(())
}
// HANDWRITE-END
