///! Write-Ahead Log (WAL) implementation
///!
///! Provides append-only logging of all write operations with batched fsync.
///!
///! ## Architecture
///!
///! - Single WAL file grows until rotation threshold (1GB default)
///! - Background thread flushes every 100ms (batched fsync)
///! - CRC32 checksum on every entry for corruption detection
///! - File rotation creates new file with timestamp suffix
///!
///! ## File Naming
///!
///! - Active WAL: `wal-current.log`
///! - Rotated WAL: `wal-{timestamp}.log`

use super::format::{encode_wal_entry, decode_wal_entry, WalEntry, WalHeader, WalOp};
use super::{PersistenceError, Result, WalConfig};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// WAL writer with batched fsync
pub struct WalWriter {
    file: BufWriter<File>,
    path: PathBuf,
    position: u64,
    unflushed_bytes: usize,
    last_fsync: Instant,
    config: WalConfig,
    data_dir: PathBuf,
}

impl WalWriter {
    /// Create a new WAL writer
    pub fn new(data_dir: PathBuf, config: WalConfig) -> Result<Self> {
        // Ensure data directory exists
        fs::create_dir_all(&data_dir).map_err(|e| {
            PersistenceError::DataDirectory(format!("Failed to create directory: {}", e))
        })?;

        let wal_path = data_dir.join("wal-current.log");

        // Open or create WAL file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)?;

        let metadata = file.metadata()?;
        let position = metadata.len();

        let mut writer = BufWriter::with_capacity(64 * 1024, file); // 64KB buffer

        // Write header if new file
        if position == 0 {
            let header = WalHeader::new();
            header.write(&mut writer)?;
            writer.flush()?;
            debug!("Created new WAL file: {}", wal_path.display());
        }

        let position = writer.seek(SeekFrom::End(0))?;

        Ok(Self {
            file: writer,
            path: wal_path,
            position,
            unflushed_bytes: 0,
            last_fsync: Instant::now(),
            config,
            data_dir,
        })
    }

    /// Append an operation to the WAL
    pub fn append(&mut self, op: WalOp) -> Result<u64> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let entry = WalEntry { timestamp, op };
        let encoded = encode_wal_entry(&entry)?;

        let position = self.position;
        self.file.write_all(&encoded)?;
        self.position += encoded.len() as u64;
        self.unflushed_bytes += encoded.len();

        Ok(position)
    }

    /// Flush pending writes to disk (fsync)
    pub fn flush(&mut self) -> Result<()> {
        if self.unflushed_bytes == 0 {
            return Ok(());
        }

        self.file.flush()?;
        self.file.get_ref().sync_data()?;
        self.unflushed_bytes = 0;
        self.last_fsync = Instant::now();

        debug!(
            "WAL fsynced at position {}, {} bytes",
            self.position, self.unflushed_bytes
        );

        Ok(())
    }

    /// Check if flush is needed based on time
    pub fn should_flush(&self) -> bool {
        self.last_fsync.elapsed().as_millis() >= self.config.flush_interval_ms as u128
    }

    /// Check if rotation is needed based on file size
    pub fn should_rotate(&self) -> bool {
        self.position >= self.config.max_file_size
    }

    /// Rotate to a new WAL file
    pub fn rotate(&mut self) -> Result<PathBuf> {
        // Flush and close current file
        self.flush()?;

        // Get timestamp for rotated file
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let rotated_path = self.data_dir.join(format!("wal-{}.log", timestamp));

        // Create new WAL file first (before renaming old one)
        let new_path = self.data_dir.join("wal-current-new.log");
        let new_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&new_path)?;

        let mut new_writer = BufWriter::with_capacity(64 * 1024, new_file);

        // Write header to new file
        let header = WalHeader::new();
        header.write(&mut new_writer)?;
        new_writer.flush()?;

        // Now rename old file
        fs::rename(&self.path, &rotated_path)?;

        // Rename new file to current
        fs::rename(&new_path, &self.path)?;

        info!("Rotated WAL: {} -> {}", self.path.display(), rotated_path.display());

        // Update writer
        self.file = new_writer;
        self.position = 32; // Header size
        self.unflushed_bytes = 0;
        self.last_fsync = Instant::now();

        Ok(rotated_path)
    }

    /// Get current file position
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Get the WAL file path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// WAL reader for replaying entries
pub struct WalReader {
    file: File,
    path: PathBuf,
    position: u64,
    file_size: u64,
}

impl WalReader {
    /// Open a WAL file for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)?;

        // Read and validate header
        let _header = WalHeader::read(&mut file)?;

        let position = file.stream_position()?;
        let file_size = file.metadata()?.len();

        debug!("Opened WAL for reading: {} ({} bytes)", path.display(), file_size);

