//! `POST /collections/{id}/reindex/stream` — NDJSON in, NDJSON out.

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

fn ndjson_body(items: &[Value]) -> String {
    let mut s = String::new();
    for item in items {
        s.push_str(&item.to_string());
        s.push('\n');
    }
    s
}

#[tokio::test]
async fn stream_indexes_items_and_emits_progress_then_done() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    // 2 500 docs → enough to span > 2 internal batches at BATCH_SIZE=1000.
    let mut items = vec![];
    for i in 0..2_500 {
        items.push(json!({
            "external_id": format!("u{i}"),
            "field": "email",
            "value": format!("u{}@x.com", i % 50),
        }));
    }
    let body = ndjson_body(&items);

    let resp = s
        .post("/collections/u/reindex/stream")
        .content_type("application/x-ndjson")
        .text(body)
        .await;
    resp.assert_status_ok();

    let text = resp.text();
    let events: Vec<Value> = text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).expect("ndjson event"))
        .collect();

    // At least 3 progress + 1 done.
    assert!(events.len() >= 4, "expected ≥ 4 events, got: {events:?}");
    let last = events.last().unwrap();
    assert_eq!(last["event"], "done");
    assert_eq!(last["indexed_total"], 2_500);

    // Sanity-check the collection actually has the data.
    let r = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "u0@x.com" } },
            "limit": 100
        }))
        .await;
    let body: Value = r.json();
    // 50 documents share each email (2500 / 50).
    assert_eq!(body["total"], 50);
}

#[tokio::test]
async fn stream_skips_blank_lines_and_reports_parse_errors() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    let body = format!(
        "{}\n\n{}\n{}\n",
        json!({"external_id":"u1","field":"email","value":"a@x.com"}),
        "not-json",
        json!({"external_id":"u2","field":"email","value":"b@x.com"}),
    );

    let resp = s
        .post("/collections/u/reindex/stream")
        .content_type("application/x-ndjson")
        .text(body)
        .await;
    resp.assert_status_ok();
    let events: Vec<Value> = resp
        .text()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    let has_error = events.iter().any(|e| e["event"] == "error");
    assert!(has_error, "expected an error event for the malformed line");

    let done = events.last().unwrap();
    assert_eq!(done["event"], "done");
    assert_eq!(done["indexed_total"], 2);
}

#[tokio::test]
async fn stream_requires_write_role_under_auth() {
    use lumen::auth::{AuthConfig, Role, TokenClaims};
    use std::collections::HashMap;

    let engine = Arc::new(Engine::new());
    let cfg = AuthConfig {
        required: true,
        tokens: HashMap::from([
            (
                "tok-r".to_string(),
                TokenClaims {
                    subject: "viewer".into(),
                    roles: HashMap::from([("u".to_string(), Role::Read)]),
                },
            ),
            (
                "tok-a".to_string(),
                TokenClaims {
                    subject: "ops".into(),
                    roles: HashMap::from([("*".to_string(), Role::Admin)]),
                },
            ),
        ]),
    };
    let app = router(AppState::new(engine, Arc::new(cfg)));
    let s = TestServer::new(app).unwrap();

    s.put("/collections/u")
        .add_header("authorization", "Bearer tok-a")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    // Read-only token is forbidden.
    let resp = s
        .post("/collections/u/reindex/stream")
        .add_header("authorization", "Bearer tok-r")
        .content_type("application/x-ndjson")
        .text(format!(
            "{}\n",
            json!({"external_id":"u1","field":"email","value":"a@x.com"})
        ))
        .await;
    resp.assert_status_forbidden();
}
