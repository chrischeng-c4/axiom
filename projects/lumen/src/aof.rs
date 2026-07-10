// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-aof-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Local append-only log (Stage 2 Phase 2f-3) — the binary's "AOF".
//!
//! The segment checkpoint ([`crate::segment_rdb`]) is the binary's "RDB": a
//! periodic, atomic snapshot of the materialized index, tagged with the WAL
//! sequence `S` it is current as of. Between checkpoints, this file is the
//! durable record of every APPLIED `(seq, WalRecord)` — the exact Redis
//! RDB+AOF split. Recovery is:
//!
//! 1. **RDB** — reopen the newest segment checkpoint (engine seeded to seq `S`),
//! 2. **AOF** — replay every frame with `seq > S` into the engine (to seq `A`),
//! 3. **Broker** — tail the log from `A + 1`.
//!
//! Because the AOF is durable through `A`, the broker stream only needs retention
//! beyond `A`, not from seq 0 — which is the whole point: broker retention can be
//! TRIMMED instead of kept forever.
//!
//! ## Frame format
//!
//! Each appended record is one self-describing frame:
//!
//! ```text
//! [ seq : u64 LE ][ len : u32 LE ][ crc : u32 LE ][ payload : len bytes ]
//! ```
//!
//! - `seq`     — the global sequence the record was applied at (the order key).
//! - `len`     — payload length in bytes.
//! - `crc`     — `crc32(payload)` (crc32fast), checked on replay.
//! - `payload` — the [`WalRecord`] encoded with ciborium (a compact, stable CBOR
//!   form — the same codec the segment checkpoint sidecars and CBOR RDB use).
//!
//! The 16-byte fixed header lets replay detect a TORN TAIL without parsing the
//! payload: if fewer than 16 header bytes remain, or `len` overruns EOF, or the
//! crc mismatches, the frame is incomplete (a crash landed mid-append) — replay
//! stops cleanly at the last good frame, with no panic and no error. The byte
//! offset of that last good frame's end is recorded so the next
//! [`AofWriter::open`] can truncate the torn tail before appending.
//!
//! ## fsync policy
//!
//! Mirrors Redis `appendfsync`:
//!
//! - [`FsyncPolicy::EverySec`] (default) — append writes to the OS buffer; a
//!   periodic [`AofWriter::maybe_sync`] (call-driven, off the apply hot path)
//!   fsyncs at most once per second. A crash loses at most ~1s of un-fsynced
//!   tail, which replay recovers as a torn tail (the frames are still in the OS
//!   page cache up to the crash point, and any partial frame is discarded).
//! - [`FsyncPolicy::Always`] — fsync after every append (tests / strict
//!   durability).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
pub use service_durability::FsyncPolicy;
use service_durability::{FramedLogReader, FramedLogWriter};

#[cfg(test)]
use std::fs::OpenOptions;
#[cfg(test)]
use std::io::{Read, Seek, SeekFrom, Write};

use crate::wal::WalRecord;

/// Fixed per-frame header width: `seq(8) + len(4) + crc(4)`.
#[cfg(test)]
const HEADER_LEN: usize = 16;

/// Encode a [`WalRecord`] payload. Common high-QPS index records use Lumen's
/// fast binary WAL codec; uncommon records fall back to compact CBOR.
fn encode_payload(rec: &WalRecord) -> Result<Vec<u8>> {
    rec.encode().context("encode AOF record")
}

/// Decode an AOF payload back into a [`WalRecord`].
fn decode_payload(bytes: &[u8]) -> Result<WalRecord> {
    WalRecord::decode(bytes).context("decode AOF record")
}

/// Append-only writer keyed by applied seq. Frames are appended in seq order;
/// `open` first truncates any torn tail left by a crash mid-append, so the file
/// always starts in a clean, fully-decodable state.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-aof-rs.md#source
pub struct AofWriter {
    inner: FramedLogWriter,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-aof-rs.md#source
impl AofWriter {
    /// Open `path` for appending with the default [`FsyncPolicy::EverySec`],
    /// first truncating any torn tail.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Self::open_with_policy(path, FsyncPolicy::EverySec)
    }

