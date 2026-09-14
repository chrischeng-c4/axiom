//! Black-box contract for a native committed oversized `docs:replace` record.
//!
//! The fixture publishes typed `WalRecord` values straight to the real
//! `MemWal`. It never calls `WriteCoordinator::submit`, an HTTP write endpoint,
//! or `stage_source`. The schema and small base are committed this same way.
//! The next record is one public-maximum 32-document `ReplaceDocs` record with
//! generic CBOR bytes above 256 MiB. The coordinator must retain the foreign
//! source, apply it, append its durable AOF record, and advance its watermark.
//!
//! The small base puts an `obsolete` field on row zero. The oversized full
//! replacement omits that field. The contract verifies complete large Keyword
//! bytes for rows zero, 16, and 31 one at a time, all document IDs and Number
//! values, and removal of the omitted field. A real checkpoint and cold open
//! must retain the same committed sequence and data. The child owns all giant
//! source memory. The parent owns its directory, `TMPDIR`, `TEMP`, and `TMP`,
//! and kills and reaps a stuck child only for cleanup.
//!
//! # Facets
//!
//! - Behavior: native_oversized_committed_replace.rs:652-720 publishes the
//!   schema, base, and 32-document generic-CBOR record through `MemWal` and
//!   requires the `WriteCoordinator` watermark to reach the committed head.
//!   :465-530 requires complete replacement, all rows, exact full Keyword
//!   bytes for rows 0, 16, and 31, all Number and Hash values, and deletion of
//!   the omitted base field. :722-733 requires the real checkpoint and cold
//!   reopen to retain that same head and state. These cover
//!   apps/lumen/src/coordinator.rs:390-647,
//!   apps/lumen/src/wal.rs:943-961, and
//!   apps/lumen/src/segment_checkpoint.rs:220-268.
//! - Security: this direct native delivery receives a typed `WalRecord`, not
//!   caller-controlled HTTP bytes, a path, or an identity, at
//!   apps/lumen/src/wal.rs:808-832 and
//!   apps/lumen/src/coordinator.rs:419-517. Its new private staged receipt is
//!   process-owned under a 0700 directory at
//!   apps/lumen/src/wal_source_stage.rs:31-63 and :155-190; this route adds no
//!   caller-selected file path. The persisted-AOF byte boundary it reaches is
//!   already fail-closed in
//!   apps/lumen/e2e/aof_oversized_committed_replace.rs:794-868, which mutates
//!   a complete frame and requires refusal before document or watermark change.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   native_oversized_committed_replace.rs:304-346 proves the actual generic
//!   CBOR source is above that limit while remaining at the public 32-document
//!   maximum. :550-564 reads public pending total and high-water gauges after
//!   native apply and checkpoint and requires both to stay within that budget.
//!   The 240-second child bound is test cleanup only, not a latency promise.
//!
//! # Root negative control
//!
//! After the native generic borrowed route exists, bypass its `ReplaceDocs`
//! dispatch in `apps/lumen/src/coordinator.rs` before it obtains a bounded
//! staged representation. The behavior assertion at :609-614 must fail because
//! the foreign committed sequence remains unapplied or requires restart. Restore
//! every changed source file by SHA-256. Never lower the public 32-document
//! fixture or its actual >256 MiB generic-CBOR payload assertion.
//!
//! Target gate: cargo test -p lumen --test native_oversized_committed_replace -- --nocapture.
//! Full declared behavior gate: cargo test -p lumen.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum_test::TestServer;

use lumen::aof::AofWriter;
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::log_entry::RaftLogEntry;
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, HammingQuery, MatchOp,
    MatchQuery, PrefixQuery, QueryNode, ReplaceDocItem, ReplaceDocsRequest, SearchRequest,
    TermQuery, MAX_BATCH_REPLACE_SIZE,
};
use lumen::wal::{MemWal, SharedWal, WalLog, WalRecord};

