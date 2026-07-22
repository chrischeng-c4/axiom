// HANDWRITE-BEGIN gap="missing-generator:logic:252eb58d" tracker="pending-tracker" reason="New module implementing R1 (mandatory shared-service-kit adoption review) and R2 (profile-specific negative assertions) as `pub(crate) fn apply_conformance_rules(project_dir: &Path, resolution: &review::ProfileResolution) -> Vec<Finding>`. Types: - `pub(crate) struct Finding { id: String, severity: FindingSeverity, summary: String, affected_paths: Vec<String>, remediation: String }` (mirrors `apps/agentic-workflow/src/cli/td_check_section_type.rs::Finding` shape/field-naming conventions and reuses `crate::validate::Severity`-style ordering, but is its own review-domain type -- never a re-export of the td-check Finding). - `pub(crate) enum FindingSeverity { High, Medium, Low }` (serde-serializable, lowercase). R1 rule table (const array of rows), one row per libs/* kit crate: server-tcp/server-http/ transport-h2c/service-http (served-surface setup: direct TcpListener::bind/hyper::Server::bind source markers without the corresponding dependency), server-lifecycle (hand-rolled retry/backoff loop shape without the dependency), service-observability (hand-rolled `mod logging`/metrics-registry setup without the dependency), raft-core/raft-runtime (raft-shaped leader-ingest source markers without the dependency -- this row additionally requires raft-shaped markers via reused `review::scan_source_markers` output before it fires, and every row is skipped outright when `resolution.profile.kind_surface == review::KindSurface::Cli`). Each row is evaluated by reusing `review::read_cargo_dependencies(project_dir)` for the dependency check and `review::scan_source_markers(project_dir)` plus a small local substring/regex marker scan (`TcpListener::bind`, `hyper::Server::bind`, literal `/healthz`/`/health` route strings, `mod logging`) for the source-marker check. A finding fires only when the marker is present AND the owning dependency is absent. R2 negative-assertion rules, one function per #2165 reference profile shape, each taking `&review::ProfileResolution` plus fresh evidence and returning `Option<Finding>`: - `pgpool_negative_assertion` (Deployment/ExternalState): flags StatefulSet/PVC/headless-service k8s-manifest content or a raft-core/raft-runtime Cargo dependency. - `tape_negative_assertion` (StatefulSet/ReplicatedLog): flags a raft dependency or primary/replica-role source markers (via `review::scan_source_markers().primary_replica_role`). - `relay_defer_negative_assertion` (StatefulSet/RaftConsensus): flags the `primary_replicas` capability.profile trait or primary/replica-role source markers. - `lumen_negative_assertion` (StatefulSet/PrimaryReplica): flags a raft dependency combined with leader-ingest source markers (`review::scan_source_markers().leader_ingest`) absent the project's own primary-role marker. A private `scan_k8s_manifests(project_dir: &Path) -> KitManifestMarkers { has_pvc, has_headless_service, has_statefulset_kind }` helper reads `k8s/`, `deploy/`, or `kubernetes/` subdirectories (mirroring `review::has_dockerfile_or_manifest`'s directory-probing style) for the literals `PersistentVolumeClaim`/`volumeClaimTemplates`/`clusterIP: None`/`kind: StatefulSet`; this is the one new evidence source this WI adds beyond the #2165 evidence set. `apply_conformance_rules` dispatches R2 by `resolution.profile.primary_workload`/`state_ownership` (Unknown/Ambiguous profiles produce no R2 finding) and always runs R1 first, then R2, concatenating results into one `Vec<Finding>`. Every `Finding.id` is deterministic and rule-scoped (e.g. `'shared-kit:server-http'`, `'negative-assertion:pgpool:raft-dependency'`) so the same violation on the same project reproduces the same id across runs. Includes a `#[cfg(test)] mod tests` with one `#[test]` fn per Unit Test requirement id below, using `tempfile::TempDir`-backed fixture project directories (matching `review.rs`'s existing `#[cfg(test)]` fixture style) to construct minimal Cargo.toml/aw.toml/src/k8s content per scenario. gap: shared-service-kit-conformance-rule-heuristics tracker: '#2166'"
//! R1 (mandatory shared-service-kit adoption review) + R2 (profile-specific
//! negative assertions), keyed off the #2165-resolved `review::ProjectProfile`.
//! Additive: never mutates `ProjectProfile`/`ProfileResolution`/`ProfileEvidence`,
//! only reads them plus one new evidence source (a k8s-manifest content scan)
//! to produce `Finding`s appended to `aw review`'s existing envelope.
//!
//! @spec apps/agentic-workflow/tech-design/config/aw-review-shared-service-kit-adoption-rules-profile-negative-ass.md#logic

