///! Snapshot creation and loading
///!
///! Provides periodic full engine state snapshots for faster recovery.

use super::{PersistenceError, Result, SnapshotConfig};
use crate::engine::{Entry, KvEngine};
use crate::types::KvValue;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Snapshot file magic: "KVSN0001"
pub const SNAPSHOT_MAGIC: &[u8; 8] = b"KVSN0001";
pub const SNAPSHOT_VERSION: u32 = 1;

/// Snapshot file header (64 bytes)
#[derive(Debug, Clone)]
pub struct SnapshotHeader {
    /// Magic number for validation
    pub magic: [u8; 8],
    /// Format version
    pub version: u32,
    /// When snapshot was created (nanoseconds since UNIX_EPOCH)
    pub created_at: i64,
    /// Number of shards in snapshot
    pub num_shards: u32,
    /// Total number of entries across all shards
    pub total_entries: u64,
    /// WAL position at time of snapshot (for recovery)
    pub wal_position: u64,
    /// SHA256 checksum of snapshot data
    pub checksum: [u8; 32],
}

impl SnapshotHeader {
    /// Create a new snapshot header
    pub fn new(num_shards: u32, total_entries: u64, wal_position: u64) -> Self {
        Self {
            magic: *SNAPSHOT_MAGIC,
            version: SNAPSHOT_VERSION,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64,
            num_shards,
            total_entries,
            wal_position,
            checksum: [0u8; 32], // Filled in after writing data
        }
    }

    /// Write header to file (64 bytes)
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.version.to_be_bytes())?;
        writer.write_all(&self.created_at.to_be_bytes())?;
        writer.write_all(&self.num_shards.to_be_bytes())?;
        writer.write_all(&self.total_entries.to_be_bytes())?;
        writer.write_all(&self.wal_position.to_be_bytes())?;
        writer.write_all(&self.checksum)?;
        Ok(())
    }

    /// Read header from file
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;

        if magic != *SNAPSHOT_MAGIC {
            return Err(PersistenceError::CorruptedSnapshot(format!(
                "Invalid snapshot magic: expected {:?}, got {:?}",
                SNAPSHOT_MAGIC, magic
            )));
        }

        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let version = u32::from_be_bytes(buf);

        if version != SNAPSHOT_VERSION {
            return Err(PersistenceError::UnsupportedVersion(version));
        }

        let mut buf8 = [0u8; 8];

        reader.read_exact(&mut buf8)?;
        let created_at = i64::from_be_bytes(buf8);

        reader.read_exact(&mut buf)?;
        let num_shards = u32::from_be_bytes(buf);

        reader.read_exact(&mut buf8)?;
        let total_entries = u64::from_be_bytes(buf8);

        reader.read_exact(&mut buf8)?;
        let wal_position = u64::from_be_bytes(buf8);

        let mut checksum = [0u8; 32];
        reader.read_exact(&mut checksum)?;

        Ok(Self {
            magic,
            version,
            created_at,
            num_shards,
            total_entries,
            wal_position,
            checksum,
        })
    }
}

/// Serializable entry with absolute timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEntry {
    pub value: KvValue,
    /// Nanoseconds since UNIX_EPOCH when entry was created
    pub created_at_nanos: i64,
    /// Nanoseconds since UNIX_EPOCH when entry expires (if any)
    pub expires_at_nanos: Option<i64>,
    pub version: u64,
}

impl SerializableEntry {
    /// Convert from runtime Entry to serializable format
    pub fn from_entry(entry: &Entry, base_instant: Instant) -> Self {
        let created_offset = entry.created_at.duration_since(base_instant);
        let created_at_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .checked_sub(created_offset)
            .unwrap_or_else(|| Duration::from_secs(0))
            .as_nanos() as i64;

        let expires_at_nanos = entry.expires_at.map(|exp| {
            let expires_offset = exp.duration_since(base_instant);
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .checked_sub(expires_offset)
                .unwrap_or_else(|| Duration::from_secs(0))
                .as_nanos() as i64
        });

        Self {
            value: entry.value.clone(),
            created_at_nanos,
            expires_at_nanos,
            version: entry.version,
        }
    }

