//! Snapshot writer/loader for the temporal knowledge graph.
//!
//! Snapshots capture full engine state (entities + relations) into a single file
//! using serde JSON serialization. They are used for fast recovery by avoiding
//! full WAL replay from the beginning of time.
//!
//! Format (v2, 80 bytes):
//!   magic(4) + version(4) + created_at(8) + entity_count(8) + relation_count(8)
//!   + wal_file_timestamp(8) + wal_position_in_file(8) + checksum(32)
//!
//! The old v1 format (72 bytes) stored a single `wal_position: u64`, which was
//! a per-file byte offset that reset to 32 on every WAL rotation — unsafe as a
//! global LSN and caused pre-snapshot re-replay across rotations. v1 files are
//! readable via graceful fallback: the stored position is untrusted and the
//! loader signals `WalReplayStart::FullReplay` (correct but slower — no
//! converter is provided).
//!
//! Atomic writes via `.tmp` → `.snap` rename.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

use crate::engine::CtxInfEngine;
use crate::error::{CtxInfError, Result};
use crate::types::{Entity, Relation};

/// Snapshot file magic: "CISN" (Context Inference SNapshot)
pub const SNAPSHOT_MAGIC: &[u8; 4] = b"CISN";

/// Current snapshot format version.
///
/// - v1 (deprecated): `wal_position: u64` — per-file byte offset. Unsafe across
///   WAL rotation (see
///   `bug-ctx-inf-db-snapshot-wal-position-file-relative-across-rotation`).
/// - v2: `(wal_file_timestamp: u64, wal_position_in_file: u64)` — rotation-safe.
pub const SNAPSHOT_VERSION: u32 = 2;

/// v1 header size in bytes (legacy; retained for graceful-fallback read path).
pub const V1_HEADER_SIZE: usize = 72;
/// v2 header size in bytes.
pub const V2_HEADER_SIZE: usize = 80;

/// Starting point for WAL replay, derived from a loaded snapshot.
///
/// A v2 snapshot captures the maximum timestamp among already-rotated WAL files
/// at snapshot-time (`wal_file_timestamp`; `0` if none rotated yet) plus the
/// byte offset within the then-active `wal-current.log` (`wal_position_in_file`).
///
/// On recovery, the first WAL file whose parsed filename timestamp is strictly
/// greater than `wal_file_timestamp` is the file that was active at snapshot-time
/// — its byte prefix up to `wal_position_in_file` is pre-snapshot and must be
/// skipped; its suffix is the post-snapshot delta to replay. All files with
/// timestamp ≤ `wal_file_timestamp` are fully pre-snapshot and skipped in their
/// entirety. `wal-current.log` is treated as having timestamp `u64::MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalReplayStart {
    /// v2 semantics: skip every WAL file with `parsed_ts <= wal_file_timestamp`
    /// entirely; the first file with `parsed_ts > wal_file_timestamp` gets a
    /// byte-offset skip at `wal_position_in_file`; subsequent newer files are
    /// replayed fully.
    FromPoint {
        wal_file_timestamp: u64,
        wal_position_in_file: u64,
    },
    /// Used when a v1 snapshot is encountered (legacy per-file byte offset is
    /// untrustworthy across rotation) OR when no snapshot exists. Replay every
    /// WAL file in full.
    FullReplay,
}

/// Snapshot file header.
///
/// Two on-disk shapes exist:
/// - **v1 (72 bytes, legacy)**: magic(4) + version(4) + created_at(8)
///   + entity_count(8) + relation_count(8) + wal_position(8) + checksum(32).
///   `wal_position` is a per-file byte offset; unsafe across rotation. Loaded
///   via graceful fallback to full WAL replay — no converter.
/// - **v2 (80 bytes, current)**: same prefix, but the `wal_position` slot is
///   replaced by `wal_file_timestamp(8) + wal_position_in_file(8)`.
#[derive(Debug, Clone)]
pub struct SnapshotHeader {
    pub magic: [u8; 4],
    pub version: u32,
    /// Nanoseconds since UNIX_EPOCH when snapshot was created.
    pub created_at: i64,
    pub entity_count: u64,
    pub relation_count: u64,
    /// Maximum `wal-<ts>.log` timestamp among WAL files that were already
    /// rotated-out at snapshot-time. `0` if no rotation had happened yet.
    /// The currently-active `wal-current.log` is treated as `u64::MAX` on replay.
    pub wal_file_timestamp: u64,
    /// Byte offset within the then-active `wal-current.log` at snapshot-time.
    pub wal_position_in_file: u64,
    pub checksum: [u8; 32],
}

