// CODEGEN-BEGIN
//! Write-ahead log abstraction — the data-plane backbone.
//!
//! lumen's write path is "turn the database inside out": a write is published
//! to an ordered log and then folded into each serving node's materialized
//! index. The log may be in-process (`MemWal`) or Lumen-owned
//! primary/replica replication.
//!
//! This mirrors Redis's AOF (the op log) + replication stream, with the
//! "master" role dissolved into "the log owner":
//!
//! - **AOF**  → this log (a stream of [`WalRecord`]).
//! - **RDB**  → periodic snapshots to object storage (see `rdb`), tagged
//!   with the log sequence they correspond to, so a fresh node loads a
//!   baseline then tails the log from there.
//!
//! The local WAL backend implements [`WalLog`]:
//!
//! - [`MemWal`] — in-process, in-memory. Unit tests + the simplest
//!   single-node dev runs. Publish applies synchronously from the
//!   caller's perspective (the subscriber sees it immediately).
//!
//! The record payload reuses [`crate::log_entry::RaftLogEntry`] — it
//! already enumerates every mutation 1:1 with an `Engine` method and is
//! the exact shape a replication record needs.

use std::io::Write;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::log_entry::RaftLogEntry;
use crate::storage::Engine;
use crate::types::{
    validate_batch_unindex_docs_request, BatchUnindexDocsRequest, FieldValue, IndexItem,
    IndexRequest, MAX_BATCH_UNINDEX_DOCS_SIZE,
};
use crate::wal_source_stage::{
    MappedFastIndexPayload, MappedGenericCborPayload, StagedWalRecord, WalSourceStager,
};

pub(crate) mod bounded_generic;
pub(crate) mod fast_index_scanner;
pub(crate) mod borrowed_replace_scanner;
pub(crate) mod borrowed_replace_spool;

/// Legacy on-the-wire record format version.  Existing writable operations
/// keep emitting this byte so a 0.4.30 reader sees their exact old wire form.
pub const WAL_FORMAT_VERSION: u8 = 1;
/// #3992 control-record version.  A pre-0.4.31 reader rejects this byte before
/// it attempts to decode the new command tag, making the downgrade boundary
/// explicit rather than reporting an opaque unknown-enum error.
pub const WAL_CONTROL_FORMAT_VERSION: u8 = 2;
const WAL_FAST_MAGIC: &[u8; 4] = b"LWAL";
/// Legacy fast-Index tag: `(external_id, field, value)` triples only, no
/// `IndexItem.version`. Every fast record written before #3952 used this tag.
/// The decode branch for it is frozen byte-for-byte so those records keep
/// replaying — `version` is reconstructed as `None`, matching what the writer
/// actually put on the wire at the time.
const WAL_FAST_INDEX: u8 = 1;
/// #3952: fast-Index tag carrying each item's optional external LWW
/// `version` (#184) on the wire.
///
/// Emitted ONLY when at least one item in the record actually carries a
/// `version`. A record with nothing to say beyond the legacy layout is still
/// written with [`WAL_FAST_INDEX`], byte-for-byte as before #3952, and that
/// asymmetry is deliberate: [`WAL_FORMAT_VERSION`] is still 1, so a pre-#3952
/// binary's version check waves every record through and then fails on the
/// tag. Emitting tag 2 unconditionally therefore made a single appended
/// `Index` record — even one from a deployment that never sets `version` —
/// enough to break replay on a rollback. Choosing the tag by content keeps the
/// downgrade path open for exactly the records an older binary could have
/// read correctly, and closes it, loudly, for the ones it could not.
const WAL_FAST_INDEX_VERSIONED: u8 = 2;
/// #3992: `POST .../docs:truncate`.  This has no request body beyond its
/// collection id, so a small control record is both clearer and stricter than
/// a generic CBOR enum payload.  It is always paired with WAL v2.
const WAL_FAST_TRUNCATE_DOCS: u8 = 3;
/// #3994: `POST .../docs:unindex`.  This stays a separate v2 control tag so
/// a reader never mistakes an identifier batch for a collection-wide swap.
const WAL_FAST_UNINDEX_DOCS: u8 = 4;
const WAL_VALUE_STRING: u8 = 1;
const WAL_VALUE_NUMBER: u8 = 2;
const WAL_VALUE_VECTOR: u8 = 3;
const WAL_VALUE_STRING_LIST: u8 = 4;

/// One durable, ordered mutation in the log. The sequence number is
/// **not** part of the record — it is assigned by the log on publish
/// and delivered alongside the record on subscribe (`MemWal` uses the append
/// index; external-log or primary/replica backends own sequence assignment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalRecord {
    pub version: u8,
    pub entry: RaftLogEntry,
}

impl WalRecord {
    pub(crate) fn is_fast_index_wire(&self) -> bool {
        self.version == WAL_FORMAT_VERSION && matches!(&self.entry, RaftLogEntry::Index { .. })
    }

