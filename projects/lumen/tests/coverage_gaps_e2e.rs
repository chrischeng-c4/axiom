// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Fills the e2e gaps surfaced by the README coverage audit.
//!
//! Each test is named after the README §-section + capability so the
//! coverage matrix in README's Status section can be traced row-by-row.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::storage::Engine;

fn server() -> TestServer {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::open(engine));
    TestServer::new(app).expect("test server")
}

// ---------------------------------------------------------------------------
// §1 Query: terms (multi-value)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s1_terms_query_matches_any_listed_value() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "color": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "a", "field": "color", "value": "red" },
            { "external_id": "b", "field": "color", "value": "blue" },
            { "external_id": "c", "field": "color", "value": "green" }
        ]}))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "terms": { "field": "color", "values": ["red", "green"] } },
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
    assert!(eids.contains(&"a") && eids.contains(&"c"));
}

// ---------------------------------------------------------------------------
// §1 Boolean: or / not
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s1_or_unions_children() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({
            "fields": {
                "tag":   { "type": "keyword" },
                "level": { "type": "number" }
            }
        }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "a", "field": "tag",   "value": "rust" },
            { "external_id": "a", "field": "level", "value": 1 },
            { "external_id": "b", "field": "tag",   "value": "go" },
            { "external_id": "b", "field": "level", "value": 5 },
            { "external_id": "c", "field": "tag",   "value": "python" },
            { "external_id": "c", "field": "level", "value": 99 }
        ]}))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "or": [
                { "term":  { "field": "tag",   "value": "rust" } },
                { "range": { "field": "level", "gte": 50 } }
            ]},
            "limit": 10
        }))
        .await;
    let body: Value = resp.json();
    assert_eq!(body["total"], 2, "expected a (rust) ∪ c (level≥50): {body}");
}

#[tokio::test]
async fn s1_not_inverts_against_universe() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "tag": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "a", "field": "tag", "value": "rust" },
            { "external_id": "b", "field": "tag", "value": "go" },
            { "external_id": "c", "field": "tag", "value": "python" }
        ]}))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "not": { "term": { "field": "tag", "value": "rust" } } },
            "limit": 10
        }))
        .await;
    let body: Value = resp.json();
    assert_eq!(body["total"], 2);
    let eids: Vec<&str> = body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap())
        .collect();
    assert!(eids.contains(&"b") && eids.contains(&"c"));
}

// ---------------------------------------------------------------------------
// §1 Cursor pagination
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s1_cursor_paginates_across_pages() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "color": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    // 25 docs all matching the same term so we can walk a 3-page cursor.
    let mut items = vec![];
    for i in 0..25 {
        items.push(json!({
            "external_id": format!("u{:02}", i),
            "field": "color",
            "value": "red"
        }));
    }
    s.post("/collections/u/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    let q = json!({
        "query": { "term": { "field": "color", "value": "red" } },
        "limit": 10
    });
    let p1: Value = s.post("/collections/u/search").json(&q).await.json();
    assert_eq!(p1["total"], 25);
    assert_eq!(p1["hits"].as_array().unwrap().len(), 10);
    let cursor1 = p1["cursor"].as_str().expect("page 1 has cursor");

    let p2: Value = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "color", "value": "red" } },
            "limit": 10,
            "cursor": cursor1
        }))
        .await
        .json();
    assert_eq!(p2["hits"].as_array().unwrap().len(), 10);
    let cursor2 = p2["cursor"].as_str().expect("page 2 has cursor");

    let p3: Value = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "color", "value": "red" } },
            "limit": 10,
            "cursor": cursor2
        }))
        .await
        .json();
    assert_eq!(p3["hits"].as_array().unwrap().len(), 5);
    assert!(p3["cursor"].is_null(), "final page has no cursor");

    // Pages disjoint, in stable order.
    let mut all_eids: Vec<String> = vec![];
    for page in [&p1, &p2, &p3] {
        for h in page["hits"].as_array().unwrap() {
            all_eids.push(h["external_id"].as_str().unwrap().into());
        }
    }
    let mut sorted = all_eids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 25, "pages must be disjoint");
}

