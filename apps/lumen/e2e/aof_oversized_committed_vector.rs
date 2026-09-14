//! Black-box contract for borrowed oversized committed Vector AOF replay.
//!
//! The child writes one real fast `Index` record with exactly 1,000 public
//! items: 950 raw `flat-cpu` vectors and 48 `hnsw-cpu` vectors with scalar
//! quantization, plus two Keyword values. Every vector has 70,000 valid f32
//! coordinates. Together their wire values exceed the documented 256 MiB
//! pending-change budget, while no single vector uses an invented dimension
//! limit. The record is written through `AofWriter`, dropped, then replayed
//! from its mapped frame.
//!
//! The Flat assertion uses an exact top-one ID. The HNSW/SQ assertions compare
//! only the complete live-ID set. They do not claim a stable approximate order,
//! score, or quantized f32 round trip. The fixture first checkpoints a small
//! Vector base, so the oversized record must also survive an incremental
//! checkpoint and cold reopen. A later versioned Flat update, stale retry, and
//! full HNSW document delete prove that later AOF records do not revive data.
//!
//! The behavior replay is isolated in a child. A baseline owned decode can
//! refuse or remain admitted behind the global budget. The parent kills and
//! reaps that child at a bounded cleanup deadline. This is cleanup only, not a
//! latency promise. The child receives a parent-owned temporary workspace via
//! `TMPDIR`, `TEMP`, and `TMP` so a killed fallback owner cannot leave files.
//!
//! # Facets
//!
//! - Behavior: aof_oversized_committed_vector.rs:876-923 establishes and
//!   cold-opens a mixed Vector base; :927-1064 replays the valid >256 MiB AOF
//!   suffix, checks raw Flat and HNSW/SQ state, checkpoints it, then applies a
//!   versioned update and delete; :1127-1139 makes a stuck child red. These
//!   assertions cover apps/lumen/src/aof.rs:285-315, vector wire borrowing in
//!   apps/lumen/src/wal/fast_index_scanner.rs:230-246, and Vector storage in
//!   apps/lumen/src/vector_index.rs:383-431 and :687-850.
//! - Security: aof_oversized_committed_vector.rs:825-854 corrupts one complete,
//!   CRC-valid Vector/Keyword AOF frame after a valid prefix and requires an
//!   error before its rows or watermark become visible. It covers the
//!   process-written AOF byte boundary in apps/lumen/src/aof.rs:279-315.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   aof_oversized_committed_vector.rs:707-725 reads public pending total and
//!   high-water metrics after replay and checkpoint. Fixture vectors and the
//!   mapped AOF frame are outside that accounting. This contract sets no
//!   latency budget.
//!
//! # Root negative control
//!
//! After the borrowed Vector route exists, remove its Vector planner routing so
//! the scanner falls back to generic owned decoding. The child oversized-replay
//! assertion below must then fail through the owned-decode refusal or its
//! cleanup watchdog. Restore the correct source hash before any other gate.
//!
//! Gate: cargo test -p lumen --test aof_oversized_committed_vector -- --nocapture.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum_test::TestServer;

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::WriteSink;
use lumen::log_entry::RaftLogEntry;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::{ApplyOutcome, Engine};
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    KnnQuery, PrefixQuery, QueryNode, SearchRequest, VectorBackend, VectorMetric,
    VectorQuantize, MAX_INDEX_BATCH_SIZE,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "oversized-aof-committed-vector";
const FLAT_FIELD: &str = "flat_raw";
const HNSW_FIELD: &str = "hnsw_sq";
const KEYWORD_FIELD: &str = "kind";
const FLAT_BASE_ID: &str = "flat-base";
const HNSW_BASE_ID: &str = "hnsw-base";
const MIXED_ID: &str = "mixed-keyword";
const BASE_KEYWORD_PREFIX: &str = "oversized-vector-base-";
const GIANT_KEYWORD_PREFIX: &str = "oversized-vector-giant-";
const UPDATED_KEYWORD_PREFIX: &str = "oversized-vector-updated-";
const DELETED_KEYWORD_PREFIX: &str = "oversized-vector-delete-";
const FLAT_ROWS: usize = 950;
const HNSW_ROWS: usize = 48;
const VECTOR_ITEMS: usize = FLAT_ROWS + HNSW_ROWS;
const VECTOR_DIM: usize = 70_000;
const VECTOR_CODE_BITS: usize = 10;
const BASE_VECTOR_CODE: usize = (1 << VECTOR_CODE_BITS) - 1;
const UPDATED_VECTOR_CODE: usize = BASE_VECTOR_CODE - 1;
const VECTOR_WIRE_BYTES: u64 = VECTOR_ITEMS as u64 * VECTOR_DIM as u64 * 4;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const FRAME_HEADER_BYTES: u64 = 16;
const CREATE_SEQUENCE: u64 = 1;
const BASE_SEQUENCE: u64 = 2;
const OVERSIZED_SEQUENCE: u64 = 3;
const UPDATE_SEQUENCE: u64 = 4;
const DELETE_SEQUENCE: u64 = 5;
const BASE_VERSION: u64 = 10;
const OVERSIZED_VERSION: u64 = 20;
const UPDATED_VERSION: u64 = 30;
const REPLAY_WATCHDOG: Duration = Duration::from_secs(240);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_AOF_OVERSIZED_VECTOR_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_AOF_OVERSIZED_VECTOR_ROOT";
const CHILD_AOF_ENV: &str = "LUMEN_AOF_OVERSIZED_VECTOR_AOF";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_AOF_OVERSIZED_VECTOR_HANDSHAKE";
const CHILD_CASE: &str = "replay";
const TEST_NAME: &str =
    "oversized_committed_vector_aof_replays_flat_hnsw_sq_checkpoints_and_cold_opens";