impl SnapshotHeader {
    pub fn new(
        entity_count: u64,
        relation_count: u64,
        wal_file_timestamp: u64,
        wal_position_in_file: u64,
    ) -> Self {
        Self {
            magic: *SNAPSHOT_MAGIC,
            version: SNAPSHOT_VERSION,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64,
            entity_count,
            relation_count,
            wal_file_timestamp,
            wal_position_in_file,
            checksum: [0u8; 32],
        }
    }

    /// Write v2 header (80 bytes). Emits the current format only; v1 writes
    /// are intentionally unsupported.
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.version.to_be_bytes())?;
        writer.write_all(&self.created_at.to_be_bytes())?;
        writer.write_all(&self.entity_count.to_be_bytes())?;
        writer.write_all(&self.relation_count.to_be_bytes())?;
        writer.write_all(&self.wal_file_timestamp.to_be_bytes())?;
        writer.write_all(&self.wal_position_in_file.to_be_bytes())?;
        writer.write_all(&self.checksum)?;
        Ok(())
    }

    /// Read a header. Supports both v1 (72 bytes, legacy) and v2 (80 bytes).
    /// Returns `(header, replay_start)` — v1 always yields `FullReplay`.
    pub fn read<R: Read>(reader: &mut R) -> Result<(Self, WalReplayStart)> {
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|e| CtxInfError::Snapshot(format!("read magic: {}", e)))?;

        if magic != *SNAPSHOT_MAGIC {
            return Err(CtxInfError::Snapshot(format!(
                "invalid magic: expected {:?}, got {:?}",
                SNAPSHOT_MAGIC, magic
            )));
        }

        let mut buf4 = [0u8; 4];

        reader
            .read_exact(&mut buf4)
            .map_err(|e| CtxInfError::Snapshot(format!("read version: {}", e)))?;
        let version = u32::from_be_bytes(buf4);

        match version {
            1 => Self::read_v1_body(reader, magic),
            SNAPSHOT_VERSION => Self::read_v2_body(reader, magic),
            other => Err(CtxInfError::Snapshot(format!(
                "unsupported snapshot version: {}",
                other
            ))),
        }
    }

    /// v1 body (60 bytes after magic+version): created_at(8) + entity_count(8)
    /// + relation_count(8) + wal_position(8) + checksum(32). Returns
    /// `WalReplayStart::FullReplay` — the legacy `wal_position` is a per-file
    /// byte offset (see bug slug) and cannot be used as a global skip point.
    fn read_v1_body<R: Read>(reader: &mut R, magic: [u8; 4]) -> Result<(Self, WalReplayStart)> {
        let mut buf8 = [0u8; 8];

        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read created_at: {}", e)))?;
        let created_at = i64::from_be_bytes(buf8);

        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read entity_count: {}", e)))?;
        let entity_count = u64::from_be_bytes(buf8);

        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read relation_count: {}", e)))?;
        let relation_count = u64::from_be_bytes(buf8);

        // v1 stored a single wal_position here; read+discard (untrusted).
        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read wal_position(v1): {}", e)))?;
        let legacy_wal_position = u64::from_be_bytes(buf8);

        let mut checksum = [0u8; 32];
        reader
            .read_exact(&mut checksum)
            .map_err(|e| CtxInfError::Snapshot(format!("read checksum: {}", e)))?;

        warn!(
            "Loaded v1 snapshot (legacy wal_position={} ignored); falling back to full WAL replay",
            legacy_wal_position
        );

        let header = Self {
            magic,
            version: 1,
            created_at,
            entity_count,
            relation_count,
            // v2 fields are meaningless for v1; zero them and signal full replay.
            wal_file_timestamp: 0,
            wal_position_in_file: 0,
            checksum,
        };
        Ok((header, WalReplayStart::FullReplay))
    }

    /// v2 body (68 bytes after magic+version): created_at(8) + entity_count(8)
    /// + relation_count(8) + wal_file_timestamp(8) + wal_position_in_file(8)
    /// + checksum(32).
    fn read_v2_body<R: Read>(reader: &mut R, magic: [u8; 4]) -> Result<(Self, WalReplayStart)> {
        let mut buf8 = [0u8; 8];

        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read created_at: {}", e)))?;
        let created_at = i64::from_be_bytes(buf8);

        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read entity_count: {}", e)))?;
        let entity_count = u64::from_be_bytes(buf8);

        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read relation_count: {}", e)))?;
        let relation_count = u64::from_be_bytes(buf8);

        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read wal_file_timestamp: {}", e)))?;
        let wal_file_timestamp = u64::from_be_bytes(buf8);

        reader
            .read_exact(&mut buf8)
            .map_err(|e| CtxInfError::Snapshot(format!("read wal_position_in_file: {}", e)))?;
        let wal_position_in_file = u64::from_be_bytes(buf8);

        let mut checksum = [0u8; 32];
        reader
            .read_exact(&mut checksum)
            .map_err(|e| CtxInfError::Snapshot(format!("read checksum: {}", e)))?;

        let replay = WalReplayStart::FromPoint {
            wal_file_timestamp,
            wal_position_in_file,
        };

        let header = Self {
            magic,
            version: SNAPSHOT_VERSION,
            created_at,
            entity_count,
            relation_count,
            wal_file_timestamp,
            wal_position_in_file,
            checksum,
        };
        Ok((header, replay))
    }
}

