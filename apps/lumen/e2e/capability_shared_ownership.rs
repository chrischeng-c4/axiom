//! Lumen's declared capability traits and its shared-versus-domain ownership
//! split, checked against the tree that implements them.
//!
//! The inputs are `aw.toml`, `README.md`, `Cargo.toml`, and the real CLI,
//! HTTP, auth, Kubernetes, Raft, peer-transport and observability seams.
//! Declaring a trait costs nothing; this gate requires every declared trait to
//! land on a shared baseline that actually exists.
//!
//! The totality-and-disjointness case is what stops the other two from being
//! gamed. Without it a mechanism can appear in neither column, and a shared
//! failure becomes a tracked skip instead of a failure.
// HANDWRITE-BEGIN gap="missing-generator:unit-test:a96e84cf" tracker="#2324" reason="Create one deterministic structural regression gate for Lumen's trait-derived capability baseline and shared-library ownership boundary. It must inspect aw.toml, README, Cargo.toml, and the actual CLI, HTTP, auth, Kubernetes, Raft, peer transport, and observability integration seams; require canonical shared delegation; keep search/CRD policy app-owned; and encode a total, disjoint shared-versus-domain classification so shared failures cannot be tracked skips. generator gap: missing-generator:test:capability-shared-ownership (#2324)."

use std::collections::BTreeSet;

const AW_TOML: &str = include_str!("../aw.toml");
const README: &str = include_str!("../README.md");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const CLI: &str = include_str!("../src/bin/lumen.rs");
const API: &str = include_str!("../src/api.rs");
const AUTH: &str = include_str!("../src/auth.rs");
const OPERATOR_RENDER: &str = include_str!("../src/operator/render.rs");

