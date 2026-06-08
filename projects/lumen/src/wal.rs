//! Write-ahead log abstraction — the data-plane backbone.
//!
//! lumen's write path is "turn the database inside out": a write is not
//! applied to the receiving node's index. It is **published** to an
//! ordered, durable log; every serving node **subscribes** to that log
//! and folds it into its own materialized index. All nodes converge by
//! applying the same totally-ordered stream — there is no primary among
//! the serving nodes, only the log decides order.
//!
//! This mirrors Redis's AOF (the op log) + replication stream, with the
//! "master" role dissolved into "the log owner":
//!
//! - **AOF**  → this log (a stream of [`WalRecord`]).
//! - **RDB**  → periodic snapshots to object storage (see `rdb`), tagged
//!   with the log sequence they correspond to, so a fresh node loads a
//!   baseline then tails the log from there.
//!
//! Two backends implement [`WalLog`]:
//!
//! - [`MemWal`] — in-process, in-memory. Unit tests + the simplest
//!   single-node dev runs. Publish applies synchronously from the
//!   caller's perspective (the subscriber sees it immediately).
//! - `NatsWal` (in `wal_nats`) — NATS JetStream. Clustered deployments:
//!   the broker owns durability, ordering, replication and fan-out. Each
//!   serving node is an independent consumer reading the full stream.
//!
//! The record payload reuses [`crate::log_entry::RaftLogEntry`] — it
//! already enumerates every mutation 1:1 with an `Engine` method and is
//! the exact shape a replication record needs.

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::log_entry::RaftLogEntry;

/// Current on-the-wire record format version.
pub const WAL_FORMAT_VERSION: u8 = 1;

/// One durable, ordered mutation in the log. The sequence number is
/// **not** part of the record — it is assigned by the log on publish
/// and delivered alongside the record on subscribe (NATS owns it in the
/// clustered case; `MemWal` uses the append index).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalRecord {
    pub version: u8,
    pub entry: RaftLogEntry,
}

impl WalRecord {
    pub fn new(entry: RaftLogEntry) -> Self {
        Self {
            version: WAL_FORMAT_VERSION,
            entry,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let rec: WalRecord = serde_json::from_slice(bytes)?;
        anyhow::ensure!(
            rec.version == WAL_FORMAT_VERSION,
            "unsupported WAL record version {} (expected {})",
            rec.version,
            WAL_FORMAT_VERSION
        );
        Ok(rec)
    }
}

/// A live, ordered subscription: `(seq, record)` pairs with strictly
/// increasing `seq`, delivered as they become available. Never
/// completes on its own (it tails the log) unless the backend closes.
pub type WalStream = Pin<Box<dyn Stream<Item = Result<(u64, WalRecord)>> + Send>>;

/// The log seam. `publish` appends and returns the assigned global
/// sequence; `subscribe` tails from a sequence; `latest_seq` reports the
/// head. Object-safe so it can live behind `Arc<dyn WalLog>`.
#[async_trait]
pub trait WalLog: Send + Sync {
    /// Append `record`, returning the global sequence assigned to it.
    async fn publish(&self, record: &WalRecord) -> Result<u64>;

    /// Tail every record with `seq > from_seq` (use `0` for "from the
    /// beginning"), in order, including future appends.
    async fn subscribe(&self, from_seq: u64) -> Result<WalStream>;

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
}

struct MemWalInner {
    records: std::collections::VecDeque<WalRecord>,
    base: u64,
    subs: std::collections::HashMap<u64, u64>, // sub id → highest delivered seq
    next_sub_id: u64,
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
impl WalLog for MemWal {
    async fn publish(&self, record: &WalRecord) -> Result<u64> {
        let seq = {
            let mut s = self
                .shared
                .lock()
                .map_err(|_| anyhow::anyhow!("MemWal poisoned"))?;
            s.records.push_back(record.clone());
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
    fn decode_rejects_bad_version() {
        let mut bytes = WalRecord::new(create_entry("u")).encode().unwrap();
        // Flip the version field in the JSON.
        let s = String::from_utf8(bytes)
            .unwrap()
            .replace("\"version\":1", "\"version\":9");
        bytes = s.into_bytes();
        assert!(WalRecord::decode(&bytes).is_err());
    }

    #[tokio::test]
    async fn mem_publish_assigns_increasing_seq() {
        let wal = MemWal::new();
        let s1 = wal
            .publish(&WalRecord::new(create_entry("a")))
            .await
            .unwrap();
        let s2 = wal
            .publish(&WalRecord::new(create_entry("b")))
            .await
            .unwrap();
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
        assert_eq!(wal.latest_seq().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn mem_subscribe_replays_backlog_then_tails() {
        let wal = MemWal::new();
        wal.publish(&WalRecord::new(index_entry("c", "u1", "e", "a@x")))
            .await
            .unwrap();
        wal.publish(&WalRecord::new(index_entry("c", "u2", "e", "b@x")))
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
            wal2.publish(&WalRecord::new(index_entry("c", "u3", "e", "c@x")))
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
            wal.publish(&WalRecord::new(create_entry(&format!("c{i}"))))
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
            wal.publish(&WalRecord::new(create_entry(&format!("c{i}"))))
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
            wal.publish(&WalRecord::new(create_entry(&format!("c{i}"))))
                .await
                .unwrap();
        }
        let mut sub = wal.subscribe(0).await.unwrap();
        let (first, _) = sub.next().await.unwrap().unwrap();
        assert_eq!(first, 1, "late subscriber must still replay from seq 1");
    }
}
