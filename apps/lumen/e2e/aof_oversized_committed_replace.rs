//! Black-box contract for an oversized committed `docs:replace` AOF record.
//!
//! The fixture writes an ordinary schema and one small full-document base through
//! the real `AofWriter`, then checkpoints and cold-opens that base. Its next
//! committed record is one `ReplaceDocs` batch with the public maximum of 32
//! complete documents. Each document has an approximately 8.5 MiB Keyword plus
//! small Number, Text, Set, and Hash fields. The payload is larger than 256 MiB.
//! The giant source record is dropped immediately after the AOF frame is synced.
//!
//! The test checks public query results rather than reconstructing the giant
//! values. It proves complete replacement by giving the base document an
//! `obsolete` field which the oversized record omits. A later higher-version
//! replace, stale retry, and delete prove document-level LWW and no value
//! revival through an incremental checkpoint and cold reopen. The child process
//! owns replay so a broken baseline capacity wait cannot leave process-wide
//! reservations behind. Its temporary paths are parent-owned through `TMPDIR`,
//! `TEMP`, and `TMP`; the 240-second watchdog is cleanup only, not a latency
//! promise.
//!
//! # Facets
//!
//! - Behavior: aof_oversized_committed_replace.rs:934-947 is the expected
//!   replay red. Assertions at :590-753 and :932-1044 require 32 complete
//!   documents, full exact Keyword bytes for rows 0, 16, and 31, Number and
//!   Hash values for all 32 rows, full-replacement deletion of `obsolete`,
//!   document-version LWW, strict suffix replay, incremental checkpoints, and
//!   cold reopen. They cover
//!   apps/lumen/src/types.rs:699-737, apps/lumen/src/log_entry.rs:18-39,
//!   apps/lumen/src/aof.rs:285-389, apps/lumen/src/wal.rs:218-249,
//!   apps/lumen/src/wal/bounded_generic.rs:43-82, and
//!   apps/lumen/src/storage.rs:5536-5807.
//! - Security: aof_oversized_committed_replace.rs:797-868 mutates one byte of
//!   an AofWriter-written oversized frame, recomputes its CRC, and requires
//!   refusal before its documents or watermark are published. This exercises
//!   the persisted-byte boundary in apps/lumen/src/aof.rs:285-389.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   aof_oversized_committed_replace.rs:773-792 reads public pending total and
//!   high-water gauges after replay and each checkpoint, requiring both to stay
//!   within that budget. Fixture source allocations and mapped AOF bytes are
//!   outside those gauges. This contract sets no latency budget.
//!
//! # Root negative control
//!
//! After borrowed ReplaceDocs replay exists, bypass its mapped ReplaceDocs
//! dispatch before generic decode in `apps/lumen/src/aof.rs`. The assertion at
//! :934 must fail with the current owned-decode refusal or child cleanup
//! watchdog. Restore each changed source file by SHA-256. Never lower the
//! 32-document fixture or its >256 MiB payload assertion.
//!
//! Target gate: cargo test -p lumen --test aof_oversized_committed_replace -- --nocapture.
//! Full declared behavior gate: cargo test -p lumen.

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
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, HammingQuery, MatchOp,
    MatchQuery, PrefixQuery, QueryNode, ReplaceDocItem, ReplaceDocsRequest, SearchRequest,
    TermQuery, MAX_BATCH_REPLACE_SIZE,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "oversized-aof-committed-replace";
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
const FRAME_HEADER_BYTES: u64 = 16;
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
const CREATE_SEQUENCE: u64 = 1;
const BASE_SEQUENCE: u64 = 2;
const GIANT_SEQUENCE: u64 = 3;
const UPDATE_SEQUENCE: u64 = 4;
const STALE_SEQUENCE: u64 = 5;
const DELETE_SEQUENCE: u64 = 6;
const BASE_VERSION: u64 = 10;
const GIANT_VERSION: u64 = 20;
const UPDATED_VERSION: u64 = 30;
const REPLAY_WATCHDOG: Duration = Duration::from_secs(240);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_AOF_OVERSIZED_REPLACE_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_AOF_OVERSIZED_REPLACE_ROOT";
const CHILD_AOF_ENV: &str = "LUMEN_AOF_OVERSIZED_REPLACE_AOF";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_AOF_OVERSIZED_REPLACE_HANDSHAKE";
const CHILD_CASE: &str = "replay";
const TEST_NAME: &str = "oversized_committed_replace_aof_replays_checkpoints_and_cold_opens";

