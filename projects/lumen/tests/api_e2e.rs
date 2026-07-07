// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! End-to-end HTTP integration tests.
//!
//! Drives the real axum router via `axum-test::TestServer`. These tests
//! double as executable documentation for the wire shapes — if the
//! README's API examples change, these tests will need to change too.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum_test::TestServer;
use lumen::coordinator::WriteCoordinator;
use lumen::routing::{document_shard_index, EngineShardSearch, EngineShardWrite};
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
};
use lumen::wal::MemWal;
use serde_json::{json, Value};

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

fn server_with_engine() -> (TestServer, Arc<lumen::storage::Engine>) {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine.clone()));
    (TestServer::new(app).expect("test server"), engine)
}

#[tokio::test]
async fn health_and_ready() {
    let s = server();
    s.get("/healthz").await.assert_status_ok();
    let ready = s.get("/readyz").await;
    ready.assert_status_ok();
    assert_eq!(ready.text(), "ok");
}

#[tokio::test]
async fn readyz_reports_draining() {
    let (s, engine) = server_with_engine();
    engine.start_drain();
    let ready = s.get("/readyz").await;
    ready.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(ready.text(), "draining");
}

#[tokio::test]
async fn version_reports_build_provenance() {
    let s = server();
    let resp = s.get("/version").await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    // version is the crate version, stamped via env!("CARGO_PKG_VERSION").
    assert_eq!(
        body["version"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "GET /version must report the crate version; body = {body}"
    );
    // git_sha + built_at are always present (degrading to "unknown" off-git).
    assert!(body["git_sha"].is_string(), "git_sha missing in {body}");
    assert!(body["built_at"].is_string(), "built_at missing in {body}");
}

#[tokio::test]
async fn create_collection_and_index_keyword_then_search() {
    let s = server();

    s.put("/collections/users")
        .json(&json!({
            "fields": {
                "email": { "type": "keyword" }
            }
        }))
        .await
        .assert_status_ok();

    s.post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": "u1", "field": "email", "value": "a@x.com" },
                { "external_id": "u2", "field": "email", "value": "b@y.com" },
                { "external_id": "u3", "field": "email", "value": "a@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    let resp = s
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 2);
    let eids: Vec<&str> = body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap())
        .collect();
    assert_eq!(eids, vec!["u1", "u3"]);
}

#[tokio::test]
async fn search_can_use_injected_sharded_backend() {
    let shard_a = test_search_shard([("u1", "a@x.com", 40), ("u2", "b@y.com", 30)]);
    let shard_b = test_search_shard([("u3", "a@x.com", 20)]);
    let state = lumen::api::AppState::open(Arc::new(lumen::storage::Engine::new()))
        .with_search_backend(Arc::new(EngineShardSearch::new(vec![shard_a, shard_b])));
    let s = TestServer::new(lumen::api::router(state)).expect("test server");

    let resp = s
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "sort": [{ "field": "age", "order": "asc" }],
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 2);
    let eids: Vec<&str> = body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap())
        .collect();
    assert_eq!(eids, vec!["u3", "u1"]);
}

#[tokio::test]
async fn index_can_use_injected_sharded_write_backend() {
    let engines: Vec<Arc<lumen::storage::Engine>> = (0..2)
        .map(|_| Arc::new(lumen::storage::Engine::new()))
        .collect();
    let writers = engines
        .iter()
        .map(|engine| WriteCoordinator::start(Arc::new(MemWal::new()), engine.clone()))
        .collect();
    let state = lumen::api::AppState::open(Arc::new(lumen::storage::Engine::new()))
        .with_search_backend(Arc::new(EngineShardSearch::new(engines.clone())))
        .with_write_backend(Arc::new(EngineShardWrite::new(writers)));
    let s = TestServer::new(lumen::api::router(state)).expect("test server");

    s.put("/collections/users")
        .json(&json!({
            "fields": {
                "email": { "type": "keyword" },
                "age": { "type": "number" }
            }
        }))
        .await
        .assert_status_ok();

    let eid0 = eid_for_document_shard("users", 0, 2);
    let eid1 = eid_for_document_shard("users", 1, 2);
    let resp = s
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": eid0, "field": "email", "value": "a@x.com" },
                { "external_id": eid0, "field": "age", "value": 40 },
                { "external_id": eid1, "field": "email", "value": "a@x.com" },
                { "external_id": eid1, "field": "age", "value": 20 }
            ]
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["indexed"], 4);

    for (expected_shard, eid) in [(0, eid0.as_str()), (1, eid1.as_str())] {
        let shard = document_shard_index("users", eid, 2);
        assert_eq!(shard, expected_shard);
        let found = engines[expected_shard]
            .search(
                "users",
                serde_json::from_value(json!({
                    "query": { "term": { "field": "email", "value": "a@x.com" } },
                    "limit": 10
                }))
                .unwrap(),
            )
            .unwrap();
        assert!(
            found.hits.iter().any(|hit| hit.external_id == eid),
            "expected {eid} on shard {expected_shard}"
        );
    }

    let resp = s
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "sort": [{ "field": "age", "order": "asc" }],
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 2);
    let eids: Vec<&str> = body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap())
        .collect();
    assert_eq!(eids, vec![eid1.as_str(), eid0.as_str()]);
}

