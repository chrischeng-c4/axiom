// HANDWRITE-BEGIN gap="missing-generator:unit-test:5f79531a" tracker="#1640" reason="Cover plain startup, identity stability, exporter construction fallback, and W3C parent propagation with local deterministic fixtures."
use service_http::{tracing_mode, HttpConfig, LogFormat, ServiceIdentity, TracingMode};

#[cfg(feature = "otlp")]
use axum::http::{HeaderMap, HeaderValue};
#[cfg(feature = "otlp")]
use service_http::{extract_trace_context, OtelFallback};

fn config(endpoint: Option<&str>) -> HttpConfig {
    HttpConfig::new(
        "127.0.0.1",
        7137,
        "info",
        LogFormat::Json,
        15,
        1024,
        endpoint.map(str::to_owned),
    )
}

#[test]
fn logging_only_default_requires_no_exporter() {
    let identity = ServiceIdentity::new("service-http-test", "0.1.0").unwrap();
    assert_eq!(
        tracing_mode(&config(None), &identity),
        TracingMode::LoggingOnly
    );
}

#[test]
fn compatibility_surface_delegates_to_protocol_neutral_owner() {
    let config = config(Some("not-an-endpoint"));
    let identity = ServiceIdentity::new("service-http-test", "0.1.0").unwrap();
    assert_eq!(
        tracing_mode(&config, &identity),
        service_observability::tracing_mode(&config.observability_config(), &identity)
    );
}

#[cfg(feature = "otlp")]
#[test]
fn otlp_identity_contract_is_stable() {
    let identity = ServiceIdentity::new("tape", "0.4.5").unwrap();
    let mode = tracing_mode(&config(Some("http://collector:4317")), &identity);
    assert_eq!(
        mode,
        TracingMode::Otel {
            endpoint: "http://collector:4317".to_string(),
            identity,
        }
    );
}

#[cfg(feature = "otlp")]
#[test]
fn exporter_setup_failure_keeps_logging_available() {
    let identity = ServiceIdentity::new("tape", "0.4.5").unwrap();
    assert_eq!(
        tracing_mode(&config(Some("mailto:collector")), &identity),
        TracingMode::OtelUnavailable {
            endpoint: "mailto:collector".to_string(),
            reason: OtelFallback::InvalidEndpoint,
        }
    );
}

#[cfg(feature = "otlp")]
#[test]
fn trace_layer_propagates_w3c_parent_context() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "traceparent",
        HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
    );
    let context = extract_trace_context(&headers);
    use opentelemetry::trace::TraceContextExt as _;
    assert!(context.span().span_context().is_valid());
    assert_eq!(
        context.span().span_context().trace_id().to_string(),
        "4bf92f3577b34da6a3ce929d0e0e4736"
    );
}
// HANDWRITE-END