/// Supplies the true replay watermark without constructing a coordinator that
/// would initialize the same Engine capture barrier at sequence zero.
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

/// Owns the child on every parent panic path. A killed child is reaped before
/// its parent-owned temporary workspace is removed.
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
        (
            HASH_FIELD.to_owned(),
            FieldValue::String("0x999".to_owned()),
        ),
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
        (
            HASH_FIELD.to_owned(),
            FieldValue::String("0x998".to_owned()),
        ),
    ])
}

fn append_base_prefix(path: &Path) {
    let mut writer = AofWriter::open(path).expect("open real ReplaceDocs AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append ReplaceDocs schema through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync real ReplaceDocs schema AOF frame");
    let base = base_record();
    writer
        .append(BASE_SEQUENCE, &base)
        .expect("append committed ReplaceDocs base through AofWriter");
    drop(base);
    writer
        .sync_strict()
        .expect("strict-sync committed ReplaceDocs base AOF prefix");
}

/// The only giant source exists inside this append. Replay reads its retained
/// AOF frame after this `WalRecord` and the encoder buffer are gone.
fn append_oversized_suffix(path: &Path) -> FrameSpan {
    let mut writer = AofWriter::open(path).expect("reopen AOF for oversized ReplaceDocs suffix");
    let start = fs::metadata(path)
        .expect("measure AOF before oversized ReplaceDocs suffix")
        .len();
    let record = oversized_replace_record();
    writer
        .append(GIANT_SEQUENCE, &record)
        .expect("append real oversized ReplaceDocs AOF frame");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync real oversized ReplaceDocs AOF frame");
    drop(writer);
    let span = FrameSpan { start };
    let payload_bytes = frame_payload_bytes(path, span);
    assert!(
        payload_bytes > PENDING_HARD_LIMIT_BYTES,
        "the real AofWriter ReplaceDocs payload must exceed 256 MiB: payload_bytes={payload_bytes}",
    );
    span
}

fn append_tail(path: &Path) {
    let mut writer = AofWriter::open(path).expect("reopen AOF for ReplaceDocs tail");
    let update = WalRecord::new(RaftLogEntry::ReplaceDocs {
        collection_id: COLLECTION.to_owned(),
        req: ReplaceDocsRequest {
            docs: vec![ReplaceDocItem {
                external_id: BASE_ID.to_owned(),
                version: Some(UPDATED_VERSION),
                fields: updated_fields(),
            }],
        },
    });
    writer
        .append(UPDATE_SEQUENCE, &update)
        .expect("append higher-version full ReplaceDocs update");
    drop(update);
    let stale = WalRecord::new(RaftLogEntry::ReplaceDocs {
        collection_id: COLLECTION.to_owned(),
        req: ReplaceDocsRequest {
            docs: vec![ReplaceDocItem {
                external_id: BASE_ID.to_owned(),
                version: Some(GIANT_VERSION),
                fields: stale_fields(),
            }],
        },
    });
    writer
        .append(STALE_SEQUENCE, &stale)
        .expect("append stale ReplaceDocs retry");
    drop(stale);
    let delete = WalRecord::new(RaftLogEntry::Delete {
        collection_id: COLLECTION.to_owned(),
        external_id: DELETE_ID.to_owned(),
        field: None,
    });
    writer
        .append(DELETE_SEQUENCE, &delete)
        .expect("append later committed complete-document delete");
    drop(delete);
    writer
        .sync_strict()
        .expect("strict-sync ReplaceDocs update/stale/delete suffix");
}

