//! Black-box contract for borrowed oversized mixed-field committed AOF replay.
//!
//! The fixture writes its schema, one oversized committed Index, and its later
//! update/delete suffix through the real `AofWriter`. The Index has eight items:
//! four fields for each of two documents. One Keyword string is 264 MiB. Text
//! stays small (`alpha beta beta`), while Number and Set use ordinary values.
//! The Set deliberately has repeated values. The large source `WalRecord` is
//! dropped after `AofWriter` writes it. Queries use only a small Keyword prefix,
//! so they never reconstruct the 264 MiB source value.
//!
//! # Facets
//!
//! - Behavior: aof_oversized_committed_mixed.rs:777-790 is the expected red:
//!   a valid mixed Text, Keyword, Number, and Set AOF frame must replay through
//!   its committed sequence. :446-591 and :792-897 require every field's query
//!   semantics, Text BM25 and average document length against an independent
//!   small reference, strict suffix replay, checkpoints, cold open, and no
//!   revival after update/delete. They cover apps/lumen/src/aof.rs:294-311,
//!   apps/lumen/src/storage/committed_index_apply.rs:166-233,
//!   apps/lumen/src/storage/committed_text_apply.rs:12-27, and
//!   apps/lumen/src/storage/text_preparation.rs:124-158.
//! - Security: aof_oversized_committed_mixed.rs:698-767 corrupts one byte in a
//!   complete, CRC-valid mixed frame which `AofWriter` first wrote. Its
//!   assertions at :729-765 require refusal, no query-visible row, and only the valid
//!   predecessor watermark after cold open. It covers the persisted-byte input
//!   boundary at apps/lumen/src/aof.rs:274-387.
//! - Performance: apps/lumen/docs/indexing.md:264-272 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   aof_oversized_committed_mixed.rs:633-650, called at :793-798, :809-814,
//!   :852-857, and :875-880, reads real public `/metrics` total and high-water
//!   gauges and requires both to stay within that budget. The fixture source
//!   allocation and mapped AOF bytes are outside these gauges. This contract
//!   sets no latency budget.
//!
//! # Root negative controls
//!
//! - After mixed borrowed routing exists, bypass its post-scalar fallback.
//!   Do not mutate the scalar planner's `Unsupported` result: that is already
//!   the current path. The replay assertion at :777 must fail at owned decode.
//! - Where the mixed fallback passes staged Text rows to normal apply, pass
//!   `None` instead. The independent Text assertions at :446-491 must fail.
//! - Make the complete-frame decoder accept the CRC-valid corrupt frame. The
//!   security assertion at :729 must fail. Restore every changed source file by
//!   SHA-256; never change the fixture threshold or expected field results.
//!
//! Gate: cargo test -p lumen --test aof_oversized_committed_mixed -- --nocapture.

use std::collections::{BTreeMap, BTreeSet};
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
    MatchOp, MatchQuery, PrefixQuery, QueryNode, SearchHit, SearchRequest, TermQuery,
    MAX_INDEX_BATCH_SIZE,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "oversized-aof-committed-mixed";
const TEXT_FIELD: &str = "body";
const KEYWORD_FIELD: &str = "title";
const NUMBER_FIELD: &str = "price";
const SET_FIELD: &str = "tags";
const PRIMARY_ID: &str = "mixed-primary";
const REFERENCE_ID: &str = "mixed-reference";
const HUGE_KEYWORD_BYTES: usize = 264 * 1024 * 1024;
const PENDING_HARD_LIMIT_BYTES: usize = 256 * 1024 * 1024;
const HUGE_KEYWORD_PREFIX: &str = "mixed-huge-keyword-prefix:";
const REFERENCE_KEYWORD_PREFIX: &str = "mixed-reference-keyword:";
const UPDATED_KEYWORD_PREFIX: &str = "mixed-updated-keyword:";
const PRIMARY_TEXT: &str = "alpha beta beta";
const REFERENCE_TEXT: &str = "alpha";
const UPDATED_TEXT: &str = "updated alpha alpha";
const PRIMARY_PRICE: f64 = 41.5;
const REFERENCE_PRICE: f64 = 9.5;
const UPDATED_PRICE: f64 = 77.0;
const CREATE_SEQUENCE: u64 = 1;
const INDEX_SEQUENCE: u64 = 2;
const UPDATE_SEQUENCE: u64 = 3;
const DELETE_SEQUENCE: u64 = 4;
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