    /// Open `path` for appending, first truncating any torn tail (a partial
    /// frame from a crash mid-append) so the next append lands after the last
    /// good frame. Creates the file (and parent dirs) if absent.
    pub fn open_with_policy(path: impl Into<PathBuf>, policy: FsyncPolicy) -> Result<Self> {
        let path = path.into();
        Ok(Self {
            inner: FramedLogWriter::open(&path, policy)?,
        })
    }

    /// Append one applied `(seq, record)` frame. Buffered; durability follows the
    /// fsync policy (`Always` fsyncs now, `EverySec` defers to `maybe_sync`).
    pub fn append(&mut self, seq: u64, record: &WalRecord) -> Result<()> {
        let payload = encode_payload(record)?;
        self.inner.append(seq, &payload)
    }

    /// Flush the buffered writer to the OS (does NOT fsync). Cheap; safe to call
    /// often.
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }

    /// Flush + fsync NOW, unconditionally. Resets the everysec timer.
    pub fn sync(&mut self) -> Result<()> {
        self.inner.sync()
    }

    /// Call-driven everysec fsync: fsync only if dirty AND ≥ the cadence has
    /// elapsed since the last sync. Under `Always` this is a no-op (already
    /// synced on append). Meant to be called off the apply hot path (a periodic
    /// tick or after a batch), so the apply loop never blocks on fsync.
    pub fn maybe_sync(&mut self) -> Result<()> {
        self.inner.maybe_sync()
    }

    /// Drop every frame with `seq <= through`, keeping only newer frames. Called
    /// at checkpoint: once a checkpoint at seq `C` is durable in the segment RDB,
    /// every AOF frame with `seq <= C` is redundant and can be reclaimed.
    ///
    /// Crash-safe rewrite-survivors-to-temp + atomic rename: surviving frames are
    /// streamed (byte-for-byte, no re-encode) into `<path>.compact.tmp`, fsynced,
    /// then renamed over `path`. A crash before the rename leaves the original
    /// AOF intact (un-checkpointed frames are never lost); a crash after leaves
    /// the compacted AOF. The temp is removed first if a prior attempt left one.
    pub fn truncate_through(&mut self, through: u64) -> Result<()> {
        self.inner.truncate_through(through)
    }
}

/// Replay frames from an AOF, applying each `(seq, WalRecord)` with `seq >
/// from_seq` to a caller closure in order, stopping cleanly at a torn tail.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-aof-rs.md#source
pub struct AofReader;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-aof-rs.md#source
impl AofReader {
    /// Iterate every frame in `path` in order, SKIP frames with `seq <=
    /// from_seq` (already covered by the RDB baseline), and call `apply(seq,
    /// record)` for each frame with `seq > from_seq`. On a TORN TAIL (short read,
    /// `len` overruns EOF, or crc mismatch) STOP cleanly at the last good frame —
    /// no panic, no error. Returns the max seq REPLAYED (0 if none applied).
    ///
    /// An absent file replays nothing and returns 0 (a node that crashed before
    /// its first append simply has no AOF).
    pub fn replay(
        path: impl AsRef<Path>,
        from_seq: u64,
        mut apply: impl FnMut(u64, WalRecord),
    ) -> Result<u64> {
        let mut max_seq = 0u64;
        for frame in FramedLogReader::read_frames(path, from_seq)? {
            let rec = match decode_payload(&frame.payload) {
                Ok(r) => r,
                Err(_) => break,
            };
            apply(frame.seq, rec);
            max_seq = max_seq.max(frame.seq);
        }
        Ok(max_seq)
    }
}

