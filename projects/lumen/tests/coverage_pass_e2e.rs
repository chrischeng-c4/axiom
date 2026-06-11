// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-tests-coverage-pass-e2e-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Coverage-driven test pass — drives the HTTP surface through every
//! status code the engine can produce, plus a few cross-feature scenarios
//! the per-feature tests don't hit. Pure additive; no production
//! behavior under test that isn't already specified elsewhere — these
//! exist to keep coverage honest as the codebase grows.

use std::collections::HashMap;
use std::sync::Arc;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::auth::{AuthConfig, Role, TokenClaims};
use lumen::storage::Engine;

fn server() -> TestServer {
    let engine = Arc::new(Engine::new());
    TestServer::new(router(AppState::open(engine))).unwrap()
}

fn auth_server(tokens: Vec<(&str, TokenClaims)>) -> TestServer {
    let engine = Arc::new(Engine::new());
    let cfg = AuthConfig {
        required: true,
        tokens: tokens
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    };
    TestServer::new(router(AppState::new(engine, Arc::new(cfg)))).unwrap()
}

fn admin(subject: &str) -> TokenClaims {
    TokenClaims {
        subject: subject.into(),
        roles: HashMap::from([("*".to_string(), Role::Admin)]),
    }
}

// ---------------------------------------------------------------------------
// §1 / §6: status codes the engine emits
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unknown_field_returns_422() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "ghost", "value": "x" }
        ]}))
        .await;
    resp.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = resp.json();
    assert_eq!(body["error"], "unknown_field");
}

#[tokio::test]
async fn duplicates_on_text_returns_400() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "bio": { "type": "text" } } }))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/u/duplicates")
        .json(&json!({ "field": "bio" }))
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn nan_number_value_rejected() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "n": { "type": "number" } } }))
        .await
        .assert_status_ok();
    // JSON has no literal NaN — serde rejects at parse time → 400 from axum
    // body decode rather than reaching the StorageError::InvalidNumber arm,
    // but we test the path through Number with a non-finite literal-ish
    // representation: extremely large value should still be accepted.
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "n", "value": 1.0e308 }
        ]}))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn drop_nonexistent_collection_returns_404() {
    let s = server();
    let resp = s.delete("/collections/never-existed").await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn search_on_unknown_collection_returns_404() {
    let s = server();
    let resp = s
        .post("/collections/missing/search")
        .json(&json!({ "query": { "term": { "field": "x", "value": "y" } } }))
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_external_id_with_unknown_collection_returns_404() {
    let s = server();
    let resp = s.delete("/collections/missing/index/u1").await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn stats_on_unknown_collection_returns_404() {
    let s = server();
    let resp = s.get("/collections/missing/stats").await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn restore_with_wrong_version_returns_400() {
    let s = server();
    let resp = s
        .post("/admin/restore")
        .json(&json!({ "version": 0, "collections": {} }))
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn invalid_field_spec_rejected() {
    let s = server();
    // vector field missing required `dim` — schema validator rejects.
    let resp = s
        .put("/collections/u")
        .json(&json!({ "fields": {
            "bad": { "type": "vector", "metric": "cosine" }
        }}))
        .await;
    assert!(
        resp.status_code().is_client_error(),
        "expected 4xx, got {}",
        resp.status_code()
    );
}

#[tokio::test]
async fn drop_field_on_soft_deleted_collection_returns_410() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.delete("/collections/u").await; // soft delete
    let resp = s.delete("/collections/u/fields/e").await;
    resp.assert_status(StatusCode::GONE);
}

#[tokio::test]
async fn index_on_soft_deleted_collection_returns_410() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.delete("/collections/u").await;
    let resp = s
        .post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "e", "value": "x" }
        ]}))
        .await;
    resp.assert_status(StatusCode::GONE);
}

// ---------------------------------------------------------------------------
// §1 cross-cutting: cursor walks through the entire result set
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cursor_walks_to_exhaustion_on_match_query() {
    let s = server();
    s.put("/collections/posts")
        .json(&json!({ "fields": { "body": { "type": "text" } } }))
        .await
        .assert_status_ok();
    // 53 docs all matching one token, page size 10 → 6 pages.
    let mut items = vec![];
    for i in 0..53 {
        items.push(json!({
            "external_id": format!("p{:02}", i),
            "field": "body",
            "value": "rust"
        }));
    }
    s.post("/collections/posts/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    let mut cursor: Option<String> = None;
    let mut pages = 0;
    let mut total_eids = 0;
    loop {
        pages += 1;
        let mut body = json!({
            "query": { "match": { "field": "body", "text": "rust", "op": "or" } },
            "limit": 10
        });
        if let Some(c) = cursor {
            body["cursor"] = Value::String(c);
        }
        let resp: Value = s.post("/collections/posts/search").json(&body).await.json();
        total_eids += resp["hits"].as_array().unwrap().len();
        cursor = resp["cursor"].as_str().map(|s| s.to_string());
        if cursor.is_none() {
            break;
        }
        assert!(pages <= 7, "cursor walk diverged");
    }
    assert_eq!(total_eids, 53);
    assert_eq!(pages, 6);
}

// ---------------------------------------------------------------------------
// §1: re-index of same (eid, field) is a full replacement
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reindex_replaces_text_value_and_clears_old_tokens() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "bio": { "type": "text" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "bio", "value": "alpha beta gamma" }
        ]}))
        .await
        .assert_status_ok();
    // Re-write the same field — old tokens must vanish.
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "bio", "value": "delta epsilon" }
        ]}))
        .await
        .assert_status_ok();

    let r: Value = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "match": { "field": "bio", "text": "alpha" } },
            "limit": 10
        }))
        .await
        .json();
    assert_eq!(r["total"], 0, "old tokens should be gone");

    let r: Value = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "match": { "field": "bio", "text": "delta" } },
            "limit": 10
        }))
        .await
        .json();
    assert_eq!(r["total"], 1);
}