/// Supplies the actual replay watermark without constructing a fresh
/// coordinator, which would initialize the Engine at sequence zero.
struct ReadOnlyMetricsSink {
    applied_sequence: u64,
}

#[async_trait]
impl WriteSink for ReadOnlyMetricsSink {
    async fn submit(&self, _entry: RaftLogEntry) -> anyhow::Result<ApplyOutcome> {
        anyhow::bail!("the read-only metrics server must not submit a mutation");
    }

    fn applied_seq(&self) -> u64 {
        self.applied_sequence
    }
}

/// Owns an isolated replay process until it has exited. A baseline admission
/// loop cannot leak a live child or its process-global budget into another test.
struct ChildCleanup(Option<Child>);

impl Drop for ChildCleanup {
    fn drop(&mut self) {
        let Some(child) = self.0.as_mut() else {
            return;
        };
        if !matches!(child.try_wait(), Ok(Some(_))) {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
}

#[derive(Clone, Copy)]
struct FrameSpan {
    start: u64,
}

fn field_spec(field_type: FieldType) -> FieldSpec {
    FieldSpec {
        field_type,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn vector_spec(backend: VectorBackend, quantize: Option<VectorQuantize>) -> FieldSpec {
    FieldSpec {
        field_type: FieldType::Vector,
        analyzer: None,
        multi: None,
        dim: Some(VECTOR_DIM as u32),
        metric: Some(VectorMetric::L2),
        backend: Some(backend),
        quantize,
    }
}

fn schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([
            (
                FLAT_FIELD.to_owned(),
                vector_spec(VectorBackend::FlatCpu, None),
            ),
            (
                HNSW_FIELD.to_owned(),
                vector_spec(VectorBackend::HnswCpu, Some(VectorQuantize::Sq)),
            ),
            (KEYWORD_FIELD.to_owned(), field_spec(FieldType::Keyword)),
        ]),
    }
}

fn create_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: schema(),
    }
}

fn item(
    external_id: impl Into<String>,
    field: &str,
    value: FieldValue,
    version: Option<u64>,
) -> IndexItem {
    IndexItem {
        external_id: external_id.into(),
        field: field.to_owned(),
        value,
        version,
    }
}

/// Encodes a small binary identity in a wide valid vector. The high empty
/// dimensions make this a realistic many-row wire-size fixture, while the
/// ten leading coordinates give distinct, exactly representable SQ values.
fn vector_for(code: usize) -> Vec<f32> {
    assert!(
        code < (1 << VECTOR_CODE_BITS),
        "fixture vector identity must fit its deterministic binary coordinates",
    );
    let mut vector = vec![0.0; VECTOR_DIM];
    for bit in 0..VECTOR_CODE_BITS {
        vector[bit] = ((code >> bit) & 1) as f32;
    }
    vector
}

fn flat_id(row: usize) -> String {
    format!("flat-{row:04}")
}

fn hnsw_id(row: usize) -> String {
    format!("hnsw-{row:04}")
}

fn base_record() -> WalRecord {
    let items = vec![
        item(
            FLAT_BASE_ID,
            FLAT_FIELD,
            FieldValue::Vector(vector_for(BASE_VECTOR_CODE)),
            Some(BASE_VERSION),
        ),
        item(
            HNSW_BASE_ID,
            HNSW_FIELD,
            FieldValue::Vector(vector_for(BASE_VECTOR_CODE)),
            Some(BASE_VERSION),
        ),
        item(
            MIXED_ID,
            KEYWORD_FIELD,
            FieldValue::String(format!("{BASE_KEYWORD_PREFIX}row")),
            Some(BASE_VERSION),
        ),
    ];
    assert!(
        items.len() <= MAX_INDEX_BATCH_SIZE,
        "the pre-existing Vector base must stay within the public 1,000-item Index limit",
    );
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
}

