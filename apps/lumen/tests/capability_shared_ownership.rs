// HANDWRITE-BEGIN gap="missing-generator:unit-test:a96e84cf" tracker="#2324" reason="Create one deterministic structural regression gate for Lumen's trait-derived capability baseline and shared-library ownership boundary. It must inspect aw.toml, README, Cargo.toml, and the actual CLI, HTTP, auth, Kubernetes, Raft, peer transport, and observability integration seams; require canonical shared delegation; keep search/CRD policy app-owned; and encode a total, disjoint shared-versus-domain classification so shared failures cannot be tracked skips. generator gap: missing-generator:test:capability-shared-ownership (#2324)."
// @spec apps/lumen/tech-design/interfaces/rest/verify-capability-contracts-and-shared-ownership.md#unit-test

use std::collections::BTreeSet;

const AW_TOML: &str = include_str!("../aw.toml");
const README: &str = include_str!("../README.md");
/// The canonical capability contract. It used to be README — capability
/// headings, claims, and verification all lived there. #2887 moved the contract
/// here and left README a human-readable summary that links to it, so a gate
/// that still read README would be asserting on prose whose job is now to be
/// readable rather than to be exact.
const CAPABILITIES: &str = include_str!("../CAPABILITIES.md");
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

    // Each trait's baseline still has to land somewhere nameable. Under the
    // two-root contract that place is a capability heading plus its stable ID
    // — the ID is what claims and gates reference, so a heading renamed
    // without its ID is a doc edit, and an ID that disappears is a broken
    // contract.
    for (heading, id) in [
        ("#### Indexing", "`indexing`"),
        ("#### Querying", "`querying`"),
        ("#### Kubernetes-Native Deployment", "`kubernetes-native-deployment`"),
        ("#### Security & Access", "`security-hardening`"),
        ("#### Scaling & Availability", "`scaling-availability`"),
        ("#### Durability & Recovery", "`durability-recovery`"),
        ("#### Operations & Observability", "`operations-observability`"),
        ("#### API, CLI & Agent Integration", "`api-cli-agent-integration`"),
    ] {
        assert!(
            CAPABILITIES.contains(heading),
            "trait-derived baseline is missing {heading}"
        );
        assert!(
            CAPABILITIES.contains(&format!("ID: {id}")),
            "{heading} must keep its stable capability ID {id}"
        );
        assert!(
            CAPABILITIES.contains(&format!("| {id} |")),
            "{heading} must stay listed in the Capability Index as {id}"
        );
    }

    // The two roots are the whole taxonomy; a third root would put a capability
    // somewhere neither `aw capability` nor a reader looks.
    for root in ["### Core Features", "### Non-Core Features"] {
        assert!(CAPABILITIES.contains(root), "capability root {root} is missing");
    }

    assert!(
        README.contains("[`CAPABILITIES.md`](CAPABILITIES.md)"),
        "README must point at the canonical contract rather than restate it"
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
        assert!(
            CLI.contains(seam),
            "CLI/runtime seam must delegate through {seam}"
        );
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

    assert_eq!(
        all.len(),
        OWNERSHIP.len(),
        "every concern must be classified exactly once"
    );
    assert!(
        shared.is_disjoint(&domain),
        "shared failures cannot be reclassified as domain skips"
    );
    assert_eq!(
        shared.len() + domain.len(),
        all.len(),
        "ownership classification must be total"
    );

    // The domain half of the table has to be visible in the contract, not just
    // in this array: search planning and shard/CRD policy are the two places a
    // reviewer would otherwise be tempted to push into a shared library.
    assert!(CAPABILITIES.contains("#### Querying"));
    assert!(CAPABILITIES.contains("#### Scaling & Availability"));
    assert!(
        CAPABILITIES.contains("Lumen owns its CRD\npolicy and app wiring"),
        "the Kubernetes capability must state that CRD policy stays app-owned"
    );
    assert!(
        CAPABILITIES.contains("Lumen owns domain policy and wiring"),
        "the security capability must state that only mechanics move to shared libraries"
    );
}

// HANDWRITE-END
