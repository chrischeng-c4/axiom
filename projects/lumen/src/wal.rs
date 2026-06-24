// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Write-ahead log abstraction — the data-plane backbone.
//!
//! lumen's write path is "turn the database inside out": a write is published
//! to an ordered log and then folded into each serving node's materialized
//! index. The log may be in-process (`MemWal`), externally owned (`RelayWal` /
//! legacy `NatsWal`), or Lumen-owned primary/replica replication.
//!
//! This mirrors Redis's AOF (the op log) + replication stream, with the
//! "master" role dissolved into "the log owner":
//!
//! - **AOF**  → this log (a stream of [`WalRecord`]).
//! - **RDB**  → periodic snapshots to object storage (see `rdb`), tagged
//!   with the log sequence they correspond to, so a fresh node loads a
//!   baseline then tails the log from there.
//!
//! Three backends implement [`WalLog`]:
//!
//! - [`MemWal`] — in-process, in-memory. Unit tests + the simplest
//!   single-node dev runs. Publish applies synchronously from the
//!   caller's perspective (the subscriber sees it immediately).
//! - `RelayWal` (in `wal_relay`) — explicit Relay broadcast broker mode. Each
//!   serving node has an independent subscriber id reading the full stream.
//! - `NatsWal` (in `wal_nats`) — legacy NATS JetStream backend retained for
//!   compatibility/tests.
//!
//! The record payload reuses [`crate::log_entry::RaftLogEntry`] — it
//! already enumerates every mutation 1:1 with an `Engine` method and is
//! the exact shape a replication record needs.

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::log_entry::RaftLogEntry;
use crate::types::{FieldValue, IndexItem, IndexRequest};

/// Current on-the-wire record format version.
pub const WAL_FORMAT_VERSION: u8 = 1;
const WAL_FAST_MAGIC: &[u8; 4] = b"LWAL";
const WAL_FAST_INDEX: u8 = 1;
const WAL_VALUE_STRING: u8 = 1;
const WAL_VALUE_NUMBER: u8 = 2;
const WAL_VALUE_VECTOR: u8 = 3;
const WAL_VALUE_STRING_LIST: u8 = 4;

/// One durable, ordered mutation in the log. The sequence number is
/// **not** part of the record — it is assigned by the log on publish
/// and delivered alongside the record on subscribe (`MemWal` uses the append
/// index; broker or primary/replica backends own sequence assignment).
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
pub struct WalRecord {
    pub version: u8,
    pub entry: RaftLogEntry,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
impl WalRecord {
    pub fn new(entry: RaftLogEntry) -> Self {
        Self {
            version: WAL_FORMAT_VERSION,
            entry,
        }
    }

    #[inline]
    pub fn encode(&self) -> Result<Vec<u8>> {
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
        let rec: WalRecord = match ciborium::de::from_reader(bytes) {
            Ok(rec) => rec,
            Err(cbor_err) => serde_json::from_slice(bytes).map_err(|json_err| {
                anyhow!("decode WAL record as cbor ({cbor_err}) or legacy json ({json_err})")
            })?,
        };
        anyhow::ensure!(
            rec.version == WAL_FORMAT_VERSION,
            "unsupported WAL record version {} (expected {})",
            rec.version,
            WAL_FORMAT_VERSION
        );
        Ok(rec)
    }