/// One valid record exercises both the raw Flat and SQ HNSW backends. The
/// two Keyword items make the maximum 1,000-item fast Index mixed-field too.
fn oversized_vector_record() -> WalRecord {
    assert!(
        VECTOR_WIRE_BYTES > PENDING_HARD_LIMIT_BYTES,
        "the many-row Vector fixture must exceed the documented 256 MiB budget",
    );
    let mut items = Vec::with_capacity(MAX_INDEX_BATCH_SIZE);
    for row in 0..FLAT_ROWS {
        items.push(item(
            flat_id(row),
            FLAT_FIELD,
            FieldValue::Vector(vector_for(row)),
            Some(OVERSIZED_VERSION),
        ));
    }
    for row in 0..HNSW_ROWS {
        items.push(item(
            hnsw_id(row),
            HNSW_FIELD,
            FieldValue::Vector(vector_for(row)),
            Some(OVERSIZED_VERSION),
        ));
    }
    items.push(item(
        MIXED_ID,
        KEYWORD_FIELD,
        FieldValue::String(format!("{GIANT_KEYWORD_PREFIX}row")),
        Some(OVERSIZED_VERSION),
    ));
    items.push(item(
        hnsw_id(0),
        KEYWORD_FIELD,
        FieldValue::String(format!("{DELETED_KEYWORD_PREFIX}row")),
        Some(OVERSIZED_VERSION),
    ));
    assert_eq!(
        items.len(),
        MAX_INDEX_BATCH_SIZE,
        "the oversized Vector fixture must exercise, but never exceed, the public 1,000-item Index cap",
    );
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
}

/// Write a small committed Vector base before the oversized suffix. Its later
/// save is therefore an incremental preservation check rather than a first
/// full checkpoint only.
fn append_base_prefix(path: &Path) {
    let mut writer = AofWriter::open(path).expect("open real Vector AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append Vector schema through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync real Vector schema AOF frame");
    let base = base_record();
    writer
        .append(BASE_SEQUENCE, &base)
        .expect("append committed Vector base through AofWriter");
    drop(base);
    writer
        .sync_strict()
        .expect("strict-sync real committed Vector base AOF prefix");
}

/// The large record is made and dropped inside the child. The mapped AOF frame
/// is the only large source held during the behavior replay.
fn append_oversized_suffix(path: &Path) -> FrameSpan {
    let mut writer = AofWriter::open(path).expect("reopen real Vector AOF writer");
    let start = fs::metadata(path)
        .expect("measure pre-oversized Vector AOF prefix")
        .len();
    let record = oversized_vector_record();
    writer
        .append(OVERSIZED_SEQUENCE, &record)
        .expect("append oversized Vector Index through AofWriter large-frame path");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync real oversized Vector AOF suffix");
    drop(writer);
    let payload_bytes = frame_payload_bytes(path, start);
    assert!(
        payload_bytes > PENDING_HARD_LIMIT_BYTES,
        "the actual AofWriter Vector frame must exceed the documented 256 MiB pending budget: payload_bytes={payload_bytes}",
    );
    assert!(
        payload_bytes >= VECTOR_WIRE_BYTES,
        "the actual AofWriter Vector frame must retain every raw f32 wire value: payload_bytes={payload_bytes}, vector_bytes={VECTOR_WIRE_BYTES}",
    );
    assert_fast_index_magic(path, start);
    FrameSpan { start }
}

fn append_later_update_and_delete(path: &Path) {
    let mut writer = AofWriter::open(path).expect("reopen real Vector AOF for suffix");
    let update = WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![
                item(
                    flat_id(0),
                    FLAT_FIELD,
                    FieldValue::Vector(vector_for(UPDATED_VECTOR_CODE)),
                    Some(UPDATED_VERSION),
                ),
                item(
                    flat_id(0),
                    FLAT_FIELD,
                    FieldValue::Vector(vector_for(0)),
                    Some(OVERSIZED_VERSION),
                ),
                item(
                    MIXED_ID,
                    KEYWORD_FIELD,
                    FieldValue::String(format!("{UPDATED_KEYWORD_PREFIX}row")),
                    Some(UPDATED_VERSION),
                ),
                item(
                    MIXED_ID,
                    KEYWORD_FIELD,
                    FieldValue::String(format!("{GIANT_KEYWORD_PREFIX}stale")),
                    Some(OVERSIZED_VERSION),
                ),
            ],
            request_id: None,
        },
    });
    writer
        .append(UPDATE_SEQUENCE, &update)
        .expect("append real versioned Vector update AOF suffix");
    drop(update);
    let delete = WalRecord::new(RaftLogEntry::Delete {
        collection_id: COLLECTION.to_owned(),
        external_id: hnsw_id(0),
        field: None,
    });
    writer
        .append(DELETE_SEQUENCE, &delete)
        .expect("append real HNSW Vector document delete AOF suffix");
    drop(delete);
    writer
        .sync_strict()
        .expect("strict-sync real Vector AOF suffix");
}

