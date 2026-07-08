// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
//! HTTP reverse proxy handler for the Jet dev server.
//!
//! Reads `dev.proxy` map from `jet.toml` (path prefix → target URL),
//! matches incoming requests using longest-prefix matching, and forwards them
//! to the configured target while:
//!
//! - Preserving method, headers, and body.
//! - Setting `X-Forwarded-For`, `X-Forwarded-Host`, `X-Forwarded-Proto`.
//! - Streaming SSE and other chunked responses without buffering.
//! - Tunnelling WebSocket connections bidirectionally via tokio-tungstenite.

use axum::body::Body;
use axum::extract::ws::WebSocket;
use axum::http::{header, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite};

/// Hop-by-hop headers that must not be forwarded (RFC 7230 §6.1).
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

fn is_hop_by_hop(name: &str) -> bool {
    HOP_BY_HOP.contains(&name)
}

/// HTTP reverse proxy handler.
///
/// Routes are matched using longest-prefix: a request to `/api/v1/foo` will
/// match the `/api/v1` rule before `/api` when both are configured.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub struct ProxyHandler {
    client: Client,
    /// Routes sorted by descending prefix length (longest first).
    routes: Vec<(String, String)>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl ProxyHandler {
    /// Build a new `ProxyHandler` from a proxy map (`path_prefix → target_url`).
    pub fn new(proxy_map: HashMap<String, String>) -> Self {
        let mut routes: Vec<(String, String)> = proxy_map.into_iter().collect();
        routes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Failed to build proxy HTTP client");

        Self { client, routes }
    }

    /// Returns `true` when no proxy routes are configured.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Longest-prefix match of `path` against the configured routes.
    ///
    /// Match is path-segment aware: `/api` matches `/api`, `/api/`,
    /// `/api/users`, but NOT `/apidocs`. Mirrors Vite / webpack-dev-server
    /// proxy semantics. GH #3090 — `starts_with` alone misroutes any URL
    /// whose first segment shares a prefix with a configured route.
    ///
    /// Returns the target base URL (e.g. `"http://localhost:3200"`) when matched.
    pub fn match_target<'a>(&'a self, path: &str) -> Option<&'a str> {
        self.routes
            .iter()
            .find(|(prefix, _)| segment_prefix_match(path, prefix))
            .map(|(_, target)| target.as_str())
    }

    /// Forward a plain HTTP (or SSE) request to the matched proxy target.
    ///
    /// Sets `X-Forwarded-For`, `X-Forwarded-Host`, and `X-Forwarded-Proto`
    /// before forwarding. Streams the response body back to the client without
    /// buffering so SSE works correctly.
    pub async fn forward_http(&self, req: Request<Body>) -> Response {
        let path_and_query = req
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
            .to_string();

        let path_only = req.uri().path();

        let target = match self.match_target(path_only) {
            Some(t) => t.to_string(),
            None => return StatusCode::NOT_FOUND.into_response(),
        };

        let target_url = format!("{}{}", target.trim_end_matches('/'), path_and_query);
        tracing::debug!("Proxy HTTP → {}", target_url);

        let method: reqwest::Method = req.method().clone();
        let in_headers = req.headers().clone();

        let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Proxy: failed to read request body: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        let mut builder = self.client.request(method, &target_url);

        // Forward non-hop-by-hop headers from client.
        for (name, value) in &in_headers {
            if !is_hop_by_hop(name.as_str()) {
                builder = builder.header(name, value);
            }
        }

        // X-Forwarded-For
        if let Some(xff) = in_headers.get("x-forwarded-for") {
            builder = builder.header("x-forwarded-for", xff);
        } else {
            builder = builder.header("x-forwarded-for", "127.0.0.1");
        }

        // X-Forwarded-Host
        if let Some(host) = in_headers.get(header::HOST) {
            builder = builder.header("x-forwarded-host", host);
        }

        // X-Forwarded-Proto
        builder = builder.header("x-forwarded-proto", "http");

        builder = builder.body(body_bytes.to_vec());

        match builder.send().await {
            Ok(res) => convert_response(res).await,
            Err(e) => {
                tracing::error!("Proxy request to {} failed: {}", target_url, e);
                (StatusCode::BAD_GATEWAY, format!("Proxy error: {}", e)).into_response()
            }
        }
    }

    /// Tunnel a WebSocket connection to the matched proxy target.
    ///
    /// Creates a fully bidirectional bridge: messages from the Axum client
    /// WebSocket are forwarded to the upstream (tungstenite), and vice versa.
    pub async fn forward_websocket(&self, ws: WebSocket, path: &str) {
        let target = match self.match_target(path) {
            Some(t) => t.to_string(),
            None => {
                tracing::warn!("WS proxy: no match for path '{}'", path);
                return;
            }
        };

        let ws_url = http_to_ws_url(&target, path);
        tracing::debug!("Proxy WS → {}", ws_url);

        let upstream_stream = match connect_async(&ws_url).await {
            Ok((stream, _)) => stream,
            Err(e) => {
                tracing::error!("WS proxy: upstream connect failed ({}): {}", ws_url, e);
                return;
            }
        };

        let (mut upstream_tx, mut upstream_rx) = upstream_stream.split();
        let (mut client_tx, mut client_rx) = ws.split();

        // Client → Upstream
        let c2u = tokio::spawn(async move {
            while let Some(Ok(msg)) = client_rx.next().await {
                let tg_msg = match msg {
                    axum::extract::ws::Message::Text(t) => {
                        tungstenite::Message::text(t.to_string())
                    }
                    axum::extract::ws::Message::Binary(b) => tungstenite::Message::Binary(b),
                    axum::extract::ws::Message::Ping(d) => tungstenite::Message::Ping(d),
                    axum::extract::ws::Message::Pong(d) => tungstenite::Message::Pong(d),
                    axum::extract::ws::Message::Close(_) => break,
                };
                if upstream_tx.send(tg_msg).await.is_err() {
                    break;
                }
            }
        });

        // Upstream → Client
        let u2c = tokio::spawn(async move {
            while let Some(Ok(msg)) = upstream_rx.next().await {
                let client_msg = match msg {
                    tungstenite::Message::Text(t) => {
                        axum::extract::ws::Message::Text(t.to_string().into())
                    }
                    tungstenite::Message::Binary(b) => axum::extract::ws::Message::Binary(b),
                    tungstenite::Message::Ping(d) => axum::extract::ws::Message::Ping(d),
                    tungstenite::Message::Pong(d) => axum::extract::ws::Message::Pong(d),
                    tungstenite::Message::Close(_) => break,
                    tungstenite::Message::Frame(_) => continue,
                };
                if client_tx.send(client_msg).await.is_err() {
                    break;
                }
            }
        });

        tokio::select! {
            _ = c2u => {},
            _ = u2c => {},
        }

        tracing::debug!("WS proxy tunnel closed for '{}'", path);
    }
}

