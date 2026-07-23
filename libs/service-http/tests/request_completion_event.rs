// HANDWRITE-BEGIN gap="missing-generator:unit-test:c501f0e9" tracker="#2420" reason="scaffold for libs/service-http/tests/request_completion_event.rs — fill in by hand and update tracker when codegen is ready"
use std::io;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use service_http::trace_layer;
use service_observability::{
    ServiceIdentity, ServiceJsonFormatter, ServiceLogEventV1, SERVICE_LOG_SCHEMA_V1,
};
use tower::ServiceExt as _;
use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;

const VALID_TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
const INVALID_TRACEPARENT: &str = "malformed-traceparent";

#[derive(Clone, Default)]
struct SharedWriter {
    bytes: Arc<Mutex<Vec<u8>>>,
}

struct SharedWriterGuard {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl io::Write for SharedWriterGuard {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.bytes
            .lock()
            .expect("writer lock")
            .extend_from_slice(buffer);
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
    fn records(&self) -> Vec<ServiceLogEventV1> {
        let output = String::from_utf8(self.bytes.lock().expect("writer lock").clone())
            .expect("JSONL is UTF-8");
        output
            .lines()
            .map(|line| serde_json::from_str(line).expect("schema-valid JSONL record"))
            .collect()
    }
}

async fn request_records(traceparent: Option<&str>) -> (StatusCode, Vec<ServiceLogEventV1>) {
    let writer = SharedWriter::default();
    let identity = ServiceIdentity::new("fixture-service", "1.2.3").expect("service identity");
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .fmt_fields(JsonFields::new())
        .event_format(ServiceJsonFormatter::new(identity))
        .with_writer(writer.clone());
    let subscriber = tracing_subscriber::registry().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = Router::new()
        .route("/test", get(|| async { StatusCode::NO_CONTENT }))
        .layer(trace_layer());
    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    if let Some(value) = traceparent {
        request
            .headers_mut()
            .insert("traceparent", value.parse().expect("header value"));
    }
    let status = app.oneshot(request).await.expect("response").status();
    drop(_guard);
    (status, writer.records())
}

fn completion(records: &[ServiceLogEventV1]) -> &ServiceLogEventV1 {
    let matches = records
        .iter()
        .filter(|record| record.event == "http_request_complete")
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "expected exactly one completion record");
    matches[0]
}

fn assert_nonzero_lower_hex(value: &str, len: usize) {
    assert_eq!(value.len(), len, "unexpected id length: {value}");
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "id is not lowercase hex: {value}"
    );
    assert!(value.bytes().any(|byte| byte != b'0'), "id must be nonzero");
}

#[tokio::test(flavor = "current_thread")]
async fn completion_record_is_schema_valid_and_complete() {
    let (status, records) = request_records(Some(VALID_TRACEPARENT)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let record = completion(&records);
    assert_eq!(record.schema, SERVICE_LOG_SCHEMA_V1);
    assert_eq!(record.severity, "INFO");
    assert_eq!(record.service.name, "fixture-service");
    assert_eq!(record.event, "http_request_complete");
    assert_eq!(record.attributes["method"], "GET");
    assert_eq!(record.attributes["uri"], "/test");
    assert_eq!(record.attributes["status"], 204_u64);
    assert!(record.attributes["latency_ms"]
        .as_f64()
        .is_some_and(|latency| latency >= 0.0));
}

#[tokio::test(flavor = "current_thread")]
async fn valid_w3c_parent_is_preserved_on_completion_record() {
    let (_, records) = request_records(Some(VALID_TRACEPARENT)).await;
    let record = completion(&records);

    assert_eq!(
        record.trace_id.as_deref(),
        Some("4bf92f3577b34da6a3ce929d0e0e4736")
    );
    assert_eq!(record.parent_span_id.as_deref(), Some("00f067aa0ba902b7"));
    assert_eq!(record.trace_flags.as_deref(), Some("01"));
    let span_id = record.span_id.as_deref().expect("local span id");
    assert_nonzero_lower_hex(span_id, 16);
    assert_ne!(span_id, "00f067aa0ba902b7");
}

#[tokio::test(flavor = "current_thread")]
async fn missing_or_malformed_parent_falls_back_without_losing_completion_event() {
    for traceparent in [None, Some(INVALID_TRACEPARENT)] {
        let (_, records) = request_records(traceparent).await;
        let record = completion(&records);
        assert_nonzero_lower_hex(record.trace_id.as_deref().expect("trace id"), 32);
        assert_nonzero_lower_hex(record.span_id.as_deref().expect("span id"), 16);
        assert_eq!(record.parent_span_id, None);
        assert_eq!(record.trace_flags.as_deref(), Some("00"));
    }
}

#[test]
fn trace_layer_has_no_collector_configuration_surface() {
    let _layer = trace_layer();
}
// HANDWRITE-END