/// Serializable snapshot payload — entities and relations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotData {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

/// Snapshot writer — captures engine state into a single file.
pub struct SnapshotWriter;

impl SnapshotWriter {
    /// Atomically create a snapshot of the engine state.
    /// Returns the final path of the snapshot file.
    ///
    /// `wal_file_timestamp` is the maximum unix-second timestamp among
    /// already-rotated WAL files at snapshot-time (`0` if none rotated yet);
    /// `wal_position_in_file` is the byte offset within the currently-active
    /// `wal-current.log`.
    pub fn create(
        engine: &CtxInfEngine,
        data_dir: impl AsRef<Path>,
        wal_file_timestamp: u64,
        wal_position_in_file: u64,
    ) -> Result<PathBuf> {
        let data_dir = data_dir.as_ref();
        fs::create_dir_all(data_dir)
            .map_err(|e| CtxInfError::Snapshot(format!("create data dir: {}", e)))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_path = data_dir.join(format!("snapshot-{}.tmp", timestamp));
        let final_path = data_dir.join(format!("snapshot-{}.snap", timestamp));

        info!(
            "Creating snapshot at {} (wal_file_timestamp={}, wal_position_in_file={})",
            final_path.display(),
            wal_file_timestamp,
            wal_position_in_file,
        );

        // Collect data from engine.
        let entities: Vec<Entity> = engine.entities.iter().map(|e| e.value().clone()).collect();
        let relations: Vec<Relation> = engine.relations.iter().map(|r| r.value().clone()).collect();

        let data = SnapshotData {
            entities,
            relations,
        };

        // Serialize to JSON.
        let data_bytes = serde_json::to_vec(&data)
            .map_err(|e| CtxInfError::Snapshot(format!("serialize: {}", e)))?;

        // Compute SHA256 of the JSON payload.
        let mut hasher = Sha256::new();
        hasher.update(&data_bytes);
        let checksum: [u8; 32] = hasher.finalize().into();

        // Build header.
        let mut header = SnapshotHeader::new(
            data.entities.len() as u64,
            data.relations.len() as u64,
            wal_file_timestamp,
            wal_position_in_file,
        );
        header.checksum = checksum;

        // Write header + data to temp file.
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .map_err(|e| CtxInfError::Snapshot(format!("create temp file: {}", e)))?;
        let mut writer = BufWriter::new(file);

        header
            .write(&mut writer)
            .map_err(|e| CtxInfError::Snapshot(format!("write header: {}", e)))?;
        writer
            .write_all(&data_bytes)
            .map_err(|e| CtxInfError::Snapshot(format!("write payload: {}", e)))?;
        writer
            .flush()
            .map_err(|e| CtxInfError::Snapshot(format!("flush: {}", e)))?;
        writer
            .get_ref()
            .sync_all()
            .map_err(|e| CtxInfError::Snapshot(format!("fsync: {}", e)))?;

        // Atomic rename .tmp → .snap.
        fs::rename(&temp_path, &final_path)
            .map_err(|e| CtxInfError::Snapshot(format!("rename: {}", e)))?;

        info!(
            "Snapshot created: {} entities, {} relations",
            header.entity_count, header.relation_count
        );

        Ok(final_path)
    }
}

