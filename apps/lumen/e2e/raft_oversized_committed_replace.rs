//! Black-box contract for a foreign committed oversized Raft `docs:replace` record.
//!
//! The child drives the public `RaftStateMachine::apply` callback on `EngineSm`.
//! It never calls proposal admission, an HTTP write route, or a private staging API.
//! First it applies a schema and a small full-document base, and cold-opens that
//! checkpoint. It then applies one generic-CBOR `ReplaceDocs` command with the
//! public maximum of 32 documents. Each document contains an approximately
//! 8.5 MiB Keyword plus small Number, Text, Set, and Hash fields. The command
//! exceeds the documented 256 MiB pending-change budget.
//!
//! The base puts an `obsolete` field on document zero. The oversized complete
//! replacement omits that field. The case checks exact complete Keyword values
//! for documents zero, 16, and 31; all Number and Hash values; one later
//! higher-version replacement; a stale retry; and a delete. It checkpoints and
//! cold-opens both the oversized state and the later final state. The same
//! retained generic-CBOR command is then malformed and must be refused without
//! changing the final watermark or visible data. The parent owns the child
//! workspace and `TMPDIR`, `TEMP`, and `TMP`. Its 240-second child bound is
//! cleanup only, not a product latency promise.
//!
//! # Facets
//!
//! - Behavior: assertions at `raft_oversized_committed_replace.rs:875-886`
//!   require the one foreign committed generic-CBOR batch to advance the Raft
//!   watermark once. Assertions at `:493-655`, `:908-948`, and `:894-906`
//!   require all 32 documents, complete Keyword bytes for rows 0, 16, and 31,
//!   Number and Hash values for every row, omitted-field deletion, version LWW,
//!   stale suppression, delete behavior, checkpoints, and cold reopen. These
//!   cover `apps/lumen/src/raft_sm.rs:223-302`,
//!   `apps/lumen/src/storage.rs:5536-5807`, and
//!   `apps/lumen/src/segment_rdb.rs:513-520`.
//! - Security: `raft_oversized_committed_replace.rs:953-964` feeds the same
//!   externally supplied generic-CBOR Raft command with an invalid root byte to
//!   `EngineSm::apply`, then requires refusal, no watermark advance, and no
//!   data mutation. It covers the peer-byte boundary at
//!   `apps/lumen/src/raft_sm.rs:223-302`. The child owns no caller-selected
//!   path; its only temporary paths are parent-created at `:967-992`.
//! - Performance: `apps/lumen/docs/indexing.md:264-276` says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   `raft_oversized_committed_replace.rs:712-737` proves the real generic-CBOR
//!   command is above 256 MiB and remains at the public 32-document maximum.
//!   `:676-695`, `:887-906`, and `:929-948` read the public pending total and
//!   high-water gauges and require both to remain within that budget. Fixture
//!   source bytes are external to those gauges.
//!
//! # Root negative control
//!
//! After the borrowed committed ReplaceDocs path exists, bypass
//! `try_apply_committed_replace_with_capacity_owner` in
//! `apps/lumen/src/raft_sm.rs` before it handles a foreign command. The
//! behavior assertion at `:875-886` must fail because the valid committed
//! command is refused or its watermark does not advance. Restore every source
//! byte by SHA-256. Never lower the 32-document fixture or its actual >256 MiB
//! generic-CBOR command assertion.
//!
//! Target gate: `cargo test -p lumen --features raft-wal --test
//! raft_oversized_committed_replace -- --nocapture`.
//! Full declared behavior gate: `cargo test -p lumen --features raft-wal`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum_test::TestServer;

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::WriteSink;
use lumen::log_entry::RaftLogEntry;
use lumen::raft_sm::EngineSm;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::{ApplyOutcome, Engine};
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, HammingQuery, MatchOp,
    MatchQuery, PrefixQuery, QueryNode, ReplaceDocItem, ReplaceDocsRequest, SearchRequest,
    TermQuery, MAX_BATCH_REPLACE_SIZE,
};
use lumen::wal::WalRecord;
use raft_runtime::{Index, RaftStateMachine};