fn field_spec(field_type: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
    FieldSpec {
        field_type,
        analyzer,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn mixed_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([
            (
                TEXT_FIELD.to_owned(),
                field_spec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
            ),
            (
                KEYWORD_FIELD.to_owned(),
                field_spec(FieldType::Keyword, None),
            ),
            (NUMBER_FIELD.to_owned(), field_spec(FieldType::Number, None)),
            (SET_FIELD.to_owned(), field_spec(FieldType::Set, None)),
        ]),
    }
}

fn create_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: mixed_schema(),
    }
}

fn item(external_id: &str, field: &str, value: FieldValue) -> IndexItem {
    IndexItem {
        external_id: external_id.to_owned(),
        field: field.to_owned(),
        value,
        version: None,
    }
}

fn huge_keyword() -> String {
    assert!(
        HUGE_KEYWORD_PREFIX.len() < HUGE_KEYWORD_BYTES,
        "the small prefix must leave a real 264 MiB Keyword value",
    );
    let mut bytes = vec![b'x'; HUGE_KEYWORD_BYTES];
    bytes[..HUGE_KEYWORD_PREFIX.len()].copy_from_slice(HUGE_KEYWORD_PREFIX.as_bytes());
    String::from_utf8(bytes).expect("giant ASCII Keyword fixture is valid UTF-8")
}

fn initial_items(primary_keyword: String) -> Vec<IndexItem> {
    let items = vec![
        item(
            PRIMARY_ID,
            TEXT_FIELD,
            FieldValue::String(PRIMARY_TEXT.to_owned()),
        ),
        item(
            PRIMARY_ID,
            KEYWORD_FIELD,
            FieldValue::String(primary_keyword),
        ),
        item(PRIMARY_ID, NUMBER_FIELD, FieldValue::Number(PRIMARY_PRICE)),
        item(
            PRIMARY_ID,
            SET_FIELD,
            FieldValue::StringList(vec!["red".to_owned(), "red".to_owned(), "blue".to_owned()]),
        ),
        item(
            REFERENCE_ID,
            TEXT_FIELD,
            FieldValue::String(REFERENCE_TEXT.to_owned()),
        ),
        item(
            REFERENCE_ID,
            KEYWORD_FIELD,
            FieldValue::String(format!("{REFERENCE_KEYWORD_PREFIX}small")),
        ),
        item(
            REFERENCE_ID,
            NUMBER_FIELD,
            FieldValue::Number(REFERENCE_PRICE),
        ),
        item(
            REFERENCE_ID,
            SET_FIELD,
            FieldValue::StringList(vec!["green".to_owned(), "green".to_owned()]),
        ),
    ];
    assert!(
        items.len() <= MAX_INDEX_BATCH_SIZE,
        "mixed fixture must stay within the public 1,000-item Index limit",
    );
    items
}

fn oversized_mixed_record() -> WalRecord {
    assert!(
        HUGE_KEYWORD_BYTES > PENDING_HARD_LIMIT_BYTES,
        "the one giant Keyword must exceed the documented 256 MiB pending budget",
    );
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: initial_items(huge_keyword()),
            request_id: None,
        },
    })
}

fn small_initial_request() -> IndexRequest {
    IndexRequest {
        items: initial_items(format!("{HUGE_KEYWORD_PREFIX}reference")),
        request_id: None,
    }
}

