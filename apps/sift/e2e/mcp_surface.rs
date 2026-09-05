//! Black-box contracts for Sift's read-only MCP surface.

use std::{
    collections::HashMap,
    process::Stdio,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, State},
    http::{header, HeaderMap, Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
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
async fn stdio_query_retries_a_detailed_retryable_response() {
    struct RetryRequest {
        body: Vec<u8>,
        project: Option<String>,
        authorization: Option<String>,
    }

    #[derive(Default)]
    struct RetryState {
        attempts: AtomicUsize,
        requests: Mutex<Vec<RetryRequest>>,
    }

    async fn query(
        State(state): State<Arc<RetryState>>,
        headers: HeaderMap,
        body: Bytes,
    ) -> axum::response::Response {
        state.requests.lock().unwrap().push(RetryRequest {
            body: body.to_vec(),
            project: headers
                .get("x-sift-project")
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
            authorization: headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
        });
        if state.attempts.fetch_add(1, Ordering::SeqCst) == 0 {
            return service_http::ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "retention_checkpoint_pending",
                "retry after the checkpoint is ready",
            )
            .with_retry_after_seconds(1)
            .into_response();
        }
        Json(serde_json::json!({
            "data": {"records": [{"event_id": "retried-log"}]},
            "next_cursor": null,
            "watermark": 1,
            "partial": false,
            "warnings": [],
            "stats": {"elapsed_ms": 0, "scanned": 1, "returned": 1},
            "query_id": null,
        }))
        .into_response()
    }

    let state = Arc::new(RetryState::default());
    let app = Router::new()
        .route("/api/v1/query", post(query))
        .with_state(state.clone());
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
        .args([
            "mcp",
            "serve",
            "--stdio",
            "--endpoint",
            &endpoint,
            "--token",
            "mcp-retry-token",
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

    let query_started = Instant::now();
    stdin
        .write_all(
            concat!(
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                "\n",
                r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"sift_query","arguments":{"request":{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"retried-log"}},"limit":10,"mode":"sync"}}}}"#,
                "\n"
            )
            .as_bytes(),
        )
        .await
        .expect("write sift_query request");
    stdin.flush().await.expect("flush sift_query request");

    let response = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let line = lines
                .next_line()
                .await
                .expect("read sift_query response")
                .expect("sift_query response line");
            let value: serde_json::Value =
                serde_json::from_str(&line).expect("sift_query response JSON");
            if value["id"] == 2 {
                break value;
            }
        }
    })
    .await
    .expect("sift_query response deadline");
    assert!(
        response.get("error").is_none(),
        "sift_query did not retry: {response}"
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("sift_query JSON text");
    let query: serde_json::Value = serde_json::from_str(text).expect("query JSON");
    assert_eq!(query["data"]["records"][0]["event_id"], "retried-log");
    assert_eq!(state.attempts.load(Ordering::SeqCst), 2);
    let requests = state.requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].body, requests[1].body);
    let request: serde_json::Value =
        serde_json::from_slice(&requests[0].body).expect("retried query request JSON");
    assert_eq!(request["project"], "sift-mvp");
    assert_eq!(request["signal"]["kind"], "logs");
    for request in requests.iter() {
        assert_eq!(request.project.as_deref(), Some("sift-mvp"));
        assert_eq!(
            request.authorization.as_deref(),
            Some("Bearer mcp-retry-token")
        );
    }
    drop(requests);
    assert!(
        query_started.elapsed() >= Duration::from_millis(900),
        "sift_query did not honor Retry-After"
    );

    drop(stdin);
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("MCP process exit deadline")
        .expect("wait for MCP process");
    assert!(status.success(), "MCP stdio server exited with {status}");
    api.abort();
}

fn retry_test_query() -> sift::api::QueryRequestV1 {
    serde_json::from_value(serde_json::json!({
        "version": 1,
        "project": "sift-mvp",
        "environment": "gke",
        "signal": {
            "kind": "logs",
            "filter": {"op": "eq", "field": "event_id", "value": "retry-boundary"}
        },
        "limit": 10,
        "mode": "sync"
    }))
    .expect("retry test query")
}