fn eid_for_document_shard(collection_id: &str, shard: usize, shard_count: usize) -> String {
    for i in 0..10_000 {
        let eid = format!("u{shard}_{i}");
        if document_shard_index(collection_id, &eid, shard_count) == shard {
            return eid;
        }
    }
    panic!("could not find eid for shard {shard}");
}

fn test_search_shard<const N: usize>(docs: [(&str, &str, i32); N]) -> Arc<lumen::storage::Engine> {
    let engine = Arc::new(lumen::storage::Engine::new());
    let mut fields = BTreeMap::new();
    fields.insert(
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
    fields.insert(
        "age".to_string(),
        FieldSpec {
            field_type: FieldType::Number,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    engine
        .create_collection("users", CreateCollectionRequest { fields })
        .unwrap();
    engine
        .index(
            "users",
            IndexRequest {
                items: docs
                    .into_iter()
                    .flat_map(|(external_id, email, age)| {
                        [
                            IndexItem {
                                external_id: external_id.to_string(),
                                field: "email".to_string(),
                                value: FieldValue::String(email.to_string()),
                                version: None,
                            },
                            IndexItem {
                                external_id: external_id.to_string(),
                                field: "age".to_string(),
                                value: FieldValue::Number(age as f64),
                                version: None,
                            },
                        ]
                    })
                    .collect(),
                request_id: None,
            },
        )
        .unwrap();
    engine
}

#[tokio::test]
async fn duplicates_finds_groups() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    let mut items = vec![];
    for (i, email) in ["a@x.com", "a@x.com", "a@x.com", "b@y.com", "b@y.com"]
        .iter()
        .enumerate()
    {
        items.push(json!({
            "external_id": format!("u{i}"),
            "field": "email",
            "value": email
        }));
    }
    s.post("/collections/users/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    let resp = s
        .post("/collections/users/duplicates")
        .json(&json!({ "field": "email", "min_group_size": 2 }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let groups = body["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["external_ids"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn match_query_text_and_range() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({
            "fields": {
                "bio":   { "type": "text" },
                "age":   { "type": "number" }
            }
        }))
        .await
        .assert_status_ok();

    s.post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": "u1", "field": "bio", "value": "senior rust engineer" },
                { "external_id": "u1", "field": "age", "value": 30 },
                { "external_id": "u2", "field": "bio", "value": "junior rust engineer" },
                { "external_id": "u2", "field": "age", "value": 22 },
                { "external_id": "u3", "field": "bio", "value": "designer" },
                { "external_id": "u3", "field": "age", "value": 28 }
            ]
        }))
        .await
        .assert_status_ok();

    let resp = s
        .post("/collections/users/search")
        .json(&json!({
            "query": { "and": [
                { "match": { "field": "bio", "text": "rust engineer", "op": "and" } },
                { "range": { "field": "age", "gte": 25, "lt": 40 } }
            ]},
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "u1");
}

#[tokio::test]
async fn keyword_multi_sugar_becomes_set() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({
            "fields": {
                "tags": { "type": "keyword", "multi": true }
            }
        }))
        .await
        .assert_status_ok();

    s.post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": "u1", "field": "tags", "value": ["rust", "db"] },
                { "external_id": "u2", "field": "tags", "value": ["go"] }
            ]
        }))
        .await
        .assert_status_ok();

    let resp = s
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "tags", "value": "rust" } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "u1");
}

