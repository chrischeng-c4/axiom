// HANDWRITE-BEGIN gap="sift-gcs-archive-manifest" tracker="1659" reason="Upload immutable Parquet segments and blobs before the commit manifest, then restore only hash-verified objects."
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use arrow_array::{Array, ArrayRef, RecordBatch, RecordBatchReader, StringArray, UInt64Array};
use arrow_schema::{DataType, Field, Schema};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use parquet::{
    arrow::{arrow_reader::ParquetRecordBatchReaderBuilder, ArrowWriter},
    basic::Compression,
    file::properties::WriterProperties,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ContentBlobRef, SignalKind, StoredEvent};

use super::{
    blob::BlobStore, shard, DataLayout, EpochMap, RawStorage, SegmentManifest, StorageRole,
};

const ARCHIVE_FORMAT_VERSION: u16 = 6;
const ARCHIVE_COMMIT_FORMAT_VERSION: u16 = 2;
const ARCHIVE_COMMIT_PATH: &str = "control/archive-commit.json";
const LOCAL_COMMIT_FORMAT_VERSION: u16 = 2;
const LOCAL_COMMIT_PATH: &str = "control/local-segment-commit.json";
const ARCHIVE_GC_FORMAT_VERSION: u16 = 2;
const ARCHIVE_GC_PATH: &str = "control/archive-gc-pending.json";
const ARCHIVE_CACHE_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

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
    /// Monotonic epoch changed only when the retained event set loses rows.
    /// Replicas use it to avoid downloading the full cumulative archive for
    /// an ordinary checkpoint that only advances the durable prefix.
    pub retention_generation: u64,
    /// Highest cursor made durable in an archive commit. WAL compaction uses
    /// this monotonic coverage even after an event expires.
    pub watermarks: ArchiveWatermarks,
    /// Highest cursor that is still present in each retained signal set.
    pub retained_watermarks: ArchiveWatermarks,
    pub epochs: Vec<EpochMap>,
    pub segments: Vec<ArchiveSegment>,
    pub blobs: Vec<ArchiveBlob>,
    /// Complete, immutable cleanup plan inherited by every voter that installs
    /// this manifest-backed Raft checkpoint. Only a post-checkpoint leader may
    /// execute it.
    #[serde(default)]
    pub gc_object_uris: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ArchiveReceipt {
    pub manifest_uri: String,
    pub manifest_sha256: String,
    pub manifest: ArchiveManifest,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArchiveGcPending {
    format_version: u16,
    replacement_manifest_uri: String,
    replacement_manifest_sha256: String,
    object_uris: Vec<String>,
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

#[derive(Clone, Debug)]
pub struct ArchiveCommitStatus {
    pub manifest_uri: String,
    pub manifest_sha256: String,
    pub committed_at: String,
    pub snapshot_index: u64,
    pub watermarks: ArchiveWatermarks,
    pub retention_generation: u64,
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
    if !receipt.manifest.gc_object_uris.is_empty() {
        write_archive_gc_pending(
            journal.storage().root(),
            &receipt.manifest_uri,
            &receipt.manifest_sha256,
            receipt.manifest.gc_object_uris.clone(),
        )?;
    }
    let watermarks = record_archive_commit(journal.storage().root(), &receipt)?;
    retire_local_commit(journal.storage().root())?;
    journal.compact_archived_wal(watermarks)?;
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
    let manifest = fetch_verified_committed_manifest(journal.storage().root())?
        .context("cold segment eviction requires a committed remote manifest")?;
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
        let exact_remote = manifest
            .segments
            .iter()
            .any(|segment| segment.signal == signal && segment.source == portable);
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
    let current_state = read_commit_state(journal.storage().root())?
        .context("180-day expiration requires a committed remote manifest")?;
    let current = fetch_verified_committed_manifest(journal.storage().root())?
        .context("180-day expiration requires a readable remote manifest")?;
    if journal.last_cursor() < current.raft_snapshot_index {
        bail!(
            "180-day expiration journal cursor {} is behind archive cursor {}",
            journal.last_cursor(),
            current.raft_snapshot_index
        );
    }
    if journal.total_event_count() < current.event_count {
        bail!("journal contains fewer events than its committed archive prefix");
    }
    let suffix_events = journal.last_cursor() - current.raft_snapshot_index;
    let expected_local_events = current
        .event_count
        .checked_add(suffix_events)
        .context("retained archive and suffix event count exhausted u64")?;
    let cutoff = now - chrono::Duration::days(180);
    let cutoff_nanos = cutoff
        .timestamp_nanos_opt()
        .context("180-day retention cutoff is outside the nanosecond range")?;
    if current
        .segments
        .iter()
        .all(|segment| segment.source.min_event_time_unix_nano >= cutoff_nanos)
    {
        let retained_blob_hashes = current
            .blobs
            .iter()
            .map(|blob| blob.reference.hash.clone())
            .collect::<BTreeSet<_>>();
        journal.prune_unreferenced_blobs(&retained_blob_hashes)?;
        if journal.total_event_count() != expected_local_events
            || journal.recovery_required()
            || journal.retention_generation() != current.retention_generation
        {
            let repair_dedupe =
                journal.recovery_required() || journal.total_event_count() != expected_local_events;
            let current_digest = decode_event_content_digest(&current.event_content_sha256)?;
            journal.apply_expiration_head(
                cutoff,
                current.event_count,
                current.event_count,
                current_digest,
                current_digest,
                repair_dedupe,
            )?;
        }
        return Ok(ExpirationReceipt {
            manifest_uri: current_state.manifest_uri,
            retained_events: current.event_count,
            retained_segments: current.segments.len(),
            expired_events: 0,
            replaced_segments: 0,
            removed_segments: 0,
        });
    }
    let (archive_bucket, current_manifest_key) = split_gcs_uri(&current_state.manifest_uri)?;
    let archive_coordinator = storage_segment::ArchiveCoordinator::new(Arc::new(
        storage_object::GcsObjectStore::new(&archive_bucket, "")?,
    ));
    let mut archive_transaction = archive_coordinator.begin();
    let parent = current_manifest_key
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("sift");
    let generation = now.format("%Y%m%dT%H%M%S%.fZ");
    let rewrite_prefix = format!("{parent}/retention-{generation}");

    let mut retained_segments = Vec::new();
    let mut retained_event_count = current.event_count;
    let mut retained_event_digest = decode_event_id_digest(&current.event_id_sha256)?;
    let mut retained_event_content_digest =
        decode_event_content_digest(&current.event_content_sha256)?;
    let mut retained_blob_counts =
        archive_blob_reference_counts(journal.storage().root(), &current)?;
    let mut expired_events = 0_u64;
    let mut replaced_segments = 0_usize;
    let mut removed_segments = 0_usize;
    let mut obsolete_objects = archive_gc_pending_objects(journal.storage().root())?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut obsolete_cache_hashes = BTreeSet::new();
    let mut obsolete_local_segments = Vec::new();
    let mut expired_source_segments = Vec::new();

    for segment in &current.segments {
        if segment.source.min_event_time_unix_nano >= cutoff_nanos {
            retained_segments.push(segment.clone());
            continue;
        }
        let bytes = cached_segment_bytes(journal.storage().root(), segment)?;
        let events = decode_parquet(&bytes)?;
        verify_archive_segment(segment, &events)?;
        let mut retained = Vec::with_capacity(events.len());
        for event in events {
            let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
                .context("archive event occurred_at must be RFC3339")?
                .with_timezone(&Utc);
            if occurred < cutoff {
                expired_events = expired_events.saturating_add(1);
                retained_event_count = retained_event_count
                    .checked_sub(1)
                    .context("expired archive event count underflow")?;
                include_event_id(&mut retained_event_digest, &event.event.event_id);
                include_event_content(&mut retained_event_content_digest, &event)?;
                for reference in &event.event.blob_refs {
                    let count =
                        retained_blob_counts
                            .get_mut(&reference.hash)
                            .with_context(|| {
                                format!(
                            "expired event references blob {} that is absent from its manifest",
                            reference.hash
                        )
                            })?;
                    *count = count
                        .checked_sub(1)
                        .context("archive blob reference count underflow")?;
                }
            } else {
                retained.push(event);
            }
        }

        if retained.len() as u64 == segment.source.event_count {
            retained_segments.push(segment.clone());
            continue;
        }

        obsolete_objects.insert(segment.object_uri.clone());
        obsolete_cache_hashes.insert(segment.parquet_sha256.clone());
        obsolete_local_segments.push((segment.signal, segment.source.segment_id.clone()));
        expired_source_segments.push(segment.clone());
        if retained.is_empty() {
            removed_segments += 1;
            continue;
        }

        replaced_segments += 1;
        let parquet = encode_parquet(&retained)?;
        let parquet_sha256 = sha256(&parquet);
        let local_replacement = journal.storage().write_retained_segment(
            segment.signal,
            &segment.source.segment_id,
            &retained,
        )?;
        let mut source = if let Some(local_replacement) = local_replacement {
            portable_manifest(local_replacement, segment.signal)
        } else {
            let mut source = segment.source.clone();
            source.segment_id = format!("retained-{}", &parquet_sha256[..32]);
            source.first_cursor = retained
                .first()
                .expect("retained segment is non-empty")
                .cursor;
            source.last_cursor = retained
                .last()
                .expect("retained segment is non-empty")
                .cursor;
            source.event_count = retained.len() as u64;
            source.bytes = parquet.len() as u64;
            source.sha256 = parquet_sha256.clone();
            source.local_path = PathBuf::from(format!(
                "segments/{}/{}.framed",
                segment.signal, source.segment_id
            ));
            source.object_uri = None;
            source
        };
        let segment_id = source.segment_id.clone();
        let key = format!(
            "{rewrite_prefix}/segments/{}/{}.parquet",
            segment.signal, segment_id
        );
        archive_transaction.put(storage_segment::ArchiveObject::new(
            key.clone(),
            parquet.clone(),
            "application/vnd.apache.parquet",
        ))?;
        let object_uri = gcs_uri(&archive_bucket, &key);
        let mut event_times = retained.iter().map(event_time_unix_nano);
        let first_event_time = event_times
            .next()
            .transpose()?
            .expect("retained segment is non-empty");
        let (min_event_time, max_event_time) = event_times.try_fold(
            (first_event_time, first_event_time),
            |(minimum, maximum), event_time| {
                let event_time = event_time?;
                anyhow::Ok((minimum.min(event_time), maximum.max(event_time)))
            },
        )?;
        source.min_event_time_unix_nano = min_event_time;
        source.max_event_time_unix_nano = max_event_time;
        retained_segments.push(ArchiveSegment {
            signal: segment.signal,
            source,
            object_uri,
            parquet_bytes: parquet.len() as u64,
            parquet_sha256,
        });
    }

    if expired_events == 0 {
        return Ok(ExpirationReceipt {
            manifest_uri: current_state.manifest_uri,
            retained_events: current.event_count,
            retained_segments: current.segments.len(),
            expired_events: 0,
            replaced_segments: 0,
            removed_segments: 0,
        });
    }

    retained_segments.sort_by_key(|segment| segment.source.first_cursor);
    let mut retained_watermarks = ArchiveWatermarks::default();
    for segment in &retained_segments {
        retained_watermarks.include(segment.signal, segment.source.last_cursor);
    }
    let mut retained_blob_hashes = BTreeSet::new();
    let mut retained_blobs = Vec::new();
    for mut blob in current.blobs.clone() {
        let count = retained_blob_counts
            .remove(&blob.reference.hash)
            .unwrap_or_default();
        if count > 0 {
            blob.reference_count = count;
            retained_blob_hashes.insert(blob.reference.hash.clone());
            retained_blobs.push(blob);
        } else {
            obsolete_objects.insert(blob.object_uri.clone());
        }
    }
    if retained_blob_counts.values().any(|count| *count != 0) {
        bail!("retained archive blob counts disagree with manifest objects");
    }
    obsolete_objects.insert(current_state.manifest_uri.clone());
    let gc_object_uris = obsolete_objects.iter().cloned().collect::<Vec<_>>();

    let manifest = ArchiveManifest {
        format_version: ARCHIVE_FORMAT_VERSION,
        generated_at: now.to_rfc3339(),
        source_cluster_id: current.source_cluster_id,
        source_node_id: current.source_node_id,
        raft_snapshot_index: current.raft_snapshot_index,
        event_count: retained_event_count,
        event_id_digest_algorithm: "xor-sha256-v1".to_string(),
        event_id_sha256: hex::encode(retained_event_digest),
        event_content_digest_algorithm: "xor-sha256-v1".to_string(),
        event_content_sha256: hex::encode(retained_event_content_digest),
        retention_generation: current.retention_generation.saturating_add(1),
        watermarks: current.watermarks,
        retained_watermarks,
        epochs: current.epochs,
        segments: retained_segments,
        blobs: retained_blobs,
        gc_object_uris,
    };
    validate_archive_manifest(&manifest)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let manifest_key = format!("{rewrite_prefix}/manifest.json");
    let commit = archive_transaction.commit(storage_segment::ArchiveObject::new(
        manifest_key.clone(),
        manifest_bytes,
        "application/json",
    ))?;
    let manifest_sha256 = commit.manifest.sha256;
    let manifest_uri = gcs_uri(&archive_bucket, &manifest_key);
    let receipt = ArchiveReceipt {
        manifest_uri: manifest_uri.clone(),
        manifest_sha256,
        manifest,
    };
    // The prior manifest and objects must remain readable until every Raft
    // voter has durably installed the replacement archive checkpoint. Persist
    // the deletion intent before publishing the replacement receipt. A crash
    // between these writes leaves an intent that cannot pass the identity
    // check in the finalizer, so it cannot orphan the old recovery source.
    write_archive_gc_pending(
        journal.storage().root(),
        &receipt.manifest_uri,
        &receipt.manifest_sha256,
        receipt.manifest.gc_object_uris.clone(),
    )?;
    journal.mark_recovery_required();
    record_archive_commit(journal.storage().root(), &receipt)?;

    // A rewritten source can contain both expired and hot rows. Generic hot
    // eviction cannot remove that mixed file because its maximum event time is
    // still hot. The replacement segment is already durable locally and in the
    // committed manifest, so remove only the exact source IDs that this
    // retention transaction replaced. If this step fails, the newer manifest
    // generation remains ahead of the journal head and startup reconciliation
    // retries the retained-prefix rewrite before serving data.
    for (signal, segment_id) in obsolete_local_segments {
        journal.storage().evict_segment(signal, &segment_id)?;
    }
    let mut removed_dedupe_entries = 0_u64;
    for segment in expired_source_segments {
        let bytes = cached_segment_bytes(journal.storage().root(), &segment)?;
        let events = decode_parquet(&bytes)?;
        verify_archive_segment(&segment, &events)?;
        let mut expired_ids = Vec::new();
        for event in &events {
            let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
                .context("archive event occurred_at must be RFC3339")?
                .with_timezone(&Utc);
            if occurred < cutoff {
                expired_ids.push(event.event.event_id.as_str());
            }
        }
        removed_dedupe_entries = removed_dedupe_entries
            .saturating_add(journal.remove_expired_event_ids(expired_ids.iter().copied())?);
    }
    if removed_dedupe_entries != expired_events {
        bail!("removed {removed_dedupe_entries} dedupe rows for {expired_events} expired events");
    }
    evict_committed_cold_segments_at(journal, now)?;
    journal.apply_expiration_head(
        cutoff,
        current.event_count,
        retained_event_count,
        decode_event_content_digest(&current.event_content_sha256)?,
        retained_event_content_digest,
        false,
    )?;
    journal.prune_unreferenced_blobs(&retained_blob_hashes)?;
    for hash in obsolete_cache_hashes {
        let cache = journal
            .storage()
            .root()
            .join("archive-cache")
            .join(format!("{hash}.parquet"));
        match std::fs::remove_file(&cache) {
            Ok(()) => storage_durable::sync_parent_dir(&cache)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(ExpirationReceipt {
        manifest_uri,
        retained_events: retained_event_count,
        retained_segments: receipt.manifest.segments.len(),
        expired_events,
        replaced_segments,
        removed_segments,
    })
}

pub(crate) fn retention_due_at(root: &Path, now: DateTime<Utc>) -> Result<bool> {
    let Some(state) = read_commit_state(root)? else {
        return Ok(false);
    };
    let cutoff_nanos = (now - chrono::Duration::days(180))
        .timestamp_nanos_opt()
        .context("180-day retention cutoff is outside the nanosecond range")?;
    Ok(state
        .manifest
        .segments
        .iter()
        .any(|segment| segment.source.min_event_time_unix_nano < cutoff_nanos))
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
    let destination = service_backup::BackupDestination::from_uri(destination_uri)?;
    let (bucket, prefix) = gcs_destination(&destination)?;
    let coordinator = storage_segment::ArchiveCoordinator::new(Arc::new(
        storage_object::GcsObjectStore::new(&bucket, "")?,
    ));
    let mut archive_transaction = coordinator.begin();
    let archive_id = format!(
        "{}-{}",
        Utc::now().format("%Y%m%dT%H%M%S%.fZ"),
        std::process::id()
    );
    let archive_prefix = format!("{prefix}/archives/{archive_id}");

    let layout: super::LayoutManifest = serde_json::from_slice(
        &std::fs::read(storage.root().join("layout.json"))
            .context("read Sift layout for archive identity")?,
    )
    .context("decode Sift layout for archive identity")?;
    let previous_state = read_commit_state(storage.root())?;
    let previous_manifest_uri = previous_state
        .as_ref()
        .map(|state| state.manifest_uri.clone());
    let previous = previous_state.map(|state| state.manifest);
    if previous
        .as_ref()
        .is_some_and(|manifest| manifest.source_cluster_id != layout.cluster_id)
    {
        bail!("committed archive belongs to a different Sift cluster");
    }
    let mut segments = previous
        .as_ref()
        .map(|manifest| {
            manifest
                .segments
                .iter()
                .cloned()
                .map(|segment| (segment.source.segment_id.clone(), segment))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut blobs = previous
        .as_ref()
        .map(|manifest| {
            manifest
                .blobs
                .iter()
                .cloned()
                .map(|blob| (blob.reference.hash.clone(), blob))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut referenced = BTreeMap::<String, (ContentBlobRef, u64)>::new();
    if let Some(previous) = &previous {
        let previous_counts = archive_blob_reference_counts(storage.root(), previous)?;
        for (hash, count) in previous_counts {
            if let Some(blob) = blobs.get_mut(&hash) {
                blob.reference_count = count;
            }
        }
    }
    let mut event_count = previous
        .as_ref()
        .map(|manifest| manifest.event_count)
        .unwrap_or(0);
    let mut event_id_digest = previous
        .as_ref()
        .map(|manifest| decode_event_id_digest(&manifest.event_id_sha256))
        .transpose()?
        .unwrap_or([0u8; 32]);
    let mut event_content_digest = previous
        .as_ref()
        .map(|manifest| decode_event_content_digest(&manifest.event_content_sha256))
        .transpose()?
        .unwrap_or([0u8; 32]);
    let mut watermarks = previous
        .as_ref()
        .map(|manifest| manifest.watermarks)
        .unwrap_or_default();
    let mut retained_watermarks = previous
        .as_ref()
        .map(|manifest| manifest.retained_watermarks)
        .unwrap_or_default();
    let retention_generation = previous
        .as_ref()
        .map(|manifest| manifest.retention_generation)
        .unwrap_or_default();
    // The local pending file records unfinished work. A previous manifest can
    // keep its historical plan after another leader completed it, so do not
    // resurrect that completed plan in a later archive.
    let mut gc_object_uris = archive_gc_pending_objects(storage.root())?
        .into_iter()
        .collect::<BTreeSet<_>>();
    if let Some(previous_manifest_uri) = previous_manifest_uri {
        gc_object_uris.insert(previous_manifest_uri);
    }
    let previously_covered = previous
        .as_ref()
        .map(|manifest| manifest.raft_snapshot_index)
        .unwrap_or_default();
    if captured_cursor < previously_covered {
        bail!(
            "captured archive cursor {captured_cursor} is behind committed prefix {previously_covered}"
        );
    }
    let mut newly_covered_cursors = BTreeSet::new();
    for (signal, source) in captured_segments {
        let portable_source = portable_manifest(source.clone(), signal);
        if let Some(committed) = segments.get(&source.segment_id) {
            if committed.signal != signal || committed.source != portable_source {
                bail!(
                    "immutable segment {} changed after archive commit",
                    source.segment_id
                );
            }
            continue;
        }
        let covered_through = previous
            .as_ref()
            .map(|manifest| manifest.watermarks.through(signal))
            .unwrap_or_default();
        if source.last_cursor <= covered_through {
            // Retention can rewrite a remote Parquet segment while an older
            // local hot copy still exists. Its cursors are already covered by
            // the committed manifest and must not be counted a second time.
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
            if event.cursor > previously_covered && !newly_covered_cursors.insert(event.cursor) {
                bail!("archive prefix contains duplicate cursor {}", event.cursor);
            }
            event_count = event_count.saturating_add(1);
            include_event_id(&mut event_id_digest, &event.event.event_id);
            include_event_content(&mut event_content_digest, event)?;
            watermarks.include(signal, event.cursor);
            retained_watermarks.include(signal, event.cursor);
            for reference in &event.event.blob_refs {
                referenced
                    .entry(reference.hash.clone())
                    .and_modify(|(_, count)| *count = count.saturating_add(1))
                    .or_insert_with(|| (reference.clone(), 1));
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
        let object_uri = gcs_uri(&bucket, &key);
        segments.insert(
            portable_source.segment_id.clone(),
            ArchiveSegment {
                signal,
                source: portable_source,
                object_uri,
                parquet_bytes: parquet.len() as u64,
                parquet_sha256,
            },
        );
    }
    let mut expected_cursor = previously_covered.saturating_add(1);
    for cursor in newly_covered_cursors {
        if cursor != expected_cursor {
            bail!(
                "archive prefix is not contiguous: expected cursor {expected_cursor}, found {cursor}"
            );
        }
        expected_cursor = expected_cursor
            .checked_add(1)
            .context("archive cursor exhausted u64")?;
    }
    if expected_cursor.saturating_sub(1) != captured_cursor {
        bail!(
            "archive prefix ended at cursor {}, before captured cursor {captured_cursor}",
            expected_cursor.saturating_sub(1)
        );
    }
    let mut segments = segments.into_values().collect::<Vec<_>>();
    segments.sort_by_key(|segment| segment.source.first_cursor);

    for (reference, reference_count) in referenced.into_values() {
        if let Some(blob) = blobs.get_mut(&reference.hash) {
            blob.reference_count = blob.reference_count.saturating_add(reference_count);
            continue;
        }
        let bytes = storage.read_blob(&reference.hash)?;
        if bytes.len() as u64 != reference.size {
            bail!("blob {} size changed before archive", reference.hash);
        }
        let digest = reference.hash.trim_start_matches("sha256:");
        let key = format!("{archive_prefix}/blobs/{digest}.blob");
        archive_transaction.put(storage_segment::ArchiveObject::new(
            key.clone(),
            bytes,
            "application/octet-stream",
        ))?;
        let object_uri = gcs_uri(&bucket, &key);
        blobs.insert(
            reference.hash.clone(),
            ArchiveBlob {
                reference,
                object_uri,
                reference_count,
            },
        );
    }
    let mut blobs = blobs.into_values().collect::<Vec<_>>();
    blobs.sort_by(|left, right| left.reference.hash.cmp(&right.reference.hash));

    let manifest = ArchiveManifest {
        format_version: ARCHIVE_FORMAT_VERSION,
        generated_at: Utc::now().to_rfc3339(),
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
        epochs: storage.epoch_maps(),
        segments,
        blobs,
        gc_object_uris: gc_object_uris.into_iter().collect(),
    };
    validate_archive_manifest(&manifest)?;
    let manifest_key = format!("{archive_prefix}/manifest.json");
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let commit = archive_transaction.commit(storage_segment::ArchiveObject::new(
        manifest_key.clone(),
        manifest_bytes,
        "application/json",
    ))?;
    let manifest_sha256 = commit.manifest.sha256;
    let manifest_uri = gcs_uri(&bucket, &manifest_key);
    Ok(ArchiveReceipt {
        manifest_uri,
        manifest_sha256,
        manifest,
    })
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

/// Verify that the last locally committed archive manifest is still readable.
/// A cold query uses this check before it claims a complete answer. The local
/// commit receipt is not enough because GCS access can be removed after the
/// commit was written.
pub fn verify_committed_manifest_available(root: &Path) -> Result<bool> {
    Ok(fetch_verified_committed_manifest(root)?.is_some())
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
    let Some(manifest) = fetch_verified_committed_manifest(root)? else {
        return Ok(None);
    };
    let start = parse_optional_archive_time("start", start)?;
    let end = parse_optional_archive_time("end", end)?;
    if start.zip(end).is_some_and(|(start, end)| start >= end) {
        bail!("archive query start must be earlier than end");
    }

    let mut scanned = 0_u64;
    let mut replayed = 0_u64;
    for segment in manifest
        .segments
        .iter()
        .filter(|segment| segment.signal == signal)
    {
        let bytes = cached_segment_bytes(root, segment)?;
        let events = decode_parquet(&bytes)?;
        verify_archive_segment(segment, &events)?;
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

/// Replay every event in the latest remote manifest. Index rebuild uses this
/// path after local hot segments have been evicted.
pub(crate) fn replay_all_committed_events<F>(
    root: &Path,
    mut visitor: F,
) -> Result<Option<ArchiveReplay>>
where
    F: FnMut(StoredEvent) -> Result<()>,
{
    let Some(manifest) = fetch_verified_committed_manifest(root)? else {
        return Ok(None);
    };
    let mut segments = manifest.segments.iter().collect::<Vec<_>>();
    segments.sort_by_key(|segment| segment.source.first_cursor);
    let mut scanned = 0_u64;
    let mut last_cursor = 0_u64;
    let mut event_id_digest = [0_u8; 32];
    let mut event_content_digest = [0_u8; 32];
    let mut retained_watermarks = ArchiveWatermarks::default();
    for segment in segments {
        let bytes = cached_segment_bytes(root, segment)?;
        let events = decode_parquet(&bytes)?;
        verify_archive_segment(segment, &events)?;
        for event in events {
            if event.cursor <= last_cursor {
                bail!("archive event cursors are not globally strictly increasing");
            }
            last_cursor = event.cursor;
            include_event_id(&mut event_id_digest, &event.event.event_id);
            include_event_content(&mut event_content_digest, &event)?;
            retained_watermarks.include(event.event.signal, event.cursor);
            visitor(event)?;
            scanned = scanned.saturating_add(1);
        }
    }
    if scanned != manifest.event_count
        || hex::encode(event_id_digest) != manifest.event_id_sha256
        || hex::encode(event_content_digest) != manifest.event_content_sha256
        || retained_watermarks != manifest.retained_watermarks
        || last_cursor != manifest.retained_watermarks.max_cursor()
    {
        bail!("committed archive event set disagrees with its manifest");
    }
    Ok(Some(ArchiveReplay {
        watermark: manifest.raft_snapshot_index,
        scanned,
        replayed: scanned,
    }))
}

fn fetch_verified_committed_manifest(root: &Path) -> Result<Option<ArchiveManifest>> {
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
    Ok(Some(manifest))
}

fn cached_segment_bytes(root: &Path, segment: &ArchiveSegment) -> Result<Vec<u8>> {
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
        let cached = std::fs::read(&cache_path)
            .with_context(|| format!("read archive cache {}", cache_path.display()))?;
        if verify_bytes(
            &segment.parquet_sha256,
            segment.parquet_bytes,
            &cached,
            "cached Parquet segment",
        )
        .is_ok()
        {
            set_private_file(&cache_path)?;
            prune_archive_cache(root, &cache_path)?;
            return Ok(cached);
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
    Ok(bytes)
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

fn validate_archive_manifest(manifest: &ArchiveManifest) -> Result<()> {
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
    {
        bail!("Sift archive manifest has invalid identity or digest fields");
    }
    if manifest.watermarks.max_cursor() > manifest.raft_snapshot_index
        || manifest.event_count > manifest.raft_snapshot_index
        || manifest.retained_watermarks.logs > manifest.watermarks.logs
        || manifest.retained_watermarks.metrics > manifest.watermarks.metrics
        || manifest.retained_watermarks.traces > manifest.watermarks.traces
    {
        bail!("Sift archive manifest has invalid coverage or retained watermarks");
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
            || !segment_ids.insert(segment.source.segment_id.clone())
        {
            bail!("Sift archive manifest has an invalid segment object");
        }
        event_count = event_count
            .checked_add(segment.source.event_count)
            .context("Sift archive manifest event count exhausted u64")?;
        retained_watermarks.include(segment.signal, segment.source.last_cursor);
    }
    if event_count != manifest.event_count || retained_watermarks != manifest.retained_watermarks {
        bail!("Sift archive manifest segment totals do not match retained event metadata");
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
    let mut prior_gc_uri: Option<&str> = None;
    for uri in &manifest.gc_object_uris {
        if !uri.starts_with("gs://") || prior_gc_uri.is_some_and(|prior| prior >= uri.as_str()) {
            bail!("Sift archive manifest has an invalid or unsorted GC object URI");
        }
        service_backup::GcsSink::from_exact_uri(uri)
            .with_context(|| format!("validate archive manifest GC object URI {uri}"))?;
        prior_gc_uri = Some(uri);
    }
    Ok(())
}

fn event_time_unix_nano(event: &StoredEvent) -> Result<i64> {
    DateTime::parse_from_rfc3339(&event.event.occurred_at)
        .context("archive event occurred_at must be RFC3339")?
        .timestamp_nanos_opt()
        .context("archive event occurred_at is outside the nanosecond range")
}

fn archive_blob_reference_counts(
    root: &Path,
    manifest: &ArchiveManifest,
) -> Result<BTreeMap<String, u64>> {
    if manifest.blobs.iter().all(|blob| blob.reference_count > 0) {
        return Ok(manifest
            .blobs
            .iter()
            .map(|blob| (blob.reference.hash.clone(), blob.reference_count))
            .collect());
    }

    // Additive rolling-upgrade fallback for manifests written before reference
    // counts existed. A newly written manifest always persists the counts, so
    // this full scan happens at most once per old archive lineage.
    let known = manifest
        .blobs
        .iter()
        .map(|blob| blob.reference.hash.as_str())
        .collect::<BTreeSet<_>>();
    let mut counts = BTreeMap::<String, u64>::new();
    for segment in &manifest.segments {
        let bytes = cached_segment_bytes(root, segment)?;
        let events = decode_parquet(&bytes)?;
        verify_archive_segment(segment, &events)?;
        for event in events {
            for reference in event.event.blob_refs {
                if !known.contains(reference.hash.as_str()) {
                    bail!(
                        "archive event references blob {} that is absent from its manifest",
                        reference.hash
                    );
                }
                *counts.entry(reference.hash).or_default() += 1;
            }
        }
    }
    if counts.len() != manifest.blobs.len() {
        bail!("archive manifest contains an unreferenced blob object");
    }
    Ok(counts)
}

fn write_archive_gc_pending(
    root: &Path,
    replacement_manifest_uri: &str,
    replacement_manifest_sha256: &str,
    object_uris: Vec<String>,
) -> Result<()> {
    if !replacement_manifest_uri.starts_with("gs://")
        || replacement_manifest_sha256.len() != 64
        || !replacement_manifest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("archive GC replacement manifest identity is invalid");
    }
    let path = root.join(ARCHIVE_GC_PATH);
    let mut uris = if path.exists() {
        let pending: ArchiveGcPending = serde_json::from_slice(
            &std::fs::read(&path)
                .with_context(|| format!("read archive GC receipt {}", path.display()))?,
        )
        .with_context(|| format!("decode archive GC receipt {}", path.display()))?;
        if pending.format_version != ARCHIVE_GC_FORMAT_VERSION {
            bail!(
                "unsupported archive GC format {}; expected {}",
                pending.format_version,
                ARCHIVE_GC_FORMAT_VERSION
            );
        }
        pending.object_uris
    } else {
        Vec::new()
    };
    uris.extend(object_uris);
    uris.sort();
    uris.dedup();
    for uri in &uris {
        service_backup::GcsSink::from_exact_uri(uri)
            .with_context(|| format!("validate archive GC object URI {uri}"))?;
    }
    persist_archive_gc_pending(
        &path,
        replacement_manifest_uri,
        replacement_manifest_sha256,
        uris,
    )
}

fn archive_gc_pending_objects(root: &Path) -> Result<Vec<String>> {
    let path = root.join(ARCHIVE_GC_PATH);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let pending: ArchiveGcPending = serde_json::from_slice(
        &std::fs::read(&path)
            .with_context(|| format!("read archive GC receipt {}", path.display()))?,
    )
    .with_context(|| format!("decode archive GC receipt {}", path.display()))?;
    if pending.format_version != ARCHIVE_GC_FORMAT_VERSION {
        bail!(
            "unsupported archive GC format {}; expected {}",
            pending.format_version,
            ARCHIVE_GC_FORMAT_VERSION
        );
    }
    Ok(pending.object_uris)
}

pub(crate) fn install_archive_gc_plan(root: &Path, receipt: &ArchiveReceipt) -> Result<()> {
    if receipt.manifest.gc_object_uris.is_empty() {
        let path = root.join(ARCHIVE_GC_PATH);
        match std::fs::remove_file(&path) {
            Ok(()) => storage_durable::sync_parent_dir(&path)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        return Ok(());
    }
    persist_archive_gc_pending(
        &root.join(ARCHIVE_GC_PATH),
        &receipt.manifest_uri,
        &receipt.manifest_sha256,
        receipt.manifest.gc_object_uris.clone(),
    )
}

pub(crate) fn archive_gc_pending(root: &Path) -> bool {
    root.join(ARCHIVE_GC_PATH).exists()
}

/// Delete obsolete remote archive objects after the replacement checkpoint is
/// durable on every voter. A single-node caller may use its local checkpoint
/// as the same barrier.
#[doc(hidden)]
pub fn finalize_archive_gc_after_checkpoint(root: &Path) -> Result<usize> {
    let path = root.join(ARCHIVE_GC_PATH);
    if !path.exists() {
        return Ok(0);
    }
    let mut pending: ArchiveGcPending = serde_json::from_slice(
        &std::fs::read(&path)
            .with_context(|| format!("read archive GC receipt {}", path.display()))?,
    )
    .with_context(|| format!("decode archive GC receipt {}", path.display()))?;
    if pending.format_version != ARCHIVE_GC_FORMAT_VERSION {
        bail!(
            "unsupported archive GC format {}; expected {}",
            pending.format_version,
            ARCHIVE_GC_FORMAT_VERSION
        );
    }
    let committed =
        committed_status(root)?.context("archive GC requires the replacement manifest receipt")?;
    if committed.manifest_uri != pending.replacement_manifest_uri
        || committed.manifest_sha256 != pending.replacement_manifest_sha256
    {
        bail!("archive GC replacement manifest is not the committed archive identity");
    }
    let mut deleted = 0_usize;
    while let Some(uri) = pending.object_uris.first().cloned() {
        let (sink, key) = service_backup::GcsSink::from_exact_uri(&uri)
            .with_context(|| format!("validate archive GC object URI {uri}"))?;
        sink.delete_object(&key)
            .with_context(|| format!("delete obsolete archive object {uri}"))?;
        pending.object_uris.remove(0);
        deleted += 1;
        persist_archive_gc_pending(
            &path,
            &pending.replacement_manifest_uri,
            &pending.replacement_manifest_sha256,
            pending.object_uris.clone(),
        )?;
    }
    Ok(deleted)
}

fn persist_archive_gc_pending(
    path: &Path,
    replacement_manifest_uri: &str,
    replacement_manifest_sha256: &str,
    object_uris: Vec<String>,
) -> Result<()> {
    if object_uris.is_empty() {
        match std::fs::remove_file(path) {
            Ok(()) => storage_durable::sync_parent_dir(path)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        return Ok(());
    }
    let pending = ArchiveGcPending {
        format_version: ARCHIVE_GC_FORMAT_VERSION,
        replacement_manifest_uri: replacement_manifest_uri.to_string(),
        replacement_manifest_sha256: replacement_manifest_sha256.to_string(),
        object_uris,
    };
    storage_durable::atomic_write(
        path,
        &serde_json::to_vec_pretty(&pending)?,
        storage_durable::FsyncPolicy::Always,
    )?;
    set_private_file(path)
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
    storage_durable::atomic_write(
        &path,
        &serde_json::to_vec_pretty(&state)?,
        storage_durable::FsyncPolicy::Always,
    )?;
    set_private_file(&path)?;
    Ok(watermarks)
}

pub(crate) fn adopt_verified_archive_receipt(
    root: &Path,
    receipt: &ArchiveReceipt,
) -> Result<ArchiveWatermarks> {
    validate_archive_manifest(&receipt.manifest)?;
    record_archive_commit(root, receipt)
}

pub fn restore_gcs(manifest_uri: &str, target: impl AsRef<Path>) -> Result<ArchiveManifest> {
    let target = target.as_ref();
    require_empty_restore_target(target)?;
    let manifest_bytes = service_backup::fetch_backup_object(manifest_uri)?;
    let manifest: ArchiveManifest =
        serde_json::from_slice(&manifest_bytes).context("decode Sift archive manifest")?;
    validate_archive_manifest(&manifest)?;

    let mut layout = DataLayout::open(target, StorageRole::All)?;
    if layout.manifest().cluster_id == manifest.source_cluster_id {
        bail!("cold restore must create a new Sift cluster ID");
    }
    layout.mark_restored_from(manifest_uri)?;
    drop(layout);
    shard::write_epoch_maps(target, &manifest.epochs)?;
    let blob_store = BlobStore::open(target, 65_536)?;
    for blob in &manifest.blobs {
        let bytes = service_backup::fetch_backup_object(&blob.object_uri)?;
        let restored = blob_store.put(&bytes, blob.reference.encoding.clone())?;
        if restored.hash != blob.reference.hash || restored.size != blob.reference.size {
            bail!(
                "restored blob {} failed hash/size verification",
                blob.reference.hash
            );
        }
    }

    let journal = crate::DurableJournal::open(target)?;
    let mut streams = [
        ArchiveSignalStream::new(&manifest.segments, SignalKind::Log),
        ArchiveSignalStream::new(&manifest.segments, SignalKind::Metric),
        ArchiveSignalStream::new(&manifest.segments, SignalKind::Span),
    ];
    let mut page = Vec::with_capacity(10_000);
    let mut event_count = 0_u64;
    let mut last_cursor = 0_u64;
    let mut event_id_digest = [0_u8; 32];
    let mut event_content_digest = [0_u8; 32];
    let mut restored_watermarks = ArchiveWatermarks::default();
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
        last_cursor = cursor;
        event_count = event_count.saturating_add(1);
        include_event_id(&mut event_id_digest, &event.event.event_id);
        include_event_content(&mut event_content_digest, &event)?;
        restored_watermarks.include(event.event.signal, event.cursor);
        page.push(event);
        if page.len() == 10_000 {
            journal.restore_stored_page(std::mem::take(&mut page))?;
            page = Vec::with_capacity(10_000);
        }
    }
    if !page.is_empty() {
        journal.restore_stored_page(page)?;
    }
    if event_count != manifest.event_count
        || hex::encode(event_id_digest) != manifest.event_id_sha256
        || hex::encode(event_content_digest) != manifest.event_content_sha256
        || last_cursor != manifest.retained_watermarks.max_cursor()
        || restored_watermarks != manifest.retained_watermarks
        || last_cursor > manifest.raft_snapshot_index
    {
        bail!("Sift archive manifest count, digest, watermark, or Raft snapshot index mismatch");
    }
    journal.set_restored_head(manifest.raft_snapshot_index, manifest.event_count)?;
    drop(journal);
    Ok(manifest)
}

struct ArchiveSignalStream<'a> {
    segments: Vec<&'a ArchiveSegment>,
    next_segment: usize,
    events: VecDeque<StoredEvent>,
    last_cursor: Option<u64>,
}

impl<'a> ArchiveSignalStream<'a> {
    fn new(segments: &'a [ArchiveSegment], signal: SignalKind) -> Self {
        let mut segments = segments
            .iter()
            .filter(|segment| segment.signal == signal)
            .collect::<Vec<_>>();
        segments.sort_by_key(|segment| segment.source.first_cursor);
        Self {
            segments,
            next_segment: 0,
            events: VecDeque::new(),
            last_cursor: None,
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
        while self.events.is_empty() && self.next_segment < self.segments.len() {
            let segment = self.segments[self.next_segment];
            self.next_segment += 1;
            let bytes = service_backup::fetch_backup_object(&segment.object_uri)?;
            verify_bytes(
                &segment.parquet_sha256,
                segment.parquet_bytes,
                &bytes,
                "Parquet segment",
            )?;
            let events = decode_parquet(&bytes)?;
            verify_archive_segment(segment, &events)?;
            self.events = events.into();
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
        let batch = batch?;
        if batch
            .columns()
            .iter()
            .any(|column| column.null_count() != 0)
        {
            bail!("archive Parquet segment contains a null required value");
        }
        let cursors = uint64_column(&batch, 0, "cursor")?;
        let acknowledged = string_column(&batch, 1, "acknowledged_at")?;
        let event_ids = string_column(&batch, 2, "event_id")?;
        let projects = string_column(&batch, 3, "project")?;
        let environments = string_column(&batch, 4, "environment")?;
        let signals = string_column(&batch, 5, "signal")?;
        let occurred = string_column(&batch, 6, "occurred_at")?;
        let json = string_column(&batch, 7, "event_json")?;
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
    }
    Ok(events)
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
    if events.len() as u64 != segment.source.event_count
        || events.first().map(|event| event.cursor) != Some(segment.source.first_cursor)
        || events.last().map(|event| event.cursor) != Some(segment.source.last_cursor)
        || events
            .windows(2)
            .any(|pair| pair[0].cursor >= pair[1].cursor)
        || events
            .iter()
            .any(|event| event.event.signal != segment.signal)
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

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
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