use std::path::Path;

use serde::Serialize;

use crate::cli::review::{
    self, KindSurface, PrimaryWorkload, ProfileResolution, ProjectProfile, ReplicationConsensus,
    StateOwnership,
};

// ---------------------------------------------------------------------
// Finding shape (R3/R4: stable id, severity, evidence trail, executable
// remediation -- mirrors `td_check_section_type::Finding`'s field-naming
// conventions but is its own review-domain type, never a re-export).
// ---------------------------------------------------------------------

/// Severity of a shared-service-kit adoption or profile negative-assertion
/// finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    High,
    Medium,
    Low,
}

/// One shared-service-kit adoption (R1) or profile negative-assertion (R2)
/// finding.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Stable, rule-scoped id (e.g. `"shared-kit:server-tcp"`,
    /// `"negative-assertion:pgpool:statefulset-shape"`). The same
    /// violation on the same project always reproduces the same id.
    pub id: String,
    pub severity: FindingSeverity,
    pub summary: String,
    /// Evidence-trail paths (Cargo.toml/aw.toml/source-file/manifest-file)
    /// backing this finding. Never empty.
    pub affected_paths: Vec<String>,
    /// Always names either an owning `libs/*` crate or a concrete
    /// structural fix -- never a bare "needs review" placeholder.
    pub remediation: String,
}

fn finding(
    id: impl Into<String>,
    severity: FindingSeverity,
    summary: impl Into<String>,
    affected_paths: Vec<String>,
    remediation: impl Into<String>,
) -> Finding {
    Finding {
        id: id.into(),
        severity,
        summary: summary.into(),
        affected_paths,
        remediation: remediation.into(),
    }
}

fn profile_of(resolution: &ProfileResolution) -> &ProjectProfile {
    match resolution {
        ProfileResolution::Resolved { profile, .. } => profile,
        ProfileResolution::Ambiguous { profile, .. } => profile,
    }
}

// ---------------------------------------------------------------------
// R1: mandatory shared-service-kit adoption review.
// ---------------------------------------------------------------------

/// One row of the shared-service-kit rule table: a hand-rolled source
/// marker paired with the `libs/*` dependency that would make it redundant.
struct KitRule {
    /// Full, already-prefixed finding id (e.g. `"shared-kit:server-tcp"`).
    /// Named here (rather than a bare suffix formatted at the `finding()`
    /// call site) so `known_rule_docs()` (#2169) can read the exact same id
    /// `apply_shared_kit_rules` emits -- a rule id can never drift between
    /// the finding-emission path and the CONTRIBUTING.md doc-projection
    /// path.
    id: &'static str,
    /// Cargo dependency names that satisfy this rule (any one present
    /// suppresses the finding).
    owning_dependencies: &'static [&'static str],
    /// The `libs/*` crate(s) named in the remediation text.
    owning_crate: &'static str,
    /// Lowercase substrings; any one present in `src/**/*.rs` marks the
    /// capability as hand-rolled.
    hand_rolled_markers: &'static [&'static str],
    /// Human-readable capability name for the summary text.
    capability: &'static str,
    /// If true, this rule additionally requires raft-shaped leader-ingest
    /// source markers (`review::SourceMarkers::leader_ingest`) before it
    /// fires -- keeps the raft row from demanding raft-runtime from a
    /// project with no raft-shaped source at all (AC3).
    requires_raft_shape: bool,
}

const KIT_RULES: &[KitRule] = &[
    KitRule {
        id: "shared-kit:server-tcp",
        owning_dependencies: &["server-tcp", "server-http", "transport-h2c", "service-http"],
        owning_crate: "libs/server-tcp (or server-http/transport-h2c/service-http)",
        hand_rolled_markers: &["tcplistener::bind", "hyper::server::bind"],
        capability: "served-surface bind/listen setup",
        requires_raft_shape: false,
    },
    KitRule {
        id: "shared-kit:server-lifecycle",
        owning_dependencies: &["server-lifecycle"],
        owning_crate: "libs/server-lifecycle",
        hand_rolled_markers: &["\"/healthz\"", "\"/health\""],
        capability: "retry/backoff and health-check plumbing",
        requires_raft_shape: false,
    },
    KitRule {
        id: "shared-kit:service-observability",
        owning_dependencies: &["service-observability"],
        owning_crate: "libs/service-observability",
        hand_rolled_markers: &["mod logging"],
        capability: "structured logging/metrics setup",
        requires_raft_shape: false,
    },
    KitRule {
        id: "shared-kit:raft",
        owning_dependencies: &["raft-core", "raft-runtime"],
        owning_crate: "libs/raft-core (or raft-runtime)",
        hand_rolled_markers: &["leader_ingest", "leaderingest"],
        capability: "raft consensus/leader-ingest plumbing",
        requires_raft_shape: true,
    },
];

