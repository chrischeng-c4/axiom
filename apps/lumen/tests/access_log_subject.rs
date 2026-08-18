// Verify access log subject field is recorded from auth context
// Tests AC1: the http.access line contains authenticated subject or "anonymous"
//
// #2871: the named-subject case is not covered here any more. Producing one
// required a bearer the registry could resolve to `alice@example.com`, and
// that registry is retired; a named subject becomes producible again when
// TokenReview returns `system:serviceaccount:<ns>:<name>` (#2869), which is
// where that assertion should be rebuilt. What is still covered is that the
// `subject` attribute is always present — on an open request, and on a
// rejected one, where losing the line entirely would be the worse failure.

use std::io;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::Request;
use axum_test::TestServer;
use tower::ServiceExt as _;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::storage::Engine;

#[derive(Clone, Default)]
struct SharedWriter {
    bytes: Arc<Mutex<Vec<u8>>>,
}

struct SharedWriterGuard {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl io::Write for SharedWriterGuard {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.bytes.lock().unwrap().extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for SharedWriter {
    type Writer = SharedWriterGuard;

    fn make_writer(&'writer self) -> Self::Writer {
        SharedWriterGuard {
            bytes: Arc::clone(&self.bytes),
        }
    }
}

impl SharedWriter {
    fn output(&self) -> String {
        String::from_utf8(self.bytes.lock().unwrap().clone()).unwrap()
    }
}

#[tokio::test]
async fn access_log_records_anonymous_when_auth_disabled() {
    let writer = SharedWriter::default();
    let identity = service_observability::ServiceIdentity::new("lumen", "test").unwrap();
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(writer.clone())
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(false)
        .json()
        .with_current_span(true)
        .event_format(service_observability::ServiceJsonFormatter::new(identity));

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::filter::EnvFilter::new("debug"));

    let _guard = tracing::subscriber::set_default(subscriber);

    // Set up lumen with auth disabled (optional)
    let engine = Arc::new(Engine::new());
    let cfg = AuthConfig::open(); // auth off
    let app = router(AppState::new(engine, Arc::new(cfg)));

    // Make a request without bearer token using tower
    let request = Request::builder()
        .method("GET")
        .uri("/collections")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status().as_u16(), 200);

    let output = writer.output();
    let records: Vec<_> = output
        .lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }
            serde_json::from_str::<serde_json::Value>(line).ok()
        })
        .collect();

    // Find the access log line for /collections
    let access_log_line = records
        .iter()
        .find(|r| {
            r.get("attributes")
                .and_then(|a| a.get("target"))
                .and_then(|t| t.as_str())
                == Some("http.access")
                && r.get("attributes")
                    .and_then(|a| a.get("uri"))
                    .and_then(|u| u.as_str())
                    .map(|uri| uri.ends_with("/collections"))
                    .unwrap_or(false)
        })
        .expect("should have http.access line for /collections");

    // Verify the subject field is "anonymous"
    let subject = access_log_line
        .get("attributes")
        .and_then(|a| a.get("subject"))
        .and_then(|s| s.as_str());

    assert_eq!(
        subject,
        Some("anonymous"),
        "unauthenticated request should contain subject='anonymous'"
    );
}

#[tokio::test]
async fn access_log_records_anonymous_for_rejected_request() {
    let writer = SharedWriter::default();
    let identity = service_observability::ServiceIdentity::new("lumen", "test").unwrap();
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(writer.clone())
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(false)
        .json()
        .with_current_span(true)
        .event_format(service_observability::ServiceJsonFormatter::new(identity));

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::filter::EnvFilter::new("debug"));

    let _guard = tracing::subscriber::set_default(subscriber);

    // Set up lumen with auth required and no review backend wired, which is
    // exactly the state this test wants: a rejected request (#2869).
    let engine = Arc::new(Engine::new());
    let cfg = AuthConfig::required_in("serving");
    let app = router(AppState::new(engine, Arc::new(cfg)));
    let server = TestServer::new(app).expect("test server");

    // Make a request WITHOUT bearer token (should be rejected with 401)
    let resp = server.get("/collections").await;
    assert_eq!(
        resp.status_code(),
        401,
        "request without auth token should be rejected with 401"
    );

    let output = writer.output();
    let records: Vec<_> = output
        .lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }
            serde_json::from_str::<serde_json::Value>(line).ok()
        })
        .collect();

    // Find the access log line for /collections
    let access_log_line = records
        .iter()
        .find(|r| {
            r.get("attributes")
                .and_then(|a| a.get("target"))
                .and_then(|t| t.as_str())
                == Some("http.access")
                && r.get("attributes")
                    .and_then(|a| a.get("uri"))
                    .and_then(|u| u.as_str())
                    .map(|uri| uri.ends_with("/collections"))
                    .unwrap_or(false)
        })
        .expect("rejected request must still produce http.access line for /collections");

    // Verify the subject field is "anonymous" even for rejected request
    let subject = access_log_line
        .get("attributes")
        .and_then(|a| a.get("subject"))
        .and_then(|s| s.as_str());

    assert_eq!(
        subject,
        Some("anonymous"),
        "rejected request must still record subject=anonymous"
    );
}