// ---------------------------------------------------------------------------
// §1 Duplicates on number / set
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s1_duplicates_on_number_field() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "age": { "type": "number" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "a", "field": "age", "value": 30 },
            { "external_id": "b", "field": "age", "value": 30 },
            { "external_id": "c", "field": "age", "value": 25 }
        ]}))
        .await
        .assert_status_ok();
    let body: Value = s
        .post("/collections/u/duplicates")
        .json(&json!({ "field": "age" }))
        .await
        .json();
    let groups = body["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["external_ids"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn s1_duplicates_on_set_field_per_element() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "tags": { "type": "set" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "a", "field": "tags", "value": ["rust", "db"] },
            { "external_id": "b", "field": "tags", "value": ["rust", "ml"] },
            { "external_id": "c", "field": "tags", "value": ["go"] }
        ]}))
        .await
        .assert_status_ok();
    let body: Value = s
        .post("/collections/u/duplicates")
        .json(&json!({ "field": "tags" }))
        .await
        .json();
    let groups = body["groups"].as_array().unwrap();
    // "rust" appears for a + b.
    let rust = groups
        .iter()
        .find(|g| g["value"] == "rust")
        .expect("rust group");
    assert_eq!(rust["external_ids"].as_array().unwrap().len(), 2);
}

// ---------------------------------------------------------------------------
// §1 ngram analyzer e2e
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s1_ngram_analyzer_matches_substring() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({
            "fields": { "code": { "type": "text", "analyzer": "ngram" } }
        }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "a", "field": "code", "value": "abcdef" },
            { "external_id": "b", "field": "code", "value": "ghijkl" }
        ]}))
        .await
        .assert_status_ok();
    // "bcd" exists as a trigram only in document a's "abcdef".
    let body: Value = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "match": { "field": "code", "text": "bcd" } },
            "limit": 10
        }))
        .await
        .json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "a");
}

// ---------------------------------------------------------------------------
// §5 /debug/cluster (local single-pod degenerate view)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s5_debug_cluster_returns_local_view() {
    let s = server();
    let body: Value = s.get("/debug/cluster").await.json();
    assert_eq!(body["pod_name"], "local");
    assert_eq!(body["role"], "leader");
    assert_eq!(body["shard_index"], 0);
    assert!(body["peers"].is_array());
}

// ---------------------------------------------------------------------------
// §8 Swagger UI /docs (FastAPI convention — interactive "Try it out")
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s8_swagger_docs_endpoint_returns_html() {
    let s = server();
    let resp = s.get("/docs").await;
    resp.assert_status_ok();
    let text = resp.text();
    assert!(
        text.contains("SwaggerUIBundle"),
        "expected Swagger UI HTML, got: {text}"
    );
    // Must point at the live spec so "Try it out" hits this pod.
    assert!(text.contains("/openapi.json"));
}

// ---------------------------------------------------------------------------
// §7 POST /admin/backup/local
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s7_admin_backup_local_writes_snapshot_to_disk() {
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

    let dir = std::env::temp_dir().join(format!("lumen-backup-e2e-{}", std::process::id()));
    let body: Value = s
        .post("/admin/backup/local")
        .json(&json!({ "path": dir.to_str().unwrap(), "prefix": "snap" }))
        .await
        .json();
    let key = body["key"].as_str().unwrap();
    let written = dir.join(key);
    assert!(
        written.exists(),
        "backup file {} missing",
        written.display()
    );
    let bytes = std::fs::read(&written).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["version"], 1);
    assert!(parsed["collections"]["u"].is_object());
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// §3 Read-consistency header accepted on /search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s3_read_consistency_header_accepted_on_search() {
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

    for level in ["leader", "any", "bounded(250)"] {
        let resp = s
            .post("/collections/u/search")
            .add_header("x-read-consistency", level)
            .json(&json!({
                "query": { "term": { "field": "e", "value": "a@x.com" } },
                "limit": 10
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(
            body["total"], 1,
            "consistency={level} should still return data"
        );
    }
}

// ---------------------------------------------------------------------------
// §1 Bulk write byte-limit (axum default body cap rejects > 2 MiB; we
// assert via a synthetic payload). Real budget is 10 MB per README §1 —
// raise once the server explicitly raises the body limit. For now this
// test pins the current behavior so a regression is loud.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s1_oversized_payload_rejected_with_413_or_422() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    // ~10 MiB body — over the explicit 8 MiB DefaultBodyLimit on the router.
    let blob = "x".repeat(5 * 1024 * 1024);
    let items = vec![
        json!({ "external_id": "u0", "field": "e", "value": blob.clone() }),
        json!({ "external_id": "u1", "field": "e", "value": blob }),
    ];
    let resp = s
        .post("/collections/u/index")
        .json(&json!({ "items": items }))
        .await;
    let st = resp.status_code().as_u16();
    assert!(
        st == 413 || st == 422 || st == 400,
        "expected 413 (size) / 422 / 400, got {st}"
    );
}
// CODEGEN-END