const COLLECTION: &str = "raft-oversized-committed-replace";
const KEYWORD_FIELD: &str = "keyword";
const NUMBER_FIELD: &str = "number";
const TEXT_FIELD: &str = "body";
const SET_FIELD: &str = "tags";
const HASH_FIELD: &str = "hash";
const OBSOLETE_FIELD: &str = "obsolete";
const BASE_ID: &str = "replace-doc-00";
const DELETE_ID: &str = "replace-doc-31";
const DOC_COUNT: usize = 32;
const PER_DOCUMENT_KEYWORD_BYTES: usize = 8 * 1024 * 1024 + 512 * 1024 + 256;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const BASE_KEYWORD_PREFIX: &str = "replace-base-keyword-";
const OBSOLETE_PREFIX: &str = "replace-obsolete-field-";
const UPDATED_KEYWORD_PREFIX: &str = "replace-updated-keyword-";
const STALE_KEYWORD_PREFIX: &str = "replace-stale-keyword-";
const BASE_TEXT: &str = "replace-base-text";
const UPDATED_TEXT: &str = "replace-updated-text";
const STALE_TEXT: &str = "replace-stale-text";
const COMMON_TEXT: &str = "replace-common";
const SHARED_SET_VALUE: &str = "replace-shared-set";
const UPDATED_SET_VALUE: &str = "replace-updated-set";
const BASE_NUMBER: f64 = 1.0;
const GIANT_NUMBER_BASE: f64 = 1000.0;
const UPDATED_NUMBER: f64 = 9999.0;
const CREATE_SEQUENCE: Index = 1;
const BASE_SEQUENCE: Index = 2;
const GIANT_SEQUENCE: Index = 3;
const UPDATE_SEQUENCE: Index = 4;
const STALE_SEQUENCE: Index = 5;
const DELETE_SEQUENCE: Index = 6;
const MALFORMED_SEQUENCE: Index = 7;
const BASE_VERSION: u64 = 10;
const GIANT_VERSION: u64 = 20;
const UPDATED_VERSION: u64 = 30;
const CHILD_CLEANUP_BOUND: Duration = Duration::from_secs(240);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_RAFT_OVERSIZED_REPLACE_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_RAFT_OVERSIZED_REPLACE_ROOT";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_RAFT_OVERSIZED_REPLACE_HANDSHAKE";
const CHILD_CASE: &str = "foreign-committed-replace";
const TEST_NAME: &str = "raft_committed_oversized_replace_applies_checkpoints_and_cold_opens";

/// Supplies the actual committed Raft watermark without constructing a fresh
/// coordinator, which would initialize this Engine's capture barrier at zero.
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

/// Owns a child on every parent panic path. A killed child is reaped before its
/// parent-owned workspace is removed.
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

fn schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([
            (
                KEYWORD_FIELD.to_owned(),
                field_spec(FieldType::Keyword, None),
            ),
            (NUMBER_FIELD.to_owned(), field_spec(FieldType::Number, None)),
            (
                TEXT_FIELD.to_owned(),
                field_spec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
            ),
            (SET_FIELD.to_owned(), field_spec(FieldType::Set, None)),
            (HASH_FIELD.to_owned(), field_spec(FieldType::Hash, None)),
            (
                OBSOLETE_FIELD.to_owned(),
                field_spec(FieldType::Keyword, None),
            ),
        ]),
    }
}

fn create_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: schema(),
    }
}

fn doc_id(index: usize) -> String {
    format!("replace-doc-{index:02}")
}

fn giant_keyword_prefix(index: usize) -> String {
    format!("replace-giant-keyword-{index:02}:")
}

fn giant_keyword(index: usize) -> String {
    let prefix = giant_keyword_prefix(index);
    assert!(
        prefix.len() < PER_DOCUMENT_KEYWORD_BYTES,
        "small public keyword prefix must leave a real approximately 8.5 MiB value",
    );
    let mut bytes = vec![b'x'; PER_DOCUMENT_KEYWORD_BYTES];
    bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
    String::from_utf8(bytes).expect("oversized ReplaceDocs Keyword fixture is valid ASCII")
}