/// Snapshot loader — restores engine state from the latest snapshot.
pub struct SnapshotLoader;

impl SnapshotLoader {
    /// Load the latest snapshot from `data_dir`. Returns the snapshot data and
    /// the WAL replay starting point, or `None` if no snapshot exists.
    ///
    /// For v1 snapshots, the legacy per-file byte offset cannot be trusted
    /// across rotation, so the return is `WalReplayStart::FullReplay` — the
    /// caller replays every WAL file in full. This is slower but correct (R2).
    pub fn load_latest(
        data_dir: impl AsRef<Path>,
    ) -> Result<Option<(SnapshotData, WalReplayStart)>> {
        Ok(Self::load_latest_full(data_dir)?.map(|(d, replay, _)| (d, replay)))
    }

    /// Like `load_latest`, but also returns the snapshot's `created_at` (nanoseconds
    /// since UNIX_EPOCH, read from the header). Used by recovery to stamp `tx_from`
    /// on pre-bitemporal snapshot rows (D1 / R6).
    pub fn load_latest_full(
        data_dir: impl AsRef<Path>,
    ) -> Result<Option<(SnapshotData, WalReplayStart, i64)>> {
        let snapshots = find_snapshot_files(data_dir.as_ref())?;
        if snapshots.is_empty() {
            return Ok(None);
        }

        let latest = snapshots.last().unwrap();
        info!("Loading snapshot: {}", latest.display());

        let file = File::open(latest).map_err(|e| CtxInfError::Snapshot(format!("open: {}", e)))?;
        let mut reader = BufReader::new(file);

        let (header, replay_start) = SnapshotHeader::read(&mut reader)?;

        let mut data_buffer = Vec::new();
        reader
            .read_to_end(&mut data_buffer)
            .map_err(|e| CtxInfError::Snapshot(format!("read payload: {}", e)))?;

        // Verify SHA256.
        let mut hasher = Sha256::new();
        hasher.update(&data_buffer);
        let computed: [u8; 32] = hasher.finalize().into();

        if computed != header.checksum {
            return Err(CtxInfError::Snapshot(
                "checksum mismatch: snapshot data corrupted".into(),
            ));
        }

        let data: SnapshotData = serde_json::from_slice(&data_buffer)
            .map_err(|e| CtxInfError::Snapshot(format!("deserialize: {}", e)))?;

        info!(
            "Loaded snapshot v{}: {} entities, {} relations (replay_start={:?})",
            header.version,
            data.entities.len(),
            data.relations.len(),
            replay_start,
        );

        Ok(Some((data, replay_start, header.created_at)))
    }
}

/// Find all snapshot files in `data_dir`, sorted by filename (oldest first).
pub fn find_snapshot_files(data_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let data_dir = data_dir.as_ref();
    if !data_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();
    for entry in
        fs::read_dir(data_dir).map_err(|e| CtxInfError::Snapshot(format!("read dir: {}", e)))?
    {
        let entry = entry.map_err(|e| CtxInfError::Snapshot(format!("read dir entry: {}", e)))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("snapshot-") && name.ends_with(".snap") {
                    snapshots.push(path);
                }
            }
        }
    }

    snapshots.sort();
    Ok(snapshots)
}

