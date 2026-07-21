// HANDWRITE-BEGIN gap="missing-generator:unit-test:a96e84cf" tracker="pending-tracker" reason="Create one deterministic structural regression gate for Lumen's trait-derived capability baseline and shared-library ownership boundary. It must inspect aw.toml, README, Cargo.toml, and the actual CLI, HTTP, auth, Kubernetes, Raft, peer transport, and observability integration seams; require canonical shared delegation; keep search/CRD policy app-owned; and encode a total, disjoint shared-versus-domain classification so shared failures cannot be tracked skips. generator gap: missing-generator:test:capability-shared-ownership (#2324)."
// @spec apps/lumen/tech-design/interfaces/rest/verify-capability-contracts-and-shared-ownership.md#unit-test

use std::collections::BTreeSet;

const AW_TOML: &str = include_str!("../aw.toml");
const README: &str = include_str!("../README.md");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const CLI: &str = include_str!("../src/bin/lumen.rs");
const API: &str = include_str!("../src/api.rs");
const AUTH: &str = include_str!("../src/auth.rs");
const OPERATOR_RENDER: &str = include_str!("../src/operator/render.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Owner {
    Shared,
    Domain,
}

const OWNERSHIP: &[(&str, Owner)] = &[
    ("cli", Owner::Shared),
    ("http", Owner::Shared),
    ("auth", Owner::Shared),
    ("observability", Owner::Shared),
    ("kubernetes-render", Owner::Shared),
    ("raft-host", Owner::Shared),
    ("peer-identity", Owner::Shared),
    ("search-planner", Owner::Domain),
    ("index-storage-policy", Owner::Domain),
    ("lumen-crd-reshard-policy", Owner::Domain),
];

#[test]
fn trait_profile_requires_shared_service_baselines() {
    for trait_name in [
        "service",
        "long_running",
        "cli_facing",
        "competitive_replacement",
        "network_exposed",
        "stateful_storage",
        "agent_facing",
    ] {
        assert!(
            AW_TOML.contains(&format!("\"{trait_name}\"")),
            "Lumen's capability profile must retain the {trait_name:?} trait"
        );
    }

    for capability in [
        "### CLI Interface",
        "### Long-Running Stability",
        "### Security Hardening",
        "### Standard Operational Endpoints",
        "### Observability",
        "### Kubernetes-Native Deployment",
        "### Stateful Service Workload",
        "### Developer & Agent Experience",
    ] {
        assert!(
            README.contains(capability),
            "trait-derived baseline is missing {capability}"
        );
    }

    assert!(
        README.contains("without a duplicate\nservice implementation"),
        "the stateful baseline must explicitly reject an app-local duplicate service stack"
    );
}

#[test]
fn platform_mechanisms_delegate_to_shared_owners() {
    for dependency in [
        "cli-std",
        "service-auth",
        "service-http",
        "raft-runtime",
        "service-k8s",
        "peer-tls",
    ] {
        assert!(
            CARGO_TOML.contains(dependency),
            "Lumen must delegate platform mechanism {dependency:?} to its shared crate"
        );
    }

    for seam in [
        "cli_std::llm::",
        "cli_std::upgrade::run",
        "cli_std::issue::search",
        "service_http::init_tracing_with_identity",
        "raft_runtime::RaftHost",
        "RaftHost::spawn_with_peer_transport",
        "service_auth::spawn_registry_file_watcher",
    ] {
        assert!(CLI.contains(seam), "CLI/runtime seam must delegate through {seam}");
    }

    for seam in [
        "service_http::AdmissionController",
        "service_http::standard_probe_routes_canonical_json",
        "service_http::trace_layer",
    ] {
        assert!(API.contains(seam), "HTTP seam must delegate through {seam}");
    }

    assert!(AUTH.contains("Thin lumen adapter over `service_auth::role_map`"));
    assert!(AUTH.contains("service_auth::auth_middleware::<LumenVerifier>"));
    assert!(OPERATOR_RENDER.contains("use service_k8s::render"));
    assert!(OPERATOR_RENDER.contains("render::service_statefulset"));

    assert!(
        !CARGO_TOML.contains("tracing-opentelemetry"),
        "Lumen must not restore an app-local tracing exporter dependency"
    );
    assert!(
        !CLI.contains("fn build_otel_tracer"),
        "OTLP trace construction belongs to service-http, not the Lumen binary"
    );
}

#[test]
fn shared_and_domain_ownership_are_total_and_disjoint() {
    let all: BTreeSet<_> = OWNERSHIP.iter().map(|(concern, _)| *concern).collect();
    let shared: BTreeSet<_> = OWNERSHIP
        .iter()
        .filter_map(|(concern, owner)| (*owner == Owner::Shared).then_some(*concern))
        .collect();
    let domain: BTreeSet<_> = OWNERSHIP
        .iter()
        .filter_map(|(concern, owner)| (*owner == Owner::Domain).then_some(*concern))
        .collect();

    assert_eq!(all.len(), OWNERSHIP.len(), "every concern must be classified exactly once");
    assert!(shared.is_disjoint(&domain), "shared failures cannot be reclassified as domain skips");
    assert_eq!(shared.len() + domain.len(), all.len(), "ownership classification must be total");

    assert!(README.contains("### Search Core"));
    assert!(README.contains("### Dynamic Shard Topology"));
    assert!(
        README.contains("operator watches one namespace and owns\nstorage topology, reshard phases"),
        "Lumen-specific search and CRD reshard policy must remain app-owned"
    );
}

<!-- marker: missing-generator:unit-test:a96e84cf path: apps/lumen/tests/capability_shared_ownership.rs reason: Create one deterministic structural regression gate for Lumen's trait-derived capability baseline and shared-library ownership boundary. It must inspect aw.toml, README, Cargo.toml, and the actual CLI, HTTP, auth, Kubernetes, Raft, peer transport, and observability integration seams; require canonical shared delegation; keep search/CRD policy app-owned; and encode a total, disjoint shared-versus-domain classification so shared failures cannot be tracked skips. generator gap: missing-generator:test:capability-shared-ownership (#2324). -->
// HANDWRITE-END
