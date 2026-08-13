// Subscriber-capture tests for access log feature (#2792)
use std::io;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use service_http::trace_layer;
use service_observability::{ServiceIdentity, ServiceJsonFormatter};
use tower::ServiceExt as _;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;

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

async fn handler_ok() -> &'static str {
    "ok"
}

#[tokio::test(flavor = "current_thread")]
async fn access_log_emits_info_for_non_probe_requests() {
    let writer = SharedWriter::default();
    let identity = ServiceIdentity::new("test-service", "1.0.0").unwrap();
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(writer.clone())
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(false)
        .json()
        .with_current_span(true)
        .event_format(ServiceJsonFormatter::new(identity));

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::filter::EnvFilter::new("debug"));

    let app = Router::new()
        .route("/test", get(handler_ok))
        .layer(trace_layer());

    let _guard = tracing::subscriber::set_default(subscriber);

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    let output = writer.output();
    eprintln!("Captured output length: {}", output.len());
    eprintln!("Output:\n{}", output);
    let records: Vec<_> = output
        .lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }
            serde_json::from_str::<serde_json::Value>(line).ok()
        })
        .collect();

    eprintln!("Parsed {} records", records.len());
    for (i, r) in records.iter().enumerate() {
        let target = r.get("attributes").and_then(|a| a.get("target")).and_then(|t| t.as_str());
        eprintln!("Record {}: target={:?}, severity={:?}", i, target, r.get("severity"));
    }

    // Find the access log line - should be INFO level for non-probe
    let access_log_line = records
        .iter()
        .find(|r| {
            r.get("attributes")
                .and_then(|a| a.get("target"))
                .and_then(|t| t.as_str()) == Some("http.access")
                && r.get("severity").and_then(|s| s.as_str()) == Some("INFO")
        })
        .expect("should have INFO-level access log line for non-probe request");

    // Verify fields
    assert_eq!(
        access_log_line
            .get("attributes")
            .and_then(|a| a.get("status"))
            .and_then(|s| s.as_u64()),
        Some(200),
        "access log should have status field"
    );
    assert!(
        access_log_line
            .get("attributes")
            .and_then(|a| a.get("latency_ms"))
            .is_some(),
        "access log should have latency_ms field"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn access_log_emits_debug_for_probe_requests() {
    let writer = SharedWriter::default();
    let identity = ServiceIdentity::new("test-service", "1.0.0").unwrap();
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(writer.clone())
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(false)
        .json()
        .with_current_span(true)
        .event_format(ServiceJsonFormatter::new(identity));

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::filter::EnvFilter::new("debug"));

    let app = Router::new()
        .route("/healthz", get(handler_ok))
        .layer(trace_layer());

    let _guard = tracing::subscriber::set_default(subscriber);

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();

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

    // Find the access log line - should be DEBUG level for probe
    let access_log_line = records
        .iter()
        .find(|r| {
            r.get("attributes")
                .and_then(|a| a.get("target"))
                .and_then(|t| t.as_str()) == Some("http.access")
                && r.get("severity").and_then(|s| s.as_str()) == Some("DEBUG")
        })
        .expect("should have DEBUG-level access log line for probe request");

    // Verify fields
    assert_eq!(
        access_log_line
            .get("attributes")
            .and_then(|a| a.get("status"))
            .and_then(|s| s.as_u64()),
        Some(200),
        "probe access log should have status field"
    );
    assert!(
        access_log_line
            .get("attributes")
            .and_then(|a| a.get("latency_ms"))
            .is_some(),
        "probe access log should have latency_ms field"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn access_log_probe_paths_all_demoted() {
    let probe_paths = ["/healthz", "/readyz", "/metrics", "/openapi.json", "/docs"];

    for path in &probe_paths {
        let writer = SharedWriter::default();
        let identity = ServiceIdentity::new("test-service", "1.0.0").unwrap();
        let layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(writer.clone())
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_line_number(false)
            .json()
            .with_current_span(true)
            .event_format(ServiceJsonFormatter::new(identity));

        let subscriber = tracing_subscriber::registry()
            .with(layer)
            .with(tracing_subscriber::filter::EnvFilter::new("debug"));

        let app = Router::new()
            .route("/healthz", get(handler_ok))
            .route("/readyz", get(handler_ok))
            .route("/metrics", get(handler_ok))
            .route("/openapi.json", get(handler_ok))
            .route("/docs", get(handler_ok))
            .layer(trace_layer());

        let _guard = tracing::subscriber::set_default(subscriber);

        let response = app
            .clone()
            .oneshot(Request::builder().uri(*path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(
            response.status().as_u16(),
            200,
            "probe request to {} should return 200",
            path
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

        let access_log = records
            .iter()
            .find(|r| r.get("attributes").and_then(|a| a.get("target")).and_then(|t| t.as_str()) == Some("http.access"))
            .expect(&format!("should have access log for probe path {}", path));

        assert_eq!(
            access_log.get("severity").and_then(|s| s.as_str()),
            Some("DEBUG"),
            "probe path {} should emit DEBUG-level access log",
            path
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn access_log_captured_sample_line() {
    let writer = SharedWriter::default();
    let identity = ServiceIdentity::new("test-service", "1.0.0").unwrap();
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(writer.clone())
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(false)
        .json()
        .with_current_span(true)
        .event_format(ServiceJsonFormatter::new(identity));

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::filter::EnvFilter::new("debug"));

    let app = Router::new()
        .route("/test", get(handler_ok))
        .layer(trace_layer());

    let _guard = tracing::subscriber::set_default(subscriber);

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

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

    let access_log_line = records
        .iter()
        .find(|r| r.get("attributes").and_then(|a| a.get("target")).and_then(|t| t.as_str()) == Some("http.access"))
        .expect("should have access log line");

    // Print the captured JSON for reference (this will show in test output)
    let json_str = serde_json::to_string_pretty(access_log_line).unwrap();
    eprintln!("Sample access log JSON line:\n{}", json_str);

    // Verify all expected fields are present
    assert!(access_log_line.get("schema").is_some());
    assert!(access_log_line.get("severity").is_some());
    assert!(access_log_line.get("timestamp").is_some());
    assert!(access_log_line.get("service").is_some());
    assert!(access_log_line.get("attributes").is_some());
    assert!(
        access_log_line
            .get("attributes")
            .and_then(|a| a.get("target"))
            .is_some(),
        "target should be in attributes"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn access_log_filter_independent() {
    let writer = SharedWriter::default();
    let identity = ServiceIdentity::new("test-service", "1.0.0").unwrap();
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(writer.clone())
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(false)
        .json()
        .with_current_span(true)
        .event_format(ServiceJsonFormatter::new(identity));

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::filter::EnvFilter::new("info,http.access=warn"));

    let app = Router::new()
        .route("/test", get(handler_ok))
        .layer(trace_layer());

    let _guard = tracing::subscriber::set_default(subscriber);

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

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

    // Find access log line at INFO level - should NOT exist when filtered to WARN
    let access_log_info = records.iter().find(|r| {
        r.get("attributes")
            .and_then(|a| a.get("target"))
            .and_then(|t| t.as_str()) == Some("http.access")
            && r.get("severity").and_then(|s| s.as_str()) == Some("INFO")
    });

    assert!(
        access_log_info.is_none(),
        "should not have INFO-level access log when http.access filtered to WARN"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn access_log_tracks_4xx_responses() {
    let writer = SharedWriter::default();
    let identity = ServiceIdentity::new("test-service", "1.0.0").unwrap();
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(writer.clone())
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(false)
        .json()
        .with_current_span(true)
        .event_format(ServiceJsonFormatter::new(identity));

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::filter::EnvFilter::new("debug"));

    let app = Router::new()
        .route("/test", get(handler_ok))
        .layer(trace_layer());

    let _guard = tracing::subscriber::set_default(subscriber);

    // Send request to non-existent path - should get 404
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/notfound").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 404);

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

    // Find access log with 404 status
    let access_log_404 = records
        .iter()
        .find(|r| {
            r.get("attributes")
                .and_then(|a| a.get("target"))
                .and_then(|t| t.as_str()) == Some("http.access")
                && r.get("attributes")
                    .and_then(|a| a.get("status"))
                    .and_then(|s| s.as_u64())
                    == Some(404)
        })
        .expect("should have access log line with 404 status");

    assert_eq!(
        access_log_404.get("severity").and_then(|s| s.as_str()),
        Some("INFO"),
        "404 should still emit INFO-level access log"
    );
}
