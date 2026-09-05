use std::sync::{Arc, Mutex};

use axum::{body::Body, http::Request};
use service_mcp::{streamable_http_router, HttpTransportConfig, McpApplication};
use tower::ServiceExt;

#[derive(Clone, Default)]
struct FakeApplication {
    forwarded: Arc<Mutex<Vec<Option<String>>>>,
}

impl rmcp::ServerHandler for FakeApplication {}

impl McpApplication for FakeApplication {
    fn with_bearer_token(&self, token: Option<String>) -> Self {
        self.forwarded.lock().unwrap().push(token);
        self.clone()
    }
}

fn initialize(origin: &str, token: Option<&str>) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("host", "localhost")
        .header("origin", origin)
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    request
        .body(Body::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
        ))
        .unwrap()
}

#[tokio::test]
async fn http_shell_rejects_origin_and_forwards_bearer_to_the_application() {
    let application = FakeApplication::default();
    let observed = application.forwarded.clone();
    let router = streamable_http_router(
        "/mcp",
        application,
        HttpTransportConfig::new(
            vec!["localhost".into()],
            vec!["https://trusted.example".into()],
        )
        .unwrap(),
    );

    let rejected = router
        .clone()
        .oneshot(initialize("https://evil.example", None))
        .await
        .unwrap();
    assert_eq!(rejected.status(), 403);

    let accepted = router
        .oneshot(initialize("https://trusted.example", Some("secret")))
        .await
        .unwrap();
    assert_eq!(accepted.status(), 200);
    assert!(accepted.headers().contains_key("mcp-session-id"));
    assert_eq!(accepted.headers()["content-type"], "text/event-stream");
    let observed = observed.lock().unwrap();
    assert_eq!(observed.last(), Some(&Some("secret".into())));
}