        Ok(Self {
            file,
            path,
            position,
            file_size,
        })
    }

    /// Read the next entry from the WAL
    pub fn read_entry(&mut self) -> Result<Option<WalEntry>> {
        if self.position >= self.file_size {
            return Ok(None); // EOF
        }

        // Read entry length (4 bytes)
        let mut length_bytes = [0u8; 4];
        match self.file.read_exact(&mut length_bytes) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(None); // Graceful EOF
            }
            Err(e) => return Err(e.into()),
        }

        let length = u32::from_be_bytes(length_bytes) as usize;

        // Validate length is reasonable (< 10MB)
        if length > 10 * 1024 * 1024 {
            return Err(PersistenceError::CorruptedWal {
                pos: self.position,
                reason: format!("Entry length too large: {} bytes", length),
            });
        }

        // Read full entry (including length prefix)
        let mut entry_bytes = vec![0u8; 4 + length];
        entry_bytes[0..4].copy_from_slice(&length_bytes);
        self.file.read_exact(&mut entry_bytes[4..])?;

        let entry_pos = self.position;
        self.position += (4 + length) as u64;

        // Decode and verify checksum
        let entry = decode_wal_entry(&entry_bytes, entry_pos)?;

        Ok(Some(entry))
    }

    /// Get current read position
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Get the WAL file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get file size
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
}

/// Find all WAL files in a directory
pub fn find_wal_files(data_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let data_dir = data_dir.as_ref();

    if !data_dir.exists() {
        return Ok(Vec::new());
    }

    let mut wal_files = Vec::new();

    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name() {
            if let Some(name) = filename.to_str() {
                if name.starts_with("wal-") && name.ends_with(".log") {
                    wal_files.push(path);
                }
            }
        }
    }

    // Sort by filename (timestamp order)
    wal_files.sort();

    Ok(wal_files)
}

