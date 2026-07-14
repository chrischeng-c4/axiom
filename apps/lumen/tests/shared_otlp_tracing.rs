// HANDWRITE-BEGIN gap="missing-generator:unit-test:acef149f" tracker="pending-tracker" reason="Verify Lumen uses the shared OTLP trace initializer and does not own a duplicate tracer constructor."
// HANDWRITE-BEGIN gap="missing-generator:unit-test:acef149f" tracker="#1661" reason="Verify Lumen uses the shared OTLP trace initializer and does not own a duplicate tracer constructor."
const LUMEN_BIN: &str = include_str!("../src/bin/lumen.rs");
const LUMEN_MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn lumen_delegates_trace_initialization_to_service_http() {
    assert!(LUMEN_BIN.contains("service_http::init_tracing_with_identity"));
    assert!(LUMEN_BIN.contains("ServiceIdentity::new(\"lumen\""));
    assert!(LUMEN_BIN.contains("fn init_otel_meter"));
}

#[test]
fn lumen_otel_feature_uses_shared_trace_capability_without_a_local_pipeline() {
    assert!(LUMEN_MANIFEST.contains("service-http/otlp"));
    assert!(!LUMEN_MANIFEST.contains("tracing-opentelemetry"));
    assert!(!LUMEN_BIN.contains("fn build_otel_tracer"));
    assert!(!LUMEN_BIN.contains("tracing_opentelemetry::"));
}
// HANDWRITE-END
// HANDWRITE-END