fn frame_payload_bytes(path: &Path, frame_start: u64) -> u64 {
    let mut file = File::open(path).expect("open real Vector AOF frame");
    file.seek(SeekFrom::Start(frame_start + 8))
        .expect("seek real Vector AOF frame length");
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)
        .expect("read real Vector AOF frame length");
    u64::from(u32::from_le_bytes(length))
}

fn assert_fast_index_magic(path: &Path, frame_start: u64) {
    let mut file = File::open(path).expect("open real Vector AOF for fast-codec check");
    file.seek(SeekFrom::Start(frame_start + FRAME_HEADER_BYTES))
        .expect("seek real Vector Index payload magic");
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic)
        .expect("read real Vector Index payload magic");
    assert_eq!(
        &magic, b"LWAL",
        "the oversized AOF fixture must use the real fast Index wire codec",
    );
}

/// The incremental checkpoint covers the large suffix. A CRC-valid but
/// semantically bad covered frame must be skipped without parsing when strict
/// suffix replay starts at the covered sequence.
fn poison_covered_prefix_frame(path: &Path, span: FrameSpan) {
    let payload_bytes = frame_payload_bytes(path, span.start);
    assert!(
        payload_bytes > PENDING_HARD_LIMIT_BYTES,
        "the covered Vector payload must still exceed the documented 256 MiB fixture threshold",
    );
    let payload_start = span
        .start
        .checked_add(FRAME_HEADER_BYTES)
        .expect("covered Vector payload offset fits u64");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open Vector AOF to poison its covered prefix");
    file.seek(SeekFrom::Start(payload_start))
        .expect("seek covered Vector AOF payload");
    let mut first = [0_u8; 1];
    file.read_exact(&mut first)
        .expect("read first covered Vector AOF payload byte");
    first[0] ^= 0xff;
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind covered Vector AOF payload");
    file.write_all(&first)
        .expect("poison one covered Vector AOF payload byte");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = payload_bytes;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind to stream covered Vector AOF payload");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..read_len])
            .expect("stream covered Vector payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len as u64;
    }
    file.seek(SeekFrom::Start(span.start + 12))
        .expect("seek covered Vector AOF CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite CRC for poisoned covered Vector AOF frame");
    file.sync_all()
        .expect("sync poisoned checkpoint-covered Vector AOF prefix");
}