const COLLECTION: &str = "native-oversized-committed-replace";
const KEYWORD_FIELD: &str = "keyword";
const NUMBER_FIELD: &str = "number";
const TEXT_FIELD: &str = "body";
const SET_FIELD: &str = "tags";
const HASH_FIELD: &str = "hash";
const OBSOLETE_FIELD: &str = "obsolete";
const BASE_ID: &str = "native-replace-doc-00";
const DOC_COUNT: usize = 32;
const PER_DOCUMENT_KEYWORD_BYTES: usize = 8 * 1024 * 1024 + 512 * 1024 + 256;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const CREATE_SEQUENCE: u64 = 1;
const BASE_SEQUENCE: u64 = 2;
const GIANT_SEQUENCE: u64 = 3;
const BASE_VERSION: u64 = 10;
const GIANT_VERSION: u64 = 20;
const BASE_KEYWORD_PREFIX: &str = "native-replace-base-keyword-";
const OBSOLETE_PREFIX: &str = "native-replace-obsolete-field-";
const BASE_TEXT: &str = "native-replace-base-text";
const COMMON_TEXT: &str = "native-replace-common";
const SHARED_SET_VALUE: &str = "native-replace-shared-set";
const BASE_NUMBER: f64 = 1.0;
const GIANT_NUMBER_BASE: f64 = 1000.0;
const PUBLISH_WATCHDOG: Duration = Duration::from_secs(30);
const APPLY_WATCHDOG: Duration = Duration::from_secs(120);
const CHECKPOINT_WATCHDOG: Duration = Duration::from_secs(60);
const CHILD_CLEANUP_BOUND: Duration = Duration::from_secs(240);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_NATIVE_OVERSIZED_REPLACE_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_NATIVE_OVERSIZED_REPLACE_ROOT";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_NATIVE_OVERSIZED_REPLACE_HANDSHAKE";
const CHILD_CASE: &str = "native-committed-replace";
const TEST_NAME: &str = "native_committed_oversized_replace_applies_checkpoints_and_cold_opens";

struct Fixture {
    engine: Arc<Engine>,
    server: TestServer,
    store: Arc<SegmentRdbStore>,
    aof_path: PathBuf,
    wal: Arc<MemWal>,
    writer: Arc<WriteCoordinator>,
}

/// Owns the child on every parent unwind path. A child stuck holding a native
/// source or change-budget reservation cannot affect another test process.
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

fn fixture(root: &Path) -> Fixture {
    let aof_path = root.join("native-oversized-replace.aof");
    let checkpoint_root = root.join("segments");
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path).expect("open native committed ReplaceDocs AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let store = Arc::new(
        SegmentRdbStore::new(&checkpoint_root).expect("open native committed ReplaceDocs store"),
    );
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
    let sink_writer: Arc<dyn WriteSink> = writer.clone();
    let checkpoint: Arc<dyn CheckpointSink> = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: sink_writer.clone(),
        aof: Some(aof),
    });
    let state = AppState::with_components(
        engine.clone(),
        Arc::new(AuthConfig::open()),
        sink_writer,
    )
    .with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("open native committed ReplaceDocs server");

    Fixture {
        engine,
        server,
        store,
        aof_path,
        wal,
        writer,
    }
}

fn doc_id(index: usize) -> String {
    format!("native-replace-doc-{index:02}")
}

fn giant_keyword_prefix(index: usize) -> String {
    format!("native-replace-giant-keyword-{index:02}:")
}