fn giant_hash(index: usize) -> String {
    format!("0x{:x}", 0x100_u64 + index as u64)
}

fn giant_fields(index: usize) -> BTreeMap<String, FieldValue> {
    BTreeMap::from([
        (
            KEYWORD_FIELD.to_owned(),
            FieldValue::String(giant_keyword(index)),
        ),
        (
            NUMBER_FIELD.to_owned(),
            FieldValue::Number(GIANT_NUMBER_BASE + index as f64),
        ),
        (
            TEXT_FIELD.to_owned(),
            FieldValue::String(format!("{COMMON_TEXT} document-{index:02}")),
        ),
        (
            SET_FIELD.to_owned(),
            FieldValue::StringList(vec![
                format!("replace-set-{index:02}"),
                SHARED_SET_VALUE.to_owned(),
                SHARED_SET_VALUE.to_owned(),
            ]),
        ),
        (HASH_FIELD.to_owned(), FieldValue::String(giant_hash(index))),
    ])
}

fn base_fields() -> BTreeMap<String, FieldValue> {
    BTreeMap::from([
        (
            KEYWORD_FIELD.to_owned(),
            FieldValue::String(format!("{BASE_KEYWORD_PREFIX}row")),
        ),
        (NUMBER_FIELD.to_owned(), FieldValue::Number(BASE_NUMBER)),
        (
            TEXT_FIELD.to_owned(),
            FieldValue::String(BASE_TEXT.to_owned()),
        ),
        (
            SET_FIELD.to_owned(),
            FieldValue::StringList(vec!["replace-base-set".to_owned()]),
        ),
        (HASH_FIELD.to_owned(), FieldValue::String("0x42".to_owned())),
        (
            OBSOLETE_FIELD.to_owned(),
            FieldValue::String(format!("{OBSOLETE_PREFIX}row")),
        ),
    ])
}

fn base_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::ReplaceDocs {
        collection_id: COLLECTION.to_owned(),
        req: ReplaceDocsRequest {
            docs: vec![ReplaceDocItem {
                external_id: BASE_ID.to_owned(),
                version: Some(BASE_VERSION),
                fields: base_fields(),
            }],
        },
    })
}

fn oversized_replace_record() -> WalRecord {
    assert_eq!(
        DOC_COUNT, MAX_BATCH_REPLACE_SIZE,
        "the fixture must use the public docs:replace maximum",
    );
    assert!(
        PER_DOCUMENT_KEYWORD_BYTES
            .checked_mul(DOC_COUNT)
            .expect("fixture byte multiplication")
            > PENDING_HARD_LIMIT_BYTES as usize,
        "32 approximately 8.5 MiB values must exceed the documented 256 MiB budget",
    );
    WalRecord::new(RaftLogEntry::ReplaceDocs {
        collection_id: COLLECTION.to_owned(),
        req: ReplaceDocsRequest {
            docs: (0..DOC_COUNT)
                .map(|index| ReplaceDocItem {
                    external_id: doc_id(index),
                    version: Some(GIANT_VERSION),
                    fields: giant_fields(index),
                })
                .collect(),
        },
    })
}

fn updated_fields() -> BTreeMap<String, FieldValue> {
    BTreeMap::from([
        (
            KEYWORD_FIELD.to_owned(),
            FieldValue::String(format!("{UPDATED_KEYWORD_PREFIX}row")),
        ),
        (NUMBER_FIELD.to_owned(), FieldValue::Number(UPDATED_NUMBER)),
        (
            TEXT_FIELD.to_owned(),
            FieldValue::String(UPDATED_TEXT.to_owned()),
        ),
        (
            SET_FIELD.to_owned(),
            FieldValue::StringList(vec![
                UPDATED_SET_VALUE.to_owned(),
                UPDATED_SET_VALUE.to_owned(),
            ]),
        ),
        (HASH_FIELD.to_owned(), FieldValue::String("0x999".to_owned())),
    ])
}

