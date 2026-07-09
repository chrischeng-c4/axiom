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

// ---------------------------------------------------------------------------
// #1271: POST /collections:search (msearch-style batch search)
// ---------------------------------------------------------------------------

/// `took_ms`/`took_us` measure real elapsed time, so they legitimately differ
/// between the standalone call and the batched call even for an identical
/// query — zero them out before comparing the rest of the response
/// byte-for-byte.
fn zero_timing(mut v: Value) -> Value {
    if let Some(obj) = v.as_object_mut() {
        obj.insert("took_ms".to_string(), json!(0));
        obj.insert("took_us".to_string(), json!(0));
    }
    v
}

#[tokio::test]
async fn batch_search_returns_per_item_results_in_order_matching_single_search() {
    let s = server();

    s.put("/collections/users")
        .json(&json!({ "fields": { "tags": { "type": "keyword", "multi": true } } }))
        .await
        .assert_status_ok();
    s.post("/collections/users/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "tags", "value": ["rust", "db"] },
            { "external_id": "u2", "field": "tags", "value": ["go"] }
        ]}))
        .await
        .assert_status_ok();

    s.put("/collections/posts")
        .json(&json!({ "fields": { "body": { "type": "text" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/posts/index")
        .json(&json!({ "items": [
            { "external_id": "p1", "field": "body", "value": "rust engineer" },
            { "external_id": "p2", "field": "body", "value": "go backend" }
        ]}))
        .await
        .assert_status_ok();

    let users_query =
        json!({ "query": { "term": { "field": "tags", "value": "rust" } }, "limit": 10 });
    let posts_query =
        json!({ "query": { "match": { "field": "body", "text": "rust" } }, "limit": 10 });

    // What the single-collection endpoint returns for each pair — the batch
    // items must be byte-identical to these.
    let single_users: Value = s
        .post("/collections/users/search")
        .json(&users_query)
        .await
        .json();
    let single_posts: Value = s
        .post("/collections/posts/search")
        .json(&posts_query)
        .await
        .json();

    let mut item0 = users_query.clone();
    item0["collection"] = Value::String("users".into());
    let mut item1 = posts_query.clone();
    item1["collection"] = Value::String("posts".into());

    let resp = s
        .post("/collections:search")
        .json(&json!({ "searches": [item0, item1] }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let results = body["results"].as_array().expect("results array");
    assert_eq!(results.len(), 2, "same length as `searches`: {body}");

    assert_eq!(results[0]["status"], "ok");
    assert_eq!(
        zero_timing(results[0]["response"].clone()),
        zero_timing(single_users)
    );
    assert_eq!(results[1]["status"], "ok");
    assert_eq!(
        zero_timing(results[1]["response"].clone()),
        zero_timing(single_posts)
    );
}

#[tokio::test]
async fn batch_search_partial_failure_reports_per_item_error_with_ok_siblings() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({ "fields": { "tags": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/users/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "tags", "value": "rust" }
        ]}))
        .await
        .assert_status_ok();

    let ok_query = json!({ "collection": "users", "query": { "term": { "field": "tags", "value": "rust" } }, "limit": 10 });
    let missing_query = json!({ "collection": "missing", "query": { "term": { "field": "tags", "value": "rust" } }, "limit": 10 });

    let resp = s
        .post("/collections:search")
        .json(&json!({ "searches": [ok_query.clone(), missing_query, ok_query] }))
        .await;
    // Batch-level status stays 200 — one bad item never fails the batch.
    resp.assert_status_ok();
    let body: Value = resp.json();
    let results = body["results"].as_array().expect("results array");
    assert_eq!(results.len(), 3);

    assert_eq!(results[0]["status"], "ok");
    assert_eq!(results[0]["response"]["hits"][0]["external_id"], "u1");

    assert_eq!(results[1]["status"], "error");
    assert_eq!(results[1]["code"], "collection_not_found");
    assert!(results[1]["message"].is_string());

    assert_eq!(results[2]["status"], "ok");
    assert_eq!(results[2]["response"]["hits"][0]["external_id"], "u1");
}

#[tokio::test]
async fn batch_search_over_limit_returns_400() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({ "fields": { "tags": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    let item =
        json!({ "collection": "users", "query": { "term": { "field": "tags", "value": "x" } } });
    let searches: Vec<Value> = std::iter::repeat(item).take(33).collect();

    let resp = s
        .post("/collections:search")
        .json(&json!({ "searches": searches }))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn batch_search_honors_per_item_limit_sort_and_cursor_resume() {
    let s = server();
    s.put("/collections/products")
        .json(&json!({ "fields": {
            "category": { "type": "keyword" },
            "price": { "type": "number" }
        }}))
        .await
        .assert_status_ok();
    let mut items = vec![];
    for (eid, price) in [("p1", 10), ("p2", 20), ("p3", 30), ("p4", 40), ("p5", 50)] {
        items.push(json!({ "external_id": eid, "field": "category", "value": "tech" }));
        items.push(json!({ "external_id": eid, "field": "price", "value": price }));
    }
    s.post("/collections/products/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    let asc = json!({
        "collection": "products",
        "query": { "term": { "field": "category", "value": "tech" } },
        "limit": 2,
        "sort": [{ "field": "price", "order": "asc" }]
    });
    let desc = json!({
        "collection": "products",
        "query": { "term": { "field": "category", "value": "tech" } },
        "limit": 3,
        "sort": [{ "field": "price", "order": "desc" }]
    });

    let resp = s
        .post("/collections:search")
        .json(&json!({ "searches": [asc, desc] }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let results = body["results"].as_array().unwrap();

    // Item A: per-item `limit: 2` + ascending sort honored independently.
    let hits_a: Vec<&str> = results[0]["response"]["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap())
        .collect();
    assert_eq!(hits_a, vec!["p1", "p2"]);

    // Item B: a different per-item `limit: 3` + descending sort in the same
    // batch call — proves per-item options do not leak across items.
    let hits_b: Vec<&str> = results[1]["response"]["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap())
        .collect();
    assert_eq!(hits_b, vec!["p5", "p4", "p3"]);

    // A cursor returned from a batch item resumes correctly when passed back
    // for the same collection/item.
    let cursor = results[0]["response"]["cursor"]
        .as_str()
        .expect("item A cursor present (5 docs, page size 2)")
        .to_string();
    let mut asc_page2 = json!({
        "collection": "products",
        "query": { "term": { "field": "category", "value": "tech" } },
        "limit": 2,
        "sort": [{ "field": "price", "order": "asc" }]
    });
    asc_page2["cursor"] = Value::String(cursor);
    let resp2 = s
        .post("/collections:search")
        .json(&json!({ "searches": [asc_page2] }))
        .await;
    resp2.assert_status_ok();
    let body2: Value = resp2.json();
    let hits_a2: Vec<&str> = body2["results"][0]["response"]["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap())
        .collect();
    assert_eq!(hits_a2, vec!["p3", "p4"]);
}

/// #1292 AC1: a field the doc had before, but that is absent from a later
/// `docs:replace` item's `fields`, must be implicitly deleted — the doc's
/// indexed state becomes exactly `fields`, not a merge with what was there.
#[tokio::test]
async fn replace_docs_implicit_delete_field_absent_from_replacement() {
    let s = server();
    s.put("/collections/rows")
        .json(&json!({ "fields": {
            "title": { "type": "text" },
            "state": { "type": "keyword" }
        }}))
        .await
        .assert_status_ok();

    let find_open =
        || json!({ "query": { "term": { "field": "state", "value": "open" } }, "limit": 10 });

    s.put("/collections/rows/docs:replace")
        .json(&json!({ "docs": [
            { "external_id": "row-1", "fields": { "title": "hello world", "state": "open" } }
        ]}))
        .await
        .assert_status_ok();

    let resp = s.post("/collections/rows/search").json(&find_open()).await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(
        body["hits"][0]["external_id"], "row-1",
        "row-1 has state=open before replace: {body}"
    );

    // Replace again with `state` OMITTED.
    let resp = s
        .put("/collections/rows/docs:replace")
        .json(&json!({ "docs": [
            { "external_id": "row-1", "fields": { "title": "hello world" } }
        ]}))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["results"][0]["status"], "ok");
    assert_eq!(body["results"][0]["fields_written"], 1);
    assert_eq!(body["results"][0]["fields_skipped"], 0);

    // `state=open` no longer matches — the field was implicitly deleted.
    let resp = s.post("/collections/rows/search").json(&find_open()).await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(
        body["hits"].as_array().unwrap().len(),
        0,
        "state field must be implicitly deleted after replace omitted it: {body}"
    );

    // `title` is still indexed — only the omitted field was dropped, not
    // the whole doc.
    let resp = s
        .post("/collections/rows/search")
        .json(&json!({ "query": { "match": { "field": "title", "text": "hello" } }, "limit": 10 }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(
        body["hits"][0]["external_id"], "row-1",
        "title still indexed after replace: {body}"
    );
}

/// #1292 AC2: doc-level LWW using the caller's source-row `version` — a
/// strictly-older (or equal) version arriving later drops the *entire*
/// item, reported as its own `dropped` status. Replaying the exact same
/// request converges to the same visible doc state either way (idempotent
/// PUT semantics).
#[tokio::test]
async fn replace_docs_stale_version_dropped_and_replay_is_idempotent() {
    let s = server();
    s.put("/collections/rows")
        .json(&json!({ "fields": { "state": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    let find_open =
        || json!({ "query": { "term": { "field": "state", "value": "open" } }, "limit": 10 });

    let write_v10 = json!({ "docs": [
        { "external_id": "row-1", "version": 10, "fields": { "state": "open" } }
    ]});
    let resp = s
        .put("/collections/rows/docs:replace")
        .json(&write_v10)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["results"][0]["status"], "ok");

    // A strictly-older version arriving later drops the entire item.
    let stale = json!({ "docs": [
        { "external_id": "row-1", "version": 5, "fields": { "state": "closed" } }
    ]});
    let resp = s.put("/collections/rows/docs:replace").json(&stale).await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["results"][0]["status"], "dropped");
    assert_eq!(body["results"][0]["current_version"], 10);

    // The stale write never applied.
    let resp = s.post("/collections/rows/search").json(&find_open()).await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["hits"][0]["external_id"], "row-1");

    // Replaying the SAME request converges to the same state — idempotent
    // PUT semantics — even though a same-version replay itself reports
    // `dropped` (not strictly newer than what is already stored).
    let resp = s
        .put("/collections/rows/docs:replace")
        .json(&write_v10)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["results"][0]["status"], "dropped");
    assert_eq!(body["results"][0]["current_version"], 10);

    let resp = s.post("/collections/rows/search").json(&find_open()).await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(
        body["hits"][0]["external_id"], "row-1",
        "replay converges to the same state: {body}"
    );
}

/// #1292 AC3: `PUT /collections/{id}/docs/{external_id}` is single-resource
/// sugar — semantically identical to a one-item `docs:replace` batch,
/// unwrapped back into a bare per-item result.
#[tokio::test]
async fn replace_doc_single_resource_identical_to_one_item_batch() {
    let s = server();
    s.put("/collections/rows")
        .json(&json!({ "fields": {
            "title": { "type": "text" },
            "state": { "type": "keyword" }
        }}))
        .await
        .assert_status_ok();

    let batch_resp = s
        .put("/collections/rows/docs:replace")
        .json(&json!({ "docs": [
            { "external_id": "row-1", "fields": { "title": "hello", "state": "open" } }
        ]}))
        .await;
    batch_resp.assert_status_ok();
    let batch_body: Value = batch_resp.json();
    let batch_result = batch_body["results"][0].clone();

    let single_resp = s
        .put("/collections/rows/docs/row-2")
        .json(&json!({ "fields": { "title": "hello", "state": "open" } }))
        .await;
    single_resp.assert_status_ok();
    let single_body: Value = single_resp.json();
    assert_eq!(
        single_body, batch_result,
        "single-resource PUT must match the one-item batch result: {single_body} vs {batch_result}"
    );

    let resp = s
        .post("/collections/rows/search")
        .json(&json!({ "query": { "term": { "field": "state", "value": "open" } }, "limit": 10 }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let ids: Vec<&str> = body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap())
        .collect();
    assert!(
        ids.contains(&"row-1") && ids.contains(&"row-2"),
        "both docs indexed identically: {body}"
    );
}

/// #1292 AC4: one bad item (an unknown field) never fails the batch — the
/// batch-level status stays 200 and the failure is reported per-item
/// alongside `ok` siblings.
#[tokio::test]
async fn replace_docs_partial_failure_reports_per_item_error_with_ok_siblings() {
    let s = server();
    s.put("/collections/rows")
        .json(&json!({ "fields": { "state": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    let ok_item = json!({ "external_id": "row-1", "fields": { "state": "open" } });
    let bad_item = json!({ "external_id": "row-2", "fields": { "nope": "x" } });

    let resp = s
        .put("/collections/rows/docs:replace")
        .json(&json!({ "docs": [ok_item.clone(), bad_item, ok_item] }))
        .await;
    // Batch-level status stays 200 — one bad item never fails the batch.
    resp.assert_status_ok();
    let body: Value = resp.json();
    let results = body["results"].as_array().expect("results array");
    assert_eq!(results.len(), 3);

    assert_eq!(results[0]["status"], "ok");
    assert_eq!(results[1]["status"], "error");
    assert_eq!(results[1]["code"], "unknown_field");
    assert!(results[1]["message"].is_string());
    assert_eq!(results[2]["status"], "ok");
}

/// #1292 AC4: a batch over [`lumen::types::MAX_BATCH_REPLACE_SIZE`] items is
/// rejected with 400 before any item runs.
#[tokio::test]
async fn replace_docs_over_limit_returns_400() {
    let s = server();
    s.put("/collections/rows")
        .json(&json!({ "fields": { "state": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    let item = json!({ "external_id": "row-1", "fields": { "state": "open" } });
    let docs: Vec<Value> = std::iter::repeat(item).take(33).collect();

    let resp = s
        .put("/collections/rows/docs:replace")
        .json(&json!({ "docs": docs }))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
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

// ---------------------------------------------------------------------------
// QUERY (RFC 10008) — dual-registered POST twins (#1297, epic #1296 R1)
// ---------------------------------------------------------------------------

/// `http::Method` has no `QUERY` constant yet; construct it the same way
/// `src/api.rs`'s interim dispatch does.
fn query_method() -> axum::http::Method {
    axum::http::Method::from_bytes(b"QUERY").expect("QUERY is a valid method token")
}

/// Batch-search analog of `zero_timing`: `BatchSearchResponse` nests each
/// item's `SearchResponse` (with its own `took_ms`/`took_us`) inside
/// `results[i].response` — zero those out before comparing the rest of the
/// body byte-for-byte.
fn zero_batch_timing(mut v: Value) -> Value {
    if let Some(results) = v.get_mut("results").and_then(|r| r.as_array_mut()) {
        for item in results.iter_mut() {
            if let Some(response) = item.get_mut("response") {
                *response = zero_timing(response.take());
            }
        }
    }
    v
}

/// AC1: `QUERY /collections/{collection_id}` must return a byte-identical
/// body to its `POST /collections/{collection_id}/search` twin, for a plain
/// lexical (`term`) query.
#[tokio::test]
async fn query_single_search_byte_identical_to_post_twin_lexical() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/users/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "email", "value": "a@x.com" },
            { "external_id": "u2", "field": "email", "value": "b@y.com" }
        ]}))
        .await
        .assert_status_ok();

    let query = json!({
        "query": { "term": { "field": "email", "value": "a@x.com" } },
        "limit": 10
    });

    let post_resp = s.post("/collections/users/search").json(&query).await;
    post_resp.assert_status_ok();

    let query_resp = s
        .method(query_method(), "/collections/users")
        .json(&query)
        .await;
    query_resp.assert_status_ok();

    assert_eq!(
        zero_timing(query_resp.json::<Value>()),
        zero_timing(post_resp.json::<Value>()),
        "QUERY /collections/{{id}} must be byte-identical to its POST twin (modulo took_ms/took_us)"
    );
}

/// AC1: the kNN case of the QUERY/POST twin-parity contract.
#[tokio::test]
async fn query_single_search_byte_identical_to_post_twin_knn() {
    let s = server();
    s.put("/collections/items")
        .json(&json!({
            "fields": { "embedding": { "type": "vector", "dim": 3, "metric": "cosine" } }
        }))
        .await
        .assert_status_ok();
    s.post("/collections/items/index")
        .json(&json!({ "items": [
            { "external_id": "e1", "field": "embedding", "value": [1.0, 0.0, 0.0] },
            { "external_id": "e2", "field": "embedding", "value": [0.0, 1.0, 0.0] }
        ]}))
        .await
        .assert_status_ok();

    let query = json!({
        "query": { "knn": { "field": "embedding", "vector": [1.0, 0.0, 0.0], "k": 5 } },
        "limit": 5
    });

    let post_resp = s.post("/collections/items/search").json(&query).await;
    post_resp.assert_status_ok();

    let query_resp = s
        .method(query_method(), "/collections/items")
        .json(&query)
        .await;
    query_resp.assert_status_ok();

    assert_eq!(
        zero_timing(query_resp.json::<Value>()),
        zero_timing(post_resp.json::<Value>()),
        "QUERY /collections/{{id}} (knn) must be byte-identical to its POST twin (modulo took_ms/took_us)"
    );
}

/// AC1: `QUERY /collections` must return a byte-identical body to its `POST
/// /collections:search` twin for an identical `BatchSearchRequest`.
#[tokio::test]
async fn query_batch_search_byte_identical_to_post_twin() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({ "fields": { "tags": { "type": "keyword", "multi": true } } }))
        .await
        .assert_status_ok();
    s.post("/collections/users/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "tags", "value": ["rust", "db"] },
            { "external_id": "u2", "field": "tags", "value": ["go"] }
        ]}))
        .await
        .assert_status_ok();

    s.put("/collections/posts")
        .json(&json!({ "fields": { "body": { "type": "text" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/posts/index")
        .json(&json!({ "items": [
            { "external_id": "p1", "field": "body", "value": "rust engineer" },
            { "external_id": "p2", "field": "body", "value": "go backend" }
        ]}))
        .await
        .assert_status_ok();

    let batch = json!({ "searches": [
        {
            "collection": "users",
            "query": { "term": { "field": "tags", "value": "rust" } },
            "limit": 10
        },
        {
            "collection": "posts",
            "query": { "match": { "field": "body", "text": "rust" } },
            "limit": 10
        }
    ]});

    let post_resp = s.post("/collections:search").json(&batch).await;
    post_resp.assert_status_ok();

    let query_resp = s.method(query_method(), "/collections").json(&batch).await;
    query_resp.assert_status_ok();

    assert_eq!(
        zero_batch_timing(query_resp.json::<Value>()),
        zero_batch_timing(post_resp.json::<Value>()),
        "QUERY /collections must be byte-identical to its POST /collections:search twin (modulo took_ms/took_us)"
    );
}

/// AC2: `Content-Type` is mandatory on QUERY (RFC 10008) — both targets
/// reject a missing `Content-Type` with 415, same as their POST twins do via
/// the shared `Json` extractor.
#[tokio::test]
async fn query_missing_content_type_returns_415() {
    let s = server();
    let raw = serde_json::to_vec(&json!({
        "query": { "match_all": {} },
        "limit": 10
    }))
    .unwrap();

    let resp = s
        .method(query_method(), "/collections/users")
        .bytes(raw.clone().into())
        .await;
    resp.assert_status(axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let batch_raw = serde_json::to_vec(&json!({ "searches": [] })).unwrap();
    let resp = s
        .method(query_method(), "/collections")
        .bytes(batch_raw.into())
        .await;
    resp.assert_status(axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE);

    // A mismatched Content-Type is rejected the same way as a missing one.
    let resp = s
        .method(query_method(), "/collections/users")
        .bytes(raw.into())
        .content_type("text/plain")
        .await;
    resp.assert_status(axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

/// AC3: `OPTIONS`/`HEAD` on `/collections/{collection_id}` advertise
/// `Accept-Query: application/json` and list `QUERY` in `Allow`.
#[tokio::test]
async fn query_options_and_head_advertise_accept_query_on_collection_id() {
    let s = server();
    for method in [axum::http::Method::OPTIONS, axum::http::Method::HEAD] {
        let resp = s.method(method.clone(), "/collections/users").await;
        assert_eq!(
            resp.headers()
                .get("accept-query")
                .and_then(|v| v.to_str().ok()),
            Some("application/json"),
            "{method} /collections/{{id}} must advertise Accept-Query"
        );
        let allow = resp
            .headers()
            .get(axum::http::header::ALLOW)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(
            allow.contains("QUERY"),
            "{method} /collections/{{id}} Allow must include QUERY: {allow}"
        );
    }
}

/// AC3: `OPTIONS`/`HEAD` on `/collections` advertise `Accept-Query:
/// application/json` and list `QUERY` in `Allow`.
#[tokio::test]
async fn query_options_and_head_advertise_accept_query_on_collections() {
    let s = server();
    for method in [axum::http::Method::OPTIONS, axum::http::Method::HEAD] {
        let resp = s.method(method.clone(), "/collections").await;
        assert_eq!(
            resp.headers()
                .get("accept-query")
                .and_then(|v| v.to_str().ok()),
            Some("application/json"),
            "{method} /collections must advertise Accept-Query"
        );
        let allow = resp
            .headers()
            .get(axum::http::header::ALLOW)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(
            allow.contains("QUERY"),
            "{method} /collections Allow must include QUERY: {allow}"
        );
    }
}
// CODEGEN-END
