// CODEGEN-BEGIN
use std::sync::Arc;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::storage::Engine;

fn server() -> TestServer {
    let engine = Arc::new(Engine::new());
    TestServer::new(router(AppState::open(engine))).expect("test server")
}

async fn create_places(server: &TestServer) {
    server
        .put("/collections/places")
        .json(&json!({
            "fields": {
                "path": { "type": "keyword" },
                "kind": { "type": "keyword" },
                "rank": { "type": "number" },
                "description": { "type": "text" }
            }
        }))
        .await
        .assert_status_ok();
}

async fn ids(server: &TestServer, request: Value) -> Vec<String> {
    let response = server
        .post("/collections/places/search")
        .json(&request)
        .await;
    response.assert_status_ok();
    response.json::<Value>()["hits"]
        .as_array()
        .expect("hits")
        .iter()
        .map(|hit| hit["external_id"].as_str().expect("external id").to_owned())
        .collect()
}

#[tokio::test]
async fn keyword_prefix_is_utf8_case_sensitive_and_composes_with_bool_and_sort() {
    let server = server();
    create_places(&server).await;
    server
        .post("/collections/places/index")
        .json(&json!({ "items": [
            { "external_id": "daan", "field": "path", "value": "台北市/大安區" },
            { "external_id": "daan", "field": "kind", "value": "district" },
            { "external_id": "daan", "field": "rank", "value": 2 },
            { "external_id": "xinyi", "field": "path", "value": "台北市/信義區" },
            { "external_id": "xinyi", "field": "kind", "value": "district" },
            { "external_id": "xinyi", "field": "rank", "value": 1 },
            { "external_id": "new-taipei", "field": "path", "value": "新北市/板橋區" },
            { "external_id": "new-taipei", "field": "kind", "value": "district" },
            { "external_id": "new-taipei", "field": "rank", "value": 3 },
            { "external_id": "lowercase", "field": "path", "value": "taipei/daan" },
            { "external_id": "lowercase", "field": "kind", "value": "district" },
            { "external_id": "lowercase", "field": "rank", "value": 4 }
        ]}))
        .await
        .assert_status_ok();

    let got = ids(
        &server,
        json!({
            "query": { "and": [
                { "prefix": { "field": "path", "value": "台北市/" } },
                { "term": { "field": "kind", "value": "district" } }
            ]},
            "sort": [{ "field": "rank", "order": "asc" }],
            "limit": 10
        }),
    )
    .await;
    assert_eq!(got, ["xinyi", "daan"]);

    let case_sensitive = ids(
        &server,
        json!({
            "query": { "prefix": { "field": "path", "value": "Taipei" } },
            "limit": 10
        }),
    )
    .await;
    assert!(case_sensitive.is_empty());
}

#[tokio::test]
async fn keyword_prefix_observes_tail_updates_and_deletes() {
    let server = server();
    create_places(&server).await;
    server
        .post("/collections/places/index")
        .json(&json!({ "items": [
            { "external_id": "old", "field": "path", "value": "台北市/舊址" },
            { "external_id": "tail", "field": "path", "value": "台北市/新址" }
        ]}))
        .await
        .assert_status_ok();
    server
        .delete("/collections/places/index/old")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let got = ids(
        &server,
        json!({
            "query": { "prefix": { "field": "path", "value": "台北市/" } },
            "limit": 10
        }),
    )
    .await;
    assert_eq!(got, ["tail"]);
}

#[tokio::test]
async fn prefix_rejects_empty_values_and_non_keyword_fields() {
    let server = server();
    create_places(&server).await;

    for request in [
        json!({
            "query": { "prefix": { "field": "path", "value": "" } },
            "limit": 10
        }),
        json!({
            "query": { "prefix": { "field": "description", "value": "台北" } },
            "limit": 10
        }),
    ] {
        server
            .post("/collections/places/search")
            .json(&request)
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}
// CODEGEN-END
