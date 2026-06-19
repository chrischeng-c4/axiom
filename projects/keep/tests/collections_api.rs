//! In-process tests for the collection + expiry routes (hash / set / zset /
//! list-range / ttl), driving the router via tower::oneshot.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use keep::{router, AppState, KvEngine};
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> Router {
    router(AppState::new(Arc::new(KvEngine::with_shards(16))))
}

async fn call(app: &Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(if body.is_null() {
            Body::empty()
        } else {
            Body::from(serde_json::to_vec(&body).unwrap())
        })
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, v)
}

#[tokio::test]
async fn hash_roundtrip() {
    let app = app();
    let (st, body) = call(&app, "POST", "/v1/hashes/h", json!({"fields": {"a": 1, "b": "x"}})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["count"], json!(2));

    let (_, body) = call(&app, "GET", "/v1/hashes/h", Value::Null).await;
    assert_eq!(body["fields"]["a"], json!(1));
    assert_eq!(body["fields"]["b"], json!("x"));

    let (_, body) = call(&app, "GET", "/v1/hashes/h/fields/a", Value::Null).await;
    assert_eq!(body["value"], json!(1));

    let (_, body) = call(&app, "POST", "/v1/hashes/h/mget", json!({"fields": ["a", "missing"]})).await;
    assert_eq!(body["values"][0], json!(1));
    assert_eq!(body["values"][1], Value::Null);

    let (_, body) = call(&app, "POST", "/v1/hashes/c/incr", json!({"field": "n", "delta": 5})).await;
    assert_eq!(body["value"], json!(5));

    let (_, body) = call(&app, "GET", "/v1/hashes/h/length", Value::Null).await;
    assert_eq!(body["count"], json!(2));

    let req = Request::builder().method("HEAD").uri("/v1/hashes/h/fields/a").body(Body::empty()).unwrap();
    assert_eq!(app.clone().oneshot(req).await.unwrap().status(), StatusCode::OK);

    let (_, body) = call(&app, "DELETE", "/v1/hashes/h", json!({"fields": ["a"]})).await;
    assert_eq!(body["count"], json!(1));
}