fn giant_keyword(index: usize) -> String {
    let prefix = giant_keyword_prefix(index);
    assert!(
        prefix.len() < PER_DOCUMENT_KEYWORD_BYTES,
        "small public keyword prefix must leave a real approximately 8.5 MiB value",
    );
    let mut bytes = vec![b'x'; PER_DOCUMENT_KEYWORD_BYTES];
    bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
    String::from_utf8(bytes).expect("native oversized ReplaceDocs Keyword fixture is valid ASCII")
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
                format!("native-replace-set-{index:02}"),
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
            FieldValue::StringList(vec!["native-replace-base-set".to_owned()]),
        ),
        (HASH_FIELD.to_owned(), FieldValue::String("0x42".to_owned())),
        (
            OBSOLETE_FIELD.to_owned(),
            FieldValue::String(format!("{OBSOLETE_PREFIX}row")),
        ),
    ])
}

fn create_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: schema(),
    })
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
        "the native fixture must use the unchanged public docs:replace maximum",
    );
    assert!(
        PER_DOCUMENT_KEYWORD_BYTES
            .checked_mul(DOC_COUNT)
            .expect("native fixture byte multiplication")
            > PENDING_HARD_LIMIT_BYTES as usize,
        "32 approximately 8.5 MiB Keyword values must exceed the documented 256 MiB budget",
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

/// This performs one fixture-only encode before native publication, then drops
/// it. `ReplaceDocs` has no fast wire tag, so this proves the source that the
/// eventual native stager receives is an actual generic-CBOR payload.
fn assert_generic_cbor_source_payload(record: &WalRecord) {
    let payload = record
        .encode()
        .expect("encode real native ReplaceDocs source through its public WAL codec");
    assert!(
        !payload.starts_with(b"LWAL"),
        "ReplaceDocs must retain its generic-CBOR source form rather than use the fast Index wire",
    );
    assert!(
        payload.len() > PENDING_HARD_LIMIT_BYTES as usize,
        "the actual generic-CBOR native source payload must exceed 256 MiB: payload_bytes={}",
        payload.len(),
    );
    drop(payload);
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

fn obsolete_ids(engine: &Engine) -> BTreeSet<String> {
    prefix_ids_for_field(engine, OBSOLETE_FIELD, OBSOLETE_PREFIX)
}

fn assert_base_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("native ReplaceDocs base collection stats")
            .documents_indexed,
        1,
        "{phase}: the native committed base must contain exactly one document",
    );
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX),
        singleton(0),
        "{phase}: the base Keyword must remain query-visible before replacement",
    );
    assert_eq!(
        term_ids(engine, NUMBER_FIELD, FieldValue::Number(BASE_NUMBER)),
        singleton(0),
        "{phase}: the base Number must remain query-visible before replacement",
    );
    assert_eq!(
        obsolete_ids(engine),
        singleton(0),
        "{phase}: the base-only obsolete field must exist before full replacement",
    );
}

fn assert_oversized_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("native oversized ReplaceDocs collection stats")
            .documents_indexed,
        DOC_COUNT as u64,
        "{phase}: the committed oversized ReplaceDocs batch must expose all 32 documents",
    );
    assert_eq!(
        match_ids(engine, COMMON_TEXT),
        all_giant_ids(),
        "{phase}: all native committed replacement document IDs must be query-visible",
    );
    assert_eq!(
        term_ids(
            engine,
            SET_FIELD,
            FieldValue::String(SHARED_SET_VALUE.to_owned()),
        ),
        all_giant_ids(),
        "{phase}: every full replacement Set value must be query-visible once per document",
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
            "{phase}: oversized Keyword {index} must retain every original byte, not only its prefix",
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
        "{phase}: fields absent from the native ReplaceDocs item must be deleted, not merged",
    );
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX),
        BTreeSet::new(),
        "{phase}: the base Keyword must not survive the complete replacement",
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