/// Recovery helper: replay every AOF frame with `seq > from_seq` into `engine`
/// via [`crate::storage::Engine::apply_raft_entry`], returning the max seq
/// replayed. This is step 2 of cold start (RDB → **AOF** → broker); the engine is
/// already seeded to `from_seq` by the segment checkpoint.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-aof-rs.md#source
pub fn replay_aof_into(
    engine: &std::sync::Arc<crate::storage::Engine>,
    path: impl AsRef<Path>,
    from_seq: u64,
) -> Result<u64> {
    AofReader::replay(path, from_seq, |seq, rec| {
        if let Err(e) = engine.apply_raft_entry(rec.entry) {
            // An apply error here mirrors the live apply loop: log + no-op. The
            // record is still durable; a divergence would surface in the crux
            // recovery test.
            tracing::warn!(seq, error = %e, "AOF replay apply error (entry no-ops)");
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use std::collections::BTreeMap;

    fn create_entry(coll: &str) -> RaftLogEntry {
        RaftLogEntry::CreateCollection {
            collection_id: coll.into(),
            req: CreateCollectionRequest {
                fields: {
                    let mut f = BTreeMap::new();
                    f.insert(
                        "email".to_string(),
                        FieldSpec {
                            field_type: FieldType::Keyword,
                            analyzer: None,
                            multi: None,
                            dim: None,
                            metric: None,
                            backend: None,
                            quantize: None,
                        },
                    );
                    f
                },
            },
        }
    }

    fn index_entry(coll: &str, eid: &str, val: &str) -> RaftLogEntry {
        RaftLogEntry::Index {
            collection_id: coll.into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.into(),
                    field: "email".into(),
                    value: FieldValue::String(val.into()),
                    version: None,
                }],
                request_id: None,
            },
        }
    }

    fn rec(entry: RaftLogEntry) -> WalRecord {
        WalRecord::new(entry)
    }

    /// Collect (seq, record-debug) by replaying with from_seq = 0.
    fn replay_seqs(path: &Path, from: u64) -> Vec<u64> {
        let mut out = Vec::new();
        AofReader::replay(path, from, |seq, _rec| out.push(seq)).unwrap();
        out
    }

    #[test]
    fn append_then_replay_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        w.append(1, &rec(create_entry("u"))).unwrap();
        w.append(2, &rec(index_entry("u", "u1", "a@x.com")))
            .unwrap();
        w.append(3, &rec(index_entry("u", "u2", "b@x.com")))
            .unwrap();
        w.sync().unwrap();

        let mut seqs = Vec::new();
        let mut kinds = Vec::new();
        let max = AofReader::replay(&path, 0, |seq, r| {
            seqs.push(seq);
            kinds.push(matches!(r.entry, RaftLogEntry::CreateCollection { .. }));
        })
        .unwrap();
        assert_eq!(seqs, vec![1, 2, 3]);
        assert_eq!(max, 3);
        assert_eq!(kinds, vec![true, false, false]);
    }

    #[test]
    fn replay_skips_at_or_below_from_seq() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        for s in 1..=5 {
            w.append(s, &rec(index_entry("u", &format!("u{s}"), "x@y")))
                .unwrap();
        }
        w.sync().unwrap();
        // from_seq = 3 → only seq 4, 5 are replayed (strict `>`).
        assert_eq!(replay_seqs(&path, 3), vec![4, 5]);
        // from_seq = 0 → all.
        assert_eq!(replay_seqs(&path, 0), vec![1, 2, 3, 4, 5]);
        // from_seq = 5 → none.
        assert_eq!(replay_seqs(&path, 5), Vec::<u64>::new());
    }

    #[test]
    fn truncate_through_keeps_only_newer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        for s in 1..=6 {
            w.append(s, &rec(index_entry("u", &format!("u{s}"), "x@y")))
                .unwrap();
        }
        w.sync().unwrap();
        w.truncate_through(4).unwrap();
        // Frames 1..=4 dropped; 5, 6 survive.
        assert_eq!(replay_seqs(&path, 0), vec![5, 6]);
        // And the re-opened append handle keeps appending after the survivors.
        w.append(7, &rec(index_entry("u", "u7", "x@y"))).unwrap();
        w.sync().unwrap();
        assert_eq!(replay_seqs(&path, 0), vec![5, 6, 7]);
    }

    #[test]
    fn truncate_through_survivors_persist_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        {
            let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
            for s in 1..=5 {
                w.append(s, &rec(index_entry("u", &format!("u{s}"), "x@y")))
                    .unwrap();
            }
            w.sync().unwrap();
            w.truncate_through(2).unwrap();
        }
        // A fresh open sees only the survivors and can extend them.
        let mut w2 = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        w2.append(6, &rec(index_entry("u", "u6", "x@y"))).unwrap();
        w2.sync().unwrap();
        assert_eq!(replay_seqs(&path, 0), vec![3, 4, 5, 6]);
    }

    #[test]
    fn torn_tail_replays_prefix_then_recovers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        w.append(1, &rec(create_entry("u"))).unwrap();
        w.append(2, &rec(index_entry("u", "u1", "a@x"))).unwrap();
        w.append(3, &rec(index_entry("u", "u2", "b@x"))).unwrap();
        w.sync().unwrap();

        // Simulate a crash mid-append: corrupt the tail by appending a partial,
        // garbage frame (a header claiming a length that overruns EOF).
        let good_len = std::fs::metadata(&path).unwrap().len();
        {
            use std::io::Write as _;
            let mut f = OpenOptions::new().append(true).open(&path).unwrap();
            // seq=99, len=1_000_000 (way past EOF), crc=0, then a single byte.
            let mut hdr = [0u8; HEADER_LEN];
            hdr[0..8].copy_from_slice(&99u64.to_le_bytes());
            hdr[8..12].copy_from_slice(&1_000_000u32.to_le_bytes());
            f.write_all(&hdr).unwrap();
            f.write_all(&[0xAB]).unwrap();
            f.sync_all().unwrap();
        }
        assert!(std::fs::metadata(&path).unwrap().len() > good_len);

        // Replay stops cleanly at the last good frame — no panic, no error.
        assert_eq!(replay_seqs(&path, 0), vec![1, 2, 3]);

        // The next open TRUNCATES the torn tail back to the last good frame.
        let mut w2 = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().len(), good_len);
        // And a fresh append lands right after frame 3.
        w2.append(4, &rec(index_entry("u", "u3", "c@x"))).unwrap();
        w2.sync().unwrap();
        assert_eq!(replay_seqs(&path, 0), vec![1, 2, 3, 4]);
    }

    #[test]
    fn torn_tail_via_crc_mismatch_stops_at_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        w.append(1, &rec(index_entry("u", "u1", "a@x"))).unwrap();
        w.append(2, &rec(index_entry("u", "u2", "b@x"))).unwrap();
        w.sync().unwrap();

        // Flip a byte in the LAST frame's payload → crc mismatch → torn tail.
        let len = std::fs::metadata(&path).unwrap().len();
        {
            let mut f = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .unwrap();
            f.seek(SeekFrom::Start(len - 1)).unwrap();
            let mut b = [0u8; 1];
            f.read_exact(&mut b).unwrap();
            f.seek(SeekFrom::Start(len - 1)).unwrap();
            f.write_all(&[b[0] ^ 0xFF]).unwrap();
            f.sync_all().unwrap();
        }
        // Only frame 1 (the un-corrupted prefix) replays.
        assert_eq!(replay_seqs(&path, 0), vec![1]);
    }

    #[test]
    fn replay_missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.aof");
        assert_eq!(replay_seqs(&path, 0), Vec::<u64>::new());
    }
}