    /// Convert to runtime Entry
    pub fn to_entry(&self) -> Entry {
        // Convert absolute timestamps back to Instant
        let now = Instant::now();
        let now_absolute = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let created_offset = Duration::from_nanos((now_absolute - self.created_at_nanos).max(0) as u64);
        let created_at = now.checked_sub(created_offset).unwrap_or(now);

        let expires_at = self.expires_at_nanos.map(|exp_nanos| {
            let expires_offset = Duration::from_nanos((now_absolute - exp_nanos).max(0) as u64);
            now.checked_sub(expires_offset).unwrap_or(now)
        });

        Entry {
            value: self.value.clone(),
            created_at,
            expires_at,
            version: self.version,
        }
    }
}

/// Serializable shard data
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableShard {
    pub shard_id: u32,
    pub entries: HashMap<String, SerializableEntry>,
}

/// Snapshot writer for creating snapshots
pub struct SnapshotWriter {
    config: SnapshotConfig,
}

impl SnapshotWriter {
    /// Create a new snapshot writer
    pub fn new(config: SnapshotConfig) -> Self {
        Self { config }
    }

    /// Create a snapshot of the engine
    pub fn create_snapshot(
        &self,
        engine: &KvEngine,
        data_dir: impl AsRef<Path>,
        wal_position: u64,
    ) -> Result<PathBuf> {
        let data_dir = data_dir.as_ref();
        fs::create_dir_all(data_dir)?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let temp_path = data_dir.join(format!("snapshot-{}.tmp", timestamp));
        let final_path = data_dir.join(format!("snapshot-{}.snap", timestamp));

        info!(
            "Creating snapshot at {} (WAL position: {})",
            final_path.display(),
            wal_position
        );

        // Write to temporary file
        let mut hasher = Sha256::new();
        let mut data_buffer = Vec::new();

        // Serialize all shards
        let base_instant = Instant::now();
        let num_shards = engine.num_shards();
        let mut total_entries = 0u64;

        for shard_id in 0..num_shards {
            let shard_data = self.export_shard(engine, shard_id, base_instant)?;
            total_entries += shard_data.entries.len() as u64;

            let shard_bytes = bincode::serialize(&shard_data)?;
            data_buffer.extend_from_slice(&shard_bytes);
        }

        // Calculate checksum
        hasher.update(&data_buffer);
        let checksum: [u8; 32] = hasher.finalize().into();

        // Write header + data atomically
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)?;
        let mut writer = BufWriter::new(file);

        let mut header = SnapshotHeader::new(num_shards as u32, total_entries, wal_position);
        header.checksum = checksum;
        header.write(&mut writer)?;

        writer.write_all(&data_buffer)?;
        writer.flush()?;
        writer.get_ref().sync_all()?; // fsync

        // Atomic rename
        fs::rename(&temp_path, &final_path)?;

        info!(
            "Snapshot created: {} entries across {} shards",
            total_entries, num_shards
        );

        // Cleanup old snapshots
        self.cleanup_old_snapshots(data_dir)?;

        Ok(final_path)
    }

    /// Export a single shard's data
    fn export_shard(
        &self,
        engine: &KvEngine,
        shard_id: usize,
        base_instant: Instant,
    ) -> Result<SerializableShard> {
        let entries = engine.export_shard(shard_id).ok_or_else(|| {
            PersistenceError::DataDirectory(format!("Invalid shard_id: {}", shard_id))
        })?;

        // Convert entries to serializable format
        let serializable_entries = entries
            .into_iter()
            .map(|(key, entry)| {
                let serializable_entry = SerializableEntry::from_entry(&entry, base_instant);
                (key, serializable_entry)
            })
            .collect();

        Ok(SerializableShard {
            shard_id: shard_id as u32,
            entries: serializable_entries,
        })
    }

    /// Cleanup old snapshots, keeping only the N most recent
    fn cleanup_old_snapshots(&self, data_dir: impl AsRef<Path>) -> Result<()> {
        let mut snapshots = find_snapshot_files(data_dir.as_ref())?;

        if snapshots.len() <= self.config.keep_count {
            return Ok(());
        }

        // Sort by timestamp (newest first)
        snapshots.sort_by(|a, b| b.cmp(a));

        let to_delete = &snapshots[self.config.keep_count..];
        for path in to_delete {
            debug!("Deleting old snapshot: {}", path.display());
            fs::remove_file(path)?;
        }

        Ok(())
    }
}

/// Snapshot loader for restoring from snapshots
pub struct SnapshotLoader;