/// Delete old WAL files (keep only the most recent N)
pub fn cleanup_old_wal_files(data_dir: impl AsRef<Path>, keep_count: usize) -> Result<usize> {
    let mut wal_files = find_wal_files(data_dir)?;

    // Don't delete wal-current.log
    wal_files.retain(|p| {
        !p.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s == "wal-current.log")
            .unwrap_or(false)
    });

    if wal_files.len() <= keep_count {
        return Ok(0);
    }

    let to_delete = wal_files.len() - keep_count;
    let mut deleted = 0;

    for path in wal_files.iter().take(to_delete) {
        match fs::remove_file(path) {
            Ok(_) => {
                debug!("Deleted old WAL file: {}", path.display());
                deleted += 1;
            }
            Err(e) => {
                warn!("Failed to delete WAL file {}: {}", path.display(), e);
            }
        }
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::KvValue;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_config() -> (TempDir, WalConfig) {
        let temp_dir = TempDir::new().unwrap();
        let config = WalConfig {
            flush_interval_ms: 100,
            max_file_size: 1024, // 1KB for testing rotation
        };
        (temp_dir, config)
    }

    #[test]
    fn test_wal_write_and_read() {
        let (temp_dir, config) = create_test_config();
        let data_dir = temp_dir.path().to_path_buf();

        // Write operations
        let mut writer = WalWriter::new(data_dir.clone(), config).unwrap();

        let ops = vec![
            WalOp::Set {
                key: "key1".to_string(),
                value: KvValue::String("value1".to_string()),
                ttl: None,
            },
            WalOp::Set {
                key: "key2".to_string(),
                value: KvValue::Int(42),
                ttl: Some(Duration::from_secs(60)),
            },
            WalOp::Delete {
                key: "key1".to_string(),
            },
        ];

        for op in &ops {
            writer.append(op.clone()).unwrap();
        }
        writer.flush().unwrap();

        // Read operations
        let wal_path = data_dir.join("wal-current.log");
        let mut reader = WalReader::new(&wal_path).unwrap();

        let mut read_ops = Vec::new();
        while let Some(entry) = reader.read_entry().unwrap() {
            read_ops.push(entry.op);
        }

        assert_eq!(read_ops.len(), ops.len());

        // Verify operations match
        for (original, read) in ops.iter().zip(read_ops.iter()) {
            match (original, read) {
                (
                    WalOp::Set { key: k1, value: v1, ttl: t1 },
                    WalOp::Set { key: k2, value: v2, ttl: t2 },
                ) => {
                    assert_eq!(k1, k2);
                    assert_eq!(v1, v2);
                    assert_eq!(t1, t2);
                }
                (WalOp::Delete { key: k1 }, WalOp::Delete { key: k2 }) => {
                    assert_eq!(k1, k2);
                }
                _ => panic!("Op type mismatch"),
            }
        }
    }

    #[test]
    fn test_wal_rotation() {
        let (temp_dir, mut config) = create_test_config();
        config.max_file_size = 256; // Small size to trigger rotation
        let data_dir = temp_dir.path().to_path_buf();

        let mut writer = WalWriter::new(data_dir.clone(), config).unwrap();

        // Write enough data to trigger rotation
        for i in 0..20 {
            writer
                .append(WalOp::Set {
                    key: format!("key{}", i),
                    value: KvValue::String("x".repeat(50)),
                    ttl: None,
                })
                .unwrap();

            if writer.should_rotate() {
                writer.rotate().unwrap();
            }
        }

        // Should have multiple WAL files
        let wal_files = find_wal_files(&data_dir).unwrap();
        assert!(wal_files.len() > 1, "Expected multiple WAL files after rotation");
    }

    #[test]
    fn test_wal_find_files() {
        let (temp_dir, config) = create_test_config();
        let data_dir = temp_dir.path().to_path_buf();

        // Create some WAL files
        let mut writer = WalWriter::new(data_dir.clone(), config).unwrap();
        writer.append(WalOp::Delete { key: "test".to_string() }).unwrap();
        writer.flush().unwrap();

        let files = find_wal_files(&data_dir).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].to_str().unwrap().contains("wal-current.log"));
    }

    #[test]
    fn test_wal_cleanup() {
        let (temp_dir, _config) = create_test_config();
        let data_dir = temp_dir.path().to_path_buf();

        // Create multiple rotated WAL files
        for i in 0..5 {
            let path = data_dir.join(format!("wal-{}.log", 1000 + i));
            File::create(&path).unwrap();
        }

        let before = find_wal_files(&data_dir).unwrap();
        assert_eq!(before.len(), 5);

        // Keep only 2 files
        let deleted = cleanup_old_wal_files(&data_dir, 2).unwrap();
        assert_eq!(deleted, 3);

        let after = find_wal_files(&data_dir).unwrap();
        assert_eq!(after.len(), 2);
    }

    #[test]
    fn test_wal_corrupted_entry() {
        let (temp_dir, config) = create_test_config();
        let data_dir = temp_dir.path().to_path_buf();

        let mut writer = WalWriter::new(data_dir.clone(), config).unwrap();
        writer.append(WalOp::Set {
            key: "test".to_string(),
            value: KvValue::String("value".to_string()),
            ttl: None,
        }).unwrap();
        writer.flush().unwrap();
        drop(writer);

        // Corrupt the WAL file
        let wal_path = data_dir.join("wal-current.log");
        let mut file = OpenOptions::new().write(true).open(&wal_path).unwrap();
        file.seek(SeekFrom::Start(50)).unwrap();
        file.write_all(&[0xFF, 0xFF, 0xFF, 0xFF]).unwrap();
        drop(file);

        // Try to read - should detect corruption
        let mut reader = WalReader::new(&wal_path).unwrap();
        let result = reader.read_entry();
        assert!(result.is_err());
    }

    #[test]
    fn test_wal_batch_operations() {
        let (temp_dir, config) = create_test_config();
        let data_dir = temp_dir.path().to_path_buf();

        let mut writer = WalWriter::new(data_dir.clone(), config).unwrap();

        // Write MSET
        writer.append(WalOp::MSet {
            pairs: vec![
                ("k1".to_string(), KvValue::Int(1)),
                ("k2".to_string(), KvValue::Int(2)),
            ],
            ttl: None,
        }).unwrap();

        // Write MDEL
        writer.append(WalOp::MDel {
            keys: vec!["k1".to_string(), "k2".to_string()],
        }).unwrap();

        writer.flush().unwrap();

        // Read back
        let mut reader = WalReader::new(&data_dir.join("wal-current.log")).unwrap();

        let entry1 = reader.read_entry().unwrap().unwrap();
        match entry1.op {
            WalOp::MSet { pairs, .. } => assert_eq!(pairs.len(), 2),
            _ => panic!("Expected MSet"),
        }

        let entry2 = reader.read_entry().unwrap().unwrap();
        match entry2.op {
            WalOp::MDel { keys } => assert_eq!(keys.len(), 2),
            _ => panic!("Expected MDel"),
        }
    }

    #[test]
    fn test_wal_flush_timing() {
        let (temp_dir, config) = create_test_config();
        let data_dir = temp_dir.path().to_path_buf();

        let mut writer = WalWriter::new(data_dir, config).unwrap();

        // Initially should not need flush
        assert!(!writer.should_flush());

        // Write an operation
        writer.append(WalOp::Set {
            key: "test".to_string(),
            value: KvValue::String("value".to_string()),
            ttl: None,
        }).unwrap();

        // Still shouldn't need flush immediately
        assert!(!writer.should_flush());

        // Wait for flush interval
        std::thread::sleep(std::time::Duration::from_millis(150));

        // Now should need flush
        assert!(writer.should_flush());

        writer.flush().unwrap();
        assert!(!writer.should_flush());
    }
}