// ---------------------------------------------------------------------------
// THE CRUX: RDB + AOF recovery WITHOUT broker replay (Stage 2 Phase 2f-3).
//
// The whole durability story end-to-end, with the log out of the picture:
//
//   1. A "live" engine applies ops 1..=A via `apply_raft_entry`, with every op
//      ALSO appended to an AOF.
//   2. At seq S (< A) the segment checkpoint is taken (`flush_to_segments`) and
//      the AOF is `truncate_through(S)`d — so on disk the RDB covers 1..=S and
//      the AOF covers S+1..=A.
//   3. "Restart": a FRESH engine reopens the segment dir (recovers to S), then
//      `replay_aof_into` replays S+1..=A (recovers to A).
//
// The restarted engine's query results — result sets, byte-identical f32 scores,
// retrieved field values, and ordered kNN — must equal the live engine at A. If
// the frame crc/len decode, the seq-skip boundary, or `truncate_through` is
// wrong, recovery diverges (or the torn-tail path panics) and this test fails.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod crux_recovery_tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::storage::Engine;
    use crate::types::{
        Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem,
        IndexRequest, KnnQuery, MatchOp, MatchQuery, QueryNode, RangeBound, RangeQuery,
        SearchRequest, TermQuery, TermsQuery, VectorBackend, VectorMetric,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    const DIM: usize = 4;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    fn vec_fieldspec() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Vector,
            analyzer: None,
            multi: None,
            dim: Some(DIM as u32),
            metric: Some(VectorMetric::L2),
            backend: Some(VectorBackend::FlatCpu),
            quantize: None,
        }
    }

    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("num".into(), fieldspec(FieldType::Number, None));
        fields.insert("kw".into(), fieldspec(FieldType::Keyword, None));
        fields.insert("tags".into(), fieldspec(FieldType::Set, None));
        fields.insert(
            "body".into(),
            fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
        );
        fields.insert("sig".into(), fieldspec(FieldType::Hash, None));
        fields.insert("emb".into(), vec_fieldspec());
        CreateCollectionRequest { fields }
    }

    /// Build an `Index` entry for one doc across all six fields.
    fn index_entry(
        coll: &str,
        eid: &str,
        n: f64,
        kw: &str,
        tag: &str,
        tok: bool,
        sig: u64,
        emb: &[f32],
    ) -> RaftLogEntry {
        RaftLogEntry::Index {
            collection_id: coll.into(),
            req: IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: eid.into(),
                        field: "num".into(),
                        value: FieldValue::Number(n),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "kw".into(),
                        value: FieldValue::String(kw.into()),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "tags".into(),
                        value: FieldValue::StringList(vec![tag.into()]),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "body".into(),
                        value: FieldValue::String(if tok {
                            "tok filler".into()
                        } else {
                            "filler".into()
                        }),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "sig".into(),
                        value: FieldValue::String(format!("{sig:016x}")),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "emb".into(),
                        value: FieldValue::Vector(emb.to_vec()),
                        version: None,
                    },
                ],
                request_id: None,
            },
        }
    }

    fn req(query: QueryNode, limit: u32) -> SearchRequest {
        SearchRequest {
            query,
            limit,
            cursor: None,
            routing_key: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, coll: &str, query: QueryNode, limit: u32) -> Vec<(String, u32)> {
        e.search(coll, req(query, limit))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score.to_bits()))
            .collect()
    }

    fn set_of(rows: &[(String, u32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }
    fn scores_of(rows: &[(String, u32)]) -> BTreeMap<String, u32> {
        rows.iter().map(|(e, s)| (e.clone(), *s)).collect()
    }

    fn driven(extra: QueryNode) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            extra,
        ])
    }

    /// Full query battery: predicate legs (range/term/setmem/point/bm25/hamming)
    /// as (set, byte-scores), kNN as the ordered ranked vec, and a doc count.
    fn battery(e: &Engine, coll: &str) -> Vec<(BTreeSet<String>, BTreeMap<String, u32>)> {
        let legs = vec![
            driven(QueryNode::Range(RangeQuery {
                field: "num".into(),
                gt: None,
                gte: Some(RangeBound::Number(2.0)),
                lt: Some(RangeBound::Number(9.0)),
                lte: None,
            })),
            driven(QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String("a".into()),
            })),
            driven(QueryNode::Terms(TermsQuery {
                field: "tags".into(),
                values: vec![FieldValue::String("red".into())],
            })),
            QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String("b".into()),
            }),
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Hamming(crate::types::HammingQuery {
                field: "sig".into(),
                hash: format!("{:016x}", 0u64),
                max_distance: 8,
            }),
        ];
        legs.into_iter()
            .map(|q| {
                let r = run(e, coll, q, 100_000);
                (set_of(&r), scores_of(&r))
            })
            .collect()
    }

    fn knn(e: &Engine, coll: &str, q: &[f32]) -> Vec<(String, u32)> {
        run(
            e,
            coll,
            QueryNode::Knn(KnnQuery {
                field: "emb".into(),
                vector: q.to_vec(),
                k: 8,
            }),
            8,
        )
    }

    /// The full op TRANSCRIPT, in apply order. Two collections, all field types;
    /// base docs (1..=S region) then tail docs (S+1..=A region). Returns the
    /// ordered RaftLogEntry list — applied with seq = index+1.
    fn transcript() -> (Vec<RaftLogEntry>, usize) {
        let mut ops = Vec::new();
        ops.push(RaftLogEntry::CreateCollection {
            collection_id: "alpha".into(),
            req: schema(),
        });
        ops.push(RaftLogEntry::CreateCollection {
            collection_id: "beta".into(),
            req: schema(),
        });
        // Base docs (these end up under the segment checkpoint at S).
        let base = [
            ("d0", 1.0, "a", "red", true, 0u64, [0.1f32, 0.2, 0.3, 0.4]),
            ("d1", 3.0, "b", "blue", true, 3, [0.9, 0.8, 0.7, 0.6]),
            ("d2", 5.0, "a", "red", false, 7, [0.5, 0.5, 0.5, 0.5]),
            ("d3", 7.0, "c", "green", true, 1, [0.2, 0.4, 0.6, 0.8]),
        ];
        for (eid, n, kw, tag, tok, sig, emb) in base {
            ops.push(index_entry("alpha", eid, n, kw, tag, tok, sig, &emb));
            ops.push(index_entry(
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            ));
        }
        // S = number of ops so far (the checkpoint boundary).
        let s = ops.len();
        // Tail docs (these end up only in the AOF, S+1..=A).
        let tail = [
            (
                "d4",
                2.5,
                "b",
                "red",
                true,
                0u64,
                [0.11f32, 0.22, 0.33, 0.44],
            ),
            ("d5", 6.5, "a", "blue", true, 7, [0.6, 0.6, 0.6, 0.6]),
            ("d6", 8.5, "c", "green", false, 2, [0.3, 0.3, 0.3, 0.3]),
        ];
        for (eid, n, kw, tag, tok, sig, emb) in tail {
            ops.push(index_entry("alpha", eid, n, kw, tag, tok, sig, &emb));
            ops.push(index_entry(
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            ));
        }
        (ops, s)
    }

    #[test]
    fn rdb_plus_aof_recovery_matches_live_without_nats() {
        let (ops, s) = transcript();
        let a = ops.len(); // every op applied; A = total.
        let qa = [0.15f32, 0.25, 0.35, 0.45];

        let dir = tempfile::tempdir().unwrap();
        let seg_dir = dir.path().join("segments");
        std::fs::create_dir_all(&seg_dir).unwrap();
        let aof_path = dir.path().join("aof.log");

        // --- LIVE: apply 1..=A, append every op to the AOF, checkpoint at S. ---
        let live = Arc::new(Engine::new());
        let mut aof = AofWriter::open_with_policy(&aof_path, FsyncPolicy::Always).unwrap();
        for (i, op) in ops.iter().enumerate() {
            let seq = (i + 1) as u64;
            live.apply_raft_entry(op.clone()).unwrap();
            aof.append(seq, &WalRecord::new(op.clone())).unwrap();
            if seq == s as u64 {
                // RDB checkpoint at S, then trim the AOF through S — so on disk the
                // segment covers 1..=S and the AOF covers S+1..=A only.
                live.flush_to_segments(&seg_dir, s as u64).unwrap();
                aof.truncate_through(s as u64).unwrap();
            }
        }
        aof.sync().unwrap();

        // On-disk shape sanity: the AOF now holds exactly S+1..=A.
        let mut remaining = Vec::new();
        AofReader::replay(&aof_path, 0, |seq, _| remaining.push(seq)).unwrap();
        assert_eq!(
            remaining,
            ((s as u64 + 1)..=(a as u64)).collect::<Vec<_>>(),
            "AOF must hold exactly the post-checkpoint tail"
        );

        let live_alpha = battery(&live, "alpha");
        let live_beta = battery(&live, "beta");
        let live_knn_alpha = knn(&live, "alpha", &qa);
        let live_knn_beta = knn(&live, "beta", &qa);

        // --- RESTART: fresh engine, RDB reopen → AOF replay (no broker tail). ---
        let restarted = Arc::new(Engine::new());
        let s_recovered = restarted.reopen_from_segment_dir(&seg_dir).unwrap();
        assert_eq!(
            s_recovered, s as u64,
            "RDB must restore to the checkpoint seq S"
        );
        let a_recovered = replay_aof_into(&restarted, &aof_path, s_recovered).unwrap();
        assert_eq!(a_recovered, a as u64, "AOF replay must advance to A");

        // The restarted engine must be byte-identical to the live engine at A.
        assert_eq!(
            battery(&restarted, "alpha"),
            live_alpha,
            "alpha legs diverged after RDB+AOF recovery"
        );
        assert_eq!(
            battery(&restarted, "beta"),
            live_beta,
            "beta legs diverged after RDB+AOF recovery"
        );
        assert_eq!(
            knn(&restarted, "alpha", &qa),
            live_knn_alpha,
            "alpha kNN diverged after RDB+AOF recovery"
        );
        assert_eq!(
            knn(&restarted, "beta", &qa),
            live_knn_beta,
            "beta kNN diverged after RDB+AOF recovery"
        );
        assert_eq!(
            restarted.stats("alpha").unwrap().documents_indexed,
            live.stats("alpha").unwrap().documents_indexed
        );
        assert_eq!(
            restarted.stats("beta").unwrap().documents_indexed,
            live.stats("beta").unwrap().documents_indexed
        );
    }

    /// Recovery is robust to a torn AOF tail: a crash mid-append leaves a partial
    /// frame; recovery replays the good prefix and the engine still converges to
    /// the last DURABLE op (no panic, no divergence on the good prefix).
    #[test]
    fn recovery_tolerates_torn_aof_tail() {
        let (ops, s) = transcript();
        let qa = [0.15f32, 0.25, 0.35, 0.45];
        let dir = tempfile::tempdir().unwrap();
        let seg_dir = dir.path().join("segments");
        std::fs::create_dir_all(&seg_dir).unwrap();
        let aof_path = dir.path().join("aof.log");

        // Apply + append all but the LAST op durably; checkpoint at S.
        let live = Arc::new(Engine::new());
        let mut aof = AofWriter::open_with_policy(&aof_path, FsyncPolicy::Always).unwrap();
        let last = ops.len() - 1;
        for (i, op) in ops.iter().enumerate().take(last) {
            let seq = (i + 1) as u64;
            live.apply_raft_entry(op.clone()).unwrap();
            aof.append(seq, &WalRecord::new(op.clone())).unwrap();
            if seq == s as u64 {
                live.flush_to_segments(&seg_dir, s as u64).unwrap();
                aof.truncate_through(s as u64).unwrap();
            }
        }
        aof.sync().unwrap();
        let good_len = std::fs::metadata(&aof_path).unwrap().len();

        // Simulate a crash mid-append of the FINAL op: a header whose length
        // overruns EOF, plus a stray byte.
        {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&aof_path)
                .unwrap();
            let mut hdr = [0u8; HEADER_LEN];
            hdr[0..8].copy_from_slice(&((ops.len()) as u64).to_le_bytes());
            hdr[8..12].copy_from_slice(&999_999u32.to_le_bytes());
            f.write_all(&hdr).unwrap();
            f.write_all(&[0x42]).unwrap();
            f.sync_all().unwrap();
        }

        // The live oracle: exactly the durable prefix (NOT the torn final op).
        let live_alpha = battery(&live, "alpha");
        let live_beta = battery(&live, "beta");
        let live_knn = knn(&live, "alpha", &qa);

        // Recovery: RDB reopen → AOF replay. The torn tail is skipped cleanly.
        let restarted = Arc::new(Engine::new());
        let s_rec = restarted.reopen_from_segment_dir(&seg_dir).unwrap();
        assert_eq!(s_rec, s as u64);
        let a_rec = replay_aof_into(&restarted, &aof_path, s_rec).unwrap();
        assert_eq!(
            a_rec,
            (ops.len() - 1) as u64,
            "torn final frame must not be replayed"
        );

        assert_eq!(
            battery(&restarted, "alpha"),
            live_alpha,
            "alpha diverged after torn-tail recovery"
        );
        assert_eq!(
            battery(&restarted, "beta"),
            live_beta,
            "beta diverged after torn-tail recovery"
        );
        assert_eq!(
            knn(&restarted, "alpha", &qa),
            live_knn,
            "kNN diverged after torn-tail recovery"
        );

        // And the next writer open truncates the torn tail back to the prefix.
        let _w = AofWriter::open(&aof_path).unwrap();
        assert_eq!(std::fs::metadata(&aof_path).unwrap().len(), good_len);
    }
}
// CODEGEN-END