fn update_request() -> IndexRequest {
    let items = vec![
        item(
            PRIMARY_ID,
            TEXT_FIELD,
            FieldValue::String(UPDATED_TEXT.to_owned()),
        ),
        item(
            PRIMARY_ID,
            KEYWORD_FIELD,
            FieldValue::String(format!("{UPDATED_KEYWORD_PREFIX}small")),
        ),
        item(PRIMARY_ID, NUMBER_FIELD, FieldValue::Number(UPDATED_PRICE)),
        item(
            PRIMARY_ID,
            SET_FIELD,
            FieldValue::StringList(vec!["yellow".to_owned(), "yellow".to_owned()]),
        ),
    ];
    assert!(
        items.len() <= MAX_INDEX_BATCH_SIZE,
        "mixed update fixture must stay within the public 1,000-item Index limit",
    );
    IndexRequest {
        items,
        request_id: None,
    }
}

fn append_real_oversized_prefix(path: &Path) -> u64 {
    let mut writer = AofWriter::open(path).expect("open real mixed AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append mixed schema through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync real mixed AOF schema");
    let oversized_frame_start = fs::metadata(path)
        .expect("measure real mixed AOF schema prefix")
        .len();
    let record = oversized_mixed_record();
    writer
        .append(INDEX_SEQUENCE, &record)
        .expect("append oversized mixed Index through AofWriter large-frame path");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync real oversized mixed AOF prefix");
    drop(writer);
    assert!(
        fs::metadata(path)
            .expect("read real oversized mixed AOF size")
            .len()
            > PENDING_HARD_LIMIT_BYTES as u64,
        "the actual AofWriter frame must exceed the 256 MiB pending budget",
    );
    assert_fast_index_magic(path, oversized_frame_start);
    oversized_frame_start
}

fn assert_fast_index_magic(path: &Path, frame_start: u64) {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .expect("open real mixed AOF frame for fast-codec check");
    file.seek(SeekFrom::Start(frame_start + FRAME_HEADER_BYTES))
        .expect("seek real mixed Index payload magic");
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic)
        .expect("read real mixed Index payload magic");
    assert_eq!(
        &magic, b"LWAL",
        "the oversized AOF fixture must use the real fast Index wire codec",
    );
}

fn append_real_update_and_delete(path: &Path) {
    let mut writer = AofWriter::open(path).expect("reopen real mixed AOF for suffix");
    writer
        .append(
            UPDATE_SEQUENCE,
            &WalRecord::new(RaftLogEntry::Index {
                collection_id: COLLECTION.to_owned(),
                req: update_request(),
            }),
        )
        .expect("append real mixed update AOF suffix");
    writer
        .append(
            DELETE_SEQUENCE,
            &WalRecord::new(RaftLogEntry::Delete {
                collection_id: COLLECTION.to_owned(),
                external_id: REFERENCE_ID.to_owned(),
                field: None,
            }),
        )
        .expect("append real mixed delete AOF suffix");
    writer
        .sync_strict()
        .expect("strict-sync real mixed AOF suffix");
}

