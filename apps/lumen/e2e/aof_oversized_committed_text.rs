//! Black-box contract for borrowed committed oversized Text AOF replay.
//!
//! The fixture writes its normal and oversized frames through `AofWriter`.
//! The oversized `Index` has 1,000 Text strings. Each has 270 KiB of repeated
//! small `alpha` and `beta` words plus one short, unique `markerNNNN` word.
//! It therefore crosses the pending budget without a giant term or a giant
//! dictionary. The source `WalRecord` and AOF writer's temporary encoded bytes
//! exist only while the file is written. The replay and analytic reference
//! checks never create a second giant source. Cold open uses its real durable
//! segment layout after the checkpoint.
//!
//! # Facets
//!
//! - Behavior: aof_oversized_committed_text.rs:651-664 is the expected red:
//!   a valid real AOF Text frame must replay through its committed sequence.
//!   :372-486 and :673-764 require all marker IDs, small-word queries,
//!   document lengths, BM25 scores, a malformed covered prefix, AOF suffixes,
//!   checkpoints, and cold open.
//!   They cover apps/lumen/src/aof.rs:266-411,
//!   apps/lumen/src/storage/committed_index_apply.rs:166-217, and
//!   apps/lumen/src/storage/record_admission.rs:404-545.
//! - Security: aof_oversized_committed_text.rs:525-641 corrupts a complete,
//!   CRC-valid Text AOF frame that `AofWriter` first wrote. Its assertions at
//!   :608-640 require replay to refuse the bad frame, leave its value absent,
//!   and persist only the valid predecessor watermark. It covers the persisted
//!   byte boundary in apps/lumen/src/aof.rs:266-387.
//! - Performance: apps/lumen/docs/indexing.md:264-272 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   aof_oversized_committed_text.rs:506-523, called at :666-671, :683-688,
//!   :719-724, and :742-747, reads real public `/metrics` total and high-water
//!   gauges after replay, checkpoint, suffix, and cold open. The source fixture
//!   and mapped AOF bytes are outside those gauges.
//!   This contract sets no latency budget.
//!
//! # Root negative controls
//!
//! - After a Text route exists, change it to return `PlanResult::Unsupported`
//!   in apps/lumen/src/storage/committed_index_plan.rs:319-330. The behavior
//!   assertion at :651 must fail at the owned-decode reservation for the Text
//!   frame.
//! - Remove the borrowed Text replay route once it exists. The same replay
//!   assertion must fail before query or checkpoint assertions run.
//! - Make the complete-frame decoder accept the CRC-valid corrupt Text frame.
//!   The security assertion must fail. Never change the fixture threshold or
//!   an expected result to create a red. Record and restore each changed source
//!   file's SHA-256 around every mutation.
//!
//! Gate: cargo test -p lumen --test aof_oversized_committed_text -- --nocapture.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;

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
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, SearchHit, SearchRequest,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "oversized-aof-committed-text";
const FIELD: &str = "body";
const ITEM_COUNT: usize = 1_000;
const VALUE_BYTES: usize = 270 * 1024;
const PENDING_HARD_LIMIT_BYTES: usize = 256 * 1024 * 1024;
const BODY_CHUNK: &str = "alpha beta beta ";
const CHUNKS_PER_DOCUMENT: usize = VALUE_BYTES / BODY_CHUNK.len();
const ORIGINAL_DOC_LEN: u32 = (CHUNKS_PER_DOCUMENT * 3 + 1) as u32;
const UPDATED_TEXT: &str = "updated alpha alpha";
const UPDATED_DOC_LEN: u32 = 3;
const ALPHA_TF: u32 = (CHUNKS_PER_DOCUMENT * 1) as u32;
const BETA_TF: u32 = (CHUNKS_PER_DOCUMENT * 2) as u32;
const CREATE_SEQUENCE: u64 = 1;
const INDEX_SEQUENCE: u64 = 2;
const UPDATE_SEQUENCE: u64 = 3;
const DELETE_SEQUENCE: u64 = 4;
const UPDATED_ORDINAL: usize = 0;
const DELETED_ORDINAL: usize = 1;
const FRAME_HEADER_BYTES: u64 = 16;