fn frame_payload_bytes(path: &Path, span: FrameSpan) -> u64 {
    let mut file = File::open(path).expect("open real ReplaceDocs AOF frame");
    file.seek(SeekFrom::Start(span.start + 8))
        .expect("seek real ReplaceDocs AOF frame length");
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)
        .expect("read real ReplaceDocs AOF frame length");
    u64::from(u32::from_le_bytes(length))
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
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(GIANT_NUMBER_BASE),),
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

/// Make a complete persisted frame structurally invalid while preserving a
/// valid CRC. Byte zero is a generic CBOR item header for the current
/// ReplaceDocs wire form; 0xff is a standalone CBOR break and must be refused.
fn corrupt_complete_frame_payload(path: &Path, span: FrameSpan) {
    let payload_bytes = frame_payload_bytes(path, span);
    assert!(
        payload_bytes > PENDING_HARD_LIMIT_BYTES,
        "security frame must stay on the real >256 MiB AOF path",
    );
    let payload_start = span.start + FRAME_HEADER_BYTES;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open authentic ReplaceDocs AOF frame for corruption");
    file.seek(SeekFrom::Start(payload_start))
        .expect("seek authentic ReplaceDocs AOF payload byte");
    file.write_all(&[0xff])
        .expect("write invalid generic ReplaceDocs payload header");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = payload_bytes;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic ReplaceDocs payload for CRC");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..read_len])
            .expect("stream authentic ReplaceDocs payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len as u64;
    }
    file.seek(SeekFrom::Start(span.start + 12))
        .expect("seek authentic ReplaceDocs AOF frame CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite authentic ReplaceDocs AOF frame CRC");
    file.sync_all()
        .expect("sync complete corrupt ReplaceDocs AOF frame");
}

