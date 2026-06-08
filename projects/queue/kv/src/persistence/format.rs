///! Binary format definitions for WAL and Snapshot files
///!
///! ## WAL File Format
///!
///! ```text
///! Header: [Magic:8 | Version:4 | Created:8 | Reserved:12] = 32 bytes
///! Entry:  [Length:4 | Timestamp:8 | OpType:1 | Payload:N | CRC32:4]
///! ```
///!
///! ## Snapshot File Format
///!
///! ```text
///! Header: [Magic:8 | Version:4 | Created:8 | NumShards:4 | TotalEntries:8 | WalPos:8 | SHA256:32]
///! Data:   [Shard0] [Shard1] ... [ShardN]
///! Shard:  [ShardID:4 | EntryCount:4 | Entries using bincode]
///! ```

use crate::types::KvValue;
use crate::persistence::{PersistenceError, Result};
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::time::Duration;

/// WAL file magic number: "KVWAL001"
pub const WAL_MAGIC: &[u8; 8] = b"KVWAL001";

/// WAL format version
pub const WAL_VERSION: u32 = 1;

/// Snapshot file magic number: "KVSNAP01"
pub const SNAPSHOT_MAGIC: &[u8; 8] = b"KVSNAP01";

/// Snapshot format version
pub const SNAPSHOT_VERSION: u32 = 1;

/// WAL operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WalOpType {
    Set = 1,
    Delete = 2,
    Incr = 3,
    Decr = 4,
    MSet = 5,
    MDel = 6,
    SetNx = 7,
    Lock = 8,
    Unlock = 9,
    ExtendLock = 10,
}

impl WalOpType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(WalOpType::Set),
            2 => Some(WalOpType::Delete),
            3 => Some(WalOpType::Incr),
            4 => Some(WalOpType::Decr),
            5 => Some(WalOpType::MSet),
            6 => Some(WalOpType::MDel),
            7 => Some(WalOpType::SetNx),
            8 => Some(WalOpType::Lock),
            9 => Some(WalOpType::Unlock),
            10 => Some(WalOpType::ExtendLock),
            _ => None,
        }
    }
}

/// WAL operation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOp {
    Set {
        key: String,
        value: KvValue,
        ttl: Option<Duration>,
    },
    Delete {
        key: String,
    },
    Incr {
        key: String,
        delta: i64,
    },
    Decr {
        key: String,
        delta: i64,
    },
    MSet {
        pairs: Vec<(String, KvValue)>,
        ttl: Option<Duration>,
    },
    MDel {
        keys: Vec<String>,
    },
    SetNx {
        key: String,
        value: KvValue,
        ttl: Option<Duration>,
    },
    Lock {
        key: String,
        owner: String,
        ttl: Duration,
    },
    Unlock {
        key: String,
        owner: String,
    },
    ExtendLock {
        key: String,
        owner: String,
        ttl: Duration,
    },
}

impl WalOp {
    pub fn op_type(&self) -> WalOpType {
        match self {
            WalOp::Set { .. } => WalOpType::Set,
            WalOp::Delete { .. } => WalOpType::Delete,
            WalOp::Incr { .. } => WalOpType::Incr,
            WalOp::Decr { .. } => WalOpType::Decr,
            WalOp::MSet { .. } => WalOpType::MSet,
            WalOp::MDel { .. } => WalOpType::MDel,
            WalOp::SetNx { .. } => WalOpType::SetNx,
            WalOp::Lock { .. } => WalOpType::Lock,
            WalOp::Unlock { .. } => WalOpType::Unlock,
            WalOp::ExtendLock { .. } => WalOpType::ExtendLock,
        }
    }
}

/// WAL entry with metadata
#[derive(Debug, Clone)]
pub struct WalEntry {
    /// Timestamp in nanoseconds since Unix epoch
    pub timestamp: i64,

    /// The operation
    pub op: WalOp,
}