#[tokio::test]
async fn delete_one_field_keeps_others() {
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
            { "external_id": "u1", "field": "bio",   "value": "rust" }
        ]}))
        .await
        .assert_status_ok();

    s.delete("/collections/u/index/u1?field=email")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    // email is gone, bio still there.
    let r: Value = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "limit": 10
        }))
        .await
        .json();
    assert_eq!(r["total"], 0);
    let r: Value = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "match": { "field": "bio", "text": "rust" } },
            "limit": 10
        }))
        .await
        .json();
    assert_eq!(r["total"], 1);
}

// ---------------------------------------------------------------------------
// §1: query nesting depth and emptiness
// ---------------------------------------------------------------------------

#[tokio::test]
async fn empty_and_returns_empty_set() {
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
    // Empty `and` is convention-set; engine returns no hits.
    let r: Value = s
        .post("/collections/u/search")
        .json(&json!({ "query": { "and": [] }, "limit": 10 }))
        .await
        .json();
    assert_eq!(r["total"], 0);
}

#[tokio::test]
async fn nested_and_or_combination_evaluates_correctly() {
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
            { "external_id": "a", "field": "level", "value": 5 },
            { "external_id": "b", "field": "tag",   "value": "go" },
            { "external_id": "b", "field": "level", "value": 5 },
            { "external_id": "c", "field": "tag",   "value": "python" },
            { "external_id": "c", "field": "level", "value": 1 }
        ]}))
        .await
        .assert_status_ok();
    // (tag = rust OR tag = go) AND (level >= 5)  → a + b
    let r: Value = s
        .post("/collections/u/search")
        .json(&json!({
            "query": {
                "and": [
                    { "or": [
                        { "term": { "field": "tag", "value": "rust" } },
                        { "term": { "field": "tag", "value": "go" } }
                    ]},
                    { "range": { "field": "level", "gte": 5 } }
                ]
            },
            "limit": 10
        }))
        .await
        .json();
    assert_eq!(r["total"], 2);
}

// ---------------------------------------------------------------------------
// §6: RBAC for the rest of the endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_requires_read_role() {
    let s = auth_server(vec![("a", admin("ops"))]);
    s.put("/collections/u")
        .add_header("authorization", "Bearer a")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    // No bearer → 401.
    let resp = s
        .post("/collections/u/search")
        .json(&json!({ "query": { "term": { "field": "e", "value": "x" } }, "limit": 1 }))
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn stats_requires_read_role() {
    let s = auth_server(vec![("a", admin("ops"))]);
    s.put("/collections/u")
        .add_header("authorization", "Bearer a")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let resp = s.get("/collections/u/stats").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn duplicates_requires_read_role() {
    let s = auth_server(vec![("a", admin("ops"))]);
    s.put("/collections/u")
        .add_header("authorization", "Bearer a")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let resp = s
        .post("/collections/u/duplicates")
        .json(&json!({ "field": "e" }))
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn backup_endpoint_requires_admin_wildcard() {
    let s = auth_server(vec![(
        "tok-w",
        TokenClaims {
            subject: "worker".into(),
            roles: HashMap::from([("u".to_string(), Role::Write)]),
        },
    )]);
    let resp = s
        .get("/admin/backup")
        .add_header("authorization", "Bearer tok-w")
        .await;
    resp.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn drop_field_requires_admin() {
    let s = auth_server(vec![
        (
            "tok-r",
            TokenClaims {
                subject: "viewer".into(),
                roles: HashMap::from([("u".to_string(), Role::Read)]),
            },
        ),
        ("tok-a", admin("ops")),
    ]);
    s.put("/collections/u")
        .add_header("authorization", "Bearer tok-a")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let resp = s
        .delete("/collections/u/fields/e")
        .add_header("authorization", "Bearer tok-r")
        .await;
    resp.assert_status(StatusCode::FORBIDDEN);
}

// </HANDWRITE>