/// Supplies the actual replayed watermark to the read-only metrics router.
/// `submit` must stay closed. This avoids a fresh coordinator that would
/// initialize the same Engine's capture barrier at sequence zero.
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

fn text_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([(
            FIELD.to_owned(),
            FieldSpec {
                field_type: FieldType::Text,
                analyzer: Some(Analyzer::WhitespaceLower),
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        )]),
    }
}

fn external_id(ordinal: usize) -> String {
    format!("oversized-aof-text-{ordinal:04}")
}

fn marker(ordinal: usize) -> String {
    format!("marker{ordinal:04}")
}

fn original_text(ordinal: usize) -> String {
    let mut text = BODY_CHUNK.repeat(CHUNKS_PER_DOCUMENT);
    text.push_str(&marker(ordinal));
    assert!(
        text.len() >= VALUE_BYTES,
        "fixture Text value must contain at least 270 KiB of repeated small words",
    );
    assert_eq!(
        text.split_whitespace().count(),
        ORIGINAL_DOC_LEN as usize,
        "fixture Text value must have its declared fixed document length",
    );
    text
}

fn create_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: text_schema(),
    }
}

fn oversized_text_record() -> WalRecord {
    assert!(
        ITEM_COUNT * VALUE_BYTES > PENDING_HARD_LIMIT_BYTES,
        "fixture source must exceed the documented 256 MiB pending budget",
    );
    let mut items = Vec::with_capacity(ITEM_COUNT);
    for ordinal in 0..ITEM_COUNT {
        items.push(IndexItem {
            external_id: external_id(ordinal),
            field: FIELD.to_owned(),
            value: FieldValue::String(original_text(ordinal)),
            version: None,
        });
    }
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
}

fn append_real_oversized_prefix(path: &Path) -> u64 {
    let mut writer = AofWriter::open(path).expect("open real Text AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append Text schema through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync real Text AOF schema");
    let oversized_frame_start = fs::metadata(path)
        .expect("measure real Text AOF schema prefix")
        .len();
    let record = oversized_text_record();
    writer
        .append(INDEX_SEQUENCE, &record)
        .expect("append oversized Text Index through AofWriter large-frame path");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync real oversized Text AOF prefix");
    drop(writer);
    assert!(
        fs::metadata(path)
            .expect("read real oversized Text AOF size")
            .len()
            > PENDING_HARD_LIMIT_BYTES as u64,
        "the actual AofWriter frame must exceed the 256 MiB pending budget",
    );
    oversized_frame_start
}

fn append_real_update_and_delete(path: &Path) {
    let mut writer = AofWriter::open(path).expect("reopen real Text AOF for suffix");
    writer
        .append(
            UPDATE_SEQUENCE,
            &WalRecord::new(RaftLogEntry::Index {
                collection_id: COLLECTION.to_owned(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: external_id(UPDATED_ORDINAL),
                        field: FIELD.to_owned(),
                        value: FieldValue::String(UPDATED_TEXT.to_owned()),
                        version: None,
                    }],
                    request_id: None,
                },
            }),
        )
        .expect("append real Text update AOF suffix");
    writer
        .append(
            DELETE_SEQUENCE,
            &WalRecord::new(RaftLogEntry::Delete {
                collection_id: COLLECTION.to_owned(),
                external_id: external_id(DELETED_ORDINAL),
                field: None,
            }),
        )
        .expect("append real Text delete AOF suffix");
    writer
        .sync_strict()
        .expect("strict-sync real Text AOF suffix");
}

fn match_hits(engine: &Engine, text: &str, limit: u32) -> Vec<SearchHit> {
    engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Match(MatchQuery {
                    field: FIELD.to_owned(),
                    text: text.to_owned(),
                    op: MatchOp::And,
                }),
                limit,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .unwrap_or_else(|error| panic!("Text query `{text}` failed: {error}"))
        .hits
}