fn assert_complete_corrupt_oversized_replace_frame_is_refused(
    root: &Path,
    path: &Path,
    span: FrameSpan,
) {
    assert!(
        frame_payload_bytes(path, span) > PENDING_HARD_LIMIT_BYTES,
        "the persisted corrupt security frame must remain the real >256 MiB ReplaceDocs record",
    );
    let engine = Arc::new(Engine::new());
    let refusal = replay_aof_into(&engine, path, 0);
    assert!(
        refusal.is_err(),
        "a CRC-valid but format-corrupt >256 MiB ReplaceDocs AOF frame must be refused before publication",
    );
    assert_base_state(
        &engine,
        "security valid ReplaceDocs prefix after refused frame",
    );
    let store = SegmentRdbStore::new(root.join("complete-corrupt-oversized-replace-segments"))
        .expect("open security ReplaceDocs segment store");
    let saved = store
        .save_with_sequence(&engine, 0)
        .expect("save only the valid ReplaceDocs AOF prefix");
    assert_eq!(
        saved, BASE_SEQUENCE,
        "a refused oversized ReplaceDocs frame must not advance the durable checkpoint watermark",
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open valid ReplaceDocs security prefix")
        .expect("valid ReplaceDocs security prefix checkpoint exists");
    assert_eq!(
        cold.sequence, BASE_SEQUENCE,
        "cold-open must retain only the valid ReplaceDocs prefix watermark",
    );
    assert_base_state(
        &cold.engine,
        "security cold ReplaceDocs prefix after refused frame",
    );
}

fn child_paths() -> Option<(PathBuf, PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("isolated ReplaceDocs replay child needs its durable root");
    let aof = std::env::var_os(CHILD_AOF_ENV)
        .map(PathBuf::from)
        .expect("isolated ReplaceDocs replay child needs its AOF path");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("isolated ReplaceDocs replay child needs a handshake path");
    Some((root, aof, handshake))
}

async fn run_replay_child(root: PathBuf, aof_path: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact isolated ReplaceDocs child entry");

    let base_engine = Arc::new(Engine::new());
    let base_replay = replay_aof_into(&base_engine, &aof_path, 0);
    assert!(
        base_replay.is_ok(),
        "the valid committed ReplaceDocs base must replay before the giant suffix: {}",
        base_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        base_replay.expect("behavior assertion above checked ReplaceDocs base replay"),
        BASE_SEQUENCE,
        "the committed ReplaceDocs base replay must advance through its exact sequence",
    );
    assert_base_state(&base_engine, "committed ReplaceDocs base AOF replay");
    assert_public_pending_budget(
        base_engine.clone(),
        BASE_SEQUENCE,
        "committed ReplaceDocs base AOF replay",
    )
    .await;

    let store = SegmentRdbStore::new(root.join("segments"))
        .expect("open segment store for oversized ReplaceDocs AOF checkpoint");
    let base_checkpoint = store
        .save_with_sequence(&base_engine, BASE_SEQUENCE)
        .expect("checkpoint committed ReplaceDocs base before giant suffix");
    assert_eq!(
        base_checkpoint, BASE_SEQUENCE,
        "the initial ReplaceDocs checkpoint must retain its established watermark",
    );
    let base_cold = store
        .load_current_generation()
        .expect("cold-open committed ReplaceDocs base checkpoint")
        .expect("committed ReplaceDocs base checkpoint must publish CURRENT");
    assert_eq!(
        base_cold.sequence, BASE_SEQUENCE,
        "cold ReplaceDocs base checkpoint must keep its exact watermark",
    );
    assert_base_state(
        &base_cold.engine,
        "cold committed ReplaceDocs base checkpoint",
    );

    let giant_span = append_oversized_suffix(&aof_path);
    let giant_replay = replay_aof_into(&base_cold.engine, &aof_path, base_cold.sequence);
    assert!(
        giant_replay.is_ok(),
        "a valid committed 32-document >256 MiB ReplaceDocs AOF suffix must replay without owned decode refusal: {}",
        giant_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        giant_replay.expect("behavior assertion above checked oversized ReplaceDocs replay"),
        GIANT_SEQUENCE,
        "valid oversized ReplaceDocs replay must advance through its committed suffix sequence",
    );
    assert_oversized_state(&base_cold.engine, "oversized ReplaceDocs AOF suffix replay");
    assert_public_pending_budget(
        base_cold.engine.clone(),
        GIANT_SEQUENCE,
        "oversized ReplaceDocs AOF suffix replay",
    )
    .await;

    let giant_checkpoint = store
        .save_with_sequence(&base_cold.engine, GIANT_SEQUENCE)
        .expect("incrementally checkpoint oversized ReplaceDocs suffix over its base");
    assert_eq!(
        giant_checkpoint, GIANT_SEQUENCE,
        "incremental ReplaceDocs checkpoint must retain the giant suffix watermark",
    );
    let giant_cold = store
        .load_current_generation()
        .expect("cold-open incremental ReplaceDocs checkpoint")
        .expect("incremental ReplaceDocs checkpoint must publish CURRENT");
    assert_eq!(
        giant_cold.sequence, GIANT_SEQUENCE,
        "cold incremental ReplaceDocs checkpoint must retain the giant sequence",
    );
    assert_oversized_state(&giant_cold.engine, "cold oversized ReplaceDocs checkpoint");
    assert_public_pending_budget(
        giant_cold.engine.clone(),
        GIANT_SEQUENCE,
        "cold oversized ReplaceDocs checkpoint",
    )
    .await;

    append_tail(&aof_path);
    // A checkpoint at GIANT_SEQUENCE covers the oversized frame. Its valid-CRC
    // corruption proves strict suffix replay skips covered input before decode.
    corrupt_complete_frame_payload(&aof_path, giant_span);
    let tail_replay = replay_aof_into(&giant_cold.engine, &aof_path, giant_cold.sequence);
    assert!(
        tail_replay.is_ok(),
        "strict ReplaceDocs suffix replay must skip the corrupt covered giant frame and apply later records: {}",
        tail_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        tail_replay.expect("behavior assertion above checked ReplaceDocs tail replay"),
        DELETE_SEQUENCE,
        "strict ReplaceDocs suffix replay must advance through its final committed sequence",
    );
    assert_tail_state(
        &giant_cold.engine,
        "strict ReplaceDocs update/stale/delete suffix replay",
    );
    assert_public_pending_budget(
        giant_cold.engine.clone(),
        DELETE_SEQUENCE,
        "strict ReplaceDocs update/stale/delete suffix replay",
    )
    .await;

    let final_checkpoint = store
        .save_with_sequence(&giant_cold.engine, DELETE_SEQUENCE)
        .expect("checkpoint ReplaceDocs update/stale/delete suffix");
    assert_eq!(
        final_checkpoint, DELETE_SEQUENCE,
        "final ReplaceDocs checkpoint must retain the completed AOF watermark",
    );
    let final_cold = store
        .load_current_generation()
        .expect("cold-open final oversized ReplaceDocs checkpoint")
        .expect("final oversized ReplaceDocs checkpoint must publish CURRENT");
    assert_eq!(
        final_cold.sequence, DELETE_SEQUENCE,
        "final cold ReplaceDocs checkpoint must retain the full watermark",
    );
    assert_tail_state(
        &final_cold.engine,
        "final oversized ReplaceDocs cold reopen",
    );
    assert_public_pending_budget(
        final_cold.engine.clone(),
        DELETE_SEQUENCE,
        "final oversized ReplaceDocs cold reopen",
    )
    .await;

    let covered_replay = replay_aof_into(&final_cold.engine, &aof_path, final_cold.sequence);
    assert!(
        covered_replay.is_ok(),
        "fully checkpoint-covered ReplaceDocs AOF frames must be skipped without replay failure: {}",
        covered_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        covered_replay.expect("behavior assertion above checked covered ReplaceDocs replay"),
        0,
        "fully checkpoint-covered ReplaceDocs frames must not advance the watermark",
    );
    assert_tail_state(&final_cold.engine, "fully covered ReplaceDocs AOF replay");
}

async fn run_isolated_replay(root: &Path, aof_path: &Path) {
    let child_root = tempfile::tempdir().expect("isolated ReplaceDocs child workspace");
    let child_tmp = child_root.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned ReplaceDocs child temporary directory");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable =
        std::env::current_exe().expect("current oversized ReplaceDocs test executable");
    let stdout = File::create(&stdout_path).expect("create ReplaceDocs child stdout");
    let stderr = File::create(&stderr_path).expect("create ReplaceDocs child stderr");
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
        .expect("spawn isolated oversized ReplaceDocs replay child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + REPLAY_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("ReplaceDocs child remains owned until it exits")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child
                    .0
                    .take()
                    .expect("timed-out ReplaceDocs child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("wait for killed ReplaceDocs child");
                let stdout =
                    fs::read_to_string(&stdout_path).expect("read killed ReplaceDocs child stdout");
                let stderr =
                    fs::read_to_string(&stderr_path).expect("read killed ReplaceDocs child stderr");
                panic!(
                    "a valid committed 32-document >256 MiB ReplaceDocs AOF frame must replay and checkpoint before cleanup; child was killed after {REPLAY_WATCHDOG:?}: status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated oversized ReplaceDocs child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited ReplaceDocs child remains owned")
        .wait()
        .expect("wait for exited ReplaceDocs child");
    let stdout = fs::read_to_string(&stdout_path).expect("read ReplaceDocs child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read ReplaceDocs child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated child did not enter exact {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must enter the intended oversized ReplaceDocs replay body",
    );
    assert!(
        status.success(),
        "isolated oversized ReplaceDocs replay child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_committed_replace_aof_replays_checkpoints_and_cold_opens() {
    if let Some((root, aof_path, handshake)) = child_paths() {
        run_replay_child(root, aof_path, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("oversized committed ReplaceDocs AOF fixture root");
    let aof_path = root.path().join("oversized-replace.aof");
    append_base_prefix(&aof_path);
    let giant_span = FrameSpan {
        start: fs::metadata(&aof_path)
            .expect("measure ReplaceDocs base prefix before isolated giant append")
            .len(),
    };
    run_isolated_replay(root.path(), &aof_path).await;
    assert_complete_corrupt_oversized_replace_frame_is_refused(root.path(), &aof_path, giant_span);
}
