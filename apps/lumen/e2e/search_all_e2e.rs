// CODEGEN-BEGIN
use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::spec::openapi_json;
use lumen::storage::Engine;

fn server() -> TestServer {
    let engine = Arc::new(Engine::new());
    TestServer::new(router(AppState::open(engine))).expect("test server")
}

#[tokio::test]
async fn search_all_returns_every_filtered_id_in_requested_sort_order() {
    let server = server();
    server
        .put("/collections/events")
        .json(&json!({
            "fields": {
                "group": { "type": "keyword" },
                "rank": { "type": "number" }
            }
        }))
        .await
        .assert_status_ok();

    let mut items = Vec::new();
    for rank in 0..73 {
        let external_id = format!("event-{rank:03}");
        items.push(json!({
            "external_id": external_id,
            "field": "group",
            "value": if rank % 5 == 0 { "drop" } else { "keep" }
        }));
        items.push(json!({
            "external_id": format!("event-{rank:03}"),
            "field": "rank",
            "value": rank
        }));
    }
    server
        .post("/collections/events/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    let response: Value = server
        .post("/collections/events/search:all")
        .json(&json!({
            "query": { "term": { "field": "group", "value": "keep" } },
            "sort": [{ "field": "rank", "order": "desc" }]
        }))
        .await
        .json();

    let ids: Vec<&str> = response["external_ids"]
        .as_array()
        .expect("typed external_ids array")
        .iter()
        .map(|id| id.as_str().expect("external id string"))
        .collect();
    let expected: Vec<String> = (0..73)
        .rev()
        .filter(|rank| rank % 5 != 0)
        .map(|rank| format!("event-{rank:03}"))
        .collect();

    assert!(ids.len() > 20, "fixture must exceed the normal search page");
    assert_eq!(response["total"], expected.len());
    assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
    assert!(response["took_us"].is_number());
    assert!(response["took_ms"].is_number());
}

#[test]
fn search_all_is_a_typed_generated_client_operation() {
    let spec: Value = serde_json::from_str(&openapi_json()).expect("valid OpenAPI JSON");
    let operation = &spec["paths"]["/collections/{collection_id}/search:all"]["post"];

    assert_eq!(operation["operationId"], "search_all");
    assert_eq!(
        operation["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SearchAllRequest"
    );
    assert_eq!(
        operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SearchAllResponse"
    );
}
// CODEGEN-END