fn stale_fields() -> BTreeMap<String, FieldValue> {
    BTreeMap::from([
        (
            KEYWORD_FIELD.to_owned(),
            FieldValue::String(format!("{STALE_KEYWORD_PREFIX}row")),
        ),
        (NUMBER_FIELD.to_owned(), FieldValue::Number(-1.0)),
        (
            TEXT_FIELD.to_owned(),
            FieldValue::String(STALE_TEXT.to_owned()),
        ),
        (
            SET_FIELD.to_owned(),
            FieldValue::StringList(vec!["replace-stale-set".to_owned()]),
        ),
        (HASH_FIELD.to_owned(), FieldValue::String("0x998".to_owned())),
    ])
}

fn search_request(query: QueryNode) -> SearchRequest {
    SearchRequest {
        query,
        limit: 64,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn search_ids(engine: &Engine, query: QueryNode, label: &str) -> BTreeSet<String> {
    let response = engine
        .search(COLLECTION, search_request(query))
        .unwrap_or_else(|error| panic!("{label} public query failed: {error}"));
    let mut ids = BTreeSet::new();
    for hit in response.hits {
        assert!(
            ids.insert(hit.external_id.clone()),
            "{label} public query must not return duplicate external ID {}",
            hit.external_id,
        );
    }
    ids
}

fn prefix_ids_for_field(engine: &Engine, field: &str, prefix: &str) -> BTreeSet<String> {
    search_ids(
        engine,
        QueryNode::Prefix(PrefixQuery {
            field: field.to_owned(),
            value: prefix.to_owned(),
        }),
        field,
    )
}

fn prefix_ids(engine: &Engine, prefix: &str) -> BTreeSet<String> {
    prefix_ids_for_field(engine, KEYWORD_FIELD, prefix)
}

fn obsolete_ids(engine: &Engine) -> BTreeSet<String> {
    prefix_ids_for_field(engine, OBSOLETE_FIELD, OBSOLETE_PREFIX)
}

fn term_ids(engine: &Engine, field: &str, value: FieldValue) -> BTreeSet<String> {
    search_ids(
        engine,
        QueryNode::Term(TermQuery {
            field: field.to_owned(),
            value,
        }),
        field,
    )
}

fn match_ids(engine: &Engine, text: &str) -> BTreeSet<String> {
    search_ids(
        engine,
        QueryNode::Match(MatchQuery {
            field: TEXT_FIELD.to_owned(),
            text: text.to_owned(),
            op: MatchOp::And,
        }),
        "Text match",
    )
}

fn hamming_ids(engine: &Engine, hash: &str) -> BTreeSet<String> {
    search_ids(
        engine,
        QueryNode::Hamming(HammingQuery {
            field: HASH_FIELD.to_owned(),
            hash: hash.to_owned(),
            max_distance: 0,
        }),
        "exact Hash Hamming",
    )
}

fn singleton(index: usize) -> BTreeSet<String> {
    BTreeSet::from([doc_id(index)])
}

fn all_giant_ids() -> BTreeSet<String> {
    (0..DOC_COUNT).map(doc_id).collect()
}

fn tail_survivor_ids() -> BTreeSet<String> {
    (1..(DOC_COUNT - 1)).map(doc_id).collect()
}

fn assert_base_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("ReplaceDocs base collection stats")
            .documents_indexed,
        1,
        "{phase}: the valid committed ReplaceDocs base must contain exactly one row",
    );
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX),
        singleton(0),
        "{phase}: the valid committed base Keyword must remain query-visible",
    );
    assert_eq!(
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(BASE_NUMBER)),
        singleton(0),
        "{phase}: the valid committed base Number must remain query-visible",
    );
    assert_eq!(
        match_ids(engine, BASE_TEXT),
        singleton(0),
        "{phase}: the valid committed base Text must remain query-visible",
    );
    assert_eq!(
        term_ids(
            engine,
            SET_FIELD,
            FieldValue::String("replace-base-set".to_owned()),
        ),
        singleton(0),
        "{phase}: the valid committed base Set member must remain query-visible",
    );
    assert_eq!(
        hamming_ids(engine, "0x42"),
        singleton(0),
        "{phase}: the valid committed base Hash must remain exact-Hamming searchable",
    );
    assert_eq!(
        obsolete_ids(engine),
        singleton(0),
        "{phase}: the pre-existing obsolete field must be visible before full replacement",
    );
}