fn search_request(query: QueryNode) -> SearchRequest {
    SearchRequest {
        query,
        limit: 16,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn search_ids(engine: &Engine, query: QueryNode, label: &str) -> Vec<String> {
    let response = engine
        .search(COLLECTION, search_request(query))
        .unwrap_or_else(|error| panic!("{label} query failed: {error}"));
    let mut ids = BTreeSet::new();
    for hit in response.hits {
        assert!(
            ids.insert(hit.external_id.clone()),
            "{label} query must not return duplicate external ID {}",
            hit.external_id,
        );
    }
    ids.into_iter().collect()
}

fn term_ids(engine: &Engine, field: &str, value: FieldValue) -> Vec<String> {
    search_ids(
        engine,
        QueryNode::Term(TermQuery {
            field: field.to_owned(),
            value,
        }),
        field,
    )
}

fn prefix_ids(engine: &Engine, prefix: &str) -> Vec<String> {
    search_ids(
        engine,
        QueryNode::Prefix(PrefixQuery {
            field: KEYWORD_FIELD.to_owned(),
            value: prefix.to_owned(),
        }),
        "Keyword prefix",
    )
}

fn match_scores(engine: &Engine, text: &str) -> BTreeMap<String, f32> {
    let response = engine
        .search(
            COLLECTION,
            search_request(QueryNode::Match(MatchQuery {
                field: TEXT_FIELD.to_owned(),
                text: text.to_owned(),
                op: MatchOp::And,
            })),
        )
        .unwrap_or_else(|error| panic!("Text query `{text}` failed: {error}"));
    let mut scores = BTreeMap::new();
    for SearchHit {
        external_id, score, ..
    } in response.hits
    {
        assert!(
            scores.insert(external_id.clone(), score).is_none(),
            "Text query `{text}` must not return duplicate external ID {external_id}",
        );
    }
    scores
}

fn assert_ids(actual: Vec<String>, expected: &[&str], phase: &str) {
    let expected = expected
        .iter()
        .map(|id| (*id).to_owned())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{phase}: returned wrong external IDs");
}

fn assert_score_maps_equal(
    actual: BTreeMap<String, f32>,
    reference: BTreeMap<String, f32>,
    query: &str,
    phase: &str,
) {
    assert_eq!(
        actual.len(),
        reference.len(),
        "{phase}: Text query `{query}` returned the wrong document count",
    );
    for (external_id, reference_score) in reference {
        let actual_score = actual
            .get(&external_id)
            .unwrap_or_else(|| panic!("{phase}: Text query `{query}` lost `{external_id}`"));
        let tolerance = reference_score.abs().max(1.0) * 1.0e-5;
        assert!(
            (*actual_score - reference_score).abs() <= tolerance,
            "{phase}: Text BM25 score drifted for `{query}` / `{external_id}`: actual={actual_score}, reference={reference_score}, tolerance={tolerance}",
        );
    }
}

fn assert_text_equivalent(actual: &Engine, reference: &Engine, phase: &str) {
    let actual_stats = actual
        .stats(COLLECTION)
        .expect("mixed actual Text collection stats");
    let reference_stats = reference
        .stats(COLLECTION)
        .expect("mixed independent reference Text collection stats");
    assert_eq!(
        actual_stats.documents_indexed, reference_stats.documents_indexed,
        "{phase}: Text document count must match the independent small reference",
    );
    let actual_field = actual_stats
        .fields
        .get(TEXT_FIELD)
        .expect("actual Text field stats");
    let reference_field = reference_stats
        .fields
        .get(TEXT_FIELD)
        .expect("reference Text field stats");
    assert_eq!(
        actual_field.field_type,
        FieldType::Text,
        "{phase}: mixed body field must remain Text",
    );
    assert_eq!(
        actual_field.unique_terms, reference_field.unique_terms,
        "{phase}: Text dictionary must match the independent small reference",
    );
    let actual_average = actual_field
        .avg_doc_len
        .expect("actual Text stats expose avg_doc_len");
    let reference_average = reference_field
        .avg_doc_len
        .expect("reference Text stats expose avg_doc_len");
    assert!(
        (actual_average - reference_average).abs() <= 1.0e-3,
        "{phase}: Text avg_doc_len drifted from independent small reference: actual={actual_average}, reference={reference_average}",
    );
    for query in ["alpha", "beta", "updated"] {
        assert_score_maps_equal(
            match_scores(actual, query),
            match_scores(reference, query),
            query,
            phase,
        );
    }
}

fn assert_initial_state(engine: &Engine, reference: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("mixed initial stats")
            .documents_indexed,
        2,
        "{phase}: both committed mixed documents must be visible",
    );
    assert_text_equivalent(engine, reference, phase);
    assert_ids(
        prefix_ids(engine, HUGE_KEYWORD_PREFIX),
        &[PRIMARY_ID],
        &format!("{phase}: huge Keyword prefix"),
    );
    assert_ids(
        prefix_ids(engine, REFERENCE_KEYWORD_PREFIX),
        &[REFERENCE_ID],
        &format!("{phase}: reference Keyword prefix"),
    );
    assert_ids(
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(PRIMARY_PRICE)),
        &[PRIMARY_ID],
        &format!("{phase}: primary Number"),
    );
    assert_ids(
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(REFERENCE_PRICE)),
        &[REFERENCE_ID],
        &format!("{phase}: reference Number"),
    );
    assert_ids(
        term_ids(engine, SET_FIELD, FieldValue::String("red".to_owned())),
        &[PRIMARY_ID],
        &format!("{phase}: duplicate Set element"),
    );
    assert_ids(
        term_ids(engine, SET_FIELD, FieldValue::String("blue".to_owned())),
        &[PRIMARY_ID],
        &format!("{phase}: second primary Set element"),
    );
    assert_ids(
        term_ids(engine, SET_FIELD, FieldValue::String("green".to_owned())),
        &[REFERENCE_ID],
        &format!("{phase}: reference Set element"),
    );
}