/// Delete old snapshots, keeping only the most recent `keep_count`.
pub fn cleanup_old_snapshots(data_dir: impl AsRef<Path>, keep_count: usize) -> Result<usize> {
    let mut snapshots = find_snapshot_files(data_dir)?;

    if snapshots.len() <= keep_count {
        return Ok(0);
    }

    // Sort newest first so we can take the tail.
    snapshots.sort_by(|a, b| b.cmp(a));

    let to_delete = &snapshots[keep_count..];
    let mut deleted = 0;

    for path in to_delete {
        match fs::remove_file(path) {
            Ok(_) => {
                debug!("Deleted old snapshot: {}", path.display());
                deleted += 1;
            }
            Err(e) => {
                warn!("Failed to delete snapshot {}: {}", path.display(), e);
            }
        }
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_snapshot_header_roundtrip() {
        let mut header = SnapshotHeader::new(100, 50, 1_700_000_000, 12345);
        header.checksum = [42u8; 32];

        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();
        assert_eq!(buffer.len(), V2_HEADER_SIZE);

        let mut cursor = &buffer[..];
        let (decoded, replay) = SnapshotHeader::read(&mut cursor).unwrap();

        assert_eq!(decoded.magic, *SNAPSHOT_MAGIC);
        assert_eq!(decoded.version, SNAPSHOT_VERSION);
        assert_eq!(decoded.entity_count, 100);
        assert_eq!(decoded.relation_count, 50);
        assert_eq!(decoded.wal_file_timestamp, 1_700_000_000);
        assert_eq!(decoded.wal_position_in_file, 12345);
        assert_eq!(decoded.checksum, [42u8; 32]);
        assert_eq!(
            replay,
            WalReplayStart::FromPoint {
                wal_file_timestamp: 1_700_000_000,
                wal_position_in_file: 12345,
            }
        );
    }

    #[test]
    fn test_snapshot_header_invalid_magic() {
        let mut buffer = vec![0u8; V2_HEADER_SIZE];
        buffer[0..4].copy_from_slice(b"BAD!");
        let result = SnapshotHeader::read(&mut &buffer[..]);
        assert!(result.is_err());
    }

    /// A hand-crafted v1 header must load without error and yield
    /// `WalReplayStart::FullReplay` (graceful fallback per R2).
    #[test]
    fn test_v1_header_falls_back_to_full_replay() {
        let mut buf = Vec::with_capacity(V1_HEADER_SIZE);
        buf.extend_from_slice(SNAPSHOT_MAGIC);
        buf.extend_from_slice(&1u32.to_be_bytes()); // version 1
        buf.extend_from_slice(&0i64.to_be_bytes()); // created_at
        buf.extend_from_slice(&7u64.to_be_bytes()); // entity_count
        buf.extend_from_slice(&3u64.to_be_bytes()); // relation_count
        buf.extend_from_slice(&9999u64.to_be_bytes()); // legacy wal_position (ignored)
        buf.extend_from_slice(&[0u8; 32]); // checksum
        assert_eq!(buf.len(), V1_HEADER_SIZE);

        let (header, replay) = SnapshotHeader::read(&mut &buf[..]).unwrap();
        assert_eq!(header.version, 1);
        assert_eq!(header.entity_count, 7);
        assert_eq!(header.relation_count, 3);
        assert_eq!(replay, WalReplayStart::FullReplay);
    }

    #[test]
    fn test_find_snapshot_files() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        File::create(data_dir.join("snapshot-1000.snap")).unwrap();
        File::create(data_dir.join("snapshot-2000.snap")).unwrap();
        File::create(data_dir.join("snapshot-3000.snap")).unwrap();
        File::create(data_dir.join("other.txt")).unwrap();

        let snapshots = find_snapshot_files(data_dir).unwrap();
        assert_eq!(snapshots.len(), 3);
        assert!(snapshots[0].ends_with("snapshot-1000.snap"));
        assert!(snapshots[2].ends_with("snapshot-3000.snap"));
    }

    #[test]
    fn test_cleanup_old_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        for i in 1..=5 {
            File::create(data_dir.join(format!("snapshot-{}000.snap", i))).unwrap();
        }

        let deleted = cleanup_old_snapshots(data_dir, 2).unwrap();
        assert_eq!(deleted, 3);

        let remaining = find_snapshot_files(data_dir).unwrap();
        assert_eq!(remaining.len(), 2);
        // Should keep the newest two: snapshot-4000.snap and snapshot-5000.snap
        assert!(remaining[0].ends_with("snapshot-4000.snap"));
        assert!(remaining[1].ends_with("snapshot-5000.snap"));
    }
}
