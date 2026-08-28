use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use axum_test::TestServer;
use serde_json::{json, Map, Value};

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::{Engine, FieldIndexSnapshot, SnapshotV1};
use lumen::types::{
    FieldValue, MatchOp, MatchQuery, QueryNode, RangeBound, RangeQuery, SearchRequest, TermQuery,
    TermsQuery,
};
use lumen::wal::{MemWal, SharedWal};

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

fn snapshot(engine: &Arc<Engine>) -> SnapshotV1 {
    engine.snapshot().expect("engine snapshot")
}

fn collection(snapshot: &SnapshotV1) -> &lumen::storage::CollectionSnapshot {
    snapshot
        .collections
        .get(COLLECTION)
        .expect("docs collection")
}

fn keyword_indexes<'a>(
    snapshot: &'a SnapshotV1,
    field: &str,
) -> (
    &'a BTreeMap<String, BTreeSet<String>>,
    &'a HashMap<String, String>,
) {
    let index = collection(snapshot)
        .fields
        .get(field)
        .expect("keyword field");
    let FieldIndexSnapshot::Keyword { terms, forward, .. } = index else {
        panic!("{field} must be keyword")
    };
    (terms, forward)
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
    let (terms, forward) = keyword_indexes(&snap, "kw");
    if !terms.is_empty() {
        assert_eq!(terms.get(COMMON).map(BTreeSet::len), Some(101));
        assert_eq!(terms.get(RARE).map(BTreeSet::len), Some(1));
        assert_eq!(terms.get(OTHER).map(BTreeSet::len), Some(6));
    }
    assert_eq!(
        forward.values().filter(|value| *value == COMMON).count(),
        466
    );
    assert_eq!(forward.values().filter(|value| *value == RARE).count(), 1);
    assert_eq!(forward.values().filter(|value| *value == OTHER).count(), 6);
    assert_eq!(forward.get("doc-466"), Some(&"台北市/大安區".to_string()));
    assert_eq!(forward.get("doc-468"), Some(&"🙂".to_string()));
    assert!(!forward.contains_key("doc-467"));
    if let Some(other) = terms.get(OTHER) {
        assert!(!other.contains("doc-467"));
    }

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
    let (prefix_terms, _) = keyword_indexes(&prefix_snapshot, "kw");
    assert_eq!(
        prefix_terms.get(COMMON).map(BTreeSet::len),
        Some(PREFIX_DOCUMENTS)
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
    let (first_terms, _) = keyword_indexes(&first_snapshot, "kw");
    assert!(
        first_terms.is_empty(),
        "sealed postings move out of the RAM snapshot"
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