    /// Stream the frozen fast Index wire format. This writes no aggregate
    /// payload buffer and rejects every length that cannot fit the public u32
    /// frame fields before it writes that field.
    pub(crate) fn write_fast_index_wire(&self, out: &mut dyn Write) -> std::io::Result<()> {
        let RaftLogEntry::Index { collection_id, req } = &self.entry else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "not a fast Index record",
            ));
        };
        if self.version != WAL_FORMAT_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "unsupported fast Index version",
            ));
        }
        let string = |out: &mut dyn Write, value: &str| -> std::io::Result<()> {
            let len = u32::try_from(value.len()).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "fast WAL string exceeds u32",
                )
            })?;
            out.write_all(&len.to_le_bytes())?;
            out.write_all(value.as_bytes())
        };
        let versioned = req.items.iter().any(|item| item.version.is_some());
        out.write_all(WAL_FAST_MAGIC)?;
        out.write_all(&[
            self.version,
            if versioned {
                WAL_FAST_INDEX_VERSIONED
            } else {
                WAL_FAST_INDEX
            },
        ])?;
        string(out, collection_id)?;
        match &req.request_id {
            Some(value) => {
                out.write_all(&[1])?;
                string(out, value)?;
            }
            None => out.write_all(&[0])?,
        }
        let items = u32::try_from(req.items.len()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "fast WAL item count exceeds u32",
            )
        })?;
        out.write_all(&items.to_le_bytes())?;
        for item in &req.items {
            string(out, &item.external_id)?;
            string(out, &item.field)?;
            if versioned {
                match item.version {
                    Some(value) => {
                        out.write_all(&[1])?;
                        out.write_all(&value.to_le_bytes())?;
                    }
                    None => out.write_all(&[0])?,
                }
            }
            match &item.value {
                FieldValue::String(value) => {
                    out.write_all(&[WAL_VALUE_STRING])?;
                    string(out, value)?;
                }
                FieldValue::Number(value) => {
                    out.write_all(&[WAL_VALUE_NUMBER])?;
                    out.write_all(&value.to_le_bytes())?;
                }
                FieldValue::Vector(values) => {
                    out.write_all(&[WAL_VALUE_VECTOR])?;
                    let len = u32::try_from(values.len()).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "fast WAL vector count exceeds u32",
                        )
                    })?;
                    out.write_all(&len.to_le_bytes())?;
                    for value in values {
                        out.write_all(&value.to_le_bytes())?;
                    }
                }
                FieldValue::StringList(values) => {
                    out.write_all(&[WAL_VALUE_STRING_LIST])?;
                    let len = u32::try_from(values.len()).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "fast WAL set count exceeds u32",
                        )
                    })?;
                    out.write_all(&len.to_le_bytes())?;
                    for value in values {
                        string(out, value)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn new(entry: RaftLogEntry) -> Self {
        Self {
            version: match entry {
                RaftLogEntry::TruncateDocs { .. } | RaftLogEntry::UnindexDocs { .. } => {
                    WAL_CONTROL_FORMAT_VERSION
                }
                _ => WAL_FORMAT_VERSION,
            },
            entry,
        }
    }

    #[inline]
    pub fn encode(&self) -> Result<Vec<u8>> {
        match &self.entry {
            RaftLogEntry::TruncateDocs { .. } => {
                return self.encode_fast_truncate_docs().ok_or_else(|| {
                    anyhow!(
                        "TruncateDocs must use WAL v{} fast control encoding",
                        WAL_CONTROL_FORMAT_VERSION
                    )
                });
            }
            RaftLogEntry::UnindexDocs { .. } => {
                return self.encode_fast_unindex_docs().ok_or_else(|| {
                    anyhow!(
                        "UnindexDocs must use a valid WAL v{} fast control encoding",
                        WAL_CONTROL_FORMAT_VERSION
                    )
                });
            }
            _ => {}
        }
        anyhow::ensure!(
            self.version != WAL_CONTROL_FORMAT_VERSION,
            "WAL v{} is reserved for fast control records",
            WAL_CONTROL_FORMAT_VERSION
        );
        if let Some(bytes) = self.encode_fast_index() {
            return Ok(bytes);
        }
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(self, &mut bytes)
            .map_err(|e| anyhow!("cbor encode WAL record: {e}"))?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.starts_with(WAL_FAST_MAGIC) {
            return decode_fast_record(bytes);
        }
        let rec = bounded_generic::decode(bytes)?;
        anyhow::ensure!(
            rec.version == WAL_FORMAT_VERSION,
            "unsupported generic WAL record version {} (expected {})",
            rec.version,
            WAL_FORMAT_VERSION
        );
        anyhow::ensure!(
            !matches!(
                &rec.entry,
                RaftLogEntry::TruncateDocs { .. } | RaftLogEntry::UnindexDocs { .. }
            ),
            "control commands must use WAL v{} fast control encoding",
            WAL_CONTROL_FORMAT_VERSION
        );
        Ok(rec)
    }

    fn encode_fast_index(&self) -> Option<Vec<u8>> {
        if self.version != WAL_FORMAT_VERSION {
            return None;
        }
        let RaftLogEntry::Index { collection_id, req } = &self.entry else {
            return None;
        };
        // Choose the tag by CONTENT, not unconditionally: a record none of
        // whose items carries a `version` is expressible on the legacy wire,
        // and writing it there is what keeps a rollback to a pre-#3952 binary
        // readable (see `WAL_FAST_INDEX_VERSIONED`).
        let versioned = req.items.iter().any(|item| item.version.is_some());
        let mut bytes = Vec::with_capacity(estimate_fast_index_len(collection_id, req, versioned));
        bytes.extend_from_slice(WAL_FAST_MAGIC);
        bytes.push(self.version);
        bytes.push(if versioned {
            WAL_FAST_INDEX_VERSIONED
        } else {
            WAL_FAST_INDEX
        });
        put_str(&mut bytes, collection_id)?;
        match &req.request_id {
            Some(request_id) => {
                bytes.push(1);
                put_str(&mut bytes, request_id)?;
            }
            None => bytes.push(0),
        }
        put_u32(&mut bytes, req.items.len())?;
        for item in &req.items {
            put_str(&mut bytes, &item.external_id)?;
            put_str(&mut bytes, &item.field)?;
            // #3952: carry the external LWW version (#184) on the wire so
            // replay reconstructs the same `cell_versions` ceiling the live
            // apply path enforced. Present only under the versioned tag — the
            // legacy layout has no per-item version byte at all, and writing
            // one under tag 1 would desynchronize every reader, old and new.
            if versioned {
                match item.version {
                    Some(v) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&v.to_le_bytes());
                    }
                    None => bytes.push(0),
                }
            }
            match &item.value {
                FieldValue::String(s) => {
                    bytes.push(WAL_VALUE_STRING);
                    put_str(&mut bytes, s)?;
                }
                FieldValue::Number(n) => {
                    bytes.push(WAL_VALUE_NUMBER);
                    bytes.extend_from_slice(&n.to_le_bytes());
                }
                FieldValue::Vector(v) => {
                    bytes.push(WAL_VALUE_VECTOR);
                    put_u32(&mut bytes, v.len())?;
                    for x in v {
                        bytes.extend_from_slice(&x.to_le_bytes());
                    }
                }
                FieldValue::StringList(values) => {
                    bytes.push(WAL_VALUE_STRING_LIST);
                    put_u32(&mut bytes, values.len())?;
                    for value in values {
                        put_str(&mut bytes, value)?;
                    }
                }
            }
        }
        Some(bytes)
    }

    fn encode_fast_truncate_docs(&self) -> Option<Vec<u8>> {
        let RaftLogEntry::TruncateDocs { collection_id } = &self.entry else {
            return None;
        };
        if self.version != WAL_CONTROL_FORMAT_VERSION {
            return None;
        }
        let mut bytes = Vec::with_capacity(WAL_FAST_MAGIC.len() + 2 + collection_id.len() + 4);
        bytes.extend_from_slice(WAL_FAST_MAGIC);
        bytes.push(WAL_CONTROL_FORMAT_VERSION);
        bytes.push(WAL_FAST_TRUNCATE_DOCS);
        put_str(&mut bytes, collection_id)?;
        Some(bytes)
    }

    fn encode_fast_unindex_docs(&self) -> Option<Vec<u8>> {
        let RaftLogEntry::UnindexDocs { collection_id, req } = &self.entry else {
            return None;
        };
        if self.version != WAL_CONTROL_FORMAT_VERSION
            || validate_batch_unindex_docs_request(req).is_err()
        {
            return None;
        }
        let ids_len: usize = req.external_ids.iter().map(|id| 4 + id.len()).sum();
        let mut bytes =
            Vec::with_capacity(WAL_FAST_MAGIC.len() + 2 + 4 + collection_id.len() + 4 + ids_len);
        bytes.extend_from_slice(WAL_FAST_MAGIC);
        bytes.push(WAL_CONTROL_FORMAT_VERSION);
        bytes.push(WAL_FAST_UNINDEX_DOCS);
        put_str(&mut bytes, collection_id)?;
        put_u32(&mut bytes, req.external_ids.len())?;
        for external_id in &req.external_ids {
            put_str(&mut bytes, external_id)?;
        }
        Some(bytes)
    }
}

/// `versioned` must be the same predicate `encode_fast_index` used to pick the
/// tag — under the legacy tag there is no per-item version byte to reserve, and
/// over-reserving one byte per item is a silent per-record allocation tax on
/// every write in a deployment that never sets `version`.
fn estimate_fast_index_len(collection_id: &str, req: &IndexRequest, versioned: bool) -> usize {
    let mut len = WAL_FAST_MAGIC.len() + 2 + 4 + collection_id.len() + 1 + 4;
    if let Some(request_id) = &req.request_id {
        len += 4 + request_id.len();
    }
    for item in &req.items {
        len += 4 + item.external_id.len() + 4 + item.field.len() + 1;
        if versioned {
            len += if item.version.is_some() { 9 } else { 1 };
        }
        match &item.value {
            FieldValue::String(s) => len += 4 + s.len(),
            FieldValue::Number(_) => len += 8,
            FieldValue::Vector(v) => len += 4 + v.len() * 4,
            FieldValue::StringList(values) => {
                len += 4;
                for value in values {
                    len += 4 + value.len();
                }
            }
        }
    }
    len
}

fn put_u32(bytes: &mut Vec<u8>, n: usize) -> Option<()> {
    let n = u32::try_from(n).ok()?;
    bytes.extend_from_slice(&n.to_le_bytes());
    Some(())
}

fn put_str(bytes: &mut Vec<u8>, s: &str) -> Option<()> {
    put_u32(bytes, s.len())?;
    bytes.extend_from_slice(s.as_bytes());
    Some(())
}

fn decode_fast_record(bytes: &[u8]) -> Result<WalRecord> {
    let mut cur = FastCursor::new(bytes);
    cur.expect_magic(WAL_FAST_MAGIC)?;
    let version = cur.read_u8()?;
    anyhow::ensure!(
        matches!(version, WAL_FORMAT_VERSION | WAL_CONTROL_FORMAT_VERSION),
        "unsupported WAL fast record version {} (expected {} or {})",
        version,
        WAL_FORMAT_VERSION,
        WAL_CONTROL_FORMAT_VERSION
    );
    let tag = cur.read_u8()?;
    if version == WAL_CONTROL_FORMAT_VERSION {
        return match tag {
            WAL_FAST_TRUNCATE_DOCS => {
                let collection_id = cur.read_string()?;
                cur.expect_eof()?;
                Ok(WalRecord {
                    version,
                    entry: RaftLogEntry::TruncateDocs { collection_id },
                })
            }
            WAL_FAST_UNINDEX_DOCS => {
                let collection_id = cur.read_string()?;
                let item_count = cur.read_u32()? as usize;
                // Validate the untrusted count before `Vec::with_capacity`.
                // A corrupt WAL frame must not force a replica to reserve an
                // attacker-sized allocation merely to reject it later.
                anyhow::ensure!(
                    (1..=MAX_BATCH_UNINDEX_DOCS_SIZE).contains(&item_count),
                    "invalid WAL UnindexDocs item count {item_count} (must be 1..={MAX_BATCH_UNINDEX_DOCS_SIZE})"
                );
                let mut external_ids = Vec::with_capacity(item_count);
                let mut seen = std::collections::BTreeSet::new();
                for _ in 0..item_count {
                    let external_id = cur.read_string()?;
                    anyhow::ensure!(
                        seen.insert(external_id.clone()),
                        "duplicate external_id in WAL UnindexDocs record"
                    );
                    external_ids.push(external_id);
                }
                cur.expect_eof()?;
                let req = BatchUnindexDocsRequest { external_ids };
                validate_batch_unindex_docs_request(&req)?;
                Ok(WalRecord {
                    version,
                    entry: RaftLogEntry::UnindexDocs { collection_id, req },
                })
            }
            _ => Err(anyhow!("unsupported WAL v2 control tag {tag}")),
        };
    }
    anyhow::ensure!(
        tag == WAL_FAST_INDEX || tag == WAL_FAST_INDEX_VERSIONED,
        "unsupported WAL fast record tag {tag}"
    );
    let collection_id = cur.read_string()?;
    let request_id = match cur.read_u8()? {
        0 => None,
        1 => Some(cur.read_string()?),
        other => return Err(anyhow!("invalid WAL fast request_id tag {other}")),
    };
    let item_count = cur.read_u32()? as usize;
    let mut items = Vec::with_capacity(item_count);
    for _ in 0..item_count {
        let external_id = cur.read_string()?;
        let field = cur.read_string()?;
        // `WAL_FAST_INDEX` (legacy, pre-#3952) never wrote a version on the
        // wire at all — every item it produced reconstructs as `None`,
        // unchanged from before this fix, so an AOF/WAL segment written by an
        // older binary keeps decoding exactly as it always did.
        // `WAL_FAST_INDEX_VERSIONED` carries an explicit presence byte per
        // item.
        let version = if tag == WAL_FAST_INDEX_VERSIONED {
            match cur.read_u8()? {
                0 => None,
                1 => Some(cur.read_u64()?),
                other => return Err(anyhow!("invalid WAL fast item version tag {other}")),
            }
        } else {
            None
        };
        let value = cur.read_field_value()?;
        items.push(IndexItem {
            external_id,
            field,
            value,
            version,
        });
    }
    cur.expect_eof()?;
    Ok(WalRecord {
        version,
        entry: RaftLogEntry::Index {
            collection_id,
            req: IndexRequest { items, request_id },
        },
    })
}