#[tokio::test]
async fn unknown_collection_404() {
    let s = server();
    let resp = s
        .post("/collections/missing/search")
        .json(&json!({ "query": { "term": { "field": "x", "value": "y" } }, "limit": 1 }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn type_mismatch_422() {
    let s = server();
    s.put("/collections/x")
        .json(&json!({ "fields": { "n": { "type": "number" } } }))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/x/index")
        .json(&json!({ "items": [
            { "external_id": "a", "field": "n", "value": "not a number" }
        ]}))
        .await;
    resp.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn unsupported_sort_shape_returns_400() {
    let s = server();
    s.put("/collections/posts")
        .json(&json!({ "fields": { "body": { "type": "text" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/posts/index")
        .json(&json!({ "items": [
            { "external_id": "p1", "field": "body", "value": "rust search" }
        ]}))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/posts/search")
        .json(&json!({
            "query": { "match": { "field": "body", "text": "rust" } },
            "sort": [{ "field": "body", "order": "asc" }],
            "limit": 10
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let body: Value = resp.json();
    assert_eq!(body["error"], "unsupported_sort");
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|message| message.contains("not sortable")),
        "body = {body}"
    );
}

#[tokio::test]
async fn idempotent_index_request_dedups() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let body = json!({
        "items": [{ "external_id": "u1", "field": "e", "value": "a@x.com" }],
        "request_id": "req-42"
    });
    let r1 = s.post("/collections/u/index").json(&body).await;
    let r2 = s.post("/collections/u/index").json(&body).await;
    let b1: Value = r1.json();
    let b2: Value = r2.json();
    assert_eq!(b1["indexed"], 1);
    assert_eq!(b2["indexed"], 0);
}

#[tokio::test]
async fn delete_external_id_removes_all_fields() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({
            "fields": {
                "email": { "type": "keyword" },
                "bio":   { "type": "text" }
            }
        }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "email", "value": "a@x.com" },
            { "external_id": "u1", "field": "bio",   "value": "rust engineer" }
        ]}))
        .await
        .assert_status_ok();

    let del = s.delete("/collections/u/index/u1").await;
    del.assert_status(axum::http::StatusCode::NO_CONTENT);

    let resp = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    let body: Value = resp.json();
    assert_eq!(body["total"], 0);
}

#[tokio::test]
async fn bm25_ranks_higher_tf_first() {
    let s = server();
    s.put("/collections/posts")
        .json(&json!({ "fields": { "body": { "type": "text" } } }))
        .await
        .assert_status_ok();
    // u1 mentions rust twice, u2 once, u3 not at all.
    s.post("/collections/posts/index")
        .json(&json!({
            "items": [
                { "external_id": "u1", "field": "body", "value": "rust rust is great" },
                { "external_id": "u2", "field": "body", "value": "rust is okay" },
                { "external_id": "u3", "field": "body", "value": "python is great" }
            ]
        }))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/posts/search")
        .json(&json!({
            "query": { "match": { "field": "body", "text": "rust", "op": "and" } },
            "limit": 10
        }))
        .await;
    let body: Value = resp.json();
    assert_eq!(body["total"], 2);
    let hits = body["hits"].as_array().unwrap();
    assert_eq!(hits[0]["external_id"], "u1");
    assert_eq!(hits[1]["external_id"], "u2");
    // Higher TF must produce a strictly higher score.
    let s1 = hits[0]["score"].as_f64().unwrap();
    let s2 = hits[1]["score"].as_f64().unwrap();
    assert!(s1 > s2, "expected u1.score > u2.score, got {s1} <= {s2}");
}

#[tokio::test]
async fn metrics_exposes_prometheus_text() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "e", "value": "a@x.com" }
        ]}))
        .await
        .assert_status_ok();
    let resp = s.get("/metrics").await;
    resp.assert_status_ok();
    let body = resp.text();
    for name in [
        "lumen_index_writes_total",
        "lumen_collections_created_total",
        "lumen_search_requests_total",
        "lumen_storage_bytes",
    ] {
        assert!(body.contains(name), "missing {name} in:\n{body}");
    }
    // Verify the indexed count actually moved.
    assert!(body.contains("lumen_index_writes_total 1"));
}

#[tokio::test]
async fn upsert_adds_new_fields_online() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    // Reapply with one new field. Should upgrade in place (version bump).
    let resp = s
        .put("/collections/u")
        .json(&json!({
            "fields": {
                "email": { "type": "keyword" },
                "age":   { "type": "number" }
            }
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["fields_count"], 2);
    assert_eq!(body["version"], 2);

    // New field is queryable immediately.
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "age", "value": 30 }
        ]}))
        .await
        .assert_status_ok();
    let r = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "range": { "field": "age", "gte": 18 } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
}

#[tokio::test]
async fn upsert_rejects_incompatible_redeclaration() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "x": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let resp = s
        .put("/collections/u")
        .json(&json!({ "fields": { "x": { "type": "number" } } }))
        .await;
    resp.assert_status_failure();
}

#[tokio::test]
async fn bulk_limit_rejected_413() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let items: Vec<_> = (0..lumen::storage::MAX_INDEX_ITEMS + 1)
        .map(|i| {
            json!({
                "external_id": format!("u{i}"),
                "field": "e",
                "value": format!("v{i}")
            })
        })
        .collect();
    let resp = s
        .post("/collections/u/index")
        .json(&json!({ "items": items }))
        .await;
    resp.assert_status(axum::http::StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn openapi_spec_served() {
    let s = server();
    let resp = s.get("/openapi.json").await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["info"]["title"], "lumen");
    // Has the new collections-based paths.
    assert!(body["paths"]["/healthz"].is_object());
    assert!(body["paths"]["/readyz"].is_object());
    assert!(body["paths"]["/metrics"].is_object());
    assert!(body["paths"]["/collections/{collection_id}/index"].is_object());
    assert!(body["paths"]["/collections/{collection_id}/search"].is_object());
    assert!(body["paths"]["/collections/{collection_id}/duplicates"].is_object());
}
// CODEGEN-END
