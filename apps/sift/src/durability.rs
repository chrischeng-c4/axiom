//! Replicated command and snapshot boundary for Sift's phase-one signals.

use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use anyhow::{bail, Context, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use raft_runtime::{Index, OutcomeWindow, RaftStateMachine};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{
    storage::SegmentManifest, AppendResult, DurableJournal, EventEnvelope, EventQuery, SignalKind,
    StoredEvent,
};

const CONTROL_STATE_FILE: &str = "sift-control-state.json";
const CONTROL_STATE_FORMAT_VERSION: u16 = 1;
pub const RAFT_BATCH_MAX_BYTES: usize = 1_048_576;
pub const RAFT_BATCH_MAX_ITEMS: usize = 1_000;
pub const RAFT_BATCH_MAX_DELAY: Duration = Duration::from_millis(10);
pub const SNAPSHOT_CONTENT_TYPE: &str = "application/vnd.axiom.sift-snapshot";

const SNAPSHOT_MAGIC: &[u8; 8] = b"SIFTSNP2";
const SNAPSHOT_FORMAT_VERSION: u16 = 2;
const SNAPSHOT_HEADER_BYTES: usize = 40;
const SNAPSHOT_FRAME_HEADER_BYTES: usize = 16;
const SNAPSHOT_PAGE_EVENTS: usize = 10_000;
const MAX_SNAPSHOT_EVENT_BYTES: usize = 16 * 1024 * 1024;
const COMMAND_MAGIC: &[u8; 8] = b"SIFTCMD1";
const COMMAND_FORMAT_VERSION: u16 = 1;
const COMMAND_FLAG_GZIP: u16 = 1;
const COMMAND_HEADER_BYTES: usize = 20;
const MAX_ENCODED_COMMAND_BYTES: usize = RAFT_BATCH_MAX_BYTES + 64 * 1024;
const APPEND_OUTCOME_WINDOW: u64 = 64;
const ARCHIVE_CHECKPOINT_MAGIC: &[u8; 8] = b"SIFTRCP1";
const ARCHIVE_CHECKPOINT_FORMAT_VERSION: u16 = 1;
const ARCHIVE_CHECKPOINT_HEADER_BYTES: usize = 16;
const MAX_ARCHIVE_CHECKPOINT_BYTES: usize = 64 * 1024;
const LOCAL_CHECKPOINT_MAGIC: &[u8; 8] = b"SIFTLCP1";
const LOCAL_CHECKPOINT_FORMAT_VERSION: u16 = 1;
const RESIDENT_CHECKPOINT_MAGIC: &[u8; 8] = b"SIFTRSD1";
const RESIDENT_CHECKPOINT_FORMAT_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CheckpointPosition {
    applied_index: u64,
    raw_cursor: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ArchiveCheckpointV1 {
    format_version: u16,
    applied_index: u64,
    raw_cursor: u64,
    archive_snapshot_index: u64,
    watermarks: crate::storage::archive::ArchiveWatermarks,
    manifest_uri: String,
    manifest_sha256: String,
    retention_generation: u64,
}

struct ValidatedArchiveStage {
    checkpoint: ArchiveCheckpointV1,
    manifest: crate::storage::archive::ArchiveManifest,
    root: tempfile::TempDir,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LocalCheckpointV1 {
    format_version: u16,
    applied_index: u64,
    raw_cursor: u64,
    local_snapshot_index: u64,
    watermarks: crate::storage::archive::ArchiveWatermarks,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ResidentCheckpointV1 {
    format_version: u16,
    applied_index: u64,
    raw_cursor: u64,
    event_count: u64,
    retention_generation: u64,
    event_content_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetentionFenceV1 {
    pub source_manifest_uri: String,
    pub source_manifest_sha256: String,
    pub target_generation: u64,
    pub evaluate_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SiftCommandV1 {
    AppendEvents {
        events: Vec<EventEnvelope>,
    },
    ArchiveCheckpointBarrier {
        retention_generation: u64,
        manifest_uri: String,
        manifest_sha256: String,
    },
    RetentionFence {
        fence: RetentionFenceV1,
    },
}

impl SiftCommandV1 {
    pub(crate) fn encoded(&self) -> Result<Vec<u8>> {
        // Append batches remain readable by the previous Sift binary. The
        // lifecycle emits the additive barrier variant only after every voter
        // advertises sift-checkpoint-v2.
        self.uncompressed()
    }

    fn encoded_compressed(&self) -> Result<Vec<u8>> {
        let raw = self.uncompressed()?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder
            .write_all(&raw)
            .context("compress Sift state-machine command")?;
        let compressed = encoder
            .finish()
            .context("finish Sift state-machine command compression")?;
        let mut encoded = Vec::with_capacity(COMMAND_HEADER_BYTES + compressed.len());
        encoded.extend_from_slice(COMMAND_MAGIC);
        encoded.extend_from_slice(&COMMAND_FORMAT_VERSION.to_le_bytes());
        encoded.extend_from_slice(&COMMAND_FLAG_GZIP.to_le_bytes());
        encoded.extend_from_slice(&(raw.len() as u32).to_le_bytes());
        encoded.extend_from_slice(&crc32fast::hash(&raw).to_le_bytes());
        encoded.extend_from_slice(&compressed);
        if encoded.len() > MAX_ENCODED_COMMAND_BYTES {
            bail!(
                "encoded Sift Raft batch exceeds the wire limit: {} bytes",
                encoded.len()
            );
        }
        Ok(encoded)
    }

    pub(crate) fn uncompressed_len(&self) -> Result<usize> {
        Ok(self.uncompressed()?.len())
    }

    fn uncompressed(&self) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(self).context("encode Sift state-machine command")?;
        if bytes.len() > RAFT_BATCH_MAX_BYTES {
            bail!(
                "Sift Raft batch exceeds the 1 MiB limit: {} bytes",
                bytes.len()
            );
        }
        Ok(bytes)
    }
}

#[doc(hidden)]
pub fn encode_raft_batch_for_diagnostics(events: Vec<EventEnvelope>) -> Result<Vec<u8>> {
    SiftCommandV1::AppendEvents { events }.encoded_compressed()
}

#[doc(hidden)]
pub fn encode_default_raft_batch_for_diagnostics(events: Vec<EventEnvelope>) -> Result<Vec<u8>> {
    SiftCommandV1::AppendEvents { events }.encoded()
}

#[doc(hidden)]
pub fn decode_raft_batch_for_diagnostics(bytes: &[u8]) -> Result<Vec<EventEnvelope>> {
    match decode_command(bytes)? {
        SiftCommandV1::AppendEvents { events } => Ok(events),
        SiftCommandV1::ArchiveCheckpointBarrier { .. } => {
            bail!("Sift Raft command is an archive checkpoint barrier")
        }
        SiftCommandV1::RetentionFence { .. } => bail!("Sift Raft command is a retention fence"),
    }
}

#[doc(hidden)]
pub fn encode_archive_checkpoint_barrier_for_diagnostics(
    retention_generation: u64,
    manifest_uri: String,
    manifest_sha256: String,
) -> Result<Vec<u8>> {
    SiftCommandV1::ArchiveCheckpointBarrier {
        retention_generation,
        manifest_uri,
        manifest_sha256,
    }
    .encoded()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotMetadata {
    pub applied_index: u64,
    pub last_cursor: u64,
    pub event_count: u64,
}

/// Write one stable journal prefix as framed binary data.
///
/// The writer receives one event at a time. A concurrent append can extend the
/// journal, but it cannot change the prefix captured in the header.
pub(crate) fn write_snapshot(
    journal: &DurableJournal,
    applied_index: u64,
    writer: &mut dyn Write,
) -> Result<SnapshotMetadata> {
    let (last_cursor, event_count) = journal.snapshot_bounds();
    let metadata = SnapshotMetadata {
        applied_index,
        last_cursor,
        event_count,
    };
    write_snapshot_header(writer, metadata)?;

    let mut after = 0_u64;
    let mut written = 0_u64;
    let mut expected_cursor = 1_u64;
    while written < event_count {
        let page = journal
            .query(EventQuery {
                signal: None,
                after,
                limit: SNAPSHOT_PAGE_EVENTS,
            })
            .context("read canonical journal page for snapshot")?;
        let mut progressed = false;
        for stored in page {
            if stored.cursor > last_cursor || written == event_count {
                break;
            }
            if stored.cursor != expected_cursor {
                bail!(
                    "snapshot journal cursor {} is out of order; expected {expected_cursor}",
                    stored.cursor
                );
            }
            write_snapshot_event(writer, &stored)?;
            after = stored.cursor;
            expected_cursor = expected_cursor
                .checked_add(1)
                .context("snapshot journal cursor exhausted u64")?;
            written = written
                .checked_add(1)
                .context("snapshot event count exhausted u64")?;
            progressed = true;
        }
        if !progressed {
            bail!(
                "snapshot ended after {written} events, before declared event count {event_count}"
            );
        }
    }
    if written != event_count || after != last_cursor {
        bail!(
            "snapshot prefix mismatch: wrote {written} events through cursor {after}, expected {event_count} events through cursor {last_cursor}"
        );
    }
    Ok(metadata)
}

/// Restore a seekable snapshot only after a complete validation pass.
///
/// This prevents a corrupt or truncated snapshot from partially filling an
/// empty Sift data directory.
pub(crate) fn restore_seekable_snapshot<R>(
    journal: &DurableJournal,
    reader: &mut R,
) -> Result<SnapshotMetadata>
where
    R: Read + Seek,
{
    if journal.last_cursor() != 0 {
        bail!("snapshot restore requires an empty Sift data directory");
    }

    reader
        .seek(SeekFrom::Start(0))
        .context("seek to the start of the Sift snapshot")?;
    let validated = read_snapshot(reader, |_| Ok(()))?;

    reader
        .seek(SeekFrom::Start(0))
        .context("rewind the validated Sift snapshot")?;
    let mut page = Vec::with_capacity(SNAPSHOT_PAGE_EVENTS);
    let restored = read_snapshot(reader, |event| {
        page.push(event);
        if page.len() == SNAPSHOT_PAGE_EVENTS {
            journal.restore_stored_page(std::mem::take(&mut page))?;
            page.reserve(SNAPSHOT_PAGE_EVENTS);
        }
        Ok(())
    })?;
    if !page.is_empty() {
        journal.restore_stored_page(page)?;
    }
    if restored != validated {
        bail!("Sift snapshot metadata changed between validation and restore");
    }
    if journal.total_event_count() != restored.event_count
        || journal.last_cursor() != restored.last_cursor
    {
        bail!("restored Sift snapshot does not match its declared event count and cursor");
    }
    Ok(restored)
}

/// Spool a non-seekable Raft snapshot in the data directory, validate it, and
/// then restore it in bounded pages. The temporary file stays on the same file
/// system as all other Sift atomic work.
fn restore_streamed_snapshot(
    journal: &DurableJournal,
    current_applied_index: u64,
    reader: &mut dyn Read,
    staged_archive: Option<ValidatedArchiveStage>,
) -> Result<SnapshotMetadata> {
    let tmp_dir = journal.data_dir().join("tmp");
    let mut spool = tempfile::tempfile_in(&tmp_dir)
        .with_context(|| format!("create snapshot spool in {}", tmp_dir.display()))?;
    std::io::copy(reader, &mut spool).context("spool incoming Sift snapshot")?;
    spool
        .sync_all()
        .context("sync incoming Sift snapshot spool")?;
    spool
        .seek(SeekFrom::Start(0))
        .context("rewind incoming Sift snapshot spool")?;
    let mut magic = [0_u8; 8];
    spool
        .read_exact(&mut magic)
        .context("read incoming Sift snapshot magic")?;
    spool
        .seek(SeekFrom::Start(0))
        .context("rewind incoming Sift snapshot after magic")?;
    if &magic == ARCHIVE_CHECKPOINT_MAGIC {
        let checkpoint = read_archive_checkpoint(&mut spool)?;
        return restore_archive_checkpoint(journal, &checkpoint, staged_archive);
    }
    if &magic == LOCAL_CHECKPOINT_MAGIC {
        let checkpoint = read_local_checkpoint(&mut spool)?;
        return restore_local_checkpoint(journal, &checkpoint);
    }
    if &magic == RESIDENT_CHECKPOINT_MAGIC {
        let checkpoint = read_resident_checkpoint(&mut spool)?;
        return restore_resident_checkpoint(journal, &checkpoint);
    }

    let metadata = read_snapshot_header(&mut spool)?;
    if metadata.applied_index <= current_applied_index {
        return Ok(metadata);
    }
    spool
        .seek(SeekFrom::Start(0))
        .context("rewind incoming full Sift snapshot")?;
    restore_seekable_snapshot(journal, &mut spool)
}

/// Validate all snapshot bytes and remote objects without changing the live
/// journal. A full archive restore is staged in `tmp` so the later durable
/// Raft install never discovers a bad manifest or segment after compaction.
fn validate_streamed_snapshot(
    journal: &DurableJournal,
    reader: &mut dyn Read,
) -> Result<Option<ValidatedArchiveStage>> {
    let tmp_dir = journal.data_dir().join("tmp");
    let mut spool = tempfile::tempfile_in(&tmp_dir)
        .with_context(|| format!("create snapshot validation spool in {}", tmp_dir.display()))?;
    std::io::copy(reader, &mut spool).context("spool incoming Sift snapshot for validation")?;
    spool
        .seek(SeekFrom::Start(0))
        .context("rewind incoming Sift snapshot validation spool")?;
    let mut magic = [0_u8; 8];
    spool
        .read_exact(&mut magic)
        .context("read incoming Sift snapshot validation magic")?;
    spool
        .seek(SeekFrom::Start(0))
        .context("rewind incoming Sift snapshot validation spool after magic")?;
    if &magic == ARCHIVE_CHECKPOINT_MAGIC {
        let checkpoint = read_archive_checkpoint(&mut spool)?;
        return validate_archive_checkpoint_install(journal, &checkpoint);
    }
    if &magic == LOCAL_CHECKPOINT_MAGIC {
        let checkpoint = read_local_checkpoint(&mut spool)?;
        restore_local_checkpoint(journal, &checkpoint)?;
        return Ok(None);
    }
    if &magic == RESIDENT_CHECKPOINT_MAGIC {
        let checkpoint = read_resident_checkpoint(&mut spool)?;
        restore_resident_checkpoint(journal, &checkpoint)?;
        return Ok(None);
    }
    read_snapshot(&mut spool, |_| Ok(())).map(|_| None)
}

fn validate_archive_checkpoint_install(
    journal: &DurableJournal,
    checkpoint: &ArchiveCheckpointV1,
) -> Result<Option<ValidatedArchiveStage>> {
    let manifest_bytes = service_backup::fetch_backup_object(&checkpoint.manifest_uri)
        .context("fetch Sift archive checkpoint manifest during validation")?;
    let actual_hash = hex::encode(sha2::Sha256::digest(&manifest_bytes));
    if actual_hash != checkpoint.manifest_sha256 {
        bail!("Sift archive checkpoint manifest failed its SHA-256 check");
    }
    let manifest: crate::storage::archive::ArchiveManifest =
        serde_json::from_slice(&manifest_bytes)
            .context("decode Sift archive checkpoint manifest during validation")?;
    if manifest.raft_snapshot_index != checkpoint.archive_snapshot_index
        || manifest.watermarks != checkpoint.watermarks
        || manifest.raft_snapshot_index != checkpoint.raw_cursor
        || manifest.retention_generation != checkpoint.retention_generation
    {
        bail!("Sift archive checkpoint is not covered by its manifest");
    }
    let empty = journal.last_cursor() == 0 && journal.total_event_count() == 0;
    let local_is_caught_up = journal.last_cursor() >= checkpoint.raw_cursor;
    let local_status = crate::storage::archive::committed_status(journal.data_dir())?;
    let local_generation = local_status
        .as_ref()
        .map(|status| status.retention_generation)
        .unwrap_or_default();
    if local_generation > checkpoint.retention_generation {
        bail!("Sift archive checkpoint retention generation moved backwards");
    }
    let suffix_events = journal.last_cursor().saturating_sub(checkpoint.raw_cursor);
    let expected_local_events = manifest
        .event_count
        .checked_add(suffix_events)
        .context("archive checkpoint event count exhausted u64")?;
    let local_prefix_matches = if local_is_caught_up {
        let (events, digest, _) = journal.checkpoint_identity(checkpoint.raw_cursor)?;
        events == manifest.event_count && hex::encode(digest) == manifest.event_content_sha256
    } else {
        false
    };
    let needs_full_restore = empty
        || !local_is_caught_up
        || checkpoint.retention_generation > local_generation
        || !local_prefix_matches
        || journal.total_event_count() != expected_local_events;
    if needs_full_restore {
        let restored_root = tempfile::tempdir_in(journal.data_dir().join("tmp"))?;
        let restored =
            crate::storage::archive::restore_gcs(&checkpoint.manifest_uri, restored_root.path())
                .context("stage Sift archive checkpoint validation restore")?;
        if restored != manifest {
            bail!("Sift archive checkpoint manifest changed during validation");
        }
        let receipt = crate::storage::archive::ArchiveReceipt {
            manifest_uri: checkpoint.manifest_uri.clone(),
            manifest_sha256: checkpoint.manifest_sha256.clone(),
            manifest: manifest.clone(),
        };
        crate::storage::archive::adopt_verified_archive_receipt(restored_root.path(), &receipt)?;
        let staged = DurableJournal::open(restored_root.path())?;
        crate::storage::archive::evict_committed_cold_segments_at(&staged, chrono::Utc::now())?;
        drop(staged);
        return Ok(Some(ValidatedArchiveStage {
            checkpoint: checkpoint.clone(),
            manifest,
            root: restored_root,
        }));
    }
    Ok(None)
}

fn write_archive_checkpoint(
    checkpoint: &ArchiveCheckpointV1,
    writer: &mut dyn Write,
) -> Result<()> {
    validate_archive_checkpoint(checkpoint)?;
    let payload = serde_json::to_vec(checkpoint).context("encode Sift archive checkpoint")?;
    if payload.is_empty() || payload.len() > MAX_ARCHIVE_CHECKPOINT_BYTES {
        bail!("Sift archive checkpoint payload has an invalid length");
    }
    let payload_len =
        u32::try_from(payload.len()).context("archive checkpoint length exceeds u32")?;
    writer
        .write_all(ARCHIVE_CHECKPOINT_MAGIC)
        .context("write Sift archive checkpoint magic")?;
    writer
        .write_all(&payload_len.to_le_bytes())
        .context("write Sift archive checkpoint length")?;
    writer
        .write_all(&crc32fast::hash(&payload).to_le_bytes())
        .context("write Sift archive checkpoint checksum")?;
    writer
        .write_all(&payload)
        .context("write Sift archive checkpoint payload")
}

fn read_archive_checkpoint(reader: &mut dyn Read) -> Result<ArchiveCheckpointV1> {
    let mut header = [0_u8; ARCHIVE_CHECKPOINT_HEADER_BYTES];
    reader
        .read_exact(&mut header)
        .context("read Sift archive checkpoint header")?;
    if &header[..8] != ARCHIVE_CHECKPOINT_MAGIC {
        bail!("invalid Sift archive checkpoint magic");
    }
    let payload_len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    let expected_checksum = u32::from_le_bytes(header[12..16].try_into().unwrap());
    if payload_len == 0 || payload_len > MAX_ARCHIVE_CHECKPOINT_BYTES {
        bail!("Sift archive checkpoint payload has an invalid length");
    }
    let mut payload = vec![0_u8; payload_len];
    reader
        .read_exact(&mut payload)
        .context("read Sift archive checkpoint payload")?;
    if crc32fast::hash(&payload) != expected_checksum {
        bail!("Sift archive checkpoint checksum mismatch");
    }
    if read_one_or_eof(reader)?.is_some() {
        bail!("Sift archive checkpoint contains trailing bytes");
    }
    let checkpoint: ArchiveCheckpointV1 =
        serde_json::from_slice(&payload).context("decode Sift archive checkpoint")?;
    validate_archive_checkpoint(&checkpoint)?;
    Ok(checkpoint)
}

fn write_local_checkpoint(checkpoint: &LocalCheckpointV1, writer: &mut dyn Write) -> Result<()> {
    validate_local_checkpoint(checkpoint)?;
    let payload = serde_json::to_vec(checkpoint).context("encode Sift local checkpoint")?;
    if payload.is_empty() || payload.len() > MAX_ARCHIVE_CHECKPOINT_BYTES {
        bail!("Sift local checkpoint payload has an invalid length");
    }
    let payload_len =
        u32::try_from(payload.len()).context("local checkpoint length exceeds u32")?;
    writer
        .write_all(LOCAL_CHECKPOINT_MAGIC)
        .context("write Sift local checkpoint magic")?;
    writer
        .write_all(&payload_len.to_le_bytes())
        .context("write Sift local checkpoint length")?;
    writer
        .write_all(&crc32fast::hash(&payload).to_le_bytes())
        .context("write Sift local checkpoint checksum")?;
    writer
        .write_all(&payload)
        .context("write Sift local checkpoint payload")
}

fn read_local_checkpoint(reader: &mut dyn Read) -> Result<LocalCheckpointV1> {
    let mut header = [0_u8; ARCHIVE_CHECKPOINT_HEADER_BYTES];
    reader
        .read_exact(&mut header)
        .context("read Sift local checkpoint header")?;
    if &header[..8] != LOCAL_CHECKPOINT_MAGIC {
        bail!("invalid Sift local checkpoint magic");
    }
    let payload_len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    let expected_checksum = u32::from_le_bytes(header[12..16].try_into().unwrap());
    if payload_len == 0 || payload_len > MAX_ARCHIVE_CHECKPOINT_BYTES {
        bail!("Sift local checkpoint payload has an invalid length");
    }
    let mut payload = vec![0_u8; payload_len];
    reader
        .read_exact(&mut payload)
        .context("read Sift local checkpoint payload")?;
    if crc32fast::hash(&payload) != expected_checksum {
        bail!("Sift local checkpoint checksum mismatch");
    }
    if read_one_or_eof(reader)?.is_some() {
        bail!("Sift local checkpoint contains trailing bytes");
    }
    let checkpoint: LocalCheckpointV1 =
        serde_json::from_slice(&payload).context("decode Sift local checkpoint")?;
    validate_local_checkpoint(&checkpoint)?;
    Ok(checkpoint)
}

fn validate_local_checkpoint(checkpoint: &LocalCheckpointV1) -> Result<()> {
    if checkpoint.format_version != LOCAL_CHECKPOINT_FORMAT_VERSION
        || checkpoint.applied_index == 0
        || checkpoint.raw_cursor != checkpoint.local_snapshot_index
        || checkpoint.watermarks.max_cursor() != checkpoint.local_snapshot_index
    {
        bail!("Sift local checkpoint metadata is invalid");
    }
    Ok(())
}

fn write_resident_checkpoint(
    checkpoint: &ResidentCheckpointV1,
    writer: &mut dyn Write,
) -> Result<()> {
    validate_resident_checkpoint(checkpoint)?;
    let payload = serde_json::to_vec(checkpoint).context("encode Sift resident checkpoint")?;
    if payload.is_empty() || payload.len() > MAX_ARCHIVE_CHECKPOINT_BYTES {
        bail!("Sift resident checkpoint payload has an invalid length");
    }
    let payload_len =
        u32::try_from(payload.len()).context("resident checkpoint length exceeds u32")?;
    writer
        .write_all(RESIDENT_CHECKPOINT_MAGIC)
        .context("write Sift resident checkpoint magic")?;
    writer
        .write_all(&payload_len.to_le_bytes())
        .context("write Sift resident checkpoint length")?;
    writer
        .write_all(&crc32fast::hash(&payload).to_le_bytes())
        .context("write Sift resident checkpoint checksum")?;
    writer
        .write_all(&payload)
        .context("write Sift resident checkpoint payload")
}

fn read_resident_checkpoint(reader: &mut dyn Read) -> Result<ResidentCheckpointV1> {
    let mut header = [0_u8; ARCHIVE_CHECKPOINT_HEADER_BYTES];
    reader
        .read_exact(&mut header)
        .context("read Sift resident checkpoint header")?;
    if &header[..8] != RESIDENT_CHECKPOINT_MAGIC {
        bail!("invalid Sift resident checkpoint magic");
    }
    let payload_len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    let expected_checksum = u32::from_le_bytes(header[12..16].try_into().unwrap());
    if payload_len == 0 || payload_len > MAX_ARCHIVE_CHECKPOINT_BYTES {
        bail!("Sift resident checkpoint payload has an invalid length");
    }
    let mut payload = vec![0_u8; payload_len];
    reader
        .read_exact(&mut payload)
        .context("read Sift resident checkpoint payload")?;
    if crc32fast::hash(&payload) != expected_checksum {
        bail!("Sift resident checkpoint checksum mismatch");
    }
    if read_one_or_eof(reader)?.is_some() {
        bail!("Sift resident checkpoint contains trailing bytes");
    }
    let checkpoint: ResidentCheckpointV1 =
        serde_json::from_slice(&payload).context("decode Sift resident checkpoint")?;
    validate_resident_checkpoint(&checkpoint)?;
    Ok(checkpoint)
}

fn validate_resident_checkpoint(checkpoint: &ResidentCheckpointV1) -> Result<()> {
    if checkpoint.format_version != RESIDENT_CHECKPOINT_FORMAT_VERSION
        || checkpoint.applied_index == 0
        || checkpoint.raw_cursor == 0
        || checkpoint.event_count == 0
        || checkpoint.event_count > checkpoint.raw_cursor
        || checkpoint.event_content_sha256.len() != 64
        || !checkpoint
            .event_content_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("Sift resident checkpoint metadata is invalid");
    }
    Ok(())
}

fn restore_resident_checkpoint(
    journal: &DurableJournal,
    checkpoint: &ResidentCheckpointV1,
) -> Result<SnapshotMetadata> {
    if journal.last_cursor() < checkpoint.raw_cursor {
        bail!(
            "Sift local journal cursor {} is behind resident checkpoint cursor {}",
            journal.last_cursor(),
            checkpoint.raw_cursor
        );
    }
    let (local_prefix_events, local_prefix_digest, retention_generation) =
        journal.checkpoint_identity(checkpoint.raw_cursor)?;
    if retention_generation != checkpoint.retention_generation {
        bail!("Sift resident checkpoint retention generation disagrees with local journal");
    }
    let suffix_events = journal.last_cursor().saturating_sub(checkpoint.raw_cursor);
    let expected_events = checkpoint
        .event_count
        .checked_add(suffix_events)
        .context("resident checkpoint event count exhausted u64")?;
    if journal.total_event_count() != expected_events
        || local_prefix_events != checkpoint.event_count
        || hex::encode(local_prefix_digest) != checkpoint.event_content_sha256
    {
        bail!("Sift resident checkpoint event count disagrees with local journal");
    }
    Ok(SnapshotMetadata {
        applied_index: checkpoint.applied_index,
        last_cursor: checkpoint.raw_cursor,
        event_count: checkpoint.event_count,
    })
}

fn restore_local_checkpoint(
    journal: &DurableJournal,
    checkpoint: &LocalCheckpointV1,
) -> Result<SnapshotMetadata> {
    if journal.last_cursor() < checkpoint.raw_cursor {
        bail!(
            "Sift local journal cursor {} is behind checkpoint cursor {}",
            journal.last_cursor(),
            checkpoint.raw_cursor
        );
    }
    let receipt = crate::storage::archive::archive_journal_local(journal)?;
    if receipt.snapshot_index < checkpoint.local_snapshot_index
        || receipt.watermarks.logs < checkpoint.watermarks.logs
        || receipt.watermarks.metrics < checkpoint.watermarks.metrics
        || receipt.watermarks.traces < checkpoint.watermarks.traces
    {
        bail!("Sift local journal does not cover the checkpoint manifest");
    }
    Ok(SnapshotMetadata {
        applied_index: checkpoint.applied_index,
        last_cursor: journal.last_cursor(),
        event_count: journal.total_event_count(),
    })
}

fn validate_archive_checkpoint(checkpoint: &ArchiveCheckpointV1) -> Result<()> {
    if checkpoint.format_version != ARCHIVE_CHECKPOINT_FORMAT_VERSION {
        bail!(
            "unsupported Sift archive checkpoint format {}",
            checkpoint.format_version
        );
    }
    if checkpoint.applied_index == 0
        || checkpoint.raw_cursor != checkpoint.archive_snapshot_index
        || checkpoint.watermarks.max_cursor() > checkpoint.archive_snapshot_index
        || !checkpoint.manifest_uri.starts_with("gs://")
        || checkpoint.manifest_sha256.len() != 64
        || !checkpoint
            .manifest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("Sift archive checkpoint metadata is invalid");
    }
    Ok(())
}

fn restore_archive_checkpoint(
    journal: &DurableJournal,
    checkpoint: &ArchiveCheckpointV1,
    staged_archive: Option<ValidatedArchiveStage>,
) -> Result<SnapshotMetadata> {
    let manifest_bytes = service_backup::fetch_backup_object(&checkpoint.manifest_uri)
        .context("fetch Sift archive checkpoint manifest")?;
    let actual_hash = hex::encode(sha2::Sha256::digest(&manifest_bytes));
    if actual_hash != checkpoint.manifest_sha256 {
        bail!("Sift archive checkpoint manifest failed its SHA-256 check");
    }
    let expected_manifest: crate::storage::archive::ArchiveManifest =
        serde_json::from_slice(&manifest_bytes)
            .context("decode Sift archive checkpoint manifest")?;
    if expected_manifest.raft_snapshot_index != checkpoint.archive_snapshot_index
        || expected_manifest.watermarks != checkpoint.watermarks
        || expected_manifest.raft_snapshot_index != checkpoint.raw_cursor
        || expected_manifest.retention_generation != checkpoint.retention_generation
    {
        bail!("Sift archive checkpoint is not covered by its manifest");
    }

    let empty = journal.last_cursor() == 0 && journal.total_event_count() == 0;
    let local_is_caught_up = journal.last_cursor() >= checkpoint.raw_cursor;
    let local_status = crate::storage::archive::committed_status(journal.data_dir())?;
    let local_generation = local_status
        .as_ref()
        .map(|status| status.retention_generation);
    if local_generation.is_some_and(|generation| generation > checkpoint.retention_generation) {
        bail!("Sift archive checkpoint retention generation moved backwards");
    }
    let suffix_events = journal.last_cursor().saturating_sub(checkpoint.raw_cursor);
    let expected_local_events = expected_manifest
        .event_count
        .checked_add(suffix_events)
        .context("archive checkpoint event count exhausted u64")?;
    let local_prefix_matches = if local_is_caught_up {
        let (events, digest, _) = journal.checkpoint_identity(checkpoint.raw_cursor)?;
        events == expected_manifest.event_count
            && hex::encode(digest) == expected_manifest.event_content_sha256
    } else {
        false
    };
    let needs_full_restore = empty
        || !local_is_caught_up
        || checkpoint.retention_generation > local_generation.unwrap_or_default()
        || !local_prefix_matches
        || journal.total_event_count() != expected_local_events;
    let receipt = crate::storage::archive::ArchiveReceipt {
        manifest_uri: checkpoint.manifest_uri.clone(),
        manifest_sha256: checkpoint.manifest_sha256.clone(),
        manifest: expected_manifest.clone(),
    };
    if needs_full_restore {
        let restored_root = match staged_archive {
            Some(stage)
                if stage.checkpoint == *checkpoint && stage.manifest == expected_manifest =>
            {
                stage.root
            }
            Some(_) => {
                bail!("Sift archive checkpoint restore lacks its validated staging directory")
            }
            None => {
                let restore_parent = journal.data_dir().join("tmp");
                let restored_root = tempfile::tempdir_in(&restore_parent).with_context(|| {
                    format!(
                        "create archive checkpoint restore directory in {}",
                        restore_parent.display()
                    )
                })?;
                let restored_manifest = crate::storage::archive::restore_gcs(
                    &checkpoint.manifest_uri,
                    restored_root.path(),
                )
                .context("restore Sift archive checkpoint into an isolated data root")?;
                if restored_manifest != expected_manifest {
                    bail!("Sift archive checkpoint manifest changed during restore");
                }
                restored_root
            }
        };
        let restored = DurableJournal::open(restored_root.path())
            .context("open isolated Sift archive checkpoint journal")?;
        journal.adopt_archive_checkpoint(&restored, &receipt, checkpoint.raw_cursor)?;
    } else {
        journal.adopt_archive_coverage(&receipt, checkpoint.raw_cursor)?;
    }
    crate::storage::archive::install_archive_gc_plan(journal.data_dir(), &receipt)?;
    crate::storage::archive::evict_committed_cold_segments_at(journal, chrono::Utc::now())?;
    Ok(SnapshotMetadata {
        applied_index: checkpoint.applied_index,
        last_cursor: expected_manifest.raft_snapshot_index,
        event_count: expected_manifest.event_count,
    })
}

fn write_snapshot_header(writer: &mut dyn Write, metadata: SnapshotMetadata) -> Result<()> {
    let mut header = Vec::with_capacity(SNAPSHOT_HEADER_BYTES);
    header.extend_from_slice(SNAPSHOT_MAGIC);
    header.extend_from_slice(&SNAPSHOT_FORMAT_VERSION.to_le_bytes());
    header.extend_from_slice(&0_u16.to_le_bytes());
    header.extend_from_slice(&metadata.applied_index.to_le_bytes());
    header.extend_from_slice(&metadata.last_cursor.to_le_bytes());
    header.extend_from_slice(&metadata.event_count.to_le_bytes());
    let checksum = crc32fast::hash(&header);
    header.extend_from_slice(&checksum.to_le_bytes());
    debug_assert_eq!(header.len(), SNAPSHOT_HEADER_BYTES);
    writer
        .write_all(&header)
        .context("write Sift snapshot header")
}

fn write_snapshot_event(writer: &mut dyn Write, stored: &StoredEvent) -> Result<()> {
    let payload = serde_json::to_vec(stored).context("encode Sift snapshot event")?;
    if payload.len() > MAX_SNAPSHOT_EVENT_BYTES {
        bail!(
            "Sift snapshot event at cursor {} exceeds the {} byte limit",
            stored.cursor,
            MAX_SNAPSHOT_EVENT_BYTES
        );
    }
    let payload_len = u32::try_from(payload.len()).context("snapshot event length exceeds u32")?;
    let checksum = crc32fast::hash(&payload);
    writer
        .write_all(&stored.cursor.to_le_bytes())
        .context("write Sift snapshot event cursor")?;
    writer
        .write_all(&payload_len.to_le_bytes())
        .context("write Sift snapshot event length")?;
    writer
        .write_all(&checksum.to_le_bytes())
        .context("write Sift snapshot event checksum")?;
    writer
        .write_all(&payload)
        .context("write Sift snapshot event payload")
}

fn read_snapshot<R, F>(reader: &mut R, mut on_event: F) -> Result<SnapshotMetadata>
where
    R: Read,
    F: FnMut(StoredEvent) -> Result<()>,
{
    let metadata = read_snapshot_header(reader)?;
    let mut expected_cursor = 1_u64;
    let mut last_cursor = 0_u64;
    for position in 0..metadata.event_count {
        let mut frame = [0_u8; SNAPSHOT_FRAME_HEADER_BYTES];
        reader
            .read_exact(&mut frame)
            .with_context(|| format!("read Sift snapshot frame {position}"))?;
        let cursor = u64::from_le_bytes(frame[0..8].try_into().unwrap());
        let payload_len = u32::from_le_bytes(frame[8..12].try_into().unwrap()) as usize;
        let expected_checksum = u32::from_le_bytes(frame[12..16].try_into().unwrap());
        if payload_len == 0 || payload_len > MAX_SNAPSHOT_EVENT_BYTES {
            bail!("Sift snapshot event at cursor {cursor} has invalid length {payload_len}");
        }
        let mut payload = vec![0_u8; payload_len];
        reader
            .read_exact(&mut payload)
            .with_context(|| format!("read Sift snapshot event payload at cursor {cursor}"))?;
        let actual_checksum = crc32fast::hash(&payload);
        if actual_checksum != expected_checksum {
            bail!("Sift snapshot checksum mismatch at cursor {cursor}");
        }
        let stored: StoredEvent = serde_json::from_slice(&payload)
            .with_context(|| format!("decode Sift snapshot event at cursor {cursor}"))?;
        if cursor != stored.cursor {
            bail!(
                "Sift snapshot frame cursor {cursor} does not match payload cursor {}",
                stored.cursor
            );
        }
        if cursor != expected_cursor {
            bail!("Sift snapshot cursor {cursor} is out of order; expected {expected_cursor}");
        }
        stored
            .event
            .validate()
            .with_context(|| format!("validate Sift snapshot event at cursor {cursor}"))?;
        on_event(stored)?;
        last_cursor = cursor;
        expected_cursor = expected_cursor
            .checked_add(1)
            .context("snapshot cursor exhausted u64")?;
    }
    if last_cursor != metadata.last_cursor {
        bail!(
            "Sift snapshot ended at cursor {last_cursor}, expected {}",
            metadata.last_cursor
        );
    }
    if metadata.event_count == 0 && metadata.last_cursor != 0 {
        bail!("empty Sift snapshot declares a non-zero last cursor");
    }
    if read_one_or_eof(reader)?.is_some() {
        bail!("Sift snapshot contains trailing bytes");
    }
    Ok(metadata)
}

fn read_snapshot_header(reader: &mut dyn Read) -> Result<SnapshotMetadata> {
    let mut header = [0_u8; SNAPSHOT_HEADER_BYTES];
    reader
        .read_exact(&mut header)
        .context("read Sift snapshot header")?;
    if &header[..SNAPSHOT_MAGIC.len()] != SNAPSHOT_MAGIC {
        if matches!(header.first(), Some(b'{') | Some(b'[')) {
            bail!("legacy Sift JSON snapshot is unsupported; create a new empty data directory");
        }
        bail!("invalid Sift snapshot magic");
    }
    let version = u16::from_le_bytes(header[8..10].try_into().unwrap());
    if version != SNAPSHOT_FORMAT_VERSION {
        bail!("unsupported Sift snapshot format version {version}");
    }
    let flags = u16::from_le_bytes(header[10..12].try_into().unwrap());
    if flags != 0 {
        bail!("unsupported Sift snapshot flags {flags}");
    }
    let expected_checksum = u32::from_le_bytes(header[36..40].try_into().unwrap());
    let actual_checksum = crc32fast::hash(&header[..36]);
    if actual_checksum != expected_checksum {
        bail!("Sift snapshot header checksum mismatch");
    }
    Ok(SnapshotMetadata {
        applied_index: u64::from_le_bytes(header[12..20].try_into().unwrap()),
        last_cursor: u64::from_le_bytes(header[20..28].try_into().unwrap()),
        event_count: u64::from_le_bytes(header[28..36].try_into().unwrap()),
    })
}

fn read_one_or_eof(reader: &mut dyn Read) -> Result<Option<u8>> {
    let mut byte = [0_u8; 1];
    loop {
        match reader.read(&mut byte) {
            Ok(0) => return Ok(None),
            Ok(_) => return Ok(Some(byte[0])),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error).context("check Sift snapshot end"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ControlState {
    format_version: u16,
    applied_index: u64,
    #[serde(default)]
    pending_retention: Option<RetentionFenceV1>,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            format_version: CONTROL_STATE_FORMAT_VERSION,
            applied_index: 0,
            pending_retention: None,
        }
    }
}

/// Applies one committed Raft batch to the canonical per-signal WAL.
pub struct SiftStateMachine {
    journal: Arc<DurableJournal>,
    commit_gate: Mutex<()>,
    control_path: PathBuf,
    control: Mutex<ControlState>,
    append_outcomes: Mutex<OutcomeWindow<Vec<AppendResult>>>,
    checkpoint_position: Mutex<CheckpointPosition>,
    archive_checkpoint: Mutex<Option<ArchiveCheckpointV1>>,
    local_checkpoint: Mutex<Option<LocalCheckpointV1>>,
    resident_checkpoint: Mutex<Option<ResidentCheckpointV1>>,
    validated_archive_stage: Mutex<Option<ValidatedArchiveStage>>,
    applied_index: AtomicU64,
}

impl SiftStateMachine {
    pub fn new(journal: Arc<DurableJournal>) -> Self {
        let data_dir = journal.data_dir().to_path_buf();
        Self::open(data_dir, journal).expect("open Sift state-machine control state")
    }

    pub fn open(data_dir: impl AsRef<Path>, journal: Arc<DurableJournal>) -> Result<Self> {
        let control_dir = data_dir.as_ref().join("control");
        fs::create_dir_all(&control_dir)
            .with_context(|| format!("create Sift control directory {}", control_dir.display()))?;
        set_directory_mode(&control_dir)?;
        let control_path = control_dir.join(CONTROL_STATE_FILE);
        let control =
            if control_path.exists() {
                serde_json::from_slice::<ControlState>(&fs::read(&control_path).with_context(
                    || format!("read Sift control state {}", control_path.display()),
                )?)
                .with_context(|| format!("decode Sift control state {}", control_path.display()))?
            } else {
                ControlState::default()
            };
        if control.format_version != CONTROL_STATE_FORMAT_VERSION {
            bail!(
                "unsupported Sift control state format {}",
                control.format_version
            );
        }
        journal.set_retention_fenced(control.pending_retention.is_some());
        persist_control(&control_path, &control)?;
        let applied_index = control.applied_index;
        let checkpoint_position = CheckpointPosition {
            applied_index,
            raw_cursor: journal.last_cursor(),
        };
        Ok(Self {
            journal,
            commit_gate: Mutex::new(()),
            control_path,
            control: Mutex::new(control),
            append_outcomes: Mutex::new(OutcomeWindow::new(APPEND_OUTCOME_WINDOW)),
            checkpoint_position: Mutex::new(checkpoint_position),
            archive_checkpoint: Mutex::new(None),
            local_checkpoint: Mutex::new(None),
            resident_checkpoint: Mutex::new(None),
            validated_archive_stage: Mutex::new(None),
            applied_index: AtomicU64::new(applied_index),
        })
    }

    pub fn applied_commit_index(&self) -> u64 {
        self.applied_index.load(Ordering::Acquire)
    }

    pub(crate) fn pending_retention_fence(&self) -> Option<RetentionFenceV1> {
        self.control
            .lock()
            .expect("Sift control state lock poisoned")
            .pending_retention
            .clone()
    }

    pub(crate) fn clear_retention_fence_after_checkpoint(
        &self,
        retention_generation: u64,
    ) -> Result<()> {
        let _gate = self
            .commit_gate
            .lock()
            .expect("Sift commit gate lock poisoned");
        let mut control = self
            .control
            .lock()
            .expect("Sift control state lock poisoned");
        if control
            .pending_retention
            .as_ref()
            .is_some_and(|fence| fence.target_generation <= retention_generation)
        {
            control.pending_retention = None;
            persist_control(&self.control_path, &control)?;
            self.journal.set_retention_fenced(false);
        }
        Ok(())
    }

    pub fn apply_local(&self, index: u64, command: &[u8]) -> Result<()> {
        <Self as RaftStateMachine>::apply(self, index, command)
    }

    pub fn take_append_outcomes(&self, index: u64) -> Option<Vec<AppendResult>> {
        self.append_outcomes
            .lock()
            .expect("Sift append outcome lock poisoned")
            .claim(index)
    }

    pub(crate) fn checkpoint_position(&self) -> (u64, u64) {
        let position = *self
            .checkpoint_position
            .lock()
            .expect("Sift checkpoint position lock poisoned");
        (position.applied_index, position.raw_cursor)
    }

    /// Seal one journal prefix while holding the same gate as Raft apply.
    ///
    /// The returned Raft index and raw cursor describe the same durable prefix.
    /// Upload can continue after this method returns while newer Raft entries
    /// append to the journal.
    #[doc(hidden)]
    pub fn capture_archive_prefix(&self) -> Result<(u64, u64, Vec<(SignalKind, SegmentManifest)>)> {
        let _gate = self
            .commit_gate
            .lock()
            .expect("Sift commit gate lock poisoned");
        let (raw_cursor, segments) = self.journal.seal_archive_prefix()?;
        let position = *self
            .checkpoint_position
            .lock()
            .expect("Sift checkpoint position lock poisoned");
        if raw_cursor != position.raw_cursor {
            bail!(
                "Sift archive cursor {raw_cursor} does not match Raft prefix cursor {}",
                position.raw_cursor
            );
        }
        Ok((position.applied_index, raw_cursor, segments))
    }

    /// Rewrite the committed archive prefix while newer Raft entries remain a
    /// local suffix. The archive code applies the retained prefix under the
    /// journal lock and preserves that suffix.
    pub(crate) fn expire_current_archive_at(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<crate::storage::archive::ExpirationReceipt>> {
        let Some(remote) = crate::storage::archive::remote_retained_state(self.journal.data_dir())?
        else {
            return Ok(None);
        };
        if self.journal.last_cursor() < remote.snapshot_index {
            bail!("Sift journal is behind its committed archive cursor");
        }
        let suffix_events = self.journal.last_cursor() - remote.snapshot_index;
        let expected_local_events = remote
            .event_count
            .checked_add(suffix_events)
            .context("Sift retained event count exhausted u64")?;
        let recovery_pending = self.journal.recovery_required()
            || self.journal.total_event_count() != expected_local_events
            || self.journal.retention_generation() != remote.retention_generation;
        let receipt = crate::storage::archive::expire_committed_events_at(&self.journal, now)?;
        if receipt.expired_events > 0 || recovery_pending {
            Ok(Some(receipt))
        } else {
            Ok(None)
        }
    }

    #[doc(hidden)]
    pub fn prepare_archive_checkpoint(&self, applied_index: u64, raw_cursor: u64) -> Result<()> {
        if applied_index == 0 {
            bail!("cannot checkpoint an empty Sift Raft prefix");
        }
        let status = crate::storage::archive::committed_status(self.journal.data_dir())?
            .context("Sift Raft checkpoint requires a committed remote archive")?;
        let archive_snapshot_index = status.snapshot_index;
        if archive_snapshot_index != raw_cursor {
            bail!(
                "committed Sift archive cursor {archive_snapshot_index} does not equal Raft prefix cursor {raw_cursor}"
            );
        }
        let checkpoint = ArchiveCheckpointV1 {
            format_version: ARCHIVE_CHECKPOINT_FORMAT_VERSION,
            applied_index,
            raw_cursor,
            archive_snapshot_index,
            watermarks: status.watermarks,
            manifest_uri: status.manifest_uri,
            manifest_sha256: status.manifest_sha256,
            retention_generation: status.retention_generation,
        };
        validate_archive_checkpoint(&checkpoint)?;
        *self
            .archive_checkpoint
            .lock()
            .expect("Sift archive checkpoint lock poisoned") = Some(checkpoint);
        Ok(())
    }

    #[doc(hidden)]
    pub fn prepare_local_checkpoint(&self, applied_index: u64, raw_cursor: u64) -> Result<()> {
        if applied_index == 0 {
            bail!("cannot checkpoint an empty Sift Raft prefix");
        }
        let status = crate::storage::archive::local_committed_status(self.journal.data_dir())?
            .context("Sift Raft checkpoint requires a committed local segment set")?;
        if status.snapshot_index != raw_cursor {
            bail!(
                "committed local cursor {} does not equal Raft prefix cursor {raw_cursor}",
                status.snapshot_index
            );
        }
        let checkpoint = LocalCheckpointV1 {
            format_version: LOCAL_CHECKPOINT_FORMAT_VERSION,
            applied_index,
            raw_cursor,
            local_snapshot_index: status.snapshot_index,
            watermarks: status.watermarks,
        };
        validate_local_checkpoint(&checkpoint)?;
        *self
            .local_checkpoint
            .lock()
            .expect("Sift local checkpoint lock poisoned") = Some(checkpoint);
        Ok(())
    }

    /// Prepare a small Raft-only checkpoint backed by each voter's durable
    /// local journal. This never authorizes WAL or archive deletion.
    #[doc(hidden)]
    pub fn prepare_resident_checkpoint(&self, applied_index: u64, raw_cursor: u64) -> Result<()> {
        let _gate = self
            .commit_gate
            .lock()
            .expect("Sift commit gate lock poisoned");
        let position = *self
            .checkpoint_position
            .lock()
            .expect("Sift checkpoint position lock poisoned");
        if applied_index == 0
            || position.applied_index != applied_index
            || position.raw_cursor != raw_cursor
            || self.journal.last_cursor() != raw_cursor
        {
            bail!("Sift resident checkpoint moved while it was being prepared");
        }
        let (event_count, event_content_digest, retention_generation) =
            self.journal.checkpoint_identity(raw_cursor)?;
        let checkpoint = ResidentCheckpointV1 {
            format_version: RESIDENT_CHECKPOINT_FORMAT_VERSION,
            applied_index,
            raw_cursor,
            event_count,
            retention_generation,
            event_content_sha256: hex::encode(event_content_digest),
        };
        validate_resident_checkpoint(&checkpoint)?;
        *self
            .resident_checkpoint
            .lock()
            .expect("Sift resident checkpoint lock poisoned") = Some(checkpoint);
        Ok(())
    }

    fn archive_checkpoint_for(&self, index: u64) -> Result<Option<ArchiveCheckpointV1>> {
        if let Some(checkpoint) = self
            .archive_checkpoint
            .lock()
            .expect("Sift archive checkpoint lock poisoned")
            .as_ref()
            .filter(|checkpoint| checkpoint.applied_index == index)
            .cloned()
        {
            return Ok(Some(checkpoint));
        }
        let (applied_index, raw_cursor) = self.checkpoint_position();
        if applied_index != index {
            return Ok(None);
        }
        if crate::storage::archive::committed_status(self.journal.data_dir())?
            .is_none_or(|status| status.snapshot_index != raw_cursor)
        {
            return Ok(None);
        }
        self.prepare_archive_checkpoint(applied_index, raw_cursor)?;
        Ok(self
            .archive_checkpoint
            .lock()
            .expect("Sift archive checkpoint lock poisoned")
            .clone())
    }

    fn local_checkpoint_for(&self, index: u64) -> Result<Option<LocalCheckpointV1>> {
        if let Some(checkpoint) = self
            .local_checkpoint
            .lock()
            .expect("Sift local checkpoint lock poisoned")
            .as_ref()
            .filter(|checkpoint| checkpoint.applied_index == index)
            .cloned()
        {
            return Ok(Some(checkpoint));
        }
        let (applied_index, raw_cursor) = self.checkpoint_position();
        if applied_index != index
            || crate::storage::archive::local_committed_status(self.journal.data_dir())?
                .is_none_or(|status| status.snapshot_index != raw_cursor)
        {
            return Ok(None);
        }
        self.prepare_local_checkpoint(applied_index, raw_cursor)?;
        Ok(self
            .local_checkpoint
            .lock()
            .expect("Sift local checkpoint lock poisoned")
            .clone())
    }

    fn resident_checkpoint_for(&self, index: u64) -> Option<ResidentCheckpointV1> {
        self.resident_checkpoint
            .lock()
            .expect("Sift resident checkpoint lock poisoned")
            .as_ref()
            .filter(|checkpoint| checkpoint.applied_index == index)
            .cloned()
    }
}

impl RaftStateMachine for SiftStateMachine {
    fn snapshot_capability(&self) -> Option<&'static str> {
        Some("sift-checkpoint-v3")
    }

    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        if index <= self.applied_index.load(Ordering::Acquire) {
            return Ok(());
        }
        let command_limit = if command.starts_with(COMMAND_MAGIC) {
            MAX_ENCODED_COMMAND_BYTES
        } else {
            RAFT_BATCH_MAX_BYTES
        };
        if command.len() > command_limit {
            bail!("Sift Raft batch exceeds its wire limit");
        }
        let command = decode_command(command)?;
        let _gate = self
            .commit_gate
            .lock()
            .expect("Sift commit gate lock poisoned");
        if index <= self.applied_index.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut pending_retention_update = None;
        let results = match command {
            SiftCommandV1::AppendEvents { events } => {
                validate_events(&events)?;
                self.journal
                    .append_durable_batch(events)?
                    .into_iter()
                    .map(|result| result.with_commit_index(index))
                    .collect()
            }
            SiftCommandV1::ArchiveCheckpointBarrier {
                retention_generation,
                manifest_uri,
                manifest_sha256,
            } => {
                if !manifest_uri.starts_with("gs://")
                    || manifest_sha256.len() != 64
                    || !manifest_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
                {
                    bail!("Sift archive checkpoint barrier identity is invalid");
                }
                if self.journal.retention_generation() > retention_generation {
                    bail!("Sift archive checkpoint barrier moved retention backwards");
                }
                Vec::new()
            }
            SiftCommandV1::RetentionFence { fence } => {
                validate_retention_fence(&fence)?;
                if fence.target_generation < self.journal.retention_generation() {
                    bail!("Sift retention fence moved retention backwards");
                }
                pending_retention_update = Some(
                    (self.journal.retention_generation() < fence.target_generation)
                        .then_some(fence),
                );
                Vec::new()
            }
        };

        let mut control = self
            .control
            .lock()
            .expect("Sift control state lock poisoned");
        control.applied_index = index;
        if let Some(update) = pending_retention_update {
            control.pending_retention = update;
        }
        persist_control(&self.control_path, &control)?;
        self.journal
            .set_retention_fenced(control.pending_retention.is_some());
        *self
            .checkpoint_position
            .lock()
            .expect("Sift checkpoint position lock poisoned") = CheckpointPosition {
            applied_index: index,
            raw_cursor: self.journal.last_cursor(),
        };
        let mut outcomes = self
            .append_outcomes
            .lock()
            .expect("Sift append outcome lock poisoned");
        outcomes.insert(index, results);
        outcomes.advance(index);
        self.applied_index.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        let applied_index = self
            .control
            .lock()
            .expect("Sift control state lock poisoned")
            .applied_index;
        write_snapshot(&self.journal, applied_index, writer)?;
        Ok(())
    }

    fn snapshot_at(&self, index: Index, writer: &mut dyn std::io::Write) -> Result<()> {
        if let Some(checkpoint) = self.archive_checkpoint_for(index)? {
            return write_archive_checkpoint(&checkpoint, writer);
        }
        if let Some(checkpoint) = self.local_checkpoint_for(index)? {
            return write_local_checkpoint(&checkpoint, writer);
        }
        if let Some(checkpoint) = self.resident_checkpoint_for(index) {
            return write_resident_checkpoint(&checkpoint, writer);
        }
        if index != self.applied_index() {
            bail!(
                "Sift cannot snapshot Raft prefix {index}; current applied index is {}",
                self.applied_index()
            );
        }
        self.snapshot(writer)
    }

    fn validate_snapshot(&self, reader: &mut dyn Read) -> Result<()> {
        let stage = validate_streamed_snapshot(&self.journal, reader)?;
        *self
            .validated_archive_stage
            .lock()
            .expect("Sift validated archive stage lock poisoned") = stage;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn std::io::Read) -> Result<()> {
        let current_applied_index = self.applied_index.load(Ordering::Acquire);
        let staged_archive = self
            .validated_archive_stage
            .lock()
            .expect("Sift validated archive stage lock poisoned")
            .take();
        let snapshot = restore_streamed_snapshot(
            &self.journal,
            current_applied_index,
            reader,
            staged_archive,
        )?;
        let pending_retention = self
            .control
            .lock()
            .expect("Sift control state lock poisoned")
            .pending_retention
            .clone()
            .filter(|fence| fence.target_generation > self.journal.retention_generation());
        if snapshot.applied_index <= current_applied_index {
            let mut control = self
                .control
                .lock()
                .expect("Sift control state lock poisoned");
            if control.pending_retention != pending_retention {
                control.pending_retention = pending_retention;
                persist_control(&self.control_path, &control)?;
            }
            self.journal
                .set_retention_fenced(control.pending_retention.is_some());
            return Ok(());
        }
        let retention_fenced = pending_retention.is_some();
        let restored = ControlState {
            format_version: CONTROL_STATE_FORMAT_VERSION,
            applied_index: snapshot.applied_index,
            pending_retention,
        };
        persist_control(&self.control_path, &restored)?;
        *self
            .control
            .lock()
            .expect("Sift control state lock poisoned") = restored;
        self.journal.set_retention_fenced(retention_fenced);
        self.applied_index
            .store(snapshot.applied_index, Ordering::Release);
        *self
            .checkpoint_position
            .lock()
            .expect("Sift checkpoint position lock poisoned") = CheckpointPosition {
            applied_index: snapshot.applied_index,
            raw_cursor: self.journal.last_cursor(),
        };
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied_index.load(Ordering::Acquire)
    }
}

fn validate_events(events: &[EventEnvelope]) -> Result<()> {
    if events.is_empty() {
        bail!("Sift Raft batch must not be empty");
    }
    let signal = events[0].signal;
    if events.iter().any(|event| event.signal != signal) {
        bail!("Sift Raft batch must contain exactly one signal");
    }
    for event in events {
        event.validate()?;
    }
    Ok(())
}

fn validate_retention_fence(fence: &RetentionFenceV1) -> Result<()> {
    if fence.target_generation == 0
        || !fence.source_manifest_uri.starts_with("gs://")
        || fence.source_manifest_sha256.len() != 64
        || !fence
            .source_manifest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || chrono::DateTime::parse_from_rfc3339(&fence.evaluate_at).is_err()
    {
        bail!("Sift retention fence metadata is invalid");
    }
    Ok(())
}

fn decode_command(bytes: &[u8]) -> Result<SiftCommandV1> {
    if !bytes.starts_with(COMMAND_MAGIC) {
        if bytes.len() > RAFT_BATCH_MAX_BYTES {
            bail!("legacy Sift Raft batch exceeds the 1 MiB limit");
        }
        return serde_json::from_slice(bytes).context("decode legacy Sift command v1");
    }
    if bytes.len() < COMMAND_HEADER_BYTES {
        bail!("compressed Sift Raft batch header is truncated");
    }
    let version = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
    let flags = u16::from_le_bytes(bytes[10..12].try_into().unwrap());
    let expected_len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let expected_crc = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
    if version != COMMAND_FORMAT_VERSION {
        bail!("unsupported Sift Raft command format {version}");
    }
    if flags != COMMAND_FLAG_GZIP {
        bail!("unsupported Sift Raft command flags {flags}");
    }
    if expected_len > RAFT_BATCH_MAX_BYTES {
        bail!("compressed Sift Raft batch declares more than 1 MiB");
    }
    let mut decoder = GzDecoder::new(&bytes[COMMAND_HEADER_BYTES..]);
    let mut raw = Vec::with_capacity(expected_len.min(RAFT_BATCH_MAX_BYTES));
    decoder
        .by_ref()
        .take((RAFT_BATCH_MAX_BYTES + 1) as u64)
        .read_to_end(&mut raw)
        .context("decompress Sift Raft command")?;
    if raw.len() != expected_len {
        bail!(
            "compressed Sift Raft batch length mismatch: expected {expected_len}, found {}",
            raw.len()
        );
    }
    if crc32fast::hash(&raw) != expected_crc {
        bail!("compressed Sift Raft batch checksum mismatch");
    }
    serde_json::from_slice(&raw).context("decode compressed Sift command v1")
}

fn persist_control(path: &Path, control: &ControlState) -> Result<()> {
    storage_durable::atomic_write(
        path,
        &serde_json::to_vec_pretty(control)?,
        storage_durable::FsyncPolicy::Always,
    )
    .with_context(|| format!("atomically persist Sift control state {}", path.display()))?;
    set_file_mode(path)
}

#[cfg(unix)]
fn set_directory_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("set private control directory mode on {}", path.display()))
}

#[cfg(not(unix))]
fn set_directory_mode(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_file_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("set private control file mode on {}", path.display()))
}

#[cfg(not(unix))]
fn set_file_mode(_path: &Path) -> Result<()> {
    Ok(())
}