/// Format the warning emitted when axum's response-builder rejects the
/// upstream response shape during proxy conversion. Names the upstream
/// status and the underlying `axum::http::Error` (which includes the
/// offending header name when the failure is header-related), and tags
/// `GH #3544` so developers grepping "dev-server proxy returns 500" can
/// land on this line. Extracted for unit-test pinning.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_proxy_body_build_warn(status: StatusCode, err: &axum::http::Error) -> String {
    format!(
        "GH #3544 dev-server proxy response builder rejected upstream response (status={}): {}; the browser will see a bare 500 with no body. The underlying axum::http::Error usually names the offending header — common causes are upstream emitting an invalid header value (CR/LF, non-ASCII without encoding) or a header name that fails HTTP token validation. Check the upstream service's response headers for the named field.",
        status, err
    )
}

/// Convert a reqwest response to an axum response, streaming the body.
///
/// Skips hop-by-hop headers and streams the body so SSE connections are
/// forwarded without buffering.
///
/// GH #3544 — the prior `.unwrap_or_else(|_| ...)` discarded the
/// `axum::http::Error` (which names the offending header), leaving
/// operators with a bare 500 and no breadcrumb. We now match on the
/// builder result so a structured warn is emitted per build failure.
async fn convert_response(res: reqwest::Response) -> Response {
    let status =
        StatusCode::from_u16(res.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let mut builder = axum::response::Response::builder().status(status);

    for (name, value) in res.headers() {
        if !is_hop_by_hop(name.as_str()) {
            builder = builder.header(name, value);
        }
    }

    let body = Body::from_stream(res.bytes_stream());

    match builder.body(body) {
        Ok(response) => response,
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_server::proxy",
                status = %status,
                error = %err,
                "{}",
                format_proxy_body_build_warn(status, &err)
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GH #3090 — match `prefix` against `path` only on path-segment
/// boundaries: `prefix == path`, or `prefix` is followed by `/` in
/// `path`. `/api` matches `/api`, `/api/`, `/api/users`, but NOT
/// `/apidocs` / `/apivertise`. Trailing slashes on the prefix are
/// normalised so `"/api/"` behaves the same as `"/api"`.
fn segment_prefix_match(path: &str, prefix: &str) -> bool {
    let prefix = prefix.trim_end_matches('/');
    if prefix.is_empty() {
        // An empty / root prefix matches every path — preserves the
        // previous "match-everything" behaviour for that degenerate case.
        return true;
    }
    if path == prefix {
        return true;
    }
    match path.strip_prefix(prefix) {
        Some(rest) => rest.starts_with('/'),
        None => false,
    }
}

/// Replace `http://` / `https://` scheme with `ws://` / `wss://` for WebSocket URLs.
fn http_to_ws_url(target: &str, path: &str) -> String {
    let base = if let Some(rest) = target.strip_prefix("https://") {
        format!("wss://{}", rest)
    } else if let Some(rest) = target.strip_prefix("http://") {
        format!("ws://{}", rest)
    } else {
        target.to_string()
    };
    format!("{}{}", base.trim_end_matches('/'), path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ─── Pure unit tests (no network) ─────────────────────────────────────────

    /// REQ-JET-02: longest-prefix match routes to correct target.
    #[test]
    fn proxy_longest_prefix_match() {
        let mut map = HashMap::new();
        map.insert("/api".to_string(), "http://localhost:3200".to_string());
        map.insert("/api/v1".to_string(), "http://localhost:3201".to_string());
        map.insert("/mcp".to_string(), "http://localhost:3202".to_string());

        let handler = ProxyHandler::new(map);

        // Longer prefix wins over shorter
        assert_eq!(
            handler.match_target("/api/v1/users"),
            Some("http://localhost:3201")
        );
        // Shorter prefix still matches when no longer one exists
        assert_eq!(
            handler.match_target("/api/health"),
            Some("http://localhost:3200")
        );
        // Unrelated prefix matches its own target
        assert_eq!(
            handler.match_target("/mcp/stream"),
            Some("http://localhost:3202")
        );
        // No match
        assert_eq!(handler.match_target("/other"), None);
        assert_eq!(handler.match_target("/"), None);
    }

    #[test]
    fn proxy_no_match_returns_none() {
        let mut map = HashMap::new();
        map.insert("/api".to_string(), "http://localhost:3200".to_string());
        let handler = ProxyHandler::new(map);
        assert_eq!(handler.match_target("/webhook"), None);
    }

    /// GH #3090 — historic bug: bare `starts_with` matched any path that
    /// shared the prefix as a substring, crossing path-segment boundaries.
    /// `/apidocs` was incorrectly forwarded to the `/api` target. The
    /// segment-aware match must reject those.
    #[test]
    fn proxy_prefix_does_not_cross_segment_boundary() {
        let mut map = HashMap::new();
        map.insert("/api".to_string(), "http://localhost:3200".to_string());
        let handler = ProxyHandler::new(map);

        // Exact match and any deeper segment must still match.
        assert_eq!(
            handler.match_target("/api"),
            Some("http://localhost:3200"),
            "exact prefix path must match",
        );
        assert_eq!(
            handler.match_target("/api/"),
            Some("http://localhost:3200"),
            "prefix with trailing slash must match",
        );
        assert_eq!(
            handler.match_target("/api/users"),
            Some("http://localhost:3200"),
            "deeper path must match",
        );

        // Substring-only collisions must NOT match.
        assert_eq!(
            handler.match_target("/apidocs"),
            None,
            "/apidocs shares /api as substring but is a different resource",
        );
        assert_eq!(
            handler.match_target("/apivertise"),
            None,
            "/apivertise shares /api as substring but is a different resource",
        );
    }

    /// GH #3090 — a prefix written with a trailing slash (`"/api/"`) in the
    /// config must behave the same as without: still match `/api`, `/api/`,
    /// `/api/users`; still reject `/apidocs`.
    #[test]
    fn proxy_prefix_trailing_slash_normalised() {
        let mut map = HashMap::new();
        map.insert("/api/".to_string(), "http://localhost:3200".to_string());
        let handler = ProxyHandler::new(map);

        assert_eq!(handler.match_target("/api"), Some("http://localhost:3200"));
        assert_eq!(handler.match_target("/api/"), Some("http://localhost:3200"));
        assert_eq!(
            handler.match_target("/api/users"),
            Some("http://localhost:3200")
        );
        assert_eq!(handler.match_target("/apidocs"), None);
    }

    #[test]
    fn proxy_is_empty_without_routes() {
        let handler = ProxyHandler::new(HashMap::new());
        assert!(handler.is_empty());
    }

    #[test]
    fn proxy_is_not_empty_with_routes() {
        let mut map = HashMap::new();
        map.insert("/api".to_string(), "http://localhost:3200".to_string());
        let handler = ProxyHandler::new(map);
        assert!(!handler.is_empty());
    }

    #[test]
    fn http_to_ws_url_http_scheme() {
        assert_eq!(
            http_to_ws_url("http://localhost:3200", "/mcp/stream"),
            "ws://localhost:3200/mcp/stream"
        );
    }

    #[test]
    fn http_to_ws_url_https_scheme() {
        assert_eq!(
            http_to_ws_url("https://example.com", "/ws"),
            "wss://example.com/ws"
        );
    }

    #[test]
    fn http_to_ws_url_trailing_slash_stripped() {
        assert_eq!(
            http_to_ws_url("http://localhost:3200/", "/ws"),
            "ws://localhost:3200/ws"
        );
    }

    // ─── Network integration tests ────────────────────────────────────────────

    /// REQ-JET-02: forward_http routes request to proxy target and returns response.
    #[tokio::test]
    async fn proxy_forwards_http_request() {
        use axum::routing::get;
        use axum::Router;

        let app = Router::new().route("/api/health", get(|| async { "ok" }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let mut map = HashMap::new();
        map.insert("/api".to_string(), format!("http://{}", addr));
        let handler = ProxyHandler::new(map);

        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/health")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = handler.forward_http(req).await;
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
    }

    /// REQ-JET-03: forward_http sets X-Forwarded-For, X-Forwarded-Host, X-Forwarded-Proto.
    #[tokio::test]
    async fn proxy_sets_forwarded_headers() {
        use axum::routing::any;
        use axum::Router;

        // The upstream echoes back the X-Forwarded headers in the response body
        // as "xff|xfh|xfp" so we can inspect them without shared state.
        let app = Router::new().route(
            "/api/test",
            any(|req: axum::http::Request<axum::body::Body>| async move {
                let xff = req
                    .headers()
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                let xfh = req
                    .headers()
                    .get("x-forwarded-host")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                let xfp = req
                    .headers()
                    .get("x-forwarded-proto")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                format!("{}|{}|{}", xff, xfh, xfp)
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let mut map = HashMap::new();
        map.insert("/api".to_string(), format!("http://{}", addr));
        let handler = ProxyHandler::new(map);

        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/test")
            .header("host", "localhost:3201")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = handler.forward_http(req).await;
        assert_eq!(resp.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(body_bytes.to_vec()).unwrap();
        let parts: Vec<&str> = body.splitn(3, '|').collect();

        assert_eq!(parts.len(), 3, "Body should be xff|xfh|xfp");
        assert!(!parts[0].is_empty(), "X-Forwarded-For should be set");
        assert_eq!(
            parts[1], "localhost:3201",
            "X-Forwarded-Host should match request Host"
        );
        assert_eq!(parts[2], "http", "X-Forwarded-Proto should be http");
    }

    /// REQ-JET-04: SSE content-type is forwarded without buffering (header-level check).
    #[tokio::test]
    async fn proxy_sse_stream_passthrough() {
        use axum::routing::get;
        use axum::Router;

        // Upstream returns an SSE-style response (text/event-stream)
        let app = Router::new().route(
            "/api/events",
            get(|| async {
                (
                    [
                        ("content-type", "text/event-stream"),
                        ("cache-control", "no-cache"),
                    ],
                    "data: hello\n\ndata: world\n\n",
                )
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let mut map = HashMap::new();
        map.insert("/api".to_string(), format!("http://{}", addr));
        let handler = ProxyHandler::new(map);

        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/events")
            .header("accept", "text/event-stream")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = handler.forward_http(req).await;
        assert_eq!(resp.status(), axum::http::StatusCode::OK);

        // Verify SSE content-type is forwarded
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("text/event-stream"),
            "SSE content-type must be forwarded; got: {}",
            content_type
        );
    }

    /// REQ-JET-04: WebSocket messages are tunnelled bidirectionally through the proxy.
    #[tokio::test]
    async fn proxy_websocket_tunnel() {
        use axum::extract::ws::{WebSocket, WebSocketUpgrade};
        use axum::routing::get;
        use axum::Router;
        use futures_util::{SinkExt, StreamExt};
        use std::sync::Arc;
        use tokio_tungstenite::connect_async;

        // 1. Upstream echo WebSocket server
        let echo_app = Router::new().route(
            "/ws",
            get(|ws: WebSocketUpgrade| async move {
                ws.on_upgrade(|socket: WebSocket| async move {
                    let (mut tx, mut rx) = socket.split();
                    while let Some(Ok(msg)) = rx.next().await {
                        if tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                })
            }),
        );
        let echo_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let echo_addr = echo_listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(echo_listener, echo_app).await.unwrap() });

        // 2. Proxy server that upgrades and tunnels to the echo server
        let mut proxy_map = HashMap::new();
        proxy_map.insert("/ws".to_string(), format!("http://{}", echo_addr));
        let proxy = Arc::new(ProxyHandler::new(proxy_map));

        let proxy_clone = proxy.clone();
        let proxy_app = Router::new().route(
            "/ws",
            get(move |ws: WebSocketUpgrade| {
                let p = proxy_clone.clone();
                async move {
                    ws.on_upgrade(move |socket| async move {
                        p.forward_websocket(socket, "/ws").await;
                    })
                }
            }),
        );
        let proxy_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let proxy_addr = proxy_listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(proxy_listener, proxy_app).await.unwrap() });

        // Allow both servers to start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // 3. Connect tungstenite client to proxy, send message, expect echo
        let (mut stream, _) = connect_async(format!("ws://{}/ws", proxy_addr))
            .await
            .expect("WS connect to proxy failed");

        stream
            .send(tokio_tungstenite::tungstenite::Message::text(
                "hello via proxy",
            ))
            .await
            .unwrap();

        let echo = tokio::time::timeout(std::time::Duration::from_secs(3), stream.next())
            .await
            .expect("timeout waiting for WS echo")
            .expect("stream ended")
            .expect("WS error");

        match echo {
            tokio_tungstenite::tungstenite::Message::Text(t) => {
                assert_eq!(t.as_str(), "hello via proxy");
            }
            other => panic!("Expected Text WS message, got: {:?}", other),
        }
    }

    // ─── GH #3544 — proxy response-builder failure surfacing ─────────────

    /// GH #3544 — fabricate a real `axum::http::Error` by feeding the
    /// builder a header name with a forbidden character. This is the
    /// portable way to produce the error type for message-shape tests.
    fn make_axum_http_err() -> axum::http::Error {
        // A header name containing whitespace fails HTTP token validation.
        axum::response::Response::builder()
            .header("bad header name", "x")
            .body(())
            .unwrap_err()
    }

    #[test]
    fn gh3544_format_proxy_body_build_warn_names_status_error_and_issue() {
        let err = make_axum_http_err();
        let msg = format_proxy_body_build_warn(StatusCode::BAD_GATEWAY, &err);
        assert!(
            msg.contains("GH #3544"),
            "warning must carry the GH #3544 tag so users grepping their logs can land here: {msg}"
        );
        assert!(
            msg.contains("502"),
            "warning must name the upstream status verbatim so operators can correlate with access logs: {msg}"
        );
        assert!(
            msg.contains("proxy"),
            "warning must mention 'proxy' so the dev-server-proxy-returns-500 search finds it: {msg}"
        );
    }

    #[test]
    fn gh3544_format_proxy_body_build_warn_includes_underlying_error() {
        let err = make_axum_http_err();
        let underlying = err.to_string();
        let msg = format_proxy_body_build_warn(StatusCode::OK, &err);
        assert!(
            msg.contains(&underlying),
            "warning must include the underlying axum::http::Error so operators see WHICH header was rejected: msg={msg} underlying={underlying}"
        );
    }

    #[test]
    fn gh3544_format_proxy_body_build_warn_mentions_response_builder() {
        let err = make_axum_http_err();
        let msg = format_proxy_body_build_warn(StatusCode::INTERNAL_SERVER_ERROR, &err);
        assert!(
            msg.to_lowercase().contains("response builder")
                || msg.to_lowercase().contains("response-builder")
                || msg.to_lowercase().contains("builder"),
            "warning must mention 'builder' so operators searching by symptom find this line: {msg}"
        );
    }
}
// CODEGEN-END
