//! In-process HTTP surface tests: drive the axum router directly via
//! `tower::oneshot` (no socket, no server task).

use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use keep::{router, AppState, KvEngine};
#[cfg(feature = "raft")]
use keep::{ClusterConfig, KvKey, KvValue};
use serde_json::{json, Value};
use tower::ServiceExt;

#[cfg(feature = "raft")]
struct RaftHttpNode {
    url: String,
    engine: Arc<KvEngine>,
    hosts: Arc<keep::raft::ShardHosts>,
    serve: tokio::task::JoinHandle<()>,
}

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

#[cfg(feature = "raft")]
fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("keep-http-raft-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(feature = "raft")]
async fn bind_local() -> (tokio::net::TcpListener, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    (listener, format!("http://127.0.0.1:{port}"))
}

#[cfg(feature = "raft")]
async fn raft_http_cluster(tag: &str) -> (std::path::PathBuf, Vec<RaftHttpNode>) {
    let dir = tmp_dir(tag);
    let mut listeners = Vec::new();
    let mut urls = Vec::new();
    for _ in 0..3 {
        let (listener, url) = bind_local().await;
        listeners.push(listener);
        urls.push(url);
    }

    let mut nodes = Vec::new();
    for (node_id, listener) in listeners.into_iter().enumerate() {
        let engine = Arc::new(KvEngine::with_shards(4));
        let cluster = ClusterConfig::from_shared_topology(node_id, 3, 1, urls.clone());
        let hosts = Arc::new(
            keep::raft::ShardHosts::new(
                cluster.clone(),
                engine.clone(),
                &dir.join(format!("node-{node_id}")),
                3,
            )
            .await
            .unwrap(),
        );
        let state = AppState::new(engine.clone())
            .with_cluster(cluster)
            .with_raft_hosts(hosts.clone());
        let app = router(state).merge(hosts.router());
        let serve = tokio::spawn(async move {
            service_http::serve(listener, app, std::future::pending::<()>()).await;
        });
        nodes.push(RaftHttpNode {
            url: urls[node_id].clone(),
            engine,
            hosts,
            serve,
        });
    }
    (dir, nodes)
}

#[cfg(feature = "raft")]
async fn await_raft_leader(nodes: &[RaftHttpNode], key: &str) -> usize {
    for _ in 0..400 {
        for (idx, node) in nodes.iter().enumerate() {
            if node.hosts.host_for(key).unwrap().is_leader().await {
                return idx;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("no raft leader elected for key {key}");
}

#[cfg(feature = "raft")]
async fn stop_raft_http_cluster(dir: std::path::PathBuf, nodes: Vec<RaftHttpNode>) {
    for node in nodes {
        node.serve.abort();
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn set_get_delete_roundtrip() {
    let (app, _) = app();

    // PUT a structured JSON value.
    let (st, _) = send_json(
        &app,
        "PUT",
        "/kv/foo",
        json!({"value": {"a": 1, "b": [2, 3]}}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    // GET it back.
    let (st, body) = send_json(&app, "GET", "/kv/foo", Value::Null).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!({"a": 1, "b": [2, 3]}));

    // HEAD => 200.
    let req = Request::builder()
        .method("HEAD")
        .uri("/kv/foo")
        .body(Body::empty())
        .unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);

    // DELETE => deleted true.
    let (st, body) = send_json(&app, "DELETE", "/kv/foo", Value::Null).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["deleted"], json!(true));

    // GET now 404.
    let (st, body) = send_json(&app, "GET", "/kv/foo", Value::Null).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], json!("key_not_found"));
}

#[cfg(feature = "raft")]
#[tokio::test]
async fn replica_mode_http_write_proposes_and_replicates() {
    let key = "raft-http-write";
    let (dir, nodes) = raft_http_cluster("write").await;
    let leader = await_raft_leader(&nodes, key).await;

    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{}/kv/{key}", nodes[leader].url))
        .json(&json!({"value": "via-http-raft"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let kv_key = KvKey::new(key).unwrap();
    for _ in 0..400 {
        if nodes.iter().all(|node| {
            node.engine.get(&kv_key) == Some(KvValue::String("via-http-raft".to_string()))
        }) {
            stop_raft_http_cluster(dir, nodes).await;
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("HTTP raft write did not reach every replica engine");
}

#[tokio::test]
async fn claim_check_blob_roundtrip() {
    let (app, _) = app();
    let blob: Vec<u8> = (0u8..=255).cycle().take(4096).collect();

    // PUT raw bytes.
    let req = Request::builder()
        .method("PUT")
        .uri("/kv/result:job-1")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from(blob.clone()))
        .unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);

    // GET returns the bytes verbatim as octet-stream.
    let req = Request::builder()
        .method("GET")
        .uri("/kv/result:job-1")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/octet-stream"
    );
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(bytes.as_ref(), blob.as_slice());
}

#[tokio::test]
async fn incr_and_cas() {
    let (app, _) = app();

    let (st, body) = send_json(&app, "POST", "/kv/counter/incr", json!({"delta": 5})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!(5));

    let (st, body) = send_json(&app, "POST", "/kv/counter/incr", json!({"delta": -2})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!(3));

    // CAS: expect 3 -> 99 succeeds.
    let (st, body) = send_json(
        &app,
        "POST",
        "/kv/counter/cas",
        json!({"expected": 3, "new": 99}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["swapped"], json!(true));

    // CAS with wrong expected fails to swap.
    let (st, body) = send_json(
        &app,
        "POST",
        "/kv/counter/cas",
        json!({"expected": 3, "new": 0}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["swapped"], json!(false));
}

#[tokio::test]
async fn batch_mset_mget() {
    let (app, _) = app();

    let (st, _) = send_json(
        &app,
        "POST",
        "/kv:mset",
        json!({"entries": {"k1": "v1", "k2": 2}}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    let (st, body) = send_json(
        &app,
        "POST",
        "/kv:mget",
        json!({"keys": ["k1", "k2", "missing"]}),
    )
    .await;
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
        let (st, _) = send_json(&app, "PUT", &format!("/kv/{k}"), json!({"value": 1})).await;
        assert_eq!(st, StatusCode::OK);
    }
    let (st, body) = send_json(&app, "GET", "/kv?prefix=user:&limit=10", Value::Null).await;
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

    let (st, body) = send_json(
        &app,
        "POST",
        "/locks/job",
        json!({"owner": "a", "ttl_ms": 60000}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["acquired"], json!(true));

    // A different owner cannot acquire.
    let (st, body) = send_json(
        &app,
        "POST",
        "/locks/job",
        json!({"owner": "b", "ttl_ms": 60000}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["acquired"], json!(false));

    // Owner releases.
    let (st, body) = send_json(&app, "DELETE", "/locks/job", json!({"owner": "a"})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["released"], json!(true));
}

#[tokio::test]
async fn list_push_pop() {
    let (app, _) = app();

    let (st, body) = send_json(
        &app,
        "POST",
        "/lists/q/rpush",
        json!({"values": ["a", "b", "c"]}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["length"], json!(3));

    let (st, body) = send_json(&app, "POST", "/lists/q/lpop", Value::Null).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["value"], json!("a"));
}

#[tokio::test]
async fn probes_and_drain() {
    let (app, state) = app();

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let (st, body) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body, b"ok");

    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);

    // Once draining, readyz flips to 503 so k8s stops routing. The shared
    // service-http probe shell (#751) preserves keep's `draining` body.
    state.start_drain();
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let (st, body) = send(&app, req).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, b"draining");
}

#[tokio::test]
async fn metrics_records_per_route_requests() {
    let (app, _) = app();
    // Exercise a couple of data-plane routes.
    send_json(&app, "PUT", "/kv/m", json!({"value": 1})).await;
    send_json(&app, "GET", "/kv/m", Value::Null).await;

    let req = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    let (st, bytes) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);
    let text = String::from_utf8_lossy(&bytes);
    // Request counter labelled by the matched route pattern (not the raw key).
    assert!(
        text.contains("keep_http_requests_total"),
        "missing request counter"
    );
    assert!(
        text.contains("route=\"/kv/{key}\""),
        "missing matched-route label:\n{text}"
    );
    // Latency histogram + the existing engine gauges.
    assert!(
        text.contains("keep_http_request_duration_seconds_bucket"),
        "missing histogram"
    );
    assert!(text.contains("keep_keys_total"), "engine gauges dropped");
}

#[tokio::test]
async fn cluster_endpoint_reports_topology() {
    let (app, _) = app();
    let req = Request::builder()
        .uri("/cluster")
        .body(Body::empty())
        .unwrap();
    let (st, bytes) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["node_count"], json!(1));
    assert_eq!(v["mode"], json!("single"));
    assert_eq!(v["owned_shards"], json!(1));
}

#[tokio::test]
async fn openapi_document_is_served() {
    let (app, _) = app();
    let req = Request::builder()
        .uri("/openapi.json")
        .body(Body::empty())
        .unwrap();
    let (st, bytes) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);
    let doc: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(doc["info"]["title"], json!("keep"));
    assert!(doc["paths"]["/kv/{key}"].is_object());
    assert!(doc["paths"]["/healthz"].is_object());
}

// ---------------------------------------------------------------------------
// Per-namespace claim-check key isolation (#464): X-Keep-Namespace is applied
// as a storage-key prefix `{ns}::{kind}:{id}` AFTER the token scope check.
// ---------------------------------------------------------------------------

fn claim_put_req(path: &str, ns: Option<&str>, body: &'static str) -> Request<Body> {
    let mut b = Request::builder()
        .method("PUT")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/octet-stream");
    if let Some(ns) = ns {
        b = b.header("x-keep-namespace", ns);
    }
    b.body(Body::from(body)).unwrap()
}

fn claim_get_req(path: &str, ns: Option<&str>) -> Request<Body> {
    let mut b = Request::builder().method("GET").uri(path);
    if let Some(ns) = ns {
        b = b.header("x-keep-namespace", ns);
    }
    b.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn keep_ns_isolates_same_bare_key() {
    let (app, _) = app();

    // Two namespaces write the SAME bare result id.
    let (st, _) = send(
        &app,
        claim_put_req("/results/job-1", Some("tenant-a"), "AAA"),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let (st, _) = send(
        &app,
        claim_put_req("/results/job-1", Some("tenant-b"), "BBB"),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    // Each namespace reads back only its own value — no cross-namespace collision.
    let (st, body) = send(&app, claim_get_req("/results/job-1", Some("tenant-a"))).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body, b"AAA".to_vec());
    let (st, body) = send(&app, claim_get_req("/results/job-1", Some("tenant-b"))).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body, b"BBB".to_vec());

    // A namespace that never wrote this id sees nothing.
    let (st, _) = send(&app, claim_get_req("/results/job-1", Some("tenant-c"))).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn keep_ns_absent_is_backcompat() {
    let (app, _) = app();

    // No X-Keep-Namespace ⇒ bare 'result:job-2' storage key.
    let (st, _) = send(&app, claim_put_req("/results/job-2", None, "PLAIN")).await;
    assert_eq!(st, StatusCode::OK);
    let (st, body) = send(&app, claim_get_req("/results/job-2", None)).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body, b"PLAIN".to_vec());

    // The bare key is observable on the generic KV path at 'result:job-2'.
    let (st, _) = send(&app, claim_get_req("/kv/result:job-2", None)).await;
    assert_eq!(st, StatusCode::OK);

    // A namespaced read does NOT see the bare-written value.
    let (st, _) = send(&app, claim_get_req("/results/job-2", Some("tenant-a"))).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn keep_ns_token_checks_bare_key() {
    let secret = b"test-secret".to_vec();
    let state =
        AppState::new(Arc::new(KvEngine::with_shards(16))).with_token_secret(secret.clone());
    let app = router(state);

    // Token scoped to the BARE result key 'job-3'.
    let scope = claimtoken::Scope {
        r: "job-3".into(),
        w: "job-3".into(),
        exp: u64::MAX,
    };
    let token = claimtoken::sign(&secret, &scope);

    // PUT with a namespace header still passes — the token is verified against
    // the bare 'job-3' from the URL, before the namespace prefix is applied.
    let req = Request::builder()
        .method("PUT")
        .uri("/results/job-3")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header("x-keep-namespace", "tenant-a")
        .body(Body::from("R3"))
        .unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::OK);

    // Without the token ⇒ rejected, namespace notwithstanding.
    let req = Request::builder()
        .method("PUT")
        .uri("/results/job-3")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header("x-keep-namespace", "tenant-a")
        .body(Body::from("R3"))
        .unwrap();
    let (st, _) = send(&app, req).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
}

/// #746: after the service-auth adoption, the claim-check contract is unchanged
/// — enforcement runs through keep's `KeepVerifier` per-handler but still
/// returns 403 (not 401) for invalid and out-of-scope tokens, and the accepted
/// read scope on GET /inputs/{id} returns 200.
#[tokio::test]
async fn keep_token_invalid_and_out_of_scope_are_403() {
    let secret = b"test-secret".to_vec();
    let state =
        AppState::new(Arc::new(KvEngine::with_shards(16))).with_token_secret(secret.clone());
    let app = router(state);

    // Seed an input the worker is allowed to read with an in-scope token.
    let scope = claimtoken::Scope {
        r: "job-7".into(),
        w: "job-7".into(),
        exp: u64::MAX,
    };
    let token = claimtoken::sign(&secret, &scope);
    let put = Request::builder()
        .method("PUT")
        .uri("/inputs/job-7")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from("IN7"))
        .unwrap();
    // PUT input is deliberately NOT scope-checked, so no token is needed.
    let (st, _) = send(&app, put).await;
    assert_eq!(st, StatusCode::OK);

    // Accepted read scope ⇒ 200.
    let get_ok = Request::builder()
        .method("GET")
        .uri("/inputs/job-7")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let (st, body) = send(&app, get_ok).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body, b"IN7".to_vec());

    // A syntactically-broken bearer value ⇒ 403 (not 401).
    let get_bad = Request::builder()
        .method("GET")
        .uri("/inputs/job-7")
        .header(header::AUTHORIZATION, "Bearer not-a-real-token")
        .body(Body::empty())
        .unwrap();
    let (st, _) = send(&app, get_bad).await;
    assert_eq!(st, StatusCode::FORBIDDEN);

    // A valid token for a DIFFERENT id (out of scope) ⇒ 403.
    let other = claimtoken::Scope {
        r: "job-other".into(),
        w: "job-other".into(),
        exp: u64::MAX,
    };
    let other_token = claimtoken::sign(&secret, &other);
    let get_oos = Request::builder()
        .method("GET")
        .uri("/inputs/job-7")
        .header(header::AUTHORIZATION, format!("Bearer {other_token}"))
        .body(Body::empty())
        .unwrap();
    let (st, _) = send(&app, get_oos).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
}

/// #746/#446: with no secret configured the verifier is absent, so claim-check
/// stays open (no-op) — a tokenless worker read still succeeds.
#[tokio::test]
async fn keep_token_disabled_is_open() {
    let (app, _) = app();

    let put = Request::builder()
        .method("PUT")
        .uri("/inputs/job-open")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from("OPEN"))
        .unwrap();
    let (st, _) = send(&app, put).await;
    assert_eq!(st, StatusCode::OK);

    // No Authorization header, enforcement off ⇒ 200.
    let get = Request::builder()
        .method("GET")
        .uri("/inputs/job-open")
        .body(Body::empty())
        .unwrap();
    let (st, body) = send(&app, get).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body, b"OPEN".to_vec());
}