const CAPABILITY_CONTRACT: &[(&str, &str)] = &[
    ("Indexing", "indexing"),
    ("Querying", "querying"),
    (
        "Kubernetes-native deployment",
        "kubernetes-native-deployment",
    ),
    (
        "Managed Fleet materialization",
        "managed-fleet-materialization",
    ),
    ("Security and access", "security-hardening"),
    ("Scaling and availability", "scaling-availability"),
    ("Durability and recovery", "durability-recovery"),
    ("Operations and observability", "operations-observability"),
    (
        "API, CLI, and agent integration",
        "api-cli-agent-integration",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapabilityContractError {
    MissingCapabilitiesSection,
    MissingSupportingDocumentsBoundary,
    Preamble,
    H3Headings,
    CapabilityIndexRows,
    StableId(&'static str),
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn capability_body(readme: &str) -> Result<&str, CapabilityContractError> {
    let (_, after_heading) = readme
        .split_once("## Capabilities\n")
        .ok_or(CapabilityContractError::MissingCapabilitiesSection)?;
    let next_h2 = after_heading
        .find("\n## ")
        .ok_or(CapabilityContractError::MissingSupportingDocumentsBoundary)?;
    let (body, boundary) = after_heading.split_at(next_h2);
    if !boundary.starts_with("\n## Supporting documents") {
        return Err(CapabilityContractError::MissingSupportingDocumentsBoundary);
    }
    Ok(body)
}

fn capability_sections(body: &str) -> Vec<(String, String)> {
    let mut sections = Vec::<(String, String)>::new();
    for line in body.lines() {
        if let Some(title) = line.strip_prefix("### ") {
            sections.push((title.to_string(), String::new()));
        } else if let Some((_, prose)) = sections.last_mut() {
            prose.push_str(line);
            prose.push('\n');
        }
    }
    sections
}

fn validate_capability_contract(readme: &str) -> Result<(), CapabilityContractError> {
    let body = capability_body(readme)?;
    let sections = capability_sections(body);

    let expected_headings: Vec<&str> = std::iter::once("Capability index")
        .chain(CAPABILITY_CONTRACT.iter().map(|(heading, _)| *heading))
        .collect();
    let actual_headings: Vec<&str> = sections
        .iter()
        .map(|(heading, _)| heading.as_str())
        .collect();
    if actual_headings != expected_headings {
        return Err(CapabilityContractError::H3Headings);
    }

    let preamble = body.split("### ").next().unwrap_or_default();
    if !normalize_whitespace(preamble).contains(
        "Every entry below is a Lumen product capability. The list has no primary and secondary classes.",
    ) {
        return Err(CapabilityContractError::Preamble);
    }

    let actual_index_rows: Vec<(String, String)> = sections[0]
        .1
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with('|') || line.starts_with("|---") {
                return None;
            }
            let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
            if cells.len() < 2 || cells[0] == "Capability" {
                return None;
            }
            Some((cells[0].to_string(), cells[1].to_string()))
        })
        .collect();
    let expected_index_rows: Vec<(String, String)> = CAPABILITY_CONTRACT
        .iter()
        .map(|(heading, id)| (heading.to_string(), format!("`{id}`")))
        .collect();
    if actual_index_rows != expected_index_rows {
        return Err(CapabilityContractError::CapabilityIndexRows);
    }

    for (offset, (_, id)) in CAPABILITY_CONTRACT.iter().enumerate() {
        let marker = format!("- ID: `{id}`");
        let count = sections[offset + 1]
            .1
            .lines()
            .filter(|line| *line == marker)
            .count();
        if count != 1 {
            return Err(CapabilityContractError::StableId(id));
        }
    }

    Ok(())
}

fn normalized_capability_section(readme: &str, title: &str) -> String {
    let body = capability_body(readme).expect("README must carry the capability contract");
    let sections = capability_sections(body);
    let (_, prose) = sections
        .iter()
        .find(|(heading, _)| heading == title)
        .unwrap_or_else(|| panic!("README capability section is missing {title}"));
    normalize_whitespace(prose)
}

fn remove_once(input: &str, needle: &str) -> String {
    assert_eq!(
        input.matches(needle).count(),
        1,
        "negative fixture must replace exactly one occurrence of {needle:?}"
    );
    input.replacen(needle, "", 1)
}

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

    if let Err(error) = validate_capability_contract(README) {
        panic!("README capability contract is invalid: {error:?}");
    }
}

#[test]
fn capability_contract_rejects_missing_heading_id_and_index_row() {
    assert_eq!(validate_capability_contract(README), Ok(()));

    let missing_heading = remove_once(README, "### Managed Fleet materialization\n");
    assert_eq!(
        validate_capability_contract(&missing_heading),
        Err(CapabilityContractError::H3Headings)
    );

    let missing_id = remove_once(README, "- ID: `managed-fleet-materialization`\n");
    assert_eq!(
        validate_capability_contract(&missing_id),
        Err(CapabilityContractError::StableId(
            "managed-fleet-materialization"
        ))
    );

    let index_row = README
        .lines()
        .find(|line| {
            line.starts_with("| Managed Fleet materialization | `managed-fleet-materialization` |")
        })
        .expect("README must carry the Managed Fleet capability index row");
    let missing_index_row = remove_once(README, &format!("{index_row}\n"));
    assert_eq!(
        validate_capability_contract(&missing_index_row),
        Err(CapabilityContractError::CapabilityIndexRows)
    );

    let boundary_marker = "\n### Security and access\n";
    assert_eq!(README.matches(boundary_marker).count(), 1);
    let unexpected_h2 = README.replacen(
        boundary_marker,
        "\n## Unexpected boundary\n\n### Security and access\n",
        1,
    );
    assert_eq!(
        validate_capability_contract(&unexpected_h2),
        Err(CapabilityContractError::MissingSupportingDocumentsBoundary)
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
    ] {
        assert!(
            CLI.contains(seam),
            "CLI/runtime seam must delegate through {seam}"
        );
    }

    // #2871 retired the projected registry file, so the binary no longer
    // spawns `service_auth::spawn_registry_file_watcher`. The ownership rule
    // that seam encoded is unchanged and is asserted from the other side
    // below (auth mechanics come from `service-auth`); what the binary must
    // not do is re-grow a local credential loader to replace the shared one.
    assert!(
        CLI.contains("AuthConfig::from_env"),
        "the binary must build its auth config through lumen's service-auth adapter"
    );
    for regrown in [
        "fn watch_token_registry",
        "notify::",
        "spawn_registry_file_watcher",
    ] {
        assert!(
            !CLI.contains(regrown),
            "credential-file watching is not lumen's to own again: {regrown}"
        );
    }

    for seam in [
        "service_http::AdmissionController",
        "service_http::standard_probe_routes_canonical_json",
        "service_http::trace_layer",
    ] {
        assert!(API.contains(seam), "HTTP seam must delegate through {seam}");
    }

    // The auth adapter's shape changed again with #2869 — the registry it
    // wrapped is gone and the decision is delegated to kube-apiserver — but
    // its ownership did not: the TokenReview/SubjectAccessReview mechanics,
    // the principal parsing, and the middleware are all `service-auth`'s, and
    // lumen holds only the resource mapping around them. A lumen-local review
    // client, principal parser, or verifier would be the regression.
    for seam in [
        "service_auth::k8s::",
        "DelegatedAuthenticator",
        "service_auth::async_auth_middleware::<LumenVerifier>",
    ] {
        assert!(
            AUTH.contains(seam),
            "auth seam must delegate through {seam}"
        );
    }
    for regrown in ["struct KubeReviewBackend", "fn parse_service_account"] {
        assert!(
            !AUTH.contains(regrown),
            "delegated-auth mechanics are not lumen's to own: {regrown}"
        );
    }
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
    // in this array. Check the app source inside each relevant capability so
    // unrelated prose cannot satisfy the ownership boundary.
    for (title, app_claim) in [
        (
            "Indexing",
            "- [`apps/lumen`](./) defines field behavior, analyzers, mutation rules, segment formats, and the derived-index lifecycle.",
        ),
        (
            "Querying",
            "- [`apps/lumen`](./) defines query validation, planning, scoring, filtering, grouping, sorting, pagination, and read-consistency behavior.",
        ),
        (
            "Kubernetes-native deployment",
            "- [`apps/lumen`](./) defines the CRD, defaults, topology policy, conditions, and Lumen resource composition.",
        ),
        (
            "Scaling and availability",
            "- [`apps/lumen`](./) defines virtual-bucket routing, shard ownership, reshard phases, write fences, checkpoints, and scatter/gather behavior.",
        ),
        (
            "Security and access",
            "- [`apps/lumen`](./) defines the security posture, Lumen permission mapping, anonymous route set, identity separation, and integration policy.",
        ),
    ] {
        let section = normalized_capability_section(README, title);
        assert!(
            section.contains(app_claim),
            "{title} must keep its app-owned policy source: {app_claim}"
        );
    }
}

// HANDWRITE-END