/// WAL file header (32 bytes)
#[derive(Debug, Clone)]
pub struct WalHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub created_at: i64, // Unix timestamp in seconds
}

impl WalHeader {
    pub fn new() -> Self {
        Self {
            magic: *WAL_MAGIC,
            version: WAL_VERSION,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.version.to_be_bytes())?;
        writer.write_all(&self.created_at.to_be_bytes())?;
        writer.write_all(&[0u8; 12])?; // Reserved
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;

        if &magic != WAL_MAGIC {
            return Err(PersistenceError::InvalidMagic {
                expected: WAL_MAGIC.to_vec(),
                actual: magic.to_vec(),
            });
        }

        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_be_bytes(version_bytes);

        if version != WAL_VERSION {
            return Err(PersistenceError::UnsupportedVersion(version));
        }

        let mut created_bytes = [0u8; 8];
        reader.read_exact(&mut created_bytes)?;
        let created_at = i64::from_be_bytes(created_bytes);

        let mut reserved = [0u8; 12];
        reader.read_exact(&mut reserved)?;

        Ok(Self {
            magic,
            version,
            created_at,
        })
    }
}

/// Snapshot file header (72 bytes)
#[derive(Debug, Clone)]
pub struct SnapshotHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub created_at: i64,
    pub num_shards: u32,
    pub total_entries: u64,
    pub wal_position: u64, // Position in WAL when snapshot was taken
    pub checksum: [u8; 32], // SHA256 of data
}

impl SnapshotHeader {
    pub fn new(num_shards: u32, total_entries: u64, wal_position: u64, checksum: [u8; 32]) -> Self {
        Self {
            magic: *SNAPSHOT_MAGIC,
            version: SNAPSHOT_VERSION,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            num_shards,
            total_entries,
            wal_position,
            checksum,
        }
    }

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

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;

        if &magic != SNAPSHOT_MAGIC {
            return Err(PersistenceError::InvalidMagic {
                expected: SNAPSHOT_MAGIC.to_vec(),
                actual: magic.to_vec(),
            });
        }

        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_be_bytes(version_bytes);

        if version != SNAPSHOT_VERSION {
            return Err(PersistenceError::UnsupportedVersion(version));
        }

        let mut created_bytes = [0u8; 8];
        reader.read_exact(&mut created_bytes)?;
        let created_at = i64::from_be_bytes(created_bytes);

        let mut num_shards_bytes = [0u8; 4];
        reader.read_exact(&mut num_shards_bytes)?;
        let num_shards = u32::from_be_bytes(num_shards_bytes);

        let mut total_entries_bytes = [0u8; 8];
        reader.read_exact(&mut total_entries_bytes)?;
        let total_entries = u64::from_be_bytes(total_entries_bytes);