impl SnapshotLoader {
    /// Load the latest snapshot from data directory
    pub fn load_latest(data_dir: impl AsRef<Path>) -> Result<Option<(SnapshotData, u64)>> {
        let snapshots = find_snapshot_files(data_dir.as_ref())?;

        if snapshots.is_empty() {
            return Ok(None);
        }

        // Latest snapshot is the last one (sorted by timestamp)
        let latest = snapshots.last().unwrap();
        info!("Loading snapshot: {}", latest.display());

        let file = File::open(latest)?;
        let mut reader = BufReader::new(file);

        // Read and validate header
        let header = SnapshotHeader::read(&mut reader)?;

        // Read all data
        let mut data_buffer = Vec::new();
        reader.read_to_end(&mut data_buffer)?;

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(&data_buffer);
        let computed_checksum: [u8; 32] = hasher.finalize().into();

        if computed_checksum != header.checksum {
            return Err(PersistenceError::ChecksumMismatch {
                pos: 0,
                expected: u32::from_be_bytes(header.checksum[0..4].try_into().unwrap()),
                actual: u32::from_be_bytes(computed_checksum[0..4].try_into().unwrap()),
            });
        }

        // Deserialize shards
        let mut shards = Vec::new();
        let mut offset = 0;

        for _ in 0..header.num_shards {
            let shard: SerializableShard = bincode::deserialize(&data_buffer[offset..])?;
            offset += bincode::serialized_size(&shard)? as usize;
            shards.push(shard);
        }

        info!(
            "Loaded snapshot: {} entries across {} shards",
            header.total_entries, header.num_shards
        );

        Ok(Some((
            SnapshotData {
                shards,
                total_entries: header.total_entries,
            },
            header.wal_position,
        )))
    }
}

/// Snapshot data loaded from file
#[derive(Debug)]
pub struct SnapshotData {
    pub shards: Vec<SerializableShard>,
    pub total_entries: u64,
}

/// Find all snapshot files in directory, sorted by timestamp (oldest first)
pub fn find_snapshot_files(data_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let data_dir = data_dir.as_ref();

    if !data_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();

    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("snapshot-") && name.ends_with(".snap") {
                    snapshots.push(path);
                }
            }
        }
    }

    // Sort by filename (which contains timestamp)
    snapshots.sort();

    Ok(snapshots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_snapshot_header_roundtrip() {
        let header = SnapshotHeader::new(256, 1000, 12345);

        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();

        let mut cursor = &buffer[..];
        let decoded = SnapshotHeader::read(&mut cursor).unwrap();

        assert_eq!(decoded.magic, *SNAPSHOT_MAGIC);
        assert_eq!(decoded.version, SNAPSHOT_VERSION);
        assert_eq!(decoded.num_shards, 256);
        assert_eq!(decoded.total_entries, 1000);
        assert_eq!(decoded.wal_position, 12345);
    }

    #[test]
    fn test_serializable_entry_roundtrip() {
        use crate::types::KvValue;

        let base = Instant::now();
        let original = Entry {
            value: KvValue::String("test".to_string()),
            created_at: base,
            expires_at: Some(base + Duration::from_secs(60)),
            version: 42,
        };

        let serializable = SerializableEntry::from_entry(&original, base);
        let restored = serializable.to_entry();

        // Values should match
        assert_eq!(restored.value, original.value);
        assert_eq!(restored.version, original.version);

        // Timestamps may have slight differences due to conversion, but should be close
        assert!(restored.created_at <= Instant::now());
        assert!(restored.expires_at.is_some());
    }

    #[test]
    fn test_find_snapshot_files() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create some snapshot files
        File::create(data_dir.join("snapshot-1000.snap")).unwrap();
        File::create(data_dir.join("snapshot-2000.snap")).unwrap();
        File::create(data_dir.join("snapshot-3000.snap")).unwrap();
        File::create(data_dir.join("other-file.txt")).unwrap();

        let snapshots = find_snapshot_files(data_dir).unwrap();

        assert_eq!(snapshots.len(), 3);
        assert!(snapshots[0].ends_with("snapshot-1000.snap"));
        assert!(snapshots[1].ends_with("snapshot-2000.snap"));
        assert!(snapshots[2].ends_with("snapshot-3000.snap"));
    }

    #[test]
    fn test_snapshot_header_invalid_magic() {
        let mut buffer = vec![0u8; 64];
        buffer[0..8].copy_from_slice(b"INVALID!");

        let result = SnapshotHeader::read(&mut &buffer[..]);
        assert!(result.is_err());
    }
}