struct FastCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> FastCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn expect_magic(&mut self, magic: &[u8]) -> Result<()> {
        let got = self.read_exact(magic.len())?;
        anyhow::ensure!(got == magic, "invalid WAL fast magic");
        Ok(())
    }

    fn expect_eof(&self) -> Result<()> {
        anyhow::ensure!(
            self.pos == self.bytes.len(),
            "trailing bytes in WAL fast record"
        );
        Ok(())
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| anyhow!("WAL fast cursor overflow"))?;
        if end > self.bytes.len() {
            return Err(anyhow!("truncated WAL fast record"));
        }
        let out = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut raw = [0u8; 4];
        raw.copy_from_slice(self.read_exact(4)?);
        Ok(u32::from_le_bytes(raw))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let mut raw = [0u8; 4];
        raw.copy_from_slice(self.read_exact(4)?);
        Ok(f32::from_le_bytes(raw))
    }

    fn read_f64(&mut self) -> Result<f64> {
        let mut raw = [0u8; 8];
        raw.copy_from_slice(self.read_exact(8)?);
        Ok(f64::from_le_bytes(raw))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut raw = [0u8; 8];
        raw.copy_from_slice(self.read_exact(8)?);
        Ok(u64::from_le_bytes(raw))
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| anyhow!("invalid WAL fast utf8: {e}"))
    }

    /// Shared `FieldValue` decode for both fast-Index tags — identical wire
    /// shape in each, only the item prefix (per-item version) differs.
    fn read_field_value(&mut self) -> Result<FieldValue> {
        match self.read_u8()? {
            WAL_VALUE_STRING => Ok(FieldValue::String(self.read_string()?)),
            WAL_VALUE_NUMBER => Ok(FieldValue::Number(self.read_f64()?)),
            WAL_VALUE_VECTOR => {
                let len = self.read_u32()? as usize;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(self.read_f32()?);
                }
                Ok(FieldValue::Vector(v))
            }
            WAL_VALUE_STRING_LIST => {
                let len = self.read_u32()? as usize;
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(self.read_string()?);
                }
                Ok(FieldValue::StringList(values))
            }
            other => Err(anyhow!("invalid WAL fast field value tag {other}")),
        }
    }
}

/// A live, ordered subscription: `(seq, record)` pairs with strictly
/// increasing `seq`, delivered as they become available. Never
/// completes on its own (it tails the log) unless the backend closes.
pub type WalStream = Pin<Box<dyn Stream<Item = Result<(u64, WalRecord)>> + Send>>;
pub type WalAdmissionStream = Pin<Box<dyn Stream<Item = Result<(u64, WalDelivery)>> + Send>>;

#[doc(hidden)]
pub enum WalDelivery {
    Resident(WalRecord),
    Deferred(WalSourceRecord),
}

#[doc(hidden)]
pub struct WalSourceRecord {
    sequence: u64,
    slot: Arc<Mutex<MemWalSlot>>,
}

#[doc(hidden)]
pub struct WalSourceRelease {
    sequence: u64,
}

impl WalSourceRelease {
    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }
}

fn check_resident_admission(record: &WalRecord, admitted_bytes: usize) -> Result<()> {
    let raw = Engine::record_owned_bytes(&record.entry)?;
    anyhow::ensure!(
        admitted_bytes >= raw,
        "resident WAL record was read without decoded-record admission"
    );
    Ok(())
}

impl WalDelivery {
    /// Return a pinned mapped fast Index payload after dropping the source-slot
    /// mutex. The owner keeps the staged directory and inode alive for callers
    /// that wait for capacity or move work to a blocking task.
    pub(crate) fn mapped_fast_index(&self) -> Result<Option<MappedFastIndexPayload>> {
        let Self::Deferred(source) = self else {
            return Ok(None);
        };
        source.mapped_fast_index()
    }
    /// Return a pinned mapped generic-CBOR body after releasing the source
    /// slot mutex. The returned bytes exclude the private stage header.
    pub(crate) fn mapped_generic_cbor(&self) -> Result<Option<MappedGenericCborPayload>> {
        let Self::Deferred(source) = self else {
            return Ok(None);
        };
        source.mapped_generic_cbor()
    }
    /// Associate source-backed reservation ownership with a native deferred
    /// source until its resident slot is staged or truncated.
    pub(crate) fn retain_source(
        &self,
        retention: crate::change_budget::SourceRetention,
    ) -> Result<()> {
        let Self::Deferred(source) = self else {
            return Ok(());
        };
        let mut slot = source
            .slot
            .lock()
            .map_err(|_| anyhow!("MemWal source slot poisoned"))?;
        if let MemWalSlot::Resident(_, retentions) = &mut *slot {
            retentions.push(retention);
        }
        Ok(())
    }

    pub(crate) fn decoded_owned_bytes(&self) -> Result<usize> {
        match self {
            Self::Resident(record) => Engine::record_owned_bytes(&record.entry).map_err(Into::into),
            Self::Deferred(record) => record.decoded_owned_bytes(),
        }
    }
    pub(crate) fn read_scratch_bytes(&self) -> usize {
        match self {
            Self::Resident(_) => 0,
            Self::Deferred(record) => record.read_scratch_bytes(),
        }
    }
    /// A staged source has no retained raw transport record. Reserve its
    /// decoder workspace and, when used, the AOF clone/encoder separately.
    pub(crate) fn extra_delivery_bytes(&self, has_aof: bool) -> Result<usize> {
        let raw = self.decoded_owned_bytes()?;
        let staged = match self {
            Self::Resident(_) => false,
            Self::Deferred(source) => matches!(
                &*source
                    .slot
                    .lock()
                    .map_err(|_| anyhow!("MemWal source slot poisoned"))?,
                MemWalSlot::Staged(_)
            ),
        };
        let copies = match (staged, has_aof) {
            (true, false) => 0,
            (true, true) => 3,
            (false, false) => 2,
            (false, true) => 4,
        };
        raw.checked_mul(copies)
            .and_then(|extra| extra.checked_add(self.read_scratch_bytes()))
            .ok_or_else(|| anyhow!("WAL delivery memory bound overflow"))
    }

    pub(crate) fn read(self, admitted_bytes: usize) -> Result<WalRecord> {
        match self {
            Self::Resident(record) => {
                check_resident_admission(&record, admitted_bytes)?;
                Ok(record)
            }
            Self::Deferred(record) => record.read(admitted_bytes),
        }
    }
}

