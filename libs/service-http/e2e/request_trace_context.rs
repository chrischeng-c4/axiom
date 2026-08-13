// HANDWRITE-BEGIN gap="missing-generator:unit-test:d2043dd3" tracker="1870" reason="Prove valid parent preservation, local id generation, malformed and zero-id fallback, request routing, and no-OTLP behavior."
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use service_http::trace_layer;
use tower::ServiceExt as _;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Record};
use tracing::subscriber::Interest;
use tracing::{Event, Id, Metadata, Subscriber};

#[derive(Clone, Default)]
struct SpanCapture {
    fields: Arc<Mutex<HashMap<u64, HashMap<String, String>>>>,
    next_id: Arc<AtomicU64>,
}

impl SpanCapture {
    fn request_fields(&self) -> HashMap<String, String> {
        self.fields
            .lock()
            .expect("capture lock")
            .values()
            .find(|fields| {
                fields
                    .get("span.name")
                    .is_some_and(|name| name == "request")
            })
            .cloned()
            .expect("request span fields")
    }
}

impl Subscriber for SpanCapture {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, attrs: &Attributes<'_>) -> Id {
        let id = Id::from_u64(self.next_id.fetch_add(1, Ordering::Relaxed) + 1);
        let mut values =
            HashMap::from([("span.name".to_string(), attrs.metadata().name().to_string())]);
        attrs.record(&mut FieldVisitor(&mut values));
        self.fields
            .lock()
            .expect("capture lock")
            .insert(id.into_u64(), values);
        id
    }

    fn record(&self, id: &Id, values: &Record<'_>) {
        let mut spans = self.fields.lock().expect("capture lock");
        let fields = spans.entry(id.into_u64()).or_default();
        values.record(&mut FieldVisitor(fields));
    }

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, _event: &Event<'_>) {}

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}

    fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
        Interest::always()
    }
}

struct FieldVisitor<'a>(&'a mut HashMap<String, String>);

impl Visit for FieldVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{value:?}"));
    }
}

async fn request_fields(traceparent: Option<&str>) -> (StatusCode, HashMap<String, String>) {
    let capture = SpanCapture::default();
    let _subscriber = tracing::subscriber::set_default(capture.clone());

    let app = Router::new()
        .route("/test", get(|| async { StatusCode::NO_CONTENT }))
        .layer(trace_layer());
    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    if let Some(value) = traceparent {
        request
            .headers_mut()
            .insert("traceparent", value.parse().unwrap());
    }
    let response = app.oneshot(request).await.unwrap();
    (response.status(), capture.request_fields())
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
async fn valid_traceparent_preserves_trace_and_creates_child_span() {
    let (_, fields) = request_fields(Some(
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
    ))
    .await;

    assert_eq!(fields["trace_id"], "4bf92f3577b34da6a3ce929d0e0e4736");
    assert_eq!(fields["parent_span_id"], "00f067aa0ba902b7");
    assert_eq!(fields["trace_flags"], "01");
    assert_nonzero_lower_hex(&fields["span_id"], 16);
    assert_ne!(fields["span_id"], fields["parent_span_id"]);
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_or_missing_traceparent_creates_safe_root() {
    let invalid = [
        None,
        Some("00-00000000000000000000000000000000-00f067aa0ba902b7-01"),
        Some("00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01"),
        Some("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
        Some("00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01"),
        Some("not-a-traceparent"),
    ];

    for value in invalid {
        let (status, fields) = request_fields(value).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_nonzero_lower_hex(&fields["trace_id"], 32);
        assert_nonzero_lower_hex(&fields["span_id"], 16);
        assert!(!fields.contains_key("parent_span_id"));
        assert_eq!(fields["trace_flags"], "00");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn trace_layer_records_context_and_routes_without_otlp() {
    let (status, fields) = request_fields(Some(
        "00-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bbbbbbbbbbbbbbbb-00",
    ))
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(fields["trace_id"], "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(fields["parent_span_id"], "bbbbbbbbbbbbbbbb");
    assert_nonzero_lower_hex(&fields["span_id"], 16);
}

#[cfg(feature = "otlp")]
#[tokio::test(flavor = "current_thread")]
async fn otlp_feature_preserves_request_trace_identity() {
    let (_, fields) = request_fields(Some(
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
    ))
    .await;

    assert_eq!(fields["trace_id"], "4bf92f3577b34da6a3ce929d0e0e4736");
    assert_eq!(fields["parent_span_id"], "00f067aa0ba902b7");
    assert_nonzero_lower_hex(&fields["span_id"], 16);
}
// HANDWRITE-END
