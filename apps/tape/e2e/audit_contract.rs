// HANDWRITE-BEGIN gap="missing-generator:unit-test:service-audit-boundary" tracker="#1641" reason="Locks Tape's low-frequency management audit boundary together with the shared redacted service-auth audit sink."
//! Audit-boundary regression contract for the Tape service.

const SERVER: &str = include_str!("../src/server.rs");
const AUTH: &str = include_str!("../src/auth.rs");

#[test]
fn backup_audit_is_redacted_and_kept_off_hot_data_plane_routes() {
    assert!(SERVER.contains("target: \"tape.audit\""));
    assert!(SERVER.contains("event = \"backup_snapshot_served\""));
    assert!(SERVER.contains("subject = principal.subject().unwrap_or(\"anonymous\")"));
    assert!(SERVER.contains("applied_index = applied"));
    assert!(SERVER.contains("bytes = bytes.len()"));

    // The service adapter delegates token/authorization audit fields to the
    // shared verifier; it does not parse or log bearer credentials itself.
    assert!(AUTH.contains("TracingAuthEventSink"));
    assert!(!AUTH.contains("tracing::"));
}
// HANDWRITE-END