#[tokio::test]
async fn api_client_stops_after_three_retryable_responses() {
    async fn query(State(attempts): State<Arc<AtomicUsize>>) -> axum::response::Response {
        attempts.fetch_add(1, Ordering::SeqCst);
        service_http::ApiErr::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "retention_checkpoint_pending",
            "retry after the checkpoint is ready",
        )
        .with_retry_after_seconds(0)
        .into_response()
    }

    let attempts = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/api/v1/query", post(query))
        .with_state(attempts.clone());
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

    let client = sift::mcp::SiftApiClient::new(&endpoint, None, Duration::from_secs(2))
        .expect("build Sift API client");
    let error = client
        .query(&retry_test_query())
        .await
        .err()
        .expect("bounded retry failure");
    assert!(error.to_string().contains("503 Service Unavailable"));
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    api.abort();
}

#[tokio::test]
async fn api_client_does_not_replay_queries_that_can_create_jobs() {
    async fn query(State(attempts): State<Arc<AtomicUsize>>) -> axum::response::Response {
        attempts.fetch_add(1, Ordering::SeqCst);
        service_http::ApiErr::new(
            StatusCode::BAD_GATEWAY,
            "upstream_response_failed",
            "the internal service role returned an incomplete response",
        )
        .with_retryable(true)
        .into_response()
    }

    let attempts = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/api/v1/query", post(query))
        .with_state(attempts.clone());
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

    let client = sift::mcp::SiftApiClient::new(&endpoint, None, Duration::from_secs(2))
        .expect("build Sift API client");
    let mut asynchronous = retry_test_query();
    asynchronous.mode = sift::api::QueryModeV1::Async;
    let error = client
        .query(&asynchronous)
        .await
        .err()
        .expect("async query proxy failure");
    assert!(error.to_string().contains("502 Bad Gateway"));

    let mut automatic = retry_test_query();
    automatic.mode = sift::api::QueryModeV1::Auto;
    automatic.limit = 501;
    let error = client
        .query(&automatic)
        .await
        .err()
        .expect("auto query proxy failure");
    assert!(error.to_string().contains("502 Bad Gateway"));

    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    api.abort();
}

#[tokio::test]
async fn api_client_refuses_a_retry_delay_above_five_seconds() {
    async fn query(State(attempts): State<Arc<AtomicUsize>>) -> axum::response::Response {
        attempts.fetch_add(1, Ordering::SeqCst);
        service_http::ApiErr::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "retention_checkpoint_pending",
            "retry after the checkpoint is ready",
        )
        .with_retry_after_seconds(6)
        .into_response()
    }

    let attempts = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/api/v1/query", post(query))
        .with_state(attempts.clone());
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

    let client = sift::mcp::SiftApiClient::new(&endpoint, None, Duration::from_secs(20))
        .expect("build Sift API client");
    let request = retry_test_query();
    let result = tokio::time::timeout(Duration::from_millis(500), client.query(&request))
        .await
        .expect("oversized retry delay must be refused without sleeping");
    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    api.abort();
}

#[tokio::test]
async fn api_client_retries_share_one_total_timeout() {
    async fn query(State(attempts): State<Arc<AtomicUsize>>) -> axum::response::Response {
        attempts.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(150)).await;
        service_http::ApiErr::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "retention_checkpoint_pending",
            "retry after the checkpoint is ready",
        )
        .with_retry_after_seconds(0)
        .into_response()
    }

    let attempts = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/api/v1/query", post(query))
        .with_state(attempts.clone());
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

    let client = sift::mcp::SiftApiClient::new(&endpoint, None, Duration::from_millis(250))
        .expect("build Sift API client");
    let started = Instant::now();
    let error = client
        .query(&retry_test_query())
        .await
        .err()
        .expect("total retry timeout");
    assert!(error.to_string().contains("send Sift API request"));
    assert!(started.elapsed() < Duration::from_millis(350));
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
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