        let mut wal_position_bytes = [0u8; 8];
        reader.read_exact(&mut wal_position_bytes)?;
        let wal_position = u64::from_be_bytes(wal_position_bytes);

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

/// Calculate CRC32 checksum
pub fn calculate_crc32(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

/// Calculate SHA256 checksum
pub fn calculate_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Encode a WAL entry with checksum
pub fn encode_wal_entry(entry: &WalEntry) -> Result<Vec<u8>> {
    // Serialize the operation
    let op_bytes = bincode::serialize(&entry.op)?;

    // Calculate total length (timestamp + op_type + payload + crc32)
    let total_length = 8 + 1 + op_bytes.len() + 4;

    let mut buffer = Vec::with_capacity(4 + total_length);

    // Write length (4 bytes)
    buffer.extend_from_slice(&(total_length as u32).to_be_bytes());

    // Write timestamp (8 bytes)
    buffer.extend_from_slice(&entry.timestamp.to_be_bytes());

    // Write op type (1 byte)
    buffer.push(entry.op.op_type() as u8);

    // Write payload
    buffer.extend_from_slice(&op_bytes);

    // Calculate and write CRC32 checksum
    let checksum = calculate_crc32(&buffer[4..]); // Checksum of everything except length
    buffer.extend_from_slice(&checksum.to_be_bytes());

    Ok(buffer)
}

/// Decode a WAL entry and verify checksum
pub fn decode_wal_entry(data: &[u8], position: u64) -> Result<WalEntry> {
    if data.len() < 4 {
        return Err(PersistenceError::CorruptedWal {
            pos: position,
            reason: format!("Too short: {} bytes", data.len()),
        });
    }

    // Read length
    let length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if data.len() < 4 + length {
        return Err(PersistenceError::CorruptedWal {
            pos: position,
            reason: format!("Incomplete entry: expected {} bytes, got {}", 4 + length, data.len()),
        });
    }

    // Verify checksum
    let expected_checksum = u32::from_be_bytes([
        data[4 + length - 4],
        data[4 + length - 3],
        data[4 + length - 2],
        data[4 + length - 1],
    ]);
    let actual_checksum = calculate_crc32(&data[4..4 + length - 4]);

    if expected_checksum != actual_checksum {
        return Err(PersistenceError::ChecksumMismatch {
            pos: position,
            expected: expected_checksum,
            actual: actual_checksum,
        });
    }

    // Read timestamp
    let timestamp = i64::from_be_bytes([
        data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
    ]);

    // Read op type
    let op_type_byte = data[12];
    let _op_type = WalOpType::from_u8(op_type_byte).ok_or_else(|| {
        PersistenceError::CorruptedWal {
            pos: position,
            reason: format!("Invalid op type: {}", op_type_byte),
        }
    })?;

    // Deserialize operation
    let op_bytes = &data[13..4 + length - 4];
    let op: WalOp = bincode::deserialize(op_bytes).map_err(|e| {
        PersistenceError::CorruptedWal {
            pos: position,
            reason: format!("Deserialization failed: {}", e),
        }
    })?;

    Ok(WalEntry { timestamp, op })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_header_roundtrip() {
        let header = WalHeader::new();
        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();

        let decoded = WalHeader::read(&mut buffer.as_slice()).unwrap();
        assert_eq!(header.magic, decoded.magic);
        assert_eq!(header.version, decoded.version);
    }

    #[test]
    fn test_wal_entry_encoding() {
        let entry = WalEntry {
            timestamp: 1234567890,
            op: WalOp::Set {
                key: "test_key".to_string(),
                value: KvValue::String("test_value".to_string()),
                ttl: None,
            },
        };

        let encoded = encode_wal_entry(&entry).unwrap();
        let decoded = decode_wal_entry(&encoded, 0).unwrap();

        assert_eq!(entry.timestamp, decoded.timestamp);
        match (&entry.op, &decoded.op) {
            (
                WalOp::Set { key: k1, value: v1, ttl: t1 },
                WalOp::Set { key: k2, value: v2, ttl: t2 },
            ) => {
                assert_eq!(k1, k2);
                assert_eq!(v1, v2);
                assert_eq!(t1, t2);
            }
            _ => panic!("Op type mismatch"),
        }
    }

    #[test]
    fn test_checksum_validation() {
        let entry = WalEntry {
            timestamp: 1234567890,
            op: WalOp::Delete {
                key: "test_key".to_string(),
            },
        };

        let mut encoded = encode_wal_entry(&entry).unwrap();

        // Corrupt a byte
        encoded[10] ^= 0xFF;

        let result = decode_wal_entry(&encoded, 0);
        assert!(matches!(result, Err(PersistenceError::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_crc32_calculation() {
        let data = b"Hello, World!";
        let checksum1 = calculate_crc32(data);
        let checksum2 = calculate_crc32(data);
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_sha256_calculation() {
        let data = b"Hello, World!";
        let hash1 = calculate_sha256(data);
        let hash2 = calculate_sha256(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }
}
