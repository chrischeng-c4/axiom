//! # Facets
//!
//! - Behavior: `indexing_durable_oracle.rs:815`, `:866`, `:874`, `:1042`,
//!   `:1063`, `:1071`, `:1087`, `:1359`, `:1365`, `:1436`, `:1441`, `:1446`,
//!   `:1453`, `:1462`, `:1496`, `:1529`, `:1553`, `:1563`, `:1597`, `:1601`,
//!   `:1652`, `:1657`, `:1663`, `:1390`, and helper `:1259` pin format, cold
//!   open, hard-link reuse, epoch allocation, provenance, and retained data.
//! - Security: `apps/lumen/src/segment_rdb.rs:1225` reads persisted catalog input;
//!   `indexing_durable_oracle.rs:968`, `:973`, `:977`, `:1120`, `:1124`, and
//!   `:1128` refuse malformed catalog bytes without changing `CURRENT`.
//! - Performance: `indexing_durable_oracle.rs:1359` checks reuse behavior only.
//!   It does not measure latency or RSS. The approved stage6 30-minute release
//!   workload covers checkpoint and merge budgets.
//!   This bounded oracle does not replace that release gate.
//! - Behavior (partial-error recovery): `indexing_durable_oracle.rs:8583` requires
//!   an error-returning committed record's live Keyword prefix to replay, and
//!   `:8676` requires cold checkpoint recovery to retain the live Number deletion.
//! - Security (persisted-input boundary): `apps/lumen/src/coordinator.rs:377`
//!   controls the AOF bytes this process later reads; `indexing_durable_oracle.rs:8583`
//!   rejects silently losing an error-returning applied record. `apps/lumen/src/storage.rs:4981`
//!   drops a caller-supplied value before fallible validation; `:8676` refuses to
//!   honor the stale persisted Number after a cold reopen.
//! - Performance (existing gate): these cases change recovery correctness at
//!   `apps/lumen/src/coordinator.rs:377` and `apps/lumen/src/storage.rs:4981`.
//!   They make no latency or RSS claim. The existing stage6 durable workload at
//!   `apps/lumen/e2e/perf_gate.rs:2865` measures checkpoint and merge work.

use std::collections::{BTreeMap, HashMap};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use axum_test::TestServer;
use futures::StreamExt;
use serde_json::{json, Map, Value};

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::log_entry::RaftLogEntry;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::{Engine, FieldIndexSnapshot, SnapshotV1};
use lumen::types::{
    FieldValue, MatchOp, MatchQuery, QueryNode, RangeBound, RangeQuery, SearchRequest, TermQuery,
    TermsQuery,
};
use lumen::wal::{MemWal, SharedWal, WalLog};

const COLLECTION: &str = "docs";
const COMMON: &str = "common";
const RARE: &str = "rare";
const OTHER: &str = "other";
const DOCUMENTS: usize = 476;
const PREFIX_DOCUMENTS: usize = 365;

struct LocalCheckpointSink {
    engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    writer: Arc<WriteCoordinator>,
    aof: SharedAof,
}

#[async_trait]
impl CheckpointSink for LocalCheckpointSink {
    async fn checkpoint_now(&self) -> Result<bool> {
        let _permit = self.writer.mutation_gate().shared().await?;
        let sequence = self.writer.applied_seq();
        self.store.save(&self.engine, sequence)?;
        self.store.prune(3)?;
        self.aof
            .lock()
            .map_err(|_| anyhow::anyhow!("aof writer poisoned"))?
            .truncate_through(sequence)?;
        Ok(true)
    }
}

struct Fixture {
    _dir: tempfile::TempDir,
    checkpoint_root: std::path::PathBuf,
    server: TestServer,
    engine: Arc<Engine>,
    writer: Arc<WriteCoordinator>,
    aof: SharedAof,
    aof_path: std::path::PathBuf,
    store: Arc<SegmentRdbStore>,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("fixture directory");
    let checkpoint_root = dir.path().join("segments");
    let aof_path = dir.path().join("aof.log");
    let store = Arc::new(SegmentRdbStore::new(&checkpoint_root).expect("segment store"));
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(&aof_path).expect("aof")));
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::new());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let checkpoint = Arc::new(LocalCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: writer.clone(),
        aof: aof.clone(),
    });
    let state = AppState::with_components(
        engine.clone(),
        Arc::new(AuthConfig::open()),
        writer.clone() as Arc<dyn WriteSink>,
    )
    .with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("test server");
    Fixture {
        _dir: dir,
        checkpoint_root,
        server,
        engine,
        writer,
        aof,
        aof_path,
        store,
    }
}

fn external_id(id: usize) -> String {
    format!("doc-{id:03}")
}

fn field_items(ids: impl Iterator<Item = usize>, field: &str) -> Vec<Value> {
    ids.map(|id| {
        let value = match field {
            "kw" if id < 466 => COMMON,
            "kw" if id == 475 => RARE,
            "kw" => OTHER,
            "num" => {
                return json!({
                    "external_id": external_id(id),
                    "field": field,
                    "value": id,
                })
            }
            "body" => {
                return json!({
                    "external_id": external_id(id),
                    "field": field,
                    "value": format!("body {id}"),
                })
            }
            _ => unreachable!("fixture field"),
        };
        json!({
            "external_id": external_id(id),
            "field": field,
            "value": value,
        })
    })
    .collect()
}

fn corpus_items(field_major: bool, end: usize) -> Vec<Value> {
    corpus_items_range(field_major, 0, end)
}

fn corpus_items_range(field_major: bool, start: usize, end: usize) -> Vec<Value> {
    if field_major {
        ["kw", "num", "body"]
            .into_iter()
            .flat_map(|field| field_items(start..end, field))
            .collect()
    } else {
        (start..end)
            .flat_map(|id| {
                ["kw", "num", "body"]
                    .into_iter()
                    .map(move |field| field_items(std::iter::once(id), field).pop().unwrap())
            })
            .collect()
    }
}

async fn create_schema(server: &TestServer) {
    server
        .put("/collections/docs")
        .json(&json!({
            "fields": {
                "kw": { "type": "keyword" },
                "num": { "type": "number" },
                "body": { "type": "text", "analyzer": "whitespace_lower" }
            }
        }))
        .await
        .assert_status_ok();
}

async fn post_index(server: &TestServer, items: Vec<Value>) {
    for chunk in items.chunks(1000) {
        server
            .post("/collections/docs/index")
            .json(&json!({ "items": chunk }))
            .await
            .assert_status_ok();
    }
}

async fn index_updates(server: &TestServer) {
    server
        .post("/collections/docs/index")
        .json(&json!({ "items": [
            { "external_id": "doc-466", "field": "kw", "value": "台北市/大安區" },
            { "external_id": "doc-466", "field": "num", "value": 9001 },
            { "external_id": "doc-466", "field": "body", "value": "éclair 🙂" }
        ]}))
        .await
        .assert_status_ok();

    server
        .put("/collections/docs/docs:replace")
        .json(&json!({ "docs": [{
            "external_id": "doc-467",
            "fields": { "num": 9002, "body": "replacement body" }
        }]}))
        .await
        .assert_status_ok();

    server
        .post("/collections/docs/index")
        .json(&json!({ "items": [
            { "external_id": "doc-468", "field": "kw", "value": "🙂" },
            { "external_id": "doc-468", "field": "body", "value": "台北市" }
        ]}))
        .await
        .assert_status_ok();
}

async fn checkpoint(server: &TestServer) {
    let response = server.post("/admin/checkpoint").await;
    response.assert_status_ok();
    assert_eq!(response.json::<Value>()["persisted"], true);
}

/// Drive a search through the public HTTP route.  The case deliberately does
/// not inspect a recovered index's private postings: callers only observe the
/// recovered value through the same search request they used before the
/// checkpoint.
async fn http_search(server: &TestServer, query: Value, context: &str) -> Value {
    let response = server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({ "query": query, "limit": 500 }))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert!(
        body["total"].is_u64(),
        "{context}: search response must expose a numeric total: {body}"
    );
    body
}

fn hit_ids(body: &Value) -> Vec<&str> {
    body["hits"]
        .as_array()
        .expect("search hits are an array")
        .iter()
        .map(|hit| hit["external_id"].as_str().expect("hit external id"))
        .collect()
}

fn snapshot(engine: &Arc<Engine>) -> SnapshotV1 {
    engine.snapshot().expect("engine snapshot")
}

fn collection(snapshot: &SnapshotV1) -> &lumen::storage::CollectionSnapshot {
    snapshot
        .collections
        .get(COLLECTION)
        .expect("docs collection")
}

/// The keyword field's forward column. As of snapshot format 2 it is the ONLY
/// representation on the wire: `from_snapshot` rebuilds the inverted index from
/// it, so the `terms` map this helper used to return alongside was a second
/// copy the reader discarded on arrival.
fn keyword_forward<'a>(snapshot: &'a SnapshotV1, field: &str) -> &'a HashMap<String, String> {
    let index = collection(snapshot)
        .fields
        .get(field)
        .expect("keyword field");
    let FieldIndexSnapshot::Keyword { forward, .. } = index else {
        panic!("{field} must be keyword")
    };
    forward
}

/// How many documents the snapshot puts under `value` — the posting count the
/// restored index will hold, counted where it now lives.
fn keyword_postings(snapshot: &SnapshotV1, field: &str, value: &str) -> usize {
    keyword_forward(snapshot, field)
        .values()
        .filter(|held| *held == value)
        .count()
}

fn number_forward<'a>(snapshot: &'a SnapshotV1, field: &str) -> &'a HashMap<String, f64> {
    let index = collection(snapshot)
        .fields
        .get(field)
        .expect("number field");
    let FieldIndexSnapshot::Number { forward, .. } = index else {
        panic!("{field} must be number")
    };
    forward
}

fn canonical(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonical).collect()),
        Value::Object(values) => {
            let mut sorted = Map::new();
            let mut entries: Vec<_> = values.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            for (key, value) in entries {
                sorted.insert(key, canonical(value));
            }
            Value::Object(sorted)
        }
        other => other,
    }
}

fn digest(engine: &Arc<Engine>) -> Value {
    canonical(logical_snapshot_value(snapshot(engine)))
}

fn logical_snapshot_value(snapshot: SnapshotV1) -> Value {
    fn normalize(value: Value) -> Value {
        match value {
            Value::Array(values) => Value::Array(values.into_iter().map(normalize).collect()),
            Value::Object(mut values) => {
                if values.get("type") == Some(&Value::String("Keyword".into())) {
                    if let Some(Value::Object(forward)) = values.get("forward") {
                        let mut terms: BTreeMap<String, Vec<String>> = BTreeMap::new();
                        for (eid, value) in forward {
                            if let Some(value) = value.as_str() {
                                terms.entry(value.to_owned()).or_default().push(eid.clone());
                            }
                        }
                        for eids in terms.values_mut() {
                            eids.sort();
                        }
                        values.insert(
                            "terms".into(),
                            serde_json::to_value(terms).expect("keyword terms"),
                        );
                    }
                }
                values.remove("bytes");
                Value::Object(
                    values
                        .into_iter()
                        .map(|(key, value)| (key, normalize(value)))
                        .collect(),
                )
            }
            other => other,
        }
    }
    normalize(serde_json::to_value(snapshot).expect("snapshot json"))
}

fn assert_index_invariants(engine: &Arc<Engine>) {
    let snap = snapshot(engine);
    let docs = collection(&snap);
    assert_eq!(docs.eid_fields.len(), DOCUMENTS);
    let forward = keyword_forward(&snap, "kw");
    // #3957: the snapshot must carry every doc's keyword, sealed or not.
    // `COMMON` is the only one of the three that straddles the seal boundary —
    // 365 of its docs are sealed, 101 are in the live tail — and before the fix
    // the persisted inverted index held exactly those 101 while `forward` held
    // all 466. `RARE` and `OTHER` never disagreed, which is why the discrepancy
    // read as a fixture detail rather than as data loss. The two maps were one
    // field seen twice; format 2 ships `forward` alone, so the disagreement is
    // now unrepresentable and these counts are the whole contract.
    assert_eq!(keyword_postings(&snap, "kw", COMMON), 466);
    assert_eq!(keyword_postings(&snap, "kw", RARE), 1);
    assert_eq!(keyword_postings(&snap, "kw", OTHER), 6);
    assert_eq!(forward.get("doc-466"), Some(&"台北市/大安區".to_string()));
    assert_eq!(forward.get("doc-468"), Some(&"🙂".to_string()));
    // doc-467 was deleted. With `forward` the only representation, its absence
    // here is the whole statement — there is no second map left to hold a
    // posting the forward column no longer backs.
    assert!(!forward.contains_key("doc-467"));

    let numbers = number_forward(&snap, "num");
    assert_eq!(numbers.get("doc-466"), Some(&9001.0));
    assert_eq!(numbers.get("doc-467"), Some(&9002.0));
}

fn sorted_hit_ids(response: lumen::types::SearchResponse) -> Vec<String> {
    let mut ids: Vec<_> = response
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect();
    ids.sort();
    ids
}

fn assert_keyword_total(engine: &Arc<Engine>, value: &str, expected: u64) {
    let response = engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: "kw".into(),
                    value: FieldValue::String(value.into()),
                }),
                limit: 500,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("keyword count query");
    assert_eq!(response.total, expected);
}

fn assert_queries(engine: &Arc<Engine>) {
    assert_keyword_total(engine, COMMON, 466);
    assert_keyword_total(engine, RARE, 1);
    assert_keyword_total(engine, OTHER, 6);
    let term = engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: "kw".into(),
                    value: FieldValue::String("台北市/大安區".into()),
                }),
                limit: 20,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("keyword term query");
    assert_eq!(term.total, 1);
    assert_eq!(sorted_hit_ids(term), vec!["doc-466"]);

    let terms = engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Terms(TermsQuery {
                    field: "kw".into(),
                    values: vec![
                        FieldValue::String("台北市/大安區".into()),
                        FieldValue::String(RARE.into()),
                        FieldValue::String("🙂".into()),
                    ],
                }),
                limit: 20,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("keyword terms query");
    assert_eq!(terms.total, 3);
    assert_eq!(sorted_hit_ids(terms), vec!["doc-466", "doc-468", "doc-475"]);

    let exact_number = engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: "num".into(),
                    value: FieldValue::Number(9001.0),
                }),
                limit: 500,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("number term query");
    assert_eq!(exact_number.total, 1);
    assert_eq!(sorted_hit_ids(exact_number), vec!["doc-466"]);

    let number_range = engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Range(RangeQuery {
                    field: "num".into(),
                    gt: None,
                    gte: Some(RangeBound::Number(9001.0)),
                    lt: None,
                    lte: Some(RangeBound::Number(9002.0)),
                }),
                limit: 20,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("number range query");
    assert_eq!(number_range.total, 2);
    assert_eq!(sorted_hit_ids(number_range), vec!["doc-466", "doc-467"]);

    let text = engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Match(MatchQuery {
                    field: "body".into(),
                    text: "body".into(),
                    op: MatchOp::And,
                }),
                limit: 500,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("text query");
    let expected_text_ids: Vec<String> = (0..DOCUMENTS)
        .filter(|id| !matches!(*id, 466 | 468))
        .map(external_id)
        .collect();
    assert_eq!(text.total, expected_text_ids.len() as u64);
    assert_eq!(sorted_hit_ids(text), expected_text_ids);

    for (text, expected_id) in [("éclair", "doc-466"), ("台北市", "doc-468")] {
        let utf8_text = engine
            .search(
                COLLECTION,
                SearchRequest {
                    query: QueryNode::Match(MatchQuery {
                        field: "body".into(),
                        text: text.into(),
                        op: MatchOp::And,
                    }),
                    limit: 20,
                    offset: 0,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .expect("UTF-8 text query");
        assert_eq!(utf8_text.total, 1);
        assert_eq!(sorted_hit_ids(utf8_text), vec![expected_id]);
    }
}

fn recover_from_checkpoint(fixture: &Fixture) -> (Arc<Engine>, u64, u64) {
    let loaded = fixture
        .store
        .load_current_generation()
        .expect("load CURRENT")
        .expect("checkpoint generation");
    let checkpoint_sequence = loaded.sequence;
    let replayed = replay_aof_into(&loaded.engine, &fixture.aof_path, checkpoint_sequence)
        .expect("replay AOF tail");
    (loaded.engine, checkpoint_sequence, replayed)
}

async fn run_history(field_major: bool) -> Value {
    let fixture = fixture();
    create_schema(&fixture.server).await;
    post_index(&fixture.server, corpus_items(field_major, PREFIX_DOCUMENTS)).await;

    let prefix_snapshot = snapshot(&fixture.engine);
    assert_eq!(
        keyword_postings(&prefix_snapshot, "kw", COMMON),
        PREFIX_DOCUMENTS
    );
    let first_sequence = fixture.writer.applied_seq();
    checkpoint(&fixture.server).await;
    let first = fixture
        .store
        .load_current_generation()
        .expect("load first CURRENT")
        .expect("first checkpoint");
    assert_eq!(first.sequence, first_sequence);
    assert_eq!(
        collection(&snapshot(&first.engine)).eid_fields.len(),
        PREFIX_DOCUMENTS
    );
    let first_snapshot = snapshot(&first.engine);
    // This asserted the postings map was EMPTY until #3957, under the reading
    // that a sealed field's postings "move out of the RAM snapshot". They do
    // move out of RAM — that is what the seal is for, and the reopened engine
    // below still answers because it holds the `.lseg`. But `to_snapshot` is
    // not a view of RAM: it is the self-contained document `GET /admin/backup`
    // returns, that `raft_sm` ships to a follower on another host, and that
    // `reshard` moves between shards. None of those readers has this node's
    // segment files, and `from_snapshot` sets `segment: None` — so a snapshot
    // truncated to the post-seal tail silently loses every sealed doc's
    // postings on the far side. This line is where that was pinned as intended.
    assert_eq!(
        keyword_postings(&first_snapshot, "kw", COMMON),
        PREFIX_DOCUMENTS,
        "#3957: the snapshot is self-contained, so a SEALED field's documents \
         must be in it and not only in the `.lseg` its reader may not have"
    );
    assert_keyword_total(&first.engine, COMMON, PREFIX_DOCUMENTS as u64);

    post_index(
        &fixture.server,
        corpus_items_range(field_major, PREFIX_DOCUMENTS, DOCUMENTS),
    )
    .await;
    index_updates(&fixture.server).await;
    fixture
        .aof
        .lock()
        .expect("aof lock")
        .sync_strict()
        .expect("strict-sync AOF");

    let live_digest = digest(&fixture.engine);
    assert_index_invariants(&fixture.engine);
    assert_queries(&fixture.engine);
    let (tail_reopened, _checkpoint_sequence, replayed) = recover_from_checkpoint(&fixture);
    assert!(
        replayed > first_sequence,
        "AOF replay must advance beyond boundary"
    );
    assert_eq!(digest(&tail_reopened), live_digest);
    assert_index_invariants(&tail_reopened);
    assert_queries(&tail_reopened);

    checkpoint(&fixture.server).await;
    let sealed = fixture
        .store
        .load_current_generation()
        .expect("load final CURRENT")
        .expect("final checkpoint");
    assert_eq!(sealed.sequence, fixture.writer.applied_seq());
    assert_eq!(digest(&sealed.engine), live_digest);
    assert_index_invariants(&sealed.engine);
    assert_queries(&sealed.engine);
    let (final_reopened, final_sequence, final_replayed) = recover_from_checkpoint(&fixture);
    assert_eq!(final_sequence, sealed.sequence);
    assert_eq!(final_replayed, 0);
    assert_eq!(digest(&final_reopened), live_digest);
    assert_index_invariants(&final_reopened);
    assert_queries(&final_reopened);
    live_digest
}

#[tokio::test]
async fn indexing_durable_oracle_converges_across_input_layouts() {
    let document_major = run_history(false).await;
    let field_major = run_history(true).await;
    assert_eq!(document_major, field_major);
}

/// A field that was sealed in an earlier segment generation remains mutable.
///
/// This is a public durability contract for #4164. A caller indexes a base
/// document, persists it with `/admin/checkpoint`, replaces the keyword through
/// the HTTP index route, and checkpoints again. A new `SegmentRdbStore` then
/// cold-opens `CURRENT`. Its HTTP search surface must expose only the new
/// keyword, retain `exists`, and still allow a complete document delete. The
/// final delete checks that no stale posting from the sealed base can return
/// after a cold reopen.
#[tokio::test]
async fn sealed_keyword_update_survives_checkpoint_and_cold_reopen() {
    const BASE: &str = "doc-000";
    const BEFORE: &str = "sealed-base-before";
    const AFTER: &str = "sealed-base-after";

    let fixture = fixture();
    create_schema(&fixture.server).await;

    // Use a corpus-sized base so the first checkpoint seals an ordinary
    // keyword segment rather than testing only an empty or tail-only field.
    post_index(&fixture.server, corpus_items(false, PREFIX_DOCUMENTS)).await;
    fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [{
            "external_id": BASE,
            "field": "kw",
            "value": BEFORE
        }] }))
        .await
        .assert_status_ok();
    checkpoint(&fixture.server).await;

    // This targets a value that existed in the sealed base generation. The
    // second checkpoint must materialize its replacement, not resurrect the
    // old segment posting during a later cold open.
    fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [{
            "external_id": BASE,
            "field": "kw",
            "value": AFTER
        }] }))
        .await
        .assert_status_ok();
    checkpoint(&fixture.server).await;

    // Re-open the active immutable generation through a new store instance.
    // This rules out the live engine and its in-memory segment handles as the
    // oracle: only persisted `CURRENT` content reaches this server.
    let cold_store = SegmentRdbStore::new(&fixture.checkpoint_root)
        .expect("open checkpoint root for cold recovery");
    let cold = cold_store
        .load_current_generation()
        .expect("load CURRENT after second checkpoint")
        .expect("second checkpoint created a generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine.clone())))
        .expect("cold recovered HTTP server");

    let new_value = http_search(
        &cold_server,
        json!({ "term": { "field": "kw", "value": AFTER } }),
        "cold reopen new keyword",
    )
    .await;
    assert_eq!(
        new_value["total"], 1,
        "cold reopen must retain the replacement keyword for the sealed base"
    );
    assert_eq!(hit_ids(&new_value), vec![BASE]);

    let old_value = http_search(
        &cold_server,
        json!({ "term": { "field": "kw", "value": BEFORE } }),
        "cold reopen old keyword",
    )
    .await;
    assert_eq!(
        old_value["total"], 0,
        "cold reopen must not resurrect the sealed base keyword that the caller replaced"
    );

    let exists = http_search(
        &cold_server,
        json!({ "exists": { "field": "kw" } }),
        "cold reopen keyword exists",
    )
    .await;
    assert_eq!(
        exists["total"], PREFIX_DOCUMENTS as u64,
        "the updated base must remain present in the keyword exists set"
    );
    assert!(
        hit_ids(&exists).contains(&BASE),
        "the updated sealed base must satisfy exists after cold reopen"
    );

    cold_server
        .delete(&format!("/collections/{COLLECTION}/docs/{BASE}"))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    let deleted = http_search(
        &cold_server,
        json!({ "term": { "field": "kw", "value": AFTER } }),
        "delete after cold reopen",
    )
    .await;
    assert_eq!(
        deleted["total"], 0,
        "a complete document delete must remove the recovered sealed posting"
    );
}

/// Stage 1 begins with a physical format boundary. A caller writes a small
/// collection, requests the normal checkpoint route, and then reads only the
/// committed generation selected by `CURRENT`. Version 1 contains sequence and
/// predecessor facts only. Version 2 must begin the complete catalog contract.
#[tokio::test]
async fn checkpoint_generation_manifest_is_v2_complete_catalog() {
    let fixture = fixture();
    create_schema(&fixture.server).await;
    post_index(&fixture.server, field_items(0..1, "kw")).await;
    let checkpoint_sequence = fixture.writer.applied_seq();

    checkpoint(&fixture.server).await;

    let current = std::fs::read_to_string(fixture.checkpoint_root.join("CURRENT"))
        .expect("read committed CURRENT pointer");
    let generation = current
        .strip_prefix("generation:")
        .expect("CURRENT names a generation")
        .trim();
    assert!(
        !generation.is_empty(),
        "CURRENT must name the committed checkpoint generation"
    );
    let manifest_path = fixture
        .checkpoint_root
        .join(generation)
        .join("_generation.json");
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(&manifest_path).expect("read committed generation manifest"),
    )
    .expect("decode committed generation manifest");

    assert_eq!(
        manifest["schema_version"],
        json!(2),
        "stage1: a checkpoint after the shipped v1 format must publish the v2 complete catalog"
    );
    assert_eq!(
        manifest["checkpoint_sequence"],
        json!(checkpoint_sequence),
        "v2 catalog must carry the checkpoint watermark"
    );
    assert!(
        manifest["collections"].is_array(),
        "v2 catalog must list every live collection"
    );
}

/// Construct bytes in the exact v0.6.0 revision-generation format. This is
/// deliberately not a current writer round-trip: it pins the reader's upgrade
/// boundary to the fields that shipped before the v2 catalog existed.
#[tokio::test]
async fn shipped_v060_v1_revision_generation_cold_loads_before_v2_upgrade() {
    let dir = tempfile::tempdir().expect("v0.6.0 fixture directory");
    let root = dir.path().join("segments");
    let generation_name = "gen-41-rev-1";
    let generation = root.join(generation_name);
    let source = Arc::new(Engine::new());
    let source_server =
        TestServer::new(router(AppState::open(source.clone()))).expect("v0.6.0 source HTTP server");
    create_schema(&source_server).await;
    post_index(&source_server, field_items(0..1, "kw")).await;

    std::fs::create_dir_all(&generation).expect("create v0.6.0 generation");
    source
        .flush_to_segments(&generation, 41)
        .expect("write v0.6.0 segment payload");
    let mut manifest = serde_json::to_vec_pretty(&json!({
        "schema_version": 1,
        "sequence": 41,
        "revision": 1,
        "previous": Value::Null,
    }))
    .expect("encode v0.6.0 generation manifest");
    manifest.push(b'\n');
    std::fs::write(generation.join("_generation.json"), manifest)
        .expect("write v0.6.0 generation manifest");
    std::fs::write(
        root.join("CURRENT"),
        format!("generation:{generation_name}\n"),
    )
    .expect("point CURRENT at v0.6.0 generation");

    let store = SegmentRdbStore::new(&root).expect("open v0.6.0 checkpoint root");
    let loaded = store.load_current_generation();
    let load_error = loaded
        .as_ref()
        .err()
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        loaded.is_ok(),
        "stage1: the exact v0.6.0 v1 generation must cold-load before a v2 save; got {}",
        load_error
    );
    let loaded = loaded
        .expect("v0.6.0 current load succeeds")
        .expect("v0.6.0 CURRENT names a generation");
    assert_eq!(loaded.sequence, 41, "v0.6.0 sequence survives cold load");
    let cold_server =
        TestServer::new(router(AppState::open(loaded.engine))).expect("v0.6.0 cold HTTP server");
    assert_eq!(
        http_search(
            &cold_server,
            json!({ "term": { "field": "kw", "value": COMMON } }),
            "v0.6.0 cold keyword query",
        )
        .await["total"],
        1,
        "the v0.6.0 cold-loaded collection must still answer public searches"
    );
}

async fn stage1_v2_root_with_predecessor() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("v2 catalog fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open v2 catalog fixture root");
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone())))
        .expect("v2 catalog fixture HTTP server");
    create_schema(&server).await;
    post_index(&server, field_items(0..1, "kw")).await;
    stage1_restore_legacy_base(&engine, "v2 catalog predecessor");
    store
        .save(&engine, 42)
        .expect("write v2 predecessor generation");
    store
        .save(&engine, 43)
        .expect("write v2 CURRENT generation");
    (dir, root)
}

fn stage1_current_generation_dir(root: &Path) -> PathBuf {
    let current = std::fs::read_to_string(root.join("CURRENT")).expect("read CURRENT");
    let generation = current
        .strip_prefix("generation:")
        .expect("CURRENT names a generation")
        .trim();
    assert!(
        !generation.is_empty(),
        "CURRENT must select a non-empty v2 generation name"
    );
    root.join(generation)
}

fn stage1_read_manifest(generation: &Path) -> Value {
    serde_json::from_slice(
        &std::fs::read(generation.join("_generation.json")).expect("read v2 generation manifest"),
    )
    .expect("decode v2 generation manifest")
}

fn stage1_write_manifest(generation: &Path, manifest: &Value) {
    let mut bytes = serde_json::to_vec_pretty(manifest).expect("encode mutated v2 manifest");
    bytes.push(b'\n');
    std::fs::write(generation.join("_generation.json"), bytes).expect("write mutated v2 manifest");
}

fn stage1_first_segment_mut(manifest: &mut Value) -> &mut Value {
    let collections = manifest
        .get_mut("collections")
        .and_then(Value::as_array_mut)
        .expect("v2 catalog collections array");
    let collection = collections.first_mut().expect("one catalog collection");
    collection
        .get_mut("segments")
        .and_then(Value::as_array_mut)
        .expect("catalog segments array")
        .first_mut()
        .expect("one catalogued segment")
}

fn stage1_duplicate_first_segment(manifest: &mut Value) {
    let collections = manifest
        .get_mut("collections")
        .and_then(Value::as_array_mut)
        .expect("v2 catalog collections array");
    let collection = collections.first_mut().expect("one catalog collection");
    let segments = collection
        .get_mut("segments")
        .and_then(Value::as_array_mut)
        .expect("catalog segments array");
    let duplicate = segments.first().expect("one catalogued segment").clone();
    segments.push(duplicate);
}

fn stage1_assert_current_refuses_catalog_input(root: &Path, expected: &str, mutation: &str) {
    let current_before = std::fs::read(root.join("CURRENT")).expect("read CURRENT before refusal");
    let store = SegmentRdbStore::new(root).expect("reopen corrupted v2 checkpoint root");
    let opened = store.load_current_generation();
    let error = opened.as_ref().err().map(ToString::to_string);
    assert!(
        error.is_some(),
        "stage1: CURRENT must refuse {mutation} instead of reopening a predecessor; got a successful load"
    );
    let error = error.unwrap_or_default();
    assert!(
        error.contains(expected),
        "stage1: {mutation} must report {expected:?}; got {error:?}"
    );
    assert_eq!(
        std::fs::read(root.join("CURRENT")).expect("read CURRENT after refusal"),
        current_before,
        "a corrupt current catalog must not rewrite CURRENT or select its predecessor"
    );
}

#[tokio::test]
async fn v2_current_refuses_duplicate_catalogued_segment_without_predecessor_fallback() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_duplicate_first_segment(&mut manifest);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_input(
        &root,
        "duplicate segment reference",
        "a duplicate catalogued segment reference",
    );
}

#[tokio::test]
async fn v2_current_refuses_catalogue_path_escape_without_predecessor_fallback() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_first_segment_mut(&mut manifest)["path"] = json!("../outside.lseg");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_input(
        &root,
        "segment reference path escapes generation",
        "a catalogued path that escapes the CURRENT generation",
    );
}

const STAGE1_WIDE_CATALOG_COLLECTIONS: usize = 182;

fn stage1_wide_catalog_id(index: usize) -> String {
    format!("catalog-{index:03}")
}

/// A complete catalog must scale with the number of live collections, not a
/// small fixed manifest cap. Empty keyword collections make the test metadata
/// only: no document volume or timing claim is needed to expose the format bug.
#[tokio::test]
async fn v2_complete_catalog_of_182_live_collections_cold_opens_every_collection() {
    let dir = tempfile::tempdir().expect("wide catalog fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open wide catalog checkpoint root");
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone())))
        .expect("wide catalog fixture HTTP server");

    for index in 0..STAGE1_WIDE_CATALOG_COLLECTIONS {
        server
            .put(&format!("/collections/{}", stage1_wide_catalog_id(index)))
            .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
    }

    let saved = store.save(&engine, 182);
    let save_error = saved
        .as_ref()
        .err()
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        saved.is_ok(),
        "stage1: a complete v2 catalog with 182 live collections must checkpoint; got {save_error}"
    );

    let generation = stage1_current_generation_dir(&root);
    let manifest = stage1_read_manifest(&generation);
    let catalog_ids: Vec<String> = manifest["collections"]
        .as_array()
        .expect("v2 collections array")
        .iter()
        .map(|collection| {
            collection["collection_id"]
                .as_str()
                .expect("catalog collection id")
                .to_owned()
        })
        .collect();
    let expected_ids: Vec<String> = (0..STAGE1_WIDE_CATALOG_COLLECTIONS)
        .map(stage1_wide_catalog_id)
        .collect();
    assert_eq!(
        catalog_ids, expected_ids,
        "the complete catalog must contain every live collection exactly once and in lexical order"
    );

    let cold_store = SegmentRdbStore::new(&root).expect("reopen wide catalog root");
    let cold = cold_store.load_current_generation();
    let cold_error = cold
        .as_ref()
        .err()
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        cold.is_ok(),
        "stage1: the 182-collection v2 catalog must cold-open; got {cold_error}"
    );
    let cold = cold
        .expect("wide catalog cold load succeeds")
        .expect("wide catalog CURRENT names a generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine)))
        .expect("wide catalog cold HTTP server");
    let listed: Value = cold_server.get("/collections").await.json();
    let listed_ids: Vec<String> = listed
        .as_array()
        .expect("cold collection listing")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("cold listed collection id")
                .to_owned()
        })
        .collect();
    assert_eq!(
        listed_ids, expected_ids,
        "cold open must restore every catalogued collection to the public listing"
    );
}

fn stage1_catalog_collection_mut(manifest: &mut Value) -> &mut Value {
    manifest
        .get_mut("collections")
        .and_then(Value::as_array_mut)
        .expect("v2 catalog collections array")
        .first_mut()
        .expect("one catalogued collection")
}

fn stage1_catalog_segment_with_role_mut<'a>(manifest: &'a mut Value, role: &str) -> &'a mut Value {
    stage1_catalog_collection_mut(manifest)
        .get_mut("segments")
        .and_then(Value::as_array_mut)
        .expect("catalog segments array")
        .iter_mut()
        .find(|segment| segment["role"] == json!(role))
        .unwrap_or_else(|| panic!("catalog needs a {role} segment"))
}

fn stage1_assert_current_refuses_catalog_shape(root: &Path, mutation: &str) {
    let current_before = std::fs::read(root.join("CURRENT")).expect("read CURRENT before refusal");
    let store = SegmentRdbStore::new(root).expect("reopen malformed catalog root");
    let opened = store.load_current_generation();
    let error = opened.as_ref().err().map(ToString::to_string);
    assert!(
        error.is_some(),
        "stage1: CURRENT must refuse {mutation} instead of reopening a predecessor"
    );
    assert!(
        !error.unwrap_or_default().is_empty(),
        "stage1: the refusal for {mutation} must carry a diagnostic"
    );
    assert_eq!(
        std::fs::read(root.join("CURRENT")).expect("read CURRENT after refusal"),
        current_before,
        "a malformed catalog must leave CURRENT on the refused generation"
    );
}

#[tokio::test]
async fn v2_current_refuses_collection_eids_role_that_names_a_field() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_with_role_mut(&mut manifest, "collection_eids")["field"] = json!("kw");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a collection_eids reference that names a field",
    );
}

#[tokio::test]
async fn v2_current_refuses_field_role_without_a_field_name() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_with_role_mut(&mut manifest, "field")["field"] = Value::Null;
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(&root, "a field reference without a field name");
}

#[tokio::test]
async fn v2_current_refuses_catalog_that_omits_a_live_collection() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    manifest["collections"]
        .as_array_mut()
        .expect("v2 catalog collections array")
        .clear();
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(&root, "a catalog that omits a live collection");
}

#[tokio::test]
async fn v2_current_refuses_catalog_schema_that_omits_a_live_field() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_collection_mut(&mut manifest)["schema"] = json!({});
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a catalog schema that omits the persisted keyword field",
    );
}

#[tokio::test]
async fn v2_current_refuses_catalogue_dot_path_without_predecessor_fallback() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_first_segment_mut(&mut manifest)["path"] = json!("./catalogued.lseg");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_input(
        &root,
        "segment reference path escapes generation",
        "a catalogued path containing a dot component",
    );
}

const STAGE1_REUSE_STABLE: &str = "reuse-stable";
const STAGE1_REUSE_CHANGED: &str = "reuse-changed";
const STAGE1_EPOCH_COLLECTION: &str = "reuse-epoch";
const STAGE1_EPOCH_AFTER_RESTART: &str = "reuse-epoch-after-restart";
const STAGE1_PROVENANCE_COLLECTION: &str = "reuse-provenance";

fn stage1_reuse_item(external_id: &str, value: &str) -> Value {
    json!({
        "external_id": external_id,
        "field": "kw",
        "value": value,
    })
}

async fn stage1_reuse_put_keyword_collection(server: &TestServer, collection: &str) {
    server
        .put(&format!("/collections/{collection}"))
        .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
}

async fn stage1_reuse_index(server: &TestServer, collection: &str, external_id: &str, value: &str) {
    server
        .post(&format!("/collections/{collection}/index"))
        .json(&json!({ "items": [stage1_reuse_item(external_id, value)] }))
        .await
        .assert_status_ok();
}

/// Older delta and compaction contracts start from a populated base. The public
/// snapshot restore marks the reconstructed collection as non-fresh, so the
/// next segment save writes that base rather than consuming its initial journal
/// as a sparse layer. The fresh-first contract deliberately does not use this.
fn stage1_restore_legacy_base(engine: &Arc<Engine>, context: &str) {
    let snapshot = engine
        .snapshot()
        .unwrap_or_else(|error| panic!("{context}: snapshot legacy-base fixture: {error:#}"));
    engine
        .restore(snapshot)
        .unwrap_or_else(|error| panic!("{context}: restore legacy-base fixture: {error:#}"));
}

/// A restored legacy-base fixture still must identify the concrete checkpoint
/// that produced each later sparse layer.  The count assertion remains at the
/// call site; this binds those counted rows to the intended applied sequence.
fn stage1_assert_delta_sequence_order(
    references: &[&Value],
    expected_sequences: &[u64],
    context: &str,
) {
    let actual_sequences: Vec<_> = references
        .iter()
        .map(|reference| {
            reference["applied_seq"]
                .as_u64()
                .unwrap_or_else(|| panic!("{context}: delta needs an applied_seq"))
        })
        .collect();
    assert_eq!(
        actual_sequences, expected_sequences,
        "{context}: counted sparse layers must identify exactly the intended applied sequences"
    );
}

async fn stage1_reuse_term_ids(server: &TestServer, collection: &str, value: &str) -> Vec<String> {
    let response = server
        .post(&format!("/collections/{collection}/search"))
        .json(&json!({
            "query": { "term": { "field": "kw", "value": value } },
            "limit": 10,
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let mut ids: Vec<String> = body["hits"]
        .as_array()
        .expect("search hits array")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("hit external id")
                .to_owned()
        })
        .collect();
    ids.sort();
    ids
}

async fn stage1_reuse_assert_term_ids(
    server: &TestServer,
    collection: &str,
    value: &str,
    expected: &[&str],
) {
    let mut expected: Vec<String> = expected.iter().map(|id| (*id).to_owned()).collect();
    expected.sort();
    assert_eq!(
        stage1_reuse_term_ids(server, collection, value).await,
        expected,
        "cold query must retain the expected external IDs for {collection}/{value}"
    );
}

fn stage1_reuse_current_name(root: &Path) -> String {
    std::fs::read_to_string(root.join("CURRENT"))
        .expect("read CURRENT")
        .strip_prefix("generation:")
        .expect("CURRENT names a generation")
        .trim()
        .to_owned()
}

fn stage1_reuse_catalog_collection<'a>(manifest: &'a Value, collection_id: &str) -> &'a Value {
    manifest["collections"]
        .as_array()
        .expect("v2 catalog collections array")
        .iter()
        .find(|collection| collection["collection_id"] == json!(collection_id))
        .unwrap_or_else(|| panic!("catalog must contain {collection_id}"))
}

fn stage1_reuse_collection_u64(collection: &Value, field: &str) -> u64 {
    collection[field]
        .as_u64()
        .unwrap_or_else(|| panic!("catalog {field} must be an unsigned integer"))
}

fn stage1_reuse_cold_load_named(root: &Path, generation_name: &str) -> (Arc<Engine>, u64) {
    let current_path = root.join("CURRENT");
    let active_current = std::fs::read(&current_path).expect("read active CURRENT");
    std::fs::write(&current_path, format!("generation:{generation_name}\n"))
        .expect("point CURRENT at retained generation");
    let opened = SegmentRdbStore::new(root).and_then(|store| store.load_current_generation());
    std::fs::write(&current_path, active_current).expect("restore active CURRENT");
    let loaded = opened
        .expect("retained generation cold load succeeds")
        .expect("retained CURRENT names a generation");
    (loaded.engine, loaded.sequence)
}

fn stage1_reuse_cold_load_current(root: &Path) -> (Arc<Engine>, u64) {
    let loaded = SegmentRdbStore::new(root)
        .expect("reopen current checkpoint root")
        .load_current_generation()
        .expect("current generation cold load succeeds")
        .expect("CURRENT names a generation");
    (loaded.engine, loaded.sequence)
}

#[cfg(unix)]
fn stage1_reuse_ref_key(segment: &Value) -> (String, Option<String>, u64) {
    (
        segment["role"].as_str().expect("segment role").to_owned(),
        segment["field"].as_str().map(str::to_owned),
        segment["ordinal"].as_u64().expect("segment ordinal"),
    )
}

#[cfg(unix)]
fn stage1_reuse_assert_hardlinked_collection(
    base_generation: &Path,
    base_collection: &Value,
    next_generation: &Path,
    next_collection: &Value,
) {
    let base_segments = base_collection["segments"]
        .as_array()
        .expect("base catalog segments");
    let next_segments = next_collection["segments"]
        .as_array()
        .expect("next catalog segments");
    assert!(
        !base_segments.is_empty(),
        "unchanged collection has catalogued segments"
    );
    assert_eq!(
        base_segments.len(),
        next_segments.len(),
        "unchanged collection must retain its complete segment set"
    );
    for base in base_segments {
        assert_eq!(base["kind"], json!("base"));
        assert!(base["local_rows"].is_null());
        let key = stage1_reuse_ref_key(base);
        let next = next_segments
            .iter()
            .find(|segment| stage1_reuse_ref_key(segment) == key)
            .unwrap_or_else(|| panic!("next catalog must retain segment ref {key:?}"));
        assert_eq!(next["kind"], json!("base"));
        assert!(next["local_rows"].is_null());
        let base_path = base_generation.join(base["path"].as_str().expect("base segment path"));
        let next_path = next_generation.join(next["path"].as_str().expect("next segment path"));
        assert_ne!(
            base_path, next_path,
            "immutable generations use distinct paths"
        );
        let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect base segment");
        let next_metadata = std::fs::symlink_metadata(&next_path).expect("inspect reused segment");
        assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
        assert!(next_metadata.is_file() && !next_metadata.file_type().is_symlink());
        assert_eq!(
            base_metadata.ino(),
            next_metadata.ino(),
            "unchanged segment {} must be a hard link, not a copied file",
            next_path.display()
        );
        assert!(
            next_metadata.nlink() >= 2,
            "reused segment {} must have at least two hard links",
            next_path.display()
        );
    }
}

#[cfg(unix)]
fn stage1_reuse_assert_not_hardlinked_collection(
    base_generation: &Path,
    base_collection: &Value,
    next_generation: &Path,
    next_collection: &Value,
) {
    for base in base_collection["segments"]
        .as_array()
        .expect("base catalog segments")
    {
        let key = stage1_reuse_ref_key(base);
        let next = next_collection["segments"]
            .as_array()
            .expect("next catalog segments")
            .iter()
            .find(|segment| stage1_reuse_ref_key(segment) == key)
            .unwrap_or_else(|| panic!("fresh engine must publish ref {key:?}"));
        let base_path = base_generation.join(base["path"].as_str().expect("base segment path"));
        let next_path = next_generation.join(next["path"].as_str().expect("next segment path"));
        assert_ne!(
            std::fs::symlink_metadata(&base_path)
                .expect("inspect base segment")
                .ino(),
            std::fs::symlink_metadata(&next_path)
                .expect("inspect fresh segment")
                .ino(),
            "fresh-engine segment {key:?} with different data must never reuse an old inode"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn v2_hardlinks_unchanged_collection_retains_old_generation_and_cold_opens_new_complete_state(
) {
    let dir = tempfile::tempdir().expect("hard-link reuse fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open hard-link reuse root");
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone())))
        .expect("hard-link reuse HTTP server");
    for collection in [STAGE1_REUSE_STABLE, STAGE1_REUSE_CHANGED] {
        stage1_reuse_put_keyword_collection(&server, collection).await;
    }
    stage1_reuse_index(&server, STAGE1_REUSE_STABLE, "stable-1", "stable-v1").await;
    stage1_reuse_index(&server, STAGE1_REUSE_CHANGED, "changed-1", "changed-v1").await;
    stage1_restore_legacy_base(&engine, "hard-link reuse base");
    store.save(&engine, 501).expect("write base generation");

    let base_name = stage1_reuse_current_name(&root);
    let base_generation = root.join(&base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    let base_stable = stage1_reuse_catalog_collection(&base_manifest, STAGE1_REUSE_STABLE);
    let base_changed = stage1_reuse_catalog_collection(&base_manifest, STAGE1_REUSE_CHANGED);

    stage1_reuse_index(&server, STAGE1_REUSE_CHANGED, "changed-1", "changed-v2").await;
    store.save(&engine, 502).expect("write changed generation");

    let next_name = stage1_reuse_current_name(&root);
    let next_generation = root.join(&next_name);
    let next_manifest = stage1_read_manifest(&next_generation);
    assert_eq!(next_manifest["checkpoint_sequence"], json!(502));
    assert_eq!(next_manifest["previous"], json!(base_name));
    let next_stable = stage1_reuse_catalog_collection(&next_manifest, STAGE1_REUSE_STABLE);
    let next_changed = stage1_reuse_catalog_collection(&next_manifest, STAGE1_REUSE_CHANGED);
    stage1_reuse_assert_hardlinked_collection(
        &base_generation,
        base_stable,
        &next_generation,
        next_stable,
    );
    assert_eq!(
        stage1_reuse_collection_u64(next_stable, "collection_generation"),
        stage1_reuse_collection_u64(base_stable, "collection_generation"),
        "an unchanged collection retains its durable collection generation"
    );
    assert_eq!(
        stage1_reuse_collection_u64(next_stable, "data_version"),
        stage1_reuse_collection_u64(base_stable, "data_version"),
        "an unchanged collection retains its data version"
    );
    assert!(
        stage1_reuse_collection_u64(next_changed, "data_version")
            > stage1_reuse_collection_u64(base_changed, "data_version"),
        "an effective mutation advances only the changed collection data version"
    );

    let (old_engine, old_sequence) = stage1_reuse_cold_load_named(&root, &base_name);
    assert_eq!(
        old_sequence, 501,
        "retained base generation has its original sequence"
    );
    let old_server = TestServer::new(router(AppState::open(old_engine)))
        .expect("retained base cold HTTP server");
    stage1_reuse_assert_term_ids(&old_server, STAGE1_REUSE_STABLE, "stable-v1", &["stable-1"])
        .await;
    stage1_reuse_assert_term_ids(
        &old_server,
        STAGE1_REUSE_CHANGED,
        "changed-v1",
        &["changed-1"],
    )
    .await;

    let (next_engine, next_sequence) = stage1_reuse_cold_load_current(&root);
    assert_eq!(
        next_sequence, 502,
        "new generation has its checkpoint sequence"
    );
    let next_server = TestServer::new(router(AppState::open(next_engine)))
        .expect("new generation cold HTTP server");
    stage1_reuse_assert_term_ids(
        &next_server,
        STAGE1_REUSE_STABLE,
        "stable-v1",
        &["stable-1"],
    )
    .await;
    stage1_reuse_assert_term_ids(&next_server, STAGE1_REUSE_CHANGED, "changed-v1", &[]).await;
    stage1_reuse_assert_term_ids(
        &next_server,
        STAGE1_REUSE_CHANGED,
        "changed-v2",
        &["changed-1"],
    )
    .await;
}

#[tokio::test]
async fn v2_collection_epoch_never_reuses_after_truncate_force_drop_sweep_recreate_and_restart() {
    let dir = tempfile::tempdir().expect("collection epoch fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open collection epoch root");
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone())))
        .expect("collection epoch HTTP server");
    stage1_reuse_put_keyword_collection(&server, STAGE1_EPOCH_COLLECTION).await;
    stage1_reuse_index(&server, STAGE1_EPOCH_COLLECTION, "epoch-first", "first").await;
    store
        .save(&engine, 601)
        .expect("write initial epoch generation");
    let initial_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let initial = stage1_reuse_catalog_collection(&initial_manifest, STAGE1_EPOCH_COLLECTION);
    let first_epoch = stage1_reuse_collection_u64(initial, "collection_generation");
    let schema_version = stage1_reuse_collection_u64(initial, "schema_version");

    server
        .post(&format!(
            "/collections/{STAGE1_EPOCH_COLLECTION}/docs:truncate"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    store
        .save(&engine, 602)
        .expect("write truncated epoch generation");
    let truncated_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let truncated = stage1_reuse_catalog_collection(&truncated_manifest, STAGE1_EPOCH_COLLECTION);
    let truncated_epoch = stage1_reuse_collection_u64(truncated, "collection_generation");
    assert!(
        truncated_epoch > first_epoch,
        "truncate clears into a new durable collection epoch"
    );

    server
        .delete(&format!(
            "/collections/{STAGE1_EPOCH_COLLECTION}?force=true"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    store
        .save(&engine, 603)
        .expect("write force-drop generation");
    assert!(
        stage1_read_manifest(&stage1_current_generation_dir(&root))["collections"]
            .as_array()
            .expect("force-drop catalog collections")
            .is_empty(),
        "force drop removes the collection from the next complete catalog"
    );
    assert_eq!(
        engine
            .sweep_deleted(Duration::from_millis(0))
            .expect("sweep after force drop"),
        0,
        "force drop leaves no tombstone for a later sweep"
    );

    stage1_reuse_put_keyword_collection(&server, STAGE1_EPOCH_COLLECTION).await;
    stage1_reuse_index(&server, STAGE1_EPOCH_COLLECTION, "epoch-second", "second").await;
    store
        .save(&engine, 604)
        .expect("write force-recreated generation");
    let force_recreated_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let second_epoch = stage1_reuse_collection_u64(
        stage1_reuse_catalog_collection(&force_recreated_manifest, STAGE1_EPOCH_COLLECTION),
        "collection_generation",
    );
    assert!(
        second_epoch > truncated_epoch,
        "force-drop recreate must allocate after the truncate epoch"
    );

    server
        .delete(&format!("/collections/{STAGE1_EPOCH_COLLECTION}"))
        .await
        .assert_status(axum::http::StatusCode::ACCEPTED);
    tokio::time::sleep(Duration::from_millis(2)).await;
    assert_eq!(
        engine
            .sweep_deleted(Duration::from_millis(1))
            .expect("sweep soft-deleted collection"),
        1,
        "the second recreate must follow a physical sweep"
    );
    store.save(&engine, 605).expect("write swept generation");
    stage1_reuse_put_keyword_collection(&server, STAGE1_EPOCH_COLLECTION).await;
    stage1_reuse_index(&server, STAGE1_EPOCH_COLLECTION, "epoch-third", "third").await;
    store
        .save(&engine, 606)
        .expect("write swept-recreated generation");
    let final_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let final_collection =
        stage1_reuse_catalog_collection(&final_manifest, STAGE1_EPOCH_COLLECTION);
    let third_epoch = stage1_reuse_collection_u64(final_collection, "collection_generation");
    assert!(
        third_epoch > second_epoch && third_epoch > truncated_epoch,
        "no recreate path may reuse an observed collection epoch"
    );
    assert_eq!(
        stage1_reuse_collection_u64(final_collection, "schema_version"),
        schema_version,
        "same-schema recreate does not fabricate a schema edit"
    );
    let observed_epoch_max = first_epoch
        .max(truncated_epoch)
        .max(second_epoch)
        .max(third_epoch);
    assert!(
        final_manifest["next_collection_generation"]
            .as_u64()
            .expect("durable collection-generation allocator")
            > observed_epoch_max,
        "allocator cursor stays above every issued collection epoch"
    );

    let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&root);
    assert_eq!(
        cold_sequence, 606,
        "restart selects the final recreated generation"
    );
    let cold_server = TestServer::new(router(AppState::open(cold_engine.clone())))
        .expect("epoch restart HTTP server");
    stage1_reuse_assert_term_ids(&cold_server, STAGE1_EPOCH_COLLECTION, "first", &[]).await;
    stage1_reuse_assert_term_ids(&cold_server, STAGE1_EPOCH_COLLECTION, "second", &[]).await;
    stage1_reuse_assert_term_ids(
        &cold_server,
        STAGE1_EPOCH_COLLECTION,
        "third",
        &["epoch-third"],
    )
    .await;

    stage1_reuse_put_keyword_collection(&cold_server, STAGE1_EPOCH_AFTER_RESTART).await;
    stage1_reuse_index(
        &cold_server,
        STAGE1_EPOCH_AFTER_RESTART,
        "epoch-after-restart",
        "after-restart",
    )
    .await;
    let restart_store = SegmentRdbStore::new(&root).expect("reopen allocator after restart");
    restart_store
        .save(&cold_engine, 607)
        .expect("write post-restart allocation generation");
    let post_restart_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let post_restart_epoch = stage1_reuse_collection_u64(
        stage1_reuse_catalog_collection(&post_restart_manifest, STAGE1_EPOCH_AFTER_RESTART),
        "collection_generation",
    );
    assert!(
        post_restart_epoch > observed_epoch_max,
        "the allocator must continue above all epochs after cold restart"
    );
    assert!(
        post_restart_manifest["next_collection_generation"]
            .as_u64()
            .expect("post-restart allocator cursor")
            > post_restart_epoch,
        "post-restart allocation advances the durable allocator cursor"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn v2_save_required_fresh_engine_same_versions_never_reuses_other_data() {
    let dir = tempfile::tempdir().expect("fresh-engine provenance fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open fresh-engine provenance root");

    let original = Arc::new(Engine::new());
    let original_server = TestServer::new(router(AppState::open(original.clone())))
        .expect("original provenance HTTP server");
    stage1_reuse_put_keyword_collection(&original_server, STAGE1_PROVENANCE_COLLECTION).await;
    stage1_reuse_index(
        &original_server,
        STAGE1_PROVENANCE_COLLECTION,
        "provenance-old",
        "old",
    )
    .await;
    store
        .save_required(&original, 701)
        .expect("write original required generation");
    let original_name = stage1_reuse_current_name(&root);
    let original_generation = root.join(&original_name);
    let original_manifest = stage1_read_manifest(&original_generation);

    let fresh = Arc::new(Engine::new());
    let fresh_server = TestServer::new(router(AppState::open(fresh.clone())))
        .expect("fresh provenance HTTP server");
    stage1_reuse_put_keyword_collection(&fresh_server, STAGE1_PROVENANCE_COLLECTION).await;
    stage1_reuse_index(
        &fresh_server,
        STAGE1_PROVENANCE_COLLECTION,
        "provenance-new",
        "new",
    )
    .await;
    store
        .save_required(&fresh, 701)
        .expect("write fresh required generation at the same sequence");
    let fresh_name = stage1_reuse_current_name(&root);
    let fresh_generation = root.join(&fresh_name);
    let fresh_manifest = stage1_read_manifest(&fresh_generation);
    assert_ne!(
        original_name, fresh_name,
        "save_required publishes a new immutable revision"
    );
    let original_collection =
        stage1_reuse_catalog_collection(&original_manifest, STAGE1_PROVENANCE_COLLECTION);
    let fresh_collection =
        stage1_reuse_catalog_collection(&fresh_manifest, STAGE1_PROVENANCE_COLLECTION);
    assert_eq!(
        stage1_reuse_collection_u64(original_collection, "schema_version"),
        stage1_reuse_collection_u64(fresh_collection, "schema_version"),
        "the regression fixture uses identical schema versions"
    );
    assert_eq!(
        stage1_reuse_collection_u64(original_collection, "data_version"),
        stage1_reuse_collection_u64(fresh_collection, "data_version"),
        "the regression fixture uses identical data versions"
    );
    stage1_reuse_assert_not_hardlinked_collection(
        &original_generation,
        original_collection,
        &fresh_generation,
        fresh_collection,
    );

    let (old_engine, old_sequence) = stage1_reuse_cold_load_named(&root, &original_name);
    assert_eq!(
        old_sequence, 701,
        "original required generation keeps its sequence"
    );
    let old_server = TestServer::new(router(AppState::open(old_engine)))
        .expect("original provenance cold HTTP server");
    stage1_reuse_assert_term_ids(
        &old_server,
        STAGE1_PROVENANCE_COLLECTION,
        "old",
        &["provenance-old"],
    )
    .await;

    let (fresh_engine, fresh_sequence) = stage1_reuse_cold_load_current(&root);
    assert_eq!(
        fresh_sequence, 701,
        "fresh required revision keeps its sequence"
    );
    let fresh_server = TestServer::new(router(AppState::open(fresh_engine)))
        .expect("fresh provenance cold HTTP server");
    stage1_reuse_assert_term_ids(&fresh_server, STAGE1_PROVENANCE_COLLECTION, "old", &[]).await;
    stage1_reuse_assert_term_ids(
        &fresh_server,
        STAGE1_PROVENANCE_COLLECTION,
        "new",
        &["provenance-new"],
    )
    .await;
}
// Append after the frozen stage1 oracle in
// apps/lumen/e2e/indexing_durable_oracle.rs (base sha256:
// 940ac59d8f5f18e28e7822e4d2d4f53e10fda37045e952aec385728ed357993e).
//
// These cases use only existing v2 base segments. Each corrupts a manifest
// written by this process, requires `load_current_generation` to reject it,
// and uses `stage1_assert_current_refuses_catalog_shape` to preserve CURRENT.
// They intentionally do not assert implementation error text.

const STAGE1_CATALOG_LEFT: &str = "catalog-integrity-left";
const STAGE1_CATALOG_RIGHT: &str = "catalog-integrity-right";

fn stage1_catalog_collection_mut_by_id<'a>(manifest: &'a mut Value, id: &str) -> &'a mut Value {
    manifest["collections"]
        .as_array_mut()
        .expect("v2 catalog collections array")
        .iter_mut()
        .find(|collection| collection["collection_id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("catalog must contain {id}"))
}

fn stage1_catalog_segment_matches(segment: &Value, role: &str, field: Option<&str>) -> bool {
    segment["role"].as_str() == Some(role)
        && match field {
            Some(field) => segment["field"].as_str() == Some(field),
            None => segment["field"].is_null(),
        }
}

fn stage1_catalog_segment_mut<'a>(
    collection: &'a mut Value,
    role: &str,
    field: Option<&str>,
) -> &'a mut Value {
    collection["segments"]
        .as_array_mut()
        .expect("catalog segments array")
        .iter_mut()
        .find(|segment| stage1_catalog_segment_matches(segment, role, field))
        .unwrap_or_else(|| panic!("catalog must contain {role}/{field:?} segment"))
}

fn stage1_catalog_remove_segment(collection: &mut Value, role: &str, field: Option<&str>) {
    let segments = collection["segments"]
        .as_array_mut()
        .expect("catalog segments array");
    let position = segments
        .iter()
        .position(|segment| stage1_catalog_segment_matches(segment, role, field))
        .unwrap_or_else(|| panic!("catalog must contain {role}/{field:?} segment"));
    segments.remove(position);
}

fn stage1_catalog_max_collection_generation(manifest: &Value) -> u64 {
    manifest["collections"]
        .as_array()
        .expect("v2 catalog collections array")
        .iter()
        .map(|collection| {
            collection["collection_generation"]
                .as_u64()
                .expect("catalog collection generation")
        })
        .max()
        .expect("non-empty catalog")
}

async fn stage1_catalog_root_with_two_keyword_collections() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("two-collection catalog fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open two-collection catalog root");
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone())))
        .expect("two-collection catalog HTTP server");
    for (collection, external_id) in [
        (STAGE1_CATALOG_LEFT, "catalog-left-id"),
        (STAGE1_CATALOG_RIGHT, "catalog-right-id"),
    ] {
        stage1_reuse_put_keyword_collection(&server, collection).await;
        stage1_reuse_index(&server, collection, external_id, collection).await;
    }
    stage1_restore_legacy_base(&engine, "two-collection catalog base");
    store
        .save(&engine, 801)
        .expect("write two-collection v2 base generation");
    (dir, root)
}

#[tokio::test]
async fn v2_current_refuses_field_reference_absent_from_catalog_schema() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut(&mut manifest),
        "field",
        Some("kw"),
    )["field"] = json!("field-not-in-schema");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a field reference whose field is absent from the catalog schema",
    );
}

#[tokio::test]
async fn v2_current_refuses_vector_eids_reference_for_keyword_field() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut(&mut manifest),
        "field",
        Some("kw"),
    )["role"] = json!("vector_eids");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a vector_eids reference for a keyword field",
    );
}

#[tokio::test]
async fn v2_current_refuses_segment_reference_owned_by_another_collection() {
    let (_dir, root) = stage1_catalog_root_with_two_keyword_collections().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let left_path = stage1_catalog_segment_mut(
        stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_LEFT),
        "field",
        Some("kw"),
    )["path"]
        .clone();
    let right_path = stage1_catalog_segment_mut(
        stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_RIGHT),
        "field",
        Some("kw"),
    )["path"]
        .clone();
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_LEFT),
        "field",
        Some("kw"),
    )["path"] = right_path;
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_RIGHT),
        "field",
        Some("kw"),
    )["path"] = left_path;
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a segment reference that points into another catalogued collection",
    );
}

#[tokio::test]
async fn v2_current_refuses_catalog_omitting_required_field_segment() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_remove_segment(
        stage1_catalog_collection_mut(&mut manifest),
        "field",
        Some("kw"),
    );
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a catalog that omits a required keyword field segment",
    );
}

#[tokio::test]
async fn v2_current_refuses_catalog_omitting_required_collection_eids_segment() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_remove_segment(
        stage1_catalog_collection_mut(&mut manifest),
        "collection_eids",
        None,
    );
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a catalog that omits the required collection external-id segment",
    );
}

#[tokio::test]
async fn v2_current_refuses_nonzero_ordinal_for_base_segment() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut(&mut manifest),
        "field",
        Some("kw"),
    )["ordinal"] = json!(1);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(&root, "a base segment with a nonzero ordinal");
}

#[tokio::test]
async fn v2_current_refuses_catalog_schema_version_mismatch() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let collection = stage1_catalog_collection_mut(&mut manifest);
    let schema_version = collection["schema_version"]
        .as_u64()
        .expect("catalog schema version");
    collection["schema_version"] = json!(schema_version + 1);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a catalog schema version that disagrees with its checkpoint schema",
    );
}

#[tokio::test]
async fn v2_current_refuses_zero_collection_generation() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_collection_mut(&mut manifest)["collection_generation"] = json!(0);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(&root, "a zero durable collection generation");
}

#[tokio::test]
async fn v2_current_refuses_duplicate_collection_generation() {
    let (_dir, root) = stage1_catalog_root_with_two_keyword_collections().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let left_generation = stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_LEFT)
        ["collection_generation"]
        .clone();
    stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_RIGHT)
        ["collection_generation"] = left_generation;
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "duplicate durable collection generations in one catalog",
    );
}

#[tokio::test]
async fn v2_current_refuses_allocator_not_above_issued_collection_generations() {
    let (_dir, root) = stage1_catalog_root_with_two_keyword_collections().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let max_generation = stage1_catalog_max_collection_generation(&manifest);
    manifest["next_collection_generation"] = json!(max_generation);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a collection-generation allocator at the highest issued epoch",
    );
}
// Append after apps/lumen/e2e/indexing_durable_oracle.rs at sha256
// 3a1f56ba158a191b2070806b8653e9c6bbfaee5c73bf1082ee5e8f6f686401ed.
//
// Facets when applied:
// - Behavior: public keyword queries, immutable base retention, cold reopen,
//   and no resurrection after a second checkpoint.
// - Security: malformed local-row metadata and duplicate stable IDs must refuse
//   CURRENT without rewriting it; no implementation error text is assumed.
// - Performance: the inode and three-row assertions prove sparse physical work.
//   The approved stage6 release workload, not this case, measures checkpoint time.

const STAGE1_KEYWORD_DELTA_COLLECTION: &str = "keyword-sparse-delta";
const STAGE1_KEYWORD_DELTA_BASE_DOCS: usize = 128;
const STAGE1_KEYWORD_DELTA_TOUCHED_IDS: u64 = 3;
const STAGE1_KEYWORD_DELTA_BASE_SEQUENCE: u64 = 1_101;
const STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE: u64 = 1_102;
const STAGE1_KEYWORD_DELTA_SECOND_SEQUENCE: u64 = 1_103;
const STAGE1_KEYWORD_DELTA_UNTOUCHED_ID: &str = "base-000";
const STAGE1_KEYWORD_DELTA_DELETED_ID: &str = "base-001";
const STAGE1_KEYWORD_DELTA_HIGH_ID: &str = "runtime-id-900000";
const STAGE1_KEYWORD_DELTA_APPENDED_ID: &str = "runtime-id-900001";
const STAGE1_KEYWORD_DELTA_HIGH_OLD: &str = "high-base-old";
const STAGE1_KEYWORD_DELTA_HIGH_NEW: &str = "high-delta-new";
const STAGE1_KEYWORD_DELTA_HIGH_FINAL: &str = "high-delta-final";
const STAGE1_KEYWORD_DELTA_APPENDED_VALUE: &str = "appended-after-base";

// `runtime-id-900000` is only a stable external-ID label. It is base row 128
// in this 129-row fixture, not a dense runtime row numbered 900000.

struct Stage1KeywordDeltaFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: SegmentRdbStore,
    engine: Arc<Engine>,
    server: TestServer,
    base_name: String,
}

fn stage1_keyword_delta_base_id(index: usize) -> String {
    format!("base-{index:03}")
}

fn stage1_keyword_delta_base_value(index: usize) -> String {
    format!("base-value-{index:03}")
}

fn stage1_keyword_delta_base_items() -> Vec<Value> {
    let mut items: Vec<Value> = (0..STAGE1_KEYWORD_DELTA_BASE_DOCS)
        .map(|index| {
            let external_id = stage1_keyword_delta_base_id(index);
            let value = stage1_keyword_delta_base_value(index);
            stage1_reuse_item(&external_id, &value)
        })
        .collect();
    items.push(stage1_reuse_item(
        STAGE1_KEYWORD_DELTA_HIGH_ID,
        STAGE1_KEYWORD_DELTA_HIGH_OLD,
    ));
    items
}

async fn stage1_keyword_delta_index_items(server: &TestServer, items: Vec<Value>) {
    server
        .post(&format!(
            "/collections/{STAGE1_KEYWORD_DELTA_COLLECTION}/index"
        ))
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();
}

async fn stage1_keyword_delta_fixture() -> Stage1KeywordDeltaFixture {
    let dir = tempfile::tempdir().expect("keyword delta fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open keyword delta root");
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine.clone()))).expect("keyword delta HTTP server");
    stage1_reuse_put_keyword_collection(&server, STAGE1_KEYWORD_DELTA_COLLECTION).await;
    stage1_keyword_delta_index_items(&server, stage1_keyword_delta_base_items()).await;
    stage1_restore_legacy_base(&engine, "keyword sparse-delta base");
    store
        .save(&engine, STAGE1_KEYWORD_DELTA_BASE_SEQUENCE)
        .expect("write keyword delta base generation");
    let base_name = stage1_reuse_current_name(&root);
    Stage1KeywordDeltaFixture {
        _dir: dir,
        root,
        store,
        engine,
        server,
        base_name,
    }
}

async fn stage1_keyword_delta_publish_first(fixture: &Stage1KeywordDeltaFixture) {
    stage1_reuse_index(
        &fixture.server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_ID,
        STAGE1_KEYWORD_DELTA_HIGH_NEW,
    )
    .await;
    fixture
        .server
        .delete(&format!(
            "/collections/{STAGE1_KEYWORD_DELTA_COLLECTION}/docs/{STAGE1_KEYWORD_DELTA_DELETED_ID}"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    stage1_reuse_index(
        &fixture.server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_APPENDED_ID,
        STAGE1_KEYWORD_DELTA_APPENDED_VALUE,
    )
    .await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE)
        .expect("write first keyword sparse delta generation");
}

async fn stage1_keyword_delta_publish_second(fixture: &Stage1KeywordDeltaFixture) {
    stage1_reuse_index(
        &fixture.server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_ID,
        STAGE1_KEYWORD_DELTA_HIGH_FINAL,
    )
    .await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_KEYWORD_DELTA_SECOND_SEQUENCE)
        .expect("write second keyword sparse delta generation");
}

fn stage1_keyword_delta_collection(manifest: &Value) -> &Value {
    stage1_reuse_catalog_collection(manifest, STAGE1_KEYWORD_DELTA_COLLECTION)
}

fn stage1_keyword_delta_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
    stage1_keyword_delta_collection(manifest)["segments"]
        .as_array()
        .expect("keyword delta catalog segments")
        .iter()
        .filter(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!("kw")
                && segment["kind"] == json!(kind)
        })
        .collect()
}

fn stage1_keyword_delta_base_ref(manifest: &Value) -> &Value {
    stage1_keyword_delta_refs(manifest, "base")
        .into_iter()
        .find(|segment| segment["ordinal"] == json!(0))
        .expect("keyword catalog must retain its base reference")
}

fn stage1_keyword_delta_single_ref(manifest: &Value, sequence: u64) -> &Value {
    let deltas = stage1_keyword_delta_refs(manifest, "delta");
    assert_eq!(
        deltas.len(),
        1,
        "the first checkpoint after three sparse keyword mutations must publish one keyword delta"
    );
    let matching: Vec<_> = deltas
        .iter()
        .copied()
        .filter(|delta| delta["applied_seq"] == json!(sequence))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "the keyword delta for applied sequence {sequence} must be uniquely catalogued",
    );
    let delta = matching[0];
    assert_eq!(
        delta["ordinal"],
        json!(1),
        "the first keyword delta must immediately follow its ordinal-zero base"
    );
    assert_eq!(delta["format"], json!("lseg-v1"));
    assert!(
        delta["path"].as_str().is_some(),
        "a keyword delta must name its segment through the catalog"
    );
    assert!(
        delta["local_rows"].is_object(),
        "a sparse keyword delta must carry local stable-ID rows"
    );
    delta
}

fn stage1_keyword_delta_local_rows<'a>(delta: &'a Value) -> &'a Map<String, Value> {
    let local_rows = delta["local_rows"].as_object();
    assert!(
        local_rows.is_some(),
        "a sparse keyword delta must include a local-row descriptor"
    );
    local_rows.expect("local-row descriptor after assertion")
}

fn stage1_keyword_delta_rows_path(generation: &Path, delta: &Value) -> PathBuf {
    let local_rows = stage1_keyword_delta_local_rows(delta);
    assert_eq!(
        local_rows["format"],
        json!("lumen-local-eids-cbor-v1"),
        "sparse keyword rows use the versioned local-ID codec"
    );
    let relative = local_rows["path"].as_str();
    assert!(
        relative.is_some(),
        "the local-row descriptor must name its sidecar through the catalog"
    );
    generation.join(relative.expect("local-row sidecar path after assertion"))
}

fn stage1_keyword_delta_rows_count(delta: &Value) -> u64 {
    let count = stage1_keyword_delta_local_rows(delta)["count"].as_u64();
    assert!(
        count.is_some(),
        "the local-row descriptor must carry a count"
    );
    count.expect("local-row count after assertion")
}

fn stage1_keyword_delta_read_rows(generation: &Path, delta: &Value) -> Vec<String> {
    let path = stage1_keyword_delta_rows_path(generation, delta);
    ciborium::from_reader(std::fs::File::open(&path).expect("open sparse local-row sidecar"))
        .expect("decode sparse local-row sidecar")
}

fn stage1_keyword_delta_write_rows(path: &Path, rows: &[String]) {
    let mut bytes = Vec::new();
    ciborium::into_writer(rows, &mut bytes).expect("encode mutated sparse local-row sidecar");
    std::fs::write(path, bytes).expect("write mutated sparse local-row sidecar");
}

fn stage1_keyword_delta_expected_rows() -> std::collections::BTreeSet<String> {
    [
        STAGE1_KEYWORD_DELTA_HIGH_ID,
        STAGE1_KEYWORD_DELTA_DELETED_ID,
        STAGE1_KEYWORD_DELTA_APPENDED_ID,
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect()
}

fn stage1_keyword_delta_assert_sparse_rows(generation: &Path, delta: &Value) {
    let count = stage1_keyword_delta_rows_count(delta);
    assert_eq!(
        count, STAGE1_KEYWORD_DELTA_TOUCHED_IDS,
        "the local-row count must equal the three changed stable external IDs"
    );
    assert!(
        count < STAGE1_KEYWORD_DELTA_BASE_DOCS as u64,
        "a sparse delta must not allocate local rows for the full base cardinality"
    );
    let rows = stage1_keyword_delta_read_rows(generation, delta);
    assert_eq!(
        rows.len() as u64,
        count,
        "sidecar rows match their declared count"
    );
    let unique: std::collections::BTreeSet<String> = rows.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        rows.len(),
        "a sparse local-row sidecar must not repeat a stable external ID"
    );
    assert_eq!(
        unique,
        stage1_keyword_delta_expected_rows(),
        "the local-row sidecar names exactly the three changed stable IDs"
    );
}

#[cfg(unix)]
fn stage1_keyword_delta_assert_base_keyword_hardlinked(
    base_generation: &Path,
    base_manifest: &Value,
    delta_generation: &Path,
    delta_manifest: &Value,
) {
    let base = stage1_keyword_delta_base_ref(base_manifest);
    let retained = stage1_keyword_delta_base_ref(delta_manifest);
    let base_path = base_generation.join(base["path"].as_str().expect("base keyword path"));
    let retained_path =
        delta_generation.join(retained["path"].as_str().expect("retained keyword path"));
    assert_ne!(
        base_path, retained_path,
        "immutable checkpoint generations must use separate path names"
    );
    let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect keyword base");
    let retained_metadata =
        std::fs::symlink_metadata(&retained_path).expect("inspect retained keyword base");
    assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
    assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
    assert_eq!(
        base_metadata.ino(),
        retained_metadata.ino(),
        "the unchanged keyword base must be retained by hard link beside its delta"
    );
    assert!(
        retained_metadata.nlink() >= 2,
        "the retained keyword base must report multiple hard links"
    );
}

async fn stage1_keyword_delta_assert_base_state(server: &TestServer) {
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_OLD,
        &[STAGE1_KEYWORD_DELTA_HIGH_ID],
    )
    .await;
    let deleted_value = stage1_keyword_delta_base_value(1);
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        &deleted_value,
        &[STAGE1_KEYWORD_DELTA_DELETED_ID],
    )
    .await;
    let untouched_value = stage1_keyword_delta_base_value(0);
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        &untouched_value,
        &[STAGE1_KEYWORD_DELTA_UNTOUCHED_ID],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_APPENDED_VALUE,
        &[],
    )
    .await;
}

async fn stage1_keyword_delta_assert_first_live_state(server: &TestServer) {
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_OLD,
        &[],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_NEW,
        &[STAGE1_KEYWORD_DELTA_HIGH_ID],
    )
    .await;
    let deleted_value = stage1_keyword_delta_base_value(1);
    stage1_reuse_assert_term_ids(server, STAGE1_KEYWORD_DELTA_COLLECTION, &deleted_value, &[])
        .await;
    let untouched_value = stage1_keyword_delta_base_value(0);
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        &untouched_value,
        &[STAGE1_KEYWORD_DELTA_UNTOUCHED_ID],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_APPENDED_VALUE,
        &[STAGE1_KEYWORD_DELTA_APPENDED_ID],
    )
    .await;
}

async fn stage1_keyword_delta_assert_second_live_state(server: &TestServer) {
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_OLD,
        &[],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_NEW,
        &[],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_FINAL,
        &[STAGE1_KEYWORD_DELTA_HIGH_ID],
    )
    .await;
    let deleted_value = stage1_keyword_delta_base_value(1);
    stage1_reuse_assert_term_ids(server, STAGE1_KEYWORD_DELTA_COLLECTION, &deleted_value, &[])
        .await;
    let untouched_value = stage1_keyword_delta_base_value(0);
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        &untouched_value,
        &[STAGE1_KEYWORD_DELTA_UNTOUCHED_ID],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_APPENDED_VALUE,
        &[STAGE1_KEYWORD_DELTA_APPENDED_ID],
    )
    .await;
}

fn stage1_keyword_delta_ref_mut(manifest: &mut Value) -> &mut Value {
    stage1_catalog_collection_mut_by_id(manifest, STAGE1_KEYWORD_DELTA_COLLECTION)["segments"]
        .as_array_mut()
        .expect("keyword delta catalog segments")
        .iter_mut()
        .find(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!("kw")
                && segment["kind"] == json!("delta")
        })
        .expect("keyword delta after behavior assertion")
}

#[cfg(unix)]
#[tokio::test]
async fn v2_keyword_sparse_delta_retains_base_and_masks_live_and_cold_queries() {
    let fixture = stage1_keyword_delta_fixture().await;
    let base_generation = fixture.root.join(&fixture.base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    stage1_keyword_delta_publish_first(&fixture).await;
    stage1_keyword_delta_assert_first_live_state(&fixture.server).await;

    let first_name = stage1_reuse_current_name(&fixture.root);
    assert_ne!(
        first_name, fixture.base_name,
        "a sparse keyword change publishes a new immutable generation"
    );
    let first_generation = fixture.root.join(&first_name);
    let first_manifest = stage1_read_manifest(&first_generation);
    let first_delta =
        stage1_keyword_delta_single_ref(&first_manifest, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE);
    stage1_keyword_delta_assert_sparse_rows(&first_generation, first_delta);
    stage1_keyword_delta_assert_base_keyword_hardlinked(
        &base_generation,
        &base_manifest,
        &first_generation,
        &first_manifest,
    );

    let (first_engine, first_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(
        first_sequence, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE,
        "cold reopen selects the first sparse-delta checkpoint"
    );
    let first_server = TestServer::new(router(AppState::open(first_engine)))
        .expect("first sparse-delta cold HTTP server");
    stage1_keyword_delta_assert_first_live_state(&first_server).await;

    stage1_keyword_delta_publish_second(&fixture).await;
    stage1_keyword_delta_assert_second_live_state(&fixture.server).await;
    let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(
        latest_sequence, STAGE1_KEYWORD_DELTA_SECOND_SEQUENCE,
        "cold reopen selects the second sparse-delta checkpoint"
    );
    let latest_server = TestServer::new(router(AppState::open(latest_engine)))
        .expect("second sparse-delta cold HTTP server");
    stage1_keyword_delta_assert_second_live_state(&latest_server).await;

    let (base_engine, base_sequence) =
        stage1_reuse_cold_load_named(&fixture.root, &fixture.base_name);
    assert_eq!(
        base_sequence, STAGE1_KEYWORD_DELTA_BASE_SEQUENCE,
        "the retained base generation keeps its original checkpoint sequence"
    );
    let base_server = TestServer::new(router(AppState::open(base_engine)))
        .expect("retained keyword base cold HTTP server");
    stage1_keyword_delta_assert_base_state(&base_server).await;
}

#[tokio::test]
async fn v2_current_refuses_keyword_delta_local_rows_count_mismatch() {
    let fixture = stage1_keyword_delta_fixture().await;
    stage1_keyword_delta_publish_first(&fixture).await;
    let generation = stage1_current_generation_dir(&fixture.root);
    let mut manifest = stage1_read_manifest(&generation);
    {
        let delta = stage1_keyword_delta_single_ref(&manifest, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE);
        stage1_keyword_delta_assert_sparse_rows(&generation, delta);
    }
    let delta = stage1_keyword_delta_ref_mut(&mut manifest);
    let local_rows = delta["local_rows"]
        .as_object_mut()
        .expect("local rows after behavior assertion");
    let count = local_rows["count"]
        .as_u64()
        .expect("local-row count after behavior assertion");
    local_rows.insert("count".to_owned(), json!(count + 1));
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &fixture.root,
        "a sparse keyword delta whose local-row count does not match its sidecar",
    );
}

#[tokio::test]
async fn v2_current_refuses_keyword_delta_duplicate_stable_local_rows() {
    let fixture = stage1_keyword_delta_fixture().await;
    stage1_keyword_delta_publish_first(&fixture).await;
    let generation = stage1_current_generation_dir(&fixture.root);
    let manifest = stage1_read_manifest(&generation);
    let rows_path = {
        let delta = stage1_keyword_delta_single_ref(&manifest, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE);
        stage1_keyword_delta_assert_sparse_rows(&generation, delta);
        stage1_keyword_delta_rows_path(&generation, delta)
    };
    let mut rows: Vec<String> = ciborium::from_reader(
        std::fs::File::open(&rows_path).expect("open sparse local-row sidecar"),
    )
    .expect("decode sparse local-row sidecar");
    assert_eq!(
        rows.len(),
        STAGE1_KEYWORD_DELTA_TOUCHED_IDS as usize,
        "duplicate-mapping mutation starts from the sparse three-row contract"
    );
    rows[1] = rows[0].clone();
    stage1_keyword_delta_write_rows(&rows_path, &rows);

    stage1_assert_current_refuses_catalog_shape(
        &fixture.root,
        "a sparse keyword delta with duplicate stable external IDs in local rows",
    );
}
// Append after apps/lumen/e2e/indexing_durable_oracle.rs at sha256
// 06752f2533c2c65519ae06c8af4a1915553e8df0388b3e52bf5b89e076a7f3bc.
//
// Facets when applied:
// - Behavior: Number, Set, and Hash each publish sparse ordered delta refs;
//   public live and cold queries match an independently built reference after
//   replacement, deletion, append, and a second checkpoint.
// - Security: a duplicate external ID in a Number delta local-row sidecar must
//   refuse CURRENT without rewriting it. The shared generic refusal helper makes
//   no implementation error-string assumption.
// - Performance: exact three-row and one-row sidecars plus base-file inode
//   equality show sparse work and hard-link reuse. The approved stage6 release
//   workload, rather than this case, measures checkpoint and merge time.
//
// Source premise read only: apps/lumen/src/segment_rdb.rs:1296-1304 currently
// emits base-only catalog refs, and :1373-1380 rejects layers. These cases must
// be red until the scalar layered writer and reader land.

const STAGE1_SCALAR_DELTA_COLLECTION: &str = "scalar-sparse-delta";
const STAGE1_SCALAR_NUMBER_FIELD: &str = "num";
const STAGE1_SCALAR_SET_FIELD: &str = "tags";
const STAGE1_SCALAR_HASH_FIELD: &str = "sig";
const STAGE1_SCALAR_FIELDS: [&str; 3] = [
    STAGE1_SCALAR_NUMBER_FIELD,
    STAGE1_SCALAR_SET_FIELD,
    STAGE1_SCALAR_HASH_FIELD,
];
const STAGE1_SCALAR_ORDINARY_BASE_ROWS: usize = 128;
const STAGE1_SCALAR_BASE_ROWS: usize = STAGE1_SCALAR_ORDINARY_BASE_ROWS + 1;
const STAGE1_SCALAR_FIRST_TOUCHED_IDS: u64 = 3;
const STAGE1_SCALAR_SECOND_TOUCHED_IDS: u64 = 1;
const STAGE1_SCALAR_BASE_SEQUENCE: u64 = 1_201;
const STAGE1_SCALAR_FIRST_SEQUENCE: u64 = 1_202;
const STAGE1_SCALAR_SECOND_SEQUENCE: u64 = 1_203;
const STAGE1_SCALAR_UNTOUCHED_ID: &str = "scalar-base-000";
const STAGE1_SCALAR_DELETED_ID: &str = "scalar-base-001";
const STAGE1_SCALAR_HIGH_ID: &str = "runtime-id-900000";
const STAGE1_SCALAR_APPENDED_ID: &str = "runtime-id-900001";
const STAGE1_SCALAR_HIGH_OLD_NUMBER: f64 = 90_000.0;
const STAGE1_SCALAR_HIGH_NEW_NUMBER: f64 = 90_001.0;
const STAGE1_SCALAR_HIGH_FINAL_NUMBER: f64 = 90_002.0;
const STAGE1_SCALAR_HIGH_OLD_TAG: &str = "scalar-high-old";
const STAGE1_SCALAR_HIGH_FINAL_TAG: &str = "scalar-high-final";
const STAGE1_SCALAR_APPENDED_TAG: &str = "scalar-appended";
const STAGE1_SCALAR_HIGH_OLD_HASH: &str = "000000000000d000";
const STAGE1_SCALAR_HIGH_NEW_HASH: &str = "000000000000d001";
const STAGE1_SCALAR_HIGH_FINAL_HASH: &str = "000000000000d002";
const STAGE1_SCALAR_APPENDED_HASH: &str = "000000000000d003";

// This opaque ID is the 129th base row (runtime row 128). Its numeric-looking
// suffix is only a caller label; the fixture does not model a dense million-row
// runtime-id space.

struct Stage1ScalarDeltaFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: SegmentRdbStore,
    engine: Arc<Engine>,
    server: TestServer,
    base_name: String,
}

fn stage1_scalar_delta_base_id(index: usize) -> String {
    format!("scalar-base-{index:03}")
}

fn stage1_scalar_delta_base_number(index: usize) -> f64 {
    1_000.0 + index as f64
}

fn stage1_scalar_delta_base_tag(index: usize) -> String {
    format!("scalar-base-tag-{index:03}")
}

fn stage1_scalar_delta_base_hash(index: usize) -> String {
    format!("{:016x}", 0x1_000_u64 + index as u64)
}

fn stage1_scalar_delta_base_items() -> Vec<Value> {
    let mut items = Vec::with_capacity(STAGE1_SCALAR_BASE_ROWS * STAGE1_SCALAR_FIELDS.len());
    for index in 0..STAGE1_SCALAR_ORDINARY_BASE_ROWS {
        let external_id = stage1_scalar_delta_base_id(index);
        let tag = stage1_scalar_delta_base_tag(index);
        items.push(json!({
            "external_id": external_id.clone(),
            "field": STAGE1_SCALAR_NUMBER_FIELD,
            "value": stage1_scalar_delta_base_number(index),
        }));
        items.push(json!({
            "external_id": external_id.clone(),
            "field": STAGE1_SCALAR_SET_FIELD,
            "value": [tag],
        }));
        items.push(json!({
            "external_id": external_id,
            "field": STAGE1_SCALAR_HASH_FIELD,
            "value": stage1_scalar_delta_base_hash(index),
        }));
    }
    items.extend([
        json!({
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "field": STAGE1_SCALAR_NUMBER_FIELD,
            "value": STAGE1_SCALAR_HIGH_OLD_NUMBER,
        }),
        json!({
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "field": STAGE1_SCALAR_SET_FIELD,
            "value": [STAGE1_SCALAR_HIGH_OLD_TAG],
        }),
        json!({
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "field": STAGE1_SCALAR_HASH_FIELD,
            "value": STAGE1_SCALAR_HIGH_OLD_HASH,
        }),
    ]);
    items
}

async fn stage1_scalar_delta_create_collection(server: &TestServer) {
    server
        .put(&format!("/collections/{STAGE1_SCALAR_DELTA_COLLECTION}"))
        .json(&json!({ "fields": {
            "num": { "type": "number" },
            "tags": { "type": "set" },
            "sig": { "type": "hash" },
        }}))
        .await
        .assert_status_ok();
}

async fn stage1_scalar_delta_post_items(server: &TestServer, items: Vec<Value>) {
    for chunk in items.chunks(1_000) {
        server
            .post(&format!(
                "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/index"
            ))
            .json(&json!({ "items": chunk }))
            .await
            .assert_status_ok();
    }
}

async fn stage1_scalar_delta_index_doc(
    server: &TestServer,
    external_id: &str,
    number: f64,
    tags: &[&str],
    hash: &str,
) {
    stage1_scalar_delta_post_items(
        server,
        vec![
            json!({
                "external_id": external_id,
                "field": STAGE1_SCALAR_NUMBER_FIELD,
                "value": number,
            }),
            json!({
                "external_id": external_id,
                "field": STAGE1_SCALAR_SET_FIELD,
                "value": tags,
            }),
            json!({
                "external_id": external_id,
                "field": STAGE1_SCALAR_HASH_FIELD,
                "value": hash,
            }),
        ],
    )
    .await;
}

async fn stage1_scalar_delta_replace_without_set(server: &TestServer, number: f64, hash: &str) {
    server
        .put(&format!(
            "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/docs:replace"
        ))
        .json(&json!({ "docs": [{
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "fields": {
                "num": number,
                "sig": hash,
            },
        }]}))
        .await
        .assert_status_ok();
}

async fn stage1_scalar_delta_replace_full(server: &TestServer, number: f64, tag: &str, hash: &str) {
    server
        .put(&format!(
            "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/docs:replace"
        ))
        .json(&json!({ "docs": [{
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "fields": {
                "num": number,
                "tags": [tag],
                "sig": hash,
            },
        }]}))
        .await
        .assert_status_ok();
}

async fn stage1_scalar_delta_delete_doc(server: &TestServer) {
    server
        .delete(&format!(
            "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/index/{STAGE1_SCALAR_DELETED_ID}"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
}

async fn stage1_scalar_delta_fixture() -> Stage1ScalarDeltaFixture {
    let dir = tempfile::tempdir().expect("scalar sparse checkpoint root");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("create scalar segment store");
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine.clone()))).expect("scalar sparse HTTP server");
    stage1_scalar_delta_create_collection(&server).await;
    stage1_scalar_delta_post_items(&server, stage1_scalar_delta_base_items()).await;
    stage1_restore_legacy_base(&engine, "scalar sparse-delta base");
    store
        .save(&engine, STAGE1_SCALAR_BASE_SEQUENCE)
        .expect("write scalar base generation");
    let base_name = stage1_reuse_current_name(&root);
    Stage1ScalarDeltaFixture {
        _dir: dir,
        root,
        store,
        engine,
        server,
        base_name,
    }
}

async fn stage1_scalar_delta_reference() -> TestServer {
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine))).expect("scalar reference server");
    stage1_scalar_delta_create_collection(&server).await;
    stage1_scalar_delta_post_items(&server, stage1_scalar_delta_base_items()).await;
    server
}

async fn stage1_scalar_delta_apply_first(server: &TestServer) {
    // Full replacement deliberately omits `tags`, so the old Set value must be
    // masked by the delta rather than carried forward from its base layer.
    stage1_scalar_delta_replace_without_set(
        server,
        STAGE1_SCALAR_HIGH_NEW_NUMBER,
        STAGE1_SCALAR_HIGH_NEW_HASH,
    )
    .await;
    stage1_scalar_delta_delete_doc(server).await;
    stage1_scalar_delta_index_doc(
        server,
        STAGE1_SCALAR_APPENDED_ID,
        90_003.0,
        &[STAGE1_SCALAR_APPENDED_TAG],
        STAGE1_SCALAR_APPENDED_HASH,
    )
    .await;
}

async fn stage1_scalar_delta_apply_second(server: &TestServer) {
    stage1_scalar_delta_replace_full(
        server,
        STAGE1_SCALAR_HIGH_FINAL_NUMBER,
        STAGE1_SCALAR_HIGH_FINAL_TAG,
        STAGE1_SCALAR_HIGH_FINAL_HASH,
    )
    .await;
}

fn stage1_scalar_delta_number_query(value: f64) -> Value {
    json!({ "range": {
        "field": STAGE1_SCALAR_NUMBER_FIELD,
        "gte": value,
        "lte": value,
    }})
}

fn stage1_scalar_delta_set_query(value: &str) -> Value {
    json!({ "term": {
        "field": STAGE1_SCALAR_SET_FIELD,
        "value": value,
    }})
}

fn stage1_scalar_delta_hash_query(value: &str) -> Value {
    json!({ "hamming": {
        "field": STAGE1_SCALAR_HASH_FIELD,
        "hash": value,
        "max_distance": 0,
    }})
}

async fn stage1_scalar_delta_search_ids(server: &TestServer, query: Value) -> Vec<String> {
    let response = server
        .post(&format!(
            "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/search"
        ))
        .json(&json!({ "query": query, "limit": 32, "track_total": true }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let mut ids: Vec<String> = body["hits"]
        .as_array()
        .expect("scalar search hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("scalar hit ID")
                .to_owned()
        })
        .collect();
    ids.sort();
    assert_eq!(
        body["total"].as_u64(),
        Some(ids.len() as u64),
        "scalar search total must agree with the bounded fixture hits: {body}"
    );
    ids
}

async fn stage1_scalar_delta_assert_query(
    server: &TestServer,
    reference: &TestServer,
    query: Value,
    expected: &[&str],
    context: &str,
) {
    let mut expected: Vec<String> = expected.iter().map(|id| (*id).to_owned()).collect();
    expected.sort();
    let reference_ids = stage1_scalar_delta_search_ids(reference, query.clone()).await;
    assert_eq!(
        reference_ids, expected,
        "independently built reference has the expected {context} state"
    );
    let durable_ids = stage1_scalar_delta_search_ids(server, query).await;
    assert_eq!(
        durable_ids, reference_ids,
        "durable scalar layers must match independent public API state for {context}"
    );
}

async fn stage1_scalar_delta_assert_base_state(server: &TestServer, reference: &TestServer) {
    stage1_scalar_delta_assert_query(
        server,
        reference,
        stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_OLD_NUMBER),
        &[STAGE1_SCALAR_HIGH_ID],
        "base high Number",
    )
    .await;
    stage1_scalar_delta_assert_query(
        server,
        reference,
        stage1_scalar_delta_set_query(STAGE1_SCALAR_HIGH_OLD_TAG),
        &[STAGE1_SCALAR_HIGH_ID],
        "base high Set",
    )
    .await;
    stage1_scalar_delta_assert_query(
        server,
        reference,
        stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_OLD_HASH),
        &[STAGE1_SCALAR_HIGH_ID],
        "base high Hash",
    )
    .await;
}

async fn stage1_scalar_delta_assert_first_state(server: &TestServer, reference: &TestServer) {
    for (query, expected, context) in [
        (
            stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_NEW_NUMBER),
            vec![STAGE1_SCALAR_HIGH_ID],
            "first checkpoint updated Number",
        ),
        (
            stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_OLD_NUMBER),
            vec![],
            "first checkpoint masks old Number",
        ),
        (
            stage1_scalar_delta_number_query(stage1_scalar_delta_base_number(1)),
            vec![],
            "first checkpoint masks deleted Number",
        ),
        (
            stage1_scalar_delta_number_query(stage1_scalar_delta_base_number(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "first checkpoint retains untouched Number",
        ),
        (
            stage1_scalar_delta_number_query(90_003.0),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "first checkpoint appends Number",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_HIGH_OLD_TAG),
            vec![],
            "full replacement omission masks old Set membership",
        ),
        (
            stage1_scalar_delta_set_query(&stage1_scalar_delta_base_tag(1)),
            vec![],
            "first checkpoint masks deleted Set membership",
        ),
        (
            stage1_scalar_delta_set_query(&stage1_scalar_delta_base_tag(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "first checkpoint retains untouched Set membership",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_APPENDED_TAG),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "first checkpoint appends Set membership",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_NEW_HASH),
            vec![STAGE1_SCALAR_HIGH_ID],
            "first checkpoint updated Hash",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_OLD_HASH),
            vec![],
            "first checkpoint masks old Hash",
        ),
        (
            stage1_scalar_delta_hash_query(&stage1_scalar_delta_base_hash(1)),
            vec![],
            "first checkpoint masks deleted Hash",
        ),
        (
            stage1_scalar_delta_hash_query(&stage1_scalar_delta_base_hash(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "first checkpoint retains untouched Hash",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_APPENDED_HASH),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "first checkpoint appends Hash",
        ),
    ] {
        stage1_scalar_delta_assert_query(server, reference, query, &expected, context).await;
    }
}

async fn stage1_scalar_delta_assert_second_state(server: &TestServer, reference: &TestServer) {
    for (query, expected, context) in [
        (
            stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_FINAL_NUMBER),
            vec![STAGE1_SCALAR_HIGH_ID],
            "second checkpoint newest Number",
        ),
        (
            stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_NEW_NUMBER),
            vec![],
            "second checkpoint prevents Number resurrection",
        ),
        (
            stage1_scalar_delta_number_query(stage1_scalar_delta_base_number(1)),
            vec![],
            "second checkpoint keeps deleted Number masked",
        ),
        (
            stage1_scalar_delta_number_query(stage1_scalar_delta_base_number(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "second checkpoint keeps untouched Number",
        ),
        (
            stage1_scalar_delta_number_query(90_003.0),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "second checkpoint keeps appended Number",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_HIGH_OLD_TAG),
            vec![],
            "second checkpoint keeps omitted Set value masked",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_HIGH_FINAL_TAG),
            vec![STAGE1_SCALAR_HIGH_ID],
            "second checkpoint newest Set membership",
        ),
        (
            stage1_scalar_delta_set_query(&stage1_scalar_delta_base_tag(1)),
            vec![],
            "second checkpoint keeps deleted Set membership masked",
        ),
        (
            stage1_scalar_delta_set_query(&stage1_scalar_delta_base_tag(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "second checkpoint keeps untouched Set membership",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_APPENDED_TAG),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "second checkpoint keeps appended Set membership",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_FINAL_HASH),
            vec![STAGE1_SCALAR_HIGH_ID],
            "second checkpoint newest Hash",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_NEW_HASH),
            vec![],
            "second checkpoint prevents Hash resurrection",
        ),
        (
            stage1_scalar_delta_hash_query(&stage1_scalar_delta_base_hash(1)),
            vec![],
            "second checkpoint keeps deleted Hash masked",
        ),
        (
            stage1_scalar_delta_hash_query(&stage1_scalar_delta_base_hash(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "second checkpoint keeps untouched Hash",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_APPENDED_HASH),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "second checkpoint keeps appended Hash",
        ),
    ] {
        stage1_scalar_delta_assert_query(server, reference, query, &expected, context).await;
    }
}

fn stage1_scalar_delta_collection(manifest: &Value) -> &Value {
    stage1_reuse_catalog_collection(manifest, STAGE1_SCALAR_DELTA_COLLECTION)
}

fn stage1_scalar_delta_field_refs<'a>(
    manifest: &'a Value,
    field: &str,
    kind: &str,
) -> Vec<&'a Value> {
    stage1_scalar_delta_collection(manifest)["segments"]
        .as_array()
        .expect("scalar catalog segments")
        .iter()
        .filter(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!(field)
                && segment["kind"] == json!(kind)
        })
        .collect()
}

fn stage1_scalar_delta_field_ref<'a>(
    manifest: &'a Value,
    field: &str,
    kind: &str,
    ordinal: u64,
) -> &'a Value {
    stage1_scalar_delta_field_refs(manifest, field, kind)
        .into_iter()
        .find(|segment| segment["ordinal"] == json!(ordinal))
        .unwrap_or_else(|| panic!("scalar catalog needs {kind} ordinal {ordinal} for {field}"))
}

fn stage1_scalar_delta_for_sequence<'a>(
    manifest: &'a Value,
    field: &str,
    sequence: u64,
) -> &'a Value {
    let matching: Vec<_> = stage1_scalar_delta_field_refs(manifest, field, "delta")
        .into_iter()
        .filter(|segment| segment["applied_seq"] == json!(sequence))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "scalar catalog must contain exactly one {field} delta for applied sequence {sequence}",
    );
    matching[0]
}

fn stage1_scalar_delta_first_touched_ids() -> std::collections::BTreeSet<String> {
    [
        STAGE1_SCALAR_HIGH_ID,
        STAGE1_SCALAR_DELETED_ID,
        STAGE1_SCALAR_APPENDED_ID,
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect()
}

fn stage1_scalar_delta_second_touched_ids() -> std::collections::BTreeSet<String> {
    [STAGE1_SCALAR_HIGH_ID]
        .iter()
        .map(|id| (*id).to_owned())
        .collect()
}

fn stage1_scalar_delta_assert_sparse_rows(
    generation: &Path,
    field: &str,
    delta: &Value,
    expected_count: u64,
    expected_ids: std::collections::BTreeSet<String>,
) {
    assert_eq!(delta["role"], json!("field"));
    assert_eq!(delta["field"], json!(field));
    assert_eq!(delta["kind"], json!("delta"));
    assert_eq!(delta["format"], json!("lseg-v1"));
    assert!(
        delta["path"].as_str().is_some(),
        "{field} delta must name its segment through the catalog"
    );
    let count = stage1_keyword_delta_rows_count(delta);
    assert_eq!(
        count, expected_count,
        "{field} delta local rows must cover exactly its changed stable IDs"
    );
    assert!(
        count < STAGE1_SCALAR_BASE_ROWS as u64,
        "{field} delta must not allocate local rows for the full base cardinality"
    );
    let rows = stage1_keyword_delta_read_rows(generation, delta);
    assert_eq!(
        rows.len() as u64,
        count,
        "{field} local rows match their count"
    );
    let unique: std::collections::BTreeSet<String> = rows.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        rows.len(),
        "{field} local rows must not repeat stable external IDs"
    );
    assert_eq!(
        unique, expected_ids,
        "{field} local rows name exactly the changed stable external IDs"
    );
}

fn stage1_scalar_delta_assert_first_catalog(generation: &Path, manifest: &Value) {
    for field in STAGE1_SCALAR_FIELDS {
        let deltas = stage1_scalar_delta_field_refs(manifest, field, "delta");
        assert_eq!(
            deltas.len(),
            1,
            "the first scalar checkpoint must publish one {field} delta"
        );
        let delta = stage1_scalar_delta_for_sequence(manifest, field, STAGE1_SCALAR_FIRST_SEQUENCE);
        assert_eq!(
            delta["ordinal"],
            json!(1),
            "first scalar {field} delta immediately follows the restored base",
        );
        stage1_scalar_delta_assert_sparse_rows(
            generation,
            field,
            delta,
            STAGE1_SCALAR_FIRST_TOUCHED_IDS,
            stage1_scalar_delta_first_touched_ids(),
        );
    }
}

fn stage1_scalar_delta_assert_second_catalog(generation: &Path, manifest: &Value) {
    for field in STAGE1_SCALAR_FIELDS {
        let deltas = stage1_scalar_delta_field_refs(manifest, field, "delta");
        assert_eq!(
            deltas.len(),
            2,
            "the second scalar checkpoint must retain ordinal-one and add ordinal-two {field} deltas"
        );
        let first = stage1_scalar_delta_for_sequence(manifest, field, STAGE1_SCALAR_FIRST_SEQUENCE);
        assert_eq!(
            first["ordinal"],
            json!(1),
            "first scalar {field} delta remains ordinal one",
        );
        stage1_scalar_delta_assert_sparse_rows(
            generation,
            field,
            first,
            STAGE1_SCALAR_FIRST_TOUCHED_IDS,
            stage1_scalar_delta_first_touched_ids(),
        );
        let second =
            stage1_scalar_delta_for_sequence(manifest, field, STAGE1_SCALAR_SECOND_SEQUENCE);
        assert_eq!(
            second["ordinal"],
            json!(2),
            "second scalar {field} delta is ordinal two",
        );
        stage1_scalar_delta_assert_sparse_rows(
            generation,
            field,
            second,
            STAGE1_SCALAR_SECOND_TOUCHED_IDS,
            stage1_scalar_delta_second_touched_ids(),
        );
    }
}

#[cfg(unix)]
fn stage1_scalar_delta_assert_base_hardlinked(
    base_generation: &Path,
    base_manifest: &Value,
    later_generation: &Path,
    later_manifest: &Value,
) {
    for field in STAGE1_SCALAR_FIELDS {
        let base = stage1_scalar_delta_field_ref(base_manifest, field, "base", 0);
        let retained = stage1_scalar_delta_field_ref(later_manifest, field, "base", 0);
        let base_path = base_generation.join(base["path"].as_str().expect("scalar base path"));
        let retained_path = later_generation.join(
            retained["path"]
                .as_str()
                .expect("retained scalar base path"),
        );
        assert_ne!(
            base_path, retained_path,
            "immutable scalar generations use distinct {field} paths"
        );
        let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect scalar base");
        let retained_metadata =
            std::fs::symlink_metadata(&retained_path).expect("inspect retained scalar base");
        assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
        assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
        assert_eq!(
            base_metadata.ino(),
            retained_metadata.ino(),
            "unchanged {field} base must be retained by hard link, not a copy"
        );
        assert!(
            retained_metadata.nlink() >= 2,
            "retained {field} base must report multiple hard links"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn v2_number_set_hash_sparse_deltas_match_independent_reference_across_two_checkpoints() {
    let fixture = stage1_scalar_delta_fixture().await;
    let reference = stage1_scalar_delta_reference().await;
    let base_generation = fixture.root.join(&fixture.base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    stage1_scalar_delta_assert_base_state(&fixture.server, &reference).await;

    stage1_scalar_delta_apply_first(&fixture.server).await;
    stage1_scalar_delta_apply_first(&reference).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_SCALAR_FIRST_SEQUENCE)
        .expect("write first scalar sparse checkpoint");
    stage1_scalar_delta_assert_first_state(&fixture.server, &reference).await;

    let first_name = stage1_reuse_current_name(&fixture.root);
    assert_ne!(
        first_name, fixture.base_name,
        "scalar mutations publish a new immutable checkpoint generation"
    );
    let first_generation = fixture.root.join(&first_name);
    let first_manifest = stage1_read_manifest(&first_generation);
    stage1_scalar_delta_assert_first_catalog(&first_generation, &first_manifest);
    stage1_scalar_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &first_generation,
        &first_manifest,
    );

    let (first_engine, first_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(
        first_sequence, STAGE1_SCALAR_FIRST_SEQUENCE,
        "cold reopen selects the first scalar checkpoint"
    );
    let first_server = TestServer::new(router(AppState::open(first_engine)))
        .expect("first scalar cold HTTP server");
    stage1_scalar_delta_assert_first_state(&first_server, &reference).await;

    stage1_scalar_delta_apply_second(&fixture.server).await;
    stage1_scalar_delta_apply_second(&reference).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_SCALAR_SECOND_SEQUENCE)
        .expect("write second scalar sparse checkpoint");
    stage1_scalar_delta_assert_second_state(&fixture.server, &reference).await;

    let latest_generation = stage1_current_generation_dir(&fixture.root);
    let latest_manifest = stage1_read_manifest(&latest_generation);
    stage1_scalar_delta_assert_second_catalog(&latest_generation, &latest_manifest);
    stage1_scalar_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &latest_generation,
        &latest_manifest,
    );

    let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(
        latest_sequence, STAGE1_SCALAR_SECOND_SEQUENCE,
        "cold reopen selects the second scalar checkpoint"
    );
    let latest_server = TestServer::new(router(AppState::open(latest_engine)))
        .expect("second scalar cold HTTP server");
    stage1_scalar_delta_assert_second_state(&latest_server, &reference).await;
}

#[tokio::test]
async fn v2_current_refuses_number_sparse_delta_duplicate_stable_local_rows() {
    let fixture = stage1_scalar_delta_fixture().await;
    stage1_scalar_delta_apply_first(&fixture.server).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_SCALAR_FIRST_SEQUENCE)
        .expect("write scalar delta before local-row corruption");
    let generation = stage1_current_generation_dir(&fixture.root);
    let manifest = stage1_read_manifest(&generation);
    let delta = stage1_scalar_delta_for_sequence(
        &manifest,
        STAGE1_SCALAR_NUMBER_FIELD,
        STAGE1_SCALAR_FIRST_SEQUENCE,
    );
    stage1_scalar_delta_assert_sparse_rows(
        &generation,
        STAGE1_SCALAR_NUMBER_FIELD,
        delta,
        STAGE1_SCALAR_FIRST_TOUCHED_IDS,
        stage1_scalar_delta_first_touched_ids(),
    );
    let rows_path = stage1_keyword_delta_rows_path(&generation, delta);
    let mut rows = stage1_keyword_delta_read_rows(&generation, delta);
    assert_eq!(
        rows.len(),
        STAGE1_SCALAR_FIRST_TOUCHED_IDS as usize,
        "duplicate local-row mutation starts from the three-ID Number contract"
    );
    rows[1] = rows[0].clone();
    stage1_keyword_delta_write_rows(&rows_path, &rows);

    stage1_assert_current_refuses_catalog_shape(
        &fixture.root,
        "a Number delta with duplicate stable external IDs in its local rows",
    );
}

// Scratch-only append for apps/lumen/e2e/indexing_durable_oracle.rs.
//
// Facets when applied:
// - Behavior: public one-token and two-token Match requests compare ordered
//   IDs and serialized BM25 scores with an independent, never-checkpointed
//   Engine across two checkpoints and cold opens. Public stats check document
//   counts and avg_doc_len.
// - Security: v2_current_refuses_number_sparse_delta_duplicate_stable_local_rows
//   already corrupts the shared lumen-local-eids-cbor-v1 LocalRows format and
//   requires a refusal without a CURRENT rewrite. Text uses that same shared
//   catalog descriptor and decoder, so another field-only duplicate test would
//   repeat that trust-boundary coverage.
// - Performance: exact three-ID local-row sidecars and inode equality prove
//   sparse layers and hard-link reuse. They do not measure latency or RSS; the
//   approved stage6 release workload owns those budgets.
//
// Text BM25 is exposed through Match. A Term query on a Text field is invalid,
// so the one-token Match is the public term-level scoring case.

const STAGE1_TEXT_DELTA_COLLECTION: &str = "text-sparse-delta";
const STAGE1_TEXT_FIELD: &str = "body";
const STAGE1_TEXT_ORDINARY_BASE_ROWS: usize = 128;
const STAGE1_TEXT_BASE_ROWS: usize = STAGE1_TEXT_ORDINARY_BASE_ROWS + 1;
const STAGE1_TEXT_BASE_SEQUENCE: u64 = 1_301;
const STAGE1_TEXT_FIRST_SEQUENCE: u64 = 1_302;
const STAGE1_TEXT_SECOND_SEQUENCE: u64 = 1_303;
const STAGE1_TEXT_UNTOUCHED_ID: &str = "text-base-000";
const STAGE1_TEXT_FIRST_DELETED_ID: &str = "text-base-001";
const STAGE1_TEXT_SECOND_DELETED_ID: &str = "text-base-002";
const STAGE1_TEXT_HIGH_ID: &str = "runtime-id-900000";
const STAGE1_TEXT_APPENDED_ID: &str = "runtime-id-900001";
const STAGE1_TEXT_COMMON: &str = "text-common";
const STAGE1_TEXT_SHARED: &str = "text-shared";
const STAGE1_TEXT_OLD: &str = "text-legacy";
const STAGE1_TEXT_UPDATED: &str = "text-updated";
const STAGE1_TEXT_APPENDED: &str = "text-appended";
const STAGE1_TEXT_FINAL: &str = "text-final";

struct Stage1TextDeltaFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: SegmentRdbStore,
    engine: Arc<Engine>,
    server: TestServer,
    base_name: String,
}

fn stage1_text_delta_base_id(index: usize) -> String {
    format!("text-base-{index:03}")
}

fn stage1_text_delta_base_body(index: usize) -> String {
    format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED} text-base-{index:03}")
}

fn stage1_text_delta_base_items() -> Vec<Value> {
    let mut items: Vec<Value> = (0..STAGE1_TEXT_ORDINARY_BASE_ROWS)
        .map(|index| {
            json!({
                "external_id": stage1_text_delta_base_id(index),
                "field": STAGE1_TEXT_FIELD,
                "value": stage1_text_delta_base_body(index),
            })
        })
        .collect();
    items.push(json!({
        "external_id": STAGE1_TEXT_HIGH_ID,
        "field": STAGE1_TEXT_FIELD,
        "value": format!("{STAGE1_TEXT_OLD} {STAGE1_TEXT_OLD} {STAGE1_TEXT_SHARED}"),
    }));
    items
}

async fn stage1_text_delta_create_collection(server: &TestServer) {
    server
        .put(&format!("/collections/{STAGE1_TEXT_DELTA_COLLECTION}"))
        .json(&json!({ "fields": {
            "body": { "type": "text", "analyzer": "whitespace_lower" },
        }}))
        .await
        .assert_status_ok();
}

async fn stage1_text_delta_post_items(server: &TestServer, items: Vec<Value>) {
    for chunk in items.chunks(1_000) {
        server
            .post(&format!(
                "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/index"
            ))
            .json(&json!({ "items": chunk }))
            .await
            .assert_status_ok();
    }
}

async fn stage1_text_delta_index_body(server: &TestServer, external_id: &str, body: String) {
    stage1_text_delta_post_items(
        server,
        vec![json!({
            "external_id": external_id,
            "field": STAGE1_TEXT_FIELD,
            "value": body,
        })],
    )
    .await;
}

async fn stage1_text_delta_delete(server: &TestServer, external_id: &str) {
    server
        .delete(&format!(
            "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/docs/{external_id}"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
}

async fn stage1_text_delta_replace_without_body(server: &TestServer) {
    let response = server
        .put(&format!(
            "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/docs:replace"
        ))
        .json(&json!({ "docs": [{
            "external_id": STAGE1_TEXT_HIGH_ID,
            "fields": {},
        }]}))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(
        body["results"][0]["status"], "ok",
        "full replacement omitting Text must succeed: {body}"
    );
}

async fn stage1_text_delta_fixture() -> Stage1TextDeltaFixture {
    let dir = tempfile::tempdir().expect("text sparse checkpoint root");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("create text segment store");
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine.clone()))).expect("text sparse HTTP server");
    stage1_text_delta_create_collection(&server).await;
    stage1_text_delta_post_items(&server, stage1_text_delta_base_items()).await;
    stage1_restore_legacy_base(&engine, "Text sparse-delta base");
    store
        .save(&engine, STAGE1_TEXT_BASE_SEQUENCE)
        .expect("write Text base generation");
    let base_name = stage1_reuse_current_name(&root);
    Stage1TextDeltaFixture {
        _dir: dir,
        root,
        store,
        engine,
        server,
        base_name,
    }
}

async fn stage1_text_delta_reference() -> TestServer {
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine))).expect("Text reference HTTP server");
    stage1_text_delta_create_collection(&server).await;
    stage1_text_delta_post_items(&server, stage1_text_delta_base_items()).await;
    server
}

async fn stage1_text_delta_apply_first(server: &TestServer) {
    stage1_text_delta_index_body(
        server,
        STAGE1_TEXT_HIGH_ID,
        format!(
            "{STAGE1_TEXT_UPDATED} {STAGE1_TEXT_UPDATED} {STAGE1_TEXT_UPDATED} {STAGE1_TEXT_SHARED}"
        ),
    )
    .await;
    stage1_text_delta_delete(server, STAGE1_TEXT_FIRST_DELETED_ID).await;
    stage1_text_delta_index_body(
        server,
        STAGE1_TEXT_APPENDED_ID,
        format!("{STAGE1_TEXT_APPENDED} {STAGE1_TEXT_APPENDED} {STAGE1_TEXT_SHARED}"),
    )
    .await;
}

async fn stage1_text_delta_apply_second_after_cold_open(server: &TestServer) {
    stage1_text_delta_index_body(
        server,
        STAGE1_TEXT_APPENDED_ID,
        format!("{STAGE1_TEXT_FINAL} {STAGE1_TEXT_FINAL} {STAGE1_TEXT_SHARED}"),
    )
    .await;
    stage1_text_delta_replace_without_body(server).await;
    stage1_text_delta_delete(server, STAGE1_TEXT_SECOND_DELETED_ID).await;
}

fn stage1_text_delta_match_query(text: &str) -> Value {
    json!({ "match": {
        "field": STAGE1_TEXT_FIELD,
        "text": text,
        "op": "and",
    }})
}

async fn stage1_text_delta_search(server: &TestServer, query: Value) -> Value {
    let response = server
        .post(&format!(
            "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/search"
        ))
        .json(&json!({ "query": query, "limit": 512, "track_total": true }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let hits = body["hits"].as_array().expect("Text search hits array");
    assert_eq!(
        body["total"].as_u64(),
        Some(hits.len() as u64),
        "bounded Text fixture returns every hit: {body}"
    );
    assert!(
        hits.iter().all(|hit| {
            hit["external_id"].as_str().is_some() && hit["score"].as_f64().is_some()
        }),
        "Text Match exposes IDs and serialized BM25 scores: {body}"
    );
    body
}

fn stage1_text_delta_hit_ids(body: &Value) -> Vec<String> {
    body["hits"]
        .as_array()
        .expect("Text search hits array")
        .iter()
        .map(|hit| hit["external_id"].as_str().expect("Text hit ID").to_owned())
        .collect()
}

async fn stage1_text_delta_assert_match(
    server: &TestServer,
    reference: &TestServer,
    text: &str,
    expected_ids: Vec<String>,
    context: &str,
) {
    let query = stage1_text_delta_match_query(text);
    let reference_body = stage1_text_delta_search(reference, query.clone()).await;
    let mut reference_ids = stage1_text_delta_hit_ids(&reference_body);
    reference_ids.sort();
    let mut expected_ids = expected_ids;
    expected_ids.sort();
    assert_eq!(
        reference_ids, expected_ids,
        "independent Text reference has expected {context} IDs"
    );

    let durable_body = stage1_text_delta_search(server, query).await;
    assert_eq!(
        durable_body["total"], reference_body["total"],
        "durable Text layers preserve {context} total"
    );
    assert_eq!(
        durable_body["hits"], reference_body["hits"],
        "durable Text layers preserve ordered {context} IDs and serialized BM25 scores"
    );
}

async fn stage1_text_delta_stats(server: &TestServer) -> Value {
    let response = server
        .get(&format!(
            "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/stats"
        ))
        .await;
    response.assert_status_ok();
    response.json()
}

async fn stage1_text_delta_assert_stats(
    server: &TestServer,
    reference: &TestServer,
    expected_documents: u64,
    expected_avg_doc_len: f64,
    context: &str,
) {
    let reference_stats = stage1_text_delta_stats(reference).await;
    assert_eq!(
        reference_stats["documents_indexed"].as_u64(),
        Some(expected_documents),
        "independent reference has expected {context} document count"
    );
    let reference_avg = reference_stats["fields"][STAGE1_TEXT_FIELD]["avg_doc_len"]
        .as_f64()
        .expect("Text stats expose avg_doc_len");
    // avg_doc_len is a public f32, so only this representation check has a
    // small f32 serialization tolerance. BM25 hit scores above remain exact.
    assert!(
        (reference_avg - expected_avg_doc_len).abs() < 1e-6,
        "independent reference {context} avg_doc_len must be {expected_avg_doc_len}, got {reference_avg}"
    );

    let durable_stats = stage1_text_delta_stats(server).await;
    assert_eq!(
        durable_stats["documents_indexed"], reference_stats["documents_indexed"],
        "durable Text layers preserve {context} document count"
    );
    assert_eq!(
        durable_stats["fields"][STAGE1_TEXT_FIELD]["avg_doc_len"],
        reference_stats["fields"][STAGE1_TEXT_FIELD]["avg_doc_len"],
        "durable Text layers preserve {context} serialized avg_doc_len"
    );
}

fn stage1_text_delta_common_ids(excluded: &[&str]) -> Vec<String> {
    (0..STAGE1_TEXT_ORDINARY_BASE_ROWS)
        .map(stage1_text_delta_base_id)
        .filter(|id| !excluded.iter().any(|excluded| id == excluded))
        .collect()
}

async fn stage1_text_delta_assert_state(
    server: &TestServer,
    reference: &TestServer,
    queries: &[(&str, Vec<String>, &str)],
    expected_documents: u64,
    expected_avg_doc_len: f64,
    context: &str,
) {
    for (text, expected_ids, query_context) in queries {
        stage1_text_delta_assert_match(
            server,
            reference,
            text,
            expected_ids.clone(),
            query_context,
        )
        .await;
    }
    stage1_text_delta_assert_stats(
        server,
        reference,
        expected_documents,
        expected_avg_doc_len,
        context,
    )
    .await;
}

fn stage1_text_delta_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
    stage1_reuse_catalog_collection(manifest, STAGE1_TEXT_DELTA_COLLECTION)["segments"]
        .as_array()
        .expect("Text catalog segments")
        .iter()
        .filter(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!(STAGE1_TEXT_FIELD)
                && segment["kind"] == json!(kind)
        })
        .collect()
}

fn stage1_text_delta_ref<'a>(manifest: &'a Value, kind: &str, ordinal: u64) -> &'a Value {
    stage1_text_delta_refs(manifest, kind)
        .into_iter()
        .find(|segment| segment["ordinal"] == json!(ordinal))
        .unwrap_or_else(|| panic!("Text catalog needs {kind} ordinal {ordinal}"))
}

fn stage1_text_delta_for_sequence(manifest: &Value, sequence: u64) -> &Value {
    let matching: Vec<_> = stage1_text_delta_refs(manifest, "delta")
        .into_iter()
        .filter(|segment| segment["applied_seq"] == json!(sequence))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "Text catalog must contain exactly one delta for applied sequence {sequence}",
    );
    matching[0]
}

fn stage1_text_delta_assert_delta(
    generation: &Path,
    delta: &Value,
    expected_ids: std::collections::BTreeSet<String>,
) {
    assert_eq!(delta["role"], json!("field"));
    assert_eq!(delta["field"], json!(STAGE1_TEXT_FIELD));
    assert_eq!(delta["kind"], json!("delta"));
    assert_eq!(delta["format"], json!("lseg-v1"));
    assert!(
        delta["path"].as_str().is_some(),
        "Text delta names a segment"
    );
    let count = stage1_keyword_delta_rows_count(delta);
    assert_eq!(
        count,
        expected_ids.len() as u64,
        "Text local rows cover exactly changed stable IDs"
    );
    assert!(
        count < STAGE1_TEXT_BASE_ROWS as u64,
        "Text local rows must stay sparse"
    );
    let rows = stage1_keyword_delta_read_rows(generation, delta);
    assert_eq!(
        rows.len() as u64,
        count,
        "Text local row count matches bytes"
    );
    let unique: std::collections::BTreeSet<String> = rows.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        rows.len(),
        "Text local rows have no duplicate IDs"
    );
    assert_eq!(
        unique, expected_ids,
        "Text local rows name changed IDs exactly"
    );
}

#[cfg(unix)]
fn stage1_text_delta_assert_base_hardlinked(
    base_generation: &Path,
    base_manifest: &Value,
    later_generation: &Path,
    later_manifest: &Value,
) {
    let base = stage1_text_delta_ref(base_manifest, "base", 0);
    let retained = stage1_text_delta_ref(later_manifest, "base", 0);
    let base_path = base_generation.join(base["path"].as_str().expect("Text base path"));
    let retained_path =
        later_generation.join(retained["path"].as_str().expect("retained Text base path"));
    assert_ne!(
        base_path, retained_path,
        "immutable Text generations use new paths"
    );
    let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect Text base");
    let retained_metadata =
        std::fs::symlink_metadata(&retained_path).expect("inspect retained Text base");
    assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
    assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
    assert_eq!(
        base_metadata.ino(),
        retained_metadata.ino(),
        "unchanged Text base is retained by hard link, not a copy"
    );
    assert!(
        retained_metadata.nlink() >= 2,
        "retained Text base has multiple links"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn v2_text_sparse_delta_preserves_bm25_docs_and_cold_mutations() {
    let fixture = stage1_text_delta_fixture().await;
    let reference = stage1_text_delta_reference().await;
    let base_generation = fixture.root.join(&fixture.base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    stage1_text_delta_assert_state(
        &fixture.server,
        &reference,
        &[
            (
                STAGE1_TEXT_OLD,
                vec![STAGE1_TEXT_HIGH_ID.to_owned()],
                "base one-token Match",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                stage1_text_delta_common_ids(&[]),
                "base two-token BM25 Match",
            ),
        ],
        129,
        3.0,
        "base Text corpus",
    )
    .await;

    stage1_text_delta_apply_first(&fixture.server).await;
    stage1_text_delta_apply_first(&reference).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_TEXT_FIRST_SEQUENCE)
        .expect("write first Text sparse checkpoint");
    let first_common = stage1_text_delta_common_ids(&[STAGE1_TEXT_FIRST_DELETED_ID]);
    stage1_text_delta_assert_state(
        &fixture.server,
        &reference,
        &[
            (STAGE1_TEXT_OLD, vec![], "first checkpoint masks old Text"),
            (
                STAGE1_TEXT_UPDATED,
                vec![STAGE1_TEXT_HIGH_ID.to_owned()],
                "first checkpoint updates Text",
            ),
            (
                STAGE1_TEXT_FIRST_DELETED_ID,
                vec![],
                "first checkpoint masks deleted Text",
            ),
            (
                STAGE1_TEXT_APPENDED,
                vec![STAGE1_TEXT_APPENDED_ID.to_owned()],
                "first checkpoint appends Text",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                first_common,
                "first checkpoint two-token BM25 Match",
            ),
        ],
        129,
        388.0 / 129.0,
        "first checkpoint Text corpus",
    )
    .await;

    let first_name = stage1_reuse_current_name(&fixture.root);
    assert_ne!(
        first_name, fixture.base_name,
        "Text mutation creates a generation"
    );
    let first_generation = fixture.root.join(&first_name);
    let first_manifest = stage1_read_manifest(&first_generation);
    let first_delta = stage1_text_delta_for_sequence(&first_manifest, STAGE1_TEXT_FIRST_SEQUENCE);
    assert_eq!(
        stage1_text_delta_refs(&first_manifest, "delta").len(),
        1,
        "first Text checkpoint publishes one delta"
    );
    assert_eq!(
        first_delta["ordinal"],
        json!(1),
        "first Text delta immediately follows the restored base",
    );
    stage1_text_delta_assert_delta(
        &first_generation,
        first_delta,
        [
            STAGE1_TEXT_HIGH_ID,
            STAGE1_TEXT_FIRST_DELETED_ID,
            STAGE1_TEXT_APPENDED_ID,
        ]
        .iter()
        .map(|id| (*id).to_owned())
        .collect(),
    );
    stage1_text_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &first_generation,
        &first_manifest,
    );

    let (first_engine, first_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(first_sequence, STAGE1_TEXT_FIRST_SEQUENCE);
    let first_server = TestServer::new(router(AppState::open(first_engine.clone())))
        .expect("first Text cold HTTP server");
    let first_common = stage1_text_delta_common_ids(&[STAGE1_TEXT_FIRST_DELETED_ID]);
    stage1_text_delta_assert_state(
        &first_server,
        &reference,
        &[
            (
                STAGE1_TEXT_UPDATED,
                vec![STAGE1_TEXT_HIGH_ID.to_owned()],
                "cold first checkpoint update",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                first_common,
                "cold first checkpoint BM25",
            ),
        ],
        129,
        388.0 / 129.0,
        "cold first checkpoint Text corpus",
    )
    .await;

    stage1_text_delta_apply_second_after_cold_open(&first_server).await;
    stage1_text_delta_apply_second_after_cold_open(&reference).await;
    SegmentRdbStore::new(&fixture.root)
        .expect("reopen Text store after cold mutations")
        .save(&first_engine, STAGE1_TEXT_SECOND_SEQUENCE)
        .expect("write second Text sparse checkpoint");
    let second_common = stage1_text_delta_common_ids(&[
        STAGE1_TEXT_FIRST_DELETED_ID,
        STAGE1_TEXT_SECOND_DELETED_ID,
    ]);
    stage1_text_delta_assert_state(
        &first_server,
        &reference,
        &[
            (
                STAGE1_TEXT_UPDATED,
                vec![],
                "second checkpoint masks Text omitted by replacement",
            ),
            (
                STAGE1_TEXT_APPENDED,
                vec![],
                "second checkpoint masks old post-cold Text",
            ),
            (
                STAGE1_TEXT_FINAL,
                vec![STAGE1_TEXT_APPENDED_ID.to_owned()],
                "second checkpoint keeps post-cold Text update",
            ),
            (
                STAGE1_TEXT_SECOND_DELETED_ID,
                vec![],
                "second checkpoint masks post-cold deletion",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                second_common,
                "second checkpoint two-token BM25 Match",
            ),
        ],
        127,
        3.0,
        "second checkpoint Text corpus",
    )
    .await;

    let latest_generation = stage1_current_generation_dir(&fixture.root);
    let latest_manifest = stage1_read_manifest(&latest_generation);
    assert_eq!(
        stage1_text_delta_refs(&latest_manifest, "delta").len(),
        2,
        "second Text checkpoint retains ordinal one and adds ordinal two"
    );
    for (ordinal, sequence, expected_ids) in [
        (
            1,
            STAGE1_TEXT_FIRST_SEQUENCE,
            [
                STAGE1_TEXT_HIGH_ID,
                STAGE1_TEXT_FIRST_DELETED_ID,
                STAGE1_TEXT_APPENDED_ID,
            ],
        ),
        (
            2,
            STAGE1_TEXT_SECOND_SEQUENCE,
            [
                STAGE1_TEXT_HIGH_ID,
                STAGE1_TEXT_SECOND_DELETED_ID,
                STAGE1_TEXT_APPENDED_ID,
            ],
        ),
    ] {
        let delta = stage1_text_delta_for_sequence(&latest_manifest, sequence);
        assert_eq!(
            delta["ordinal"],
            json!(ordinal),
            "Text delta for applied sequence {sequence} keeps ordinal {ordinal}",
        );
        stage1_text_delta_assert_delta(
            &latest_generation,
            delta,
            expected_ids.iter().map(|id| (*id).to_owned()).collect(),
        );
    }
    stage1_text_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &latest_generation,
        &latest_manifest,
    );

    let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(latest_sequence, STAGE1_TEXT_SECOND_SEQUENCE);
    let latest_server = TestServer::new(router(AppState::open(latest_engine)))
        .expect("second Text cold HTTP server");
    let second_common = stage1_text_delta_common_ids(&[
        STAGE1_TEXT_FIRST_DELETED_ID,
        STAGE1_TEXT_SECOND_DELETED_ID,
    ]);
    stage1_text_delta_assert_state(
        &latest_server,
        &reference,
        &[
            (
                STAGE1_TEXT_FINAL,
                vec![STAGE1_TEXT_APPENDED_ID.to_owned()],
                "cold second checkpoint post-cold update",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                second_common,
                "cold second checkpoint BM25",
            ),
        ],
        127,
        3.0,
        "cold second checkpoint Text corpus",
    )
    .await;
}
// Scratch-only append for apps/lumen/e2e/indexing_durable_oracle.rs.
//
// Facets when applied:
// - Behavior: the two backend wrappers below call one parameterized contract.
//   It drives public vector indexing and kNN, then checks v2 sparse local rows,
//   hard-linked base files, cold reopen, and a second ordered layer.
// - Security: this contract uses the existing v2 local-row descriptor. The
//   established duplicate-local-row refusal in
//   `v2_current_refuses_number_sparse_delta_duplicate_stable_local_rows` covers
//   the shared CBOR decoder and CURRENT preservation. Vector-specific corrupt
//   payload refusal remains a later catalog-validation case once a vector delta
//   reader exists.
// - Performance: three local rows and inode equality are structure assertions,
//   not a latency or RSS claim. The approved stage6 30-minute release workload
//   owns that budget; this structural case does not run it. The controller-owned
//   vector scan-counter unit seam covers full-corpus enumeration separately.
//
// The high external ID is an opaque caller label. It is the 129th base row, not
// a dense runtime row numbered 900000.

const STAGE1_VECTOR_DELTA_COLLECTION: &str = "vector-sparse-delta";
const STAGE1_VECTOR_DELTA_FIELD: &str = "embedding";
const STAGE1_VECTOR_DELTA_ORDINARY_BASE_ROWS: usize = 128;
const STAGE1_VECTOR_DELTA_BASE_ROWS: usize = STAGE1_VECTOR_DELTA_ORDINARY_BASE_ROWS + 1;
const STAGE1_VECTOR_DELTA_BASE_SEQUENCE: u64 = 1_401;
const STAGE1_VECTOR_DELTA_FIRST_SEQUENCE: u64 = 1_402;
const STAGE1_VECTOR_DELTA_SECOND_SEQUENCE: u64 = 1_403;
const STAGE1_VECTOR_DELTA_HIGH_ID: &str = "runtime-id-900000";
const STAGE1_VECTOR_DELTA_APPENDED_ID: &str = "runtime-id-900001";
const STAGE1_VECTOR_DELTA_FIRST_DELETED_ID: &str = "vector-base-002";
const STAGE1_VECTOR_DELTA_SECOND_DELETED_ID: &str = "vector-base-004";

struct Stage1VectorDeltaFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: SegmentRdbStore,
    engine: Arc<Engine>,
    server: TestServer,
    base_name: String,
}

fn stage1_vector_delta_base_id(index: usize) -> String {
    format!("vector-base-{index:03}")
}

fn stage1_vector_delta_base_x(index: usize) -> f32 {
    match index {
        // Each anchor is only 0.1 from one changed or deleted vector. This
        // gives a clear nearest result without relying on HNSW score values.
        0 => 0.1,
        1 => 10.1,
        2 => 20.0,
        3 => 20.1,
        4 => 30.0,
        5 => 30.1,
        6 => 40.1,
        _ => 1_000.0 + index as f32,
    }
}

fn stage1_vector_delta_vector(x: f32) -> Value {
    json!([x, 0.0])
}

fn stage1_vector_delta_base_items() -> Vec<Value> {
    let mut items: Vec<Value> = (0..STAGE1_VECTOR_DELTA_ORDINARY_BASE_ROWS)
        .map(|index| {
            json!({
                "external_id": stage1_vector_delta_base_id(index),
                "field": STAGE1_VECTOR_DELTA_FIELD,
                "value": stage1_vector_delta_vector(stage1_vector_delta_base_x(index)),
            })
        })
        .collect();
    items.push(json!({
        "external_id": STAGE1_VECTOR_DELTA_HIGH_ID,
        "field": STAGE1_VECTOR_DELTA_FIELD,
        "value": stage1_vector_delta_vector(0.0),
    }));
    items
}

async fn stage1_vector_delta_create_collection(server: &TestServer, backend: &str) {
    server
        .put(&format!("/collections/{STAGE1_VECTOR_DELTA_COLLECTION}"))
        .json(&json!({ "fields": {
            STAGE1_VECTOR_DELTA_FIELD: {
                "type": "vector",
                "dim": 2,
                "metric": "l2",
                "backend": backend,
            },
        }}))
        .await
        .assert_status_ok();
}

async fn stage1_vector_delta_post_items(server: &TestServer, items: Vec<Value>) {
    for chunk in items.chunks(1_000) {
        server
            .post(&format!(
                "/collections/{STAGE1_VECTOR_DELTA_COLLECTION}/index"
            ))
            .json(&json!({ "items": chunk }))
            .await
            .assert_status_ok();
    }
}

async fn stage1_vector_delta_index(server: &TestServer, external_id: &str, x: f32) {
    stage1_vector_delta_post_items(
        server,
        vec![json!({
            "external_id": external_id,
            "field": STAGE1_VECTOR_DELTA_FIELD,
            "value": stage1_vector_delta_vector(x),
        })],
    )
    .await;
}

async fn stage1_vector_delta_delete(server: &TestServer, external_id: &str) {
    server
        .delete(&format!(
            "/collections/{STAGE1_VECTOR_DELTA_COLLECTION}/index/{external_id}"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
}

async fn stage1_vector_delta_fixture(backend: &str) -> Stage1VectorDeltaFixture {
    let dir = tempfile::tempdir().expect("vector sparse checkpoint root");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("create vector segment store");
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine.clone()))).expect("vector sparse HTTP server");
    stage1_vector_delta_create_collection(&server, backend).await;
    stage1_vector_delta_post_items(&server, stage1_vector_delta_base_items()).await;
    stage1_restore_legacy_base(&engine, "vector sparse-delta base");
    store
        .save(&engine, STAGE1_VECTOR_DELTA_BASE_SEQUENCE)
        .expect("write vector base generation");
    let base_name = stage1_reuse_current_name(&root);
    Stage1VectorDeltaFixture {
        _dir: dir,
        root,
        store,
        engine,
        server,
        base_name,
    }
}

async fn stage1_vector_delta_reference(backend: &str) -> TestServer {
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine))).expect("vector reference server");
    stage1_vector_delta_create_collection(&server, backend).await;
    stage1_vector_delta_post_items(&server, stage1_vector_delta_base_items()).await;
    server
}

async fn stage1_vector_delta_apply_first(server: &TestServer) {
    stage1_vector_delta_index(server, STAGE1_VECTOR_DELTA_HIGH_ID, 10.0).await;
    stage1_vector_delta_delete(server, STAGE1_VECTOR_DELTA_FIRST_DELETED_ID).await;
    stage1_vector_delta_index(server, STAGE1_VECTOR_DELTA_APPENDED_ID, 40.0).await;
}

async fn stage1_vector_delta_apply_second_after_cold_open(server: &TestServer) {
    stage1_vector_delta_index(server, STAGE1_VECTOR_DELTA_HIGH_ID, 60.0).await;
    stage1_vector_delta_delete(server, STAGE1_VECTOR_DELTA_SECOND_DELETED_ID).await;
    stage1_vector_delta_index(server, STAGE1_VECTOR_DELTA_APPENDED_ID, 50.0).await;
}

async fn stage1_vector_delta_knn_ids(server: &TestServer, x: f32, k: usize) -> Vec<String> {
    let response = server
        .post(&format!(
            "/collections/{STAGE1_VECTOR_DELTA_COLLECTION}/search"
        ))
        .json(&json!({
            "query": { "knn": {
                "field": STAGE1_VECTOR_DELTA_FIELD,
                "vector": stage1_vector_delta_vector(x),
                "k": k,
            }},
            "limit": k,
            "track_total": true,
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    body["hits"]
        .as_array()
        .expect("vector kNN hits array")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("vector hit external ID")
                .to_owned()
        })
        .collect()
}

async fn stage1_vector_delta_assert_nearest(
    server: &TestServer,
    reference: &TestServer,
    x: f32,
    expected: &str,
    context: &str,
) {
    let reference_ids = stage1_vector_delta_knn_ids(reference, x, 1).await;
    assert_eq!(
        reference_ids,
        vec![expected.to_owned()],
        "independent vector reference has the expected nearest ID for {context}"
    );
    assert_eq!(
        stage1_vector_delta_knn_ids(server, x, 1).await,
        reference_ids,
        "durable vector layers preserve the nearest ID for {context} without claiming a score"
    );
}

async fn stage1_vector_delta_assert_live_ids(
    server: &TestServer,
    reference: &TestServer,
    expected: &std::collections::BTreeSet<String>,
    context: &str,
) {
    let reference_ids: std::collections::BTreeSet<String> =
        stage1_vector_delta_knn_ids(reference, 0.0, STAGE1_VECTOR_DELTA_BASE_ROWS)
            .await
            .into_iter()
            .collect();
    assert_eq!(
        reference_ids, *expected,
        "independent vector reference has exactly the expected live IDs for {context}"
    );
    let durable_ids: std::collections::BTreeSet<String> =
        stage1_vector_delta_knn_ids(server, 0.0, STAGE1_VECTOR_DELTA_BASE_ROWS)
            .await
            .into_iter()
            .collect();
    assert_eq!(
        durable_ids, reference_ids,
        "durable vector layers retain and mask the same IDs for {context}"
    );
}

async fn stage1_vector_delta_assert_backend(server: &TestServer, backend: &str, context: &str) {
    let response = server.get("/admin/backup").await;
    response.assert_status_ok();
    let snapshot: Value = response.json();
    assert_eq!(
        snapshot["collections"][STAGE1_VECTOR_DELTA_COLLECTION]["fields"]
            [STAGE1_VECTOR_DELTA_FIELD]["spec"]["backend"],
        json!(backend),
        "cold and live vector schema must retain backend {backend} for {context}"
    );
}

fn stage1_vector_delta_base_ids() -> std::collections::BTreeSet<String> {
    (0..STAGE1_VECTOR_DELTA_ORDINARY_BASE_ROWS)
        .map(stage1_vector_delta_base_id)
        .chain(std::iter::once(STAGE1_VECTOR_DELTA_HIGH_ID.to_owned()))
        .collect()
}

fn stage1_vector_delta_first_ids() -> std::collections::BTreeSet<String> {
    let mut ids = stage1_vector_delta_base_ids();
    ids.remove(STAGE1_VECTOR_DELTA_FIRST_DELETED_ID);
    ids.insert(STAGE1_VECTOR_DELTA_APPENDED_ID.to_owned());
    ids
}

fn stage1_vector_delta_second_ids() -> std::collections::BTreeSet<String> {
    let mut ids = stage1_vector_delta_first_ids();
    ids.remove(STAGE1_VECTOR_DELTA_SECOND_DELETED_ID);
    ids
}

fn stage1_vector_delta_field_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
    stage1_reuse_catalog_collection(manifest, STAGE1_VECTOR_DELTA_COLLECTION)["segments"]
        .as_array()
        .expect("vector catalog segments")
        .iter()
        .filter(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!(STAGE1_VECTOR_DELTA_FIELD)
                && segment["kind"] == json!(kind)
        })
        .collect()
}

fn stage1_vector_delta_for_sequence(manifest: &Value, sequence: u64) -> &Value {
    let matching: Vec<_> = stage1_vector_delta_field_refs(manifest, "delta")
        .into_iter()
        .filter(|segment| segment["applied_seq"] == json!(sequence))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "vector catalog must contain exactly one delta for applied sequence {sequence}",
    );
    matching[0]
}

fn stage1_vector_delta_base_ref<'a>(manifest: &'a Value, role: &str) -> &'a Value {
    stage1_reuse_catalog_collection(manifest, STAGE1_VECTOR_DELTA_COLLECTION)["segments"]
        .as_array()
        .expect("vector catalog segments")
        .iter()
        .find(|segment| {
            segment["role"] == json!(role)
                && segment["field"] == json!(STAGE1_VECTOR_DELTA_FIELD)
                && segment["kind"] == json!("base")
                && segment["ordinal"] == json!(0)
        })
        .unwrap_or_else(|| panic!("vector catalog needs base {role} reference"))
}

fn stage1_vector_delta_touched_ids() -> std::collections::BTreeSet<String> {
    [
        STAGE1_VECTOR_DELTA_HIGH_ID,
        STAGE1_VECTOR_DELTA_FIRST_DELETED_ID,
        STAGE1_VECTOR_DELTA_APPENDED_ID,
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect()
}

fn stage1_vector_delta_second_touched_ids() -> std::collections::BTreeSet<String> {
    [
        STAGE1_VECTOR_DELTA_HIGH_ID,
        STAGE1_VECTOR_DELTA_SECOND_DELETED_ID,
        STAGE1_VECTOR_DELTA_APPENDED_ID,
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect()
}

fn stage1_vector_delta_assert_delta(
    generation: &Path,
    delta: &Value,
    expected_ids: std::collections::BTreeSet<String>,
) {
    assert_eq!(delta["role"], json!("field"));
    assert_eq!(delta["field"], json!(STAGE1_VECTOR_DELTA_FIELD));
    assert_eq!(delta["kind"], json!("delta"));
    assert_eq!(delta["format"], json!("lseg-v1"));
    assert!(
        delta["path"].as_str().is_some(),
        "vector delta names a segment"
    );
    let count = stage1_keyword_delta_rows_count(delta);
    assert_eq!(
        count, 3,
        "vector delta local rows must cover exactly update, delete, and append"
    );
    assert!(
        count < STAGE1_VECTOR_DELTA_BASE_ROWS as u64,
        "vector delta must not allocate local rows for every base vector"
    );
    let rows = stage1_keyword_delta_read_rows(generation, delta);
    let unique: std::collections::BTreeSet<String> = rows.iter().cloned().collect();
    assert_eq!(
        rows.len() as u64,
        count,
        "vector local rows match their count"
    );
    assert_eq!(
        unique.len(),
        rows.len(),
        "vector local rows must not repeat stable external IDs"
    );
    assert_eq!(
        unique, expected_ids,
        "vector local rows must name exactly the vectors changed in this layer"
    );
}

#[cfg(unix)]
fn stage1_vector_delta_assert_base_hardlinked(
    base_generation: &Path,
    base_manifest: &Value,
    later_generation: &Path,
    later_manifest: &Value,
) {
    for role in ["field", "vector_eids"] {
        let base = stage1_vector_delta_base_ref(base_manifest, role);
        let retained = stage1_vector_delta_base_ref(later_manifest, role);
        let base_path = base_generation.join(base["path"].as_str().expect("vector base path"));
        let retained_path = later_generation.join(
            retained["path"]
                .as_str()
                .expect("retained vector base path"),
        );
        let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect vector base");
        let retained_metadata =
            std::fs::symlink_metadata(&retained_path).expect("inspect retained vector base");
        assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
        assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
        assert_eq!(
            base_metadata.ino(),
            retained_metadata.ino(),
            "unchanged vector {role} base is retained by hard link, not a copy"
        );
        assert!(
            retained_metadata.nlink() >= 2,
            "retained vector {role} base has multiple links"
        );
    }
}

#[cfg(unix)]
async fn stage1_vector_delta_contract(backend: &str) {
    let fixture = stage1_vector_delta_fixture(backend).await;
    let reference = stage1_vector_delta_reference(backend).await;
    let base_generation = fixture.root.join(&fixture.base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    stage1_vector_delta_assert_backend(&fixture.server, backend, "base checkpoint").await;
    stage1_vector_delta_assert_live_ids(
        &fixture.server,
        &reference,
        &stage1_vector_delta_base_ids(),
        "base checkpoint",
    )
    .await;
    stage1_vector_delta_assert_nearest(
        &fixture.server,
        &reference,
        0.0,
        STAGE1_VECTOR_DELTA_HIGH_ID,
        "base high vector",
    )
    .await;

    stage1_vector_delta_apply_first(&fixture.server).await;
    stage1_vector_delta_apply_first(&reference).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_VECTOR_DELTA_FIRST_SEQUENCE)
        .expect("write first vector sparse checkpoint");
    let first_name = stage1_reuse_current_name(&fixture.root);
    assert_ne!(
        first_name, fixture.base_name,
        "vector mutation creates a generation"
    );
    let first_generation = fixture.root.join(&first_name);
    let first_manifest = stage1_read_manifest(&first_generation);
    assert_eq!(
        stage1_vector_delta_field_refs(&first_manifest, "delta").len(),
        1,
        "first vector checkpoint publishes one delta"
    );
    let first_delta =
        stage1_vector_delta_for_sequence(&first_manifest, STAGE1_VECTOR_DELTA_FIRST_SEQUENCE);
    assert_eq!(
        first_delta["ordinal"],
        json!(1),
        "first vector delta immediately follows the restored base",
    );
    stage1_vector_delta_assert_delta(
        &first_generation,
        first_delta,
        stage1_vector_delta_touched_ids(),
    );
    stage1_vector_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &first_generation,
        &first_manifest,
    );
    stage1_vector_delta_assert_backend(&fixture.server, backend, "first live checkpoint").await;
    stage1_vector_delta_assert_live_ids(
        &fixture.server,
        &reference,
        &stage1_vector_delta_first_ids(),
        "first live checkpoint",
    )
    .await;
    for (x, expected, context) in [
        (
            0.0,
            "vector-base-000",
            "first checkpoint masks old high vector",
        ),
        (
            20.0,
            "vector-base-003",
            "first checkpoint masks deleted vector",
        ),
        (
            10.0,
            STAGE1_VECTOR_DELTA_HIGH_ID,
            "first checkpoint retains updated high vector",
        ),
        (
            40.0,
            STAGE1_VECTOR_DELTA_APPENDED_ID,
            "first checkpoint retains appended vector",
        ),
        (
            stage1_vector_delta_base_x(7),
            "vector-base-007",
            "first checkpoint retains untouched base vector",
        ),
    ] {
        stage1_vector_delta_assert_nearest(&fixture.server, &reference, x, expected, context).await;
    }

    let (first_engine, first_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(first_sequence, STAGE1_VECTOR_DELTA_FIRST_SEQUENCE);
    let first_server = TestServer::new(router(AppState::open(first_engine.clone())))
        .expect("first vector cold HTTP server");
    stage1_vector_delta_assert_backend(&first_server, backend, "first cold checkpoint").await;
    stage1_vector_delta_assert_live_ids(
        &first_server,
        &reference,
        &stage1_vector_delta_first_ids(),
        "first cold checkpoint",
    )
    .await;
    stage1_vector_delta_assert_nearest(
        &first_server,
        &reference,
        10.0,
        STAGE1_VECTOR_DELTA_HIGH_ID,
        "first cold checkpoint updated high vector",
    )
    .await;

    stage1_vector_delta_apply_second_after_cold_open(&first_server).await;
    stage1_vector_delta_apply_second_after_cold_open(&reference).await;
    SegmentRdbStore::new(&fixture.root)
        .expect("reopen vector store after cold mutations")
        .save(&first_engine, STAGE1_VECTOR_DELTA_SECOND_SEQUENCE)
        .expect("write second vector sparse checkpoint");
    let latest_generation = stage1_current_generation_dir(&fixture.root);
    let latest_manifest = stage1_read_manifest(&latest_generation);
    let deltas = stage1_vector_delta_field_refs(&latest_manifest, "delta");
    assert_eq!(
        deltas.len(),
        2,
        "second vector checkpoint retains ordinal one and adds ordinal two"
    );
    let ordinals: Vec<u64> = deltas
        .iter()
        .map(|segment| segment["ordinal"].as_u64().expect("vector delta ordinal"))
        .collect();
    assert_eq!(ordinals, [1, 2], "vector delta layers remain ordered");
    let first_delta =
        stage1_vector_delta_for_sequence(&latest_manifest, STAGE1_VECTOR_DELTA_FIRST_SEQUENCE);
    let second_delta =
        stage1_vector_delta_for_sequence(&latest_manifest, STAGE1_VECTOR_DELTA_SECOND_SEQUENCE);
    assert_eq!(first_delta["ordinal"], json!(1));
    assert_eq!(second_delta["ordinal"], json!(2));
    stage1_vector_delta_assert_delta(
        &latest_generation,
        first_delta,
        stage1_vector_delta_touched_ids(),
    );
    stage1_vector_delta_assert_delta(
        &latest_generation,
        second_delta,
        stage1_vector_delta_second_touched_ids(),
    );
    stage1_vector_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &latest_generation,
        &latest_manifest,
    );
    stage1_vector_delta_assert_live_ids(
        &first_server,
        &reference,
        &stage1_vector_delta_second_ids(),
        "second live checkpoint",
    )
    .await;
    for (x, expected, context) in [
        (
            10.0,
            "vector-base-001",
            "second checkpoint masks old high vector",
        ),
        (
            30.0,
            "vector-base-005",
            "second checkpoint masks post-cold deleted vector",
        ),
        (
            40.0,
            "vector-base-006",
            "second checkpoint masks old appended vector",
        ),
        (
            60.0,
            STAGE1_VECTOR_DELTA_HIGH_ID,
            "second checkpoint retains post-cold high update",
        ),
        (
            50.0,
            STAGE1_VECTOR_DELTA_APPENDED_ID,
            "second checkpoint retains post-cold appended update",
        ),
        (
            stage1_vector_delta_base_x(7),
            "vector-base-007",
            "second checkpoint retains untouched base vector",
        ),
    ] {
        stage1_vector_delta_assert_nearest(&first_server, &reference, x, expected, context).await;
    }

    let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(latest_sequence, STAGE1_VECTOR_DELTA_SECOND_SEQUENCE);
    let latest_server = TestServer::new(router(AppState::open(latest_engine)))
        .expect("second vector cold HTTP server");
    stage1_vector_delta_assert_backend(&latest_server, backend, "second cold checkpoint").await;
    stage1_vector_delta_assert_live_ids(
        &latest_server,
        &reference,
        &stage1_vector_delta_second_ids(),
        "second cold checkpoint",
    )
    .await;
    for (x, expected, context) in [
        (60.0, STAGE1_VECTOR_DELTA_HIGH_ID, "second cold high update"),
        (
            50.0,
            STAGE1_VECTOR_DELTA_APPENDED_ID,
            "second cold appended update",
        ),
        (
            stage1_vector_delta_base_x(7),
            "vector-base-007",
            "second cold untouched base vector",
        ),
    ] {
        stage1_vector_delta_assert_nearest(&latest_server, &reference, x, expected, context).await;
    }
}

#[cfg(unix)]
#[tokio::test]
async fn v2_flat_cpu_vector_sparse_delta_reuses_base_and_preserves_cold_layers() {
    stage1_vector_delta_contract("flat-cpu").await;
}

#[cfg(unix)]
#[tokio::test]
async fn v2_hnsw_cpu_vector_sparse_delta_reuses_base_and_preserves_cold_layers() {
    stage1_vector_delta_contract("hnsw-cpu").await;
}
const STAGE1_CAPTURE_BARRIER_HOT_COLLECTION: &str = "capture-barrier-hot";
const STAGE1_CAPTURE_BARRIER_IDLE_COLLECTION: &str = "capture-barrier-idle";
const STAGE1_CAPTURE_BARRIER_FIELD: &str = "kw";
const STAGE1_CAPTURE_BARRIER_HOT_ID: &str = "hot-record";
const STAGE1_CAPTURE_BARRIER_IDLE_ID: &str = "idle-record";
const STAGE1_CAPTURE_BARRIER_HOT_VALUE: &str = "not-yet-aof-durable";
const STAGE1_CAPTURE_BARRIER_IDLE_VALUE: &str = "idle-value";

struct Stage1CaptureBarrierFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    aof_path: PathBuf,
    store: Arc<SegmentRdbStore>,
    engine: Arc<Engine>,
    writer: Arc<WriteCoordinator>,
    aof: SharedAof,
    wal: Arc<MemWal>,
    checkpoint: Arc<LocalCheckpointSink>,
    server: TestServer,
}

struct Stage1CaptureBarrierAofHold {
    release: Option<std::sync::mpsc::Sender<()>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Stage1CaptureBarrierAofHold {
    fn release(&mut self) {
        self.release
            .take()
            .expect("AOF hold is released once")
            .send(())
            .expect("AOF hold worker waits for release");
        self.worker
            .take()
            .expect("AOF hold worker exists")
            .join()
            .expect("AOF hold worker exits cleanly");
    }
}

impl Drop for Stage1CaptureBarrierAofHold {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn stage1_capture_barrier_fixture() -> Stage1CaptureBarrierFixture {
    let dir = tempfile::tempdir().expect("capture barrier fixture directory");
    let root = dir.path().join("segments");
    let aof_path = dir.path().join("aof.log");
    let store = Arc::new(SegmentRdbStore::new(&root).expect("capture barrier segment store"));
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path).expect("capture barrier AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
    let checkpoint = Arc::new(LocalCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: writer.clone(),
        aof: aof.clone(),
    });
    let state = AppState::with_components(
        engine.clone(),
        Arc::new(AuthConfig::open()),
        writer.clone() as Arc<dyn WriteSink>,
    )
    .with_checkpoint(checkpoint.clone());
    let server = TestServer::new(router(state)).expect("capture barrier HTTP server");
    Stage1CaptureBarrierFixture {
        _dir: dir,
        root,
        aof_path,
        store,
        engine,
        writer,
        aof,
        wal,
        checkpoint,
        server,
    }
}

fn stage1_capture_barrier_create_entry(collection_id: &str) -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: collection_id.to_owned(),
        req: serde_json::from_value(json!({
            "fields": {
                STAGE1_CAPTURE_BARRIER_FIELD: { "type": "keyword" },
            },
        }))
        .expect("capture barrier keyword schema"),
    }
}

fn stage1_capture_barrier_index_entry(
    collection_id: &str,
    external_id: &str,
    value: &str,
) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: collection_id.to_owned(),
        req: serde_json::from_value(json!({
            "items": [{
                "external_id": external_id,
                "field": STAGE1_CAPTURE_BARRIER_FIELD,
                "value": value,
            }],
        }))
        .expect("capture barrier keyword index entry"),
    }
}

async fn stage1_capture_barrier_setup(fixture: &Stage1CaptureBarrierFixture) -> u64 {
    for collection in [
        STAGE1_CAPTURE_BARRIER_HOT_COLLECTION,
        STAGE1_CAPTURE_BARRIER_IDLE_COLLECTION,
    ] {
        fixture
            .writer
            .submit(stage1_capture_barrier_create_entry(collection))
            .await
            .expect("create capture barrier collection");
    }
    fixture
        .writer
        .submit(stage1_capture_barrier_index_entry(
            STAGE1_CAPTURE_BARRIER_IDLE_COLLECTION,
            STAGE1_CAPTURE_BARRIER_IDLE_ID,
            STAGE1_CAPTURE_BARRIER_IDLE_VALUE,
        ))
        .await
        .expect("index idle fixture value");
    fixture
        .checkpoint
        .checkpoint_now()
        .await
        .expect("write baseline checkpoint");
    let baseline = fixture.writer.applied_seq();
    assert!(
        baseline > 0,
        "baseline setup must apply real records before the cut"
    );
    baseline
}

async fn stage1_capture_barrier_http_term_ids(
    server: &TestServer,
    collection: &str,
    value: &str,
) -> Vec<String> {
    let response = server
        .post(&format!("/collections/{collection}/search"))
        .json(&json!({
            "query": { "term": {
                "field": STAGE1_CAPTURE_BARRIER_FIELD,
                "value": value,
            }},
            "limit": 16,
            "track_total": true,
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let mut ids: Vec<String> = body["hits"]
        .as_array()
        .expect("capture barrier term hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("capture barrier hit ID")
                .to_owned()
        })
        .collect();
    ids.sort();
    ids
}

fn stage1_capture_barrier_engine_term_ids(
    engine: &Arc<Engine>,
    collection: &str,
    value: &str,
) -> Vec<String> {
    let response = engine
        .search(
            collection,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: STAGE1_CAPTURE_BARRIER_FIELD.to_owned(),
                    value: FieldValue::String(value.to_owned()),
                }),
                limit: 16,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("capture barrier engine term query");
    let mut ids: Vec<String> = response
        .hits
        .iter()
        .map(|hit| hit.external_id.clone())
        .collect();
    ids.sort();
    ids
}

fn stage1_capture_barrier_recovery_cut(root: &Path, aof_path: &Path) -> (u64, u64, Vec<String>) {
    let loaded = SegmentRdbStore::new(root)
        .expect("open capture barrier root")
        .load_current_generation()
        .expect("load capture barrier CURRENT")
        .expect("capture barrier CURRENT generation");
    let checkpoint_sequence = loaded.sequence;
    let replayed =
        replay_aof_into(&loaded.engine, aof_path, checkpoint_sequence).expect("replay cut AOF");
    let hot_ids = stage1_capture_barrier_engine_term_ids(
        &loaded.engine,
        STAGE1_CAPTURE_BARRIER_HOT_COLLECTION,
        STAGE1_CAPTURE_BARRIER_HOT_VALUE,
    );
    (checkpoint_sequence, replayed, hot_ids)
}

fn stage1_capture_barrier_start_aof_hold(
    aof: SharedAof,
) -> (
    Stage1CaptureBarrierAofHold,
    tokio::sync::oneshot::Receiver<()>,
) {
    let (locked_tx, locked_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let _guard = aof.lock().expect("capture barrier AOF lock");
        let _ = locked_tx.send(());
        let _ = release_rx.recv();
    });
    (
        Stage1CaptureBarrierAofHold {
            release: Some(release_tx),
            worker: Some(worker),
        },
        locked_rx,
    )
}

async fn stage1_capture_barrier_hold_aof(aof: SharedAof) -> Stage1CaptureBarrierAofHold {
    let (hold, locked) = stage1_capture_barrier_start_aof_hold(aof);
    tokio::time::timeout(Duration::from_secs(1), locked)
        .await
        .expect("dedicated AOF-lock thread must acquire the mutex")
        .expect("AOF-lock thread must report acquisition");
    hold
}

async fn stage1_capture_barrier_wait_for_engine_visibility(engine: &Arc<Engine>) -> bool {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if stage1_capture_barrier_engine_term_ids(
                engine,
                STAGE1_CAPTURE_BARRIER_HOT_COLLECTION,
                STAGE1_CAPTURE_BARRIER_HOT_VALUE,
            ) == vec![STAGE1_CAPTURE_BARRIER_HOT_ID.to_owned()]
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .is_ok()
}

async fn stage1_capture_barrier_current_changed_during_hold(root: &Path, before: &[u8]) -> bool {
    tokio::time::timeout(Duration::from_millis(250), async {
        loop {
            if std::fs::read(root.join("CURRENT")).expect("read CURRENT during capture") != before {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .is_ok()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_capture_never_publishes_engine_state_before_its_aof_record_is_durable() {
    let fixture = stage1_capture_barrier_fixture();
    let baseline_sequence = stage1_capture_barrier_setup(&fixture).await;

    let mut observed_wal = fixture
        .wal
        .subscribe(baseline_sequence)
        .await
        .expect("observe the hot record in MemWal");
    let mut aof_hold = stage1_capture_barrier_hold_aof(fixture.aof.clone()).await;
    let write = {
        let writer = fixture.writer.clone();
        tokio::spawn(async move {
            writer
                .submit(stage1_capture_barrier_index_entry(
                    STAGE1_CAPTURE_BARRIER_HOT_COLLECTION,
                    STAGE1_CAPTURE_BARRIER_HOT_ID,
                    STAGE1_CAPTURE_BARRIER_HOT_VALUE,
                ))
                .await
        })
    };

    let (hot_sequence, _) = tokio::time::timeout(Duration::from_secs(1), observed_wal.next())
        .await
        .expect("hot record must publish to MemWal before the capture cut")
        .expect("MemWal stream remains open")
        .expect("MemWal delivers the hot record");
    assert_eq!(
        hot_sequence,
        baseline_sequence + 1,
        "the cut follows a concrete next WAL record"
    );
    assert_eq!(
        fixture.wal.latest_seq().await.expect("read MemWal head"),
        hot_sequence,
        "the cut cannot pass because no record was published"
    );
    drop(observed_wal);

    // Current code makes the engine visible before it can acquire the held
    // AOF lock. A correct implementation may choose AOF-first ordering, so this
    // observation only synchronizes the old path and is never a requirement.
    let _engine_was_visible_before_aof =
        stage1_capture_barrier_wait_for_engine_visibility(&fixture.engine).await;

    let checkpoint_sequence = fixture.writer.applied_seq();
    assert_eq!(
        checkpoint_sequence, baseline_sequence,
        "the held AOF must keep the checkpoint watermark at the prior durable record"
    );
    let current_before_capture =
        std::fs::read(fixture.root.join("CURRENT")).expect("read CURRENT before capture save");
    let checkpoint_save = {
        let store = fixture.store.clone();
        let engine = fixture.engine.clone();
        tokio::task::spawn_blocking(move || store.save(&engine, checkpoint_sequence))
    };
    let current_changed_while_aof_is_held =
        stage1_capture_barrier_current_changed_during_hold(&fixture.root, &current_before_capture)
            .await;

    let idle_query = tokio::time::timeout(
        Duration::from_millis(500),
        stage1_capture_barrier_http_term_ids(
            &fixture.server,
            STAGE1_CAPTURE_BARRIER_IDLE_COLLECTION,
            STAGE1_CAPTURE_BARRIER_IDLE_VALUE,
        ),
    )
    .await;
    let idle_ids = idle_query.expect("idle collection query must finish while capture save runs");
    let current_changed_while_aof_is_held = current_changed_while_aof_is_held
        || std::fs::read(fixture.root.join("CURRENT")).expect("read CURRENT before AOF release")
            != current_before_capture;
    let pre_release_cut = current_changed_while_aof_is_held
        .then(|| stage1_capture_barrier_recovery_cut(&fixture.root, &fixture.aof_path));

    // Always release and join the real capture and write before checking the
    // recovery cut. The test must not leave a blocked mutex owner or pass by
    // ignoring a timed-out path.
    aof_hold.release();
    tokio::time::timeout(Duration::from_secs(2), checkpoint_save)
        .await
        .expect("real checkpoint save must finish after AOF release")
        .expect("checkpoint save task must not panic")
        .expect("real checkpoint save must succeed");
    let write_result = tokio::time::timeout(Duration::from_secs(2), write)
        .await
        .expect("hot write must finish after AOF release")
        .expect("hot write task must not panic")
        .expect("hot write must succeed");

    assert_eq!(
        idle_ids,
        vec![STAGE1_CAPTURE_BARRIER_IDLE_ID.to_owned()],
        "an idle collection query must finish while the hot AOF write is blocked"
    );
    let _ = write_result;
    assert_eq!(
        fixture.writer.applied_seq(),
        hot_sequence,
        "the test must observe the concrete hot record apply after release"
    );

    if let Some((pre_release_checkpoint_sequence, pre_release_replayed, pre_release_hot_ids)) =
        pre_release_cut
    {
        assert_eq!(
            pre_release_checkpoint_sequence, baseline_sequence,
            "a premature capture must label CURRENT with the prior durable watermark"
        );
        assert_eq!(
            pre_release_replayed, 0,
            "the held AOF has no durable tail to repair a premature checkpoint"
        );
        assert_eq!(
            pre_release_hot_ids,
            Vec::<String>::new(),
            "a recovery cut at the prior watermark must not expose a record whose AOF append is still blocked"
        );
    }

    let (final_checkpoint_sequence, final_replayed, final_hot_ids) =
        stage1_capture_barrier_recovery_cut(&fixture.root, &fixture.aof_path);
    assert_eq!(
        final_hot_ids,
        vec![STAGE1_CAPTURE_BARRIER_HOT_ID.to_owned()],
        "after release, recovery from CURRENT plus the AOF tail must retain the real hot write"
    );
    assert!(
        final_checkpoint_sequence == hot_sequence || final_replayed == hot_sequence,
        "the final recovery cut must account for the hot record in the checkpoint or AOF tail"
    );
}

mod published_checkpoint_overlay_release {
    //! # Facets
    //!
    //! - Behavior: `apps/lumen/e2e/indexing_durable_oracle.rs:5338` asserts
    //!   public IDs; `:5522` compares live/cold snapshots; calls at `:5540`,
    //!   `:5544`, `:5552`, and `:5556` cover both checkpoint layers.
    //! - Security: `apps/lumen/src/segment_rdb.rs:500` starts the persisted
    //!   generation read; `:2543` and `:2569` mutate LocalRows; `:1138` and
    //!   `:1145` assert refusal and unchanged `CURRENT`. This release-only
    //!   path adds no caller-controlled byte, path, or identifier input.
    //! - Performance: `:5505` measures actual forward/token driver size;
    //!   `:5546`, `:5547`, `:5558`, and `:5559` require zero after durable publish.
    //!   This case does not claim latency/RSS. The user-approved #4246 stage6
    //!   budget is 2.5 CPU and 16 GiB for 30 minutes, 100 mixed doc ops/s at >=95%,
    //!   10 QPS p99 <=1s, every query <=5s, zero errors, and RSS <=12 GiB.
    //!   This bounded structural check does not run that workload.

    use super::*;

    const FIELDS: [&str; 5] = ["kw", "num", "tags", "sig", "body"];
    const UPDATED_ID: &str = "overlay-doc-000";
    const DELETED_ID: &str = "overlay-doc-001";
    const UNTOUCHED_ID: &str = "overlay-doc-002";
    const APPENDED_ID: &str = "overlay-doc-003";

    async fn create_all_field_schema(server: &TestServer) {
        create_schema(server).await;
        server
            .put("/collections/docs")
            .json(&json!({
                "fields": {
                    "tags": { "type": "set" },
                    "sig": { "type": "hash" }
                }
            }))
            .await
            .assert_status_ok();
    }

    async fn index_document(
        server: &TestServer,
        external_id: &str,
        keyword: &str,
        number: f64,
        tags: &[&str],
        hash: &str,
        body: &str,
    ) {
        server
            .post("/collections/docs/index")
            .json(&json!({
                "items": [
                    {
                        "external_id": external_id,
                        "field": "kw",
                        "value": keyword,
                    },
                    {
                        "external_id": external_id,
                        "field": "num",
                        "value": number,
                    },
                    {
                        "external_id": external_id,
                        "field": "tags",
                        "value": tags,
                    },
                    {
                        "external_id": external_id,
                        "field": "sig",
                        "value": hash,
                    },
                    {
                        "external_id": external_id,
                        "field": "body",
                        "value": body,
                    }
                ]
            }))
            .await
            .assert_status_ok();
    }

    async fn index_base_documents(server: &TestServer) {
        index_document(
            server,
            UPDATED_ID,
            "overlay-kw-base-0",
            1_000.0,
            &["overlay-tag-base-0"],
            "000000000000a000",
            "overlay-text-base-0 shared",
        )
        .await;
        index_document(
            server,
            DELETED_ID,
            "overlay-kw-base-1",
            1_001.0,
            &["overlay-tag-base-1"],
            "000000000000a001",
            "overlay-text-base-1 shared",
        )
        .await;
        index_document(
            server,
            UNTOUCHED_ID,
            "overlay-kw-base-2",
            1_002.0,
            &["overlay-tag-base-2"],
            "000000000000a002",
            "overlay-text-base-2 shared",
        )
        .await;
    }

    async fn apply_first_delta(server: &TestServer) {
        index_document(
            server,
            UPDATED_ID,
            "overlay-kw-first",
            9_001.0,
            &["overlay-tag-first"],
            "000000000000f001",
            "overlay-text-first shared",
        )
        .await;
        server
            .delete(&format!("/collections/docs/docs/{DELETED_ID}"))
            .await
            .assert_status(axum::http::StatusCode::NO_CONTENT);
        index_document(
            server,
            APPENDED_ID,
            "overlay-kw-appended",
            9_003.0,
            &["overlay-tag-appended"],
            "000000000000f003",
            "overlay-text-appended shared",
        )
        .await;
    }

    async fn apply_second_delta(server: &TestServer) {
        index_document(
            server,
            UPDATED_ID,
            "overlay-kw-final",
            9_002.0,
            &["overlay-tag-final"],
            "000000000000f002",
            "overlay-text-final shared",
        )
        .await;
    }

    async fn search_ids(server: &TestServer, query: Value, context: &str) -> Vec<String> {
        let body = http_search(server, query, context).await;
        let mut ids: Vec<String> = hit_ids(&body).into_iter().map(|id| id.to_owned()).collect();
        ids.sort();
        assert_eq!(
            body["total"].as_u64(),
            Some(ids.len() as u64),
            "{context}: total must equal the complete small fixture result: {body}"
        );
        ids
    }

    async fn assert_query(server: &TestServer, query: Value, expected: &[&str], context: &str) {
        let mut expected: Vec<String> = expected.iter().map(|id| (*id).to_owned()).collect();
        expected.sort();
        assert_eq!(
            search_ids(server, query, context).await,
            expected,
            "{context}: public search result"
        );
    }

    async fn assert_first_semantics(server: &TestServer, phase: &str) {
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-first" } }),
            &[UPDATED_ID],
            &format!("{phase}: updated Keyword"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-base-0" } }),
            &[],
            &format!("{phase}: old Keyword is masked"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-base-1" } }),
            &[],
            &format!("{phase}: deleted document remains masked"),
        )
        .await;
        assert_query(
            server,
            json!({ "range": {
                "field": "num",
                "gte": 9_001.0,
                "lte": 9_001.0,
            }}),
            &[UPDATED_ID],
            &format!("{phase}: updated Number"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "tags", "value": "overlay-tag-first" } }),
            &[UPDATED_ID],
            &format!("{phase}: updated Set"),
        )
        .await;
        assert_query(
            server,
            json!({ "hamming": {
                "field": "sig",
                "hash": "000000000000f001",
                "max_distance": 0,
            }}),
            &[UPDATED_ID],
            &format!("{phase}: updated Hash"),
        )
        .await;
        assert_query(
            server,
            json!({ "match": {
                "field": "body",
                "text": "overlay-text-first",
                "op": "and",
            }}),
            &[UPDATED_ID],
            &format!("{phase}: updated Text"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-appended" } }),
            &[APPENDED_ID],
            &format!("{phase}: appended document"),
        )
        .await;
    }

    async fn assert_second_semantics(server: &TestServer, phase: &str) {
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-final" } }),
            &[UPDATED_ID],
            &format!("{phase}: newest Keyword"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-first" } }),
            &[],
            &format!("{phase}: first Keyword does not resurrect"),
        )
        .await;
        assert_query(
            server,
            json!({ "range": {
                "field": "num",
                "gte": 9_002.0,
                "lte": 9_002.0,
            }}),
            &[UPDATED_ID],
            &format!("{phase}: newest Number"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "tags", "value": "overlay-tag-final" } }),
            &[UPDATED_ID],
            &format!("{phase}: newest Set"),
        )
        .await;
        assert_query(
            server,
            json!({ "hamming": {
                "field": "sig",
                "hash": "000000000000f002",
                "max_distance": 0,
            }}),
            &[UPDATED_ID],
            &format!("{phase}: newest Hash"),
        )
        .await;
        assert_query(
            server,
            json!({ "match": {
                "field": "body",
                "text": "overlay-text-final",
                "op": "and",
            }}),
            &[UPDATED_ID],
            &format!("{phase}: newest Text"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-base-1" } }),
            &[],
            &format!("{phase}: deleted document stays masked"),
        )
        .await;
    }

    fn assert_probe_observes_live_delta(engine: &Arc<Engine>, phase: &str) {
        for field in FIELDS {
            let (driver_len, has_segment) = engine
                .segment_field_probe(COLLECTION, field)
                .expect("all tested overlay fields remain probeable");
            assert!(
                has_segment,
                "{phase}: {field} must retain its published base segment while a delta is live"
            );
            assert!(
                driver_len > 0,
                "{phase}: {field} probe must see the test's live overlay before publication"
            );
        }
    }

    fn assert_probe_is_empty(engine: &Arc<Engine>, phase: &str) {
        for field in FIELDS {
            let (driver_len, has_segment) = engine
                .segment_field_probe(COLLECTION, field)
                .expect("all tested overlay fields remain probeable");
            assert!(
                has_segment,
                "{phase}: {field} must retain an attached immutable segment after publication"
            );
            assert_eq!(
                driver_len, 0,
                "{phase}: published {field} delta must not remain in the live forward/token driver"
            );
        }
    }

    fn load_current_engine(fixture: &Fixture) -> Arc<Engine> {
        fixture
            .store
            .load_current_generation()
            .expect("load checkpoint CURRENT")
            .expect("published checkpoint generation")
            .engine
    }

    fn assert_snapshot_matches_cold(live: &Arc<Engine>, cold: &Arc<Engine>, phase: &str) {
        assert_eq!(
            digest(live),
            digest(cold),
            "{phase}: public snapshots must agree across the published recovery cut"
        );
    }

    #[tokio::test]
    async fn published_incremental_checkpoints_release_scalar_and_text_overlays_live_and_cold() {
        let fixture = fixture();
        create_all_field_schema(&fixture.server).await;
        index_base_documents(&fixture.server).await;
        checkpoint(&fixture.server).await;
        assert_probe_is_empty(&fixture.engine, "sealed base");

        apply_first_delta(&fixture.server).await;
        assert_probe_observes_live_delta(&fixture.engine, "first live delta");
        checkpoint(&fixture.server).await;
        assert_first_semantics(&fixture.server, "first live checkpoint").await;
        let first_cold = load_current_engine(&fixture);
        let first_cold_server = TestServer::new(router(AppState::open(first_cold.clone())))
            .expect("first cold query server");
        assert_first_semantics(&first_cold_server, "first cold checkpoint").await;
        assert_snapshot_matches_cold(&fixture.engine, &first_cold, "first checkpoint");
        assert_probe_is_empty(&fixture.engine, "first live checkpoint");
        assert_probe_is_empty(&first_cold, "first cold checkpoint");

        apply_second_delta(&fixture.server).await;
        assert_probe_observes_live_delta(&fixture.engine, "second live delta");
        checkpoint(&fixture.server).await;
        assert_second_semantics(&fixture.server, "second live checkpoint").await;
        let second_cold = load_current_engine(&fixture);
        let second_cold_server = TestServer::new(router(AppState::open(second_cold.clone())))
            .expect("second cold query server");
        assert_second_semantics(&second_cold_server, "second cold checkpoint").await;
        assert_snapshot_matches_cold(&fixture.engine, &second_cold, "second checkpoint");
        assert_probe_is_empty(&fixture.engine, "second live checkpoint");
        assert_probe_is_empty(&second_cold, "second cold checkpoint");
    }
}

mod first_compaction_contract {
    //! # Facets
    //!
    //! - Behavior: v2_four_keyword_deltas_compact_and_preserve_tombstone_across_live_cold_and_retained_base
    //!   writes a real base and four real sparse checkpoints through SegmentRdbStore.
    //!   It requires layer reduction, then checks current live and cold searches
    //!   plus a retained first generation.
    //! - Security: apps/lumen/src/segment_rdb.rs:543 reads the local generation
    //!   bytes on cold open. Existing index_durable_oracle refusal cases beginning
    //!   at apps/lumen/e2e/indexing_durable_oracle.rs:970 feed malformed catalog
    //!   bytes and preserve CURRENT. This compaction case uses the same reader for
    //!   the compacted generation and the retained predecessor.
    //! - Performance: .aw/workitems/deliveries/lumen061-04-bounded-compaction-backpressure.md:3
    //!   and :30 require a compaction request at four deltas and a hard limit of
    //!   sixteen. This bounded structural case asserts the four-delta reduction.
    //!   It does not measure the separate stage-6 latency or RSS budget.
    //!
    //! Append this source to apps/lumen/e2e/indexing_durable_oracle.rs. It relies
    //! only on helpers already in that target and uses no compactor-specific API.

    #[cfg(unix)]
    const FIRST_COMPACTION_COLLECTION: &str = "first-compaction";
    #[cfg(unix)]
    const FIRST_COMPACTION_BASE_ROWS: usize = 256;
    #[cfg(unix)]
    const FIRST_COMPACTION_BASE_SEQUENCE: u64 = 9_100;
    #[cfg(unix)]
    const FIRST_COMPACTION_HOT_ID: &str = "first-compaction-hot";
    #[cfg(unix)]
    const FIRST_COMPACTION_DELETED_ID: &str = "first-compaction-deleted";
    #[cfg(unix)]
    const FIRST_COMPACTION_UNTOUCHED_ID: &str = "first-compaction-base-000";

    #[cfg(unix)]
    struct FirstCompactionFixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        store: SegmentRdbStore,
        engine: Arc<Engine>,
        server: TestServer,
        base_name: String,
    }

    #[cfg(unix)]
    fn first_compaction_base_id(index: usize) -> String {
        format!("first-compaction-base-{index:03}")
    }

    #[cfg(unix)]
    fn first_compaction_base_value(index: usize) -> String {
        // Do not use a repeated prefix here. Keyword dictionaries compact common
        // prefixes very well, which could make a visually large base physically
        // smaller than four sparse files. These deterministic high-entropy terms
        // make the physical-size premise below meaningful without randomness.
        first_compaction_entropy_term(index as u64 + 1)
    }

    #[cfg(unix)]
    fn first_compaction_round_value(round: usize) -> String {
        // Every update has the same row count and term length. The largest of the
        // first three physical layers is thus the measured fourth-layer bound.
        first_compaction_entropy_term(10_000 + round as u64)
    }

    #[cfg(unix)]
    fn first_compaction_entropy_term(seed: u64) -> String {
        let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
        let mut value = String::with_capacity(512);
        for _ in 0..32 {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state = state.wrapping_mul(0x2545_f491_4f6c_dd1d);
            value.push_str(&format!("{state:016x}"));
        }
        value
    }

    #[cfg(unix)]
    async fn first_compaction_create_collection(server: &TestServer) {
        server
            .put(&format!("/collections/{FIRST_COMPACTION_COLLECTION}"))
            .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
    }

    #[cfg(unix)]
    async fn first_compaction_index(server: &TestServer, items: Vec<Value>) {
        for chunk in items.chunks(1_000) {
            server
                .post(&format!("/collections/{FIRST_COMPACTION_COLLECTION}/index"))
                .json(&json!({ "items": chunk }))
                .await
                .assert_status_ok();
        }
    }

    #[cfg(unix)]
    async fn first_compaction_fixture() -> FirstCompactionFixture {
        let dir = tempfile::tempdir().expect("first compaction fixture root");
        let root = dir.path().join("segments");
        let store = SegmentRdbStore::new(&root).expect("create first compaction store");
        let engine = Arc::new(Engine::new());
        let server = TestServer::new(router(AppState::open(engine.clone())))
            .expect("first compaction HTTP server");
        first_compaction_create_collection(&server).await;

        let mut items: Vec<_> = (0..FIRST_COMPACTION_BASE_ROWS)
            .map(|index| {
                json!({
                    "external_id": first_compaction_base_id(index),
                    "field": "kw",
                    "value": first_compaction_base_value(index),
                })
            })
            .collect();
        items.extend([
            json!({
                "external_id": FIRST_COMPACTION_HOT_ID,
                "field": "kw",
                "value": "first-hot-base",
            }),
            json!({
                "external_id": FIRST_COMPACTION_DELETED_ID,
                "field": "kw",
                "value": "first-deleted-base",
            }),
        ]);
        first_compaction_index(&server, items).await;
        stage1_restore_legacy_base(&engine, "first compaction base");
        store
            .save(&engine, FIRST_COMPACTION_BASE_SEQUENCE)
            .expect("publish first compaction base");

        FirstCompactionFixture {
            _dir: dir,
            root: root.clone(),
            store,
            engine,
            server,
            base_name: stage1_reuse_current_name(&root),
        }
    }

    #[cfg(unix)]
    async fn first_compaction_apply_round(server: &TestServer, round: usize) {
        first_compaction_index(
            server,
            vec![json!({
                "external_id": FIRST_COMPACTION_HOT_ID,
                "field": "kw",
                "value": first_compaction_round_value(round),
            })],
        )
        .await;
        if round == 1 {
            server
                .delete(&format!(
                "/collections/{FIRST_COMPACTION_COLLECTION}/index/{FIRST_COMPACTION_DELETED_ID}"
            ))
                .await
                .assert_status(axum::http::StatusCode::NO_CONTENT);
        }
    }

    #[cfg(unix)]
    fn first_compaction_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
        let mut refs: Vec<_> =
            stage1_reuse_catalog_collection(manifest, FIRST_COMPACTION_COLLECTION)["segments"]
                .as_array()
                .expect("first compaction catalog segments")
                .iter()
                .filter(|segment| {
                    segment["role"] == json!("field")
                        && segment["field"] == json!("kw")
                        && segment["kind"] == json!(kind)
                })
                .collect();
        refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("keyword ordinal"));
        refs
    }

    #[cfg(unix)]
    fn first_compaction_base_ref<'a>(manifest: &'a Value) -> &'a Value {
        first_compaction_refs(manifest, "base")
            .into_iter()
            .find(|segment| segment["ordinal"] == json!(0))
            .expect("keyword base reference")
    }

    #[cfg(unix)]
    fn first_compaction_delta_sizes(
        generation: &Path,
        manifest: &Value,
        expected_sequences: &[u64],
    ) -> Vec<u64> {
        let deltas = first_compaction_refs(manifest, "delta");
        stage1_assert_delta_sequence_order(
            &deltas,
            expected_sequences,
            "first compaction pre-merge Keyword layers",
        );
        deltas
            .into_iter()
            .map(|delta| {
                std::fs::metadata(
                    generation.join(delta["path"].as_str().expect("delta segment path")),
                )
                .expect("inspect delta segment")
                .len()
                    + std::fs::metadata(stage1_keyword_delta_rows_path(generation, delta))
                        .expect("inspect delta row map")
                        .len()
            })
            .collect()
    }

    #[cfg(unix)]
    fn first_compaction_assert_base_hardlinked(
        base_generation: &Path,
        base_manifest: &Value,
        latest_generation: &Path,
        latest_manifest: &Value,
    ) {
        let original = base_generation.join(
            first_compaction_base_ref(base_manifest)["path"]
                .as_str()
                .expect("base keyword path"),
        );
        let retained = latest_generation.join(
            first_compaction_base_ref(latest_manifest)["path"]
                .as_str()
                .expect("latest keyword base path"),
        );
        let original_metadata =
            std::fs::symlink_metadata(&original).expect("inspect original base");
        let retained_metadata =
            std::fs::symlink_metadata(&retained).expect("inspect retained base");
        assert!(original_metadata.is_file() && !original_metadata.file_type().is_symlink());
        assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
        assert_eq!(
            original_metadata.ino(),
            retained_metadata.ino(),
            "small sparse deltas must merge without rewriting the older keyword base"
        );
        assert!(
            retained_metadata.nlink() >= 2,
            "retained keyword base must be a hard link in the compacted generation"
        );
    }

    #[cfg(unix)]
    async fn first_compaction_search_ids(server: &TestServer, value: &str) -> Vec<String> {
        let response = server
            .post(&format!(
                "/collections/{FIRST_COMPACTION_COLLECTION}/search"
            ))
            .json(&json!({
                "query": { "term": { "field": "kw", "value": value } },
                "limit": 32,
                "track_total": true,
            }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        let mut ids: Vec<_> = body["hits"]
            .as_array()
            .expect("keyword search hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("keyword hit ID")
                    .to_owned()
            })
            .collect();
        ids.sort();
        assert_eq!(
            body["total"].as_u64(),
            Some(ids.len() as u64),
            "bounded keyword total"
        );
        ids
    }

    #[cfg(unix)]
    async fn first_compaction_assert_state(server: &TestServer, phase: &str) {
        assert_eq!(
            first_compaction_search_ids(server, &first_compaction_round_value(4)).await,
            vec![FIRST_COMPACTION_HOT_ID.to_owned()],
            "{phase}: newest delta value remains visible"
        );
        assert!(
            first_compaction_search_ids(server, &first_compaction_round_value(1))
                .await
                .is_empty(),
            "{phase}: compaction must mask an older hot value"
        );
        assert!(
            first_compaction_search_ids(server, "first-deleted-base")
                .await
                .is_empty(),
            "{phase}: compaction must retain the document tombstone"
        );
        assert_eq!(
            first_compaction_search_ids(server, &first_compaction_base_value(0)).await,
            vec![FIRST_COMPACTION_UNTOUCHED_ID.to_owned()],
            "{phase}: compaction retains untouched base data"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn v2_four_keyword_deltas_compact_and_preserve_tombstone_across_live_cold_and_retained_base(
    ) {
        let fixture = first_compaction_fixture().await;
        let base_generation = fixture.root.join(&fixture.base_name);
        let base_manifest = stage1_read_manifest(&base_generation);
        let base_bytes = std::fs::metadata(
            base_generation.join(
                first_compaction_base_ref(&base_manifest)["path"]
                    .as_str()
                    .expect("original base path"),
            ),
        )
        .expect("inspect original keyword base")
        .len();

        for round in 1..=3 {
            first_compaction_apply_round(&fixture.server, round).await;
            fixture
                .store
                .save(
                    &fixture.engine,
                    FIRST_COMPACTION_BASE_SEQUENCE + round as u64,
                )
                .expect("publish sparse keyword checkpoint");
        }
        let third_generation = stage1_current_generation_dir(&fixture.root);
        let third_manifest = stage1_read_manifest(&third_generation);
        let third_sizes = first_compaction_delta_sizes(
            &third_generation,
            &third_manifest,
            &[
                FIRST_COMPACTION_BASE_SEQUENCE + 1,
                FIRST_COMPACTION_BASE_SEQUENCE + 2,
                FIRST_COMPACTION_BASE_SEQUENCE + 3,
            ],
        );
        assert_eq!(
            third_sizes.len(),
            3,
            "the fourth checkpoint alone must request the first compaction"
        );
        let third_bytes: u64 = third_sizes.iter().sum();
        let fourth_upper_bound = *third_sizes
            .iter()
            .max()
            .expect("three physical sparse delta sizes");
        assert!(
        third_bytes
            .checked_add(fourth_upper_bound)
            .expect("bounded fourth sparse layer byte total")
            < base_bytes,
        "fixture precondition: the first three actual sparse layers ({third_bytes}) plus a bounded fourth layer ({fourth_upper_bound}) must stay below the actual base ({base_bytes}), so the fourth-delta request may not merge the base"
    );

        first_compaction_apply_round(&fixture.server, 4).await;
        fixture
            .store
            .save(&fixture.engine, FIRST_COMPACTION_BASE_SEQUENCE + 4)
            .expect("publish fourth sparse keyword checkpoint");
        fixture
            .store
            .wait_for_merges(Duration::from_secs(30))
            .expect("wait for first keyword compaction");

        let latest_generation = stage1_current_generation_dir(&fixture.root);
        let latest_manifest = stage1_read_manifest(&latest_generation);
        let deltas = first_compaction_refs(&latest_manifest, "delta");
        assert!(
        deltas.len() < 4,
        "the fourth keyword delta must request and publish a reduction before the next checkpoint; got {} catalogued layers",
        deltas.len()
    );
        assert!(
            deltas.len() <= 16,
            "a published keyword catalog must stay below the sixteen-delta hard limit"
        );
        first_compaction_assert_base_hardlinked(
            &base_generation,
            &base_manifest,
            &latest_generation,
            &latest_manifest,
        );
        first_compaction_assert_state(&fixture.server, "live compacted checkpoint").await;

        let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
        assert_eq!(
            cold_sequence,
            FIRST_COMPACTION_BASE_SEQUENCE + 4,
            "cold CURRENT carries the fourth checkpoint watermark"
        );
        let cold_server =
            TestServer::new(router(AppState::open(cold_engine))).expect("cold compacted server");
        first_compaction_assert_state(&cold_server, "cold compacted checkpoint").await;

        let (base_engine, base_sequence) =
            stage1_reuse_cold_load_named(&fixture.root, &fixture.base_name);
        assert_eq!(
            base_sequence, FIRST_COMPACTION_BASE_SEQUENCE,
            "retained base generation preserves its original sequence"
        );
        let base_server = TestServer::new(router(AppState::open(base_engine)))
            .expect("retained base cold server");
        assert_eq!(
            first_compaction_search_ids(&base_server, "first-hot-base").await,
            vec![FIRST_COMPACTION_HOT_ID.to_owned()],
            "retained first generation keeps its original hot value"
        );
        assert_eq!(
            first_compaction_search_ids(&base_server, "first-deleted-base").await,
            vec![FIRST_COMPACTION_DELETED_ID.to_owned()],
            "retained first generation keeps data predating the later tombstone"
        );
    }

    use super::*;
}

#[cfg(unix)]
mod full_compaction_contract {
    //! # Facets
    //!
    //! - Behavior: `v2_base_eligible_keyword_compaction_keeps_all_field_results_live_cold_and_retained`
    //!   drives only public collection, index, replace, delete, search, and
    //!   `SegmentRdbStore::save` operations. It proves a measured base-eligibility
    //!   premise, requires a Keyword base replacement, and compares all seven
    //!   field types against independent live Engines and retained generations.
    //!   `v2_partial_keyword_compaction_replaces_the_whole_measured_delta_stack_below_base`
    //!   separately pins the whole-stack policy: below the measured
    //!   base-eligibility threshold, one job folds every measured delta layer
    //!   for the field into a single compacted delta whose ID set unions all
    //!   four inputs, the base stays untouched, and the retained
    //!   pre-compaction generation still holds the replaced layers' bytes.
    //! - Security: `apps/lumen/src/segment_rdb.rs:450-480` writes and validates
    //!   the staged catalog before publication. The existing malformed-CURRENT
    //!   refusals at `apps/lumen/e2e/indexing_durable_oracle.rs:970-1018` cover
    //!   the same persisted catalog trust boundary and assert that CURRENT is not
    //!   changed. This case cold-opens both the compacted and retained bytes; it
    //!   adds no new caller-controlled path or format.
    //! - Performance: the user-approved #4246 plan for this non-AW run requires
    //!   a compaction request at four deltas, a hard limit of sixteen, and a base
    //!   merge only when total delta bytes meet or exceed base bytes. The measured
    //!   threshold, base replacement, hard-link checks, and <=16 assertions carry
    //!   that structural promise. This case does not claim a latency or RSS result.

    use super::*;

    const FULL_COMPACTION_COLLECTION: &str = "full-base-compaction";
    const FULL_COMPACTION_BASE_ROWS: usize = 256;
    const FULL_COMPACTION_UPDATED_ROWS: usize = 64;
    const FULL_COMPACTION_BASE_SEQUENCE: u64 = 9_200;
    const FULL_COMPACTION_HOT_ID: &str = "full-compaction-hot";
    const FULL_COMPACTION_DELETED_ID: &str = "full-compaction-deleted";
    const FULL_COMPACTION_APPENDED_ID: &str = "full-compaction-appended";
    const FULL_COMPACTION_UNTOUCHED_ID: &str = "full-compaction-base-255";

    const FULL_KEYWORD: &str = "kw";
    const FULL_NUMBER: &str = "num";
    const FULL_SET: &str = "tags";
    const FULL_HASH: &str = "sig";
    const FULL_TEXT: &str = "body";
    const FULL_FLAT: &str = "flat";
    const FULL_HNSW: &str = "hnsw";

    const FULL_HOT_BASE_KEYWORD: &str = "full-hot-keyword-base";
    const FULL_HOT_FINAL_KEYWORD: &str = "full-hot-keyword-final";
    const FULL_DELETED_KEYWORD: &str = "full-deleted-keyword";
    const FULL_APPENDED_KEYWORD: &str = "full-appended-keyword";
    const FULL_HOT_BASE_NUMBER: f64 = 90_001.0;
    const FULL_HOT_FINAL_NUMBER: f64 = 90_002.0;
    const FULL_APPENDED_NUMBER: f64 = 90_003.0;
    const FULL_HOT_BASE_TAG: &str = "full-hot-tag-base";
    const FULL_APPENDED_TAG: &str = "full-appended-tag";
    const FULL_HOT_BASE_HASH: &str = "000000000000fa01";
    const FULL_HOT_FINAL_HASH: &str = "000000000000fa02";
    const FULL_APPENDED_HASH: &str = "000000000000fa03";
    const FULL_HOT_BASE_TEXT: &str = "full-text-old full-common";
    const FULL_HOT_FINAL_TEXT: &str = "full-text-final full-common";
    const FULL_APPENDED_TEXT: &str = "full-text-appended full-common";

    struct FullCompactionFixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        store: SegmentRdbStore,
        engine: Arc<Engine>,
        server: TestServer,
        base_name: String,
    }

    fn full_compaction_base_id(index: usize) -> String {
        format!("full-compaction-base-{index:03}")
    }

    fn full_compaction_entropy_term(seed: u64, words: usize) -> String {
        let mut state = seed ^ 0xd1b5_4a32_d192_ed03;
        let mut value = String::with_capacity(words * 16);
        for _ in 0..words {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state = state.wrapping_mul(0x2545_f491_4f6c_dd1d);
            value.push_str(&format!("{state:016x}"));
        }
        value
    }

    fn full_compaction_base_keyword(index: usize) -> String {
        // Values have no shared textual prefix. This keeps the observed base size
        // tied to real distinct keyword bytes instead of dictionary compression.
        full_compaction_entropy_term(index as u64 + 1, 16)
    }

    fn full_compaction_round_keyword(round: usize, index: usize) -> String {
        // Each changed Keyword value is much larger than its base value. The test
        // still proves base eligibility from catalogued physical bytes, never from
        // this requested payload size alone.
        full_compaction_entropy_term(100_000 + (round * 1_000 + index) as u64, 64)
    }

    fn full_compaction_base_number(index: usize) -> f64 {
        1_000.0 + index as f64
    }

    fn full_compaction_base_tag(index: usize) -> String {
        format!("full-base-tag-{index:03}")
    }

    fn full_compaction_base_hash(index: usize) -> String {
        format!("{:016x}", 0x1_0000_u64 + index as u64)
    }

    fn full_compaction_base_text(index: usize) -> String {
        format!("full-common full-base-text-{index:03}")
    }

    fn full_compaction_flat_x(index: usize) -> f32 {
        10_000.0 + index as f32
    }

    fn full_compaction_hnsw_x(index: usize) -> f32 {
        20_000.0 + index as f32
    }

    fn full_compaction_vector(x: f32) -> Value {
        // The HNSW witness uses L2. The query rebuilds this exact vector, so
        // its intended ID has distance zero. One label coordinate prevents a
        // collision, while seven deterministic mixed coordinates avoid the
        // old all-collinear `[x, 0]` shape that permitted an approximate line
        // neighbour to win. This changes no expected ID or score assertion.
        let mut state = (x as u32)
            .wrapping_mul(0x9e37_79b9)
            .wrapping_add(0x7f4a_7c15);
        let mut values = Vec::with_capacity(8);
        values.push(Value::from(x as f64 / 10_000.0));
        values.extend((0..7).map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            Value::from(((state >> 8) & 0xffff) as f64 / 8_192.0 - 4.0)
        }));
        Value::Array(values)
    }

    fn full_compaction_document_items(
        external_id: &str,
        keyword: String,
        number: f64,
        tag: String,
        hash: String,
        body: String,
        flat_x: f32,
        hnsw_x: f32,
    ) -> Vec<Value> {
        vec![
            json!({ "external_id": external_id, "field": FULL_KEYWORD, "value": keyword }),
            json!({ "external_id": external_id, "field": FULL_NUMBER, "value": number }),
            json!({ "external_id": external_id, "field": FULL_SET, "value": [tag] }),
            json!({ "external_id": external_id, "field": FULL_HASH, "value": hash }),
            json!({ "external_id": external_id, "field": FULL_TEXT, "value": body }),
            json!({ "external_id": external_id, "field": FULL_FLAT, "value": full_compaction_vector(flat_x) }),
            json!({ "external_id": external_id, "field": FULL_HNSW, "value": full_compaction_vector(hnsw_x) }),
        ]
    }

    fn full_compaction_base_items() -> Vec<Value> {
        let mut items = Vec::with_capacity((FULL_COMPACTION_BASE_ROWS + 2) * 7);
        for index in 0..FULL_COMPACTION_BASE_ROWS {
            items.extend(full_compaction_document_items(
                &full_compaction_base_id(index),
                full_compaction_base_keyword(index),
                full_compaction_base_number(index),
                full_compaction_base_tag(index),
                full_compaction_base_hash(index),
                full_compaction_base_text(index),
                full_compaction_flat_x(index),
                full_compaction_hnsw_x(index),
            ));
        }
        items.extend(full_compaction_document_items(
            FULL_COMPACTION_HOT_ID,
            FULL_HOT_BASE_KEYWORD.to_owned(),
            FULL_HOT_BASE_NUMBER,
            FULL_HOT_BASE_TAG.to_owned(),
            FULL_HOT_BASE_HASH.to_owned(),
            FULL_HOT_BASE_TEXT.to_owned(),
            1.0,
            2.0,
        ));
        items.extend(full_compaction_document_items(
            FULL_COMPACTION_DELETED_ID,
            FULL_DELETED_KEYWORD.to_owned(),
            80_001.0,
            "full-deleted-tag".to_owned(),
            "000000000000fb01".to_owned(),
            "full-text-deleted full-common".to_owned(),
            3.0,
            4.0,
        ));
        items
    }

    async fn full_compaction_create_collection(server: &TestServer) {
        server
            .put(&format!("/collections/{FULL_COMPACTION_COLLECTION}"))
            .json(&json!({ "fields": {
                FULL_KEYWORD: { "type": "keyword" },
                FULL_NUMBER: { "type": "number" },
                FULL_SET: { "type": "set" },
                FULL_HASH: { "type": "hash" },
                FULL_TEXT: { "type": "text", "analyzer": "whitespace_lower" },
                FULL_FLAT: {
                    "type": "vector", "dim": 8, "metric": "l2", "backend": "flat-cpu"
                },
                FULL_HNSW: {
                    "type": "vector", "dim": 8, "metric": "l2", "backend": "hnsw-cpu"
                },
            }}))
            .await
            .assert_status_ok();
    }

    async fn full_compaction_post_items(server: &TestServer, items: Vec<Value>) {
        for chunk in items.chunks(1_000) {
            server
                .post(&format!("/collections/{FULL_COMPACTION_COLLECTION}/index"))
                .json(&json!({ "items": chunk }))
                .await
                .assert_status_ok();
        }
    }

    async fn full_compaction_fixture() -> FullCompactionFixture {
        let dir = tempfile::tempdir().expect("full compaction fixture root");
        let root = dir.path().join("segments");
        let store = SegmentRdbStore::new(&root).expect("create full compaction store");
        let engine = Arc::new(Engine::new());
        let server = TestServer::new(router(AppState::open(engine.clone())))
            .expect("full compaction HTTP server");
        full_compaction_create_collection(&server).await;
        full_compaction_post_items(&server, full_compaction_base_items()).await;
        stage1_restore_legacy_base(&engine, "full compaction base");
        store
            .save(&engine, FULL_COMPACTION_BASE_SEQUENCE)
            .expect("publish full compaction base");
        FullCompactionFixture {
            _dir: dir,
            root: root.clone(),
            store,
            engine,
            server,
            base_name: stage1_reuse_current_name(&root),
        }
    }

    async fn full_compaction_reference() -> TestServer {
        let engine = Arc::new(Engine::new());
        let server = TestServer::new(router(AppState::open(engine)))
            .expect("full compaction reference server");
        full_compaction_create_collection(&server).await;
        full_compaction_post_items(&server, full_compaction_base_items()).await;
        server
    }

    async fn full_compaction_apply_keyword_round(server: &TestServer, round: usize) {
        let mut items: Vec<_> = (0..FULL_COMPACTION_UPDATED_ROWS)
            .map(|index| {
                json!({
                    "external_id": full_compaction_base_id(index),
                    "field": FULL_KEYWORD,
                    "value": full_compaction_round_keyword(round, index),
                })
            })
            .collect();
        items.push(json!({
            "external_id": FULL_COMPACTION_HOT_ID,
            "field": FULL_KEYWORD,
            "value": full_compaction_round_keyword(round, FULL_COMPACTION_UPDATED_ROWS),
        }));
        full_compaction_post_items(server, items).await;
        if round == 1 {
            server
                .delete(&format!(
                    "/collections/{FULL_COMPACTION_COLLECTION}/index/{FULL_COMPACTION_DELETED_ID}"
                ))
                .await
                .assert_status(axum::http::StatusCode::NO_CONTENT);
        }
    }

    async fn full_compaction_apply_final_all_field_change(server: &TestServer) {
        server
            .put(&format!(
                "/collections/{FULL_COMPACTION_COLLECTION}/docs:replace"
            ))
            .json(&json!({ "docs": [{
                "external_id": FULL_COMPACTION_HOT_ID,
                "fields": {
                    FULL_KEYWORD: FULL_HOT_FINAL_KEYWORD,
                    FULL_NUMBER: FULL_HOT_FINAL_NUMBER,
                    FULL_HASH: FULL_HOT_FINAL_HASH,
                    FULL_TEXT: FULL_HOT_FINAL_TEXT,
                    FULL_FLAT: full_compaction_vector(9_000.0),
                    FULL_HNSW: full_compaction_vector(19_000.0),
                },
            }]}))
            .await
            .assert_status_ok();
        full_compaction_post_items(
            server,
            full_compaction_document_items(
                FULL_COMPACTION_APPENDED_ID,
                FULL_APPENDED_KEYWORD.to_owned(),
                FULL_APPENDED_NUMBER,
                FULL_APPENDED_TAG.to_owned(),
                FULL_APPENDED_HASH.to_owned(),
                FULL_APPENDED_TEXT.to_owned(),
                8_000.0,
                18_000.0,
            ),
        )
        .await;
    }

    fn full_compaction_collection<'a>(manifest: &'a Value) -> &'a Value {
        stage1_reuse_catalog_collection(manifest, FULL_COMPACTION_COLLECTION)
    }

    fn full_compaction_field_refs<'a>(
        manifest: &'a Value,
        field: &str,
        kind: &str,
    ) -> Vec<&'a Value> {
        let mut refs: Vec<_> = full_compaction_collection(manifest)["segments"]
            .as_array()
            .expect("full compaction catalog segments")
            .iter()
            .filter(|segment| {
                segment["role"] == json!("field")
                    && segment["field"] == json!(field)
                    && segment["kind"] == json!(kind)
            })
            .collect();
        refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("field ordinal"));
        refs
    }

    fn full_compaction_base_ref<'a>(manifest: &'a Value, field: &str, role: &str) -> &'a Value {
        full_compaction_collection(manifest)["segments"]
            .as_array()
            .expect("full compaction catalog segments")
            .iter()
            .find(|segment| {
                segment["role"] == json!(role)
                    && segment["field"] == json!(field)
                    && segment["kind"] == json!("base")
                    && segment["ordinal"] == json!(0)
            })
            .unwrap_or_else(|| panic!("full compaction catalog needs {role} base for {field}"))
    }

    fn full_compaction_delta_bytes(generation: &Path, manifest: &Value, field: &str) -> u64 {
        full_compaction_field_refs(manifest, field, "delta")
            .into_iter()
            .map(|segment| {
                let payload = std::fs::metadata(
                    generation.join(segment["path"].as_str().expect("delta segment path")),
                )
                .expect("inspect delta segment")
                .len();
                let local = segment["local_rows"]
                    .as_object()
                    .expect("delta local rows object");
                let rows = std::fs::metadata(
                    generation.join(local["path"].as_str().expect("delta local row path")),
                )
                .expect("inspect delta row map")
                .len();
                payload.checked_add(rows).expect("delta byte sum")
            })
            .sum()
    }

    fn full_compaction_base_bytes(generation: &Path, manifest: &Value, field: &str) -> u64 {
        std::fs::metadata(
            generation.join(
                full_compaction_base_ref(manifest, field, "field")["path"]
                    .as_str()
                    .expect("base segment path"),
            ),
        )
        .expect("inspect base segment")
        .len()
    }

    fn full_compaction_assert_hardlinked_base(
        old_generation: &Path,
        old_manifest: &Value,
        new_generation: &Path,
        new_manifest: &Value,
        field: &str,
        role: &str,
        context: &str,
    ) {
        let old_path = old_generation.join(
            full_compaction_base_ref(old_manifest, field, role)["path"]
                .as_str()
                .expect("old base path"),
        );
        let new_path = new_generation.join(
            full_compaction_base_ref(new_manifest, field, role)["path"]
                .as_str()
                .expect("new base path"),
        );
        let old_metadata = std::fs::symlink_metadata(&old_path).expect("inspect old base");
        let new_metadata = std::fs::symlink_metadata(&new_path).expect("inspect new base");
        assert!(old_metadata.is_file() && !old_metadata.file_type().is_symlink());
        assert!(new_metadata.is_file() && !new_metadata.file_type().is_symlink());
        assert_eq!(
            old_metadata.ino(),
            new_metadata.ino(),
            "{context}: unchanged {field}/{role} base must be a hard link, not a copy"
        );
        assert!(
            new_metadata.nlink() >= 2,
            "{context}: unchanged {field}/{role} base must retain more than one link"
        );
    }

    fn full_compaction_assert_rewritten_base(
        old_generation: &Path,
        old_manifest: &Value,
        new_generation: &Path,
        new_manifest: &Value,
    ) {
        let old_path = old_generation.join(
            full_compaction_base_ref(old_manifest, FULL_KEYWORD, "field")["path"]
                .as_str()
                .expect("old Keyword base path"),
        );
        let new_path = new_generation.join(
            full_compaction_base_ref(new_manifest, FULL_KEYWORD, "field")["path"]
                .as_str()
                .expect("rewritten Keyword base path"),
        );
        let old_metadata = std::fs::symlink_metadata(&old_path).expect("inspect old Keyword base");
        let new_metadata = std::fs::symlink_metadata(&new_path).expect("inspect new Keyword base");
        assert!(old_metadata.is_file() && !old_metadata.file_type().is_symlink());
        assert!(new_metadata.is_file() && !new_metadata.file_type().is_symlink());
        assert_ne!(
        old_metadata.ino(),
        new_metadata.ino(),
        "base-eligible Keyword compaction must publish a new base, not relink its obsolete base"
    );
    }

    async fn full_compaction_search_ids(server: &TestServer, query: Value) -> Vec<String> {
        let response = server
            .post(&format!("/collections/{FULL_COMPACTION_COLLECTION}/search"))
            .json(&json!({ "query": query, "limit": 512, "track_total": true }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        let mut ids: Vec<_> = body["hits"]
            .as_array()
            .expect("bounded scalar search hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("search hit ID")
                    .to_owned()
            })
            .collect();
        ids.sort();
        assert_eq!(
            body["total"].as_u64(),
            Some(ids.len() as u64),
            "bounded scalar fixture reports every matching ID: {body}"
        );
        ids
    }

    fn full_compaction_keyword_query(value: &str) -> Value {
        json!({ "term": { "field": FULL_KEYWORD, "value": value } })
    }

    fn full_compaction_number_query(value: f64) -> Value {
        json!({ "range": {
            "field": FULL_NUMBER,
            "gte": value,
            "lte": value,
        }})
    }

    fn full_compaction_set_query(value: &str) -> Value {
        json!({ "term": { "field": FULL_SET, "value": value } })
    }

    fn full_compaction_hash_query(value: &str) -> Value {
        json!({ "hamming": {
            "field": FULL_HASH,
            "hash": value,
            "max_distance": 0,
        }})
    }

    async fn full_compaction_assert_scalar(
        server: &TestServer,
        reference: &TestServer,
        query: Value,
        mut expected: Vec<String>,
        context: &str,
    ) {
        expected.sort();
        let reference_ids = full_compaction_search_ids(reference, query.clone()).await;
        assert_eq!(
            reference_ids, expected,
            "independent reference has expected IDs for {context}"
        );
        assert_eq!(
            full_compaction_search_ids(server, query).await,
            reference_ids,
            "durable compaction state matches independent public API state for {context}"
        );
    }

    async fn full_compaction_text_search(server: &TestServer, text: &str) -> Value {
        let response = server
            .post(&format!("/collections/{FULL_COMPACTION_COLLECTION}/search"))
            .json(&json!({
                "query": { "match": { "field": FULL_TEXT, "text": text, "op": "and" } },
                "limit": 512,
                "track_total": true,
            }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        let hits = body["hits"].as_array().expect("Text Match hits");
        assert_eq!(
            body["total"].as_u64(),
            Some(hits.len() as u64),
            "bounded Text fixture returns all Match hits: {body}"
        );
        assert!(
            hits.iter().all(|hit| {
                hit["external_id"].as_str().is_some() && hit["score"].as_f64().is_some()
            }),
            "Text Match exposes an ID and serialized BM25 score for every hit: {body}"
        );
        body
    }

    async fn full_compaction_assert_text(
        server: &TestServer,
        reference: &TestServer,
        text: &str,
        mut expected: Vec<String>,
        context: &str,
    ) {
        let reference_body = full_compaction_text_search(reference, text).await;
        let mut reference_ids: Vec<_> = reference_body["hits"]
            .as_array()
            .expect("reference Text hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("reference Text ID")
                    .to_owned()
            })
            .collect();
        reference_ids.sort();
        expected.sort();
        assert_eq!(
            reference_ids, expected,
            "independent Text reference has expected IDs for {context}"
        );
        let durable_body = full_compaction_text_search(server, text).await;
        assert_eq!(
            durable_body["total"], reference_body["total"],
            "durable Text compaction preserves total for {context}"
        );
        assert_eq!(
            durable_body["hits"], reference_body["hits"],
            "durable Text compaction preserves ordered IDs and BM25 scores for {context}"
        );
    }

    fn full_compaction_ids(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    fn full_compaction_current_common_ids() -> Vec<String> {
        (0..FULL_COMPACTION_BASE_ROWS)
            .map(full_compaction_base_id)
            .chain(std::iter::once(FULL_COMPACTION_HOT_ID.to_owned()))
            .chain(std::iter::once(FULL_COMPACTION_APPENDED_ID.to_owned()))
            .collect()
    }

    async fn full_compaction_vector_ids(server: &TestServer, field: &str, x: f32) -> Vec<String> {
        let response = server
            .post(&format!("/collections/{FULL_COMPACTION_COLLECTION}/search"))
            .json(&json!({
                "query": { "knn": { "field": field, "vector": full_compaction_vector(x), "k": 1 } },
                "limit": 1,
                "track_total": true,
            }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        body["hits"]
            .as_array()
            .expect("vector kNN hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("vector hit ID")
                    .to_owned()
            })
            .collect()
    }

    async fn full_compaction_assert_vector(
        server: &TestServer,
        reference: &TestServer,
        field: &str,
        x: f32,
        expected: &str,
        context: &str,
    ) {
        let reference_ids = full_compaction_vector_ids(reference, field, x).await;
        assert_eq!(
            reference_ids,
            vec![expected.to_owned()],
            "independent {field} reference has expected nearest ID for {context}"
        );
        assert_eq!(
        full_compaction_vector_ids(server, field, x).await,
        reference_ids,
        "durable {field} compaction preserves nearest ID for {context} without claiming a backend score"
    );
    }

    async fn full_compaction_assert_current_state(
        server: &TestServer,
        reference: &TestServer,
        phase: &str,
    ) {
        let untouched_index = FULL_COMPACTION_BASE_ROWS - 1;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
            vec![full_compaction_base_id(0)],
            &format!("{phase} compacted Keyword update"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(&full_compaction_round_keyword(1, 0)),
            vec![],
            &format!("{phase} masks an older Keyword delta"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_HOT_FINAL_KEYWORD),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} final Keyword replacement"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_HOT_BASE_KEYWORD),
            vec![],
            &format!("{phase} masks old Keyword field"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_DELETED_KEYWORD),
            vec![],
            &format!("{phase} keeps document tombstone"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_APPENDED_KEYWORD),
            full_compaction_ids(&[FULL_COMPACTION_APPENDED_ID]),
            &format!("{phase} appended Keyword"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(&full_compaction_base_keyword(untouched_index)),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Keyword"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_number_query(FULL_HOT_FINAL_NUMBER),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} final Number"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_number_query(FULL_HOT_BASE_NUMBER),
            vec![],
            &format!("{phase} masks old Number"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_number_query(full_compaction_base_number(untouched_index)),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Number"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_set_query(FULL_HOT_BASE_TAG),
            vec![],
            &format!("{phase} omits old Set field"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_set_query(FULL_APPENDED_TAG),
            full_compaction_ids(&[FULL_COMPACTION_APPENDED_ID]),
            &format!("{phase} appended Set field"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_set_query(&full_compaction_base_tag(untouched_index)),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Set field"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_hash_query(FULL_HOT_FINAL_HASH),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} final Hash"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_hash_query(FULL_HOT_BASE_HASH),
            vec![],
            &format!("{phase} masks old Hash"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_hash_query(&full_compaction_base_hash(untouched_index)),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Hash"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            FULL_HOT_FINAL_TEXT,
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} final Text"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            FULL_HOT_BASE_TEXT,
            vec![],
            &format!("{phase} masks old Text"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            &format!("full-base-text-{untouched_index:03}"),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Text"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            "full-common",
            full_compaction_current_common_ids(),
            &format!("{phase} cannot use fixed expected common corpus IDs"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_FLAT,
            9_000.0,
            FULL_COMPACTION_HOT_ID,
            &format!("{phase} Flat final vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_HNSW,
            19_000.0,
            FULL_COMPACTION_HOT_ID,
            &format!("{phase} HNSW final vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_FLAT,
            8_000.0,
            FULL_COMPACTION_APPENDED_ID,
            &format!("{phase} Flat appended vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_HNSW,
            18_000.0,
            FULL_COMPACTION_APPENDED_ID,
            &format!("{phase} HNSW appended vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_FLAT,
            full_compaction_flat_x(untouched_index),
            FULL_COMPACTION_UNTOUCHED_ID,
            &format!("{phase} untouched Flat vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_HNSW,
            full_compaction_hnsw_x(untouched_index),
            FULL_COMPACTION_UNTOUCHED_ID,
            &format!("{phase} untouched HNSW vector"),
        )
        .await;
    }

    async fn full_compaction_assert_retained_base(
        server: &TestServer,
        reference: &TestServer,
        phase: &str,
    ) {
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_HOT_BASE_KEYWORD),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Keyword"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_number_query(FULL_HOT_BASE_NUMBER),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Number"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_set_query(FULL_HOT_BASE_TAG),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Set"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_hash_query(FULL_HOT_BASE_HASH),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Hash"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            FULL_HOT_BASE_TEXT,
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Text"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_FLAT,
            1.0,
            FULL_COMPACTION_HOT_ID,
            &format!("{phase} Flat"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_HNSW,
            2.0,
            FULL_COMPACTION_HOT_ID,
            &format!("{phase} HNSW"),
        )
        .await;
    }

    async fn full_compaction_assert_third_retained(server: &TestServer) {
        assert_eq!(
            full_compaction_search_ids(
                server,
                full_compaction_keyword_query(&full_compaction_round_keyword(3, 0)),
            )
            .await,
            vec![full_compaction_base_id(0)],
            "retained pre-merge generation keeps its third Keyword update"
        );
        assert!(
            full_compaction_search_ids(
                server,
                full_compaction_keyword_query(&full_compaction_round_keyword(1, 0)),
            )
            .await
            .is_empty(),
            "retained pre-merge generation masks its older Keyword layer"
        );
        assert!(
            full_compaction_search_ids(server, full_compaction_keyword_query(FULL_DELETED_KEYWORD))
                .await
                .is_empty(),
            "retained pre-merge generation keeps its tombstone"
        );
    }

    #[tokio::test]
    async fn v2_base_eligible_keyword_compaction_keeps_all_field_results_live_cold_and_retained() {
        let fixture = full_compaction_fixture().await;
        let baseline_reference = full_compaction_reference().await;
        let reference = full_compaction_reference().await;
        let base_generation = fixture.root.join(&fixture.base_name);
        let base_manifest = stage1_read_manifest(&base_generation);

        for round in 1..=3 {
            full_compaction_apply_keyword_round(&fixture.server, round).await;
            full_compaction_apply_keyword_round(&reference, round).await;
            fixture
                .store
                .save(
                    &fixture.engine,
                    FULL_COMPACTION_BASE_SEQUENCE + round as u64,
                )
                .expect("publish Keyword delta before base eligibility");
        }
        let third_name = stage1_reuse_current_name(&fixture.root);
        let third_generation = fixture.root.join(&third_name);
        let third_manifest = stage1_read_manifest(&third_generation);
        let third_keyword_deltas =
            full_compaction_field_refs(&third_manifest, FULL_KEYWORD, "delta");
        assert_eq!(
            third_keyword_deltas.len(),
            3,
            "the fourth save alone must request compaction for the uniquely deepest Keyword field"
        );
        stage1_assert_delta_sequence_order(
            &third_keyword_deltas,
            &[
                FULL_COMPACTION_BASE_SEQUENCE + 1,
                FULL_COMPACTION_BASE_SEQUENCE + 2,
                FULL_COMPACTION_BASE_SEQUENCE + 3,
            ],
            "full base-eligible Keyword pre-merge layers",
        );
        let base_bytes = full_compaction_base_bytes(&base_generation, &base_manifest, FULL_KEYWORD);
        let third_delta_bytes =
            full_compaction_delta_bytes(&third_generation, &third_manifest, FULL_KEYWORD);
        assert!(
        third_delta_bytes >= base_bytes,
        "fixture precondition: three actual Keyword delta-plus-rowmap bytes ({third_delta_bytes}) must meet or exceed the actual base bytes ({base_bytes}) before the fourth save requests base compaction"
    );

        full_compaction_apply_keyword_round(&fixture.server, 4).await;
        full_compaction_apply_keyword_round(&reference, 4).await;
        fixture
            .store
            .save(&fixture.engine, FULL_COMPACTION_BASE_SEQUENCE + 4)
            .expect("publish base-eligible Keyword checkpoint");
        fixture
            .store
            .wait_for_merges(Duration::from_secs(30))
            .expect("wait for base-eligible Keyword compaction");
        let compacted_name = stage1_reuse_current_name(&fixture.root);
        let compacted_generation = fixture.root.join(&compacted_name);
        let compacted_manifest = stage1_read_manifest(&compacted_generation);
        assert!(
        full_compaction_field_refs(&compacted_manifest, FULL_KEYWORD, "delta").is_empty(),
        "when actual Keyword delta bytes meet the base before the fourth request, publication must replace the base and consume every captured Keyword delta"
    );
        full_compaction_assert_rewritten_base(
            &base_generation,
            &base_manifest,
            &compacted_generation,
            &compacted_manifest,
        );
        for field in [
            FULL_NUMBER,
            FULL_SET,
            FULL_HASH,
            FULL_TEXT,
            FULL_FLAT,
            FULL_HNSW,
        ] {
            assert_eq!(
            full_compaction_field_refs(&compacted_manifest, field, "delta").len(),
            1,
            "only the Keyword field has four deltas, so {field} keeps its one sparse tombstone layer"
        );
            full_compaction_assert_hardlinked_base(
                &base_generation,
                &base_manifest,
                &compacted_generation,
                &compacted_manifest,
                field,
                "field",
                "base-eligible Keyword compaction",
            );
        }
        for field in [FULL_FLAT, FULL_HNSW] {
            full_compaction_assert_hardlinked_base(
                &base_generation,
                &base_manifest,
                &compacted_generation,
                &compacted_manifest,
                field,
                "vector_eids",
                "base-eligible Keyword compaction",
            );
        }

        full_compaction_apply_final_all_field_change(&fixture.server).await;
        full_compaction_apply_final_all_field_change(&reference).await;
        fixture
            .store
            .save(&fixture.engine, FULL_COMPACTION_BASE_SEQUENCE + 5)
            .expect("publish all-field checkpoint after Keyword base compaction");
        fixture
            .store
            .wait_for_merges(Duration::from_secs(30))
            .expect("wait for post-compaction all-field merges");
        let latest_generation = stage1_current_generation_dir(&fixture.root);
        let latest_manifest = stage1_read_manifest(&latest_generation);
        for field in [
            FULL_KEYWORD,
            FULL_NUMBER,
            FULL_SET,
            FULL_HASH,
            FULL_TEXT,
            FULL_FLAT,
            FULL_HNSW,
        ] {
            assert!(
                full_compaction_field_refs(&latest_manifest, field, "delta").len() <= 16,
                "published {field} catalog must stay at or below the sixteen-delta hard limit"
            );
        }
        full_compaction_assert_hardlinked_base(
            &compacted_generation,
            &compacted_manifest,
            &latest_generation,
            &latest_manifest,
            FULL_KEYWORD,
            "field",
            "post-compaction all-field checkpoint",
        );
        for field in [
            FULL_NUMBER,
            FULL_SET,
            FULL_HASH,
            FULL_TEXT,
            FULL_FLAT,
            FULL_HNSW,
        ] {
            full_compaction_assert_hardlinked_base(
                &base_generation,
                &base_manifest,
                &latest_generation,
                &latest_manifest,
                field,
                "field",
                "post-compaction all-field checkpoint",
            );
        }
        for field in [FULL_FLAT, FULL_HNSW] {
            full_compaction_assert_hardlinked_base(
                &base_generation,
                &base_manifest,
                &latest_generation,
                &latest_manifest,
                field,
                "vector_eids",
                "post-compaction all-field checkpoint",
            );
        }
        full_compaction_assert_current_state(
            &fixture.server,
            &reference,
            "live current generation",
        )
        .await;

        let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
        assert_eq!(
            cold_sequence,
            FULL_COMPACTION_BASE_SEQUENCE + 5,
            "cold CURRENT uses the all-field checkpoint watermark"
        );
        let cold_server = TestServer::new(router(AppState::open(cold_engine)))
            .expect("cold full compaction server");
        full_compaction_assert_current_state(&cold_server, &reference, "cold current generation")
            .await;

        let (third_engine, third_sequence) =
            stage1_reuse_cold_load_named(&fixture.root, &third_name);
        assert_eq!(
            third_sequence,
            FULL_COMPACTION_BASE_SEQUENCE + 3,
            "retained pre-merge generation preserves its original watermark"
        );
        let third_server =
            TestServer::new(router(AppState::open(third_engine))).expect("retained third server");
        full_compaction_assert_third_retained(&third_server).await;

        let (base_engine, base_sequence) =
            stage1_reuse_cold_load_named(&fixture.root, &fixture.base_name);
        assert_eq!(
            base_sequence, FULL_COMPACTION_BASE_SEQUENCE,
            "retained base generation preserves its original watermark"
        );
        let base_server =
            TestServer::new(router(AppState::open(base_engine))).expect("retained base server");
        full_compaction_assert_retained_base(
            &base_server,
            &baseline_reference,
            "retained base generation",
        )
        .await;
    }

    const FULL_PAIR_WITNESS_COLLECTION: &str = "full-partial-pair-witness";
    const FULL_PAIR_BASE_SEQUENCE: u64 = 9_600;

    #[derive(Clone, Debug)]
    struct FullPairDelta {
        ordinal: u64,
        path: String,
        bytes: u64,
        ids: std::collections::BTreeSet<String>,
        inode: u64,
    }


    fn full_pair_value(round: usize) -> String {
        // The values deliberately vary in physical size. The selection oracle
        // nevertheless derives the legal pair from actual written file sizes.
        let words = match round {
            1 => 2,
            2 => 8,
            3 => 32,
            4 => 4,
            _ => panic!("pair policy needs one of four rounds"),
        };
        full_compaction_entropy_term(700_000 + round as u64, words)
    }

    fn full_pair_collection<'a>(manifest: &'a Value, collection: &str) -> &'a Value {
        manifest["collections"]
            .as_array()
            .expect("pair policy catalog collections")
            .iter()
            .find(|candidate| candidate["collection_id"] == json!(collection))
            .unwrap_or_else(|| panic!("pair policy catalog needs {collection}"))
    }

    fn full_pair_refs<'a>(manifest: &'a Value, collection: &str, kind: &str) -> Vec<&'a Value> {
        let mut refs: Vec<_> = full_pair_collection(manifest, collection)["segments"]
            .as_array()
            .expect("pair policy catalog segments")
            .iter()
            .filter(|segment| {
                segment["role"] == json!("field")
                    && segment["field"] == json!(FULL_KEYWORD)
                    && segment["kind"] == json!(kind)
            })
            .collect();
        refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("pair delta ordinal"));
        refs
    }

    fn full_pair_ref_bytes(generation: &Path, reference: &Value) -> u64 {
        let payload = std::fs::metadata(
            generation.join(
                reference["path"]
                    .as_str()
                    .expect("pair segment reference path"),
            ),
        )
        .expect("inspect pair segment")
        .len();
        let rows = reference["local_rows"]
            .as_object()
            .expect("pair local row reference");
        let rows =
            std::fs::metadata(generation.join(rows["path"].as_str().expect("pair local row path")))
                .expect("inspect pair row map")
                .len();
        payload.checked_add(rows).expect("pair reference byte sum")
    }

    fn full_pair_snapshot(generation: &Path, references: Vec<&Value>) -> Vec<FullPairDelta> {
        references
            .into_iter()
            .map(|reference| {
                let path = reference["path"]
                    .as_str()
                    .expect("pair segment path")
                    .to_owned();
                let ids = stage1_keyword_delta_read_rows(generation, reference)
                    .into_iter()
                    .collect();
                let inode = std::fs::symlink_metadata(generation.join(&path))
                    .expect("inspect pair segment inode")
                    .ino();
                FullPairDelta {
                    ordinal: reference["ordinal"].as_u64().expect("pair segment ordinal"),
                    bytes: full_pair_ref_bytes(generation, reference),
                    path,
                    ids,
                    inode,
                }
            })
            .collect()
    }

    async fn full_pair_add_witness_collection(fixture: &FullCompactionFixture) -> String {
        fixture
            .server
            .put(&format!("/collections/{FULL_PAIR_WITNESS_COLLECTION}"))
            .json(&json!({ "fields": { FULL_KEYWORD: { "type": "keyword" } } }))
            .await
            .assert_status_ok();
        fixture
            .server
            .post(&format!(
                "/collections/{FULL_PAIR_WITNESS_COLLECTION}/index"
            ))
            .json(&json!({ "items": [{
                "external_id": full_compaction_base_id(3),
                "field": FULL_KEYWORD,
                "value": "pair-witness-base",
            }] }))
            .await
            .assert_status_ok();
        fixture
            .store
            .save(&fixture.engine, FULL_PAIR_BASE_SEQUENCE)
            .expect("publish pair-policy base generation");
        stage1_reuse_current_name(&fixture.root)
    }

    async fn full_pair_update_target(server: &TestServer, round: usize) {
        full_compaction_post_items(
            server,
            vec![json!({
                "external_id": full_compaction_base_id(round - 1),
                "field": FULL_KEYWORD,
                "value": full_pair_value(round),
            })],
        )
        .await;
    }

    async fn full_pair_update_witness(server: &TestServer) {
        server
            .post(&format!(
                "/collections/{FULL_PAIR_WITNESS_COLLECTION}/index"
            ))
            .json(&json!({ "items": [{
                "external_id": full_compaction_base_id(3),
                "field": FULL_KEYWORD,
                "value": full_pair_value(4),
            }] }))
            .await
            .assert_status_ok();
    }

    async fn full_pair_assert_query_state(server: &TestServer, phase: &str) {
        for round in 1..=4 {
            assert_eq!(
                full_compaction_search_ids(
                    server,
                    full_compaction_keyword_query(&full_pair_value(round))
                )
                .await,
                vec![full_compaction_base_id(round - 1)],
                "{phase}: compacted pair keeps round-{round} Keyword value"
            );
        }
        assert_eq!(
            full_compaction_search_ids(
                server,
                full_compaction_keyword_query(&full_compaction_base_keyword(
                    FULL_COMPACTION_BASE_ROWS - 1
                )),
            )
            .await,
            vec![FULL_COMPACTION_UNTOUCHED_ID.to_owned()],
            "{phase}: compacted pair keeps untouched base Keyword value"
        );
    }

    #[tokio::test]
    async fn v2_partial_keyword_compaction_replaces_the_whole_measured_delta_stack_below_base() {
        let fixture = full_compaction_fixture().await;
        let pair_base_name = full_pair_add_witness_collection(&fixture).await;
        let pair_base_generation = fixture.root.join(&pair_base_name);
        let pair_base_manifest = stage1_read_manifest(&pair_base_generation);
        let base_bytes =
            full_compaction_base_bytes(&pair_base_generation, &pair_base_manifest, FULL_KEYWORD);

        for round in 1..=3 {
            full_pair_update_target(&fixture.server, round).await;
            fixture
                .store
                .save(&fixture.engine, FULL_PAIR_BASE_SEQUENCE + round as u64)
                .expect("publish pair-policy target delta");
        }
        let before_generation = stage1_current_generation_dir(&fixture.root);
        let before_manifest = stage1_read_manifest(&before_generation);
        let before_references =
            full_pair_refs(&before_manifest, FULL_COMPACTION_COLLECTION, "delta");
        assert_eq!(
            before_references.len(),
            3,
            "the fourth pair-policy save alone must request compaction"
        );
        stage1_assert_delta_sequence_order(
            &before_references,
            &[
                FULL_PAIR_BASE_SEQUENCE + 1,
                FULL_PAIR_BASE_SEQUENCE + 2,
                FULL_PAIR_BASE_SEQUENCE + 3,
            ],
            "pre-merge Keyword delta layers",
        );
        let before = full_pair_snapshot(&before_generation, before_references);
        assert_eq!(
            before.len(),
            3,
            "the fourth pair-policy save alone must request compaction"
        );
        assert!(
            before.iter().all(|layer| layer.ids.len() == 1),
            "each measured pre-compaction policy layer must name one distinct ID"
        );
        let distinct_before_ids: std::collections::BTreeSet<_> = before
            .iter()
            .flat_map(|layer| layer.ids.iter().cloned())
            .collect();
        assert_eq!(
            distinct_before_ids.len(),
            before.len(),
            "the measured pre-compaction policy layers must name distinct IDs"
        );

        full_pair_update_target(&fixture.server, 4).await;
        full_pair_update_witness(&fixture.server).await;
        fixture
            .store
            .save(&fixture.engine, FULL_PAIR_BASE_SEQUENCE + 4)
            .expect("publish fourth pair-policy target delta");
        fixture
            .store
            .wait_for_merges(Duration::from_secs(30))
            .expect("wait for whole-stack delta compaction");
        let latest_generation = stage1_current_generation_dir(&fixture.root);
        let latest_manifest = stage1_read_manifest(&latest_generation);
        let witness_references =
            full_pair_refs(&latest_manifest, FULL_PAIR_WITNESS_COLLECTION, "delta");
        assert_eq!(
            witness_references.len(),
            2,
            "a fresh witness retains its initial sparse layer and the same-checkpoint raw delta"
        );
        stage1_assert_delta_sequence_order(
            &witness_references,
            &[FULL_PAIR_BASE_SEQUENCE, FULL_PAIR_BASE_SEQUENCE + 4],
            "fresh witness layers before the fourth-round whole-stack merge",
        );
        let fourth_witness_reference = witness_references
            .iter()
            .copied()
            .find(|reference| reference["applied_seq"] == json!(FULL_PAIR_BASE_SEQUENCE + 4))
            .expect("same-checkpoint witness delta");
        let witness = full_pair_snapshot(&latest_generation, vec![fourth_witness_reference]);
        assert_eq!(
            witness.len(),
            1,
            "selected same-checkpoint witness delta must have one measured layer"
        );
        assert_eq!(
            witness[0].ids,
            std::collections::BTreeSet::from([full_compaction_base_id(3)]),
            "same-checkpoint witness uses the target fourth ID and local-row shape"
        );
        let before_plus_fourth = before
            .iter()
            .map(|layer| layer.bytes)
            .sum::<u64>()
            .checked_add(witness[0].bytes)
            .expect("pair-policy total bytes");
        assert!(
            before_plus_fourth < base_bytes,
            "fixture precondition: the three measured target layers plus the same-sequence fourth witness ({before_plus_fourth}) stay below the measured base ({base_bytes}), so policy must fold the delta stack instead of merging the base"
        );

        let latest = full_pair_snapshot(
            &latest_generation,
            full_pair_refs(&latest_manifest, FULL_COMPACTION_COLLECTION, "delta"),
        );
        assert_eq!(
            latest.len(),
            1,
            "below the measured base threshold, one job must fold the whole four-layer delta stack into a single compacted delta"
        );
        let compacted = &latest[0];
        let fourth_ordinal = before
            .last()
            .expect("three pre-compaction target layers")
            .ordinal
            .checked_add(1)
            .expect("fourth delta ordinal");
        assert_eq!(
            compacted.ordinal,
            fourth_ordinal,
            "the compacted output must carry the newest (fourth) input's ordinal, matching write_compacted_field's non-base output path"
        );
        let mut expected_ids: std::collections::BTreeSet<String> = before
            .iter()
            .flat_map(|layer| layer.ids.iter().cloned())
            .collect();
        expected_ids.insert(full_compaction_base_id(3));
        assert_eq!(
            compacted.ids,
            expected_ids,
            "the compacted delta's ID set must union all four measured inputs, not one adjacent pair"
        );

        let latest_delta_paths: std::collections::BTreeSet<&str> =
            latest.iter().map(|layer| layer.path.as_str()).collect();
        for old in &before {
            assert!(
                !latest_delta_paths.contains(old.path.as_str()),
                "superseded input at ordinal {} must not remain catalogued once the whole stack is compacted",
                old.ordinal
            );
            let retained_metadata = std::fs::symlink_metadata(before_generation.join(&old.path))
                .unwrap_or_else(|error| {
                    panic!(
                        "superseded input at ordinal {} must still be present, unpruned, in its retained pre-compaction generation: {error}",
                        old.ordinal
                    )
                });
            assert_eq!(
                retained_metadata.ino(),
                old.inode,
                "the retained pre-compaction generation must keep the original, unmutated file for ordinal {}",
                old.ordinal
            );
        }
        full_compaction_assert_hardlinked_base(
            &pair_base_generation,
            &pair_base_manifest,
            &latest_generation,
            &latest_manifest,
            FULL_KEYWORD,
            "field",
            "whole-stack delta compaction",
        );
        full_pair_assert_query_state(&fixture.server, "live whole-stack compaction generation")
            .await;

        let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
        assert_eq!(
            cold_sequence,
            FULL_PAIR_BASE_SEQUENCE + 4,
            "cold whole-stack compaction generation preserves the fourth watermark"
        );
        let cold_server =
            TestServer::new(router(AppState::open(cold_engine))).expect("cold pair policy server");
        full_pair_assert_query_state(&cold_server, "cold whole-stack compaction generation").await;
    }

    mod paused_merge_checkpoint_contract {
        //! # Facets
        //!
        //! - Behavior: `indexing_durable_oracle.rs:7713` requires the other checkpoint
        //!   while encoding is paused; `:7728` and `:7771` retain the fifth Keyword layer.
        //! - Security: `apps/lumen/src/segment_rdb.rs:66-73` is a test-only observer seam.
        //!   `indexing_durable_oracle.rs:7740` and `:7760` require CURRENT and cold reopen
        //!   to retain the later collection reference, without a new input boundary.
        //! - Performance: `indexing_durable_oracle.rs:7713` measures the approved bounded
        //!   checkpoint-progress path with two seconds. It does not claim stage6 latency,
        //!   RSS, or throughput acceptance.

        use super::*;

        const OTHER_COLLECTION: &str = "merge-pause-other";
        const OTHER_FIELD: &str = "kw";
        const OTHER_ID: &str = "merge-pause-other-id";
        const OTHER_BASE_VALUE: &str = "merge-pause-other-base";
        const OTHER_NEW_VALUE: &str = "merge-pause-other-new";

        struct PauseBeforeEncode {
            reached: std::sync::mpsc::SyncSender<()>,
            published: std::sync::mpsc::SyncSender<()>,
            release: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
            paused: std::sync::atomic::AtomicBool,
        }

        impl lumen::segment_rdb::MergeObserver for PauseBeforeEncode {
            fn observe(&self, phase: lumen::segment_rdb::MergePhase) -> std::io::Result<()> {
                if phase == lumen::segment_rdb::MergePhase::AfterPublish {
                    self.published.send(()).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "merge publication observer receiver dropped",
                        )
                    })?;
                }
                if phase == lumen::segment_rdb::MergePhase::BeforeEncode
                    && !self.paused.swap(true, std::sync::atomic::Ordering::SeqCst)
                {
                    self.reached.send(()).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "merge pause observer receiver dropped",
                        )
                    })?;
                    self.release
                        .lock()
                        .expect("merge pause release mutex")
                        .take()
                        .expect("merge pause releases exactly once")
                        .recv()
                        .map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::BrokenPipe,
                                "merge pause release sender dropped",
                            )
                        })?;
                }
                Ok(())
            }
        }

        async fn other_term_ids(server: &TestServer, value: &str) -> Vec<String> {
            let response = server
                .post(&format!("/collections/{OTHER_COLLECTION}/search"))
                .json(&json!({
                    "query": { "term": { "field": OTHER_FIELD, "value": value } },
                    "limit": 16,
                    "track_total": true,
                }))
                .await;
            response.assert_status_ok();
            let mut ids = response.json::<Value>()["hits"]
                .as_array()
                .expect("other collection term hits")
                .iter()
                .map(|hit| {
                    hit["external_id"]
                        .as_str()
                        .expect("other collection external ID")
                        .to_owned()
                })
                .collect::<Vec<_>>();
            ids.sort();
            ids
        }

        async fn create_other_collection(server: &TestServer) {
            server
                .put(&format!("/collections/{OTHER_COLLECTION}"))
                .json(&json!({ "fields": { OTHER_FIELD: { "type": "keyword" } } }))
                .await
                .assert_status_ok();
            server
                .post(&format!("/collections/{OTHER_COLLECTION}/index"))
                .json(&json!({ "items": [{
                    "external_id": OTHER_ID,
                    "field": OTHER_FIELD,
                    "value": OTHER_BASE_VALUE,
                }] }))
                .await
                .assert_status_ok();
        }

        async fn update_other_collection(server: &TestServer) {
            server
                .post(&format!("/collections/{OTHER_COLLECTION}/index"))
                .json(&json!({ "items": [{
                    "external_id": OTHER_ID,
                    "field": OTHER_FIELD,
                    "value": OTHER_NEW_VALUE,
                }] }))
                .await
                .assert_status_ok();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn paused_merge_does_not_block_another_collection_checkpoint() {
            let fixture = full_compaction_fixture().await;
            let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
            let (published_tx, published_rx) = std::sync::mpsc::sync_channel(1);
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let observer = Arc::new(PauseBeforeEncode {
                reached: reached_tx,
                published: published_tx,
                release: Mutex::new(Some(release_rx)),
                paused: std::sync::atomic::AtomicBool::new(false),
            });
            let store = Arc::new(
                SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                    .expect("open observed full-compaction store"),
            );
            let baseline_sequence = FULL_COMPACTION_BASE_SEQUENCE + 1;
            create_other_collection(&fixture.server).await;
            store
                .save(&fixture.engine, baseline_sequence)
                .expect("publish other collection baseline");

            for round in 1..=3 {
                full_compaction_apply_keyword_round(&fixture.server, round).await;
                store
                    .save(&fixture.engine, baseline_sequence + round as u64)
                    .expect("publish measured pre-merge delta");
            }
            full_compaction_apply_keyword_round(&fixture.server, 4).await;

            let merge_sequence = baseline_sequence + 4;
            let paused_merge = {
                let store = store.clone();
                let engine = fixture.engine.clone();
                tokio::task::spawn_blocking(move || store.save(&engine, merge_sequence))
            };
            let reached = tokio::task::spawn_blocking(move || {
                reached_rx.recv_timeout(Duration::from_secs(30))
            })
            .await
            .expect("merge-pause receiver task must not panic");

            let mut later_checkpoint = if reached.is_ok() {
                full_compaction_apply_keyword_round(&fixture.server, 5).await;
                update_other_collection(&fixture.server).await;
                let store = store.clone();
                let engine = fixture.engine.clone();
                Some(tokio::task::spawn_blocking(move || {
                    store.save(&engine, merge_sequence + 1)
                }))
            } else {
                None
            };
            let completed_while_merge_paused = match later_checkpoint.as_mut() {
                Some(save) => Some(tokio::time::timeout(Duration::from_secs(2), save).await),
                None => None,
            };
            let checkpoint_finished_while_paused =
                matches!(&completed_while_merge_paused, Some(Ok(Ok(Ok(())))));

            // Send before every join. Thus an assertion below cannot leave the
            // real encoding callback or its checkpoint thread blocked.
            let _ = release_tx.send(());
            let after_publish = if reached.is_ok() {
                Some(
                    tokio::task::spawn_blocking(move || {
                        published_rx.recv_timeout(Duration::from_secs(30))
                    })
                    .await
                    .expect("merge-publication receiver task must not panic"),
                )
            } else {
                None
            };
            let merge_result = paused_merge.await;
            let later_finished_after_release = match &completed_while_merge_paused {
                Some(Ok(result)) => {
                    drop(later_checkpoint.take());
                    Some(matches!(result, Ok(Ok(()))))
                }
                Some(Err(_)) => Some(
                    later_checkpoint
                        .take()
                        .expect("timed-out checkpoint handle")
                        .await
                        .is_ok_and(|result| result.is_ok()),
                ),
                None => None,
            };

            assert!(
                reached.is_ok(),
                "a fourth real delta checkpoint must select a merge candidate before encoding: {reached:?}"
            );
            assert!(
                matches!(&after_publish, Some(Ok(()))),
                "a selected merge must report durable publication after release: {after_publish:?}"
            );
            assert!(
                matches!(&merge_result, Ok(Ok(()))),
                "paused merge must finish cleanly after release: {merge_result:?}"
            );
            assert!(
                checkpoint_finished_while_paused && later_finished_after_release == Some(true),
                "a real checkpoint for another collection must finish while merge encoding is paused: {completed_while_merge_paused:?}; after release joined successfully: {later_finished_after_release:?}"
            );

            assert_eq!(
                other_term_ids(&fixture.server, OTHER_NEW_VALUE).await,
                vec![OTHER_ID.to_owned()],
                "live state must retain the checkpoint published during the paused merge"
            );
            assert_eq!(
                other_term_ids(&fixture.server, OTHER_BASE_VALUE).await,
                Vec::<String>::new(),
                "live state must not resurrect the other collection base value"
            );
            assert_eq!(
                full_compaction_search_ids(
                    &fixture.server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(5, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "the later merge must retain the fifth Keyword layer appended during its pause"
            );

            let current_dir = stage1_current_generation_dir(&fixture.root);
            let current_manifest = stage1_read_manifest(&current_dir);
            assert_eq!(
                current_manifest["checkpoint_sequence"],
                json!(merge_sequence + 1),
                "CURRENT must retain the newer checkpoint watermark after the paused merge finishes"
            );
            assert!(
                current_manifest["collections"]
                    .as_array()
                    .expect("current collection catalog")
                    .iter()
                    .any(|collection| collection["collection_id"] == json!(OTHER_COLLECTION)),
                "CURRENT must retain the newer collection catalog reference"
            );

            let cold = SegmentRdbStore::new(&fixture.root)
                .expect("reopen paused-merge root")
                .load_current_generation()
                .expect("load paused-merge CURRENT")
                .expect("paused-merge CURRENT generation");
            assert_eq!(
                cold.sequence,
                merge_sequence + 1,
                "cold reopen must select the newer checkpoint rather than the paused merge cut"
            );
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("cold paused-merge server");
            assert_eq!(
                other_term_ids(&cold_server, OTHER_NEW_VALUE).await,
                vec![OTHER_ID.to_owned()],
                "cold CURRENT must retain the other collection change published during merge pause"
            );
            assert_eq!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(5, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "cold CURRENT must retain the fifth Keyword layer appended during merge pause"
            );
        }
    }
    mod background_merge_cap_restore_prune_contract {
        //! # Facets
        //!
        //! - Behavior: `cap_sixteen_deltas_waits_without_publishing_seventeen_and_keeps_other_work_live`
        //!   requires a seventeenth dirty field checkpoint to remain uncommitted while its
        //!   selected four-layer merge is paused, then requires its final live and cold
        //!   results after release. `paused_merge_before_publish_cannot_overwrite_a_truncated_epoch`
        //!   requires an old selected merge to leave a newer truncate epoch and CURRENT
        //!   unchanged. `prune_and_second_opener_keep_paused_merge_source_and_staging_alive`
        //!   requires prune and a second public store opener to preserve the worker's source
        //!   and staging until the selected merge publishes.
        //! - Security: these cases exercise the process-owned generation root through
        //!   `apps/lumen/src/segment_background_merge.rs` and `apps/lumen/src/segment_rdb.rs`.
        //!   They do not add caller-controlled bytes, paths, or identifiers. Existing malformed
        //!   CURRENT refusal cases in the parent target keep the file-input boundary covered.
        //!   The prune case asserts that a second opener cannot mistake an active worker staging
        //!   directory for abandoned input and delete it.
        //! - Performance: the approved #4246 plan requires a merge request at four delta layers
        //!   and a hard maximum of sixteen. The hard-cap case carries those structural limits;
        //!   its two-second observations are bounded progress checks, not a throughput, latency,
        //!   or RSS acceptance claim. Cleanup waits tolerate the process-wide single merge worker.

        use super::*;
        use std::collections::BTreeSet;

        const HARD_CAP_SEQUENCE: u64 = 10_100;
        const HARD_CAP_OTHER_COLLECTION: &str = "background-cap-other";
        const HARD_CAP_OTHER_FIELD: &str = "kw";
        const HARD_CAP_OTHER_ID: &str = "background-cap-other-id";
        const HARD_CAP_OTHER_BASE: &str = "background-cap-other-base";
        const HARD_CAP_OTHER_CHECKPOINTED: &str = "background-cap-other-checkpointed";
        const HARD_CAP_OTHER_LIVE: &str = "background-cap-other-live";

        const STALE_EPOCH_SEQUENCE: u64 = 11_100;
        const PRUNE_SEQUENCE: u64 = 12_100;

        /// Pauses one real background merge outside the root save lock. Each test sends
        /// `release` before it inspects a result, so a failed assertion cannot strand
        /// the process-wide worker or a checkpoint task.
        struct PauseOneMergePhase {
            phase: lumen::segment_rdb::MergePhase,
            reached: std::sync::mpsc::SyncSender<()>,
            release: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
            paused: std::sync::atomic::AtomicBool,
        }

        /// Sends the observer release on every exit path, including an assertion
        /// panic in the test body. The worker must never remain paused for a later
        /// test in this process.
        struct MergeRelease {
            sender: Option<std::sync::mpsc::Sender<()>>,
        }

        impl MergeRelease {
            fn new(sender: std::sync::mpsc::Sender<()>) -> Self {
                Self {
                    sender: Some(sender),
                }
            }

            fn release(&mut self) {
                if let Some(sender) = self.sender.take() {
                    let _ = sender.send(());
                }
            }
        }

        impl Drop for MergeRelease {
            fn drop(&mut self) {
                self.release();
            }
        }

        impl lumen::segment_rdb::MergeObserver for PauseOneMergePhase {
            fn observe(&self, phase: lumen::segment_rdb::MergePhase) -> std::io::Result<()> {
                if phase != self.phase
                    || self.paused.swap(true, std::sync::atomic::Ordering::SeqCst)
                {
                    return Ok(());
                }
                self.reached.send(()).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "background merge readiness receiver dropped",
                    )
                })?;
                self.release
                    .lock()
                    .expect("background merge release mutex")
                    .take()
                    .expect("background merge releases exactly once")
                    .recv()
                    .map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "background merge release sender dropped",
                        )
                    })?;
                Ok(())
            }
        }

        fn current_bytes(root: &Path) -> Vec<u8> {
            std::fs::read(root.join("CURRENT")).expect("read committed CURRENT bytes")
        }

        /// A merge scratch directory is a real root directory without the manifest that
        /// makes a generation publishable. This observes lifecycle state without baking
        /// the private staging filename into the contract.
        fn unpublished_generation_directories(root: &Path) -> BTreeSet<String> {
            std::fs::read_dir(root)
                .expect("read checkpoint root")
                .map(|entry| entry.expect("read checkpoint root entry"))
                .filter(|entry| {
                    entry.file_type().expect("inspect root entry type").is_dir()
                        && !entry.path().join("_generation.json").is_file()
                })
                .map(|entry| {
                    entry
                        .file_name()
                        .into_string()
                        .expect("checkpoint root entry is UTF-8")
                })
                .collect()
        }

        async fn create_hard_cap_other_collection(server: &TestServer) {
            server
                .put(&format!("/collections/{HARD_CAP_OTHER_COLLECTION}"))
                .json(&json!({ "fields": { HARD_CAP_OTHER_FIELD: { "type": "keyword" } } }))
                .await
                .assert_status_ok();
            hard_cap_index_other(server, HARD_CAP_OTHER_BASE).await;
        }

        async fn hard_cap_index_other(server: &TestServer, value: &str) {
            server
                .post(&format!("/collections/{HARD_CAP_OTHER_COLLECTION}/index"))
                .json(&json!({ "items": [{
                    "external_id": HARD_CAP_OTHER_ID,
                    "field": HARD_CAP_OTHER_FIELD,
                    "value": value,
                }] }))
                .await
                .assert_status_ok();
        }

        async fn hard_cap_other_ids(server: &TestServer, value: &str) -> Vec<String> {
            let response = server
                .post(&format!("/collections/{HARD_CAP_OTHER_COLLECTION}/search"))
                .json(&json!({
                    "query": { "term": { "field": HARD_CAP_OTHER_FIELD, "value": value } },
                    "limit": 16,
                    "track_total": true,
                }))
                .await;
            response.assert_status_ok();
            let mut ids = response.json::<Value>()["hits"]
                .as_array()
                .expect("hard-cap other hits")
                .iter()
                .map(|hit| {
                    hit["external_id"]
                        .as_str()
                        .expect("hard-cap other external ID")
                        .to_owned()
                })
                .collect::<Vec<_>>();
            ids.sort();
            ids
        }

        async fn wait_for_worker_ready(receiver: std::sync::mpsc::Receiver<()>) -> Result<()> {
            tokio::task::spawn_blocking(move || receiver.recv_timeout(Duration::from_secs(30)))
                .await
                .map_err(|error| {
                    anyhow::anyhow!("merge readiness receiver task panicked: {error}")
                })?
                .map_err(|error| anyhow::anyhow!("merge readiness timed out: {error}"))
        }

        async fn wait_for_background_idle(store: Arc<SegmentRdbStore>) -> Result<()> {
            tokio::task::spawn_blocking(move || store.wait_for_merges(Duration::from_secs(30)))
                .await
                .map_err(|error| anyhow::anyhow!("background wait task panicked: {error}"))?
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn cap_sixteen_deltas_waits_without_publishing_seventeen_and_keeps_other_work_live() {
            let fixture = full_compaction_fixture().await;
            let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
            let mut reached_rx = Some(reached_rx);
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let mut release = MergeRelease::new(release_tx);
            let observer = Arc::new(PauseOneMergePhase {
                phase: lumen::segment_rdb::MergePhase::BeforeEncode,
                reached: reached_tx,
                release: Mutex::new(Some(release_rx)),
                paused: std::sync::atomic::AtomicBool::new(false),
            });
            let store = Arc::new(
                SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                    .expect("open hard-cap observed store"),
            );
            create_hard_cap_other_collection(&fixture.server).await;
            store
                .save(&fixture.engine, HARD_CAP_SEQUENCE)
                .expect("publish unrelated collection base");

            let mut first_merge_ready = None;
            for round in 1..=16 {
                full_compaction_apply_keyword_round(&fixture.server, round).await;
                store
                    .save(&fixture.engine, HARD_CAP_SEQUENCE + round as u64)
                    .expect("publish sparse Keyword layer before hard-cap checkpoint");
                if round == 4 {
                    first_merge_ready = Some(
                        wait_for_worker_ready(
                            reached_rx
                                .take()
                                .expect("fourth save consumes merge readiness receiver"),
                        )
                        .await,
                    );
                }
            }
            let first_merge_ready =
                first_merge_ready.expect("fourth save waits for merge selection");
            let sixteenth_generation = stage1_current_generation_dir(&fixture.root);
            let sixteenth_manifest = stage1_read_manifest(&sixteenth_generation);
            let sixteenth_deltas =
                full_compaction_field_refs(&sixteenth_manifest, FULL_KEYWORD, "delta");

            // This save happens before the capped field becomes dirty for round 17. It
            // proves that a checkpoint with no new capped-field layer still progresses.
            hard_cap_index_other(&fixture.server, HARD_CAP_OTHER_CHECKPOINTED).await;
            let mut unrelated_checkpoint = Some(tokio::task::spawn_blocking({
                let store = store.clone();
                let engine = fixture.engine.clone();
                move || store.save(&engine, HARD_CAP_SEQUENCE + 17)
            }));
            let unrelated_observation = tokio::time::timeout(
                Duration::from_secs(2),
                unrelated_checkpoint
                    .as_mut()
                    .expect("unrelated checkpoint handle"),
            )
            .await;
            let unrelated_finished_while_paused = matches!(&unrelated_observation, Ok(Ok(Ok(_))));

            let mut capped_checkpoint = None;
            let mut capped_observation = None;
            let mut capped_delta_count_after_unrelated = None;
            let mut current_before_capped = None;
            let mut current_while_capped = None;
            let mut live_apply_observation = None;
            let mut live_apply_while_capped = None;
            let mut capped_round17_live = None;
            if first_merge_ready.is_ok() && unrelated_finished_while_paused {
                let after_unrelated =
                    stage1_read_manifest(&stage1_current_generation_dir(&fixture.root));
                capped_delta_count_after_unrelated =
                    Some(full_compaction_field_refs(&after_unrelated, FULL_KEYWORD, "delta").len());
                full_compaction_apply_keyword_round(&fixture.server, 17).await;
                current_before_capped = Some(current_bytes(&fixture.root));
                capped_checkpoint = Some(tokio::task::spawn_blocking({
                    let store = store.clone();
                    let engine = fixture.engine.clone();
                    move || store.save(&engine, HARD_CAP_SEQUENCE + 18)
                }));
                capped_observation = Some(
                    tokio::time::timeout(
                        Duration::from_secs(2),
                        capped_checkpoint
                            .as_mut()
                            .expect("capped checkpoint handle"),
                    )
                    .await,
                );
                current_while_capped = Some(current_bytes(&fixture.root));
                live_apply_observation = Some(
                    tokio::time::timeout(
                        Duration::from_secs(2),
                        hard_cap_index_other(&fixture.server, HARD_CAP_OTHER_LIVE),
                    )
                    .await,
                );
                if matches!(&live_apply_observation, Some(Ok(()))) {
                    if let Ok(ids) = tokio::time::timeout(
                        Duration::from_secs(2),
                        hard_cap_other_ids(&fixture.server, HARD_CAP_OTHER_LIVE),
                    )
                    .await
                    {
                        live_apply_while_capped = Some(ids);
                    }
                    if let Ok(ids) = tokio::time::timeout(
                        Duration::from_secs(2),
                        full_compaction_search_ids(
                            &fixture.server,
                            full_compaction_keyword_query(&full_compaction_round_keyword(17, 0)),
                        ),
                    )
                    .await
                    {
                        capped_round17_live = Some(ids);
                    }
                }
            }

            // Always release before joining a task or evaluating an assertion.
            release.release();
            let capped_waited = matches!(&capped_observation, Some(Err(_)));
            let unrelated_after_release = match &unrelated_observation {
                Err(_) => unrelated_checkpoint
                    .take()
                    .expect("timed-out unrelated checkpoint handle")
                    .await
                    .is_ok_and(|result| result.is_ok()),
                Ok(Ok(result)) => {
                    drop(unrelated_checkpoint.take());
                    result.is_ok()
                }
                Ok(Err(_)) => {
                    drop(unrelated_checkpoint.take());
                    false
                }
            };
            let capped_after_release = match &capped_observation {
                Some(Err(_)) => capped_checkpoint
                    .take()
                    .expect("timed-out capped checkpoint handle")
                    .await
                    .is_ok_and(|result| result.is_ok()),
                Some(Ok(Ok(result))) => {
                    drop(capped_checkpoint.take());
                    result.is_ok()
                }
                Some(Ok(Err(_))) => {
                    drop(capped_checkpoint.take());
                    false
                }
                None => false,
            };
            let drained = wait_for_background_idle(store.clone()).await;

            assert!(
                first_merge_ready.is_ok(),
                "the fourth real Keyword delta must select the paused merge: {first_merge_ready:?}",
            );
            assert_eq!(
                sixteenth_deltas.len(),
                16,
                "the paused worker must allow exactly sixteen published deltas before hard-cap admission engages",
            );
            assert!(
                unrelated_finished_while_paused && unrelated_after_release,
                "a checkpoint without a newly dirty capped field must finish while encoding is paused: {unrelated_observation:?}",
            );
            assert_eq!(
                capped_delta_count_after_unrelated,
                Some(16),
                "unrelated checkpoint must not add a seventeenth layer for the capped Keyword field",
            );
            assert!(
                capped_waited,
                "the seventeenth dirty Keyword checkpoint must wait for the selected merge instead of publishing layer 17: {capped_observation:?}",
            );
            assert_eq!(
                current_while_capped,
                current_before_capped,
                "the current pointer must stay on the sixteen-layer checkpoint until the paused merge releases",
            );
            assert!(
                matches!(&live_apply_observation, Some(Ok(()))),
                "an unrelated HTTP apply must finish within the bounded wait while the capped checkpoint is blocked: {live_apply_observation:?}",
            );
            assert_eq!(
                live_apply_while_capped,
                Some(vec![HARD_CAP_OTHER_ID.to_owned()]),
                "an unrelated live apply and query must progress while the capped checkpoint waits",
            );
            assert_eq!(
                capped_round17_live,
                Some(vec![full_compaction_base_id(0)]),
                "the live engine keeps the seventeenth update visible while durable publication waits",
            );
            assert!(
                capped_after_release,
                "the waiting seventeenth checkpoint must finish after merge release",
            );
            drained.expect("all queued background compactions must finish after release");

            let latest_generation = stage1_current_generation_dir(&fixture.root);
            let latest_manifest = stage1_read_manifest(&latest_generation);
            assert_eq!(
                latest_manifest["checkpoint_sequence"],
                json!(HARD_CAP_SEQUENCE + 18),
                "the released capped checkpoint must become the final durable cut",
            );
            assert!(
                full_compaction_field_refs(&latest_manifest, FULL_KEYWORD, "delta").len() <= 16,
                "no durable catalog may publish a seventeenth Keyword delta",
            );
            assert_eq!(
                hard_cap_other_ids(&fixture.server, HARD_CAP_OTHER_LIVE).await,
                vec![HARD_CAP_OTHER_ID.to_owned()],
                "live state retains the unrelated apply through the released capped checkpoint",
            );
            let cold = SegmentRdbStore::new(&fixture.root)
                .expect("open hard-cap cold store")
                .load_current_generation()
                .expect("cold-open hard-cap CURRENT")
                .expect("hard-cap CURRENT generation");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("hard-cap cold HTTP server");
            assert_eq!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(17, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "cold CURRENT must retain the seventeenth update after capacity admission releases",
            );
            assert_eq!(
                hard_cap_other_ids(&cold_server, HARD_CAP_OTHER_LIVE).await,
                vec![HARD_CAP_OTHER_ID.to_owned()],
                "cold CURRENT must retain unrelated work that progressed while the cap waited",
            );
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn paused_merge_before_publish_cannot_overwrite_a_truncated_epoch() {
            let fixture = full_compaction_fixture().await;
            let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let mut release = MergeRelease::new(release_tx);
            let observer = Arc::new(PauseOneMergePhase {
                phase: lumen::segment_rdb::MergePhase::BeforePublish,
                reached: reached_tx,
                release: Mutex::new(Some(release_rx)),
                paused: std::sync::atomic::AtomicBool::new(false),
            });
            let store = Arc::new(
                SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                    .expect("open stale-epoch observed store"),
            );

            for round in 1..=4 {
                full_compaction_apply_keyword_round(&fixture.server, round).await;
                store
                    .save(&fixture.engine, STALE_EPOCH_SEQUENCE + round as u64)
                    .expect("publish delta before stale-epoch merge");
            }
            let reached = wait_for_worker_ready(reached_rx).await;
            let source_name = stage1_reuse_current_name(&fixture.root);
            let source_manifest = stage1_read_manifest(&fixture.root.join(&source_name));
            let source_epoch = stage1_reuse_collection_u64(
                full_compaction_collection(&source_manifest),
                "collection_generation",
            );
            let active_staging = unpublished_generation_directories(&fixture.root);

            let truncate = fixture
                .server
                .post(&format!(
                    "/collections/{FULL_COMPACTION_COLLECTION}/docs:truncate"
                ))
                .await;
            let truncate_status = truncate.status_code();
            let replacement = if reached.is_ok() {
                store.save(&fixture.engine, STALE_EPOCH_SEQUENCE + 5)
            } else {
                Err(anyhow::anyhow!("merge never reached BeforePublish"))
            };
            let replacement_current = current_bytes(&fixture.root);
            let replacement_manifest =
                stage1_read_manifest(&stage1_current_generation_dir(&fixture.root));
            let replacement_epoch = stage1_reuse_collection_u64(
                full_compaction_collection(&replacement_manifest),
                "collection_generation",
            );
            let old_value_live = full_compaction_search_ids(
                &fixture.server,
                full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
            )
            .await;

            // Release before inspection. A stale worker is allowed to refuse; it is not
            // allowed to install its old collection epoch after this replacement checkpoint.
            release.release();
            let drained = wait_for_background_idle(store.clone()).await;
            let final_current = current_bytes(&fixture.root);
            let final_staging = unpublished_generation_directories(&fixture.root);

            assert!(
                reached.is_ok(),
                "the fourth delta merge must reach BeforePublish outside the root lock: {reached:?}",
            );
            assert_eq!(
                truncate_status,
                axum::http::StatusCode::NO_CONTENT,
                "truncate must advance the live collection epoch before the old merge releases",
            );
            assert!(
                replacement.is_ok(),
                "the replacement checkpoint for a truncated epoch must publish while old merge publication is paused: {replacement:?}",
            );
            assert!(
                !active_staging.is_empty(),
                "a paused worker must own unpublished staging before the epoch replacement",
            );
            assert!(
                replacement_epoch > source_epoch,
                "truncate must allocate a new collection generation before the old merge can publish",
            );
            assert!(
                old_value_live.is_empty(),
                "live truncated state must mask the old merge input before release",
            );
            drained
                .expect("old merge must reach a terminal stale refusal or safe publication result");
            assert_eq!(
                final_current, replacement_current,
                "a stale merge selected from the old epoch must not overwrite replacement CURRENT",
            );
            assert!(
                final_staging.is_empty(),
                "the stale worker must clean its owned staging after terminal refusal",
            );

            let cold = SegmentRdbStore::new(&fixture.root)
                .expect("open stale-epoch cold store")
                .load_current_generation()
                .expect("cold-open replacement CURRENT")
                .expect("replacement CURRENT generation");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("stale-epoch cold HTTP server");
            assert!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
                )
                .await
                .is_empty(),
                "cold CURRENT must retain the truncated epoch rather than the old merge data",
            );
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn prune_and_second_opener_keep_paused_merge_source_and_staging_alive() {
            let fixture = full_compaction_fixture().await;
            let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let mut release = MergeRelease::new(release_tx);
            let observer = Arc::new(PauseOneMergePhase {
                phase: lumen::segment_rdb::MergePhase::BeforeEncode,
                reached: reached_tx,
                release: Mutex::new(Some(release_rx)),
                paused: std::sync::atomic::AtomicBool::new(false),
            });
            let store = Arc::new(
                SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                    .expect("open prune observed store"),
            );

            for round in 1..=4 {
                full_compaction_apply_keyword_round(&fixture.server, round).await;
                store
                    .save(&fixture.engine, PRUNE_SEQUENCE + round as u64)
                    .expect("publish delta before prune race");
            }
            let reached = wait_for_worker_ready(reached_rx).await;
            let source_name = stage1_reuse_current_name(&fixture.root);
            let source_path = fixture.root.join(&source_name);
            let staging_before_prune = unpublished_generation_directories(&fixture.root);

            let prune = if reached.is_ok() {
                tokio::task::spawn_blocking({
                    let store = store.clone();
                    move || store.prune(1)
                })
                .await
                .expect("prune task must not panic")
            } else {
                Err(anyhow::anyhow!("merge never reached BeforeEncode"))
            };
            let source_after_prune = source_path.is_dir();
            let staging_after_prune = unpublished_generation_directories(&fixture.root);
            let second_open = SegmentRdbStore::new(&fixture.root)
                .and_then(|second| second.load_current_generation());
            let second_engine = second_open
                .as_ref()
                .ok()
                .and_then(|loaded| loaded.as_ref().map(|loaded| loaded.engine.clone()));
            let source_after_second_open = source_path.is_dir();
            let staging_after_second_open = unpublished_generation_directories(&fixture.root);

            // Always release the real worker before evaluating the prune/open observations.
            release.release();
            let drained = wait_for_background_idle(store.clone()).await;
            let latest_name = stage1_reuse_current_name(&fixture.root);
            let latest_generation = fixture.root.join(&latest_name);
            let latest_manifest = stage1_read_manifest(&latest_generation);
            let staging_after_release = unpublished_generation_directories(&fixture.root);

            assert!(
                reached.is_ok(),
                "the fourth delta merge must own a source and staging directory before prune",
            );
            assert!(
                prune.is_ok(),
                "prune(1) must finish while worker encoding is paused: {prune:?}"
            );
            assert!(
                !staging_before_prune.is_empty(),
                "the paused worker must have unpublished staging for the selected merge",
            );
            assert!(
                source_after_prune && source_after_second_open,
                "prune and a second store opener must retain the selected source generation",
            );
            assert_eq!(
                staging_after_prune, staging_before_prune,
                "prune must not reclaim active background staging",
            );
            assert_eq!(
                staging_after_second_open, staging_before_prune,
                "a second store opener must not sweep active staging as abandoned",
            );
            assert!(
                second_engine.is_some(),
                "a second store must cold-open CURRENT while the worker owns source and staging",
            );
            drained.expect("released background merge must finish after prune/open race");
            assert_ne!(
                latest_name, source_name,
                "released worker must publish a new merged CURRENT after its protected source survives",
            );
            assert!(
                full_compaction_field_refs(&latest_manifest, FULL_KEYWORD, "delta").len() < 4,
                "released selected merge must reduce the four-layer Keyword catalog",
            );
            assert!(
                staging_after_release.is_empty(),
                "completed worker must clean its now-unowned staging directory",
            );
            assert_eq!(
                full_compaction_search_ids(
                    &fixture.server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "live state must retain the merged fourth Keyword update after prune/open race",
            );
            let premerge_server = TestServer::new(router(AppState::open(
                second_engine.expect("second store cold engine"),
            )))
            .expect("premerge second-store HTTP server");
            assert_eq!(
                full_compaction_search_ids(
                    &premerge_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "the second opener's cold source must keep the fourth layer while merge is paused",
            );
            let cold = SegmentRdbStore::new(&fixture.root)
                .expect("open post-prune cold store")
                .load_current_generation()
                .expect("cold-open merged CURRENT")
                .expect("merged CURRENT generation");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("post-prune cold HTTP server");
            assert_eq!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "cold merged CURRENT must retain the fourth Keyword update after prune/open race",
            );
        }

        mod durable_restore_vs_paused_merge_contract {
            //! # Facets
            //!
            //! - Behavior: `indexing_durable_oracle.rs:8667`, `:8671`, `:8710`, and
            //!   `:8723` require a durable restore to activate its candidate before the
            //!   paused old merge releases, then retain that candidate live and cold.
            //!   Change points: `apps/lumen/src/segment_restore.rs:154-289` and
            //!   `apps/lumen/src/segment_background_merge.rs:449-552`.
            //! - Security: `indexing_durable_oracle.rs:8694` and `:8723` require the
            //!   process-written `CURRENT` input to remain on the restore candidate after
            //!   stale-worker release. Boundary: `apps/lumen/src/segment_rdb.rs:635-676`.
            //! - Performance: `apps/lumen/ROADMAP.md:73-78` promises one background
            //!   merge that rechecks segment input and collection epoch before publication.
            //!   `indexing_durable_oracle.rs:8694` asserts that structural promise; the
            //!   two-second observation is only an interleaving control, not a latency claim.

            use super::*;

            const RESTORED_COLLECTION: &str = "background-restored";
            const RESTORED_FIELD: &str = "kw";
            const RESTORED_ID: &str = "background-restored-id";
            const RESTORED_VALUE: &str = "background-restored-value";

            struct RestoreRaceFixture {
                _dir: tempfile::TempDir,
                root: PathBuf,
                engine: Arc<Engine>,
                writer: Arc<WriteCoordinator>,
                aof: SharedAof,
                server: TestServer,
            }

            async fn restore_race_fixture() -> RestoreRaceFixture {
                let dir = tempfile::tempdir().expect("restore-versus-merge fixture root");
                let root = dir.path().join("segments");
                let store = SegmentRdbStore::new(&root).expect("open restore-versus-merge store");
                let engine = Arc::new(Engine::new());
                let aof: SharedAof = Arc::new(Mutex::new(
                    AofWriter::open(dir.path().join("restore-versus-merge.aof"))
                        .expect("open restore-versus-merge AOF"),
                ));
                let wal: SharedWal = Arc::new(MemWal::new());
                let writer =
                    WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
                let sink_writer: Arc<dyn WriteSink> = writer.clone();
                let server = TestServer::new(router(AppState::with_components(
                    engine.clone(),
                    Arc::new(AuthConfig::open()),
                    sink_writer,
                )))
                .expect("restore-versus-merge HTTP server");
                full_compaction_create_collection(&server).await;
                full_compaction_post_items(&server, full_compaction_base_items()).await;
                stage1_restore_legacy_base(&engine, "restore-versus-merge base");
                store
                    .save(&engine, writer.applied_seq())
                    .expect("publish restore-versus-merge base at real writer sequence");
                RestoreRaceFixture {
                    _dir: dir,
                    root,
                    engine,
                    writer,
                    aof,
                    server,
                }
            }

            async fn restored_snapshot() -> SnapshotV1 {
                let candidate = Arc::new(Engine::new());
                let server = TestServer::new(router(AppState::open(candidate.clone())))
                    .expect("restored candidate server");
                server
                    .put(&format!("/collections/{RESTORED_COLLECTION}"))
                    .json(&json!({ "fields": { RESTORED_FIELD: { "type": "keyword" } } }))
                    .await
                    .assert_status_ok();
                server
                    .post(&format!("/collections/{RESTORED_COLLECTION}/index"))
                    .json(&json!({ "items": [{
                        "external_id": RESTORED_ID,
                        "field": RESTORED_FIELD,
                        "value": RESTORED_VALUE,
                    }] }))
                    .await
                    .assert_status_ok();
                candidate.snapshot().expect("snapshot restored candidate")
            }

            async fn restored_ids(server: &TestServer) -> Vec<String> {
                let response = server
                    .post(&format!("/collections/{RESTORED_COLLECTION}/search"))
                    .json(&json!({
                        "query": { "term": { "field": RESTORED_FIELD, "value": RESTORED_VALUE } },
                        "limit": 8,
                        "track_total": true,
                    }))
                    .await;
                response.assert_status_ok();
                let mut ids = response.json::<Value>()["hits"]
                    .as_array()
                    .expect("restored query hits")
                    .iter()
                    .map(|hit| {
                        hit["external_id"]
                            .as_str()
                            .expect("restored query external ID")
                            .to_owned()
                    })
                    .collect::<Vec<_>>();
                ids.sort();
                ids
            }

            #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
            async fn durable_restore_while_before_publish_merge_is_paused_rejects_old_engine_epoch()
            {
                let fixture = restore_race_fixture().await;
                let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
                let (release_tx, release_rx) = std::sync::mpsc::channel();
                let mut release = MergeRelease::new(release_tx);
                let observer = Arc::new(PauseOneMergePhase {
                    phase: lumen::segment_rdb::MergePhase::BeforePublish,
                    reached: reached_tx,
                    release: Mutex::new(Some(release_rx)),
                    paused: std::sync::atomic::AtomicBool::new(false),
                });
                let store = Arc::new(
                    SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                        .expect("open restore-versus-merge store"),
                );
                for round in 1..=4 {
                    full_compaction_apply_keyword_round(&fixture.server, round).await;
                    store
                        .save(&fixture.engine, fixture.writer.applied_seq())
                        .expect("publish delta before restore-versus-merge race");
                }
                let reached = wait_for_worker_ready(reached_rx).await;
                let source_current = current_bytes(&fixture.root);
                let staging_before_restore = unpublished_generation_directories(&fixture.root);

                let sink = Arc::new(
                    lumen::segment_restore::SegmentRestoreSink::new(
                        fixture.engine.clone(),
                        store.clone(),
                        fixture.writer.clone() as Arc<dyn WriteSink>,
                        fixture.aof.clone(),
                    )
                    .expect("real durable restore sink"),
                );
                let snapshot = restored_snapshot().await;
                let mut restore = tokio::spawn({
                    let sink = sink.clone();
                    async move { lumen::api::RestoreSink::restore(sink.as_ref(), snapshot).await }
                });
                let restore_before_release =
                    tokio::time::timeout(Duration::from_secs(2), &mut restore).await;
                let restore_finished_before_release =
                    matches!(&restore_before_release, Ok(Ok(Ok(()))));
                let candidate_current = current_bytes(&fixture.root);
                let candidate_collections = store
                    .load_current_generation()
                    .ok()
                    .flatten()
                    .and_then(|loaded| loaded.engine.list_collections().ok());
                let live_collections_before_release = fixture.engine.list_collections().ok();

                // Always release before joining. `MergeRelease` repeats this on unwinding.
                release.release();
                let restore_result = match restore_before_release {
                    Ok(result) => result.expect("restore task must not panic"),
                    Err(_) => tokio::time::timeout(Duration::from_secs(30), restore)
                        .await
                        .expect("restore must finish after worker release")
                        .expect("restore task must not panic"),
                };
                let drained = wait_for_background_idle(store.clone()).await;
                let final_current = current_bytes(&fixture.root);
                let final_staging = unpublished_generation_directories(&fixture.root);

                assert!(
                    reached.is_ok(),
                    "the fourth real Keyword delta must select a merge before its publication pause: {reached:?}",
                );
                assert!(
                    restore_finished_before_release,
                    "the real durable restore must finish while the old merge pauses outside the root lock",
                );
                assert!(
                    restore_result.is_ok(),
                    "restore must activate its candidate after durable CURRENT publication: {restore_result:?}",
                );
                assert_ne!(
                    candidate_current, source_current,
                    "the durable restore must replace the old merge source CURRENT before release",
                );
                assert!(
                    !staging_before_restore.is_empty(),
                    "the selected old merge must own staging before the restore changes the Engine epoch",
                );
                assert_eq!(
                    candidate_collections,
                    Some(vec![RESTORED_COLLECTION.to_owned()]),
                    "the durable candidate CURRENT must name only the restored collection before old merge release",
                );
                assert_eq!(
                    live_collections_before_release,
                    Some(vec![RESTORED_COLLECTION.to_owned()]),
                    "the live Engine must activate the restored epoch before old merge release",
                );
                drained.expect("the released stale merge must reach a terminal safe result");
                assert_eq!(
                    final_current, candidate_current,
                    "the old merge must not overwrite restore-owned CURRENT after its old Engine epoch is invalid",
                );
                assert!(
                    final_staging.is_empty(),
                    "the old merge must clean owned staging after its stale epoch is refused",
                );
                assert_eq!(
                    fixture
                        .engine
                        .list_collections()
                        .expect("list restored live collections"),
                    vec![RESTORED_COLLECTION.to_owned()],
                    "live state must not resurrect the old merge collection after release",
                );
                assert_eq!(
                    restored_ids(&fixture.server).await,
                    vec![RESTORED_ID.to_owned()],
                    "live search must retain the restored value after the stale merge releases",
                );

                let cold = SegmentRdbStore::new(&fixture.root)
                    .expect("open final restore root")
                    .load_current_generation()
                    .expect("cold-open final restore CURRENT")
                    .expect("final restore CURRENT generation");
                let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                    .expect("final restore cold server");
                assert_eq!(
                    restored_ids(&cold_server).await,
                    vec![RESTORED_ID.to_owned()],
                    "cold CURRENT must retain the restored value and not the old merge output",
                );
            }
        }

        mod pending_frozen_selected_merge_contract {
            //! # Facets
            //!
            //! - Behavior: `indexing_durable_oracle.rs:8860`, `:8883`, `:8891`, and
            //!   `:8900` require the failed fifth cut to retry before the sixth live
            //!   mutation is captured. Change points: `apps/lumen/src/segment_rdb.rs:459-513`
            //!   and `apps/lumen/src/segment_background_merge.rs:447-501`.
            //! - Security: `indexing_durable_oracle.rs:8864`, `:8875`, and `:8891` keep
            //!   process-written `CURRENT` and its predecessor from making frozen input
            //!   stale. Boundary: `apps/lumen/src/segment_rdb.rs:637-676`.
            //! - Performance: `apps/lumen/ROADMAP.md:52-55` promises that a
            //!   pre-publication failure retains frozen changes; `:73-78` requires merge
            //!   input revalidation. `indexing_durable_oracle.rs:8875` asserts both
            //!   structural rules. Its waits are cleanup and interleaving controls only.

            use super::*;
            use std::sync::atomic::{AtomicBool, Ordering};
            use storage_durable::{CommitStep, FailureInjector, FailurePoint};

            const PENDING_FROZEN_SEQUENCE: u64 = 14_100;

            #[derive(Default)]
            struct FailNextSyncFile {
                armed: AtomicBool,
            }

            impl FailNextSyncFile {
                fn arm(&self) {
                    self.armed.store(true, Ordering::Release);
                }
            }

            impl FailureInjector for FailNextSyncFile {
                fn check(&self, point: &FailurePoint) -> std::io::Result<()> {
                    if point.step == CommitStep::SyncFile
                        && self.armed.swap(false, Ordering::AcqRel)
                    {
                        return Err(std::io::Error::other(
                            "injected pending-frozen SyncFile failure",
                        ));
                    }
                    Ok(())
                }
            }

            #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
            async fn failed_frozen_checkpoint_blocks_selected_merge_and_retries_its_original_cut() {
                let fixture = full_compaction_fixture().await;
                let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
                let (release_tx, release_rx) = std::sync::mpsc::channel();
                let mut release = MergeRelease::new(release_tx);
                let observer = Arc::new(PauseOneMergePhase {
                    phase: lumen::segment_rdb::MergePhase::BeforeEncode,
                    reached: reached_tx,
                    release: Mutex::new(Some(release_rx)),
                    paused: AtomicBool::new(false),
                });
                let injector = Arc::new(FailNextSyncFile::default());
                let store = Arc::new(
                    SegmentRdbStore::with_failure_injector_and_merge_observer(
                        &fixture.root,
                        injector.clone(),
                        observer,
                    )
                    .expect("open failure-injected observed store"),
                );

                for round in 1..=4 {
                    full_compaction_apply_keyword_round(&fixture.server, round).await;
                    store
                        .save(&fixture.engine, PENDING_FROZEN_SEQUENCE + round as u64)
                        .expect("publish four sparse Keyword layers before selected merge");
                }
                let reached = wait_for_worker_ready(reached_rx).await;
                let current_before_failure = current_bytes(&fixture.root);
                let staging_before_failure = unpublished_generation_directories(&fixture.root);

                full_compaction_apply_keyword_round(&fixture.server, 5).await;
                injector.arm();
                let failed_checkpoint = store.save(&fixture.engine, PENDING_FROZEN_SEQUENCE + 5);
                let current_after_failure = current_bytes(&fixture.root);

                // This sixth mutation stays live. A retry of the frozen fifth cut must
                // not capture it early. If the old merge advances CURRENT, the original
                // pending predecessor goes stale and this distinction catches the loss.
                full_compaction_apply_keyword_round(&fixture.server, 6).await;

                // Release before every join or assertion. The drop guard repeats this on panic.
                release.release();
                let merge_after_release = wait_for_background_idle(store.clone()).await;
                let current_after_old_merge = current_bytes(&fixture.root);
                let staging_after_old_merge = unpublished_generation_directories(&fixture.root);

                let retry_fifth = store.save(&fixture.engine, PENDING_FROZEN_SEQUENCE + 5);
                let retry_current = current_bytes(&fixture.root);
                let retry_cold = SegmentRdbStore::new(&fixture.root)
                    .expect("open retry root")
                    .load_current_generation()
                    .expect("cold-open fifth retry")
                    .expect("fifth retry CURRENT exists");
                let retry_server = TestServer::new(router(AppState::open(retry_cold.engine)))
                    .expect("fifth retry cold server");
                let fifth_ids = full_compaction_search_ids(
                    &retry_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(5, 0)),
                )
                .await;
                let sixth_ids_before_fresh_capture = full_compaction_search_ids(
                    &retry_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(6, 0)),
                )
                .await;

                let fresh_sixth = store.save(&fixture.engine, PENDING_FROZEN_SEQUENCE + 6);
                wait_for_background_idle(store.clone())
                    .await
                    .expect("fresh sixth checkpoint and requested merge finish");
                let final_cold = SegmentRdbStore::new(&fixture.root)
                    .expect("open final retry root")
                    .load_current_generation()
                    .expect("cold-open sixth checkpoint")
                    .expect("sixth checkpoint CURRENT exists");
                let final_server = TestServer::new(router(AppState::open(final_cold.engine)))
                    .expect("sixth checkpoint cold server");

                assert!(
                    reached.is_ok(),
                    "the fourth real Keyword delta must select the merge before checkpoint failure: {reached:?}",
                );
                assert!(
                    failed_checkpoint.is_err(),
                    "the armed pre-publication SyncFile failure must fail the fifth checkpoint",
                );
                assert_eq!(
                    current_after_failure, current_before_failure,
                    "a failed checkpoint must not move CURRENT before it retains the frozen cut",
                );
                assert!(
                    !staging_before_failure.is_empty(),
                    "the selected merge must own staging before the fifth checkpoint fails",
                );
                merge_after_release.expect(
                    "the old selected merge must safely refuse once pending frozen payload owns its predecessor",
                );
                assert_eq!(
                    current_after_old_merge, current_before_failure,
                    "the released old merge must not move CURRENT past the pending frozen predecessor",
                );
                assert!(
                    staging_after_old_merge.is_empty(),
                    "the old merge must clean staging after it observes pending frozen work",
                );
                assert!(
                    retry_fifth.is_ok(),
                    "retrying the original fifth sequence must publish retained frozen data: {retry_fifth:?}",
                );
                assert_ne!(
                    retry_current, current_before_failure,
                    "the fifth retry must create its durable generation after old merge refusal",
                );
                assert_eq!(
                    fifth_ids,
                    vec![full_compaction_base_id(0)],
                    "cold retry must retain the fifth frozen Keyword value",
                );
                assert!(
                    sixth_ids_before_fresh_capture.is_empty(),
                    "retrying fifth frozen payload must not silently recapture the later sixth live mutation",
                );
                assert!(
                    fresh_sixth.is_ok(),
                    "a later explicit sixth checkpoint must publish the remaining live mutation: {fresh_sixth:?}",
                );
                assert_eq!(
                    full_compaction_search_ids(
                        &final_server,
                        full_compaction_keyword_query(&full_compaction_round_keyword(6, 0)),
                    )
                    .await,
                    vec![full_compaction_base_id(0)],
                    "the later fresh checkpoint must retain the sixth mutation after fifth retry",
                );
            }
        }
    }

    #[cfg(unix)]
    mod vector_base_compaction_contract {
        //! # Facets
        //!
        //! - Behavior: `v2_flat_cpu_vector_base_compaction_maps_new_ids_live_cold_and_retained`
        //!   and `v2_hnsw_cpu_vector_base_compaction_maps_new_ids_live_cold_and_retained`
        //!   use public collection, index, replace, delete, search, and
        //!   `SegmentRdbStore::save` operations. They require a new mapped vector
        //!   base after four measured delta layers, then verify Flat and HNSW live,
        //!   cold, retained, absent, deleted, appended, newer-update, and
        //!   full-replace behavior.
        //! - Security: `v2_current_refuses_malformed_mapped_flat_vector_base_without_fallback`
        //!   corrupts the persisted mapped-base local-row descriptor and checksum.
        //!   It requires `load_current_generation` to refuse each input and leave
        //!   CURRENT unchanged. This covers the changed file-read boundary in
        //!   `apps/lumen/src/segment_rdb.rs:1625-1795`.
        //! - Performance: the user-approved #4246 plan requires a compaction
        //!   request at four deltas and base replacement only when measured delta
        //!   bytes meet or exceed the complete base. These cases assert both
        //!   conditions and hard-link reuse. They do not claim a graph-rebuild,
        //!   latency, or RSS result because no public probe measures those paths.

        use super::*;
        use sha2::{Digest, Sha256};

        const VECTOR_BASE_COLLECTION: &str = "vector-base-compaction";
        const VECTOR_BASE_KIND: &str = "kind";
        const VECTOR_BASE_FIELD: &str = "embedding";
        const VECTOR_BASE_ROWS: usize = 96;
        const VECTOR_BASE_SEQUENCE: u64 = 13_100;
        const VECTOR_BASE_HOT_ID: &str = "vector-base-hot";
        const VECTOR_BASE_DELETED_ID: &str = "vector-base-deleted";
        const VECTOR_BASE_ABSENT_ID: &str = "vector-base-absent";
        const VECTOR_BASE_APPENDED_ID: &str = "vector-base-appended";

        struct VectorBaseFixture {
            _dir: tempfile::TempDir,
            root: PathBuf,
            store: SegmentRdbStore,
            engine: Arc<Engine>,
            server: TestServer,
            base_name: String,
        }

        fn vector_base_id(index: usize) -> String {
            format!("vector-base-{index:03}")
        }

        fn vector_base_vector(x: f32) -> Value {
            // The earlier `[x, -x]` fixture put every L2 point on one line.
            // That makes an approximate HNSW walk depend on insertion order.
            // This odd-multiplier permutation gives every integral fixture key
            // a unique, deterministic two-coordinate point. The query uses
            // the same point, so its expected ID has distance zero while every
            // other fixture ID has a strictly positive L2 distance.
            let key = x as u32;
            assert_eq!(key as f32, x, "vector fixture keys must be integral");
            let mixed = key.wrapping_mul(0x9e37_79b1).wrapping_add(0x7f4a_7c15);
            json!([(mixed & 0xffff) as f32, (mixed >> 16) as f32])
        }

        fn vector_base_initial_x(index: usize) -> f32 {
            1_000.0 + index as f32
        }

        fn vector_base_round_x(round: usize, index: usize) -> f32 {
            10_000.0 * round as f32 + index as f32
        }

        fn vector_base_hot_x(round: usize) -> f32 {
            100_000.0 + round as f32
        }

        fn vector_base_appended_x(round: usize) -> f32 {
            200_000.0 + round as f32
        }

        async fn vector_base_create_collection(server: &TestServer, backend: &str) {
            server
                .put(&format!("/collections/{VECTOR_BASE_COLLECTION}"))
                .json(&json!({ "fields": {
                    VECTOR_BASE_KIND: { "type": "keyword" },
                    VECTOR_BASE_FIELD: {
                        "type": "vector", "dim": 2, "metric": "l2", "backend": backend,
                    },
                }}))
                .await
                .assert_status_ok();
        }

        async fn vector_base_post_items(server: &TestServer, items: Vec<Value>) {
            for chunk in items.chunks(1_000) {
                server
                    .post(&format!("/collections/{VECTOR_BASE_COLLECTION}/index"))
                    .json(&json!({ "items": chunk }))
                    .await
                    .assert_status_ok();
            }
        }

        fn vector_base_initial_items() -> Vec<Value> {
            let mut items = Vec::with_capacity((VECTOR_BASE_ROWS + 3) * 2);
            for index in 0..VECTOR_BASE_ROWS {
                let id = vector_base_id(index);
                items.push(json!({
                    "external_id": id,
                    "field": VECTOR_BASE_KIND,
                    "value": format!("base-kind-{index:03}"),
                }));
                items.push(json!({
                    "external_id": vector_base_id(index),
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_initial_x(index)),
                }));
            }
            items.extend([
                json!({
                    "external_id": VECTOR_BASE_HOT_ID,
                    "field": VECTOR_BASE_KIND,
                    "value": "hot-base",
                }),
                json!({
                    "external_id": VECTOR_BASE_HOT_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_hot_x(0)),
                }),
                json!({
                    "external_id": VECTOR_BASE_DELETED_ID,
                    "field": VECTOR_BASE_KIND,
                    "value": "deleted-base",
                }),
                json!({
                    "external_id": VECTOR_BASE_DELETED_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(500.0),
                }),
                // This document is in the collection EID metadata but never has a
                // vector. A mapped vector base must not make it searchable.
                json!({
                    "external_id": VECTOR_BASE_ABSENT_ID,
                    "field": VECTOR_BASE_KIND,
                    "value": "absent-base-vector",
                }),
            ]);
            items
        }

        async fn vector_base_fixture(backend: &str) -> VectorBaseFixture {
            let dir = tempfile::tempdir().expect("vector base compaction fixture root");
            let root = dir.path().join("segments");
            let store = SegmentRdbStore::new(&root).expect("create vector base store");
            let engine = Arc::new(Engine::new());
            let server = TestServer::new(router(AppState::open(engine.clone())))
                .expect("vector base compaction HTTP server");
            vector_base_create_collection(&server, backend).await;
            vector_base_post_items(&server, vector_base_initial_items()).await;
            stage1_restore_legacy_base(&engine, "vector base compaction fixture");
            store
                .save(&engine, VECTOR_BASE_SEQUENCE)
                .expect("publish vector base generation");
            VectorBaseFixture {
                _dir: dir,
                root: root.clone(),
                store,
                engine,
                server,
                base_name: stage1_reuse_current_name(&root),
            }
        }

        async fn vector_base_apply_round(server: &TestServer, round: usize) {
            assert!(
                (1..=4).contains(&round),
                "vector base contract has four rounds"
            );
            let mut items = Vec::with_capacity(VECTOR_BASE_ROWS + 2);
            for index in 0..VECTOR_BASE_ROWS {
                items.push(json!({
                    "external_id": vector_base_id(index),
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_round_x(round, index)),
                }));
            }
            if round < 4 {
                items.push(json!({
                    "external_id": VECTOR_BASE_HOT_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_hot_x(round)),
                }));
            }
            if round > 1 {
                items.push(json!({
                    "external_id": VECTOR_BASE_APPENDED_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_appended_x(round)),
                }));
            }
            vector_base_post_items(server, items).await;

            if round == 1 {
                server
                    .delete(&format!(
                        "/collections/{VECTOR_BASE_COLLECTION}/index/{VECTOR_BASE_DELETED_ID}"
                    ))
                    .await
                    .assert_status(axum::http::StatusCode::NO_CONTENT);
                vector_base_post_items(
                    server,
                    vec![
                        json!({
                            "external_id": VECTOR_BASE_APPENDED_ID,
                            "field": VECTOR_BASE_KIND,
                            "value": "appended-after-base",
                        }),
                        json!({
                            "external_id": VECTOR_BASE_APPENDED_ID,
                            "field": VECTOR_BASE_FIELD,
                            "value": vector_base_vector(vector_base_appended_x(round)),
                        }),
                    ],
                )
                .await;
            }
            if round == 4 {
                server
                    .put(&format!(
                        "/collections/{VECTOR_BASE_COLLECTION}/docs:replace"
                    ))
                    .json(&json!({ "docs": [{
                        "external_id": VECTOR_BASE_HOT_ID,
                        "fields": {
                            VECTOR_BASE_KIND: "hot-full-replace",
                            VECTOR_BASE_FIELD: vector_base_vector(vector_base_hot_x(round)),
                        },
                    }]}))
                    .await
                    .assert_status_ok();
            }
        }

        async fn vector_base_apply_newer_update(server: &TestServer) {
            vector_base_post_items(
                server,
                vec![json!({
                    "external_id": VECTOR_BASE_APPENDED_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_appended_x(5)),
                })],
            )
            .await;
        }

        fn vector_base_collection<'a>(manifest: &'a Value) -> &'a Value {
            stage1_reuse_catalog_collection(manifest, VECTOR_BASE_COLLECTION)
        }

        fn vector_base_field_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
            let mut refs: Vec<_> = vector_base_collection(manifest)["segments"]
                .as_array()
                .expect("vector base catalog segments")
                .iter()
                .filter(|segment| {
                    segment["role"] == json!("field")
                        && segment["field"] == json!(VECTOR_BASE_FIELD)
                        && segment["kind"] == json!(kind)
                })
                .collect();
            refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("vector base ordinal"));
            refs
        }

        fn vector_base_ref<'a>(manifest: &'a Value, role: &str) -> &'a Value {
            vector_base_collection(manifest)["segments"]
                .as_array()
                .expect("vector base catalog segments")
                .iter()
                .find(|segment| {
                    segment["role"] == json!(role)
                        && segment["field"] == json!(VECTOR_BASE_FIELD)
                        && segment["kind"] == json!("base")
                        && segment["ordinal"] == json!(0)
                })
                .unwrap_or_else(|| panic!("vector base catalog needs {role} base reference"))
        }

        fn vector_base_ref_mut<'a>(manifest: &'a mut Value, role: &str) -> &'a mut Value {
            manifest["collections"]
                .as_array_mut()
                .expect("vector base catalog collections")
                .iter_mut()
                .find(|collection| collection["collection_id"] == json!(VECTOR_BASE_COLLECTION))
                .expect("vector base catalog collection")["segments"]
                .as_array_mut()
                .expect("vector base catalog segments")
                .iter_mut()
                .find(|segment| {
                    segment["role"] == json!(role)
                        && segment["field"] == json!(VECTOR_BASE_FIELD)
                        && segment["kind"] == json!("base")
                        && segment["ordinal"] == json!(0)
                })
                .unwrap_or_else(|| {
                    panic!("vector base catalog needs mutable {role} base reference")
                })
        }

        fn vector_base_ref_bytes(generation: &Path, reference: &Value) -> u64 {
            let payload = std::fs::metadata(
                generation.join(
                    reference["path"]
                        .as_str()
                        .expect("vector base segment path"),
                ),
            )
            .expect("inspect vector base segment")
            .len();
            let rows = reference["local_rows"].as_object().map(|rows| {
                std::fs::metadata(
                    generation.join(rows["path"].as_str().expect("vector base local-row path")),
                )
                .expect("inspect vector base local-row map")
                .len()
            });
            rows.map_or(payload, |rows| {
                payload.checked_add(rows).expect("vector base byte sum")
            })
        }

        /// Reproduce the versioned base-payload framing so this test can replace a
        /// vector EID segment with independently valid bytes and a matching
        /// catalog checksum.  The forged sidecar therefore reaches the mapped-row
        /// consistency check instead of failing an earlier integrity check.
        fn vector_base_payload_sha256(path: &Path) -> String {
            let bytes = std::fs::read(path).expect("read forged vector EID sidecar for checksum");
            let mut hasher = Sha256::new();
            hasher.update(b"lumen.base.payload-sha256.v1\0");
            hasher.update((b"segment".len() as u64).to_be_bytes());
            hasher.update(b"segment");
            hasher.update((bytes.len() as u64).to_be_bytes());
            hasher.update(bytes);
            hasher
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect()
        }

        /// Build a second valid vector-EID base through the public Engine and
        /// SegmentRdbStore APIs. It has the compacted checkpoint sequence and
        /// count, while distinct stable IDs make its row mapping disagree with the
        /// original mapped-base local rows.
        async fn vector_base_forged_eid_sidecar(sequence: u64, ids: &[String]) -> Vec<u8> {
            let forge_dir = tempfile::tempdir().expect("forge vector EID sidecar directory");
            let forge_engine = Arc::new(Engine::new());
            let forge_server = TestServer::new(router(AppState::open(forge_engine.clone())))
                .expect("forge vector EID sidecar server");
            vector_base_create_collection(&forge_server, "flat-cpu").await;
            let items = ids
                .iter()
                .enumerate()
                .map(|(row, external_id)| {
                    json!({
                        "external_id": external_id,
                        "field": VECTOR_BASE_FIELD,
                        "value": vector_base_vector(1_000_000.0 + row as f32),
                    })
                })
                .collect();
            vector_base_post_items(&forge_server, items).await;
            let forge_store =
                SegmentRdbStore::new(forge_dir.path()).expect("forge vector EID store");
            forge_store
                .save_required(&forge_engine, sequence)
                .expect("save independently valid same-sequence vector EID base");
            let forge_generation = stage1_current_generation_dir(forge_dir.path());
            let forge_manifest = stage1_read_manifest(&forge_generation);
            let forge_eids = vector_base_ref(&forge_manifest, "vector_eids");
            std::fs::read(
                forge_generation.join(
                    forge_eids["path"]
                        .as_str()
                        .expect("forged vector EID sidecar path"),
                ),
            )
            .expect("read independently valid same-sequence vector EID sidecar")
        }

        fn vector_base_delta_bytes(generation: &Path, manifest: &Value) -> u64 {
            vector_base_field_refs(manifest, "delta")
                .into_iter()
                .map(|reference| vector_base_ref_bytes(generation, reference))
                .sum()
        }

        fn vector_base_complete_base_bytes(generation: &Path, manifest: &Value) -> u64 {
            ["field", "vector_eids"]
                .into_iter()
                .map(|role| vector_base_ref_bytes(generation, vector_base_ref(manifest, role)))
                .sum()
        }

        fn vector_base_mapped_rows_path(generation: &Path, reference: &Value) -> PathBuf {
            let local = reference["local_rows"]
                .as_object()
                .expect("compacted vector base has a local-row descriptor");
            generation.join(
                local["path"]
                    .as_str()
                    .expect("compacted vector base local-row path"),
            )
        }

        fn vector_base_assert_new_mapped_base(
            base_generation: &Path,
            base_manifest: &Value,
            compacted_generation: &Path,
            compacted_manifest: &Value,
        ) {
            let old_field = vector_base_ref(base_manifest, "field");
            let new_field = vector_base_ref(compacted_manifest, "field");
            let local = new_field["local_rows"]
                .as_object()
                .expect("vector base compaction must publish a mapped base");
            assert_eq!(
                local["format"],
                json!("lumen-local-eids-cbor-v1"),
                "mapped vector base must declare the versioned local-row codec"
            );
            let rows = stage1_keyword_delta_read_rows(compacted_generation, new_field);
            assert_eq!(
                local["count"].as_u64(),
                Some(rows.len() as u64),
                "mapped vector base row count must match the decoded local-row map"
            );
            assert!(
            rows.contains(&VECTOR_BASE_APPENDED_ID.to_owned()),
            "mapped vector base must include an ID added after the original collection EID metadata"
        );
            assert!(
                !rows.contains(&VECTOR_BASE_ABSENT_ID.to_owned()),
                "mapped vector base must not invent a row for a document without this vector field"
            );
            assert_eq!(
                new_field["payload_sha256"].as_str().map(str::len),
                Some(64),
                "mapped vector base must checksum its payload and local-row map"
            );

            for role in ["field", "vector_eids"] {
                let old = vector_base_ref(base_manifest, role);
                let new = vector_base_ref(compacted_manifest, role);
                let old_path =
                    base_generation.join(old["path"].as_str().expect("old vector base path"));
                let new_path = compacted_generation
                    .join(new["path"].as_str().expect("compacted vector base path"));
                let old_metadata =
                    std::fs::symlink_metadata(&old_path).expect("inspect old vector base");
                let new_metadata =
                    std::fs::symlink_metadata(&new_path).expect("inspect compacted vector base");
                assert!(old_metadata.is_file() && !old_metadata.file_type().is_symlink());
                assert!(new_metadata.is_file() && !new_metadata.file_type().is_symlink());
                assert_ne!(
                old_metadata.ino(),
                new_metadata.ino(),
                "mapped vector base compaction must rewrite {role} for new or masked vector IDs"
            );
            }
        }

        fn vector_base_assert_mapped_base_hardlinked(
            prior_generation: &Path,
            prior_manifest: &Value,
            latest_generation: &Path,
            latest_manifest: &Value,
        ) {
            let prior_field = vector_base_ref(prior_manifest, "field");
            let latest_field = vector_base_ref(latest_manifest, "field");
            let paths = [
                (
                    prior_generation.join(
                        prior_field["path"]
                            .as_str()
                            .expect("prior mapped vector payload"),
                    ),
                    latest_generation.join(
                        latest_field["path"]
                            .as_str()
                            .expect("latest mapped vector payload"),
                    ),
                    "mapped vector payload",
                ),
                (
                    vector_base_mapped_rows_path(prior_generation, prior_field),
                    vector_base_mapped_rows_path(latest_generation, latest_field),
                    "mapped vector local-row map",
                ),
                (
                    prior_generation.join(
                        vector_base_ref(prior_manifest, "vector_eids")["path"]
                            .as_str()
                            .expect("prior vector EID sidecar"),
                    ),
                    latest_generation.join(
                        vector_base_ref(latest_manifest, "vector_eids")["path"]
                            .as_str()
                            .expect("latest vector EID sidecar"),
                    ),
                    "vector EID sidecar",
                ),
            ];
            for (prior, latest, label) in paths {
                let old =
                    std::fs::symlink_metadata(&prior).expect("inspect prior mapped base file");
                let new =
                    std::fs::symlink_metadata(&latest).expect("inspect latest mapped base file");
                assert!(old.is_file() && !old.file_type().is_symlink());
                assert!(new.is_file() && !new.file_type().is_symlink());
                assert_eq!(
                    old.ino(),
                    new.ino(),
                    "unchanged {label} must be hard linked into the next generation"
                );
                assert!(
                    new.nlink() >= 2,
                    "hard-linked {label} must retain multiple links"
                );
            }
        }

        fn vector_base_live_ids() -> std::collections::BTreeSet<String> {
            (0..VECTOR_BASE_ROWS)
                .map(vector_base_id)
                .chain(std::iter::once(VECTOR_BASE_HOT_ID.to_owned()))
                .chain(std::iter::once(VECTOR_BASE_DELETED_ID.to_owned()))
                .collect()
        }

        fn vector_base_current_ids() -> std::collections::BTreeSet<String> {
            let mut ids = vector_base_live_ids();
            ids.remove(VECTOR_BASE_DELETED_ID);
            ids.insert(VECTOR_BASE_APPENDED_ID.to_owned());
            ids
        }

        async fn vector_base_knn_ids(server: &TestServer, x: f32, k: usize) -> Vec<String> {
            let response = server
                .post(&format!("/collections/{VECTOR_BASE_COLLECTION}/search"))
                .json(&json!({
                    "query": { "knn": {
                        "field": VECTOR_BASE_FIELD,
                        "vector": vector_base_vector(x),
                        "k": k,
                    }},
                    "limit": k,
                    "track_total": true,
                }))
                .await;
            response.assert_status_ok();
            let body: Value = response.json();
            body["hits"]
                .as_array()
                .expect("vector base kNN hits")
                .iter()
                .map(|hit| {
                    hit["external_id"]
                        .as_str()
                        .expect("vector base kNN external ID")
                        .to_owned()
                })
                .collect()
        }

        async fn vector_base_assert_backend(server: &TestServer, backend: &str, phase: &str) {
            let response = server.get("/admin/backup").await;
            response.assert_status_ok();
            let snapshot: Value = response.json();
            assert_eq!(
                snapshot["collections"][VECTOR_BASE_COLLECTION]["fields"][VECTOR_BASE_FIELD]
                    ["spec"]["backend"],
                json!(backend),
                "{phase}: vector backend schema must survive live and cold states"
            );
        }

        async fn vector_base_assert_ids(
            server: &TestServer,
            expected: std::collections::BTreeSet<String>,
            phase: &str,
        ) {
            let ids = vector_base_knn_ids(server, 0.0, 256).await;
            let unique: std::collections::BTreeSet<_> = ids.iter().cloned().collect();
            assert_eq!(
                unique.len(),
                ids.len(),
                "{phase}: vector search must not duplicate external IDs"
            );
            assert_eq!(
                unique, expected,
                "{phase}: vector search must expose exact live IDs"
            );
            assert!(
                !ids.iter().any(|id| id == VECTOR_BASE_ABSENT_ID),
                "{phase}: a document without a base vector must not become searchable"
            );
        }

        async fn vector_base_assert_nearest(
            server: &TestServer,
            x: f32,
            expected: &str,
            phase: &str,
        ) {
            assert_eq!(
                vector_base_knn_ids(server, x, 1).await,
                vec![expected.to_owned()],
                "{phase}: exact vector query must return the newest expected external ID"
            );
        }

        async fn vector_base_assert_base_state(server: &TestServer, backend: &str, phase: &str) {
            vector_base_assert_backend(server, backend, phase).await;
            vector_base_assert_ids(server, vector_base_live_ids(), phase).await;
            vector_base_assert_nearest(server, vector_base_initial_x(0), &vector_base_id(0), phase)
                .await;
            vector_base_assert_nearest(server, vector_base_hot_x(0), VECTOR_BASE_HOT_ID, phase)
                .await;
            vector_base_assert_nearest(server, 500.0, VECTOR_BASE_DELETED_ID, phase).await;
        }

        async fn vector_base_assert_round_state(
            server: &TestServer,
            backend: &str,
            round: usize,
            phase: &str,
        ) {
            vector_base_assert_backend(server, backend, phase).await;
            vector_base_assert_ids(server, vector_base_current_ids(), phase).await;
            assert!(
                !vector_base_knn_ids(server, 0.0, 256)
                    .await
                    .iter()
                    .any(|id| id == VECTOR_BASE_DELETED_ID),
                "{phase}: a deleted base vector must stay masked"
            );
            vector_base_assert_nearest(
                server,
                vector_base_round_x(round, 0),
                &vector_base_id(0),
                phase,
            )
            .await;
            vector_base_assert_nearest(
                server,
                vector_base_round_x(round, VECTOR_BASE_ROWS - 1),
                &vector_base_id(VECTOR_BASE_ROWS - 1),
                phase,
            )
            .await;
            vector_base_assert_nearest(server, vector_base_hot_x(round), VECTOR_BASE_HOT_ID, phase)
                .await;
            vector_base_assert_nearest(
                server,
                vector_base_appended_x(round),
                VECTOR_BASE_APPENDED_ID,
                phase,
            )
            .await;
        }

        async fn vector_base_assert_newer_state(server: &TestServer, backend: &str, phase: &str) {
            vector_base_assert_backend(server, backend, phase).await;
            vector_base_assert_ids(server, vector_base_current_ids(), phase).await;
            vector_base_assert_nearest(
                server,
                vector_base_round_x(4, 0),
                &vector_base_id(0),
                phase,
            )
            .await;
            vector_base_assert_nearest(server, vector_base_hot_x(4), VECTOR_BASE_HOT_ID, phase)
                .await;
            vector_base_assert_nearest(
                server,
                vector_base_appended_x(5),
                VECTOR_BASE_APPENDED_ID,
                phase,
            )
            .await;
        }

        async fn vector_base_publish_four_checkpoints(
            fixture: &VectorBaseFixture,
        ) -> (String, String) {
            let base_generation = fixture.root.join(&fixture.base_name);
            let base_manifest = stage1_read_manifest(&base_generation);
            for round in 1..=3 {
                vector_base_apply_round(&fixture.server, round).await;
                fixture
                    .store
                    .save(&fixture.engine, VECTOR_BASE_SEQUENCE + round as u64)
                    .expect("publish vector base-eligible delta checkpoint");
            }
            let third_name = stage1_reuse_current_name(&fixture.root);
            let third_generation = fixture.root.join(&third_name);
            let third_manifest = stage1_read_manifest(&third_generation);
            let third_deltas = vector_base_field_refs(&third_manifest, "delta");
            assert_eq!(
                third_deltas.len(),
                3,
                "the fourth checkpoint alone must request vector base compaction"
            );
            stage1_assert_delta_sequence_order(
                &third_deltas,
                &[
                    VECTOR_BASE_SEQUENCE + 1,
                    VECTOR_BASE_SEQUENCE + 2,
                    VECTOR_BASE_SEQUENCE + 3,
                ],
                "vector base pre-merge layers",
            );
            let base_bytes = vector_base_complete_base_bytes(&base_generation, &base_manifest);
            let delta_bytes = vector_base_delta_bytes(&third_generation, &third_manifest);
            assert!(
            delta_bytes >= base_bytes,
            "fixture precondition: three actual vector delta-plus-row-map bytes ({delta_bytes}) must meet or exceed the complete vector base-plus-EID-sidecar bytes ({base_bytes}) before the fourth checkpoint"
        );

            vector_base_apply_round(&fixture.server, 4).await;
            let publication = fixture
                .store
                .save(&fixture.engine, VECTOR_BASE_SEQUENCE + 4);
            assert!(
            publication.is_ok(),
            "base-eligible vector publication must preserve the deleted base-vector tombstone and publish a mapped base: {publication:?}"
        );
            fixture
                .store
                .wait_for_merges(Duration::from_secs(30))
                .expect("wait for base-eligible vector compaction");
            let compacted_name = stage1_reuse_current_name(&fixture.root);
            let compacted_generation = fixture.root.join(&compacted_name);
            let compacted_manifest = stage1_read_manifest(&compacted_generation);
            assert!(
                vector_base_field_refs(&compacted_manifest, "delta").is_empty(),
                "base-eligible vector publication must consume all four captured vector deltas"
            );
            vector_base_assert_new_mapped_base(
                &base_generation,
                &base_manifest,
                &compacted_generation,
                &compacted_manifest,
            );
            (third_name, compacted_name)
        }

        async fn vector_base_behavior_contract(backend: &str) {
            let fixture = vector_base_fixture(backend).await;
            vector_base_assert_base_state(&fixture.server, backend, "live original base").await;
            let (third_name, compacted_name) = vector_base_publish_four_checkpoints(&fixture).await;
            vector_base_assert_round_state(&fixture.server, backend, 4, "live mapped base").await;

            let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
            assert_eq!(
                cold_sequence,
                VECTOR_BASE_SEQUENCE + 4,
                "cold CURRENT carries the mapped-base checkpoint sequence"
            );
            let cold_server = TestServer::new(router(AppState::open(cold_engine.clone())))
                .expect("cold mapped vector base server");
            vector_base_assert_round_state(&cold_server, backend, 4, "cold mapped base").await;

            vector_base_apply_newer_update(&cold_server).await;
            SegmentRdbStore::new(&fixture.root)
                .expect("reopen store after cold mapped-base update")
                .save(&cold_engine, VECTOR_BASE_SEQUENCE + 5)
                .expect("publish newer vector delta above mapped base");
            let latest_generation = stage1_current_generation_dir(&fixture.root);
            let latest_manifest = stage1_read_manifest(&latest_generation);
            let latest_deltas = vector_base_field_refs(&latest_manifest, "delta");
            assert_eq!(
                latest_deltas.len(),
                1,
                "newer vector update remains a single delta above the mapped base"
            );
            stage1_assert_delta_sequence_order(
                &latest_deltas,
                &[VECTOR_BASE_SEQUENCE + 5],
                "newer vector delta above mapped base",
            );
            let compacted_generation = fixture.root.join(&compacted_name);
            let compacted_manifest = stage1_read_manifest(&compacted_generation);
            vector_base_assert_mapped_base_hardlinked(
                &compacted_generation,
                &compacted_manifest,
                &latest_generation,
                &latest_manifest,
            );
            vector_base_assert_newer_state(&cold_server, backend, "live newer mapped-base update")
                .await;

            let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
            assert_eq!(
                latest_sequence,
                VECTOR_BASE_SEQUENCE + 5,
                "cold latest generation carries the newer vector update"
            );
            let latest_server = TestServer::new(router(AppState::open(latest_engine)))
                .expect("cold newer mapped vector base server");
            vector_base_assert_newer_state(
                &latest_server,
                backend,
                "cold newer mapped-base update",
            )
            .await;

            let (third_engine, third_sequence) =
                stage1_reuse_cold_load_named(&fixture.root, &third_name);
            assert_eq!(
                third_sequence,
                VECTOR_BASE_SEQUENCE + 3,
                "retained third vector generation keeps its sequence"
            );
            let third_server = TestServer::new(router(AppState::open(third_engine)))
                .expect("retained third vector server");
            vector_base_assert_round_state(
                &third_server,
                backend,
                3,
                "retained third vector generation",
            )
            .await;

            let (compacted_engine, compacted_sequence) =
                stage1_reuse_cold_load_named(&fixture.root, &compacted_name);
            assert_eq!(
                compacted_sequence,
                VECTOR_BASE_SEQUENCE + 4,
                "retained mapped vector base keeps its sequence"
            );
            let compacted_server = TestServer::new(router(AppState::open(compacted_engine)))
                .expect("retained mapped vector base server");
            vector_base_assert_round_state(
                &compacted_server,
                backend,
                4,
                "retained mapped vector base",
            )
            .await;

            let (base_engine, base_sequence) =
                stage1_reuse_cold_load_named(&fixture.root, &fixture.base_name);
            assert_eq!(
                base_sequence, VECTOR_BASE_SEQUENCE,
                "retained original vector base keeps its sequence"
            );
            let base_server = TestServer::new(router(AppState::open(base_engine)))
                .expect("retained original vector base server");
            vector_base_assert_base_state(&base_server, backend, "retained original vector base")
                .await;
        }

        fn vector_base_assert_current_refuses_mapped_base(root: &Path, mutation: &str) {
            let current_before = std::fs::read(root.join("CURRENT"))
                .expect("read CURRENT before mapped-base refusal");
            let opened = SegmentRdbStore::new(root)
                .expect("reopen mapped-base checkpoint root")
                .load_current_generation();
            assert!(
            opened.is_err(),
            "CURRENT must explicitly refuse {mutation}, not open a predecessor or malformed mapped base"
        );
            assert_eq!(
                std::fs::read(root.join("CURRENT"))
                    .expect("read CURRENT after mapped-base refusal"),
                current_before,
                "refusing {mutation} must leave CURRENT unchanged"
            );
        }

        #[tokio::test]
        async fn v2_flat_cpu_vector_base_compaction_maps_new_ids_live_cold_and_retained() {
            vector_base_behavior_contract("flat-cpu").await;
        }

        #[tokio::test]
        async fn v2_hnsw_cpu_vector_base_compaction_maps_new_ids_live_cold_and_retained() {
            vector_base_behavior_contract("hnsw-cpu").await;
        }

        #[tokio::test]
        async fn v2_current_refuses_malformed_mapped_flat_vector_base_without_fallback() {
            let fixture = vector_base_fixture("flat-cpu").await;
            let (_, compacted_name) = vector_base_publish_four_checkpoints(&fixture).await;
            let generation = fixture.root.join(compacted_name);
            let original_manifest = stage1_read_manifest(&generation);
            let original_rows_path = vector_base_mapped_rows_path(
                &generation,
                vector_base_ref(&original_manifest, "field"),
            );
            let original_rows =
                std::fs::read(&original_rows_path).expect("read mapped vector rows");
            let original_checksum =
                vector_base_ref(&original_manifest, "field")["payload_sha256"].clone();

            let mut bad_format = original_manifest.clone();
            let local = vector_base_ref_mut(&mut bad_format, "field")["local_rows"]
                .as_object_mut()
                .expect("mapped vector base local rows");
            local.insert("format".to_owned(), json!("untrusted-local-row-format"));
            assert_eq!(
                vector_base_ref(&bad_format, "field")["payload_sha256"],
                original_checksum,
                "format corruption leaves the valid mapped payload checksum in place"
            );
            stage1_write_manifest(&generation, &bad_format);
            vector_base_assert_current_refuses_mapped_base(
                &fixture.root,
                "an unknown mapped local-row format",
            );
            stage1_write_manifest(&generation, &original_manifest);

            let mut bad_count = original_manifest.clone();
            let local = vector_base_ref_mut(&mut bad_count, "field")["local_rows"]
                .as_object_mut()
                .expect("mapped vector base local rows");
            let count = local["count"].as_u64().expect("mapped vector row count");
            local.insert("count".to_owned(), json!(count + 1));
            assert_eq!(
                vector_base_ref(&bad_count, "field")["payload_sha256"],
                original_checksum,
                "count corruption leaves the valid mapped payload checksum in place"
            );
            stage1_write_manifest(&generation, &bad_count);
            vector_base_assert_current_refuses_mapped_base(
                &fixture.root,
                "a mapped local-row count mismatch",
            );
            stage1_write_manifest(&generation, &original_manifest);

            std::fs::remove_file(&original_rows_path).expect("remove mapped vector local-row map");
            vector_base_assert_current_refuses_mapped_base(
                &fixture.root,
                "a missing mapped local-row file",
            );
            std::fs::write(&original_rows_path, &original_rows)
                .expect("restore mapped vector local-row map");

            // This rewrites the compacted vector-EID sidecar with a separately
            // encoded, same-sequence segment whose IDs have the same count but a
            // different order. Its payload checksum is recomputed in the catalog.
            // The field payload and its mapped rows remain valid, so refusal must
            // reach the named vector-map consistency check.
            let original_eids = vector_base_ref(&original_manifest, "vector_eids");
            let original_eids_path = generation.join(
                original_eids["path"]
                    .as_str()
                    .expect("mapped vector EID sidecar path"),
            );
            let compacted_eids = std::fs::read(&original_eids_path)
                .expect("read mapped vector EID sidecar before mismatch mutation");
            let compacted_sequence = original_eids["applied_seq"]
                .as_u64()
                .expect("compacted vector EID sidecar sequence");
            let mapped_rows = stage1_keyword_delta_read_rows(
                &generation,
                vector_base_ref(&original_manifest, "field"),
            );
            assert_eq!(
                mapped_rows.len(),
                vector_base_ref(&original_manifest, "field")["local_rows"]["count"]
                    .as_u64()
                    .expect("mapped vector row count") as usize,
                "mapped vector rows must match their catalogued count before the mismatch mutation"
            );
            // FrozenField::capture sorts vector rows by external ID. A rotation of
            // the same IDs would therefore be normalized back to the valid order.
            // Use a distinct, zero-padded set instead: it retains the count and
            // gives the forged EID segment a deterministic row order that cannot
            // agree with the original mapped-base local rows.
            assert!(
                !mapped_rows.is_empty(),
                "mismatch fixture needs at least one compacted vector row"
            );
            let forged_ids: Vec<_> = mapped_rows
                .iter()
                .enumerate()
                .map(|(row, id)| format!("forged-mapped-vector-{row:06}-{id}"))
                .collect();
            assert_ne!(
                forged_ids, mapped_rows,
                "the valid forged vector EID sidecar must use distinct stable IDs"
            );
            let forged_eids = vector_base_forged_eid_sidecar(compacted_sequence, &forged_ids).await;
            assert_ne!(
            forged_eids, compacted_eids,
            "the sidecar mismatch fixture must replace bytes, not merely rewrite the same vector IDs"
        );
            std::fs::write(&original_eids_path, &forged_eids)
                .expect("install independently valid same-sequence vector EID sidecar");
            let mut bad_eid_sidecar = original_manifest.clone();
            vector_base_ref_mut(&mut bad_eid_sidecar, "vector_eids")["payload_sha256"] =
                json!(vector_base_payload_sha256(&original_eids_path));
            stage1_write_manifest(&generation, &bad_eid_sidecar);
            let current_before_mismatch = std::fs::read(fixture.root.join("CURRENT"))
                .expect("read CURRENT before vector map mismatch");
            let mismatch = match SegmentRdbStore::new(&fixture.root)
            .expect("reopen mapped-base root for vector map mismatch")
            .load_current_generation()
        {
            Ok(_) => panic!("CURRENT must refuse a checksum-valid vector EID sidecar that disagrees with mapped rows"),
            Err(error) => error,
        };
            assert!(
            format!("{mismatch:#}").contains("mapped vector row map differs from EID sidecar"),
            "a valid same-sequence vector EID sidecar with permuted stable IDs must reach the mapped-vector row mismatch refusal, got: {mismatch:#}"
        );
            assert_eq!(
                std::fs::read(fixture.root.join("CURRENT"))
                    .expect("read CURRENT after vector map mismatch"),
                current_before_mismatch,
                "refusing a checksum-valid mapped-vector row mismatch must leave CURRENT unchanged"
            );
            std::fs::write(&original_eids_path, &compacted_eids)
                .expect("restore mapped vector EID sidecar");
            stage1_write_manifest(&generation, &original_manifest);

            let mut bad_checksum = original_manifest.clone();
            vector_base_ref_mut(&mut bad_checksum, "field")["payload_sha256"] =
                json!("0".repeat(64));
            stage1_write_manifest(&generation, &bad_checksum);
            vector_base_assert_current_refuses_mapped_base(
                &fixture.root,
                "a mapped-base payload checksum mismatch",
            );
            stage1_write_manifest(&generation, &original_manifest);

            assert!(
                SegmentRdbStore::new(&fixture.root)
                    .expect("reopen restored mapped-base root")
                    .load_current_generation()
                    .is_ok(),
                "restoring exact mapped-base bytes must restore a cold-loadable CURRENT"
            );
        }
    }
}

// A committed `Index` record can return an error after changing the engine.
// These cases use the real coordinator/AOF/checkpoint route rather than a
// local Engine shortcut, because recovery consumes the persisted `RaftLogEntry`
// rather than an already-materialized index.
fn partial_error_index_entry(items: Vec<Value>) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: serde_json::from_value(json!({ "items": items })).expect("partial-error index entry"),
    }
}

fn partial_error_term_ids(engine: &Arc<Engine>, field: &str, value: FieldValue) -> Vec<String> {
    sorted_hit_ids(
        engine
            .search(
                COLLECTION,
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: field.to_owned(),
                        value,
                    }),
                    limit: 20,
                    offset: 0,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .expect("partial-error term query"),
    )
}

#[tokio::test]
async fn partial_error_mixed_index_replays_its_live_keyword_prefix_from_aof() {
    let fixture = fixture();
    create_schema(&fixture.server).await;
    post_index(
        &fixture.server,
        vec![json!({
            "external_id": "partial-error-baseline",
            "field": "kw",
            "value": "baseline",
        })],
    )
    .await;
    checkpoint(&fixture.server).await;
    let checkpoint_sequence = fixture.writer.applied_seq();

    let error = fixture
        .writer
        .submit(partial_error_index_entry(vec![
            json!({
                "external_id": "partial-error-prefix",
                "field": "kw",
                "value": "survives-error",
            }),
            json!({
                "external_id": "partial-error-prefix",
                "field": "unknown-after-prefix",
                "value": "must-error",
            }),
        ]))
        .await
        .expect_err("mixed index must report the unknown field after applying its valid prefix");
    assert!(
        format!("{error:#}").contains("unknown field `unknown-after-prefix`"),
        "the public write must retain its existing unknown-field error, got: {error:#}"
    );
    let committed_sequence = fixture.writer.applied_seq();
    assert_eq!(
        committed_sequence,
        checkpoint_sequence + 1,
        "the error return belongs to one concrete committed record"
    );

    let live = http_search(
        &fixture.server,
        json!({ "term": { "field": "kw", "value": "survives-error" } }),
        "live mixed-index prefix",
    )
    .await;
    assert_eq!(
        hit_ids(&live),
        vec!["partial-error-prefix"],
        "the valid prefix is visible even though the same committed record returned an error"
    );

    fixture
        .aof
        .lock()
        .expect("AOF lock")
        .sync_strict()
        .expect("strict-sync AOF tail");
    let (replayed_engine, recovered_checkpoint_sequence, replayed_sequence) =
        recover_from_checkpoint(&fixture);
    assert_eq!(
        recovered_checkpoint_sequence, checkpoint_sequence,
        "recovery starts from the published base before the partial-error record"
    );
    assert_eq!(
        replayed_sequence, committed_sequence,
        "a committed record with a visible valid prefix must be retained in the AOF tail"
    );
    assert_eq!(
        partial_error_term_ids(
            &replayed_engine,
            "kw",
            FieldValue::String("survives-error".to_owned()),
        ),
        vec!["partial-error-prefix".to_owned()],
        "AOF replay must restore the live valid prefix from an error-returning record"
    );

    checkpoint(&fixture.server).await;
    let (checkpointed_engine, checkpointed_sequence, checkpointed_tail) =
        recover_from_checkpoint(&fixture);
    assert_eq!(
        checkpointed_sequence, committed_sequence,
        "the incremental checkpoint must publish through the partial-error record"
    );
    assert_eq!(
        checkpointed_tail, 0,
        "the checkpointed partial-error record has no remaining AOF tail"
    );
    assert_eq!(
        partial_error_term_ids(
            &checkpointed_engine,
            "kw",
            FieldValue::String("survives-error".to_owned()),
        ),
        vec!["partial-error-prefix".to_owned()],
        "a cold incremental checkpoint must keep the prefix that live reads exposed"
    );
}

#[tokio::test]
async fn partial_error_number_reindex_checkpoint_never_resurrects_the_dropped_value() {
    let fixture = fixture();
    create_schema(&fixture.server).await;
    post_index(
        &fixture.server,
        vec![json!({
            "external_id": "partial-error-number",
            "field": "num",
            "value": 41.0,
        })],
    )
    .await;
    checkpoint(&fixture.server).await;
    let checkpoint_sequence = fixture.writer.applied_seq();

    let error = fixture
        .writer
        .submit(partial_error_index_entry(vec![json!({
            "external_id": "partial-error-number",
            "field": "num",
            "value": "not-a-number",
        })]))
        .await
        .expect_err("a Number field must reject a string replacement");
    assert!(
        format!("{error:#}").contains("type mismatch on field `num`"),
        "the public write must retain its existing Number type-mismatch error, got: {error:#}"
    );
    let committed_sequence = fixture.writer.applied_seq();
    assert_eq!(
        committed_sequence,
        checkpoint_sequence + 1,
        "the error return belongs to one concrete committed Number record"
    );

    let live = http_search(
        &fixture.server,
        json!({ "term": { "field": "num", "value": 41.0 } }),
        "live Number after invalid reindex",
    )
    .await;
    assert!(
        hit_ids(&live).is_empty(),
        "the existing partial-error semantics drop the old Number value before returning its error"
    );

    checkpoint(&fixture.server).await;
    let (cold_engine, cold_sequence, replayed_tail) = recover_from_checkpoint(&fixture);
    assert_eq!(
        cold_sequence, committed_sequence,
        "the incremental checkpoint must cover the error-returning Number record"
    );
    assert_eq!(
        replayed_tail, 0,
        "the current generation must contain this cut without an uncheckpointed tail"
    );
    assert!(
        partial_error_term_ids(&cold_engine, "num", FieldValue::Number(41.0)).is_empty(),
        "a cold incremental checkpoint must not resurrect the Number value that live reads lost"
    );
}

mod capacity_http_contract {
    //! # Facets
    //!
    //! - Behavior: `indexing_durable_oracle.rs:10600`, `:10620`, `:10640`, and
    //!   `:10669` require a real `POST /collections/{id}/index` pre-submit refusal,
    //!   successful retry after publication, and live plus cold query recovery. Change points:
    //!   `apps/lumen/src/coordinator.rs:566-621`,
    //!   `apps/lumen/src/segment_checkpoint.rs:171-217`, and
    //!   `apps/lumen/src/api.rs:1554-1586`.
    //! - Security: `indexing_durable_oracle.rs:10603`, `:10608`, and `:10612` feed
    //!   caller-controlled large `/index` bytes and require the closed `429`, exact
    //!   `Retry-After: 1`, and unchanged MemWal/applied sequences. Change point:
    //!   `apps/lumen/src/api.rs:3269-3273` maps the pre-publication capacity error
    //!   to the HTTP boundary.
    //! - Performance: `apps/lumen/ROADMAP.md:60-70` promises a 256 MiB pending
    //!   active/frozen/reserved budget and checkpoint progress at that limit.
    //!   `indexing_durable_oracle.rs:10431-10435` pins the documented limit and
    //!   `:10593-10597` refuses a missing-429 result only after the actual raw payload
    //!   crosses it. This structural case makes no latency or RSS claim; the
    //!   30-minute workload remains the gate for those measurements.
    //! - Behavior (externally committed record): `indexing_durable_oracle.rs:11584`,
    //!   `:11592`, `:11600`, and `:11650` pin direct WAL publication, stationary
    //!   applied state while capacity is held, later application, and cold recovery.
    //!   Change points: `apps/lumen/src/coordinator.rs:297-358`
    //!   and `apps/lumen/src/segment_checkpoint.rs:172-217`.
    //! - Security (capacity boundary): `indexing_durable_oracle.rs:11530`, `:11536`,
    //!   `:11564`, and `:11592` require rejected caller-controlled input to leave WAL
    //!   and the applied watermark unchanged, and a committed record to stay invisible
    //!   before admission owns it.
    //! - Performance (same approved budget): `apps/lumen/ROADMAP.md:60-70`;
    //!   `indexing_durable_oracle.rs:10773-10803` bounds real public accounting and
    //!   `:10813-10868` establishes a staged public capacity witness without
    //!   claiming latency or RSS.

    use super::*;

    use axum::http::{header::RETRY_AFTER, StatusCode};
    use lumen::segment_checkpoint::{PendingChangeSpill, SegmentCheckpointSink};
    use lumen::segment_rdb::{MergeObserver, MergePhase};
    use std::io;
    use std::sync::mpsc;
    use storage_durable::{CommitStep, FailureInjector, FailurePoint};

    const CAPACITY_COLLECTION: &str = "pending-capacity";
    // One 6 MiB record is safely below the documented 256 MiB limit even when
    // admission conservatively reserves its future capture representation.
    const FROZEN_VALUE_COUNT: usize = 1;
    // This stays below the current 8 MiB HTTP limit even after JSON framing.
    const LARGE_KEYWORD_VALUE_BYTES: usize = 6 * 1024 * 1024 - 32 * 1024;
    const MAX_HTTP_BODY_BYTES: usize = 8 * 1024 * 1024;
    const PENDING_HARD_LIMIT_BYTES: usize = 256 * 1024 * 1024;
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
    // Local admission keeps the owned record, four AOF copies, and its
    // checkpointable Keyword state before publication. Start with 1 MiB rows
    // to fill quickly, then use 64 KiB rows if the first local price no longer
    // fits even though the exact 6 MiB committed value still can. A new
    // Keyword's 64 KiB value has only fixed metadata beyond its two retained
    // state copies, so that stage fits throughout the remaining <6 MiB gap
    // without mirroring the private admission estimator in this oracle.
    const FILLER_VALUE_STAGES: [usize; 2] = [1024 * 1024, 64 * 1024];
    const MAX_FILLER_VALUE_COUNT_PER_STAGE: usize = 256;
    const FILLER_RETRY_DELAY: Duration = Duration::from_millis(30);
    const MAX_FILLER_RETRIES_PER_ORDINAL: usize = 32;
    const MAX_WITNESS_SCRAPE_RETRIES: usize = 8;
    const FILL_SETUP_TIMEOUT: Duration = Duration::from_secs(30);
    // With one frozen value, this gives the active side enough unique requests
    // to cross the actual 256 MiB payload limit if the implementation never
    // applies admission. It is a finite ~270 MiB source-payload fixture.
    const MAX_LARGE_VALUE_COUNT: usize = 43;

    #[derive(Default)]
    struct NoopMergeObserver;

    impl MergeObserver for NoopMergeObserver {
        fn observe(&self, _: MergePhase) -> io::Result<()> {
            Ok(())
        }
    }

    /// Pauses the first real generation-file sync after it has captured the
    /// frozen layer. `SegmentCheckpointSink` performs this save in
    /// `spawn_blocking`, so blocking this injector never starves the Tokio
    /// workers that drive HTTP and release it.
    #[derive(Default)]
    struct HoldNextSyncFile {
        hold: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
    }

    impl HoldNextSyncFile {
        fn arm(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
            assert!(
                self.hold
                    .lock()
                    .expect("capacity sync hold mutex")
                    .replace((entered, release))
                    .is_none(),
                "capacity test arms exactly one checkpoint sync hold",
            );
        }
    }

    impl FailureInjector for HoldNextSyncFile {
        fn check(&self, point: &FailurePoint) -> io::Result<()> {
            if point.step != CommitStep::SyncFile {
                return Ok(());
            }
            let Some((entered, release)) =
                self.hold.lock().expect("capacity sync hold mutex").take()
            else {
                return Ok(());
            };
            entered.send(()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "capacity checkpoint readiness receiver dropped",
                )
            })?;
            // The test owns this pause until `SyncRelease` sends. A timeout here
            // would let a still-running HTTP loop publish behind its back.
            release.recv().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "capacity checkpoint release sender dropped",
                )
            })?;
            Ok(())
        }
    }

    /// Guarantees that the real checkpoint's blocking sync is released even
    /// when a later HTTP assertion fails.
    struct SyncRelease(Option<mpsc::SyncSender<()>>);

    impl SyncRelease {
        fn release(&mut self) {
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }

    impl Drop for SyncRelease {
        fn drop(&mut self) {
            self.release();
        }
    }

    struct CapacityFixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        server: TestServer,
        store: Arc<SegmentRdbStore>,
        checkpoint: Arc<SegmentCheckpointSink>,
        writer: Arc<WriteCoordinator>,
        wal: Arc<MemWal>,
        hold: Arc<HoldNextSyncFile>,
        // The externally committed Full path must reuse this root. Otherwise
        // it can create a temporary spill store whose publication bypasses the
        // held configured-root SyncFile in this contract.
        _configured_capacity_owner: Option<PendingChangeSpill>,
    }

    fn capacity_fixture(configured_capacity_owner: bool) -> CapacityFixture {
        let dir = tempfile::tempdir().expect("capacity fixture directory");
        let root = dir.path().join("segments");
        let hold = Arc::new(HoldNextSyncFile::default());
        let store = Arc::new(
            SegmentRdbStore::with_failure_injector_and_merge_observer(
                &root,
                hold.clone(),
                Arc::new(NoopMergeObserver),
            )
            .expect("open observed capacity segment store"),
        );
        let aof: SharedAof = Arc::new(Mutex::new(
            AofWriter::open(dir.path().join("aof.log")).expect("open capacity AOF"),
        ));
        let engine = Arc::new(Engine::new());
        let configured_capacity_owner = configured_capacity_owner.then(|| {
            PendingChangeSpill::configured_replay(
                engine.clone(),
                store.clone(),
                Duration::from_secs(3600),
            )
        });
        let wal = Arc::new(MemWal::new());
        let shared_wal: SharedWal = wal.clone();
        let writer =
            WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
        let sink_writer: Arc<dyn WriteSink> = writer.clone();
        let checkpoint = Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: sink_writer.clone(),
            aof: Some(aof),
        });
        let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
        let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink_writer)
            .with_checkpoint(checkpoint_api);
        let server = TestServer::new(router(state)).expect("capacity HTTP server");
        CapacityFixture {
            _dir: dir,
            root,
            server,
            store,
            checkpoint,
            writer,
            wal,
            hold,
            _configured_capacity_owner: configured_capacity_owner,
        }
    }

    fn capacity_external_id(ordinal: usize) -> String {
        format!("capacity-large-{ordinal:03}")
    }

    #[derive(Clone, Copy, Debug)]
    struct FillerRecord {
        stage: usize,
        ordinal: usize,
        value_bytes: usize,
    }

    fn capacity_filler_external_id(record: FillerRecord) -> String {
        format!("capacity-filler-{:02}-{:03}", record.stage, record.ordinal)
    }

    /// Each ordinal owns one unique `String` allocation. These are neither
    /// duplicate values nor overwrites, so the fixture never inflates its raw
    /// pending-byte count by resubmitting one field.
    fn capacity_payload(label: &str, ordinal: usize, bytes_len: usize) -> String {
        let mut bytes = vec![b'a' + (ordinal % 26) as u8; bytes_len];
        for offset in (0..bytes.len()).step_by(4096) {
            bytes[offset] = b'A' + ((ordinal + offset / 4096) % 26) as u8;
        }
        let prefix = format!("{label}-{ordinal:03}-");
        assert!(
            prefix.len() <= bytes.len(),
            "capacity fixture prefix must fit its legal Keyword value",
        );
        bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
        String::from_utf8(bytes).expect("ASCII capacity payload")
    }

    fn capacity_value(ordinal: usize) -> String {
        capacity_payload("capacity-payload", ordinal, LARGE_KEYWORD_VALUE_BYTES)
    }

    fn capacity_filler_value(record: FillerRecord) -> String {
        capacity_payload(
            &format!("capacity-filler-{:02}", record.stage),
            record.ordinal,
            record.value_bytes,
        )
    }

    fn capacity_index_request(ordinal: usize) -> Value {
        let value = capacity_value(ordinal);
        assert_eq!(
            value.len(),
            LARGE_KEYWORD_VALUE_BYTES,
            "each capacity item must carry the counted retained Keyword bytes",
        );
        json!({
            "items": [{
                "external_id": capacity_external_id(ordinal),
                "field": "kw",
                "value": value,
            }]
        })
    }

    fn capacity_filler_index_request(record: FillerRecord) -> Value {
        let value = capacity_filler_value(record);
        assert_eq!(
            value.len(),
            record.value_bytes,
            "each filler item must carry its distinct retained Keyword bytes",
        );
        json!({
            "items": [{
                "external_id": capacity_filler_external_id(record),
                "field": "kw",
                "value": value,
            }]
        })
    }

    async fn post_capacity_index(
        server: &TestServer,
        request: Value,
        label: &str,
    ) -> (StatusCode, Option<String>) {
        assert_eq!(
            request["items"].as_array().map(Vec::len),
            Some(1),
            "{label} uses one legal field item per /index request",
        );
        let body = serde_json::to_vec(&request).expect("serialize capacity index body");
        assert!(
            body.len() < MAX_HTTP_BODY_BYTES,
            "{label} must stay below the current 8 MiB HTTP body limit: {} bytes",
            body.len(),
        );
        let response = server
            .post(&format!("/collections/{CAPACITY_COLLECTION}/index"))
            .json(&request)
            .await;
        let retry_after = response
            .maybe_header(RETRY_AFTER)
            .and_then(|value| value.to_str().ok().map(ToOwned::to_owned));
        (response.status_code(), retry_after)
    }

    async fn capacity_index(server: &TestServer, ordinal: usize) -> (StatusCode, Option<String>) {
        post_capacity_index(server, capacity_index_request(ordinal), "capacity test").await
    }

    async fn capacity_filler_index(
        server: &TestServer,
        record: FillerRecord,
    ) -> (StatusCode, Option<String>) {
        post_capacity_index(
            server,
            capacity_filler_index_request(record),
            "capacity filler",
        )
        .await
    }

    async fn capacity_index_bounded(
        server: &TestServer,
        ordinal: usize,
    ) -> Result<(StatusCode, Option<String>)> {
        tokio::time::timeout(REQUEST_TIMEOUT, capacity_index(server, ordinal))
            .await
            .map_err(|_| anyhow::anyhow!("capacity /index request {ordinal} did not finish"))
    }

    async fn capacity_filler_index_bounded(
        server: &TestServer,
        record: FillerRecord,
    ) -> Result<(StatusCode, Option<String>)> {
        tokio::time::timeout(REQUEST_TIMEOUT, capacity_filler_index(server, record))
            .await
            .map_err(|_| anyhow::anyhow!("capacity filler /index request {record:?} did not finish"))
    }

    async fn capacity_term_ids(server: &TestServer, value: &str) -> Vec<String> {
        let request = json!({
            "query": { "term": { "field": "kw", "value": value } },
            "limit": 8,
        });
        assert!(
            serde_json::to_vec(&request)
                .expect("serialize capacity search body")
                .len()
                < MAX_HTTP_BODY_BYTES,
            "capacity query must remain a legal HTTP body",
        );
        let response = server
            .post(&format!("/collections/{CAPACITY_COLLECTION}/search"))
            .json(&request)
            .await;
        response.assert_status_ok();
        let mut ids = response.json::<Value>()["hits"]
            .as_array()
            .expect("capacity search hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("capacity search hit external ID")
                    .to_owned()
            })
            .collect::<Vec<_>>();
        ids.sort();
        ids
    }

    fn metric_u64(metrics: &str, name: &str) -> u64 {
        let values = metrics
            .lines()
            .filter_map(|line| {
                let (metric, value) =
                    line.split_once(|character: char| character.is_whitespace())?;
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
            .unwrap_or_else(|_| panic!("{name} must be an unsigned integer: {}", values[0]))
    }

    #[derive(Clone, Copy, Debug)]
    struct PendingBudget {
        reserved: u64,
        active: u64,
        frozen: u64,
        total: u64,
        high_water: u64,
    }

    impl PendingBudget {
        fn checkpointable_bytes(self) -> u64 {
            self.active
                .checked_add(self.frozen)
                .expect("public active plus frozen bytes must not overflow")
        }

        fn assert_documented_limit(self, phase: &str) {
            let hard_limit = PENDING_HARD_LIMIT_BYTES as u64;
            assert_eq!(
                self.total,
                self.reserved + self.active + self.frozen,
                "{phase}: public pending-change total must equal reserved plus active plus frozen: {self:?}",
            );
            assert!(
                self.total <= hard_limit,
                "{phase}: public pending-change total must stay within the documented 256 MiB budget: {self:?}",
            );
            assert!(
                self.high_water <= hard_limit,
                "{phase}: public pending-change high water must stay within the documented 256 MiB budget: {self:?}",
            );
        }
    }

    async fn public_pending_budget(server: &TestServer, phase: &str) -> PendingBudget {
        let response = server.get("/metrics").await;
        response.assert_status_ok();
        let metrics = response.text();
        let budget = PendingBudget {
            reserved: metric_u64(&metrics, "lumen_pending_change_reserved_bytes"),
            active: metric_u64(&metrics, "lumen_pending_change_active_bytes"),
            frozen: metric_u64(&metrics, "lumen_pending_change_frozen_bytes"),
            total: metric_u64(&metrics, "lumen_pending_change_total_bytes"),
            high_water: metric_u64(&metrics, "lumen_pending_change_high_water_bytes"),
        };
        budget.assert_documented_limit(phase);
        budget
    }

    fn external_value_cannot_fit_checkpointable_budget(budget: PendingBudget) -> bool {
        budget
            .checkpointable_bytes()
            .checked_add(LARGE_KEYWORD_VALUE_BYTES as u64)
            .map_or(true, |required| required > PENDING_HARD_LIMIT_BYTES as u64)
    }

    async fn fill_checkpointable_budget_until_external_value_cannot_fit(
        server: &TestServer,
    ) -> Result<Option<FillerRecord>> {
        let mut last_accepted = None;
        let mut last_local_refusal = None;
        'stage: for (stage, value_bytes) in FILLER_VALUE_STAGES.into_iter().enumerate() {
            for ordinal in 0..MAX_FILLER_VALUE_COUNT_PER_STAGE {
                let record = FillerRecord {
                    stage,
                    ordinal,
                    value_bytes,
                };
                for retry in 0..=MAX_FILLER_RETRIES_PER_ORDINAL {
                    let before = public_pending_budget(server, "before capacity filler request").await;
                    if external_value_cannot_fit_checkpointable_budget(before) {
                        return Ok(last_accepted);
                    }
                    match capacity_filler_index_bounded(server, record).await? {
                        (StatusCode::OK, _) => {
                            last_accepted = Some(record);
                            break;
                        }
                        (StatusCode::TOO_MANY_REQUESTS, retry_after) => {
                            let after = public_pending_budget(
                                server,
                                "after transient capacity filler refusal",
                            )
                            .await;
                            if external_value_cannot_fit_checkpointable_budget(after) {
                                return Ok(last_accepted);
                            }
                            last_local_refusal = Some((record, retry_after, after));
                            if retry == MAX_FILLER_RETRIES_PER_ORDINAL {
                                // This size cannot consume the remaining gap.
                                // Move to the next legal value size rather than
                                // retrying an identical admission price forever.
                                continue 'stage;
                            }
                            tokio::time::sleep(FILLER_RETRY_DELAY).await;
                        }
                        (status, retry_after) => anyhow::bail!(
                            "legal capacity filler {record:?} returned {status}, retry-after={retry_after:?}"
                        ),
                    }
                }
            }
        }
        let final_budget = public_pending_budget(server, "after bounded staged capacity filler loop").await;
        if external_value_cannot_fit_checkpointable_budget(final_budget) {
            Ok(last_accepted)
        } else {
            anyhow::bail!(
                "{} staged legal capacity fillers did not leave less than the exact 6 MiB external Keyword lower bound in public active-plus-frozen capacity; last local refusal={last_local_refusal:?}, budget={final_budget:?}",
                FILLER_VALUE_STAGES.len() * MAX_FILLER_VALUE_COUNT_PER_STAGE,
            );
        }
    }

    #[derive(Debug)]
    struct Refusal {
        ordinal: usize,
        retry_after: Option<String>,
        applied_before: u64,
        applied_after: u64,
        wal_before: u64,
        wal_after: u64,
    }

    const CAPACITY_CHILD_CASE_ENV: &str = "LUMEN_CAPACITY_CHILD_CASE";
    const CAPACITY_CHILD_HANDSHAKE_ENV: &str = "LUMEN_CAPACITY_CHILD_HANDSHAKE";
    const LOCAL_CAPACITY_CHILD_CASE: &str = "local-http-capacity";
    const LOCAL_CAPACITY_TEST_NAME: &str = "capacity_http_contract::pending_capacity_refuses_precommit_http_index_until_paused_checkpoint_publishes";
    const EXTERNAL_CAPACITY_CHILD_CASE: &str = "externally-committed-capacity";
    const EXTERNAL_CAPACITY_TEST_NAME: &str = "capacity_http_contract::committed_external_wal_capacity_contract::externally_committed_record_waits_at_full_pending_capacity_then_survives_cold_reopen";

    /// The capacity ledger is process-global.  The parent process runs this
    /// exact test body in a fresh e2e child, and the child proves it entered the
    /// intended body before any fixture allocation.  A wrong test filter cannot
    /// turn an empty child run into a false green.
    fn capacity_child_enters(case_key: &str) -> bool {
        if std::env::var(CAPACITY_CHILD_CASE_ENV).ok().as_deref() != Some(case_key) {
            return false;
        }
        let handshake = std::env::var_os(CAPACITY_CHILD_HANDSHAKE_ENV)
            .unwrap_or_else(|| panic!("{case_key}: child needs a handshake path"));
        std::fs::write(&handshake, case_key)
            .unwrap_or_else(|error| panic!("{case_key}: write child handshake: {error}"));
        true
    }

    async fn run_isolated_capacity_case(case_key: &'static str, test_name: &'static str) {
        let child_dir = tempfile::tempdir().expect("capacity child fixture directory");
        let handshake = child_dir.path().join("entered-case");
        let executable = std::env::current_exe().expect("current e2e test executable");
        let child_handshake = handshake.clone();
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new(executable)
                .env(CAPACITY_CHILD_CASE_ENV, case_key)
                .env(CAPACITY_CHILD_HANDSHAKE_ENV, &child_handshake)
                .arg(test_name)
                .arg("--exact")
                .arg("--nocapture")
                .arg("--test-threads=1")
                .output()
        })
        .await
        .expect("capacity child command task must join")
        .expect("start exact capacity child test");
        let entered = std::fs::read_to_string(&handshake).unwrap_or_else(|error| {
            panic!(
                "capacity child for {test_name} did not reach its intended body: {error}; stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )
        });
        assert_eq!(
            entered, case_key,
            "capacity child must enter exactly {test_name}, not a different filtered body"
        );
        assert!(
            output.status.success(),
            "isolated capacity child {test_name} failed: status={} stdout={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    async fn pending_capacity_refuses_precommit_http_index_until_paused_checkpoint_publishes_body()
    {
        assert_eq!(
            PENDING_HARD_LIMIT_BYTES,
            256 * 1024 * 1024,
            "this real fixture is pinned to the approved 256 MiB pending-change limit",
        );
        let fixture = capacity_fixture(false);
        fixture
            .server
            .put(&format!("/collections/{CAPACITY_COLLECTION}"))
            .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
        fixture
            .server
            .post(&format!("/collections/{CAPACITY_COLLECTION}/index"))
            .json(&json!({ "items": [{
                "external_id": "durable-base",
                "field": "kw",
                "value": "capacity-base",
            }] }))
            .await
            .assert_status_ok();
        assert!(
            CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                .await
                .expect("publish initial durable capacity base"),
            "initial capacity base must publish",
        );

        let mut frozen_ids = BTreeMap::new();
        let mut frozen_payload_bytes = 0usize;
        for ordinal in 0..FROZEN_VALUE_COUNT {
            let (status, retry_after) = capacity_index_bounded(&fixture.server, ordinal)
                .await
                .expect("the bounded frozen seed request must finish");
            assert_eq!(
                status,
                StatusCode::OK,
                "the one frozen seed must be accepted before the 256 MiB boundary, retry-after={retry_after:?}",
            );
            assert!(
                frozen_ids
                    .insert(capacity_external_id(ordinal), ordinal)
                    .is_none(),
                "frozen fixture rows must use distinct external IDs",
            );
            frozen_payload_bytes += LARGE_KEYWORD_VALUE_BYTES;
        }
        assert_eq!(
            frozen_payload_bytes, LARGE_KEYWORD_VALUE_BYTES,
            "the real paused checkpoint must retain its one distinct frozen payload",
        );

        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        fixture.hold.arm(entered_tx, release_rx);
        let mut release = SyncRelease(Some(release_tx));
        let mut paused_checkpoint = tokio::spawn({
            let checkpoint = fixture.checkpoint.clone();
            async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
        });
        let ready =
            tokio::task::spawn_blocking(move || entered_rx.recv_timeout(Duration::from_secs(30)))
                .await;
        if !matches!(ready, Ok(Ok(()))) {
            release.release();
            paused_checkpoint.abort();
            let _ = paused_checkpoint.await;
            panic!(
                "fixture checkpoint did not capture and reach its real SyncFile pause: {ready:?}"
            );
        }

        let mut active_ids = BTreeMap::new();
        let mut active_payload_bytes = 0usize;
        let mut refusal = None;
        let mut unexpected_status = None;
        for ordinal in FROZEN_VALUE_COUNT..MAX_LARGE_VALUE_COUNT {
            let applied_before = fixture.writer.applied_seq();
            let wal_before = fixture
                .wal
                .latest_seq()
                .await
                .expect("read MemWal before index");
            let response = capacity_index_bounded(&fixture.server, ordinal).await;
            let applied_after = fixture.writer.applied_seq();
            let wal_after = fixture
                .wal
                .latest_seq()
                .await
                .expect("read MemWal after index");
            let (status, retry_after) = match response {
                Ok(response) => response,
                Err(error) => {
                    unexpected_status = Some((ordinal, None, Some(error.to_string())));
                    break;
                }
            };
            match status {
                StatusCode::OK => {
                    assert!(
                        active_ids
                            .insert(capacity_external_id(ordinal), ordinal)
                            .is_none(),
                        "active fixture rows must use distinct external IDs",
                    );
                    active_payload_bytes += LARGE_KEYWORD_VALUE_BYTES;
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    refusal = Some(Refusal {
                        ordinal,
                        retry_after,
                        applied_before,
                        applied_after,
                        wal_before,
                        wal_after,
                    });
                    break;
                }
                other => {
                    unexpected_status = Some((ordinal, Some(other), retry_after));
                    break;
                }
            }
        }

        // Do all worker cleanup before making any capacity assertion. The drop
        // guard also protects every panic path above.
        release.release();
        let checkpoint_result =
            tokio::time::timeout(Duration::from_secs(30), &mut paused_checkpoint).await;
        let checkpoint_error = match checkpoint_result {
            Ok(Ok(Ok(true))) => None,
            Ok(Ok(Ok(false))) => Some("paused checkpoint reported persisted=false".to_owned()),
            Ok(Ok(Err(error))) => Some(format!("paused checkpoint returned {error:#}")),
            Ok(Err(error)) => Some(format!("paused checkpoint task failed: {error}")),
            Err(_) => {
                paused_checkpoint.abort();
                let _ = paused_checkpoint.await;
                Some("paused checkpoint did not finish after release".to_owned())
            }
        };
        assert!(
            checkpoint_error.is_none(),
            "paused checkpoint must publish after its SyncFile release: {checkpoint_error:?}",
        );
        assert!(
            !frozen_ids.is_empty(),
            "fixture must retain a distinct frozen external-ID payload before capacity refusal",
        );
        assert!(
            frozen_ids
                .keys()
                .all(|external_id| !active_ids.contains_key(external_id)),
            "the active side must not overwrite frozen rows to manufacture pending bytes",
        );
        if unexpected_status.is_some() {
            panic!(
                "legal one-item /index request returned an unrelated status before capacity admission: {unexpected_status:?}",
            );
        }
        if refusal.is_none() {
            let accepted_raw_payload_bytes = frozen_payload_bytes + active_payload_bytes;
            assert!(
                accepted_raw_payload_bytes > PENDING_HARD_LIMIT_BYTES,
                "the missing-429 oracle must cross 256 MiB of actually retained distinct Keyword payload before it fails: {accepted_raw_payload_bytes}",
            );
        }
        let refusal = refusal.expect(
            "a legal pre-submit /index request must receive 429 while the captured frozen payload remains unpublished",
        );
        assert_eq!(
            refusal.retry_after.as_deref(),
            Some("1"),
            "capacity refusal must expose the documented Retry-After: 1 header",
        );
        assert_eq!(
            refusal.wal_after, refusal.wal_before,
            "capacity refusal must not publish a caller-controlled record to MemWal",
        );
        assert_eq!(
            refusal.applied_after, refusal.applied_before,
            "capacity refusal must not advance the local applied sequence",
        );

        let (retry_status, retry_after) = capacity_index_bounded(&fixture.server, refusal.ordinal)
            .await
            .expect("the bounded capacity retry must finish");
        assert_eq!(
            retry_status,
            StatusCode::OK,
            "the same request must become admissible after the frozen checkpoint publishes, retry-after={retry_after:?}",
        );
        assert_eq!(
            fixture.writer.applied_seq(),
            refusal.applied_before + 1,
            "only the retry, never the refused pre-submit request, may allocate the next sequence",
        );
        assert_eq!(
            fixture
                .wal
                .latest_seq()
                .await
                .expect("read MemWal after retry"),
            refusal.wal_before + 1,
            "MemWal must contain exactly the retry after capacity publication",
        );

        assert_eq!(
            capacity_term_ids(&fixture.server, "capacity-base").await,
            vec!["durable-base".to_owned()],
            "the initial durable base remains queryable after capacity backpressure",
        );
        assert_eq!(
            capacity_term_ids(&fixture.server, &capacity_value(0)).await,
            vec![capacity_external_id(0)],
            "a frozen distinct Keyword payload remains live after publication",
        );
        assert_eq!(
            capacity_term_ids(&fixture.server, &capacity_value(refusal.ordinal)).await,
            vec![capacity_external_id(refusal.ordinal)],
            "the retry's caller-controlled payload becomes live exactly once",
        );

        assert!(
            CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                .await
                .expect("publish retry checkpoint"),
            "retry checkpoint must publish",
        );
        let cold = fixture
            .store
            .load_current_generation()
            .expect("cold-open post-capacity CURRENT")
            .expect("post-capacity CURRENT exists");
        let cold_server = TestServer::new(router(AppState::open(cold.engine)))
            .expect("cold capacity HTTP server");
        assert_eq!(
            capacity_term_ids(&cold_server, "capacity-base").await,
            vec!["durable-base".to_owned()],
            "cold CURRENT preserves the durable base",
        );
        assert_eq!(
            capacity_term_ids(&cold_server, &capacity_value(0)).await,
            vec![capacity_external_id(0)],
            "cold CURRENT preserves frozen work published after the pause",
        );
        assert_eq!(
            capacity_term_ids(&cold_server, &capacity_value(refusal.ordinal)).await,
            vec![capacity_external_id(refusal.ordinal)],
            "cold CURRENT preserves the admitted retry and never a refused phantom write",
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pending_capacity_refuses_precommit_http_index_until_paused_checkpoint_publishes() {
        if capacity_child_enters(LOCAL_CAPACITY_CHILD_CASE) {
            pending_capacity_refuses_precommit_http_index_until_paused_checkpoint_publishes_body()
                .await;
        } else {
            run_isolated_capacity_case(LOCAL_CAPACITY_CHILD_CASE, LOCAL_CAPACITY_TEST_NAME).await;
        }
    }

    mod committed_external_wal_capacity_contract {
        //! # Facets
        //!
        //! - Behavior: `indexing_durable_oracle.rs:11584-11597` requires the
        //!   committed record to wait behind the held checkpoint and remain
        //!   query-invisible. `:11600-11634` and `:11636-11676` then require
        //!   exactly-once live application and cold recovery. These assertions
        //!   exercise externally delivered capacity handling in
        //!   `apps/lumen/src/coordinator.rs:481-531` and configured checkpoint
        //!   ownership in `apps/lumen/src/segment_checkpoint.rs:411-435`.
        //! - Security: `indexing_durable_oracle.rs:11530-11541` proves the
        //!   caller-controlled local input rejected at the HTTP boundary did
        //!   not publish a WAL record or advance the applied watermark.
        //!   `:11564-11597` keeps the later committed copy in the WAL but
        //!   invisible until it owns budget. This change reads only the process's
        //!   public metrics; it opens no new caller path, parser, or file input
        //!   boundary.
        //! - Performance: `apps/lumen/ROADMAP.md:60-70` promises, verbatim,
        //!   "Pending active, frozen, and reserved changes have a 256 MiB total
        //!   budget." `indexing_durable_oracle.rs:10773-10803` checks that
        //!   public total and high-water accounting stay within that limit.
        //!   `:10813-10868` and `:11543-11552` use public active plus frozen
        //!   bytes as the stable retained-work lower bound before the 6 MiB
        //!   committed value enters. Test timeouts bound cleanup only; they make
        //!   no latency claim.

        use super::*;
        use lumen::wal::{WalLog, WalRecord};

        fn externally_committed_refused_entry(ordinal: usize) -> RaftLogEntry {
            RaftLogEntry::Index {
                collection_id: CAPACITY_COLLECTION.to_owned(),
                // This is the same valid logical item that the local HTTP route
                // rejected before publication. It enters through a real
                // already-committed WAL publisher, not `submit`.
                req: serde_json::from_value(capacity_index_request(ordinal))
                    .expect("construct externally committed capacity Index record"),
            }
        }

        async fn wait_for_applied_sequence(
            writer: &WriteCoordinator,
            sequence: u64,
            context: &str,
        ) {
            let observed = tokio::time::timeout(Duration::from_secs(30), async {
                loop {
                    if writer.applied_seq() >= sequence {
                        return;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await;
            assert!(
                observed.is_ok(),
                "{context}: applied sequence stayed {} below committed sequence {sequence}",
                writer.applied_seq(),
            );
        }

        async fn externally_committed_record_waits_at_full_pending_capacity_then_survives_cold_reopen_body(
        ) {
            assert_eq!(
                PENDING_HARD_LIMIT_BYTES,
                256 * 1024 * 1024,
                "this contract uses the approved 256 MiB pending-change limit",
            );
            let fixture = capacity_fixture(true);
            fixture
                .server
                .put(&format!("/collections/{CAPACITY_COLLECTION}"))
                .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
                .await
                .assert_status_ok();
            fixture
                .server
                .post(&format!("/collections/{CAPACITY_COLLECTION}/index"))
                .json(&json!({ "items": [{
                        "external_id": "committed-capacity-base",
                        "field": "kw",
                        "value": "committed-capacity-base-value",
                    }] }))
                .await
                .assert_status_ok();
            assert!(
                CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                    .await
                    .expect("publish committed-capacity baseline"),
                "baseline must publish before the held checkpoint",
            );

            // Capture one real, distinct frozen payload. The configured capacity
            // owner already shares this root, and its SyncFile pause is the only
            // artificial block; all admission and WAL work remains production.
            let (frozen_status, frozen_retry_after) = capacity_index_bounded(&fixture.server, 0)
                .await
                .expect("frozen seed request must finish");
            assert_eq!(
                frozen_status,
                StatusCode::OK,
                "the bounded frozen seed must fit before capacity is full, retry-after={frozen_retry_after:?}",
            );
            let (entered_tx, entered_rx) = mpsc::sync_channel(1);
            let (release_tx, release_rx) = mpsc::sync_channel(1);
            fixture.hold.arm(entered_tx, release_rx);
            let mut release = SyncRelease(Some(release_tx));
            let mut paused_checkpoint = tokio::spawn({
                let checkpoint = fixture.checkpoint.clone();
                async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
            });
            let readiness = tokio::task::spawn_blocking(move || {
                entered_rx.recv_timeout(Duration::from_secs(30))
            })
            .await;
            if !matches!(readiness, Ok(Ok(()))) {
                release.release();
                paused_checkpoint.abort();
                let _ = paused_checkpoint.await;
                panic!(
                    "held checkpoint did not reach its real SyncFile pause before capacity setup: {readiness:?}"
                );
            }

            // The initial 429 remains a public HTTP boundary assertion. It does
            // not itself prove that a foreign record's lower retained-state cost
            // cannot fit, so the filler below establishes that premise separately.
            let mut refusal = None;
            let mut unexpected = None;
            let mut first_active_ordinal = None;
            for ordinal in FROZEN_VALUE_COUNT..MAX_LARGE_VALUE_COUNT {
                let applied_before = fixture.writer.applied_seq();
                let wal_before = fixture
                    .wal
                    .latest_seq()
                    .await
                    .expect("read WAL before local capacity request");
                let result = capacity_index_bounded(&fixture.server, ordinal).await;
                let applied_after = fixture.writer.applied_seq();
                let wal_after = fixture
                    .wal
                    .latest_seq()
                    .await
                    .expect("read WAL after local capacity request");
                match result {
                    Ok((StatusCode::OK, _)) => {
                        first_active_ordinal.get_or_insert(ordinal);
                    }
                    Ok((StatusCode::TOO_MANY_REQUESTS, retry_after)) => {
                        refusal = Some(Refusal {
                            ordinal,
                            retry_after,
                            applied_before,
                            applied_after,
                            wal_before,
                            wal_after,
                        });
                        break;
                    }
                    Ok((status, retry_after)) => {
                        unexpected = Some((ordinal, status, retry_after));
                        break;
                    }
                    Err(error) => {
                        unexpected = Some((
                            ordinal,
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Some(error.to_string()),
                        ));
                        break;
                    }
                }
            }

            // Stage-source relief may release the transient local reservation,
            // but cannot lower active plus frozen retained work. Fill through
            // the real HTTP/coordinator path until that public lower bound leaves
            // less than the exact external Keyword value itself.
            let filler_result = tokio::time::timeout(
                FILL_SETUP_TIMEOUT,
                fill_checkpointable_budget_until_external_value_cannot_fit(&fixture.server),
            )
            .await
            .map_err(|_| anyhow::anyhow!("capacity filler setup exceeded its bounded test timeout"))
            .and_then(|result| result);

            // Adjacent public snapshots reject a transient source-ledger
            // observation. If retained work changes while the witness is being
            // sampled, resample before publication. The held shared-root
            // SyncFile prevents a successful checkpoint publication from
            // reducing active plus frozen during this setup.
            let capacity_witness = if filler_result.is_ok()
                && refusal.is_some()
                && unexpected.is_none()
            {
                let mut changed = None;
                let mut stable = None;
                for _ in 0..MAX_WITNESS_SCRAPE_RETRIES {
                    let first = public_pending_budget(
                        &fixture.server,
                        "first external-capacity witness scrape",
                    )
                    .await;
                    let second = public_pending_budget(
                        &fixture.server,
                        "second external-capacity witness scrape",
                    )
                    .await;
                    if first.checkpointable_bytes() == second.checkpointable_bytes() {
                        stable = Some(second);
                        break;
                    }
                    changed = Some((first, second));
                    tokio::time::sleep(FILLER_RETRY_DELAY).await;
                }
                match stable {
                    None => Err(anyhow::anyhow!(
                        "active-plus-frozen retained capacity did not stabilize across {} bounded public witness retries: {changed:?}",
                        MAX_WITNESS_SCRAPE_RETRIES,
                    )),
                    Some(second) if !external_value_cannot_fit_checkpointable_budget(second) => {
                        Err(anyhow::anyhow!(
                            "public active-plus-frozen capacity still leaves room for the exact {}-byte external Keyword lower bound: {second:?}",
                            LARGE_KEYWORD_VALUE_BYTES,
                        ))
                    }
                    Some(_) if paused_checkpoint.is_finished() => Err(anyhow::anyhow!(
                        "the held shared-root checkpoint completed before external publication"
                    )),
                    Some(second) => Ok(second),
                }
            } else {
                Err(anyhow::anyhow!(
                    "capacity filler, required local HTTP refusal, or local HTTP status was unavailable"
                ))
            };

            let external_baseline = if capacity_witness.is_ok() {
                Some((
                    fixture.writer.applied_seq(),
                    fixture
                        .wal
                        .latest_seq()
                        .await
                        .expect("read WAL before externally committed publish"),
                ))
            } else {
                None
            };
            let external_publish = if let (Some(refusal), Some(_)) = (refusal.as_ref(), external_baseline) {
                let entry = externally_committed_refused_entry(refusal.ordinal);
                Some(
                    tokio::time::timeout(
                        Duration::from_secs(2),
                        fixture.wal.publish(WalRecord::new(entry)),
                    )
                    .await,
                )
            } else {
                None
            };
            let externally_committed_sequence = external_publish
                .as_ref()
                .and_then(|timed| timed.as_ref().ok())
                .and_then(|published| published.as_ref().ok())
                .copied();
            let applied_while_held = if let Some(sequence) = externally_committed_sequence {
                Some(
                    tokio::time::timeout(Duration::from_secs(2), async {
                        loop {
                            if fixture.writer.applied_seq() >= sequence {
                                return fixture.writer.applied_seq();
                            }
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    })
                    .await,
                )
            } else {
                None
            };
            let applied_after_held_observation = fixture.writer.applied_seq();
            let visible_while_held = if let Some(refusal) = refusal.as_ref() {
                Some(
                    tokio::time::timeout(
                        Duration::from_secs(2),
                        capacity_term_ids(&fixture.server, &capacity_value(refusal.ordinal)),
                    )
                    .await,
                )
            } else {
                None
            };

            // This checkpoint was selected and captured before the external
            // record was published. Its successful completion independently
            // frees the frozen charge; it must not wait for a record that cannot
            // enter the public active-plus-frozen capacity witness.
            release.release();
            let paused_completion =
                tokio::time::timeout(Duration::from_secs(30), &mut paused_checkpoint).await;
            let paused_error = match paused_completion {
                Ok(Ok(Ok(true))) => None,
                Ok(Ok(Ok(false))) => Some("held checkpoint reported persisted=false".to_owned()),
                Ok(Ok(Err(error))) => Some(format!("held checkpoint returned {error:#}")),
                Ok(Err(error)) => Some(format!("held checkpoint task failed: {error}")),
                Err(_) => {
                    paused_checkpoint.abort();
                    let _ = paused_checkpoint.await;
                    Some("held checkpoint did not finish after its release".to_owned())
                }
            };

            assert!(
                unexpected.is_none(),
                "a legal local capacity request returned an unrelated result: {unexpected:?}",
            );
            let refusal = refusal.expect(
                "fixture must reach real pre-submit 429 before testing committed external WAL input",
            );
            let first_active_ordinal = first_active_ordinal.expect(
                "a full-capacity fixture must retain at least one distinct active payload before refusal",
            );
            assert_eq!(
                refusal.retry_after.as_deref(),
                Some("1"),
                "the local full-capacity boundary must retain Retry-After: 1",
            );
            assert_eq!(
                refusal.wal_after, refusal.wal_before,
                "the rejected local request must not become a committed WAL record",
            );
            assert_eq!(
                refusal.applied_after, refusal.applied_before,
                "the rejected local request must not advance the applied watermark",
            );
            let last_filler = filler_result.expect(
                "bounded legal HTTP filler must establish the public active-plus-frozen capacity witness",
            );
            let witness = capacity_witness.expect(
                "public active-plus-frozen capacity must prove the external Keyword cannot fit before publication",
            );
            assert!(
                external_value_cannot_fit_checkpointable_budget(witness),
                "the final public witness must leave less than the exact external Keyword lower bound: {witness:?}",
            );
            assert!(
                paused_error.is_none(),
                "the independently captured checkpoint must complete after release: {paused_error:?}",
            );
            let (applied_before_external, wal_before_external) = external_baseline.expect(
                "external publish baseline must be sampled only after the stable capacity witness",
            );
            let external_sequence = external_publish
                .expect("externally committed publish must run after the stable capacity witness")
                .expect("externally committed MemWal publish must not block at full capacity")
                .expect("externally committed MemWal publish must allocate a durable sequence");
            assert_eq!(
                external_sequence,
                wal_before_external + 1,
                "the exact request rejected before submission must become the next committed WAL record after filler work",
            );
            assert!(
                external_sequence > refusal.wal_before,
                "the local pre-submit refusal must not consume a WAL sequence before the later committed record",
            );
            assert_eq!(
                fixture
                    .wal
                    .latest_seq()
                    .await
                    .expect("read WAL after externally committed publish"),
                external_sequence,
                "the externally committed record must remain in the WAL while apply waits",
            );
            let applied_while_held = applied_while_held
                .expect("applied-watermark observation must run after external publication");
            assert!(
                applied_while_held.is_err(),
                "a committed record whose exact retained Keyword value cannot fit the public active-plus-frozen budget must wait for checkpoint release instead of applying early: {applied_while_held:?}",
            );
            assert_eq!(
                applied_after_held_observation, applied_before_external,
                "the applied watermark must remain stationary while the full-budget checkpoint is held",
            );
            assert_eq!(
                visible_while_held
                    .expect("read query observation must run after external publication")
                    .expect("read query must finish while capacity is held"),
                Vec::<String>::new(),
                "an externally committed record must not become query-visible before capacity owns it",
            );

            wait_for_applied_sequence(
                fixture.writer.as_ref(),
                external_sequence,
                "the independently completed checkpoint must let the committed record apply",
            )
            .await;
            let external_value = capacity_value(refusal.ordinal);
            assert_eq!(
                capacity_term_ids(&fixture.server, &external_value).await,
                vec![capacity_external_id(refusal.ordinal)],
                "the formerly refused request must become live exactly once through its committed WAL record",
            );
            assert_eq!(
                capacity_term_ids(&fixture.server, "committed-capacity-base-value").await,
                vec!["committed-capacity-base".to_owned()],
                "the independently published checkpoint must retain prior durable data",
            );
            assert_eq!(
                capacity_term_ids(&fixture.server, &capacity_value(0)).await,
                vec![capacity_external_id(0)],
                "the frozen payload that released capacity remains live",
            );
            assert_eq!(
                capacity_term_ids(&fixture.server, &capacity_value(first_active_ordinal)).await,
                vec![capacity_external_id(first_active_ordinal)],
                "the later live state must retain active work from before the external record",
            );
            if let Some(last_filler) = last_filler {
                assert_eq!(
                    capacity_term_ids(&fixture.server, &capacity_filler_value(last_filler))
                        .await,
                    vec![capacity_filler_external_id(last_filler)],
                    "the final public filler record must remain live after it establishes the capacity witness",
                );
            }

            assert!(
                CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                    .await
                    .expect("publish externally committed record"),
                "a later ordinary checkpoint must persist the applied external record",
            );
            let cold = fixture
                .store
                .load_current_generation()
                .expect("cold-open externally committed capacity CURRENT")
                .expect("externally committed capacity CURRENT exists");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("cold externally committed capacity HTTP server");
            assert_eq!(
                capacity_term_ids(&cold_server, &external_value).await,
                vec![capacity_external_id(refusal.ordinal)],
                "cold reopen must retain the externally committed record after checkpoint release",
            );
            assert_eq!(
                capacity_term_ids(&cold_server, "committed-capacity-base-value").await,
                vec!["committed-capacity-base".to_owned()],
                "cold reopen must retain the prior durable base",
            );
            assert_eq!(
                capacity_term_ids(&cold_server, &capacity_value(0)).await,
                vec![capacity_external_id(0)],
                "cold reopen must retain the frozen checkpoint payload that made capacity available",
            );
            assert_eq!(
                capacity_term_ids(&cold_server, &capacity_value(first_active_ordinal)).await,
                vec![capacity_external_id(first_active_ordinal)],
                "cold reopen must retain active work from before the external record",
            );
            if let Some(last_filler) = last_filler {
                assert_eq!(
                    capacity_term_ids(&cold_server, &capacity_filler_value(last_filler))
                        .await,
                    vec![capacity_filler_external_id(last_filler)],
                    "cold reopen must retain the final public filler record that established capacity",
                );
            }
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn externally_committed_record_waits_at_full_pending_capacity_then_survives_cold_reopen(
        ) {
            if super::capacity_child_enters(super::EXTERNAL_CAPACITY_CHILD_CASE) {
                externally_committed_record_waits_at_full_pending_capacity_then_survives_cold_reopen_body().await;
            } else {
                super::run_isolated_capacity_case(
                    super::EXTERNAL_CAPACITY_CHILD_CASE,
                    super::EXTERNAL_CAPACITY_TEST_NAME,
                )
                .await;
            }
        }
    }
}

mod first_sparse_checkpoint_contract {
    //! # Facets
    //!
    //! - Behavior: the assertions at
    //!   indexing_durable_oracle.rs:11854-12024 create one fresh seven-field
    //!   collection, asserts each zero-row base and first sparse delta, writes
    //!   during real checkpoint file I/O, and checks live and cold results. It
    //!   covers the first-capture changes in apps/lumen/src/storage.rs:13668-13691
    //!   and apps/lumen/src/segment_rdb.rs:540-604.
    //! - Security: apps/lumen/src/segment_rdb.rs:1910-2070 validates the
    //!   persisted catalog and local-row bytes this path reads. Existing
    //!   v2_current_refuses_keyword_delta_local_rows_count_mismatch and
    //!   v2_current_refuses_keyword_delta_duplicate_stable_local_rows at
    //!   indexing_durable_oracle.rs:2605-2656 feed the shared
    //!   lumen-local-eids-cbor-v1 boundary malformed bytes and require refusal
    //!   without changing CURRENT. This valid-generation case carries behavior.
    //! - Performance: apps/lumen/ROADMAP.md:60-70 promises a 256 MiB total
    //!   active/frozen/reserved budget and checkpoint progress at that limit.
    //!   indexing_durable_oracle.rs:10714-10941 already measures that budget.
    //!   The zero-row-base and bounded-local-row assertions below are
    //!   structural only. They make no latency or RSS claim.

    use super::*;

    use lumen::segment_checkpoint::SegmentCheckpointSink;
    use lumen::segment_rdb::{MergeObserver, MergePhase};
    use std::collections::BTreeSet;
    use std::io;
    use std::sync::{mpsc, Mutex};
    use storage_durable::{CommitStep, FailureInjector, FailurePoint};

    const COLLECTION: &str = "first-sparse-all-fields";
    const KW: &str = "kw";
    const NUM: &str = "num";
    const SET: &str = "tags";
    const HASH: &str = "sig";
    const TEXT: &str = "body";
    const FLAT: &str = "flat";
    const HNSW: &str = "hnsw";

    const LIVE: &str = "first-sparse-live";
    const DELETED: &str = "first-sparse-deleted";
    const REPLACED: &str = "first-sparse-replaced";
    const EMPTY: &str = "first-sparse-empty";
    const CONCURRENT: &str = "first-sparse-concurrent";
    const INITIAL_ROWS: u64 = 4;

    #[derive(Default)]
    struct NoopMergeObserver;

    impl MergeObserver for NoopMergeObserver {
        fn observe(&self, _: MergePhase) -> io::Result<()> {
            Ok(())
        }
    }

    /// The real sync hook fires after capture. SegmentCheckpointSink offloads
    /// its save, so the test can safely drive HTTP while this hook is held.
    #[derive(Default)]
    struct HoldNextSyncFile {
        hold: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
    }

    impl HoldNextSyncFile {
        fn arm(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
            assert!(
                self.hold
                    .lock()
                    .expect("first sparse sync hold mutex")
                    .replace((entered, release))
                    .is_none(),
                "first sparse fixture arms one checkpoint sync hold",
            );
        }
    }

    impl FailureInjector for HoldNextSyncFile {
        fn check(&self, point: &FailurePoint) -> io::Result<()> {
            if point.step != CommitStep::SyncFile {
                return Ok(());
            }
            let Some((entered, release)) = self
                .hold
                .lock()
                .expect("first sparse sync hold mutex")
                .take()
            else {
                return Ok(());
            };
            entered.send(()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "first sparse checkpoint readiness receiver dropped",
                )
            })?;
            release.recv().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "first sparse checkpoint release sender dropped",
                )
            })?;
            Ok(())
        }
    }

    /// Every failure path releases the blocking SyncFile callback.
    struct SyncRelease(Option<mpsc::SyncSender<()>>);

    impl SyncRelease {
        fn release(&mut self) {
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }

    impl Drop for SyncRelease {
        fn drop(&mut self) {
            self.release();
        }
    }

    struct Fixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        server: TestServer,
        store: Arc<SegmentRdbStore>,
        checkpoint: Arc<SegmentCheckpointSink>,
        writer: Arc<WriteCoordinator>,
        hold: Arc<HoldNextSyncFile>,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().expect("first sparse fixture directory");
        let root = dir.path().join("segments");
        let hold = Arc::new(HoldNextSyncFile::default());
        let store = Arc::new(
            SegmentRdbStore::with_failure_injector_and_merge_observer(
                &root,
                hold.clone(),
                Arc::new(NoopMergeObserver),
            )
            .expect("open first sparse segment store"),
        );
        let aof: SharedAof = Arc::new(Mutex::new(
            AofWriter::open(dir.path().join("aof.log")).expect("open first sparse AOF"),
        ));
        let engine = Arc::new(Engine::new());
        let wal: SharedWal = Arc::new(MemWal::new());
        let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
        let sink_writer: Arc<dyn WriteSink> = writer.clone();
        let checkpoint = Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: sink_writer.clone(),
            aof: Some(aof),
        });
        let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
        let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink_writer)
            .with_checkpoint(checkpoint_api);
        Fixture {
            _dir: dir,
            root,
            server: TestServer::new(router(state)).expect("first sparse HTTP server"),
            store,
            checkpoint,
            writer,
            hold,
        }
    }

    fn vector(seed: f64) -> Value {
        json!([
            seed,
            seed * 0.37 + 0.11,
            seed * -0.23 + 0.07,
            seed * 0.19 - 0.13,
        ])
    }

    async fn create_collection(server: &TestServer) {
        server
            .put(&format!("/collections/{COLLECTION}"))
            .json(&json!({ "fields": {
                KW: { "type": "keyword" },
                NUM: { "type": "number" },
                SET: { "type": "set" },
                HASH: { "type": "hash" },
                TEXT: { "type": "text", "analyzer": "whitespace_lower" },
                FLAT: { "type": "vector", "dim": 4, "metric": "l2", "backend": "flat-cpu" },
                HNSW: { "type": "vector", "dim": 4, "metric": "l2", "backend": "hnsw-cpu" },
            }}))
            .await
            .assert_status_ok();
    }

    async fn index_full(
        server: &TestServer,
        id: &str,
        keyword: &str,
        number: f64,
        tag: &str,
        hash: &str,
        text: &str,
        flat: f64,
        hnsw: f64,
    ) {
        server
            .post(&format!("/collections/{COLLECTION}/index"))
            .json(&json!({ "items": [
                { "external_id": id, "field": KW, "value": keyword },
                { "external_id": id, "field": NUM, "value": number },
                { "external_id": id, "field": SET, "value": [tag] },
                { "external_id": id, "field": HASH, "value": hash },
                { "external_id": id, "field": TEXT, "value": text },
                { "external_id": id, "field": FLAT, "value": vector(flat) },
                { "external_id": id, "field": HNSW, "value": vector(hnsw) },
            ] }))
            .await
            .assert_status_ok();
    }

    async fn replace_without_set(
        server: &TestServer,
        id: &str,
        keyword: &str,
        number: f64,
        hash: &str,
        text: &str,
        flat: f64,
        hnsw: f64,
    ) {
        server
            .put(&format!("/collections/{COLLECTION}/docs:replace"))
            .json(&json!({ "docs": [{
                "external_id": id,
                "fields": {
                    KW: keyword,
                    NUM: number,
                    HASH: hash,
                    TEXT: text,
                    FLAT: vector(flat),
                    HNSW: vector(hnsw),
                },
            }]}))
            .await
            .assert_status_ok();
    }

    async fn replace_empty(server: &TestServer, id: &str) {
        server
            .put(&format!("/collections/{COLLECTION}/docs:replace"))
            .json(&json!({ "docs": [{ "external_id": id, "fields": {} }] }))
            .await
            .assert_status_ok();
    }

    async fn search_ids(server: &TestServer, query: Value) -> Vec<String> {
        let response = server
            .post(&format!("/collections/{COLLECTION}/search"))
            .json(&json!({ "query": query, "limit": 32, "track_total": true }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        let hits = body["hits"].as_array().expect("first sparse search hits");
        let mut ids: Vec<_> = hits
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("first sparse hit external ID")
                    .to_owned()
            })
            .collect();
        ids.sort();
        ids
    }

    async fn assert_ids(server: &TestServer, query: Value, expected: &[&str], context: &str) {
        let mut expected: Vec<_> = expected.iter().map(|id| (*id).to_owned()).collect();
        expected.sort();
        assert_eq!(
            search_ids(server, query).await,
            expected,
            "{context}: public search must retain exactly the expected IDs",
        );
    }

    async fn assert_text(server: &TestServer, text: &str, expected: &[&str], context: &str) {
        let response = server
            .post(&format!("/collections/{COLLECTION}/search"))
            .json(&json!({ "query": {
                "match": { "field": TEXT, "text": text, "op": "and" }
            }, "limit": 32, "track_total": true }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        assert!(
            body["hits"]
                .as_array()
                .expect("first sparse Text hits")
                .iter()
                .all(|hit| hit["score"].as_f64().is_some()),
            "{context}: Text Match exposes serialized BM25 scores: {body}",
        );
        let mut actual: Vec<_> = body["hits"]
            .as_array()
            .expect("first sparse Text hits")
            .iter()
            .map(|hit| hit["external_id"].as_str().expect("Text ID").to_owned())
            .collect();
        actual.sort();
        let mut expected: Vec<_> = expected.iter().map(|id| (*id).to_owned()).collect();
        expected.sort();
        assert_eq!(
            actual, expected,
            "{context}: Text Match retains expected IDs"
        );
    }

    fn term(field: &str, value: &str) -> Value {
        json!({ "term": { "field": field, "value": value } })
    }

    fn number(value: f64) -> Value {
        json!({ "range": { "field": NUM, "gte": value, "lte": value } })
    }

    fn hash(value: &str) -> Value {
        json!({ "hamming": { "field": HASH, "hash": value, "max_distance": 0 } })
    }

    fn knn(field: &str, seed: f64) -> Value {
        json!({ "knn": { "field": field, "vector": vector(seed), "k": 1 } })
    }

    fn collection(manifest: &Value) -> &Value {
        stage1_reuse_catalog_collection(manifest, COLLECTION)
    }

    fn base<'a>(manifest: &'a Value, role: &str, field: Option<&str>) -> &'a Value {
        collection(manifest)["segments"]
            .as_array()
            .expect("first sparse catalog segments")
            .iter()
            .find(|segment| {
                segment["role"] == json!(role)
                    && segment["field"] == field.map_or(Value::Null, |value| json!(value))
                    && segment["kind"] == json!("base")
                    && segment["ordinal"] == json!(0)
            })
            .unwrap_or_else(|| panic!("first sparse catalog needs {role}/{field:?} base"))
    }

    fn delta<'a>(manifest: &'a Value, field: &str) -> &'a Value {
        let deltas: Vec<_> = collection(manifest)["segments"]
            .as_array()
            .expect("first sparse catalog segments")
            .iter()
            .filter(|segment| {
                segment["role"] == json!("field")
                    && segment["field"] == json!(field)
                    && segment["kind"] == json!("delta")
            })
            .collect();
        assert_eq!(
            deltas.len(),
            1,
            "first sparse checkpoint publishes exactly one {field} delta",
        );
        assert_eq!(
            deltas[0]["ordinal"],
            json!(1),
            "first {field} delta immediately follows its zero-row base",
        );
        deltas[0]
    }

    fn lseg_rows(generation: &Path, reference: &Value) -> u32 {
        assert_eq!(reference["format"], json!("lseg-v1"));
        let path = generation.join(reference["path"].as_str().expect("catalog segment path"));
        let metadata = std::fs::symlink_metadata(&path).expect("inspect first sparse segment");
        assert!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "first sparse catalog names a regular local segment: {}",
            path.display(),
        );
        let bytes = std::fs::read(&path).expect("read lseg-v1 header");
        assert!(
            bytes.len() >= 24,
            "lseg-v1 segment contains its fixed 24-byte header prefix",
        );
        u32::from_le_bytes(bytes[20..24].try_into().expect("read lseg n_docs"))
    }

    fn assert_manifest(generation: &Path, manifest: &Value) {
        assert_eq!(manifest["schema_version"], json!(2));
        assert_eq!(
            manifest["collections"].as_array().map(Vec::len),
            Some(1),
            "first sparse catalog contains exactly its fresh collection",
        );
        for (role, field) in [
            ("collection_eids", None),
            ("field", Some(KW)),
            ("field", Some(NUM)),
            ("field", Some(SET)),
            ("field", Some(HASH)),
            ("field", Some(TEXT)),
            ("field", Some(FLAT)),
            ("field", Some(HNSW)),
            ("vector_eids", Some(FLAT)),
            ("vector_eids", Some(HNSW)),
        ] {
            let base = base(manifest, role, field);
            assert!(base["local_rows"].is_null());
            assert_eq!(
                lseg_rows(generation, base),
                0,
                "fresh {role}/{field:?} base contains zero rows; captured records belong in sparse deltas",
            );
        }
        let expected: BTreeSet<_> = [LIVE, DELETED, REPLACED, EMPTY]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();
        for field in [KW, NUM, SET, HASH, TEXT, FLAT, HNSW] {
            let delta = delta(manifest, field);
            assert_eq!(delta["format"], json!("lseg-v1"));
            assert_eq!(
                stage1_keyword_delta_rows_count(delta),
                INITIAL_ROWS,
                "first {field} delta maps exactly the four fresh IDs touched before capture",
            );
            assert_eq!(
                stage1_keyword_delta_read_rows(generation, delta)
                    .into_iter()
                    .collect::<BTreeSet<_>>(),
                expected,
                "first {field} local rows retain update, delete, omission, and empty replacement",
            );
        }
    }

    async fn assert_state(server: &TestServer, include_concurrent: bool, context: &str) {
        assert_ids(server, term(KW, "live-keyword"), &[LIVE], context).await;
        assert_ids(server, term(KW, "deleted-keyword"), &[], context).await;
        assert_ids(server, term(KW, "empty-keyword"), &[], context).await;
        assert_ids(server, term(KW, "replace-old"), &[], context).await;
        assert_ids(server, term(KW, "replace-new"), &[REPLACED], context).await;
        assert_ids(server, number(101.0), &[LIVE], context).await;
        assert_ids(server, number(202.0), &[REPLACED], context).await;
        assert_ids(server, term(SET, "live-tag"), &[LIVE], context).await;
        assert_ids(server, term(SET, "replace-old-tag"), &[], context).await;
        assert_ids(server, hash("0000000000000a11"), &[LIVE], context).await;
        assert_ids(server, hash("0000000000000a13"), &[REPLACED], context).await;
        assert_text(server, "live text", &[LIVE], context).await;
        assert_text(server, "replaced text", &[REPLACED], context).await;
        assert_text(server, "deleted text", &[], context).await;
        for (field, seed, expected) in [
            (FLAT, 1.0, LIVE),
            (HNSW, 2.0, LIVE),
            (FLAT, 11.0, REPLACED),
            (HNSW, 12.0, REPLACED),
        ] {
            assert_ids(server, knn(field, seed), &[expected], context).await;
        }
        if include_concurrent {
            assert_ids(
                server,
                term(KW, "concurrent-keyword"),
                &[CONCURRENT],
                context,
            )
            .await;
            assert_ids(server, number(303.0), &[CONCURRENT], context).await;
            assert_ids(server, term(SET, "concurrent-tag"), &[CONCURRENT], context).await;
            assert_ids(server, hash("0000000000000a14"), &[CONCURRENT], context).await;
            assert_text(server, "concurrent text", &[CONCURRENT], context).await;
            assert_ids(server, knn(FLAT, 21.0), &[CONCURRENT], context).await;
            assert_ids(server, knn(HNSW, 22.0), &[CONCURRENT], context).await;
        }
        let response = server.get("/admin/backup").await;
        response.assert_status_ok();
        let snapshot: Value = response.json();
        assert_eq!(
            snapshot["collections"][COLLECTION]["fields"][FLAT]["spec"]["backend"],
            json!("flat-cpu"),
            "{context}: Flat backend survives checkpoint",
        );
        assert_eq!(
            snapshot["collections"][COLLECTION]["fields"][HNSW]["spec"]["backend"],
            json!("hnsw-cpu"),
            "{context}: HNSW backend survives checkpoint",
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn first_checkpoint_from_fresh_collection_uses_zero_bases_sparse_deltas_and_preserves_post_capture_write(
    ) {
        let fixture = fixture();
        create_collection(&fixture.server).await;
        index_full(
            &fixture.server,
            LIVE,
            "live-keyword",
            101.0,
            "live-tag",
            "0000000000000a11",
            "first sparse live text",
            1.0,
            2.0,
        )
        .await;
        index_full(
            &fixture.server,
            DELETED,
            "deleted-keyword",
            111.0,
            "deleted-tag",
            "0000000000000a10",
            "first sparse deleted text",
            3.0,
            4.0,
        )
        .await;
        index_full(
            &fixture.server,
            REPLACED,
            "replace-old",
            201.0,
            "replace-old-tag",
            "0000000000000a12",
            "first sparse stale text",
            5.0,
            6.0,
        )
        .await;
        index_full(
            &fixture.server,
            EMPTY,
            "empty-keyword",
            211.0,
            "empty-tag",
            "0000000000000a15",
            "first sparse empty text",
            7.0,
            8.0,
        )
        .await;
        fixture
            .server
            .delete(&format!("/collections/{COLLECTION}/index/{DELETED}"))
            .await
            .assert_status(axum::http::StatusCode::NO_CONTENT);
        replace_without_set(
            &fixture.server,
            REPLACED,
            "replace-new",
            202.0,
            "0000000000000a13",
            "first sparse replaced text",
            11.0,
            12.0,
        )
        .await;
        replace_empty(&fixture.server, EMPTY).await;

        let cut = fixture.writer.applied_seq();
        assert!(
            cut > 0,
            "first sparse checkpoint captures real public writes"
        );
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        fixture.hold.arm(entered_tx, release_rx);
        let mut release = SyncRelease(Some(release_tx));
        let mut checkpoint = tokio::spawn({
            let checkpoint = fixture.checkpoint.clone();
            async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
        });
        let ready =
            tokio::task::spawn_blocking(move || entered_rx.recv_timeout(Duration::from_secs(30)))
                .await;
        if !matches!(ready, Ok(Ok(()))) {
            release.release();
            checkpoint.abort();
            let _ = checkpoint.await;
            panic!("first sparse checkpoint never reached the real SyncFile pause: {ready:?}");
        }

        let concurrent = tokio::time::timeout(
            Duration::from_secs(2),
            index_full(
                &fixture.server,
                CONCURRENT,
                "concurrent-keyword",
                303.0,
                "concurrent-tag",
                "0000000000000a14",
                "first sparse concurrent text",
                21.0,
                22.0,
            ),
        )
        .await;
        release.release();
        let checkpoint_error =
            match tokio::time::timeout(Duration::from_secs(30), &mut checkpoint).await {
                Ok(Ok(Ok(true))) => None,
                Ok(Ok(Ok(false))) => Some("checkpoint reported persisted=false".to_owned()),
                Ok(Ok(Err(error))) => Some(format!("checkpoint returned {error:#}")),
                Ok(Err(error)) => Some(format!("checkpoint task failed: {error}")),
                Err(_) => {
                    checkpoint.abort();
                    let _ = checkpoint.await;
                    Some("checkpoint did not finish after SyncFile release".to_owned())
                }
            };
        assert!(
            concurrent.is_ok(),
            "public all-field write must finish while first checkpoint is in file I/O: {concurrent:?}",
        );
        assert!(
            checkpoint_error.is_none(),
            "first sparse checkpoint finishes after release: {checkpoint_error:?}",
        );

        let first_generation = stage1_current_generation_dir(&fixture.root);
        let first_manifest = stage1_read_manifest(&first_generation);
        assert_eq!(
            first_manifest["checkpoint_sequence"],
            json!(cut),
            "paused first checkpoint retains the pre-pause capture cut",
        );
        assert_manifest(&first_generation, &first_manifest);
        let first_cold = fixture
            .store
            .load_current_generation()
            .expect("cold-open first sparse CURRENT")
            .expect("first sparse CURRENT");
        assert_eq!(first_cold.sequence, cut);
        let first_server =
            TestServer::new(router(AppState::open(first_cold.engine))).expect("first cold server");
        assert_state(&first_server, false, "first cold capture cut").await;
        assert_ids(
            &first_server,
            term(KW, "concurrent-keyword"),
            &[],
            "first cold capture cut excludes post-capture write",
        )
        .await;

        assert!(
            CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                .await
                .expect("publish post-capture write"),
            "second checkpoint persists the write that completed during first file I/O",
        );
        assert_state(&fixture.server, true, "live after second checkpoint").await;
        let latest = fixture
            .store
            .load_current_generation()
            .expect("cold-open latest first sparse CURRENT")
            .expect("latest first sparse CURRENT");
        assert!(latest.sequence > cut);
        let latest_server =
            TestServer::new(router(AppState::open(latest.engine))).expect("latest cold server");
        assert_state(&latest_server, true, "cold after second checkpoint").await;
    }
}