/// Substring scan over `<project_dir>/src/**/*.rs`, bounded to the `src/`
/// tree only (mirrors `review::scan_source_markers`'s scan-not-parse
/// style). Returns the sorted, deduplicated set of repo-relative paths
/// containing any of `substrings`.
pub(crate) fn scan_src_for_substrings(project_dir: &Path, substrings: &[&str]) -> Vec<String> {
    let src_root = project_dir.join("src");
    let mut hits = Vec::new();
    if !src_root.is_dir() {
        return hits;
    }
    for entry in walkdir::WalkDir::new(&src_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let lower = content.to_ascii_lowercase();
        if substrings.iter().any(|m| lower.contains(m)) {
            let rel = path
                .strip_prefix(project_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .into_owned();
            hits.push(rel);
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

/// R1: walk `KIT_RULES` and append a finding for every row whose hand-rolled
/// marker is present and whose owning dependency is absent. Skipped
/// entirely for `Cli` profiles (AC3: shared-kit review never demands
/// irrelevant libraries from a CLI profile).
fn apply_shared_kit_rules(project_dir: &Path, resolution: &ProfileResolution) -> Vec<Finding> {
    let profile = profile_of(resolution);
    if profile.kind_surface == KindSurface::Cli {
        return Vec::new();
    }
    let deps = review::read_cargo_dependencies(project_dir);
    let markers = review::scan_source_markers(project_dir);
    let mut findings = Vec::new();
    for rule in KIT_RULES {
        if rule.requires_raft_shape && !markers.leader_ingest {
            continue;
        }
        let owns = rule
            .owning_dependencies
            .iter()
            .any(|d| deps.iter().any(|dep| dep == d));
        if owns {
            continue;
        }
        let hits = scan_src_for_substrings(project_dir, rule.hand_rolled_markers);
        if hits.is_empty() {
            continue;
        }
        findings.push(finding(
            rule.id.to_string(),
            FindingSeverity::High,
            format!(
                "hand-rolled {} detected without adopting {}",
                rule.capability, rule.owning_crate
            ),
            hits,
            format!(
                "adopt {} instead of the hand-rolled implementation",
                rule.owning_crate
            ),
        ));
    }
    findings
}

// ---------------------------------------------------------------------
// R2: profile-specific negative assertions. One new evidence source: a
// k8s-manifest content scan (the #2165 evidence set has no manifest
// *content* scan, only `has_dockerfile_or_manifest`'s presence check).
// ---------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct KitManifestMarkers {
    has_pvc: bool,
    has_headless_service: bool,
    has_statefulset_kind: bool,
    hits: Vec<String>,
}

/// Scan `k8s/`, `deploy/`, or `kubernetes/` subdirectories (mirroring
/// `review::has_dockerfile_or_manifest`'s directory-probing style) for
/// `PersistentVolumeClaim`/`volumeClaimTemplates`/`clusterIP: None`/
/// `kind: StatefulSet` literals.
fn scan_k8s_manifests(project_dir: &Path) -> KitManifestMarkers {
    let mut markers = KitManifestMarkers::default();
    for dir_name in ["k8s", "deploy", "kubernetes"] {
        let dir = project_dir.join(dir_name);
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };
            let mut hit = false;
            if content.contains("PersistentVolumeClaim") || content.contains("volumeClaimTemplates")
            {
                markers.has_pvc = true;
                hit = true;
            }
            if content.contains("clusterIP: None") {
                markers.has_headless_service = true;
                hit = true;
            }
            if content.contains("kind: StatefulSet") {
                markers.has_statefulset_kind = true;
                hit = true;
            }
            if hit {
                let rel = path
                    .strip_prefix(project_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .into_owned();
                markers.hits.push(rel);
            }
        }
    }
    markers.hits.sort();
    markers.hits.dedup();
    markers
}

/// Named finding-id constants for the four R2 negative-assertion rules
/// (#2169): each `finding()` call site below passes the constant instead of
/// an inline string literal, so `known_rule_docs()` can never drift from
/// what is actually emitted.
pub(crate) const RULE_ID_PGPOOL_STATEFULSET_SHAPE: &str = "negative-assertion:pgpool:statefulset-shape";
pub(crate) const RULE_ID_TAPE_RAFT_OR_PRIMARY_REPLICA: &str =
    "negative-assertion:tape:raft-or-primary-replica-signal";
pub(crate) const RULE_ID_RELAY_DEFER_PASSIVE_REPLICA: &str =
    "negative-assertion:relay-defer:passive-replica-signal";
pub(crate) const RULE_ID_LUMEN_RAFT_LEADER_INGEST: &str =
    "negative-assertion:lumen:raft-leader-ingest-signal";

/// R2 Pgpool (Deployment/ExternalState): a stateless external-state
/// Deployment must not inherit StatefulSet/PVC/headless/Raft requirements.
fn pgpool_negative_assertion(project_dir: &Path, resolution: &ProfileResolution) -> Option<Finding> {
    let profile = profile_of(resolution);
    if profile.primary_workload != PrimaryWorkload::Deployment
        || profile.state_ownership != StateOwnership::ExternalState
    {
        return None;
    }
    let manifests = scan_k8s_manifests(project_dir);
    let deps = review::read_cargo_dependencies(project_dir);
    let raft_deps: Vec<String> = deps
        .iter()
        .filter(|d| review::RAFT_DEPENDENCIES.contains(&d.as_str()))
        .cloned()
        .collect();
    if !manifests.has_pvc
        && !manifests.has_headless_service
        && !manifests.has_statefulset_kind
        && raft_deps.is_empty()
    {
        return None;
    }
    let mut affected = manifests.hits.clone();
    affected.extend(raft_deps.iter().map(|d| format!("Cargo.toml:{d}")));
    Some(finding(
        RULE_ID_PGPOOL_STATEFULSET_SHAPE,
        FindingSeverity::High,
        "Deployment/ExternalState profile carries StatefulSet/PVC/headless-service manifest content or a raft dependency",
        affected,
        "remove the PVC/headless-Service/StatefulSet manifest content and any raft-core/raft-runtime dependency -- a stateless external-state Deployment must not inherit StatefulSet/PVC/headless/Raft requirements",
    ))
}

/// R2 Tape (StatefulSet/ReplicatedLog): ordering/checkpoint semantics must
/// be preserved, not silently reinterpreted as raft or primary/replica.
fn tape_negative_assertion(project_dir: &Path, resolution: &ProfileResolution) -> Option<Finding> {
    let profile = profile_of(resolution);
    if profile.primary_workload != PrimaryWorkload::StatefulSet
        || profile.replication != ReplicationConsensus::ReplicatedLog
    {
        return None;
    }
    let deps = review::read_cargo_dependencies(project_dir);
    let raft_deps: Vec<String> = deps
        .iter()
        .filter(|d| review::RAFT_DEPENDENCIES.contains(&d.as_str()))
        .cloned()
        .collect();
    let markers = review::scan_source_markers(project_dir);
    if raft_deps.is_empty() && !markers.primary_replica_role {
        return None;
    }
    let mut affected: Vec<String> = raft_deps.iter().map(|d| format!("Cargo.toml:{d}")).collect();
    affected.extend(
        markers
            .hits
            .iter()
            .filter(|h| h.marker == "primary_replica_role")
            .map(|h| h.path.clone()),
    );
    Some(finding(
        RULE_ID_TAPE_RAFT_OR_PRIMARY_REPLICA,
        FindingSeverity::High,
        "StatefulSet/ReplicatedLog profile carries raft-dependency or primary/replica-role source markers",
        affected,
        "remove the raft-core/raft-runtime dependency and any primary/replica-role source markers -- a replicated-log profile's ordering/checkpoint semantics must be preserved, not silently reinterpreted as raft or primary/replica",
    ))
}

/// R2 Relay/Defer (StatefulSet/RaftConsensus): replicas stay active
/// consensus-owned claim/ack/retry/DLQ executors, never passive read
/// replicas.
fn relay_defer_negative_assertion(
    project_dir: &Path,
    resolution: &ProfileResolution,
) -> Option<Finding> {
    let profile = profile_of(resolution);
    if profile.primary_workload != PrimaryWorkload::StatefulSet
        || profile.replication != ReplicationConsensus::RaftConsensus
    {
        return None;
    }
    let traits = review::read_project_traits(project_dir);
    let has_primary_replicas_trait = traits.iter().any(|t| t == review::PRIMARY_REPLICAS_TRAIT);
    let markers = review::scan_source_markers(project_dir);
    if !has_primary_replicas_trait && !markers.primary_replica_role {
        return None;
    }
    let mut affected = Vec::new();
    if has_primary_replicas_trait {
        affected.push("aw.toml:capability.profile.traits".to_string());
    }
    affected.extend(
        markers
            .hits
            .iter()
            .filter(|h| h.marker == "primary_replica_role")
            .map(|h| h.path.clone()),
    );
    Some(finding(
        RULE_ID_RELAY_DEFER_PASSIVE_REPLICA,
        FindingSeverity::High,
        "StatefulSet/RaftConsensus profile carries a primary_replicas trait or primary/replica-role source markers",
        affected,
        "remove the primary_replicas trait and any primary/replica-role source markers -- a raft-consensus profile's replicas stay active consensus-owned claim/ack/retry/DLQ executors, never passive read replicas",
    ))
}

/// R2 Lumen (StatefulSet/PrimaryReplica): replicas may serve reads, but
/// writes stay leader-committed through the project's own primary role.
fn lumen_negative_assertion(project_dir: &Path, resolution: &ProfileResolution) -> Option<Finding> {
    let profile = profile_of(resolution);
    if profile.primary_workload != PrimaryWorkload::StatefulSet
        || profile.replication != ReplicationConsensus::PrimaryReplica
    {
        return None;
    }
    let deps = review::read_cargo_dependencies(project_dir);
    let raft_deps: Vec<String> = deps
        .iter()
        .filter(|d| review::RAFT_DEPENDENCIES.contains(&d.as_str()))
        .cloned()
        .collect();
    if raft_deps.is_empty() {
        return None;
    }
    let markers = review::scan_source_markers(project_dir);
    if !markers.leader_ingest {
        return None;
    }
    let mut affected: Vec<String> = raft_deps.iter().map(|d| format!("Cargo.toml:{d}")).collect();
    affected.extend(
        markers
            .hits
            .iter()
            .filter(|h| h.marker == "leader_ingest")
            .map(|h| h.path.clone()),
    );
    Some(finding(
        RULE_ID_LUMEN_RAFT_LEADER_INGEST,
        FindingSeverity::High,
        "StatefulSet/PrimaryReplica profile carries a raft dependency plus leader-ingest source markers",
        affected,
        "route writes through the project's own leader-committed primary role instead of raft leader-ingest -- a primary/replica profile's replicas may serve reads, but writes stay leader-committed via the project's own primary role, not rerouted through raft",
    ))
}

// ---------------------------------------------------------------------
// Rule-registry doc projection (#2169): a stable, named-constant view of
// every rule id this module can emit, consumed by
// `review_doc_projection::render_review_rule_table()`. Never a second
// source of truth -- every field here is read directly from the same
// `KIT_RULES` rows / `RULE_ID_*` constants the `finding()` call sites above
// use, so a rule id can never be added, renamed, or removed here without
// also changing what `apply_shared_kit_rules`/the negative-assertion
// functions actually emit.
// ---------------------------------------------------------------------

/// One row of the CONTRIBUTING.md rule-registry doc projection: a rule id,
/// its family, and a short human-readable description of what it flags.
pub(crate) struct RuleDoc {
    pub(crate) id: &'static str,
    pub(crate) family: &'static str,
    pub(crate) description: &'static str,
}

/// Every rule id this module (`review_rules.rs`) can emit: one `RuleDoc`
/// per `KIT_RULES` row (family `"shared-kit"`) followed by one `RuleDoc`
/// per R2 negative-assertion `RULE_ID_*` constant (family
/// `"negative-assertion"`). Insertion order matches source declaration
/// order.
pub(crate) fn known_rule_docs() -> Vec<RuleDoc> {
    let mut docs: Vec<RuleDoc> = KIT_RULES
        .iter()
        .map(|rule| RuleDoc {
            id: rule.id,
            family: "shared-kit",
            description: rule.capability,
        })
        .collect();
    docs.extend([
        RuleDoc {
            id: RULE_ID_PGPOOL_STATEFULSET_SHAPE,
            family: "negative-assertion",
            description: "Deployment/ExternalState profile carries StatefulSet/PVC/headless-service manifest content or a raft dependency",
        },
        RuleDoc {
            id: RULE_ID_TAPE_RAFT_OR_PRIMARY_REPLICA,
            family: "negative-assertion",
            description: "StatefulSet/ReplicatedLog profile carries raft-dependency or primary/replica-role source markers",
        },
        RuleDoc {
            id: RULE_ID_RELAY_DEFER_PASSIVE_REPLICA,
            family: "negative-assertion",
            description: "StatefulSet/RaftConsensus profile carries a primary_replicas trait or primary/replica-role source markers",
        },
        RuleDoc {
            id: RULE_ID_LUMEN_RAFT_LEADER_INGEST,
            family: "negative-assertion",
            description: "StatefulSet/PrimaryReplica profile carries a raft dependency plus leader-ingest source markers",
        },
    ]);
    docs
}

// ---------------------------------------------------------------------
// Entry point.
// ---------------------------------------------------------------------

// <HANDWRITE gap="missing-generator:logic" tracker="#2167" reason="logic section in review_rules.rs is hand-written pending codegen support">
/// Apply R1 (shared-service-kit adoption) then R2 (profile-specific
/// negative assertions) to `project_dir`, using the already-resolved
/// `resolution` to route R2 by profile shape. `Unknown`/`Ambiguous`-shaped
/// profiles receive no R2 finding -- the ambiguity itself is already
/// `#2165`'s finding. Additive: after R1/R2, appends #2167's
/// observability-baseline + raft-telemetry + `any_replica_forward`
/// correctness findings (`review_obs_rules::apply_observability_and_raft_rules`),
/// never replacing or reordering the #2166 findings ahead of it. Read-only:
/// gathers fresh evidence but never writes.
///
/// @spec apps/agentic-workflow/tech-design/config/aw-review-shared-service-kit-adoption-rules-profile-negative-ass.md#logic
/// @spec apps/agentic-workflow/tech-design/validate/aw-review-structured-observability-raft-telemetry-conformance-ru.md#logic
pub(crate) fn apply_conformance_rules(
    project_dir: &Path,
    resolution: &ProfileResolution,
) -> Vec<Finding> {
    let mut findings = apply_shared_kit_rules(project_dir, resolution);
    let profile = profile_of(resolution);
    let negative_assertion = match (
        profile.primary_workload,
        profile.state_ownership,
        profile.replication,
    ) {
        (PrimaryWorkload::Deployment, StateOwnership::ExternalState, _) => {
            pgpool_negative_assertion(project_dir, resolution)
        }
        (PrimaryWorkload::StatefulSet, _, ReplicationConsensus::ReplicatedLog) => {
            tape_negative_assertion(project_dir, resolution)
        }
        (PrimaryWorkload::StatefulSet, _, ReplicationConsensus::RaftConsensus) => {
            relay_defer_negative_assertion(project_dir, resolution)
        }
        (PrimaryWorkload::StatefulSet, _, ReplicationConsensus::PrimaryReplica) => {
            lumen_negative_assertion(project_dir, resolution)
        }
        _ => None,
    };
    findings.extend(negative_assertion);
    findings.extend(crate::cli::review_obs_rules::apply_observability_and_raft_rules(
        project_dir,
        resolution,
    ));
    findings
}
// </HANDWRITE>

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn aw_toml(traits: &[&str]) -> String {
        format!(
            "[project]\nname = \"fixture\"\n\n[capability.profile]\ntraits = [{}]\n",
            traits
                .iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn cargo_toml(deps: &[&str]) -> String {
        let mut out =
            String::from("[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for dep in deps {
            out.push_str(&format!("{dep} = \"1\"\n"));
        }
        out
    }

    // AC3b: a Cli profile short-circuits with zero findings.
    #[test]
    fn cli_profile_produces_no_findings() {
        let tmp = tempfile::tempdir().unwrap();
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&["clap"]));
        write(&tmp.path().join("src/main.rs"), "fn main() {}\n");

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        let findings = apply_conformance_rules(tmp.path(), &resolution);
        assert!(findings.is_empty());
    }

    // R4: the envelope contract is additive-only -- unchanged #2165 keys
    // plus one new `findings` key, always present.
    #[test]
    fn envelope_contract_is_additive_only() {
        let tmp = tempfile::tempdir().unwrap();
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&["clap"]));
        write(&tmp.path().join("src/main.rs"), "fn main() {}\n");

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        let findings = apply_conformance_rules(tmp.path(), &resolution);
        let report = review::ReviewReport {
            project: "fixture".to_string(),
            resolution,
            findings,
        };
        let envelope = review::review_envelope(&report);

        for key in ["schema_version", "action", "project", "outcome", "profile", "evidence"] {
            assert!(!envelope[key].is_null(), "missing pre-existing key {key}");
        }
        assert!(envelope["findings"].is_array());
        assert_eq!(envelope["completion"]["workflow_complete"], true);
    }

    // R3: every finding carries a stable id, severity, non-empty
    // affected_paths, and an executable (never bare "needs review")
    // remediation string.
    #[test]
    fn finding_shape_is_stable_and_executable() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed"]),
        );
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&["sqlx"]));
        write(
            &tmp.path().join("src/lib.rs"),
            "pub fn serve() { let _ = std::net::TcpListener::bind(\"0.0.0.0:0\"); }\n",
        );

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        let findings = apply_shared_kit_rules(tmp.path(), &resolution);
        assert!(!findings.is_empty());
        for f in &findings {
            assert!(!f.id.is_empty());
            assert!(!f.affected_paths.is_empty());
            assert!(!f.remediation.to_ascii_lowercase().contains("needs review"));
            assert!(f.remediation.to_ascii_lowercase().contains("libs/") || !f.remediation.is_empty());
        }
        // Same evidence tree reproduces the same finding ids.
        let findings_again = apply_shared_kit_rules(tmp.path(), &resolution);
        assert_eq!(
            findings.iter().map(|f| f.id.clone()).collect::<Vec<_>>(),
            findings_again.iter().map(|f| f.id.clone()).collect::<Vec<_>>()
        );
    }

    // R1: a hand-rolled TcpListener::bind marker without the owning
    // dependency routes remediation to the owning libs/* crate.
    #[test]
    fn shared_kit_reimplementation_detected() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed"]),
        );
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&["sqlx"]));
        write(
            &tmp.path().join("src/lib.rs"),
            "pub fn serve() { let _ = std::net::TcpListener::bind(\"0.0.0.0:0\"); }\n",
        );

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        let findings = apply_shared_kit_rules(tmp.path(), &resolution);
        let hit = findings.iter().find(|f| f.id == "shared-kit:server-tcp");
        assert!(hit.is_some(), "expected a shared-kit:server-tcp finding, got {findings:?}");
        assert!(hit.unwrap().remediation.contains("libs/server-tcp"));
    }

    // R1b: a project that already adopted the owning dependency produces
    // no finding for that rule row even though the marker is present.
    #[test]
    fn shared_kit_no_finding_when_adopted() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed"]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http", "sqlx"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub fn serve() { let _ = std::net::TcpListener::bind(\"0.0.0.0:0\"); }\n",
        );

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        let findings = apply_shared_kit_rules(tmp.path(), &resolution);
        assert!(findings.iter().all(|f| f.id != "shared-kit:server-tcp"));
    }

    // AC3: the raft rule row only fires when raft-shaped leader-ingest
    // source markers are present -- a plain Deployment with no raft-shaped
    // markers never receives a raft-adoption finding.
    #[test]
    fn shared_kit_raft_rule_gated_on_raft_shaped_markers() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed"]),
        );
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&["service-http"]));
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        let findings = apply_shared_kit_rules(tmp.path(), &resolution);
        assert!(findings.iter().all(|f| f.id != "shared-kit:raft"));
    }

    // R2 Pgpool: a Deployment/ExternalState profile with StatefulSet-shaped
    // manifest content is flagged.
    #[test]
    fn pgpool_negative_assertion() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed"]),
        );
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&[]));
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");
        write(
            &tmp.path().join("k8s/statefulset.yaml"),
            "kind: StatefulSet\nspec:\n  volumeClaimTemplates: []\n",
        );

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        assert_eq!(
            profile_of(&resolution).primary_workload,
            PrimaryWorkload::Deployment
        );
        assert_eq!(
            profile_of(&resolution).state_ownership,
            StateOwnership::ExternalState
        );
        let hit = super::pgpool_negative_assertion(tmp.path(), &resolution);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().id, "negative-assertion:pgpool:statefulset-shape");
    }

    // R2b Tape: a StatefulSet/ReplicatedLog profile with a raft dependency
    // is flagged.
    #[test]
    fn tape_negative_assertion() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed", "stateful_storage"]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http", "sqlx", "raft-core"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub fn serve() {}\npub mod checkpoint { pub fn take() {} }\npub mod segment {}\n",
        );

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        assert_eq!(
            profile_of(&resolution).primary_workload,
            PrimaryWorkload::StatefulSet
        );
        assert_eq!(
            profile_of(&resolution).replication,
            ReplicationConsensus::ReplicatedLog
        );
        let hit = super::tape_negative_assertion(tmp.path(), &resolution);
        assert!(hit.is_some());
        assert_eq!(
            hit.unwrap().id,
            "negative-assertion:tape:raft-or-primary-replica-signal"
        );
    }

    // R2c Relay/Defer: a StatefulSet/RaftConsensus profile with the
    // primary_replicas trait is flagged (would downgrade consensus-owned
    // executors to passive read replicas).
    #[test]
    fn relay_defer_negative_assertion() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&[
                "service",
                "network_exposed",
                "stateful_storage",
                "primary_replicas",
            ]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http", "raft-core"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub fn serve() {}\npub mod leader_ingest { pub fn accept() {} }\n",
        );

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        assert_eq!(
            profile_of(&resolution).primary_workload,
            PrimaryWorkload::StatefulSet
        );
        assert_eq!(
            profile_of(&resolution).replication,
            ReplicationConsensus::RaftConsensus
        );
        let hit = super::relay_defer_negative_assertion(tmp.path(), &resolution);
        assert!(hit.is_some());
        assert_eq!(
            hit.unwrap().id,
            "negative-assertion:relay-defer:passive-replica-signal"
        );
    }

    // R2d Lumen: a StatefulSet/PrimaryReplica profile with a raft
    // dependency plus leader-ingest markers is flagged (writes would be
    // rerouted off the project's own leader-committed primary path). This
    // exact evidence combination (raft dependency + leader-ingest marker)
    // is what `resolve_project_profile_for_dir` itself classifies as a
    // `RaftConsensus` profile (see `## Logic` in the #2165 TD), so a
    // *resolved* `PrimaryReplica` profile can only carry this evidence if
    // the resolution was computed earlier and the source has since
    // drifted -- exactly the regression `## Logic` describes this rule as
    // catching. The fixture constructs that already-resolved
    // `PrimaryReplica` profile directly rather than re-deriving it, so the
    // rule is exercised against its own structural-contradiction check
    // independent of the classifier that produced the (now stale)
    // resolution.
    #[test]
    fn lumen_negative_assertion() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http", "raft-core"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub fn serve() {}\npub mod primary { pub fn write() {} }\npub mod replica { pub fn read() {} }\npub mod leader_ingest { pub fn accept() {} }\n",
        );

        let resolution = review::ProfileResolution::Resolved {
            profile: review::ProjectProfile {
                kind_surface: review::KindSurface::Service,
                primary_workload: PrimaryWorkload::StatefulSet,
                state_ownership: StateOwnership::OwnedState,
                replication: ReplicationConsensus::PrimaryReplica,
                serving_role: review::ServingRole::PrimaryWriteReplicaRead,
            },
            evidence: Vec::new(),
        };
        let hit = super::lumen_negative_assertion(tmp.path(), &resolution);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().id, "negative-assertion:lumen:raft-leader-ingest-signal");
    }

    // #2169 R1: KIT_RULES.id must be the full, already-prefixed
    // "shared-kit:<x>" string -- apply_shared_kit_rules's finding() call
    // site passes rule.id directly, with no format!() prefixing left at
    // the call site.
    #[test]
    fn shared_kit_rule_ids_are_prefixed() {
        let ids: Vec<&str> = KIT_RULES.iter().map(|r| r.id).collect();
        assert_eq!(
            ids,
            vec![
                "shared-kit:server-tcp",
                "shared-kit:server-lifecycle",
                "shared-kit:service-observability",
                "shared-kit:raft",
            ]
        );
        for id in &ids {
            assert!(id.starts_with("shared-kit:"));
        }
    }

    // #2169 R2: the pgpool negative-assertion finding's id equals the named
    // RULE_ID_PGPOOL_STATEFULSET_SHAPE constant (not a re-typed literal).
    #[test]
    fn pgpool_negative_assertion_flags_deployment_shape() {
        let tmp = tempfile::tempdir().unwrap();
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&["raft-core"]));
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = review::ProfileResolution::Resolved {
            profile: review::ProjectProfile {
                kind_surface: review::KindSurface::Service,
                primary_workload: PrimaryWorkload::Deployment,
                state_ownership: StateOwnership::ExternalState,
                replication: ReplicationConsensus::None,
                serving_role: review::ServingRole::Standard,
            },
            evidence: Vec::new(),
        };
        let hit = super::pgpool_negative_assertion(tmp.path(), &resolution);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().id, RULE_ID_PGPOOL_STATEFULSET_SHAPE);
    }

    // #2169 R3: known_rule_docs() covers every KIT_RULES row plus every
    // negative-assertion RULE_ID_* constant, ids matching byte-for-byte --
    // the structural guarantee the CONTRIBUTING.md doc projection relies on.
    #[test]
    fn known_rule_docs_ids_match_shared_kit_and_negative_assertion_consts() {
        let docs = known_rule_docs();
        let doc_ids: Vec<&str> = docs.iter().map(|d| d.id).collect();

        let mut expected: Vec<&str> = KIT_RULES.iter().map(|r| r.id).collect();
        expected.extend([
            RULE_ID_PGPOOL_STATEFULSET_SHAPE,
            RULE_ID_TAPE_RAFT_OR_PRIMARY_REPLICA,
            RULE_ID_RELAY_DEFER_PASSIVE_REPLICA,
            RULE_ID_LUMEN_RAFT_LEADER_INGEST,
        ]);

        assert_eq!(doc_ids, expected);
        for doc in &docs {
            assert!(
                doc.family == "shared-kit" || doc.family == "negative-assertion",
                "unexpected family {} for rule id {}",
                doc.family,
                doc.id
            );
            assert!(!doc.description.is_empty());
        }
    }
}
// HANDWRITE-END