    fn encode_fast_index(&self) -> Option<Vec<u8>> {
        let RaftLogEntry::Index { collection_id, req } = &self.entry else {
            return None;
        };
        let mut bytes = Vec::with_capacity(estimate_fast_index_len(collection_id, req));
        bytes.extend_from_slice(WAL_FAST_MAGIC);
        bytes.push(self.version);
        bytes.push(WAL_FAST_INDEX);
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
}

fn estimate_fast_index_len(collection_id: &str, req: &IndexRequest) -> usize {
    let mut len = WAL_FAST_MAGIC.len() + 2 + 4 + collection_id.len() + 1 + 4;
    if let Some(request_id) = &req.request_id {
        len += 4 + request_id.len();
    }
    for item in &req.items {
        len += 4 + item.external_id.len() + 4 + item.field.len() + 1;
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
        version == WAL_FORMAT_VERSION,
        "unsupported WAL fast record version {} (expected {})",
        version,
        WAL_FORMAT_VERSION
    );
    let tag = cur.read_u8()?;
    anyhow::ensure!(
        tag == WAL_FAST_INDEX,
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
        let value = match cur.read_u8()? {
            WAL_VALUE_STRING => FieldValue::String(cur.read_string()?),
            WAL_VALUE_NUMBER => FieldValue::Number(cur.read_f64()?),
            WAL_VALUE_VECTOR => {
                let len = cur.read_u32()? as usize;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(cur.read_f32()?);
                }
                FieldValue::Vector(v)
            }
            WAL_VALUE_STRING_LIST => {
                let len = cur.read_u32()? as usize;
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(cur.read_string()?);
                }
                FieldValue::StringList(values)
            }
            other => return Err(anyhow!("invalid WAL fast field value tag {other}")),
        };
        items.push(IndexItem {
            external_id,
            field,
            value,
            version: None,
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

    fn read_string(&mut self) -> Result<String> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| anyhow!("invalid WAL fast utf8: {e}"))
    }
}

/// A live, ordered subscription: `(seq, record)` pairs with strictly
/// increasing `seq`, delivered as they become available. Never
/// completes on its own (it tails the log) unless the backend closes.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
pub type WalStream = Pin<Box<dyn Stream<Item = Result<(u64, WalRecord)>> + Send>>;

/// The log seam. `publish` appends and returns the assigned global
/// sequence; `subscribe` tails from a sequence; `latest_seq` reports the
/// head. Object-safe so it can live behind `Arc<dyn WalLog>`.
#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
pub trait WalLog: Send + Sync {
    /// Append `record`, returning the global sequence assigned to it.
    async fn publish(&self, record: WalRecord) -> Result<u64>;

    /// Tail every record with `seq > from_seq` (use `0` for "from the
    /// beginning"), in order, including future appends.
    async fn subscribe(&self, from_seq: u64) -> Result<WalStream>;

    /// Highest sequence currently in the log (`0` if empty).
    async fn latest_seq(&self) -> Result<u64>;
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
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
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
pub struct MemWal {
    shared: Arc<Mutex<MemWalInner>>,
    len_tx: Arc<watch::Sender<u64>>,
}

struct MemWalInner {
    records: std::collections::VecDeque<WalRecord>,
    base: u64,
    subs: std::collections::HashMap<u64, u64>, // sub id → highest delivered seq
    next_sub_id: u64,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
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

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
impl Drop for SubGuard {
    fn drop(&mut self) {
        if let Ok(mut s) = self.shared.lock() {
            s.subs.remove(&self.id);
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
impl Default for MemWal {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
impl MemWal {
    pub fn new() -> Self {
        let (len_tx, _rx) = watch::channel(0u64);
        Self {
            shared: Arc::new(Mutex::new(MemWalInner {
                records: std::collections::VecDeque::new(),
                base: 0,
                subs: std::collections::HashMap::new(),
                next_sub_id: 0,
            })),
            len_tx: Arc::new(len_tx),
        }
    }
}

#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal-rs.md#source
impl WalLog for MemWal {
    async fn publish(&self, record: WalRecord) -> Result<u64> {
        let seq = {
            let mut s = self
                .shared
                .lock()
                .map_err(|_| anyhow::anyhow!("MemWal poisoned"))?;
            s.records.push_back(record);
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
        // State: (delivered seq, watch rx, shared, guard). Dropping the
        // stream drops the guard → unregisters the subscription.
        let stream = futures::stream::unfold(
            (from_seq, rx, shared, guard),
            |(delivered, mut rx, shared, guard)| async move {
                loop {
                    let next = {
                        let mut s = match shared.lock() {
                            Ok(s) => s,
                            Err(_) => return None,
                        };
                        // Deliver the next seq after `delivered`, clamped
                        // above the truncation floor (never < base+1).
                        let want = (delivered + 1).max(s.base + 1);
                        let idx = (want - s.base - 1) as usize;
                        match s.records.get(idx).cloned() {
                            Some(rec) => {
                                s.subs.insert(guard.id, want);
                                Some((want, rec))
                            }
                            None => None,
                        }
                    };
                    if let Some((seq, rec)) = next {
                        return Some((Ok((seq, rec)), (seq, rx, shared, guard)));
                    }
                    if rx.changed().await.is_err() {
                        return None;
                    }
                }
            },
        );
        Ok(Box::pin(stream))
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
    use crate::types::{CreateCollectionRequest, FieldValue, IndexItem, IndexRequest};
    use futures::StreamExt;
    use std::collections::BTreeMap;

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

    #[test]
    fn record_round_trips() {
        let rec = WalRecord::new(create_entry("users"));
        let bytes = rec.encode().unwrap();
        let back = WalRecord::decode(&bytes).unwrap();
        assert!(matches!(back.entry, RaftLogEntry::CreateCollection { .. }));
        assert_eq!(back.version, WAL_FORMAT_VERSION);
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
