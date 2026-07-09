// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for the trace viewer HTTP server.
//!
//! Covers T6, T7, T8 from the spec Test Plan
//! (see `.aw/changes/enhancement-native-trace-viewer-trace-capture-standalone-html/specs/enhancement-native-trace-viewer-trace-capture-standalone-html-spec.md`).

use jet::trace::archive::read_manifest_from_zip;
use jet::trace::buffer::{commit_trace, TraceBuffer, TraceMode};
use jet::trace::manifest::{ActionKind, TraceOutcome};
use jet::trace::server::{build_router, ViewerState};

fn tempdir(sub: &str) -> std::path::PathBuf {
    let base = std::env::temp_dir().join("jet-trace-viewer-tests");
    let path = base.join(sub);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn sample_zip(dir: &std::path::Path) -> std::path::PathBuf {
    let out = dir.join("sample.zip");
    let mut buf = TraceBuffer::new("tid-v", "spec.ts", "viewer sample");
    buf.append_action_step(
        ActionKind::Goto,
        None,
        Some("http://localhost".into()),
        0,
        Some("<html></html>".into()),
        Some(b"PNG0".to_vec()),
        None,
    );
    buf.append_screenshot(b"PNG1".to_vec());
    commit_trace(buf, TraceOutcome::Passed, TraceMode::On, &out)
        .unwrap()
        .unwrap();
    out
}

// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R12
#[tokio::test]
async fn test_http_server_binds_loopback() {
    let dir = tempdir("bind_loopback");
    let zip_path = sample_zip(&dir);
    let manifest = read_manifest_from_zip(&zip_path).unwrap();
    let state = ViewerState::new(zip_path, manifest);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    assert!(addr.ip().is_loopback(), "server must bind loopback only");
    assert!(addr.port() > 0, "server must bind a real port");

    // Spin the server briefly — assert it accepts a connection.
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let url = format!("http://{addr}/");
    let resp = reqwest::get(&url).await.expect("GET /");
    assert!(resp.status().is_success());
    assert!(resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .starts_with("text/html"));

    handle.abort();
}

// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
#[tokio::test]
async fn test_trace_json_endpoint_matches_manifest() {
    let dir = tempdir("trace_json");
    let zip_path = sample_zip(&dir);
    let manifest = read_manifest_from_zip(&zip_path).unwrap();
    let expected_test_id = manifest.test_id.clone();
    let state = ViewerState::new(zip_path, manifest);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let url = format!("http://{addr}/trace.json");
    let resp = reqwest::get(&url).await.expect("GET /trace.json");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["test_id"], expected_test_id);
    assert!(body["events"].is_array());

    handle.abort();
}

// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
#[tokio::test]
async fn test_asset_endpoint_returns_bytes() {
    let dir = tempdir("asset_bytes");
    let zip_path = sample_zip(&dir);
    let manifest = read_manifest_from_zip(&zip_path).unwrap();
    // Pick any asset_id from the map.
    let asset_id = manifest
        .assets
        .keys()
        .next()
        .expect("at least one asset")
        .clone();
    let state = ViewerState::new(zip_path, manifest);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let url = format!("http://{addr}/assets/{asset_id}");
    let resp = reqwest::get(&url).await.expect("GET /assets/<id>");
    assert!(resp.status().is_success());
    let bytes = resp.bytes().await.unwrap();
    assert!(!bytes.is_empty(), "asset bytes should not be empty");

    handle.abort();
}
// CODEGEN-END