impl WalSourceRecord {
    fn mapped_fast_index(&self) -> Result<Option<MappedFastIndexPayload>> {
        let stage = {
            let slot = self
                .slot
                .lock()
                .map_err(|_| anyhow!("MemWal source slot poisoned"))?;
            match &*slot {
                MemWalSlot::Resident(_, _) => return Ok(None),
                MemWalSlot::Staged(stage) => {
                    anyhow::ensure!(
                        stage.sequence() == self.sequence,
                        "staged MemWal descriptor sequence mismatch"
                    );
                    stage.clone()
                }
            }
        };
        stage.mapped_fast_index().map_err(Into::into)
    }
    fn mapped_generic_cbor(&self) -> Result<Option<MappedGenericCborPayload>> {
        let stage = {
            let slot = self
                .slot
                .lock()
                .map_err(|_| anyhow!("MemWal source slot poisoned"))?;
            match &*slot {
                MemWalSlot::Resident(_, _) => return Ok(None),
                MemWalSlot::Staged(stage) => {
                    anyhow::ensure!(
                        stage.sequence() == self.sequence,
                        "staged MemWal descriptor sequence mismatch"
                    );
                    stage.clone()
                }
            }
        };
        stage.mapped_generic_cbor().map_err(Into::into)
    }
    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }
    pub(crate) fn decoded_owned_bytes(&self) -> Result<usize> {
        let slot = self
            .slot
            .lock()
            .map_err(|_| anyhow!("MemWal source slot poisoned"))?;
        match &*slot {
            MemWalSlot::Resident(record, _) => {
                Engine::record_owned_bytes(&record.entry).map_err(Into::into)
            }
            MemWalSlot::Staged(stage) => Ok(stage.decoded_owned_bytes()),
        }
    }
    pub(crate) fn read_scratch_bytes(&self) -> usize {
        self.slot
            .lock()
            .ok()
            .map(|slot| match &*slot {
                MemWalSlot::Resident(_, _) => 0,
                MemWalSlot::Staged(stage) => stage.read_scratch_bytes(),
            })
            .unwrap_or(0)
    }
    pub(crate) fn read(self, admitted_bytes: usize) -> Result<WalRecord> {
        let slot = self
            .slot
            .lock()
            .map_err(|_| anyhow!("MemWal source slot poisoned"))?;
        match &*slot {
            MemWalSlot::Resident(record, _) => {
                check_resident_admission(record, admitted_bytes)?;
                Ok(record.clone())
            }
            MemWalSlot::Staged(stage) => {
                anyhow::ensure!(
                    stage.sequence() == self.sequence,
                    "staged MemWal descriptor sequence mismatch"
                );
                stage.read(admitted_bytes).map_err(Into::into)
            }
        }
    }
}

/// The log seam. `publish` appends and returns the assigned global
/// sequence; `subscribe` tails from a sequence; `latest_seq` reports the
/// head. Object-safe so it can live behind `Arc<dyn WalLog>`.
#[async_trait]
pub trait WalLog: Send + Sync {
    /// Append `record`, returning the global sequence assigned to it.
    async fn publish(&self, record: WalRecord) -> Result<u64>;

    /// Tail every record with `seq > from_seq` (use `0` for "from the
    /// beginning"), in order, including future appends.
    async fn subscribe(&self, from_seq: u64) -> Result<WalStream>;

    async fn subscribe_admitted(&self, from_seq: u64) -> Result<WalAdmissionStream> {
        Ok(Box::pin(self.subscribe(from_seq).await?.map(|item| {
            item.map(|(seq, record)| (seq, WalDelivery::Resident(record)))
        })))
    }

    async fn stage_source(&self, _: u64) -> Result<Option<WalSourceRelease>> {
        Ok(None)
    }

    /// Highest sequence currently in the log (`0` if empty).
    async fn latest_seq(&self) -> Result<u64>;
}

pub type SharedWal = Arc<dyn WalLog>;

// ---------------------------------------------------------------------------
// MemWal — in-process backend
// ---------------------------------------------------------------------------

/// In-memory log with **truncation behind the consumed watermark**.
///
/// Sequences are 1-based and stable: `base` counts records already
/// dropped, so the record at `records[i]` has seq `base + i + 1`. Once
/// every live subscriber has consumed past a record, it is dropped from
/// the front — so under a steady single-subscriber workload (the serving
/// node's apply loop, always caught up) memory stays flat regardless of
/// throughput, instead of the log growing forever.
///
/// A registered subscriber never loses data: truncation only drops up to
/// the *minimum* delivered sequence across all subscribers, which is by
/// definition ≤ what each one has already consumed. With no subscribers,
/// nothing is dropped (a future `subscribe(0)` can still replay).
#[derive(Clone)]
pub struct MemWal {
    shared: Arc<Mutex<MemWalInner>>,
    len_tx: Arc<watch::Sender<u64>>,
    source_stager: Arc<Mutex<Option<WalSourceStager>>>,
}

struct MemWalInner {
    records: std::collections::VecDeque<Arc<Mutex<MemWalSlot>>>,
    base: u64,
    // Sub id → highest sequence safe to discard. The most recently returned
    // record stays pinned until the subscriber begins its next poll.
    subs: std::collections::HashMap<u64, u64>,
    next_sub_id: u64,
}

enum MemWalSlot {
    Resident(WalRecord, Vec<crate::change_budget::SourceRetention>),
    Staged(Arc<StagedWalRecord>),
}

impl MemWalInner {
    fn latest(&self) -> u64 {
        self.base + self.records.len() as u64
    }

    fn maybe_truncate(&mut self) {
        if self.subs.is_empty() {
            return; // no consumers → keep everything for a future replay
        }
        let low_water = self.subs.values().copied().min().unwrap_or(0);
        while !self.records.is_empty() && self.base + 1 <= low_water {
            self.records.pop_front();
            self.base += 1;
        }
    }
}

/// Removes a subscription from `subs` when its stream is dropped, so a
/// gone subscriber never pins truncation forever.
struct SubGuard {
    shared: Arc<Mutex<MemWalInner>>,
    id: u64,
}

impl Drop for SubGuard {
    fn drop(&mut self) {
        if let Ok(mut s) = self.shared.lock() {
            s.subs.remove(&self.id);
        }
    }
}

impl Default for MemWal {
    fn default() -> Self {
        Self::new()
    }
}

impl MemWal {
    pub fn new() -> Self {
        Self::starting_at(0)
    }