fn scores_by_id(engine: &Engine, text: &str) -> BTreeMap<String, f32> {
    let mut scores = BTreeMap::new();
    for hit in match_hits(engine, text, ITEM_COUNT as u32 + 1) {
        assert!(
            scores.insert(hit.external_id.clone(), hit.score).is_none(),
            "Text query `{text}` must not return a duplicated external ID: {}",
            hit.external_id,
        );
    }
    scores
}

fn reference_bm25(n: u32, df: u32, tf: u32, doc_len: u32, average_doc_len: f32) -> f32 {
    const K1: f32 = 1.2;
    const B: f32 = 0.75;
    let n = n as f32;
    let df = df as f32;
    let tf = tf as f32;
    let doc_len = doc_len as f32;
    let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
    idf * tf * (K1 + 1.0) / (tf + K1 * (1.0 - B + B * doc_len / average_doc_len))
}

fn assert_score_close(actual: f32, expected: f32, context: &str) {
    let tolerance = expected.abs().max(1.0) * 1.0e-5;
    assert!(
        (actual - expected).abs() <= tolerance,
        "{context}: BM25 score drifted from the fixed-corpus reference: actual={actual}, expected={expected}, tolerance={tolerance}",
    );
}

fn assert_average_doc_len(actual: f32, expected: f32, context: &str) {
    let tolerance = 1.0e-3;
    assert!(
        (actual - expected).abs() <= tolerance,
        "{context}: Text avg_doc_len drifted from the fixed-corpus reference: actual={actual}, expected={expected}, tolerance={tolerance}",
    );
}

fn assert_text_stats(
    engine: &Engine,
    documents: u64,
    unique_terms: u64,
    average_doc_len: f32,
    phase: &str,
) {
    let stats = engine.stats(COLLECTION).expect("Text collection stats");
    assert_eq!(
        stats.documents_indexed, documents,
        "{phase}: Text documents_indexed must match the committed live rows",
    );
    let field = stats.fields.get(FIELD).expect("Text field stats");
    assert_eq!(field.field_type, FieldType::Text, "{phase}: field stays Text");
    assert_eq!(
        field.unique_terms, unique_terms,
        "{phase}: repeated words and compact markers must retain the expected dictionary",
    );
    let actual = field.avg_doc_len.expect("Text stats expose avg_doc_len");
    assert_average_doc_len(actual, average_doc_len, phase);
}

fn assert_single_text_hit(
    engine: &Engine,
    text: &str,
    expected_id: &str,
    expected_score: f32,
    phase: &str,
) {
    let hits = match_hits(engine, text, 2);
    assert_eq!(
        hits.len(),
        1,
        "{phase}: Text query `{text}` must return one exact marker row",
    );
    assert_eq!(
        hits[0].external_id, expected_id,
        "{phase}: Text query `{text}` must retain its exact external ID",
    );
    assert_score_close(hits[0].score, expected_score, phase);
}

fn assert_word_scores(
    engine: &Engine,
    text: &str,
    expected: &BTreeMap<String, f32>,
    phase: &str,
) {
    let actual = scores_by_id(engine, text);
    assert_eq!(
        actual.len(),
        expected.len(),
        "{phase}: Text query `{text}` returned the wrong live document count",
    );
    for (external_id, expected_score) in expected {
        let actual_score = actual.get(external_id).unwrap_or_else(|| {
            panic!("{phase}: Text query `{text}` lost external ID `{external_id}`")
        });
        assert_score_close(
            *actual_score,
            *expected_score,
            &format!("{phase}: Text query `{text}` for `{external_id}`"),
        );
    }
}