fn assert_oversized_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized ReplaceDocs collection stats")
            .documents_indexed,
        DOC_COUNT as u64,
        "{phase}: the committed oversized ReplaceDocs batch must expose all 32 documents",
    );
    assert_eq!(
        match_ids(engine, COMMON_TEXT),
        all_giant_ids(),
        "{phase}: every oversized full-replacement Text field must be query-visible",
    );
    assert_eq!(
        term_ids(
            engine,
            SET_FIELD,
            FieldValue::String(SHARED_SET_VALUE.to_owned()),
        ),
        all_giant_ids(),
        "{phase}: every oversized full-replacement Set field must be query-visible once per document",
    );
    for index in [0, DOC_COUNT / 2, DOC_COUNT - 1] {
        assert_eq!(
            prefix_ids(engine, &giant_keyword_prefix(index)),
            singleton(index),
            "{phase}: oversized Keyword {index} must be query-visible without rebuilding its source",
        );
        assert_eq!(
            term_ids(
                engine,
                KEYWORD_FIELD,
                FieldValue::String(giant_keyword(index)),
            ),
            singleton(index),
            "{phase}: oversized Keyword {index} must retain every original byte, not only its searchable prefix",
        );
    }
    for index in 0..DOC_COUNT {
        assert_eq!(
            term_ids(
                engine,
                NUMBER_FIELD,
                FieldValue::Number(GIANT_NUMBER_BASE + index as f64),
            ),
            singleton(index),
            "{phase}: oversized Number {index} must remain paired with its complete document",
        );
        assert_eq!(
            hamming_ids(engine, &giant_hash(index)),
            singleton(index),
            "{phase}: oversized Hash {index} must remain paired with its complete document",
        );
    }
    assert_eq!(
        obsolete_ids(engine),
        BTreeSet::new(),
        "{phase}: fields absent from the oversized ReplaceDocs item must be deleted, not merged",
    );
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX),
        BTreeSet::new(),
        "{phase}: the base Keyword must not survive the complete replacement",
    );
}