    /// Like [`new`](Self::new) but the sequence domain starts above
    /// `base_seq` instead of `0` — required whenever the engine was seeded
    /// from a restored checkpoint (and, if applicable, replayed AOF tail):
    /// without this, a fresh in-process `MemWal` reassigns sequences `1..N`
    /// to genuinely new writes while the coordinator's `applied` watermark
    /// (seeded from the same restore) is already `>= N`, so the apply
    /// loop's redelivery-dedup guard silently discards them (#1486). The
    /// caller passes the final restored watermark (checkpoint `up_to_seq`,
    /// or the AOF-tail-replayed sequence if that is higher) so the first
    /// `publish` after restore is assigned `base_seq + 1` — strictly above
    /// anything the watermark already considers applied.
    pub fn starting_at(base_seq: u64) -> Self {
        let (len_tx, _rx) = watch::channel(base_seq);
        Self {
            shared: Arc::new(Mutex::new(MemWalInner {
                records: std::collections::VecDeque::new(),
                base: base_seq,
                subs: std::collections::HashMap::new(),
                next_sub_id: 0,
            })),
            len_tx: Arc::new(len_tx),
            source_stager: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl WalLog for MemWal {
    async fn publish(&self, record: WalRecord) -> Result<u64> {
        let seq = {
            let mut s = self
                .shared
                .lock()
                .map_err(|_| anyhow::anyhow!("MemWal poisoned"))?;
            s.records
                .push_back(Arc::new(Mutex::new(MemWalSlot::Resident(
                    record,
                    Vec::new(),
                ))));
            let seq = s.latest();
            s.maybe_truncate();
            seq
        };
        let _ = self.len_tx.send(seq);
        Ok(seq)
    }

    async fn subscribe(&self, from_seq: u64) -> Result<WalStream> {
        let shared = self.shared.clone();
        let rx = self.len_tx.subscribe();
        let id = {
            let mut s = shared
                .lock()
                .map_err(|_| anyhow::anyhow!("MemWal poisoned"))?;
            let id = s.next_sub_id;
            s.next_sub_id += 1;
            s.subs.insert(id, from_seq);
            id
        };
        let guard = SubGuard {
            shared: shared.clone(),
            id,
        };
        // State: (most recently delivered seq, watch rx, shared, guard).
        // A poll first acknowledges the prior record, so the record returned
        // by this poll remains replayable until the caller polls again.
        // Dropping the stream drops the guard → unregisters the subscription.
        let stream = futures::stream::unfold(
            (from_seq, rx, shared, guard),
            |(delivered, mut rx, shared, guard)| async move {
                loop {
                    let next = {
                        let mut s = match shared.lock() {
                            Ok(s) => s,
                            Err(_) => return None,
                        };
                        // The caller has started another poll, so it no
                        // longer needs its prior delivered record. Do this
                        // before selecting the next item; a record returned
                        // below is retained until the following poll.
                        s.subs.insert(guard.id, delivered);
                        s.maybe_truncate();
                        // Deliver the next seq after `delivered`, clamped
                        // above the truncation floor (never < base+1).
                        let want = (delivered + 1).max(s.base + 1);
                        let idx = (want - s.base - 1) as usize;
                        match s.records.get(idx).cloned() {
                            Some(slot) => Some((want, slot)),
                            None => None,
                        }
                    };
                    if let Some((seq, slot)) = next {
                        let rec = slot
                            .lock()
                            .map_err(|_| anyhow!("MemWal source slot poisoned"))
                            .and_then(|slot| match &*slot {
                                MemWalSlot::Resident(record, _) => Ok(record.clone()),
                                MemWalSlot::Staged(stage) => {
                                    stage.read(usize::MAX).map_err(Into::into)
                                }
                            });
                        return Some((rec.map(|rec| (seq, rec)), (seq, rx, shared, guard)));
                    }
                    if rx.changed().await.is_err() {
                        return None;
                    }
                }
            },
        );
        Ok(Box::pin(stream))
    }

    async fn subscribe_admitted(&self, from_seq: u64) -> Result<WalAdmissionStream> {
        let shared = self.shared.clone();
        let rx = self.len_tx.subscribe();
        let id = {
            let mut state = shared.lock().map_err(|_| anyhow!("MemWal poisoned"))?;
            let id = state.next_sub_id;
            state.next_sub_id += 1;
            state.subs.insert(id, from_seq);
            id
        };
        let guard = SubGuard {
            shared: shared.clone(),
            id,
        };
        let stream = futures::stream::unfold(
            (from_seq, rx, shared, guard),
            |(delivered, mut rx, shared, guard)| async move {
                loop {
                    let next = {
                        let mut state = match shared.lock() {
                            Ok(state) => state,
                            Err(_) => return None,
                        };
                        state.subs.insert(guard.id, delivered);
                        state.maybe_truncate();
                        let want = (delivered + 1).max(state.base + 1);
                        let index = (want - state.base - 1) as usize;
                        state.records.get(index).cloned().map(|slot| (want, slot))
                    };
                    if let Some((sequence, slot)) = next {
                        let delivery = WalDelivery::Deferred(WalSourceRecord { sequence, slot });
                        return Some((Ok((sequence, delivery)), (sequence, rx, shared, guard)));
                    }
                    if rx.changed().await.is_err() {
                        return None;
                    }
                }
            },
        );
        Ok(Box::pin(stream))
    }

    async fn stage_source(&self, sequence: u64) -> Result<Option<WalSourceRelease>> {
        let slot = {
            let state = self.shared.lock().map_err(|_| anyhow!("MemWal poisoned"))?;
            if sequence <= state.base {
                return Ok(None);
            }
            state
                .records
                .get((sequence - state.base - 1) as usize)
                .cloned()
        };
        let Some(slot) = slot else {
            return Ok(None);
        };
        let source_stager = self.source_stager.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<WalSourceRelease>> {
            let stager = {
                let mut source_stager = source_stager
                    .lock()
                    .map_err(|_| anyhow!("MemWal stager poisoned"))?;
                if source_stager.is_none() {
                    *source_stager = Some(WalSourceStager::for_mem_wal()?);
                }
                source_stager.as_ref().expect("just created stager").clone()
            };
            let mut slot = slot
                .lock()
                .map_err(|_| anyhow!("MemWal source slot poisoned"))?;
            match &mut *slot {
                MemWalSlot::Staged(stage) => {
                    if stage.sequence() != sequence || stage.source_id() != stager.source_id() {
                        anyhow::bail!("staged MemWal source identity mismatch");
                    }
                    Ok(Some(WalSourceRelease { sequence }))
                }
                MemWalSlot::Resident(record, _) => {
                    let staged = match stager.stage_fast_index(sequence, record)? {
                        Some(staged) => staged,
                        None => stager.stage(sequence, record)?,
                    };
                    if staged.sequence() != sequence || staged.source_id() != stager.source_id() {
                        anyhow::bail!("staged MemWal receipt identity mismatch");
                    }
                    *slot = MemWalSlot::Staged(Arc::new(staged));
                    Ok(Some(WalSourceRelease { sequence }))
                }
            }
        })
        .await
        .map_err(|error| anyhow!("MemWal stage worker failed: {error}"))?
    }

    async fn latest_seq(&self) -> Result<u64> {
        Ok(self
            .shared
            .lock()
            .map_err(|_| anyhow::anyhow!("MemWal poisoned"))?
            .latest())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_budget::ChangeBudget;
    use crate::types::{
        BatchUnindexDocsRequest, CreateCollectionRequest, FieldValue, IndexItem, IndexRequest,
    };
    use futures::StreamExt;
    use std::collections::BTreeMap;

    // Append inside `apps/lumen/src/wal.rs`'s existing `#[cfg(test)] mod tests`.
    // The injected-fault test needs this test-only helper in `wal_source_stage.rs`:
    //
    // #[cfg(test)] pub(crate) fn for_mem_wal_with_injector(
    //     injector: Arc<dyn StageFailureInjector>,
    // ) -> io::Result<Self>
    //
    // It must create the normal private directory and StageStore::with_injector;
    // do not expose it outside cfg(test).

    #[tokio::test]
    async fn admitted_locator_observes_exact_record_after_its_source_is_staged() {
        let wal = MemWal::new();
        let mut stream = wal.subscribe_admitted(0).await.unwrap();
        wal.publish(WalRecord::new(index_entry("orders", "a", "email", "a@x")))
            .await
            .unwrap();
        let (sequence, delivery) = stream.next().await.unwrap().unwrap();
        let WalDelivery::Deferred(locator) = delivery else {
            panic!("MemWal must deliver source locator");
        };
        assert_eq!(locator.sequence(), sequence);
        assert_eq!(
            wal.stage_source(sequence)
                .await
                .unwrap()
                .unwrap()
                .sequence(),
            sequence
        );
        // Staging changes the owned representation. Reprice its decode bound
        // after the source replacement and before reading it.
        let admitted = locator.decoded_owned_bytes().unwrap() + locator.read_scratch_bytes();
        let decoded = locator.read(admitted).unwrap();
        let RaftLogEntry::Index { collection_id, req } = decoded.entry else {
            panic!("expected Index");
        };
        assert_eq!(collection_id, "orders");
        assert!(matches!(&req.items[0].value, FieldValue::String(value) if value == "a@x"));
    }

    #[tokio::test]
    async fn staged_locator_pins_source_after_subscription_truncates_it() {
        let wal = MemWal::new();
        let mut stream = wal.subscribe_admitted(0).await.unwrap();
        wal.publish(WalRecord::new(create_entry("one")))
            .await
            .unwrap();
        let (sequence, delivery) = stream.next().await.unwrap().unwrap();
        let WalDelivery::Deferred(locator) = delivery else {
            panic!("expected locator");
        };
        wal.stage_source(sequence).await.unwrap();
        wal.publish(WalRecord::new(create_entry("two")))
            .await
            .unwrap();
        let _ = stream.next().await.unwrap().unwrap();
        assert_eq!(wal.shared.lock().unwrap().base, sequence);
        // Staging changes the owned representation. Reprice its decode bound
        // after the source replacement and before reading it.
        let admitted = locator.decoded_owned_bytes().unwrap() + locator.read_scratch_bytes();
        assert!(
            matches!(locator.read(admitted).unwrap().entry, RaftLogEntry::CreateCollection { collection_id, .. } if collection_id == "one")
        );
    }

    #[tokio::test]
    async fn admission_is_checked_for_resident_and_staged_sources() {
        let wal = MemWal::new();
        let mut stream = wal.subscribe_admitted(0).await.unwrap();
        wal.publish(WalRecord::new(create_entry("orders")))
            .await
            .unwrap();
        let (sequence, delivery) = stream.next().await.unwrap().unwrap();
        let WalDelivery::Deferred(resident) = delivery else {
            panic!("expected locator");
        };
        let raw = resident.decoded_owned_bytes().unwrap();
        assert!(resident.read(raw - 1).is_err());
        let (_, delivery) = wal
            .subscribe_admitted(0)
            .await
            .unwrap()
            .next()
            .await
            .unwrap()
            .unwrap();
        let WalDelivery::Deferred(staged) = delivery else {
            panic!("expected locator");
        };
        wal.stage_source(sequence).await.unwrap();
        assert!(
            staged.read(raw).is_err(),
            "stage scratch must also be admitted"
        );
    }

    #[tokio::test]
    async fn stage_proof_follows_slot_replacement_and_is_repeatable_per_source() {
        let wal = MemWal::new();
        wal.publish(WalRecord::new(create_entry("orders")))
            .await
            .unwrap();
        let first = wal.stage_source(1).await.unwrap().unwrap();
        assert_eq!(first.sequence(), 1);
        assert!(matches!(
            &*wal.shared.lock().unwrap().records[0].lock().unwrap(),
            MemWalSlot::Staged(_)
        ));
        assert_eq!(wal.stage_source(1).await.unwrap().unwrap().sequence(), 1);
        wal.publish(WalRecord::new(create_entry("other")))
            .await
            .unwrap();
        assert_eq!(wal.stage_source(2).await.unwrap().unwrap().sequence(), 2);
    }

    #[tokio::test]
    async fn compatibility_subscribe_replays_values_after_staging() {
        let wal = MemWal::new();
        wal.publish(WalRecord::new(index_entry("orders", "a", "email", "a@x")))
            .await
            .unwrap();
        wal.stage_source(1).await.unwrap();
        let mut stream = wal.subscribe(0).await.unwrap();
        let (sequence, record) = stream.next().await.unwrap().unwrap();
        assert_eq!(sequence, 1);
        assert_eq!(record.version, WAL_FORMAT_VERSION);
        assert!(matches!(record.entry, RaftLogEntry::Index { .. }));
    }

    // Injected fault contract: after adding the cfg(test) stager factory above,
    // replace `wal.source_stager` with the injected stager before calling
    // `stage_source`. Assert it returns Err, the slot remains Resident, and no
    // WalSourceRelease is produced. This needs a small MemWal cfg(test) setter
    // because the source stager is intentionally lazy and private.

    #[test]
    fn owned_delivery_rejects_missing_admission() {
        let record = WalRecord::new(create_entry("orders"));
        let raw = Engine::record_owned_bytes(&record.entry).unwrap();
        assert!(
            WalDelivery::Resident(record).read(raw - 1).is_err(),
            "owned delivery needs raw memory admission"
        );
    }

    #[tokio::test]
    async fn staged_descriptor_rejects_a_different_sequence() {
        let wal = MemWal::new();
        wal.publish(WalRecord::new(create_entry("orders")))
            .await
            .unwrap();
        let mut stream = wal.subscribe_admitted(0).await.unwrap();
        let (_, delivery) = stream.next().await.unwrap().unwrap();
        let WalDelivery::Deferred(mut locator) = delivery else {
            panic!("expected locator");
        };
        wal.stage_source(1).await.unwrap();
        let admitted = locator.decoded_owned_bytes().unwrap() + locator.read_scratch_bytes();
        locator.sequence = 2;
        assert!(
            locator.read(admitted).is_err(),
            "staged descriptor must match its source sequence"
        );
    }

    #[tokio::test]
    async fn failed_source_stage_keeps_resident_until_a_durable_retry() {
        use crate::committed_stage::{StageFailureInjector, StageFailurePoint};
        struct FailOnce(Mutex<Option<StageFailurePoint>>);
        impl StageFailureInjector for FailOnce {
            fn check(&self, point: StageFailurePoint) -> std::io::Result<()> {
                let mut fail = self.0.lock().unwrap();
                if fail.as_ref() == Some(&point) {
                    *fail = None;
                    return Err(std::io::Error::other(
                        "injected native source staging fault",
                    ));
                }
                Ok(())
            }
        }
        for point in [
            StageFailurePoint::SyncPayload,
            StageFailurePoint::RenamePayload,
            StageFailurePoint::SyncPayloadDirectory,
            StageFailurePoint::SyncMarker,
            StageFailurePoint::PublishMarker,
            StageFailurePoint::SyncMarkerDirectory,
        ] {
            let wal = MemWal::new();
            *wal.source_stager.lock().unwrap() = Some(
                WalSourceStager::for_mem_wal_with_injector(Arc::new(FailOnce(Mutex::new(Some(
                    point,
                )))))
                .unwrap(),
            );
            let original = WalRecord::new(index_entry("orders", "a", "email", "a@x"));
            let expected = original.encode().unwrap();
            wal.publish(original).await.unwrap();
            assert!(wal.stage_source(1).await.is_err(), "{point:?}");
            {
                let state = wal.shared.lock().unwrap();
                let slot = state.records[0].lock().unwrap();
                let MemWalSlot::Resident(record, _) = &*slot else {
                    panic!("failed stage discarded resident at {point:?}");
                };
                assert_eq!(record.encode().unwrap(), expected, "{point:?}");
            }
            assert_eq!(wal.stage_source(1).await.unwrap().unwrap().sequence(), 1);
            let (_, delivered) = wal
                .subscribe(0)
                .await
                .unwrap()
                .next()
                .await
                .unwrap()
                .unwrap();
            assert_eq!(delivered.encode().unwrap(), expected, "{point:?}");
        }
    }

    fn create_entry(coll: &str) -> RaftLogEntry {
        RaftLogEntry::CreateCollection {
            collection_id: coll.into(),
            req: CreateCollectionRequest {
                fields: BTreeMap::new(),
            },
        }
    }

    fn index_entry(coll: &str, eid: &str, field: &str, val: &str) -> RaftLogEntry {
        RaftLogEntry::Index {
            collection_id: coll.into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.into(),
                    field: field.into(),
                    value: FieldValue::String(val.into()),
                    version: None,
                }],
                request_id: None,
            },
        }
    }

