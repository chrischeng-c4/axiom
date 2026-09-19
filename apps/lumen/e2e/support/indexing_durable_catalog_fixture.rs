//! Shared fixtures and assertions for focused durable-indexing contracts.

// Each focused target uses a different subset of these shared helpers.
#![allow(dead_code, unused_imports)]

pub(crate) use std::collections::{BTreeMap, HashMap};
#[cfg(unix)]
pub(crate) use std::os::unix::fs::MetadataExt;
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::sync::{Arc, Condvar, Mutex};
pub(crate) use std::time::Duration;

pub(crate) use anyhow::Result;
pub(crate) use axum_test::TestServer;
pub(crate) use futures::StreamExt;
pub(crate) use serde_json::{json, Map, Value};

pub(crate) use lumen::aof::{replay_aof_into, AofWriter};
pub(crate) use lumen::api::{router, AppState, CheckpointSink};
pub(crate) use lumen::auth::AuthConfig;
pub(crate) use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
pub(crate) use lumen::log_entry::RaftLogEntry;
pub(crate) use lumen::segment_rdb::SegmentRdbStore;
pub(crate) use lumen::storage::{Engine, FieldIndexSnapshot, SnapshotV1};
pub(crate) use lumen::types::{
    FieldValue, MatchOp, MatchQuery, QueryNode, RangeBound, RangeQuery, SearchRequest, TermQuery,
    TermsQuery,
};
pub(crate) use lumen::wal::{MemWal, SharedWal, WalLog};

#[path = "indexing_durable_fixture.rs"]
pub(crate) mod durable_fixture;

pub(crate) use durable_fixture::{
    checkpoint, create_schema, fixture, hit_ids, http_search, post_index, recover_from_checkpoint,
    local_checkpoint_sink, sorted_hit_ids, Fixture, COLLECTION,
};

pub(crate) const COMMON: &str = "common";
pub(crate) const RARE: &str = "rare";
pub(crate) const OTHER: &str = "other";
pub(crate) const DOCUMENTS: usize = 476;
pub(crate) const PREFIX_DOCUMENTS: usize = 365;

pub(crate) fn external_id(id: usize) -> String {
    format!("doc-{id:03}")
}

pub(crate) fn field_items(ids: impl Iterator<Item = usize>, field: &str) -> Vec<Value> {
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

pub(crate) fn corpus_items(field_major: bool, end: usize) -> Vec<Value> {
    corpus_items_range(field_major, 0, end)
}

pub(crate) fn corpus_items_range(field_major: bool, start: usize, end: usize) -> Vec<Value> {
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

pub(crate) async fn index_updates(server: &TestServer) {
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

pub(crate) fn snapshot(engine: &Arc<Engine>) -> SnapshotV1 {
    engine.snapshot().expect("engine snapshot")
}

pub(crate) fn collection(snapshot: &SnapshotV1) -> &lumen::storage::CollectionSnapshot {
    snapshot
        .collections
        .get(COLLECTION)
        .expect("docs collection")
}

/// The keyword field's forward column. As of snapshot format 2 it is the ONLY
/// representation on the wire: `from_snapshot` rebuilds the inverted index from
/// it, so the `terms` map this helper used to return alongside was a second
/// copy the reader discarded on arrival.
pub(crate) fn keyword_forward<'a>(snapshot: &'a SnapshotV1, field: &str) -> &'a HashMap<String, String> {
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
pub(crate) fn keyword_postings(snapshot: &SnapshotV1, field: &str, value: &str) -> usize {
    keyword_forward(snapshot, field)
        .values()
        .filter(|held| *held == value)
        .count()
}

pub(crate) fn number_forward<'a>(snapshot: &'a SnapshotV1, field: &str) -> &'a HashMap<String, f64> {
    let index = collection(snapshot)
        .fields
        .get(field)
        .expect("number field");
    let FieldIndexSnapshot::Number { forward, .. } = index else {
        panic!("{field} must be number")
    };
    forward
}

