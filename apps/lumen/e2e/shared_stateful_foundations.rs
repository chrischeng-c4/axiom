//! Ownership boundary: lumen delegates trace initialization and peer transport
//! to the shared service libraries, and keeps no local duplicate of either.
//!
//! Asserted against the text of `src/bin/lumen.rs`, which this file embeds.
//! That is deliberate. A duplicate tracer is not a behavioural failure — both
//! pipelines emit spans and every runtime assertion stays green — so the only
//! observation point that can see it is the wiring itself.
//!
//! The third case makes the classification total: every completed shared root
//! must resolve to a lumen-owned adapter or a runtime projection. Without it a
//! root can be absent from both lists while the first two cases still pass.
//!
//! ## Contracts inherited from the retired EC shells
//!
//! This sentence was the whole of the `// Contract:` comment in an AW-EC shell under
//! `apps/lumen/e2e/`, which ran `cargo test -p lumen --test
//! shared_stateful_foundations` in a subprocess and asserted the child's exit status.
//! `cargo test -p lumen` already runs this target directly, so the shell added a
//! second, nested run and nothing else. It was deleted on 2026-08-20 with the EC
//! machinery it belonged to, and the sentence is the only thing it held that nothing
//! else did. The line below is prefixed with the EC id the shell was filed under.
//!
//! - `lumen-claim-long-running-shared-foundations` — Lumen delegates tracing, peer
//!   transport, and completed shared stateful roots to the reusable service libraries.
// HANDWRITE-BEGIN gap="missing-generator:unit-test:c90bbb42" tracker="#1646" reason="Lock Lumen's ownership boundary: shared OTLP tracing and shared reloadable peer transport, with no local duplicate tracer. generator gap: missing-generator:lumen-foundation-ownership-test (#1646)."
const LUMEN_BIN: &str = include_str!("../src/bin/lumen.rs");
const LUMEN_MANIFEST: &str = include_str!("../Cargo.toml");
const LUMEN_AUTH: &str = include_str!("../src/auth.rs");
const LUMEN_API: &str = include_str!("../src/api.rs");
const LUMEN_OPERATOR: &str = include_str!("../src/operator/render.rs");
const LUMEN_RIG_ADAPTER: &str = include_str!("rig_stateful_adapter.rs");

#[test]
fn lumen_delegates_trace_initialization_without_a_local_trace_pipeline() {
    assert!(LUMEN_BIN.contains("service_http::init_tracing_with_identity"));
    assert!(LUMEN_BIN.contains("ServiceIdentity::new(\"lumen\""));
    assert!(LUMEN_BIN.contains("fn init_otel_meter"));
    assert!(!LUMEN_BIN.contains("fn build_otel_tracer"));
    assert!(!LUMEN_BIN.contains("tracing_opentelemetry::"));
    assert!(LUMEN_MANIFEST.contains("service-http/otlp"));
    assert!(!LUMEN_MANIFEST.contains("tracing-opentelemetry"));
}

#[test]
fn configured_peer_identity_uses_shared_https_transport_and_dedicated_listener() {
    for required in [
        "PeerTlsConfig::from_env()",
        "ClusterTopology::from_env_with_scheme",
        "RaftHost::spawn_with_peer_transport",
        ".serve(peer_listener, peer_router",
        "raft_peer_transport.is_none()",
    ] {
        assert!(
            LUMEN_BIN.contains(required),
            "lumen serve is missing shared peer-transport wiring: {required}"
        );
    }
    // #2890 R3: the peer scheme stopped being a choice. This used to assert
    // both arms of `if peer_transport.is_some() { (args.raft_port, "https") }
    // else { (args.port, "http") }` — the second arm moved replicated Raft
    // traffic onto the *client* port over h2c whenever TLS material was
    // absent. It is gone, so what is locked here now is its absence plus the
    // fail-closed message that replaced it.
    assert!(LUMEN_BIN.contains("\"https\","));
    assert!(
        !LUMEN_BIN.contains("(args.port, \"http\")"),
        "the plaintext peer fallback must not come back"
    );
    assert!(
        LUMEN_BIN.contains("Raft peer traffic has no plaintext path"),
        "a replicated group with no peer material must refuse to start, \
         not pick a scheme"
    );
}

#[test]
fn completed_shared_roots_have_lumen_owned_adapters_or_runtime_projection() {
    // #2871 retired the reloadable registry and #2869 delegated the decision
    // to kube-apiserver, but the ownership boundary this test exists to lock
    // is unchanged: the verifier mechanics still come from `service-auth`, and
    // lumen keeps only the resource mapping around them.
    assert!(LUMEN_AUTH.contains("service_auth::"));
    assert!(LUMEN_AUTH.contains("service_auth::k8s::"));
    assert!(!LUMEN_AUTH.contains("ReloadableRoleMapVerifier"));
    assert!(!LUMEN_AUTH.contains("StaticRoleMapVerifier"));
    assert!(LUMEN_API.contains("service_http::AdmissionController"));
    assert!(LUMEN_OPERATOR.contains("service_statefulset"));
    assert!(LUMEN_OPERATOR.contains("headless_service_with_ports"));
    assert!(LUMEN_OPERATOR.contains("LUMEN_RAFT_PORT"));
    assert!(LUMEN_RIG_ADAPTER.contains("run_stateful"));
}
// HANDWRITE-END