fn assert_tail_state(engine: &Engine, reference: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("mixed tail stats")
            .documents_indexed,
        1,
        "{phase}: the deleted reference document must stay absent",
    );
    assert_text_equivalent(engine, reference, phase);
    assert_ids(
        prefix_ids(engine, HUGE_KEYWORD_PREFIX),
        &[],
        &format!("{phase}: replaced giant Keyword"),
    );
    assert_ids(
        prefix_ids(engine, REFERENCE_KEYWORD_PREFIX),
        &[],
        &format!("{phase}: deleted reference Keyword"),
    );
    assert_ids(
        prefix_ids(engine, UPDATED_KEYWORD_PREFIX),
        &[PRIMARY_ID],
        &format!("{phase}: updated Keyword"),
    );
    assert_ids(
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(PRIMARY_PRICE)),
        &[],
        &format!("{phase}: replaced Number"),
    );
    assert_ids(
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(REFERENCE_PRICE)),
        &[],
        &format!("{phase}: deleted Number"),
    );
    assert_ids(
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(UPDATED_PRICE)),
        &[PRIMARY_ID],
        &format!("{phase}: updated Number"),
    );
    for tag in ["red", "blue", "green"] {
        assert_ids(
            term_ids(engine, SET_FIELD, FieldValue::String(tag.to_owned())),
            &[],
            &format!("{phase}: retired Set `{tag}`"),
        );
    }
    assert_ids(
        term_ids(engine, SET_FIELD, FieldValue::String("yellow".to_owned())),
        &[PRIMARY_ID],
        &format!("{phase}: updated duplicate Set element"),
    );
}

fn small_reference_engine() -> Arc<Engine> {
    let engine = Arc::new(Engine::new());
    engine
        .create_collection(COLLECTION, mixed_schema())
        .expect("create independent small mixed reference collection");
    engine
        .index(COLLECTION, small_initial_request())
        .expect("index independent small mixed reference rows");
    engine
}

fn apply_reference_tail(reference: &Engine) {
    reference
        .index(COLLECTION, update_request())
        .expect("apply small mixed reference update");
    reference
        .delete(COLLECTION, REFERENCE_ID, None)
        .expect("apply small mixed reference delete");
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

fn corrupt_complete_frame_payload(path: &Path, frame_start: u64, payload_offset: u64) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open authentic mixed AOF frame for corruption");
    file.seek(SeekFrom::Start(frame_start + 8))
        .expect("seek authentic mixed AOF frame length");
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)
        .expect("read authentic mixed AOF frame length");
    let payload_bytes = u32::from_le_bytes(length) as u64;
    assert!(
        payload_bytes > payload_offset,
        "security fixture payload has target byte"
    );
    let payload_start = frame_start + FRAME_HEADER_BYTES;
    let corrupt_at = payload_start + payload_offset;
    file.seek(SeekFrom::Start(corrupt_at))
        .expect("seek authentic mixed payload byte");
    let mut byte = [0_u8; 1];
    file.read_exact(&mut byte)
        .expect("read authentic mixed payload byte");
    byte[0] ^= 0xff;
    file.seek(SeekFrom::Start(corrupt_at))
        .expect("rewind authentic mixed payload byte");
    file.write_all(&byte)
        .expect("corrupt one authentic mixed payload byte");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = payload_bytes as usize;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic mixed payload for CRC");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len());
        file.read_exact(&mut buffer[..read_len])
            .expect("stream authentic mixed payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len;
    }
    file.seek(SeekFrom::Start(frame_start + 12))
        .expect("seek authentic mixed frame CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite authentic mixed frame CRC");
    file.sync_all()
        .expect("sync complete corrupted authentic mixed frame");
}

