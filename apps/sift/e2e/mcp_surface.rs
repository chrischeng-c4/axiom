//! Black-box contracts for Sift's read-only MCP surface.

use std::{collections::HashMap, process::Stdio, time::Duration};

use axum::{
    body::Body,
    extract::{Path, Query},
    http::Request,
    routing::get,
    Json, Router,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tower::ServiceExt;

#[test]
fn mcp_exposes_only_the_phase_one_read_tools() {
    assert_eq!(
        sift::mcp::tool_names(),
        [
            "sift_correlate",
            "sift_get_trace",
            "sift_list_services",
            "sift_query",
            "sift_tail_logs",
        ]
    );
}

#[tokio::test]
async fn stdio_server_negotiates_and_lists_the_five_sift_tools() {
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "mcp",
            "serve",
            "--stdio",
            "--endpoint",
            "http://127.0.0.1:9",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .expect("start Sift MCP stdio server");
    let mut stdin = child.stdin.take().expect("MCP stdin");
    let stdout = child.stdout.take().expect("MCP stdout");
    let mut lines = BufReader::new(stdout).lines();

    stdin
        .write_all(
            concat!(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sift-e2e","version":"1"}}}"#,
                "\n"
            )
            .as_bytes(),
        )
        .await
        .expect("write initialize request");
    stdin.flush().await.expect("flush initialize request");
    let initialize = tokio::time::timeout(Duration::from_secs(5), lines.next_line())
        .await
        .expect("initialize response deadline")
        .expect("read initialize response")
        .expect("initialize response line");
    let initialize: serde_json::Value =
        serde_json::from_str(&initialize).expect("initialize response JSON");
    assert_eq!(initialize["id"], 1);
    assert_eq!(initialize["result"]["serverInfo"]["name"], "sift");

    stdin
        .write_all(
            concat!(
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                "\n",
                r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
                "\n"
            )
            .as_bytes(),
        )
        .await
        .expect("write tools/list request");
    stdin.flush().await.expect("flush tools/list request");

    let tools = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let line = lines
                .next_line()
                .await
                .expect("read tools/list response")
                .expect("tools/list response line");
            let value: serde_json::Value =
                serde_json::from_str(&line).expect("tools/list response JSON");
            if value["id"] == 2 {
                break value;
            }
        }
    })
    .await
    .expect("tools/list response deadline");
    let mut names = tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect::<Vec<_>>();
    names.sort_unstable();
    assert_eq!(names, sift::mcp::tool_names());

    drop(stdin);
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("MCP process exit deadline")
        .expect("wait for MCP process");
    assert!(status.success(), "MCP stdio server exited with {status}");
}

#[tokio::test]
async fn stdio_get_trace_uses_the_canonical_trace_path() {
    async fn get_trace(
        Path(trace_id): Path<String>,
        Query(query): Query<HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "project": query.get("project"),
            "trace_id": trace_id,
            "spans": [{"span_id": "bbbbbbbbbbbbbbbb"}],
        }))
    }

    let app = Router::new().route("/api/v1/traces/{trace_id}", get(get_trace));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake Sift API");
    let endpoint = format!(
        "http://{}",
        listener.local_addr().expect("fake API address")
    );
    let api = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve fake Sift API");
    });

    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_sift"))
        .args(["mcp", "serve", "--stdio", "--endpoint", &endpoint])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .expect("start Sift MCP stdio server");
    let mut stdin = child.stdin.take().expect("MCP stdin");
    let stdout = child.stdout.take().expect("MCP stdout");
    let mut lines = BufReader::new(stdout).lines();

    stdin
        .write_all(
            concat!(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sift-e2e","version":"1"}}}"#,
                "\n"
            )
            .as_bytes(),
        )
        .await
        .expect("write initialize request");
    stdin.flush().await.expect("flush initialize request");
    let initialize = tokio::time::timeout(Duration::from_secs(5), lines.next_line())
        .await
        .expect("initialize response deadline")
        .expect("read initialize response")
        .expect("initialize response line");
    let initialize: serde_json::Value =
        serde_json::from_str(&initialize).expect("initialize response JSON");
    assert_eq!(initialize["id"], 1);

    stdin
        .write_all(
            concat!(
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                "\n",
                r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"sift_get_trace","arguments":{"project":"sift-mvp","trace_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}}"#,
                "\n"
            )
            .as_bytes(),
        )
        .await
        .expect("write sift_get_trace request");
    stdin.flush().await.expect("flush sift_get_trace request");

    let response = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let line = lines
                .next_line()
                .await
                .expect("read sift_get_trace response")
                .expect("sift_get_trace response line");
            let value: serde_json::Value =
                serde_json::from_str(&line).expect("sift_get_trace response JSON");
            if value["id"] == 2 {
                break value;
            }
        }
    })
    .await
    .expect("sift_get_trace response deadline");
    assert!(
        response.get("error").is_none(),
        "sift_get_trace failed: {response}"
    );
    assert_ne!(response["result"]["isError"], true);
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("sift_get_trace JSON text");
    let trace: serde_json::Value = serde_json::from_str(text).expect("trace JSON");
    assert_eq!(trace["project"], "sift-mvp");
    assert_eq!(trace["trace_id"], "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(trace["spans"][0]["span_id"], "bbbbbbbbbbbbbbbb");

    drop(stdin);
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("MCP process exit deadline")
        .expect("wait for MCP process");
    assert!(status.success(), "MCP stdio server exited with {status}");
    api.abort();
}

#[tokio::test]
async fn streamable_http_rejects_an_untrusted_browser_origin() {
    let app = sift::mcp::http_router("http://127.0.0.1:7380").expect("build MCP HTTP router");
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("host", "localhost")
        .header("origin", "https://evil.example")
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .body(Body::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn streamable_http_accepts_a_configured_local_origin() {
    let app = sift::mcp::http_router("http://127.0.0.1:7380").expect("build MCP HTTP router");
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("host", "localhost")
        .header("origin", "http://127.0.0.1:7380")
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .body(Body::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert!(response.headers().contains_key("mcp-session-id"));
    assert_eq!(response.headers()["content-type"], "text/event-stream");
}