async fn assert_public_pending_budget(server: &TestServer, phase: &str) {
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

async fn publish_committed(wal: &MemWal, record: WalRecord, phase: &str) -> u64 {
    let published = tokio::time::timeout(PUBLISH_WATCHDOG, wal.publish(record)).await;
    assert!(
        published.is_ok(),
        "{phase}: native MemWal publish must return a committed sequence before test cleanup",
    );
    let published = published.expect("behavior assertion above checked native publish watchdog");
    let diagnostic = published
        .as_ref()
        .err()
        .map(|error| format!("{error:#}"))
        .unwrap_or_default();
    assert!(
        published.is_ok(),
        "{phase}: a valid native committed record must not receive an error acknowledgement: {diagnostic}",
    );
    published.expect("behavior assertion above checked native committed acknowledgement")
}

#[derive(Debug)]
enum ApplyProgress {
    Applied,
    RestartRequired,
}

async fn wait_for_committed_apply(fixture: &Fixture, sequence: u64, phase: &str) {
    let progress = tokio::time::timeout(APPLY_WATCHDOG, async {
        loop {
            if fixture.writer.applied_seq() >= sequence {
                return ApplyProgress::Applied;
            }
            if fixture.writer.is_restart_required() {
                return ApplyProgress::RestartRequired;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await;
    let wal_head = fixture
        .wal
        .latest_seq()
        .await
        .expect("read native MemWal head after committed publication");
    assert!(
        matches!(&progress, Ok(ApplyProgress::Applied)),
        "{phase}: a valid native committed generic-CBOR ReplaceDocs record must reach sequence {sequence}; progress={progress:?}, applied_sequence={}, wal_head={wal_head}, restart_required={}",
        fixture.writer.applied_seq(),
        fixture.writer.is_restart_required(),
    );
}

async fn checkpoint_now(server: &TestServer) {
    let response = tokio::time::timeout(CHECKPOINT_WATCHDOG, async {
        server.post("/admin/checkpoint").await
    })
    .await;
    assert!(
        response.is_ok(),
        "the real native committed ReplaceDocs checkpoint must finish before test cleanup",
    );
    let response = response.expect("behavior assertion above checked checkpoint watchdog");
    response.assert_status_ok();
    assert_eq!(
        response.json::<serde_json::Value>()["persisted"],
        true,
        "the real native committed ReplaceDocs checkpoint must publish durable state",
    );
}

fn child_paths() -> Option<(PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("native committed ReplaceDocs child needs a parent-owned durable root");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("native committed ReplaceDocs child needs a handshake path");
    Some((root, handshake))
}

async fn run_native_committed_replace_child(root: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact native ReplaceDocs child entry");
    let fixture = fixture(&root);

    let create_sequence = publish_committed(&fixture.wal, create_record(), "collection schema").await;
    assert_eq!(
        create_sequence, CREATE_SEQUENCE,
        "the schema must be the first native committed MemWal record",
    );
    wait_for_committed_apply(&fixture, create_sequence, "collection schema").await;

    let base_sequence = publish_committed(&fixture.wal, base_record(), "full-document base").await;
    assert_eq!(
        base_sequence, BASE_SEQUENCE,
        "the small complete-document base must follow the native schema record",
    );
    wait_for_committed_apply(&fixture, base_sequence, "full-document base").await;
    assert_base_state(&fixture.engine, "live native committed ReplaceDocs base");
    checkpoint_now(&fixture.server).await;
    let base_cold = fixture
        .store
        .load_current_generation()
        .expect("cold-open native committed ReplaceDocs base checkpoint")
        .expect("native committed ReplaceDocs base checkpoint must publish CURRENT");
    assert_eq!(
        base_cold.sequence, BASE_SEQUENCE,
        "cold native base checkpoint must retain the established watermark",
    );
    assert_base_state(&base_cold.engine, "cold native committed ReplaceDocs base checkpoint");

    let giant_record = oversized_replace_record();
    assert_generic_cbor_source_payload(&giant_record);
    let aof_before = fs::metadata(&fixture.aof_path)
        .expect("measure native committed AOF before generic-CBOR ReplaceDocs")
        .len();
    let giant_sequence = publish_committed(
        &fixture.wal,
        giant_record,
        "oversized generic-CBOR ReplaceDocs",
    )
    .await;
    assert_eq!(
        giant_sequence, GIANT_SEQUENCE,
        "the oversized ReplaceDocs source must become the next committed native MemWal record",
    );
    wait_for_committed_apply(&fixture, giant_sequence, "oversized generic-CBOR ReplaceDocs").await;
    assert_eq!(
        fixture.writer.applied_seq(),
        giant_sequence,
        "the applied watermark must equal the committed oversized ReplaceDocs sequence",
    );
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read native MemWal after oversized ReplaceDocs apply"),
        giant_sequence,
        "the native WAL head must retain the committed oversized ReplaceDocs sequence",
    );
    assert!(
        !fixture.writer.is_restart_required(),
        "a valid native committed generic-CBOR ReplaceDocs record must not leave the coordinator restart-required",
    );
    assert!(
        fs::metadata(&fixture.aof_path)
            .expect("measure AOF after native committed ReplaceDocs apply")
            .len()
            > aof_before,
        "WriteCoordinator must append the applied foreign committed ReplaceDocs record to its real AOF before checkpoint",
    );
    assert_oversized_state(&fixture.engine, "live native committed oversized ReplaceDocs apply");
    assert_public_pending_budget(&fixture.server, "live native committed oversized ReplaceDocs apply").await;

    checkpoint_now(&fixture.server).await;
    assert_public_pending_budget(&fixture.server, "native committed oversized ReplaceDocs checkpoint").await;
    let cold = fixture
        .store
        .load_current_generation()
        .expect("cold-open native committed oversized ReplaceDocs checkpoint")
        .expect("native committed oversized ReplaceDocs checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, GIANT_SEQUENCE,
        "cold native checkpoint must retain the applied oversized ReplaceDocs watermark",
    );
    assert_oversized_state(&cold.engine, "cold native committed oversized ReplaceDocs checkpoint");
}

async fn run_isolated_child(root: &Path) {
    let child_root = tempfile::tempdir().expect("native committed ReplaceDocs child workspace");
    let child_tmp = child_root.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned native ReplaceDocs child temporary directory");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable = std::env::current_exe().expect("current native ReplaceDocs test executable");
    let stdout = File::create(&stdout_path).expect("create native ReplaceDocs child stdout");
    let stderr = File::create(&stderr_path).expect("create native ReplaceDocs child stderr");
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
        .expect("spawn isolated native committed ReplaceDocs child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_CLEANUP_BOUND;
    loop {
        match child
            .0
            .as_mut()
            .expect("native ReplaceDocs child remains owned until exit")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child
                    .0
                    .take()
                    .expect("timed-out native ReplaceDocs child remains owned");
                let _ = raw.kill();
                let status = raw
                    .wait()
                    .expect("wait for killed native ReplaceDocs child");
                let stdout = fs::read_to_string(&stdout_path)
                    .expect("read killed native ReplaceDocs child stdout");
                let stderr = fs::read_to_string(&stderr_path)
                    .expect("read killed native ReplaceDocs child stderr");
                panic!(
                    "a valid committed 32-document >256 MiB native ReplaceDocs record must apply and checkpoint before cleanup; child was killed after {CHILD_CLEANUP_BOUND:?}: status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated native ReplaceDocs child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited native ReplaceDocs child remains owned")
        .wait()
        .expect("wait for exited native ReplaceDocs child");
    let stdout = fs::read_to_string(&stdout_path).expect("read native ReplaceDocs child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read native ReplaceDocs child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated native ReplaceDocs child did not enter exact {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must enter the intended native committed ReplaceDocs body",
    );
    assert!(
        status.success(),
        "isolated native committed ReplaceDocs child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_committed_oversized_replace_applies_checkpoints_and_cold_opens() {
    if let Some((root, handshake)) = child_paths() {
        run_native_committed_replace_child(root, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("native committed oversized ReplaceDocs fixture root");
    run_isolated_child(root.path()).await;
}