fn assert_tail_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("ReplaceDocs tail collection stats")
            .documents_indexed,
        (DOC_COUNT - 1) as u64,
        "{phase}: one later delete must remove exactly its committed full document",
    );
    assert_eq!(
        prefix_ids(engine, UPDATED_KEYWORD_PREFIX),
        singleton(0),
        "{phase}: the higher-version complete replacement must win",
    );
    assert_eq!(
        prefix_ids(engine, STALE_KEYWORD_PREFIX),
        BTreeSet::new(),
        "{phase}: a strictly older ReplaceDocs retry must drop the entire item",
    );
    assert_eq!(
        prefix_ids(engine, &giant_keyword_prefix(0)),
        BTreeSet::new(),
        "{phase}: the large previous Keyword must not revive after the higher version wins",
    );
    assert_eq!(
        prefix_ids(engine, &giant_keyword_prefix(DOC_COUNT - 1)),
        BTreeSet::new(),
        "{phase}: the deleted oversized document must not revive",
    );
    assert_eq!(
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(UPDATED_NUMBER)),
        singleton(0),
        "{phase}: the higher-version Number must replace the giant row value",
    );
    assert_eq!(
        term_ids(
            engine,
            NUMBER_FIELD,
            FieldValue::Number(GIANT_NUMBER_BASE),
        ),
        BTreeSet::new(),
        "{phase}: the old first-document Number must not survive full replacement",
    );
    assert_eq!(
        match_ids(engine, UPDATED_TEXT),
        singleton(0),
        "{phase}: the higher-version Text must replace the giant row Text",
    );
    assert_eq!(
        match_ids(engine, STALE_TEXT),
        BTreeSet::new(),
        "{phase}: stale Text must not become visible",
    );
    assert_eq!(
        match_ids(engine, COMMON_TEXT),
        tail_survivor_ids(),
        "{phase}: full replacement and delete must remove old Text from exactly their two rows",
    );
    assert_eq!(
        term_ids(
            engine,
            SET_FIELD,
            FieldValue::String(UPDATED_SET_VALUE.to_owned()),
        ),
        singleton(0),
        "{phase}: the higher-version Set must replace the giant row Set",
    );
    assert_eq!(
        term_ids(
            engine,
            SET_FIELD,
            FieldValue::String(SHARED_SET_VALUE.to_owned()),
        ),
        tail_survivor_ids(),
        "{phase}: omitted Set values must not revive after full replacement or delete",
    );
    assert_eq!(
        hamming_ids(engine, "0x999"),
        singleton(0),
        "{phase}: the higher-version Hash must replace the giant row Hash",
    );
    assert_eq!(
        hamming_ids(engine, "0x100"),
        BTreeSet::new(),
        "{phase}: the old first-document Hash must not revive",
    );
    assert_eq!(
        hamming_ids(engine, &giant_hash(DOC_COUNT - 1)),
        BTreeSet::new(),
        "{phase}: the deleted final-document Hash must not revive",
    );
    assert_eq!(
        obsolete_ids(engine),
        BTreeSet::new(),
        "{phase}: obsolete fields must remain deleted after incremental persistence",
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


fn encode_record(record: WalRecord, phase: &str) -> Vec<u8> {
    record
        .encode()
        .unwrap_or_else(|error| panic!("{phase}: encode valid committed Raft record: {error}"))
}

fn create_command() -> Vec<u8> {
    encode_record(WalRecord::new(create_entry()), "collection schema")
}

fn base_command() -> Vec<u8> {
    encode_record(base_record(), "complete base replacement")
}

fn oversized_replace_command() -> Vec<u8> {
    assert_eq!(
        DOC_COUNT, MAX_BATCH_REPLACE_SIZE,
        "the Raft fixture must use the unchanged public docs:replace maximum",
    );
    assert!(
        PER_DOCUMENT_KEYWORD_BYTES
            .checked_mul(DOC_COUNT)
            .expect("Raft fixture byte multiplication")
            > PENDING_HARD_LIMIT_BYTES as usize,
        "32 approximately 8.5 MiB Keyword values must exceed the documented 256 MiB budget",
    );
    let command = encode_record(
        oversized_replace_record(),
        "oversized generic-CBOR full replacement",
    );
    assert!(
        !command.starts_with(b"LWAL"),
        "ReplaceDocs must use its real generic-CBOR Raft command form, not the fast Index wire",
    );
    assert!(
        command.len() > PENDING_HARD_LIMIT_BYTES as usize,
        "the actual committed generic-CBOR ReplaceDocs command must exceed 256 MiB: command_bytes={}",
        command.len(),
    );
    command
}

fn update_command() -> Vec<u8> {
    encode_record(
        WalRecord::new(RaftLogEntry::ReplaceDocs {
            collection_id: COLLECTION.to_owned(),
            req: ReplaceDocsRequest {
                docs: vec![ReplaceDocItem {
                    external_id: BASE_ID.to_owned(),
                    version: Some(UPDATED_VERSION),
                    fields: updated_fields(),
                }],
            },
        }),
        "higher-version complete replacement",
    )
}

fn stale_command() -> Vec<u8> {
    encode_record(
        WalRecord::new(RaftLogEntry::ReplaceDocs {
            collection_id: COLLECTION.to_owned(),
            req: ReplaceDocsRequest {
                docs: vec![ReplaceDocItem {
                    external_id: BASE_ID.to_owned(),
                    version: Some(GIANT_VERSION),
                    fields: stale_fields(),
                }],
            },
        }),
        "stale complete replacement retry",
    )
}

fn delete_command() -> Vec<u8> {
    encode_record(
        WalRecord::new(RaftLogEntry::Delete {
            collection_id: COLLECTION.to_owned(),
            external_id: DELETE_ID.to_owned(),
            field: None,
        }),
        "later committed full-document delete",
    )
}

fn apply_foreign_committed(
    state_machine: &EngineSm,
    sequence: Index,
    command: &[u8],
    phase: &str,
) {
    let result = state_machine.apply(sequence, command);
    let diagnostic = result
        .as_ref()
        .err()
        .map(|error| format!("{error:#}"))
        .unwrap_or_default();
    assert!(
        result.is_ok(),
        "{phase}: valid foreign committed Raft command must apply without owned decode refusal: {diagnostic}",
    );
    assert_eq!(
        state_machine.applied_index(),
        sequence,
        "{phase}: one whole committed command must advance the Raft watermark exactly once",
    );
}

fn checkpoint_and_cold(
    store: &SegmentRdbStore,
    engine: &Arc<Engine>,
    sequence: Index,
    phase: &str,
) -> Arc<Engine> {
    let saved = store
        .save_with_sequence(engine, sequence)
        .unwrap_or_else(|error| panic!("{phase}: real Raft checkpoint failed: {error:#}"));
    assert_eq!(
        saved, sequence,
        "{phase}: checkpoint must retain the exact committed Raft watermark",
    );
    let cold = store
        .load_current_generation()
        .unwrap_or_else(|error| panic!("{phase}: cold-open CURRENT failed: {error:#}"))
        .unwrap_or_else(|| panic!("{phase}: checkpoint must publish CURRENT"));
    assert_eq!(
        cold.sequence, sequence,
        "{phase}: cold CURRENT must retain the exact committed Raft watermark",
    );
    cold.engine
}

fn child_paths() -> Option<(PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("foreign committed ReplaceDocs child needs a parent-owned durable root");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("foreign committed ReplaceDocs child needs a handshake path");
    Some((root, handshake))
}

async fn run_foreign_committed_replace_child(root: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact foreign committed ReplaceDocs child entry");
    let store = Arc::new(
        SegmentRdbStore::new(root.join("segments"))
            .expect("open Raft committed ReplaceDocs segment store"),
    );
    let engine = Arc::new(Engine::new());
    let state_machine = EngineSm::new_with_segment_store(engine.clone(), 0, store.clone());

    let create = create_command();
    apply_foreign_committed(
        state_machine.as_ref(),
        CREATE_SEQUENCE,
        &create,
        "collection schema",
    );
    let base = base_command();
    apply_foreign_committed(
        state_machine.as_ref(),
        BASE_SEQUENCE,
        &base,
        "complete base replacement",
    );
    assert_base_state(&engine, "live foreign committed ReplaceDocs base");
    let base_cold = checkpoint_and_cold(
        store.as_ref(),
        &engine,
        BASE_SEQUENCE,
        "base Raft checkpoint",
    );
    assert_base_state(&base_cold, "cold foreign committed ReplaceDocs base");

    let mut giant = oversized_replace_command();
    apply_foreign_committed(
        state_machine.as_ref(),
        GIANT_SEQUENCE,
        &giant,
        "oversized generic-CBOR ReplaceDocs",
    );
    assert_eq!(
        state_machine.applied_index(),
        GIANT_SEQUENCE,
        "the 32-document foreign command must advance one Raft index, not one index per document",
    );
    assert_oversized_state(&engine, "live oversized foreign committed ReplaceDocs");
    assert_public_pending_budget(
        engine.clone(),
        GIANT_SEQUENCE,
        "live oversized foreign committed ReplaceDocs",
    )
    .await;
    let giant_cold = checkpoint_and_cold(
        store.as_ref(),
        &engine,
        GIANT_SEQUENCE,
        "oversized Raft checkpoint",
    );
    assert_oversized_state(&giant_cold, "cold oversized foreign committed ReplaceDocs");
    assert_public_pending_budget(
        giant_cold.clone(),
        GIANT_SEQUENCE,
        "cold oversized foreign committed ReplaceDocs",
    )
    .await;

    let update = update_command();
    apply_foreign_committed(
        state_machine.as_ref(),
        UPDATE_SEQUENCE,
        &update,
        "higher-version complete replacement",
    );
    let stale = stale_command();
    apply_foreign_committed(
        state_machine.as_ref(),
        STALE_SEQUENCE,
        &stale,
        "stale complete replacement retry",
    );
    let delete = delete_command();
    apply_foreign_committed(
        state_machine.as_ref(),
        DELETE_SEQUENCE,
        &delete,
        "later committed full-document delete",
    );
    assert_tail_state(&engine, "live Raft version/stale/delete suffix");
    assert_public_pending_budget(
        engine.clone(),
        DELETE_SEQUENCE,
        "live Raft version/stale/delete suffix",
    )
    .await;
    let final_cold = checkpoint_and_cold(
        store.as_ref(),
        &engine,
        DELETE_SEQUENCE,
        "final Raft checkpoint",
    );
    assert_tail_state(&final_cold, "cold Raft version/stale/delete suffix");
    assert_public_pending_budget(
        final_cold.clone(),
        DELETE_SEQUENCE,
        "cold Raft version/stale/delete suffix",
    )
    .await;

    // The peer-owned command stays retained until all behavior assertions are
    // done. Corrupting its generic-CBOR root exercises the same external byte
    // boundary without allocating a second oversized source.
    giant[0] = 0xff;
    let malformed = state_machine.apply(MALFORMED_SEQUENCE, &giant);
    assert!(
        malformed.is_err(),
        "a malformed oversized generic-CBOR foreign Raft command must be refused before publication",
    );
    assert_eq!(
        state_machine.applied_index(),
        DELETE_SEQUENCE,
        "a refused malformed foreign command must not advance the Raft watermark",
    );
    assert_tail_state(&engine, "malformed foreign command leaves live state unchanged");
}

async fn run_isolated_child(root: &Path) {
    let child_root = tempfile::tempdir().expect("foreign committed ReplaceDocs child workspace");
    let child_tmp = child_root.path().join("child-tmp");
    fs::create_dir(&child_tmp)
        .expect("create parent-owned foreign committed ReplaceDocs child temporary directory");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable = std::env::current_exe().expect("current Raft committed ReplaceDocs test executable");
    let stdout = File::create(&stdout_path).expect("create Raft committed ReplaceDocs child stdout");
    let stderr = File::create(&stderr_path).expect("create Raft committed ReplaceDocs child stderr");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
        .env(CHILD_ROOT_ENV, root)
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
        .expect("spawn isolated foreign committed ReplaceDocs child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_CLEANUP_BOUND;
    loop {
        match child
            .0
            .as_mut()
            .expect("Raft committed ReplaceDocs child remains owned until it exits")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child
                    .0
                    .take()
                    .expect("timed-out Raft committed ReplaceDocs child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("wait for killed Raft committed ReplaceDocs child");
                let stdout = fs::read_to_string(&stdout_path)
                    .expect("read killed Raft committed ReplaceDocs child stdout");
                let stderr = fs::read_to_string(&stderr_path)
                    .expect("read killed Raft committed ReplaceDocs child stderr");
                panic!(
                    "a valid foreign committed 32-document >256 MiB Raft ReplaceDocs command must apply and checkpoint before cleanup; child was killed after {CHILD_CLEANUP_BOUND:?}: status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated Raft committed ReplaceDocs child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited Raft committed ReplaceDocs child remains owned")
        .wait()
        .expect("wait for exited Raft committed ReplaceDocs child");
    let stdout = fs::read_to_string(&stdout_path).expect("read Raft committed ReplaceDocs child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read Raft committed ReplaceDocs child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated child did not enter exact {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must enter the intended foreign committed ReplaceDocs body",
    );
    assert!(
        status.success(),
        "isolated foreign committed ReplaceDocs child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raft_committed_oversized_replace_applies_checkpoints_and_cold_opens() {
    if let Some((root, handshake)) = child_paths() {
        run_foreign_committed_replace_child(root, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("foreign committed Raft ReplaceDocs fixture root");
    run_isolated_child(root.path()).await;
}