pub(crate) fn canonical(value: Value) -> Value {
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

pub(crate) fn digest(engine: &Arc<Engine>) -> Value {
    canonical(logical_snapshot_value(snapshot(engine)))
}

pub(crate) fn logical_snapshot_value(snapshot: SnapshotV1) -> Value {
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

pub(crate) fn assert_index_invariants(engine: &Arc<Engine>) {
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

pub(crate) fn assert_keyword_total(engine: &Arc<Engine>, value: &str, expected: u64) {
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

pub(crate) fn assert_queries(engine: &Arc<Engine>) {
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


pub(crate) async fn stage1_v2_root_with_predecessor() -> (tempfile::TempDir, PathBuf) {
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

pub(crate) fn stage1_current_generation_dir(root: &Path) -> PathBuf {
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

pub(crate) fn stage1_read_manifest(generation: &Path) -> Value {
    serde_json::from_slice(
        &std::fs::read(generation.join("_generation.json")).expect("read v2 generation manifest"),
    )
    .expect("decode v2 generation manifest")
}

pub(crate) fn stage1_write_manifest(generation: &Path, manifest: &Value) {
    let mut bytes = serde_json::to_vec_pretty(manifest).expect("encode mutated v2 manifest");
    bytes.push(b'\n');
    std::fs::write(generation.join("_generation.json"), bytes).expect("write mutated v2 manifest");
}

pub(crate) fn stage1_first_segment_mut(manifest: &mut Value) -> &mut Value {
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

pub(crate) fn stage1_duplicate_first_segment(manifest: &mut Value) {
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

pub(crate) fn stage1_assert_current_refuses_catalog_input(root: &Path, expected: &str, mutation: &str) {
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


pub(crate) fn stage1_catalog_collection_mut(manifest: &mut Value) -> &mut Value {
    manifest
        .get_mut("collections")
        .and_then(Value::as_array_mut)
        .expect("v2 catalog collections array")
        .first_mut()
        .expect("one catalogued collection")
}

pub(crate) fn stage1_catalog_segment_with_role_mut<'a>(manifest: &'a mut Value, role: &str) -> &'a mut Value {
    stage1_catalog_collection_mut(manifest)
        .get_mut("segments")
        .and_then(Value::as_array_mut)
        .expect("catalog segments array")
        .iter_mut()
        .find(|segment| segment["role"] == json!(role))
        .unwrap_or_else(|| panic!("catalog needs a {role} segment"))
}

pub(crate) fn stage1_assert_current_refuses_catalog_shape(root: &Path, mutation: &str) {
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


pub(crate) const STAGE1_REUSE_STABLE: &str = "reuse-stable";
pub(crate) const STAGE1_REUSE_CHANGED: &str = "reuse-changed";
pub(crate) const STAGE1_EPOCH_COLLECTION: &str = "reuse-epoch";
pub(crate) const STAGE1_EPOCH_AFTER_RESTART: &str = "reuse-epoch-after-restart";
pub(crate) const STAGE1_PROVENANCE_COLLECTION: &str = "reuse-provenance";

pub(crate) fn stage1_reuse_item(external_id: &str, value: &str) -> Value {
    json!({
        "external_id": external_id,
        "field": "kw",
        "value": value,
    })
}

pub(crate) async fn stage1_reuse_put_keyword_collection(server: &TestServer, collection: &str) {
    server
        .put(&format!("/collections/{collection}"))
        .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
}

pub(crate) async fn stage1_reuse_index(server: &TestServer, collection: &str, external_id: &str, value: &str) {
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
pub(crate) fn stage1_restore_legacy_base(engine: &Arc<Engine>, context: &str) {
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
pub(crate) fn stage1_assert_delta_sequence_order(
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

pub(crate) async fn stage1_reuse_term_ids(server: &TestServer, collection: &str, value: &str) -> Vec<String> {
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

pub(crate) async fn stage1_reuse_assert_term_ids(
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

pub(crate) fn stage1_reuse_current_name(root: &Path) -> String {
    std::fs::read_to_string(root.join("CURRENT"))
        .expect("read CURRENT")
        .strip_prefix("generation:")
        .expect("CURRENT names a generation")
        .trim()
        .to_owned()
}

pub(crate) fn stage1_reuse_catalog_collection<'a>(manifest: &'a Value, collection_id: &str) -> &'a Value {
    manifest["collections"]
        .as_array()
        .expect("v2 catalog collections array")
        .iter()
        .find(|collection| collection["collection_id"] == json!(collection_id))
        .unwrap_or_else(|| panic!("catalog must contain {collection_id}"))
}

pub(crate) fn stage1_reuse_collection_u64(collection: &Value, field: &str) -> u64 {
    collection[field]
        .as_u64()
        .unwrap_or_else(|| panic!("catalog {field} must be an unsigned integer"))
}

pub(crate) fn stage1_reuse_cold_load_named(root: &Path, generation_name: &str) -> (Arc<Engine>, u64) {
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

pub(crate) fn stage1_reuse_cold_load_current(root: &Path) -> (Arc<Engine>, u64) {
    let loaded = SegmentRdbStore::new(root)
        .expect("reopen current checkpoint root")
        .load_current_generation()
        .expect("current generation cold load succeeds")
        .expect("CURRENT names a generation");
    (loaded.engine, loaded.sequence)
}

#[cfg(unix)]
pub(crate) fn stage1_reuse_ref_key(segment: &Value) -> (String, Option<String>, u64) {
    (
        segment["role"].as_str().expect("segment role").to_owned(),
        segment["field"].as_str().map(str::to_owned),
        segment["ordinal"].as_u64().expect("segment ordinal"),
    )
}

#[cfg(unix)]
pub(crate) fn stage1_reuse_assert_hardlinked_collection(
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
pub(crate) fn stage1_reuse_assert_not_hardlinked_collection(
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


// Append after the frozen stage1 oracle in
// apps/lumen/e2e/indexing_durable_oracle.rs (base sha256:
// 940ac59d8f5f18e28e7822e4d2d4f53e10fda37045e952aec385728ed357993e).
//
// These cases use only existing v2 base segments. Each corrupts a manifest
// written by this process, requires `load_current_generation` to reject it,
// and uses `stage1_assert_current_refuses_catalog_shape` to preserve CURRENT.
// They intentionally do not assert implementation error text.

pub(crate) const STAGE1_CATALOG_LEFT: &str = "catalog-integrity-left";
pub(crate) const STAGE1_CATALOG_RIGHT: &str = "catalog-integrity-right";

pub(crate) fn stage1_catalog_collection_mut_by_id<'a>(manifest: &'a mut Value, id: &str) -> &'a mut Value {
    manifest["collections"]
        .as_array_mut()
        .expect("v2 catalog collections array")
        .iter_mut()
        .find(|collection| collection["collection_id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("catalog must contain {id}"))
}

pub(crate) fn stage1_catalog_segment_matches(segment: &Value, role: &str, field: Option<&str>) -> bool {
    segment["role"].as_str() == Some(role)
        && match field {
            Some(field) => segment["field"].as_str() == Some(field),
            None => segment["field"].is_null(),
        }
}

pub(crate) fn stage1_catalog_segment_mut<'a>(
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

pub(crate) fn stage1_catalog_remove_segment(collection: &mut Value, role: &str, field: Option<&str>) {
    let segments = collection["segments"]
        .as_array_mut()
        .expect("catalog segments array");
    let position = segments
        .iter()
        .position(|segment| stage1_catalog_segment_matches(segment, role, field))
        .unwrap_or_else(|| panic!("catalog must contain {role}/{field:?} segment"));
    segments.remove(position);
}

pub(crate) fn stage1_catalog_max_collection_generation(manifest: &Value) -> u64 {
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

pub(crate) async fn stage1_catalog_root_with_two_keyword_collections() -> (tempfile::TempDir, PathBuf) {
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


pub(crate) fn stage1_keyword_delta_local_rows<'a>(delta: &'a Value) -> &'a Map<String, Value> {
    let local_rows = delta["local_rows"].as_object();
    assert!(
        local_rows.is_some(),
        "a sparse keyword delta must include a local-row descriptor"
    );
    local_rows.expect("local-row descriptor after assertion")
}

pub(crate) fn stage1_keyword_delta_rows_path(generation: &Path, delta: &Value) -> PathBuf {
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

pub(crate) fn stage1_keyword_delta_rows_count(delta: &Value) -> u64 {
    let count = stage1_keyword_delta_local_rows(delta)["count"].as_u64();
    assert!(
        count.is_some(),
        "the local-row descriptor must carry a count"
    );
    count.expect("local-row count after assertion")
}

pub(crate) fn stage1_keyword_delta_read_rows(generation: &Path, delta: &Value) -> Vec<String> {
    let path = stage1_keyword_delta_rows_path(generation, delta);
    ciborium::from_reader(std::fs::File::open(&path).expect("open sparse local-row sidecar"))
        .expect("decode sparse local-row sidecar")
}

pub(crate) fn stage1_keyword_delta_write_rows(path: &Path, rows: &[String]) {
    let mut bytes = Vec::new();
    ciborium::into_writer(rows, &mut bytes).expect("encode mutated sparse local-row sidecar");
    std::fs::write(path, bytes).expect("write mutated sparse local-row sidecar");
}