fn assert_initial_text_state(engine: &Engine, phase: &str) {
    let doc_count = ITEM_COUNT as u32;
    let average_doc_len = ORIGINAL_DOC_LEN as f32;
    assert_text_stats(
        engine,
        ITEM_COUNT as u64,
        (ITEM_COUNT + 2) as u64,
        average_doc_len,
        phase,
    );
    let marker_score = reference_bm25(doc_count, 1, 1, ORIGINAL_DOC_LEN, average_doc_len);
    for ordinal in 0..ITEM_COUNT {
        assert_single_text_hit(
            engine,
            &marker(ordinal),
            &external_id(ordinal),
            marker_score,
            phase,
        );
    }

    let mut alpha = BTreeMap::new();
    let alpha_score = reference_bm25(
        doc_count,
        doc_count,
        ALPHA_TF,
        ORIGINAL_DOC_LEN,
        average_doc_len,
    );
    let beta_score = reference_bm25(
        doc_count,
        doc_count,
        BETA_TF,
        ORIGINAL_DOC_LEN,
        average_doc_len,
    );
    for ordinal in 0..ITEM_COUNT {
        alpha.insert(external_id(ordinal), alpha_score);
    }
    let mut beta = BTreeMap::new();
    for ordinal in 0..ITEM_COUNT {
        beta.insert(external_id(ordinal), beta_score);
    }
    assert_word_scores(engine, "alpha", &alpha, phase);
    assert_word_scores(engine, "beta", &beta, phase);
}

fn tail_average_doc_len() -> f32 {
    let total = (ITEM_COUNT as u64 - 2) * ORIGINAL_DOC_LEN as u64 + UPDATED_DOC_LEN as u64;
    total as f32 / (ITEM_COUNT as u64 - 1) as f32
}

fn assert_tail_text_state(engine: &Engine, phase: &str) {
    let doc_count = ITEM_COUNT as u32 - 1;
    let average_doc_len = tail_average_doc_len();
    assert_text_stats(
        engine,
        doc_count as u64,
        (ITEM_COUNT + 1) as u64,
        average_doc_len,
        phase,
    );

    let updated_score = reference_bm25(doc_count, 1, 1, UPDATED_DOC_LEN, average_doc_len);
    assert_single_text_hit(
        engine,
        "updated",
        &external_id(UPDATED_ORDINAL),
        updated_score,
        phase,
    );
    let updated_alpha_score = reference_bm25(doc_count, doc_count, 2, UPDATED_DOC_LEN, average_doc_len);
    assert_single_text_hit(
        engine,
        "updated alpha",
        &external_id(UPDATED_ORDINAL),
        updated_score + updated_alpha_score,
        phase,
    );
    assert!(
        scores_by_id(engine, &marker(UPDATED_ORDINAL)).is_empty(),
        "{phase}: replaced Text marker must not revive",
    );
    assert!(
        scores_by_id(engine, &marker(DELETED_ORDINAL)).is_empty(),
        "{phase}: deleted Text marker must not revive",
    );

    let marker_score = reference_bm25(doc_count, 1, 1, ORIGINAL_DOC_LEN, average_doc_len);
    let original_alpha_score = reference_bm25(
        doc_count,
        doc_count,
        ALPHA_TF,
        ORIGINAL_DOC_LEN,
        average_doc_len,
    );
    let beta_score = reference_bm25(
        doc_count,
        doc_count - 1,
        BETA_TF,
        ORIGINAL_DOC_LEN,
        average_doc_len,
    );
    let mut alpha = BTreeMap::new();
    let mut beta = BTreeMap::new();
    alpha.insert(external_id(UPDATED_ORDINAL), updated_alpha_score);
    for ordinal in 2..ITEM_COUNT {
        let external_id = external_id(ordinal);
        assert_single_text_hit(engine, &marker(ordinal), &external_id, marker_score, phase);
        alpha.insert(external_id.clone(), original_alpha_score);
        beta.insert(external_id, beta_score);
    }
    assert_word_scores(engine, "alpha", &alpha, phase);
    assert_word_scores(engine, "beta", &beta, phase);
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

async fn assert_public_pending_budget(engine: Arc<Engine>, applied_sequence: u64, phase: &str) {
    let writer: Arc<dyn WriteSink> = Arc::new(ReadOnlyMetricsSink { applied_sequence });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), writer);
    let server = TestServer::new(router(state)).expect("open read-only public metrics server");
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES as u64,
        "{phase}: public pending-change total must stay within the approved 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES as u64,
        "{phase}: public pending-change high water must stay within the approved 256 MiB budget: high_water={high_water}",
    );
}