#[tokio::test]
async fn set_roundtrip() {
    let app = app();
    let (st, body) = call(&app, "POST", "/v1/sets/s", json!({"members": ["a", "b", "a"]})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["count"], json!(2)); // dedup

    let (_, body) = call(&app, "GET", "/v1/sets/s/length", Value::Null).await;
    assert_eq!(body["count"], json!(2));

    let req = Request::builder().method("HEAD").uri("/v1/sets/s/members/a").body(Body::empty()).unwrap();
    assert_eq!(app.clone().oneshot(req).await.unwrap().status(), StatusCode::OK);
    let req = Request::builder().method("HEAD").uri("/v1/sets/s/members/z").body(Body::empty()).unwrap();
    assert_eq!(app.clone().oneshot(req).await.unwrap().status(), StatusCode::NOT_FOUND);

    let (_, body) = call(&app, "GET", "/v1/sets/s", Value::Null).await;
    let mut members: Vec<String> = body["members"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    members.sort();
    assert_eq!(members, vec!["a".to_string(), "b".to_string()]);

    let (_, body) = call(&app, "DELETE", "/v1/sets/s", json!({"members": ["a"]})).await;
    assert_eq!(body["count"], json!(1));
}

#[tokio::test]
async fn zset_roundtrip() {
    let app = app();
    let (st, body) = call(&app, "POST", "/v1/zsets/z",
        json!({"members": [{"member": "a", "score": 1.0}, {"member": "b", "score": 3.0}, {"member": "c", "score": 2.0}]})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["count"], json!(3));

    // ascending by score: a(1), c(2), b(3)
    let (_, body) = call(&app, "GET", "/v1/zsets/z?start=0&stop=-1", Value::Null).await;
    let order: Vec<String> = body["entries"].as_array().unwrap().iter().map(|e| e["member"].as_str().unwrap().to_string()).collect();
    assert_eq!(order, vec!["a".to_string(), "c".to_string(), "b".to_string()]);

    let (_, body) = call(&app, "GET", "/v1/zsets/z/members/b/score", Value::Null).await;
    assert_eq!(body["score"], json!(3.0));
    let (_, body) = call(&app, "GET", "/v1/zsets/z/members/a/rank", Value::Null).await;
    assert_eq!(body["rank"], json!(0));

    let (_, body) = call(&app, "POST", "/v1/zsets/z/incr", json!({"member": "a", "delta": 10.0})).await;
    assert_eq!(body["value"], json!(11.0));

    let (_, body) = call(&app, "GET", "/v1/zsets/z/length", Value::Null).await;
    assert_eq!(body["count"], json!(3));
}

#[tokio::test]
async fn list_range_and_expiry() {
    let app = app();
    call(&app, "POST", "/v1/lists/l/rpush", json!({"values": [1, 2, 3]})).await;
    let (_, body) = call(&app, "GET", "/v1/lists/l?start=0&stop=-1", Value::Null).await;
    assert_eq!(body["values"], json!([1, 2, 3]));
    let (_, body) = call(&app, "GET", "/v1/lists/l/length", Value::Null).await;
    assert_eq!(body["length"], json!(3));

    // expiry on a scalar key
    call(&app, "PUT", "/v1/kv/e", json!({"value": "x"})).await;
    let (_, body) = call(&app, "GET", "/v1/kv/e/ttl", Value::Null).await;
    assert_eq!(body["ttl_secs"], json!(-1)); // exists, no expiry
    let (_, body) = call(&app, "POST", "/v1/kv/e/expire", json!({"seconds": 100})).await;
    assert_eq!(body["applied"], json!(true));
    let (_, body) = call(&app, "GET", "/v1/kv/e/ttl", Value::Null).await;
    assert!(body["ttl_secs"].as_i64().unwrap() > 0);
    let (_, body) = call(&app, "POST", "/v1/kv/e/persist", Value::Null).await;
    assert_eq!(body["applied"], json!(true));
}

#[tokio::test]
async fn blpop_returns_present_element() {
    let app = app();
    call(&app, "POST", "/v1/lists/q/rpush", json!({"values": ["x", "y"]})).await;
    let (st, body) = call(&app, "POST", "/v1/lists/q/blpop", json!({"timeout_ms": 1000})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!("x"));
}

#[tokio::test]
async fn blpop_times_out_on_empty() {
    let app = app();
    let (st, body) = call(&app, "POST", "/v1/lists/empty/blpop", json!({"timeout_ms": 50})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], Value::Null);
}

#[tokio::test]
async fn blpop_wakes_on_concurrent_push() {
    let app = app();
    // A popper blocks on an empty list...
    let popper = {
        let app = app.clone();
        tokio::spawn(async move {
            call(&app, "POST", "/v1/lists/wq/blpop", json!({"timeout_ms": 5000})).await
        })
    };
    // ...then a push from elsewhere must wake it.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    call(&app, "POST", "/v1/lists/wq/rpush", json!({"values": ["job1"]})).await;

    let (st, body) = popper.await.unwrap();
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!("job1"));
}

#[tokio::test]
async fn getex_reads_and_adjusts_ttl() {
    let app = app();
    call(&app, "PUT", "/v1/kv/g", json!({"value": "hi"})).await;

    // GETEX with a TTL returns the value and arms the expiry.
    let (st, body) = call(&app, "POST", "/v1/kv/g/getex", json!({"ttl_ms": 100000})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!("hi"));
    let (_, body) = call(&app, "GET", "/v1/kv/g/ttl", Value::Null).await;
    assert!(body["ttl_secs"].as_i64().unwrap() > 0, "ttl should be set");

    // GETEX persist clears it.
    call(&app, "POST", "/v1/kv/g/getex", json!({"persist": true})).await;
    let (_, body) = call(&app, "GET", "/v1/kv/g/ttl", Value::Null).await;
    assert_eq!(body["ttl_secs"], json!(-1), "ttl should be cleared");
}

#[tokio::test]
async fn openapi_lists_the_new_paths() {
    let app = app();
    let (st, doc) = call(&app, "GET", "/openapi.json", Value::Null).await;
    assert_eq!(st, StatusCode::OK);
    for p in [
        "/v1/hashes/{key}",
        "/v1/sets/{key}",
        "/v1/zsets/{key}",
        "/v1/lists/{key}",
        "/v1/kv/{key}/ttl",
        "/v1/kv/{key}/getex",
    ] {
        assert!(doc["paths"][p].is_object(), "openapi missing path {p}");
    }
}