fn search_request(query: QueryNode, limit: u32) -> SearchRequest {
    SearchRequest {
        query,
        limit,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn knn_ids(engine: &Engine, field: &str, vector: Vec<f32>, k: u32, label: &str) -> Vec<String> {
    engine
        .search(
            COLLECTION,
            search_request(
                QueryNode::Knn(KnnQuery {
                    field: field.to_owned(),
                    vector,
                    k,
                }),
                k,
            ),
        )
        .unwrap_or_else(|error| panic!("{label} query failed: {error}"))
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn knn_set(engine: &Engine, field: &str, vector: Vec<f32>, k: u32, label: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for id in knn_ids(engine, field, vector, k, label) {
        assert!(ids.insert(id.clone()), "{label} query must not return duplicate ID {id}");
    }
    ids
}

fn prefix_ids(engine: &Engine, prefix: &str, label: &str) -> BTreeSet<String> {
    let response = engine
        .search(
            COLLECTION,
            search_request(
                QueryNode::Prefix(PrefixQuery {
                    field: KEYWORD_FIELD.to_owned(),
                    value: prefix.to_owned(),
                }),
                16,
            ),
        )
        .unwrap_or_else(|error| panic!("{label} prefix query failed: {error}"));
    let mut ids = BTreeSet::new();
    for hit in response.hits {
        assert!(
            ids.insert(hit.external_id.clone()),
            "{label} prefix query must not return duplicate ID {}",
            hit.external_id,
        );
    }
    ids
}

fn expected_hnsw_ids(with_first_row: bool) -> BTreeSet<String> {
    let mut ids = BTreeSet::from([HNSW_BASE_ID.to_owned()]);
    for row in 0..HNSW_ROWS {
        if with_first_row || row != 0 {
            ids.insert(hnsw_id(row));
        }
    }
    ids
}

fn assert_flat_top(engine: &Engine, vector_code: usize, expected_id: &str, phase: &str) {
    assert_eq!(
        knn_ids(
            engine,
            FLAT_FIELD,
            vector_for(vector_code),
            1,
            "exact Flat raw top-one",
        ),
        vec![expected_id.to_owned()],
        "{phase}: the raw Flat vector must retain its exact nearest ID",
    );
}

fn assert_hnsw_live_set(engine: &Engine, expected: BTreeSet<String>, phase: &str) {
    let actual = knn_set(
        engine,
        HNSW_FIELD,
        vector_for(BASE_VECTOR_CODE),
        expected.len() as u32,
        "HNSW SQ complete live-ID set",
    );
    assert_eq!(
        actual, expected,
        "{phase}: HNSW/SQ must retain the complete live-ID set without claiming approximate order or scores",
    );
}

fn assert_base_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized Vector base stats")
            .documents_indexed,
        3,
        "{phase}: the small mixed Vector base must retain three documents",
    );
    assert_flat_top(engine, BASE_VECTOR_CODE, FLAT_BASE_ID, phase);
    assert_hnsw_live_set(engine, BTreeSet::from([HNSW_BASE_ID.to_owned()]), phase);
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX, "base mixed Keyword"),
        BTreeSet::from([MIXED_ID.to_owned()]),
        "{phase}: the committed mixed Keyword base must remain visible",
    );
    assert_eq!(
        prefix_ids(engine, GIANT_KEYWORD_PREFIX, "future giant Keyword"),
        BTreeSet::new(),
        "{phase}: the future oversized mixed Keyword must not appear in the base",
    );
    assert_eq!(
        prefix_ids(engine, DELETED_KEYWORD_PREFIX, "future delete Keyword"),
        BTreeSet::new(),
        "{phase}: the future deleted HNSW document must not appear in the base",
    );
}

fn assert_oversized_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized Vector incremental stats")
            .documents_indexed,
        3 + VECTOR_ITEMS as u64,
        "{phase}: the oversized record must retain its full Flat and HNSW corpus plus the base",
    );
    assert_flat_top(engine, 0, &flat_id(0), phase);
    assert_hnsw_live_set(engine, expected_hnsw_ids(true), phase);
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX, "old base Keyword"),
        BTreeSet::new(),
        "{phase}: the older mixed Keyword base must not revive after the oversized version wins",
    );
    assert_eq!(
        prefix_ids(engine, GIANT_KEYWORD_PREFIX, "giant mixed Keyword"),
        BTreeSet::from([MIXED_ID.to_owned()]),
        "{phase}: the oversized record's mixed Keyword must remain visible",
    );
    assert_eq!(
        prefix_ids(engine, DELETED_KEYWORD_PREFIX, "future deleted HNSW Keyword"),
        BTreeSet::from([hnsw_id(0)]),
        "{phase}: the later HNSW document must be visible before its delete",
    );
}

fn assert_tail_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized Vector tail stats")
            .documents_indexed,
        2 + VECTOR_ITEMS as u64,
        "{phase}: the full HNSW document delete must remove exactly one live document",
    );
    assert_flat_top(engine, UPDATED_VECTOR_CODE, &flat_id(0), phase);
    assert_hnsw_live_set(engine, expected_hnsw_ids(false), phase);
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX, "old base Keyword after tail"),
        BTreeSet::new(),
        "{phase}: the original mixed Keyword base must not revive after later versions",
    );
    assert_eq!(
        prefix_ids(engine, GIANT_KEYWORD_PREFIX, "stale giant Keyword after tail"),
        BTreeSet::new(),
        "{phase}: the stale lower-version mixed Keyword must not replace the update",
    );
    assert_eq!(
        prefix_ids(engine, UPDATED_KEYWORD_PREFIX, "updated mixed Keyword"),
        BTreeSet::from([MIXED_ID.to_owned()]),
        "{phase}: the higher-version mixed Keyword must remain visible",
    );
    assert_eq!(
        prefix_ids(engine, DELETED_KEYWORD_PREFIX, "deleted HNSW Keyword"),
        BTreeSet::new(),
        "{phase}: the deleted HNSW document's Keyword must not revive",
    );
}