fn corrupt_complete_frame_payload(path: &Path, frame_start: u64) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open authentic AOF frame for corruption");
    file.seek(SeekFrom::Start(frame_start + 8))
        .expect("seek authentic AOF frame length");
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)
        .expect("read authentic AOF frame length");
    let payload_bytes = u32::from_le_bytes(length) as usize;
    assert!(payload_bytes > 0, "security fixture frame has a payload");
    let payload_start = frame_start + FRAME_HEADER_BYTES;
    let corrupt_at = payload_start + payload_bytes as u64 - 1;
    file.seek(SeekFrom::Start(corrupt_at))
        .expect("seek final authentic Text payload byte");
    let mut byte = [0_u8; 1];
    file.read_exact(&mut byte)
        .expect("read final authentic Text payload byte");
    byte[0] = if byte[0] == 0xff { 0xfe } else { 0xff };
    file.seek(SeekFrom::Start(corrupt_at))
        .expect("rewind final authentic Text payload byte");
    file.write_all(&byte)
        .expect("corrupt one authentic Text payload byte");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = payload_bytes;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic Text payload for CRC");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len());
        file.read_exact(&mut buffer[..read_len])
            .expect("stream authentic Text payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len;
    }
    file.seek(SeekFrom::Start(frame_start + 12))
        .expect("seek authentic Text frame CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite authentic Text frame CRC");
    file.sync_all()
        .expect("sync complete corrupted authentic Text frame");
}