fn assert_complete_corrupt_mixed_frame_is_refused(root: &Path) {
    let path = root.join("complete-corrupt-mixed-frame.aof");
    let mut writer = AofWriter::open(&path).expect("open security mixed AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append valid security mixed schema");
    writer
        .sync_strict()
        .expect("strict-sync valid security mixed schema");
    let corrupt_frame_start = fs::metadata(&path)
        .expect("measure authentic mixed AOF prefix")
        .len();
    let record = WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: small_initial_request(),
    });
    writer
        .append(INDEX_SEQUENCE, &record)
        .expect("append authentic security mixed frame through AofWriter");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync authentic security mixed frame");
    drop(writer);
    assert_fast_index_magic(&path, corrupt_frame_start);
    // Byte zero is LWAL's magic. The CRC is refreshed below, so replay must
    // refuse the format rather than accepting a checksum failure as a success.
    corrupt_complete_frame_payload(&path, corrupt_frame_start, 0);

    let engine = Arc::new(Engine::new());
    let refusal = replay_aof_into(&engine, &path, 0);
    assert!(
        refusal.is_err(),
        "a CRC-valid but corrupt complete mixed AOF frame must be refused before publication",
    );
    assert_ids(
        prefix_ids(&engine, HUGE_KEYWORD_PREFIX),
        &[],
        "refused mixed AOF frame",
    );
    assert!(
        match_scores(&engine, "alpha").is_empty(),
        "a refused complete mixed frame must not become Text-query-visible",
    );
    let store = SegmentRdbStore::new(root.join("complete-corrupt-mixed-segments"))
        .expect("open security mixed segment store");
    let saved = store
        .save_with_sequence(&engine, 0)
        .expect("save only the replayed valid mixed prefix");
    assert_eq!(
        saved, CREATE_SEQUENCE,
        "a refused complete mixed frame must not advance the durable checkpoint watermark",
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open valid security mixed prefix")
        .expect("valid security mixed prefix checkpoint exists");
    assert_eq!(
        cold.sequence, CREATE_SEQUENCE,
        "cold-open must retain only the valid mixed AOF prefix watermark",
    );
    assert_eq!(
        cold.engine
            .stats(COLLECTION)
            .expect("schema survives security mixed replay")
            .documents_indexed,
        0,
        "the refused mixed frame's rows must stay absent after cold reopen",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_committed_mixed_aof_replays_suffixes_checkpoints_and_cold_opens() {
    let root = tempfile::tempdir().expect("oversized committed mixed AOF fixture root");
    let aof_path = root.path().join("oversized-mixed.aof");
    let oversized_frame_start = append_real_oversized_prefix(&aof_path);

    let prefix_engine = Arc::new(Engine::new());
    let prefix_replay = replay_aof_into(&prefix_engine, &aof_path, 0);
    assert!(
        prefix_replay.is_ok(),
        "a valid retained mixed fast Index AOF frame must replay; current refusal: {}",
        prefix_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        prefix_replay.expect("behavior assertion above checked mixed replay result"),
        INDEX_SEQUENCE,
        "valid oversized mixed prefix replay must advance through its committed Index sequence",
    );
    let reference = small_reference_engine();
    assert_initial_state(
        &prefix_engine,
        &reference,
        "initial oversized mixed AOF replay",
    );
    assert_public_pending_budget(
        prefix_engine.clone(),
        INDEX_SEQUENCE,
        "initial oversized mixed AOF replay",
    )
    .await;

    let store = SegmentRdbStore::new(root.path().join("segments"))
        .expect("open segment store for oversized mixed AOF checkpoint");
    let prefix_checkpoint = store
        .save_with_sequence(&prefix_engine, INDEX_SEQUENCE)
        .expect("checkpoint the fully replayed oversized mixed AOF prefix");
    assert_eq!(
        prefix_checkpoint, INDEX_SEQUENCE,
        "prefix checkpoint must retain the committed mixed AOF watermark",
    );
    assert_public_pending_budget(
        prefix_engine.clone(),
        INDEX_SEQUENCE,
        "oversized mixed AOF prefix checkpoint",
    )
    .await;

    append_real_update_and_delete(&aof_path);
    // The checkpoint covers this huge frame. Its CRC-valid bad byte proves that
    // strict suffix replay must skip the covered payload before it parses it.
    corrupt_complete_frame_payload(&aof_path, oversized_frame_start, 0);

    let prefix_cold = store
        .load_current_generation()
        .expect("cold-open oversized mixed AOF prefix checkpoint")
        .expect("oversized mixed AOF prefix checkpoint must publish CURRENT");
    assert_eq!(
        prefix_cold.sequence, INDEX_SEQUENCE,
        "cold prefix checkpoint must keep its exact mixed AOF watermark",
    );
    assert_initial_state(
        &prefix_cold.engine,
        &reference,
        "cold oversized mixed AOF prefix checkpoint",
    );

    let suffix_replay = replay_aof_into(&prefix_cold.engine, &aof_path, prefix_cold.sequence);
    assert!(
        suffix_replay.is_ok(),
        "AOF replay from the checkpoint watermark must skip the CRC-valid malformed covered mixed prefix and apply its strict later suffix: {}",
        suffix_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        suffix_replay.expect("behavior assertion above checked mixed suffix replay"),
        DELETE_SEQUENCE,
        "AOF replay must apply only sequences strictly greater than the checkpoint watermark",
    );
    apply_reference_tail(&reference);
    assert_tail_state(
        &prefix_cold.engine,
        &reference,
        "strict mixed AOF suffix after checkpoint",
    );
    assert_public_pending_budget(
        prefix_cold.engine.clone(),
        DELETE_SEQUENCE,
        "strict mixed AOF suffix replay",
    )
    .await;

    let final_checkpoint = store
        .save_with_sequence(&prefix_cold.engine, DELETE_SEQUENCE)
        .expect("checkpoint mixed update and delete after oversized AOF replay");
    assert_eq!(
        final_checkpoint, DELETE_SEQUENCE,
        "final checkpoint must retain the completed mixed AOF suffix watermark",
    );
    let final_cold = store
        .load_current_generation()
        .expect("cold-open final oversized mixed AOF checkpoint")
        .expect("final oversized mixed AOF checkpoint must publish CURRENT");
    assert_eq!(
        final_cold.sequence, DELETE_SEQUENCE,
        "final cold checkpoint must retain the full mixed AOF watermark",
    );
    assert_tail_state(&final_cold.engine, &reference, "final mixed cold reopen");
    assert_public_pending_budget(
        final_cold.engine.clone(),
        DELETE_SEQUENCE,
        "final mixed AOF cold reopen",
    )
    .await;

    let covered_replay = replay_aof_into(&final_cold.engine, &aof_path, final_cold.sequence);
    assert!(
        covered_replay.is_ok(),
        "fully checkpoint-covered mixed AOF frames must be skipped without replay failure: {}",
        covered_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        covered_replay.expect("behavior assertion above checked covered mixed replay"),
        0,
        "fully checkpoint-covered mixed AOF frames must not advance the watermark",
    );
    assert_tail_state(
        &final_cold.engine,
        &reference,
        "fully covered mixed AOF replay",
    );

    assert_complete_corrupt_mixed_frame_is_refused(root.path());
}