    fn source_retention(budget: &ChangeBudget) -> crate::change_budget::SourceRetention {
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(7).unwrap();
        let retention = reservation.source_retention();
        drop(reservation);
        retention
    }

    #[tokio::test]
    async fn resident_source_retention_outlives_delivery_and_first_subscriber_poll() {
        let budget = ChangeBudget::with_hard_limit(16);
        let wal = MemWal::new();
        let mut first = wal.subscribe_admitted(0).await.unwrap();
        let mut second = wal.subscribe_admitted(0).await.unwrap();
        wal.publish(WalRecord::new(create_entry("one")))
            .await
            .unwrap();

        let (sequence, delivery) = first.next().await.unwrap().unwrap();
        assert_eq!(sequence, 1);
        let retention = source_retention(&budget);
        delivery.retain_source(retention.clone()).unwrap();
        drop(retention);
        drop(delivery);
        assert_eq!(
            budget.snapshot().total,
            7,
            "resident slot owns the source charge"
        );

        // The first subscriber has released its owned delivery and polls again,
        // but the second subscriber has not yet released this native source.
        wal.publish(WalRecord::new(create_entry("two")))
            .await
            .unwrap();
        assert_eq!(first.next().await.unwrap().unwrap().0, 2);
        assert_eq!(
            budget.snapshot().total,
            7,
            "second source pin still owns charge"
        );

        // Polling second through sequence two lets MemWal truncate sequence one.
        // The resident slot's final drop, rather than any timer or idle poll,
        // must release the charge.
        assert_eq!(second.next().await.unwrap().unwrap().0, 1);
        assert_eq!(second.next().await.unwrap().unwrap().0, 2);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[tokio::test]
    async fn verified_stage_replacement_releases_resident_source_retention_while_pinned() {
        let budget = ChangeBudget::with_hard_limit(16);
        let wal = MemWal::new();
        let mut subscriber = wal.subscribe_admitted(0).await.unwrap();
        wal.publish(WalRecord::new(create_entry("one")))
            .await
            .unwrap();

        let (sequence, delivery) = subscriber.next().await.unwrap().unwrap();
        let retention = source_retention(&budget);
        delivery.retain_source(retention.clone()).unwrap();
        drop(retention);
        assert_eq!(budget.snapshot().total, 7);

        assert_eq!(
            wal.stage_source(sequence)
                .await
                .unwrap()
                .unwrap()
                .sequence(),
            sequence
        );
        assert_eq!(
            budget.snapshot().total,
            0,
            "verified replacement releases only the resident-source charge"
        );

        // `subscriber` still pins the source slot and `delivery` still owns a
        // descriptor. The replacement must not make the staged record unreadable.
        let admitted = delivery.decoded_owned_bytes().unwrap() + delivery.read_scratch_bytes();
        assert!(matches!(
            delivery.read(admitted).unwrap().entry,
            RaftLogEntry::CreateCollection { collection_id, .. } if collection_id == "one"
        ));
    }

    #[test]
    fn record_round_trips() {
        let rec = WalRecord::new(create_entry("users"));
        let bytes = rec.encode().unwrap();
        let back = WalRecord::decode(&bytes).unwrap();
        assert!(matches!(back.entry, RaftLogEntry::CreateCollection { .. }));
        assert_eq!(back.version, WAL_FORMAT_VERSION);
    }

    #[test]
    fn truncate_control_record_round_trips_and_fails_a_v1_reader_at_version_gate() {
        let bytes = WalRecord::new(RaftLogEntry::TruncateDocs {
            collection_id: "users".into(),
        })
        .encode()
        .unwrap();
        assert!(bytes.starts_with(WAL_FAST_MAGIC));
        assert_eq!(bytes[WAL_FAST_MAGIC.len()], WAL_CONTROL_FORMAT_VERSION);
        assert_eq!(bytes[WAL_FAST_MAGIC.len() + 1], WAL_FAST_TRUNCATE_DOCS);

        let back = WalRecord::decode(&bytes).expect("0.4.31 must read a v2 control record");
        assert!(matches!(
            back.entry,
            RaftLogEntry::TruncateDocs { collection_id } if collection_id == "users"
        ));

        // This is the pre-0.4.31 fast-record entrance check.  It reads and
        // rejects the version byte before looking at the command tag, so a
        // direct downgrade fails closed with the intended compatibility
        // boundary rather than attempting to decode `TruncateDocs`.
        let v1_reader = || -> Result<()> {
            anyhow::ensure!(bytes.starts_with(WAL_FAST_MAGIC), "invalid WAL fast magic");
            anyhow::ensure!(
                bytes[WAL_FAST_MAGIC.len()] == WAL_FORMAT_VERSION,
                "unsupported WAL fast record version {} (expected {})",
                bytes[WAL_FAST_MAGIC.len()],
                WAL_FORMAT_VERSION
            );
            Ok(())
        };
        assert!(
            v1_reader().is_err(),
            "a v1 reader must refuse the v2 byte first"
        );
    }

    #[test]
    fn generic_wal_envelope_cannot_smuggle_control_versions_or_commands() {
        let truncate = RaftLogEntry::TruncateDocs {
            collection_id: "users".into(),
        };
        let unindex = RaftLogEntry::UnindexDocs {
            collection_id: "users".into(),
            req: BatchUnindexDocsRequest {
                external_ids: vec!["d1".into()],
            },
        };
        assert!(
            WalRecord {
                version: WAL_FORMAT_VERSION,
                entry: truncate.clone(),
            }
            .encode()
            .is_err(),
            "a v1 envelope must never emit TruncateDocs"
        );
        assert!(
            WalRecord {
                version: WAL_FORMAT_VERSION,
                entry: unindex.clone(),
            }
            .encode()
            .is_err(),
            "a v1 envelope must never emit UnindexDocs"
        );
        assert!(
            WalRecord {
                version: WAL_CONTROL_FORMAT_VERSION,
                entry: create_entry("users"),
            }
            .encode()
            .is_err(),
            "v2 is reserved for the fast control tag"
        );

        // Decode must enforce the same boundary even for bytes that bypassed
        // this build's encoder (for example, a malformed external WAL frame).
        for record in [
            WalRecord {
                version: WAL_FORMAT_VERSION,
                entry: truncate,
            },
            WalRecord {
                version: WAL_CONTROL_FORMAT_VERSION,
                entry: create_entry("users"),
            },
            WalRecord {
                version: WAL_FORMAT_VERSION,
                entry: unindex,
            },
        ] {
            let mut bytes = Vec::new();
            ciborium::ser::into_writer(&record, &mut bytes).unwrap();
            assert!(WalRecord::decode(&bytes).is_err());
        }
    }

    #[test]
    fn unindex_control_record_round_trips_and_rejects_invalid_fast_shapes() {
        let entry = RaftLogEntry::UnindexDocs {
            collection_id: "users".into(),
            req: BatchUnindexDocsRequest {
                external_ids: vec!["one".into(), "two".into()],
            },
        };
        let bytes = WalRecord::new(entry).encode().unwrap();
        assert!(bytes.starts_with(WAL_FAST_MAGIC));
        assert_eq!(bytes[WAL_FAST_MAGIC.len()], WAL_CONTROL_FORMAT_VERSION);
        assert_eq!(bytes[WAL_FAST_MAGIC.len() + 1], WAL_FAST_UNINDEX_DOCS);
        assert!(matches!(
            WalRecord::decode(&bytes).unwrap().entry,
            RaftLogEntry::UnindexDocs { collection_id, req }
                if collection_id == "users" && req.external_ids == ["one", "two"]
        ));

        // This is the legacy entrance guard: an older reader refuses the v2
        // byte before it can observe either control tag.
        assert_ne!(bytes[WAL_FAST_MAGIC.len()], WAL_FORMAT_VERSION);

        let mut over_limit = Vec::new();
        over_limit.extend_from_slice(WAL_FAST_MAGIC);
        over_limit.push(WAL_CONTROL_FORMAT_VERSION);
        over_limit.push(WAL_FAST_UNINDEX_DOCS);
        put_str(&mut over_limit, "users").unwrap();
        put_u32(&mut over_limit, MAX_BATCH_UNINDEX_DOCS_SIZE + 1).unwrap();
        let err = WalRecord::decode(&over_limit).unwrap_err();
        assert!(
            err.to_string().contains("item count"),
            "count must fail before allocation/read, got: {err}"
        );

        let mut duplicate = Vec::new();
        duplicate.extend_from_slice(WAL_FAST_MAGIC);
        duplicate.push(WAL_CONTROL_FORMAT_VERSION);
        duplicate.push(WAL_FAST_UNINDEX_DOCS);
        put_str(&mut duplicate, "users").unwrap();
        put_u32(&mut duplicate, 2).unwrap();
        put_str(&mut duplicate, "same").unwrap();
        put_str(&mut duplicate, "same").unwrap();
        let err = WalRecord::decode(&duplicate).unwrap_err();
        assert!(err.to_string().contains("duplicate"), "got: {err}");
    }

    #[test]
    fn fast_index_record_round_trips_all_value_shapes() {
        let rec = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: Some("req-1".into()),
                items: vec![
                    IndexItem {
                        external_id: "doc-1".into(),
                        field: "title".into(),
                        value: FieldValue::String("lumen".into()),
                        version: None,
                    },
                    IndexItem {
                        external_id: "doc-1".into(),
                        field: "score".into(),
                        value: FieldValue::Number(42.5),
                        version: None,
                    },
                    IndexItem {
                        external_id: "doc-1".into(),
                        field: "embedding".into(),
                        value: FieldValue::Vector(vec![0.25, 0.5, 0.75]),
                        version: None,
                    },
                    IndexItem {
                        external_id: "doc-1".into(),
                        field: "tags".into(),
                        value: FieldValue::StringList(vec!["rust".into(), "search".into()]),
                        version: None,
                    },
                ],
            },
        });
        let bytes = rec.encode().unwrap();
        assert!(bytes.starts_with(WAL_FAST_MAGIC));

        let back = WalRecord::decode(&bytes).unwrap();
        assert_eq!(back.version, WAL_FORMAT_VERSION);
        let RaftLogEntry::Index { collection_id, req } = back.entry else {
            panic!("expected index record");
        };
        assert_eq!(collection_id, "docs");
        assert_eq!(req.request_id.as_deref(), Some("req-1"));
        assert_eq!(req.items.len(), 4);
        assert!(matches!(
            &req.items[0].value,
            FieldValue::String(s) if s == "lumen"
        ));
        assert!(matches!(
            req.items[1].value,
            FieldValue::Number(n) if (n - 42.5).abs() < f64::EPSILON
        ));
        assert!(matches!(
            &req.items[2].value,
            FieldValue::Vector(v) if v == &[0.25, 0.5, 0.75]
        ));
        assert!(matches!(
            &req.items[3].value,
            FieldValue::StringList(values) if values == &["rust".to_string(), "search".to_string()]
        ));
        assert!(
            req.items.iter().all(|item| item.version.is_none()),
            "no item in this record carried a version; decode must not invent one"
        );
    }

    /// #3952: `IndexItem.version` (#184 external LWW) must survive the fast
    /// codec round trip — this is the exact gap the AOF replay bug (#3952)
    /// came from: the wire never carried it, so every replayed item looked
    /// unversioned and LWW silently degraded to arrival order.
    #[test]
    fn fast_index_record_round_trips_versioned_items() {
        let rec = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: None,
                items: vec![
                    IndexItem {
                        external_id: "d1".into(),
                        field: "kw".into(),
                        value: FieldValue::String("v5".into()),
                        version: Some(5),
                    },
                    IndexItem {
                        external_id: "d1".into(),
                        field: "kw".into(),
                        value: FieldValue::String("v3".into()),
                        version: Some(3),
                    },
                    IndexItem {
                        external_id: "d2".into(),
                        field: "kw".into(),
                        value: FieldValue::String("unversioned".into()),
                        version: None,
                    },
                ],
            },
        });
        let bytes = rec.encode().unwrap();
        assert!(bytes.starts_with(WAL_FAST_MAGIC));
        assert_eq!(
            bytes[5], WAL_FAST_INDEX_VERSIONED,
            "a record carrying versions must be written with the versioned tag \
             (byte layout: 4-byte magic, then a 1-byte format version, then this tag)"
        );

        let back = WalRecord::decode(&bytes).unwrap();
        let RaftLogEntry::Index { req, .. } = back.entry else {
            panic!("expected index record");
        };
        assert_eq!(req.items[0].version, Some(5));
        assert_eq!(req.items[1].version, Some(3));
        assert_eq!(
            req.items[2].version, None,
            "an item with no version must decode back to None, not 0 or Some(anything)"
        );
    }

    /// #3952 negative control: an AOF/WAL segment written by a binary before
    /// this fix used tag `WAL_FAST_INDEX` (1) and never wrote a version byte
    /// per item at all. That decode branch must stay byte-for-byte readable.
    ///
    /// Today's encoder still emits that tag — it picks the tag from the content,
    /// and a batch where no item carries a version is written unversioned, which
    /// is what keeps a segment readable by a peer that has not been upgraded
    /// yet. So this test does not bypass the encoder to reach an unreachable
    /// shape; it hand-builds the bytes so the branch is measured against a
    /// literal layout rather than against whatever the encoder currently emits
    /// — the encoder is the thing under suspicion in a compatibility case.
    /// It confirms decode still succeeds with every item's version
    /// reconstructed as `None`, matching the pre-fix behavior exactly.
    #[test]
    fn decode_fast_record_still_reads_legacy_unversioned_tag() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(WAL_FAST_MAGIC);
        bytes.push(WAL_FORMAT_VERSION);
        bytes.push(WAL_FAST_INDEX); // legacy tag — no per-item version byte
        put_str(&mut bytes, "docs").unwrap();
        bytes.push(0); // no request_id
        put_u32(&mut bytes, 1).unwrap(); // one item
        put_str(&mut bytes, "d1").unwrap();
        put_str(&mut bytes, "kw").unwrap();
        bytes.push(WAL_VALUE_STRING);
        put_str(&mut bytes, "v3").unwrap();

        let back = WalRecord::decode(&bytes).expect("legacy fast-Index record must still decode");
        let RaftLogEntry::Index { collection_id, req } = back.entry else {
            panic!("expected index record");
        };
        assert_eq!(collection_id, "docs");
        assert_eq!(req.items.len(), 1);
        assert_eq!(req.items[0].external_id, "d1");
        assert!(matches!(&req.items[0].value, FieldValue::String(s) if s == "v3"));
        assert_eq!(
            req.items[0].version, None,
            "a pre-#3952 record never had a version on the wire; it must decode as None, \
             exactly as it did before this fix — never fabricated from thin air"
        );
    }

    #[test]
    fn decode_rejects_bad_version() {
        let bytes = WalRecord {
            version: 9,
            entry: create_entry("u"),
        }
        .encode()
        .unwrap();
        assert!(WalRecord::decode(&bytes).is_err());
    }

    #[test]
    fn decode_accepts_legacy_json_payload() {
        let rec = WalRecord::new(create_entry("legacy-json"));
        let bytes = serde_json::to_vec(&rec).unwrap();
        let back = WalRecord::decode(&bytes).unwrap();
        assert!(matches!(back.entry, RaftLogEntry::CreateCollection { .. }));
        assert_eq!(back.version, WAL_FORMAT_VERSION);
    }

    #[tokio::test]
    async fn mem_publish_assigns_increasing_seq() {
        let wal = MemWal::new();
        let s1 = wal
            .publish(WalRecord::new(create_entry("a")))
            .await
            .unwrap();
        let s2 = wal
            .publish(WalRecord::new(create_entry("b")))
            .await
            .unwrap();
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
        assert_eq!(wal.latest_seq().await.unwrap(), 2);
    }

    /// #1486 R1: a `MemWal` seeded from a restored watermark assigns its
    /// first fresh sequence strictly above that watermark — required so the
    /// coordinator's `applied` watermark (also seeded from the same
    /// restore) never sees a fresh write land at or below it.
    #[tokio::test]
    async fn mem_starting_at_assigns_seq_above_base() {
        let wal = MemWal::starting_at(5);
        assert_eq!(wal.latest_seq().await.unwrap(), 5);
        let s1 = wal
            .publish(WalRecord::new(create_entry("a")))
            .await
            .unwrap();
        let s2 = wal
            .publish(WalRecord::new(create_entry("b")))
            .await
            .unwrap();
        assert_eq!(s1, 6);
        assert_eq!(s2, 7);
        assert_eq!(wal.latest_seq().await.unwrap(), 7);
    }

    /// #1486 R1: a subscriber tailing from exactly the restored watermark
    /// (mirrors the apply loop's `wal.subscribe(applied)` on cold start)
    /// receives every fresh record published after `starting_at`, in order
    /// — the exact delivery path the original bug broke.
    #[tokio::test]
    async fn mem_starting_at_subscribe_from_watermark_delivers_fresh_writes() {
        let wal = MemWal::starting_at(5);
        let mut sub = wal.subscribe(5).await.unwrap();
        let seq = wal
            .publish(WalRecord::new(create_entry("a")))
            .await
            .unwrap();
        assert_eq!(seq, 6);
        let (delivered_seq, _rec) = sub.next().await.unwrap().unwrap();
        assert_eq!(
            delivered_seq, 6,
            "first fresh write after a watermark restore must be delivered promptly"
        );
    }

    #[tokio::test]
    async fn mem_subscribe_replays_backlog_then_tails() {
        let wal = MemWal::new();
        wal.publish(WalRecord::new(index_entry("c", "u1", "e", "a@x")))
            .await
            .unwrap();
        wal.publish(WalRecord::new(index_entry("c", "u2", "e", "b@x")))
            .await
            .unwrap();

        let mut sub = wal.subscribe(0).await.unwrap();
        // Backlog.
        let (seq1, _) = sub.next().await.unwrap().unwrap();
        let (seq2, _) = sub.next().await.unwrap().unwrap();
        assert_eq!((seq1, seq2), (1, 2));

        // Live tail: publish after subscribing, the stream must deliver it.
        let wal2 = wal.clone();
        tokio::spawn(async move {
            wal2.publish(WalRecord::new(index_entry("c", "u3", "e", "c@x")))
                .await
                .unwrap();
        });
        let (seq3, _) = sub.next().await.unwrap().unwrap();
        assert_eq!(seq3, 3);
    }

    #[tokio::test]
    async fn mem_subscribe_from_offset_skips_backlog() {
        let wal = MemWal::new();
        for i in 0..5 {
            wal.publish(WalRecord::new(create_entry(&format!("c{i}"))))
                .await
                .unwrap();
        }
        // Subscribe from seq 3 → first delivered is seq 4.
        let mut sub = wal.subscribe(3).await.unwrap();
        let (seq, _) = sub.next().await.unwrap().unwrap();
        assert_eq!(seq, 4);
    }

    #[tokio::test]
    async fn mem_truncates_behind_a_caught_up_subscriber() {
        // The single-subscriber steady-state contract: a subscriber that
        // keeps up lets the log drop everything it has consumed, so the
        // retained record count stays bounded no matter how much is
        // published.
        let wal = MemWal::new();
        let mut sub = wal.subscribe(0).await.unwrap();
        for i in 0..200u32 {
            wal.publish(WalRecord::new(create_entry(&format!("c{i}"))))
                .await
                .unwrap();
            // Consume each as it arrives — stays caught up.
            let (seq, _) = sub.next().await.unwrap().unwrap();
            assert_eq!(seq, i as u64 + 1);
        }
        // latest_seq keeps climbing (stable, monotonic) ...
        assert_eq!(wal.latest_seq().await.unwrap(), 200);
        // ... but retained records are bounded near zero, not 200.
        let retained = wal.shared.lock().unwrap().records.len();
        assert!(
            retained <= 1,
            "log should truncate behind the consumer, retained={retained}"
        );
    }

    #[tokio::test]
    async fn mem_keeps_delivered_record_until_the_consumer_polls_again() {
        let wal = MemWal::new();
        let mut first = wal.subscribe(0).await.unwrap();

        wal.publish(WalRecord::new(create_entry("one")))
            .await
            .unwrap();
        let (first_seq, _) = first.next().await.unwrap().unwrap();
        assert_eq!(first_seq, 1);

        // The first consumer still owns the returned record. A publisher
        // must not discard it before another subscriber can install a replay
        // cursor for that record.
        wal.publish(WalRecord::new(create_entry("two")))
            .await
            .unwrap();
        let mut replay = wal.subscribe(0).await.unwrap();
        let (replayed_seq, _) = replay.next().await.unwrap().unwrap();
        assert_eq!(replayed_seq, 1, "the held record must remain replayable");

        // Once the first consumer asks for its next record, its previous
        // record is safe to truncate. Drop the replay cursor first so it no
        // longer pins record 1.
        drop(replay);
        let (second_seq, _) = first.next().await.unwrap().unwrap();
        assert_eq!(second_seq, 2);
        let state = wal.shared.lock().unwrap();
        assert_eq!(state.base, 1, "polling forward must advance truncation");
        assert_eq!(state.records.len(), 1);
    }

    #[tokio::test]
    async fn mem_no_subscriber_retains_for_future_replay() {
        // With no subscribers, nothing is dropped — a late subscriber can
        // still replay from the beginning.
        let wal = MemWal::new();
        for i in 0..10u32 {
            wal.publish(WalRecord::new(create_entry(&format!("c{i}"))))
                .await
                .unwrap();
        }
        let mut sub = wal.subscribe(0).await.unwrap();
        let (first, _) = sub.next().await.unwrap().unwrap();
        assert_eq!(first, 1, "late subscriber must still replay from seq 1");
    }
}
// CODEGEN-END
