//! In-process HTTP surface tests: drive the axum router directly via
//! `tower::oneshot` (no socket, no server task).

use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use keep::{router, AppState, KvEngine};
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> (Router, AppState) {
    let state = AppState::new(Arc::new(KvEngine::with_shards(16)));
    (router(state.clone()), state)
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Vec<u8>) {
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, bytes.to_vec())
}

async fn send_json(app: &Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let (status, bytes) = send(app, req).await;
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, value)
}

#[tokio::test]
async fn set_get_delete_roundtrip() {
    let (app, _) = app();

    // PUT a structured JSON value.
    let (st, _) = send_json(&app, "PUT", "/v1/kv/foo", json!({"value": {"a": 1, "b": [2, 3]}})).await;
    assert_eq!(st, StatusCode::OK);

    // GET it back.
    let (st, body) = send_json(&app, "GET", "/v1/kv/foo", Value::Null).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!({"a": 1, "b": [2, 3]}));

    // HEAD => 200.
    let req = Request::builder().method("HEAD").uri("/v1/kv/foo").body(Body::empty()).unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);

    // DELETE => deleted true.
    let (st, body) = send_json(&app, "DELETE", "/v1/kv/foo", Value::Null).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["deleted"], json!(true));

    // GET now 404.
    let (st, body) = send_json(&app, "GET", "/v1/kv/foo", Value::Null).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], json!("key_not_found"));
}

#[tokio::test]
async fn claim_check_blob_roundtrip() {
    let (app, _) = app();
    let blob: Vec<u8> = (0u8..=255).cycle().take(4096).collect();

    // PUT raw bytes.
    let req = Request::builder()
        .method("PUT")
        .uri("/v1/kv/result:job-1")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from(blob.clone()))
        .unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);

    // GET returns the bytes verbatim as octet-stream.
    let req = Request::builder().method("GET").uri("/v1/kv/result:job-1").body(Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/octet-stream"
    );
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(bytes.as_ref(), blob.as_slice());
}

#[tokio::test]
async fn incr_and_cas() {
    let (app, _) = app();

    let (st, body) = send_json(&app, "POST", "/v1/kv/counter/incr", json!({"delta": 5})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!(5));

    let (st, body) = send_json(&app, "POST", "/v1/kv/counter/incr", json!({"delta": -2})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!(3));

    // CAS: expect 3 -> 99 succeeds.
    let (st, body) = send_json(&app, "POST", "/v1/kv/counter/cas", json!({"expected": 3, "new": 99})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["swapped"], json!(true));

    // CAS with wrong expected fails to swap.
    let (st, body) = send_json(&app, "POST", "/v1/kv/counter/cas", json!({"expected": 3, "new": 0})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["swapped"], json!(false));
}

#[tokio::test]
async fn batch_mset_mget() {
    let (app, _) = app();

    let (st, _) = send_json(
        &app,
        "POST",
        "/v1/kv:mset",
        json!({"entries": {"k1": "v1", "k2": 2}}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    let (st, body) = send_json(&app, "POST", "/v1/kv:mget", json!({"keys": ["k1", "k2", "missing"]})).await;
    assert_eq!(st, StatusCode::OK);
    let values = body["values"].as_array().unwrap();
    assert_eq!(values[0], json!("v1"));
    assert_eq!(values[1], json!(2));
    assert_eq!(values[2], Value::Null);
}

#[tokio::test]
async fn scan_by_prefix() {
    let (app, _) = app();
    for k in ["user:1", "user:2", "post:1"] {
        let (st, _) = send_json(&app, "PUT", &format!("/v1/kv/{k}"), json!({"value": 1})).await;
        assert_eq!(st, StatusCode::OK);
    }
    let (st, body) = send_json(&app, "GET", "/v1/kv?prefix=user:&limit=10", Value::Null).await;
    assert_eq!(st, StatusCode::OK);
    let mut keys: Vec<String> = body["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    keys.sort();
    assert_eq!(keys, vec!["user:1".to_string(), "user:2".to_string()]);
}

#[tokio::test]
async fn locks_lifecycle() {
    let (app, _) = app();

    let (st, body) = send_json(&app, "POST", "/v1/locks/job", json!({"owner": "a", "ttl_ms": 60000})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["acquired"], json!(true));

    // A different owner cannot acquire.
    let (st, body) = send_json(&app, "POST", "/v1/locks/job", json!({"owner": "b", "ttl_ms": 60000})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["acquired"], json!(false));

    // Owner releases.
    let (st, body) = send_json(&app, "DELETE", "/v1/locks/job", json!({"owner": "a"})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["released"], json!(true));
}

#[tokio::test]
async fn list_push_pop() {
    let (app, _) = app();

    let (st, body) = send_json(&app, "POST", "/v1/lists/q/rpush", json!({"values": ["a", "b", "c"]})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["length"], json!(3));

    let (st, body) = send_json(&app, "POST", "/v1/lists/q/lpop", Value::Null).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!("a"));
}

#[tokio::test]
async fn probes_and_drain() {
    let (app, state) = app();

    let req = Request::builder().uri("/healthz").body(Body::empty()).unwrap();
    let (st, body) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body, b"ok");

    let req = Request::builder().uri("/readyz").body(Body::empty()).unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);

    // Once draining, readyz flips to 503 so k8s stops routing.
    state.start_drain();
    let req = Request::builder().uri("/readyz").body(Body::empty()).unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn openapi_document_is_served() {
    let (app, _) = app();
    let req = Request::builder().uri("/openapi.json").body(Body::empty()).unwrap();
    let (st, bytes) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);
    let doc: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(doc["info"]["title"], json!("keep"));
    assert!(doc["paths"]["/v1/kv/{key}"].is_object());
    assert!(doc["paths"]["/healthz"].is_object());
}
