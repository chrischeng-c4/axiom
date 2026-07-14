// HANDWRITE-BEGIN gap="missing-generator:unit-test:a1f57ba8" tracker="#1662" reason="Lock Tape shared tracing wiring and feature propagation."
const TAPE_BIN: &str = include_str!("../src/bin/tape.rs");
const TAPE_MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn tape_maps_optional_otlp_to_the_shared_initializer() {
    assert!(TAPE_BIN.contains("TAPE_OTLP_ENDPOINT"));
    assert!(TAPE_BIN.contains("service_http::init_tracing_with_identity"));
    assert!(TAPE_BIN.contains("ServiceIdentity::new(\"tape\""));
}

#[test]
fn tape_otel_feature_enables_shared_service_http_export() {
    assert!(TAPE_MANIFEST.contains("service-http/otlp"));
    assert!(!TAPE_BIN.contains("opentelemetry_otlp::new_pipeline"));
}
// HANDWRITE-END