fn assert_complete_corrupt_text_frame_is_refused(root: &Path) {
    let path = root.join("complete-corrupt-text-frame.aof");
    let mut writer = AofWriter::open(&path).expect("open security Text AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append valid security Text schema");
    writer
        .sync_strict()
        .expect("strict-sync valid security Text schema");
    let corrupt_frame_start = fs::metadata(&path)
        .expect("measure authentic Text prefix")
        .len();
    writer
        .append(
            INDEX_SEQUENCE,
            &WalRecord::new(RaftLogEntry::Index {
                collection_id: COLLECTION.to_owned(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "must-not-publish".to_owned(),
                        field: FIELD.to_owned(),
                        value: FieldValue::String("mustnotpublish alpha".to_owned()),
                        version: None,
                    }],
                    request_id: None,
                },
            }),
        )
        .expect("append authentic security Text frame through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync authentic security Text frame");
    drop(writer);
    corrupt_complete_frame_payload(&path, corrupt_frame_start);

    let engine = Arc::new(Engine::new());
    let refusal = replay_aof_into(&engine, &path, 0);
    assert!(
        refusal.is_err(),
        "a CRC-valid but corrupted complete Text AOF frame must be refused before publication",
    );
    assert!(
        scores_by_id(&engine, "mustnotpublish").is_empty(),
        "a refused complete Text frame must not become query-visible",
    );
    let store = SegmentRdbStore::new(root.join("complete-corrupt-text-segments"))
        .expect("open security Text segment store");
    let saved = store
        .save_with_sequence(&engine, 0)
        .expect("save only the replayed valid Text prefix");
    assert_eq!(
        saved, CREATE_SEQUENCE,
        "a refused complete Text frame must not advance the durable checkpoint watermark",
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open valid security Text prefix")
        .expect("valid security Text prefix checkpoint exists");
    assert_eq!(
        cold.sequence, CREATE_SEQUENCE,
        "cold-open must retain only the valid Text AOF prefix watermark",
    );
    assert_eq!(
        cold.engine
            .stats(COLLECTION)
            .expect("schema survives security Text replay")
            .documents_indexed,
        0,
        "the refused Text frame's row must stay absent after cold reopen",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_committed_text_aof_replays_suffixes_checkpoints_and_cold_opens() {
    let root = tempfile::tempdir().expect("oversized committed Text AOF fixture root");
    let aof_path = root.path().join("oversized-text.aof");
    let oversized_frame_start = append_real_oversized_prefix(&aof_path);

    let prefix_engine = Arc::new(Engine::new());
    let prefix_replay = replay_aof_into(&prefix_engine, &aof_path, 0);
    assert!(
        prefix_replay.is_ok(),
        "a valid real 1,000-by-270-KiB Text AOF frame must replay without owned decode refusal: {}",
        prefix_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        prefix_replay.expect("behavior assertion above checked Text prefix replay"),
        INDEX_SEQUENCE,
        "valid oversized Text prefix replay must advance through its committed Index sequence",
    );
    assert_initial_text_state(&prefix_engine, "initial oversized Text AOF replay");
    assert_public_pending_budget(
        prefix_engine.clone(),
        INDEX_SEQUENCE,
        "initial oversized Text AOF replay",
    )
    .await;

    let store = SegmentRdbStore::new(root.path().join("segments"))
        .expect("open Text segment store for oversized AOF checkpoint");
    let prefix_checkpoint = store
        .save_with_sequence(&prefix_engine, INDEX_SEQUENCE)
        .expect("checkpoint fully replayed oversized Text prefix");
    assert_eq!(
        prefix_checkpoint, INDEX_SEQUENCE,
        "Text prefix checkpoint must retain the committed oversized AOF watermark",
    );
    assert_initial_text_state(&prefix_engine, "oversized Text prefix checkpoint");
    assert_public_pending_budget(
        prefix_engine.clone(),
        INDEX_SEQUENCE,
        "oversized Text prefix checkpoint",
    )
    .await;

    drop(prefix_engine);
    append_real_update_and_delete(&aof_path);
    corrupt_complete_frame_payload(&aof_path, oversized_frame_start);
    let prefix_cold = store
        .load_current_generation()
        .expect("cold-open oversized Text prefix checkpoint")
        .expect("oversized Text prefix checkpoint must publish CURRENT");
    assert_eq!(
        prefix_cold.sequence, INDEX_SEQUENCE,
        "cold Text prefix checkpoint must retain its exact AOF watermark",
    );
    assert_initial_text_state(&prefix_cold.engine, "cold oversized Text prefix checkpoint");

    let suffix_replay = replay_aof_into(&prefix_cold.engine, &aof_path, prefix_cold.sequence);
    assert!(
        suffix_replay.is_ok(),
        "valid Text update and delete AOF suffix must replay after the giant prefix: {}",
        suffix_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        suffix_replay.expect("behavior assertion above checked Text suffix replay"),
        DELETE_SEQUENCE,
        "Text suffix replay must advance to the final committed sequence",
    );
    assert_tail_text_state(&prefix_cold.engine, "Text AOF suffix after cold checkpoint");
    assert_public_pending_budget(
        prefix_cold.engine.clone(),
        DELETE_SEQUENCE,
        "Text AOF suffix after cold checkpoint",
    )
    .await;

    let final_checkpoint = store
        .save_with_sequence(&prefix_cold.engine, DELETE_SEQUENCE)
        .expect("checkpoint Text update and delete suffix");
    assert_eq!(
        final_checkpoint, DELETE_SEQUENCE,
        "final Text checkpoint must retain the completed AOF suffix watermark",
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open final oversized Text checkpoint")
        .expect("final oversized Text checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, DELETE_SEQUENCE,
        "final cold Text checkpoint must retain the full AOF watermark",
    );
    assert_tail_text_state(&cold.engine, "final Text cold reopen");
    assert_public_pending_budget(
        cold.engine.clone(),
        DELETE_SEQUENCE,
        "final Text cold reopen",
    )
    .await;

    let covered_replay = replay_aof_into(&cold.engine, &aof_path, cold.sequence);
    assert!(
        covered_replay.is_ok(),
        "fully checkpoint-covered Text AOF frames must be skipped without replay failure: {}",
        covered_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        covered_replay.expect("behavior assertion above checked covered Text replay"),
        0,
        "fully checkpoint-covered Text AOF frames must not advance the watermark",
    );
    assert_tail_text_state(&cold.engine, "fully covered Text AOF replay");

    assert_complete_corrupt_text_frame_is_refused(root.path());
}