fn metric_u64(metrics: &str, name: &str) -> u64 {
    let values = metrics
        .lines()
        .filter_map(|line| {
            let (metric, value) = line.split_once(|character: char| character.is_whitespace())?;
            (metric == name).then_some(value.trim())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        values.len(),
        1,
        "public /metrics must publish exactly one {name} sample: {metrics}",
    );
    values[0]
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{name} must be an unsigned byte value: {}", values[0]))
}

async fn assert_public_pending_budget(engine: Arc<Engine>, sequence: u64, phase: &str) {
    let sink: Arc<dyn WriteSink> = Arc::new(ReadOnlyMetricsSink {
        applied_sequence: sequence,
    });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink);
    let server = TestServer::new(router(state)).expect("open read-only public metrics server");
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change total must stay within the documented 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change high water must stay within the documented 256 MiB budget: high_water={high_water}",
    );
}

fn corrupt_complete_frame_payload(path: &Path, frame_start: u64) {
    let payload_bytes = frame_payload_bytes(path, frame_start);
    assert!(
        payload_bytes > 0,
        "security Vector payload must have a byte to corrupt",
    );
    let payload_start = frame_start + FRAME_HEADER_BYTES;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open authentic Vector AOF frame for corruption");
    file.seek(SeekFrom::Start(payload_start))
        .expect("seek authentic Vector AOF payload byte");
    let mut byte = [0_u8; 1];
    file.read_exact(&mut byte)
        .expect("read authentic Vector AOF payload byte");
    byte[0] ^= 0xff;
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic Vector AOF payload byte");
    file.write_all(&byte)
        .expect("corrupt one authentic Vector AOF payload byte");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = payload_bytes;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic Vector AOF payload for CRC");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..read_len])
            .expect("stream authentic Vector payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len as u64;
    }
    file.seek(SeekFrom::Start(frame_start + 12))
        .expect("seek authentic Vector AOF frame CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite authentic Vector AOF frame CRC");
    file.sync_all()
        .expect("sync complete corrupt Vector AOF frame");
}

fn corrupt_vector_keyword_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![
                item(
                    "corrupt-vector-row",
                    FLAT_FIELD,
                    FieldValue::Vector(vector_for(1)),
                    Some(UPDATED_VERSION),
                ),
                item(
                    "corrupt-vector-row",
                    KEYWORD_FIELD,
                    FieldValue::String("oversized-vector-corrupt-row".to_owned()),
                    Some(UPDATED_VERSION),
                ),
            ],
            request_id: None,
        },
    })
}

/// A valid mixed prefix followed by an authenticated malformed frame must
/// refuse the frame and retain only the prefix's visible state and watermark.
fn assert_complete_corrupt_vector_frame_is_refused(root: &Path) {
    let path = root.join("complete-corrupt-vector-frame.aof");
    let mut writer = AofWriter::open(&path).expect("open security Vector AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append valid security Vector schema");
    let base = base_record();
    writer
        .append(BASE_SEQUENCE, &base)
        .expect("append valid security Vector base");
    drop(base);
    writer
        .sync_strict()
        .expect("strict-sync valid security Vector prefix");
    let corrupt_frame_start = fs::metadata(&path)
        .expect("measure authentic security Vector prefix")
        .len();
    let corrupt = corrupt_vector_keyword_record();
    writer
        .append(OVERSIZED_SEQUENCE, &corrupt)
        .expect("append authentic mixed Vector/Keyword security frame");
    drop(corrupt);
    writer
        .sync_strict()
        .expect("strict-sync authentic mixed Vector/Keyword security frame");
    drop(writer);
    assert_fast_index_magic(&path, corrupt_frame_start);
    corrupt_complete_frame_payload(&path, corrupt_frame_start);

    let engine = Arc::new(Engine::new());
    let refusal = replay_aof_into(&engine, &path, 0);
    assert!(
        refusal.is_err(),
        "a CRC-valid but format-corrupt complete Vector/Keyword AOF frame must be refused before publication",
    );
    assert_base_state(&engine, "security valid Vector prefix after refused frame");
    assert_eq!(
        prefix_ids(&engine, "oversized-vector-corrupt-", "refused corrupt Keyword"),
        BTreeSet::new(),
        "a refused complete Vector/Keyword frame must not become Keyword-query-visible",
    );
    let store = SegmentRdbStore::new(root.join("complete-corrupt-vector-frame-segments"))
        .expect("open security Vector segment store");
    let saved = store
        .save_with_sequence(&engine, 0)
        .expect("save only the replayed valid Vector prefix");
    assert_eq!(
        saved, BASE_SEQUENCE,
        "a refused complete Vector/Keyword frame must not advance the durable checkpoint watermark",
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open valid security Vector prefix")
        .expect("valid security Vector prefix checkpoint exists");
    assert_eq!(
        cold.sequence, BASE_SEQUENCE,
        "cold-open must retain only the valid Vector prefix watermark",
    );
    assert_base_state(&cold.engine, "security cold Vector prefix after refused frame");
}

fn child_paths() -> Option<(PathBuf, PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("isolated Vector replay child needs its durable root");
    let aof = std::env::var_os(CHILD_AOF_ENV)
        .map(PathBuf::from)
        .expect("isolated Vector replay child needs its AOF path");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("isolated Vector replay child needs a handshake path");
    Some((root, aof, handshake))
}

async fn run_replay_child(root: PathBuf, aof_path: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact isolated Vector replay child entry");
    let base_engine = Arc::new(Engine::new());
    let base_replay = replay_aof_into(&base_engine, &aof_path, 0);
    assert!(
        base_replay.is_ok(),
        "the valid committed Vector base AOF prefix must replay before the oversized suffix: {}",
        base_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        base_replay.expect("behavior assertion above checked Vector base replay"),
        BASE_SEQUENCE,
        "the committed Vector base replay must advance through its exact sequence",
    );
    assert_base_state(&base_engine, "committed Vector base AOF replay");
    assert_public_pending_budget(
        base_engine.clone(),
        BASE_SEQUENCE,
        "committed Vector base AOF replay",
    )
    .await;

    let store = SegmentRdbStore::new(root.join("segments"))
        .expect("open segment store for oversized Vector AOF checkpoint");
    let base_checkpoint = store
        .save_with_sequence(&base_engine, BASE_SEQUENCE)
        .expect("checkpoint the committed Vector base before the oversized suffix");
    assert_eq!(
        base_checkpoint, BASE_SEQUENCE,
        "the initial Vector checkpoint must retain the established base watermark",
    );
    assert_public_pending_budget(
        base_engine.clone(),
        BASE_SEQUENCE,
        "committed Vector base checkpoint",
    )
    .await;

    let base_cold = store
        .load_current_generation()
        .expect("cold-open committed Vector base checkpoint")
        .expect("committed Vector base checkpoint must publish CURRENT");
    assert_eq!(
        base_cold.sequence, BASE_SEQUENCE,
        "cold Vector base checkpoint must keep its exact watermark",
    );
    assert_base_state(&base_cold.engine, "cold committed Vector base checkpoint");

    let span = append_oversized_suffix(&aof_path);
    let oversized_replay = replay_aof_into(&base_cold.engine, &aof_path, base_cold.sequence);
    assert!(
        oversized_replay.is_ok(),
        "a valid committed >256 MiB mixed Vector AOF suffix must replay without an owned decode refusal: {}",
        oversized_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        oversized_replay.expect("behavior assertion above checked oversized Vector replay"),
        OVERSIZED_SEQUENCE,
        "valid oversized Vector replay must advance through its committed suffix sequence",
    );
    assert_oversized_state(&base_cold.engine, "oversized mixed Vector AOF suffix replay");
    assert_public_pending_budget(
        base_cold.engine.clone(),
        OVERSIZED_SEQUENCE,
        "oversized mixed Vector AOF suffix replay",
    )
    .await;

    let incremental_checkpoint = store
        .save_with_sequence(&base_cold.engine, OVERSIZED_SEQUENCE)
        .expect("incrementally checkpoint the oversized Vector AOF suffix over its base");
    assert_eq!(
        incremental_checkpoint, OVERSIZED_SEQUENCE,
        "incremental Vector checkpoint must retain the oversized suffix watermark",
    );
    assert_public_pending_budget(
        base_cold.engine.clone(),
        OVERSIZED_SEQUENCE,
        "incremental oversized Vector checkpoint",
    )
    .await;

    let incremental_cold = store
        .load_current_generation()
        .expect("cold-open incremental oversized Vector checkpoint")
        .expect("incremental oversized Vector checkpoint must publish CURRENT");
    assert_eq!(
        incremental_cold.sequence, OVERSIZED_SEQUENCE,
        "cold incremental Vector checkpoint must keep its exact suffix watermark",
    );
    assert_oversized_state(
        &incremental_cold.engine,
        "cold incremental oversized Vector checkpoint",
    );
    assert_public_pending_budget(
        incremental_cold.engine.clone(),
        OVERSIZED_SEQUENCE,
        "cold incremental oversized Vector checkpoint",
    )
    .await;

    append_later_update_and_delete(&aof_path);
    poison_covered_prefix_frame(&aof_path, span);
    let suffix_replay = replay_aof_into(
        &incremental_cold.engine,
        &aof_path,
        incremental_cold.sequence,
    );
    assert!(
        suffix_replay.is_ok(),
        "AOF replay from the Vector checkpoint watermark must skip its CRC-valid malformed covered prefix and apply later records: {}",
        suffix_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        suffix_replay.expect("behavior assertion above checked Vector suffix replay"),
        DELETE_SEQUENCE,
        "AOF replay must apply only sequences strictly greater than the Vector checkpoint watermark",
    );
    assert_tail_state(
        &incremental_cold.engine,
        "strict Vector AOF suffix after incremental checkpoint",
    );
    assert_public_pending_budget(
        incremental_cold.engine.clone(),
        DELETE_SEQUENCE,
        "strict Vector AOF suffix replay",
    )
    .await;

    let final_checkpoint = store
        .save_with_sequence(&incremental_cold.engine, DELETE_SEQUENCE)
        .expect("checkpoint Vector update and delete after oversized AOF replay");
    assert_eq!(
        final_checkpoint, DELETE_SEQUENCE,
        "final Vector checkpoint must retain the completed AOF suffix watermark",
    );
    assert_public_pending_budget(
        incremental_cold.engine.clone(),
        DELETE_SEQUENCE,
        "final Vector checkpoint",
    )
    .await;
    let final_cold = store
        .load_current_generation()
        .expect("cold-open final oversized Vector AOF checkpoint")
        .expect("final oversized Vector checkpoint must publish CURRENT");
    assert_eq!(
        final_cold.sequence, DELETE_SEQUENCE,
        "final cold Vector checkpoint must retain the full AOF watermark",
    );
    assert_tail_state(&final_cold.engine, "final Vector cold reopen");
    assert_public_pending_budget(
        final_cold.engine.clone(),
        DELETE_SEQUENCE,
        "final oversized Vector AOF cold reopen",
    )
    .await;

    let covered_replay = replay_aof_into(&final_cold.engine, &aof_path, final_cold.sequence);
    assert!(
        covered_replay.is_ok(),
        "fully checkpoint-covered Vector AOF frames must be skipped without replay failure: {}",
        covered_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        covered_replay.expect("behavior assertion above checked covered Vector replay"),
        0,
        "fully checkpoint-covered Vector AOF frames must not advance the watermark",
    );
    assert_tail_state(&final_cold.engine, "fully covered Vector AOF replay");
    assert_public_pending_budget(
        final_cold.engine.clone(),
        DELETE_SEQUENCE,
        "fully checkpoint-covered Vector AOF replay",
    )
    .await;
}

async fn run_isolated_replay(root: &Path, aof_path: &Path) {
    let child_root = tempfile::tempdir().expect("isolated Vector replay child workspace");
    let child_tmp = child_root.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned Vector child temporary directory");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable = std::env::current_exe().expect("current oversized Vector test executable");
    let stdout = File::create(&stdout_path).expect("create isolated Vector replay child stdout");
    let stderr = File::create(&stderr_path).expect("create isolated Vector replay child stderr");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
        .env(CHILD_ROOT_ENV, root)
        .env(CHILD_AOF_ENV, aof_path)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("spawn isolated oversized Vector replay child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + REPLAY_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("Vector child remains owned until it exits")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child.0.take().expect("timed-out Vector child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("wait for killed oversized Vector replay child");
                let stdout = fs::read_to_string(&stdout_path)
                    .expect("read killed oversized Vector replay child stdout");
                let stderr = fs::read_to_string(&stderr_path)
                    .expect("read killed oversized Vector replay child stderr");
                panic!(
                    "a valid committed >256 MiB mixed Vector AOF frame must replay and checkpoint before test cleanup; isolated child was killed after {REPLAY_WATCHDOG:?}: status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated oversized Vector replay child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited Vector child remains owned")
        .wait()
        .expect("wait for exited oversized Vector replay child");
    let stdout = fs::read_to_string(&stdout_path).expect("read oversized Vector replay child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read oversized Vector replay child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated child did not enter exact {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must enter the intended oversized Vector replay body",
    );
    assert!(
        status.success(),
        "isolated oversized Vector replay child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_committed_vector_aof_replays_flat_hnsw_sq_checkpoints_and_cold_opens() {
    if let Some((root, aof_path, handshake)) = child_paths() {
        run_replay_child(root, aof_path, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("oversized committed Vector AOF fixture root");
    let aof_path = root.path().join("oversized-vector.aof");
    append_base_prefix(&aof_path);
    run_isolated_replay(root.path(), &aof_path).await;
    assert_complete_corrupt_vector_frame_is_refused(root.path());
}
