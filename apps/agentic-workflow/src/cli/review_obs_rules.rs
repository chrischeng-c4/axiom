// HANDWRITE-BEGIN gap="missing-generator:logic:157b86c6" tracker="pending-tracker" reason="New module implementing R1 (structured-observability baseline) + R2 (raft-runtime telemetry for `any_replica_forward`) + R3 (`any_replica_forward` correctness) as `pub(crate) fn apply_observability_and_raft_rules(project_dir: &Path, resolution: &review::ProfileResolution) -> Vec<crate::cli::review_rules::Finding>`. Reuses the exact `Finding`/`FindingSeverity` type from `review_rules.rs` -- never a new or re-exported type -- and reuses `review::read_cargo_dependencies`, `review::scan_source_markers`, and `review_rules::scan_src_for_substrings` (widened to `pub(crate)` by the sibling `review_rules.rs` change below) for evidence gathering; adds no new evidence source of its own. R1 (skipped entirely when `resolution.profile.kind_surface != review::KindSurface::Service`): - `obs:structured-logging-metrics-adoption` (severity: high) fires whenever the `service-observability` Cargo dependency is absent, regardless of whether the project hand-rolled a substitute (e.g. a local `obs_counter!`-style macro) or has no logging/metrics/correlation setup at all -- a mandatory-baseline rule, not an anti-pattern-detection rule (contrast with #2166's `shared-kit:service-observability` row, which only fires when a hand-rolled marker is present). - `obs:w3c-context-propagation-adoption` (severity: high) fires when neither `service-http` nor `transport-h2c` is a Cargo dependency (both carry the W3C `traceparent` baseline referenced in `libs/service-http/src/transport.rs`). R2 (applies only when `resolution.profile.replication == review::ReplicationConsensus::RaftConsensus` AND `review::scan_source_markers(project_dir).leader_ingest` is true -- reviewed against the profiled project's own source, never `libs/raft-runtime`'s internals directly): - `raft:proposal-routing-telemetry-gap` (high): fires unless the source contains at least one of the lowercase substrings `local_proposals`, `forwarded_proposals`, `forward_duration`, `forwarded_bytes`. - `raft:leader-route-and-replication-lag-telemetry-gap` (high): fires unless the source contains at least one of `leader_route_retr`, `leader_change`, `commit_lag`, `applied_lag`, `peer_rpc`. - `raft:high-cardinality-label-antipattern` (high): fires when a metric-emission marker (`counter!(`, `histogram!(`, `gauge!(`, or `obs_counter!(` -- any substring containing `counter!(`/`histogram!(`/`gauge!(` matches the `obs_counter!` shape too) and a high-cardinality label-key literal (`'queue'`, `'topic'`, `'message_id'`, `'message'`) both appear in the same file (computed by intersecting the file sets returned by two `scan_src_for_substrings` calls -- a positive-violation check, never fired on absence). - `raft:trace-context-continuity-gap` (medium): fires unless the source contains at least one of `traceparent`, `trace_id`, `span_id`, `tracing::instrument`, `info_span!`. R3 (same RaftConsensus + `leader_ingest` gate as R2; positive-violation only -- never fired on the absence of a marker, since `libs/raft-runtime`'s own `propose()`/`forward()` path already fails closed by construction): - `raft:follower-local-mutation-outside-consensus` (high -- this module's ceiling severity, the one correctness blocker as opposed to every other finding here being an observability-completeness gap): fires when the source shows a `follower` or `replica` marker co-occurring with a `bypass_raft`, `bypass_consensus`, `local_write_outside_consensus`, or `direct_local_write` marker in the same file. - `raft:loss-of-leader-fail-open-bypass` (high): fires when the source contains any of `accept_without_leader`, `bypass_leader_check`, `skip_quorum_check`, `local_write_fallback`, `fail_open`. There is no rule anywhere in this module keyed to 'does this project expose direct-leader ingress' -- its absence structurally can never produce a finding (AC5/R3). Includes a `#[cfg(test)] mod tests` with one `#[test]` fn per Unit Test requirement id below, using `tempfile::TempDir`-backed fixture project directories (matching `review.rs`/`review_rules.rs`'s existing `#[cfg(test)]` fixture style) to construct minimal Cargo.toml/aw.toml/src content per scenario, plus fixtures that directly construct an already-resolved `ProfileResolution` (matching `review_rules.rs`'s `lumen_negative_assertion` test precedent) where the scenario needs a specific profile shape independent of the #2165 classifier. gap: observability-raft-conformance-rule-heuristics tracker: '#2167'"
//! R1 (structured-observability baseline) + R2 (raft-runtime telemetry gaps
//! for `any_replica_forward`) + R3 (`any_replica_forward` correctness),
//! additive to the #2166 findings computed by
//! `review_rules::apply_conformance_rules`. Reuses the exact
//! `Finding`/`FindingSeverity` type from `review_rules.rs` -- never a new or
//! re-exported type -- and reuses `review::read_cargo_dependencies`,
//! `review::scan_source_markers`, and `review_rules::scan_src_for_substrings`
//! for evidence gathering; adds no new evidence source of its own.
//!
//! @spec apps/agentic-workflow/tech-design/validate/aw-review-structured-observability-raft-telemetry-conformance-ru.md#logic

use std::path::Path;

use crate::cli::review::{
    self, KindSurface, PrimaryWorkload, ProfileResolution, ProjectProfile, ReplicationConsensus,
    ServingRole, SourceMarkers, StateOwnership,
};
use crate::cli::review_rules;
use crate::cli::review_rules::{scan_src_for_substrings, Finding, FindingSeverity};

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
// Entry point.
// ---------------------------------------------------------------------

// <HANDWRITE gap="missing-generator:logic" tracker="#2169" reason="logic section in review_obs_rules.rs is hand-written pending codegen support">
/// Apply R1 (structured-observability baseline) then R2/R3 (raft-runtime
/// telemetry + `any_replica_forward` correctness) to `project_dir`, using
/// the already-resolved `resolution` to gate each rule group. Additive to
/// `review_rules::apply_conformance_rules`'s #2166 findings -- never
/// mutates `ProjectProfile`/`ProfileResolution`, only reads it. Read-only:
/// gathers fresh evidence but never writes.
///
/// @spec apps/agentic-workflow/tech-design/validate/aw-review-structured-observability-raft-telemetry-conformance-ru.md#logic
pub(crate) fn apply_observability_and_raft_rules(
    project_dir: &Path,
    resolution: &ProfileResolution,
) -> Vec<Finding> {
    let mut findings = apply_observability_baseline_rules(project_dir, resolution);
    findings.extend(apply_raft_telemetry_and_correctness_rules(
        project_dir,
        resolution,
    ));
    findings
}
// </HANDWRITE>

// ---------------------------------------------------------------------
// R1: structured-observability baseline. Mandatory-baseline polarity --
// fires on absence of adoption, regardless of whether a hand-rolled
// substitute exists. Skipped entirely for `Cli` profiles (a pure CLI/tool
// project has no served surface to observe).
// ---------------------------------------------------------------------

/// Named finding-id constants for this module's eight rules (#2169): each
/// `finding()` call site below passes the matching constant instead of an
/// inline string literal, so `known_rule_docs()` can never drift from what
/// is actually emitted.
pub(crate) const RULE_ID_OBS_STRUCTURED_LOGGING: &str = "obs:structured-logging-metrics-adoption";
pub(crate) const RULE_ID_OBS_W3C_CONTEXT: &str = "obs:w3c-context-propagation-adoption";
pub(crate) const RULE_ID_RAFT_PROPOSAL_ROUTING_TELEMETRY: &str =
    "raft:proposal-routing-telemetry-gap";
pub(crate) const RULE_ID_RAFT_LEADER_ROUTE_REPLICATION_LAG: &str =
    "raft:leader-route-and-replication-lag-telemetry-gap";
pub(crate) const RULE_ID_RAFT_HIGH_CARDINALITY_LABEL: &str =
    "raft:high-cardinality-label-antipattern";
pub(crate) const RULE_ID_RAFT_TRACE_CONTEXT_CONTINUITY: &str = "raft:trace-context-continuity-gap";
pub(crate) const RULE_ID_RAFT_FOLLOWER_LOCAL_MUTATION: &str =
    "raft:follower-local-mutation-outside-consensus";
pub(crate) const RULE_ID_RAFT_LOSS_OF_LEADER_FAIL_OPEN: &str =
    "raft:loss-of-leader-fail-open-bypass";

fn apply_observability_baseline_rules(
    project_dir: &Path,
    resolution: &ProfileResolution,
) -> Vec<Finding> {
    let profile = profile_of(resolution);
    if profile.kind_surface != KindSurface::Service {
        return Vec::new();
    }
    let deps = review::read_cargo_dependencies(project_dir);
    let mut findings = Vec::new();
    if !deps.iter().any(|d| d == "service-observability") {
        findings.push(finding(
            RULE_ID_OBS_STRUCTURED_LOGGING,
            FindingSeverity::High,
            "service surface has not adopted libs/service-observability for structured logging/metrics/correlation",
            vec!["Cargo.toml".to_string()],
            "adopt libs/service-observability for the axiom.service.log.v1 structured-logging + metrics + correlation baseline instead of ad hoc or hand-rolled logging/metrics",
        ));
    }
    if !deps
        .iter()
        .any(|d| d == "service-http" || d == "transport-h2c")
    {
        findings.push(finding(
            RULE_ID_OBS_W3C_CONTEXT,
            FindingSeverity::High,
            "service surface has not adopted libs/service-http or libs/transport-h2c for W3C trace-context (traceparent) propagation",
            vec!["Cargo.toml".to_string()],
            "adopt libs/service-http (or libs/transport-h2c) so inbound/outbound requests carry and propagate the W3C traceparent header",
        ));
    }
    findings
}

// ---------------------------------------------------------------------
// R2/R3: raft-runtime telemetry + `any_replica_forward` correctness.
// Applies only when `resolution.profile.replication ==
// ReplicationConsensus::RaftConsensus` AND
// `review::scan_source_markers(project_dir).leader_ingest` is true --
// reviewed against the profiled project's own source, never
// `libs/raft-runtime`'s internals directly. R2 (telemetry) rules are
// mandatory-baseline polarity (fire on absence); R3 (correctness) rules are
// positive-violation-only polarity (fire only on an explicit anti-pattern
// marker, never on absence) -- `libs/raft-runtime`'s own `propose()`/
// `forward()` path already fails closed by construction, so an
// absence-based correctness rule would false-positive on every compliant
// adopter. There is no rule anywhere in this module keyed to "does this
// project expose direct-leader ingress" -- its absence structurally can
// never produce a finding.
// ---------------------------------------------------------------------

fn apply_raft_telemetry_and_correctness_rules(
    project_dir: &Path,
    resolution: &ProfileResolution,
) -> Vec<Finding> {
    let profile = profile_of(resolution);
    if profile.replication != ReplicationConsensus::RaftConsensus {
        return Vec::new();
    }
    let markers = review::scan_source_markers(project_dir);
    if !markers.leader_ingest {
        return Vec::new();
    }
    let mut findings = Vec::new();
    findings.extend(proposal_routing_telemetry_gap(project_dir, &markers));
    findings.extend(leader_route_and_replication_lag_telemetry_gap(
        project_dir,
        &markers,
    ));
    findings.extend(high_cardinality_label_antipattern(project_dir));
    findings.extend(trace_context_continuity_gap(project_dir, &markers));
    findings.extend(follower_local_mutation_outside_consensus(project_dir));
    findings.extend(loss_of_leader_fail_open_bypass(project_dir));
    findings
}

/// Evidence-citation paths for an absence-triggered raft rule: the files
/// where the leader-ingest marker was found (the surface that lacks the
/// telemetry), falling back to `src/` if the marker hit list is empty.
fn leader_ingest_paths(markers: &SourceMarkers) -> Vec<String> {
    let hits: Vec<String> = markers
        .hits
        .iter()
        .filter(|h| h.marker == "leader_ingest")
        .map(|h| h.path.clone())
        .collect();
    if hits.is_empty() {
        vec!["src/".to_string()]
    } else {
        hits
    }
}

/// R2a: fires unless the source shows local-vs-forwarded proposal routing
/// telemetry (counts and/or forward duration/bytes).
fn proposal_routing_telemetry_gap(project_dir: &Path, markers: &SourceMarkers) -> Option<Finding> {
    const TELEMETRY_MARKERS: &[&str] = &[
        "local_proposals",
        "forwarded_proposals",
        "forward_duration",
        "forwarded_bytes",
    ];
    if !scan_src_for_substrings(project_dir, TELEMETRY_MARKERS).is_empty() {
        return None;
    }
    Some(finding(
        RULE_ID_RAFT_PROPOSAL_ROUTING_TELEMETRY,
        FindingSeverity::High,
        "raft leader-ingest surface has no proposal-routing telemetry (local-vs-forwarded proposal counts, forward duration, or forwarded bytes)",
        leader_ingest_paths(markers),
        "instrument any_replica_forward's proposal routing with local_proposals/forwarded_proposals counters plus forward_duration/forwarded_bytes so operators can see local-vs-forwarded proposal volume and forwarding cost",
    ))
}

/// R2b: fires unless the source shows leader-route-retry/leader-change or
/// commit/applied-lag/peer-RPC telemetry.
fn leader_route_and_replication_lag_telemetry_gap(
    project_dir: &Path,
    markers: &SourceMarkers,
) -> Option<Finding> {
    const TELEMETRY_MARKERS: &[&str] = &[
        "leader_route_retr",
        "leader_change",
        "commit_lag",
        "applied_lag",
        "peer_rpc",
    ];
    if !scan_src_for_substrings(project_dir, TELEMETRY_MARKERS).is_empty() {
        return None;
    }
    Some(finding(
        RULE_ID_RAFT_LEADER_ROUTE_REPLICATION_LAG,
        FindingSeverity::High,
        "raft leader-ingest surface has no leader-route or replication-lag telemetry (leader-route retries/changes, commit/applied lag, or peer RPC visibility)",
        leader_ingest_paths(markers),
        "instrument leader-route retries/leader-change events plus commit_lag/applied_lag and peer_rpc visibility so operators can see replication health and leader-routing behavior",
    ))
}

/// R2c: positive-violation check -- fires when a metric-emission marker
/// (`counter!(`/`histogram!(`/`gauge!(`, which also matches the
/// `obs_counter!(` shape) and a high-cardinality label-key literal
/// (`"queue"`/`"topic"`/`"message_id"`/`"message"`) both appear in the same
/// file. Never fired on absence.
fn high_cardinality_label_antipattern(project_dir: &Path) -> Option<Finding> {
    const METRIC_MARKERS: &[&str] = &["counter!(", "histogram!(", "gauge!(", "obs_counter!("];
    const HIGH_CARDINALITY_LABELS: &[&str] =
        &["\"queue\"", "\"topic\"", "\"message_id\"", "\"message\""];
    let metric_hits = scan_src_for_substrings(project_dir, METRIC_MARKERS);
    if metric_hits.is_empty() {
        return None;
    }
    let label_hits = scan_src_for_substrings(project_dir, HIGH_CARDINALITY_LABELS);
    let overlap: Vec<String> = metric_hits
        .into_iter()
        .filter(|p| label_hits.contains(p))
        .collect();
    if overlap.is_empty() {
        return None;
    }
    Some(finding(
        RULE_ID_RAFT_HIGH_CARDINALITY_LABEL,
        FindingSeverity::High,
        "a metric-emission site carries a high-cardinality label key (queue/topic/message_id/message) in the same file as a raft telemetry metric",
        overlap,
        "drop the high-cardinality queue/topic/message_id/message label from raft telemetry metrics -- cardinality explosion breaks metric-backend scrape/storage at scale; carry that context on a trace span or log field instead",
    ))
}

/// R2d: fires unless the source shows W3C trace-context or internal
/// trace/span-id continuity evidence across the forwarded-proposal path.
fn trace_context_continuity_gap(project_dir: &Path, markers: &SourceMarkers) -> Option<Finding> {
    const CONTINUITY_MARKERS: &[&str] = &[
        "traceparent",
        "trace_id",
        "span_id",
        "tracing::instrument",
        "info_span!",
    ];
    if !scan_src_for_substrings(project_dir, CONTINUITY_MARKERS).is_empty() {
        return None;
    }
    Some(finding(
        RULE_ID_RAFT_TRACE_CONTEXT_CONTINUITY,
        FindingSeverity::Medium,
        "raft leader-ingest surface has no trace-context continuity evidence (traceparent/trace_id/span_id propagation or tracing::instrument/info_span! spans) across the forwarded-proposal path",
        leader_ingest_paths(markers),
        "propagate the inbound W3C traceparent (or an internal trace_id/span_id) across any_replica_forward's forwarded-proposal path, wrapping the routing/forwarding call in a tracing::instrument or info_span! span",
    ))
}

/// R3a: positive-violation check -- fires when a follower/replica-role
/// marker co-occurs with an explicit consensus-bypass marker in the same
/// file. Never fired on absence: `libs/raft-runtime`'s own consensus path
/// already fails closed by construction.
fn follower_local_mutation_outside_consensus(project_dir: &Path) -> Option<Finding> {
    const ROLE_MARKERS: &[&str] = &["follower", "replica"];
    const BYPASS_MARKERS: &[&str] = &[
        "bypass_raft",
        "bypass_consensus",
        "local_write_outside_consensus",
        "direct_local_write",
    ];
    let role_hits = scan_src_for_substrings(project_dir, ROLE_MARKERS);
    if role_hits.is_empty() {
        return None;
    }
    let bypass_hits = scan_src_for_substrings(project_dir, BYPASS_MARKERS);
    let overlap: Vec<String> = role_hits
        .into_iter()
        .filter(|p| bypass_hits.contains(p))
        .collect();
    if overlap.is_empty() {
        return None;
    }
    Some(finding(
        RULE_ID_RAFT_FOLLOWER_LOCAL_MUTATION,
        FindingSeverity::High,
        "a follower/replica-role marker co-occurs with an explicit consensus-bypass marker in the same file -- a follower appears to mutate local state outside raft consensus",
        overlap,
        "route the mutation through the raft leader via any_replica_forward instead of writing local state directly on a follower/replica -- consensus-owned state must never be mutated outside the raft commit path",
    ))
}

/// R3b: positive-violation check -- fires when the source contains an
/// explicit fail-open/bypass-on-loss-of-leader marker. Never fired on
/// absence: `libs/raft-runtime`'s `propose()` already `bail!`s when no
/// leader is elected.
fn loss_of_leader_fail_open_bypass(project_dir: &Path) -> Option<Finding> {
    const BYPASS_MARKERS: &[&str] = &[
        "accept_without_leader",
        "bypass_leader_check",
        "skip_quorum_check",
        "local_write_fallback",
        "fail_open",
    ];
    let hits = scan_src_for_substrings(project_dir, BYPASS_MARKERS);
    if hits.is_empty() {
        return None;
    }
    Some(finding(
        RULE_ID_RAFT_LOSS_OF_LEADER_FAIL_OPEN,
        FindingSeverity::High,
        "an explicit fail-open marker allows accepting writes without a confirmed leader / bypassing the quorum check on loss-of-leader",
        hits,
        "fail closed on loss-of-leader -- reject writes until a leader is elected instead of accepting them without a leader/quorum check (libs/raft-runtime's own propose() already fails closed by construction; this project's own bypass must be removed)",
    ))
}

// ---------------------------------------------------------------------
// Rule-registry doc projection (#2169): a stable, named-constant view of
// every rule id this module can emit, consumed by
// `review_doc_projection::render_review_rule_table()`. Never a second
// source of truth -- every id here is read directly from the same
// `RULE_ID_*` constants the `finding()` call sites above use.
// ---------------------------------------------------------------------

/// Every rule id this module (`review_obs_rules.rs`) can emit: the two
/// R1 observability-baseline constants (family `"obs"`) followed by the
/// six R2/R3 raft-telemetry-and-correctness constants (family `"raft"`).
/// Insertion order matches source declaration order.
pub(crate) fn known_rule_docs() -> Vec<review_rules::RuleDoc> {
    vec![
        review_rules::RuleDoc {
            id: RULE_ID_OBS_STRUCTURED_LOGGING,
            family: "obs",
            description: "service surface has not adopted libs/service-observability for structured logging/metrics/correlation",
        },
        review_rules::RuleDoc {
            id: RULE_ID_OBS_W3C_CONTEXT,
            family: "obs",
            description: "service surface has not adopted libs/service-http or libs/transport-h2c for W3C trace-context (traceparent) propagation",
        },
        review_rules::RuleDoc {
            id: RULE_ID_RAFT_PROPOSAL_ROUTING_TELEMETRY,
            family: "raft",
            description: "raft leader-ingest surface has no proposal-routing telemetry (local-vs-forwarded proposal counts, forward duration, or forwarded bytes)",
        },
        review_rules::RuleDoc {
            id: RULE_ID_RAFT_LEADER_ROUTE_REPLICATION_LAG,
            family: "raft",
            description: "raft leader-ingest surface has no leader-route or replication-lag telemetry (leader-route retries/changes, commit/applied lag, or peer RPC visibility)",
        },
        review_rules::RuleDoc {
            id: RULE_ID_RAFT_HIGH_CARDINALITY_LABEL,
            family: "raft",
            description: "a metric-emission site carries a high-cardinality label key (queue/topic/message_id/message) in the same file as a raft telemetry metric",
        },
        review_rules::RuleDoc {
            id: RULE_ID_RAFT_TRACE_CONTEXT_CONTINUITY,
            family: "raft",
            description: "raft leader-ingest surface has no trace-context continuity evidence (traceparent/trace_id/span_id propagation or tracing::instrument/info_span! spans) across the forwarded-proposal path",
        },
        review_rules::RuleDoc {
            id: RULE_ID_RAFT_FOLLOWER_LOCAL_MUTATION,
            family: "raft",
            description: "a follower/replica-role marker co-occurs with an explicit consensus-bypass marker in the same file -- a follower appears to mutate local state outside raft consensus",
        },
        review_rules::RuleDoc {
            id: RULE_ID_RAFT_LOSS_OF_LEADER_FAIL_OPEN,
            family: "raft",
            description: "an explicit fail-open marker allows accepting writes without a confirmed leader / bypassing the quorum check on loss-of-leader",
        },
    ]
}

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

    fn cargo_toml(deps: &[&str]) -> String {
        let mut out =
            String::from("[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for dep in deps {
            out.push_str(&format!("{dep} = \"1\"\n"));
        }
        out
    }

    fn service_profile() -> ProjectProfile {
        ProjectProfile {
            kind_surface: KindSurface::Service,
            primary_workload: PrimaryWorkload::Deployment,
            state_ownership: StateOwnership::ExternalState,
            replication: ReplicationConsensus::None,
            serving_role: ServingRole::Standard,
        }
    }

    fn raft_profile() -> ProjectProfile {
        ProjectProfile {
            kind_surface: KindSurface::Service,
            primary_workload: PrimaryWorkload::StatefulSet,
            state_ownership: StateOwnership::OwnedState,
            replication: ReplicationConsensus::RaftConsensus,
            serving_role: ServingRole::LeaderIngest,
        }
    }

    fn resolved(profile: ProjectProfile) -> ProfileResolution {
        ProfileResolution::Resolved {
            profile,
            evidence: Vec::new(),
        }
    }

    // AC-CLI: a Cli profile short-circuits with zero observability/raft
    // findings (no served surface to observe, no replication to review).
    #[test]
    fn cli_profile_produces_no_observability_or_raft_findings() {
        let tmp = tempfile::tempdir().unwrap();
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&["clap"]));
        write(&tmp.path().join("src/main.rs"), "fn main() {}\n");

        let resolution = review::resolve_project_profile_for_dir(tmp.path());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings.is_empty());
    }

    // AC4a: a service surface with no service-observability dependency is
    // flagged, regardless of any other setup.
    #[test]
    fn service_profile_missing_service_observability_dependency_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = resolved(service_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == "obs:structured-logging-metrics-adoption"));
    }

    // AC4b: adopting service-observability suppresses the baseline finding.
    #[test]
    fn service_profile_with_service_observability_adopted_produces_no_finding() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http", "service-observability"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = resolved(service_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .all(|f| f.id != "obs:structured-logging-metrics-adoption"));
    }

    // AC4c: a hand-rolled logging/metrics substitute (e.g. a local
    // obs_counter! macro) does not suppress the finding -- this is a
    // mandatory-baseline rule, not an anti-pattern-detection rule.
    #[test]
    fn service_profile_hand_rolled_logging_substitute_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "macro_rules! obs_counter { ($name:expr) => {}; }\npub fn serve() { obs_counter!(\"served\"); }\n",
        );

        let resolution = resolved(service_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        let hit = findings
            .iter()
            .find(|f| f.id == "obs:structured-logging-metrics-adoption");
        assert!(
            hit.is_some(),
            "hand-rolled substitute must still be flagged when service-observability is absent"
        );
    }

    // AC4d: neither service-http nor transport-h2c adopted -> flagged for
    // missing W3C trace-context propagation.
    #[test]
    fn service_profile_missing_w3c_transport_adoption_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-observability"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = resolved(service_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == "obs:w3c-context-propagation-adoption"));
    }

    // AC6a: raft leader-ingest surface with no proposal-routing telemetry
    // is flagged.
    #[test]
    fn raft_profile_missing_proposal_routing_telemetry_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub mod leader_ingest { pub fn accept() {} }\n",
        );

        let resolution = resolved(raft_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == "raft:proposal-routing-telemetry-gap"));
    }

    // AC6b: raft leader-ingest surface with no leader-route/replication-lag
    // telemetry is flagged.
    #[test]
    fn raft_profile_missing_leader_route_and_lag_telemetry_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub mod leader_ingest { pub fn accept() {} }\n",
        );

        let resolution = resolved(raft_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == "raft:leader-route-and-replication-lag-telemetry-gap"));
    }

    // AC6c: a metric-emission marker co-occurring with a high-cardinality
    // label literal in the same file is flagged.
    #[test]
    fn raft_profile_high_cardinality_label_antipattern_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub mod leader_ingest { pub fn accept() {} }\n",
        );
        write(
            &tmp.path().join("src/metrics.rs"),
            "pub fn record() { counter!(\"raft_forward\", \"queue\" => \"default\"); }\n",
        );

        let resolution = resolved(raft_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == "raft:high-cardinality-label-antipattern"));
    }

    // AC6d: raft leader-ingest surface with no trace-context continuity
    // evidence is flagged.
    #[test]
    fn raft_profile_missing_trace_context_continuity_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub mod leader_ingest { pub fn accept() {} }\n",
        );

        let resolution = resolved(raft_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == "raft:trace-context-continuity-gap"));
    }

    // AC5a: a compliant leader-only-commit-with-forwarding pattern (no
    // consensus-bypass markers anywhere) never receives an R3 correctness
    // finding.
    #[test]
    fn raft_profile_leader_only_commit_with_forwarding_passes() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub mod leader_ingest {\n    pub fn forward() {\n        // forwarded_proposals, leader_change, commit_lag, traceparent all instrumented.\n    }\n}\n",
        );

        let resolution = resolved(raft_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .all(|f| f.id != "raft:follower-local-mutation-outside-consensus"));
        assert!(findings
            .iter()
            .all(|f| f.id != "raft:loss-of-leader-fail-open-bypass"));
    }

    // AC5b: a follower/replica-role marker co-occurring with an explicit
    // consensus-bypass marker in the same file is flagged.
    #[test]
    fn raft_profile_follower_local_mutation_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub mod leader_ingest { pub fn accept() {} }\n",
        );
        write(
            &tmp.path().join("src/follower.rs"),
            "pub fn handle_follower_write() { bypass_raft(); }\nfn bypass_raft() {}\n",
        );

        let resolution = resolved(raft_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == "raft:follower-local-mutation-outside-consensus"));
    }

    // AC5c: an explicit fail-open/bypass-on-loss-of-leader marker is
    // flagged.
    #[test]
    fn raft_profile_loss_of_leader_fail_open_bypass_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub mod leader_ingest { pub fn accept() {} }\n",
        );
        write(
            &tmp.path().join("src/fallback.rs"),
            "pub fn handle() { accept_without_leader(); }\nfn accept_without_leader() {}\n",
        );

        let resolution = resolved(raft_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == "raft:loss-of-leader-fail-open-bypass"));
    }

    // AC5d: absence of direct-leader-ingress evidence is NOT itself a
    // finding -- the R2/R3 gate requires the leader_ingest marker to be
    // present; its absence produces zero raft: findings (obs: findings from
    // R1 may still fire, since this is still a Service-kind profile).
    #[test]
    fn raft_profile_missing_direct_leader_ingress_produces_no_finding() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = resolved(raft_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(
            findings.iter().all(|f| !f.id.starts_with("raft:")),
            "absence of direct-leader-ingress evidence must never itself produce a raft finding, got {findings:?}"
        );
    }

    // R3: a non-raft profile never receives raft: findings, even when the
    // project's source happens to carry raft-shaped markers (a stale
    // dependency/marker on a project that isn't actually a raft profile
    // must not leak raft: findings into an unrelated profile's report).
    #[test]
    fn non_raft_profile_never_receives_raft_findings() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["raft-runtime"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub mod leader_ingest { pub fn accept() {} }\nfn bypass_raft() {}\n",
        );

        let resolution = resolved(service_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings.iter().all(|f| !f.id.starts_with("raft:")));
    }

    // #2169 R4: the structured-logging finding's id equals the named
    // RULE_ID_OBS_STRUCTURED_LOGGING constant (not a re-typed literal).
    #[test]
    fn structured_logging_metrics_rule_uses_named_const_id() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = resolved(service_profile());
        let findings = apply_observability_and_raft_rules(tmp.path(), &resolution);
        assert!(findings
            .iter()
            .any(|f| f.id == RULE_ID_OBS_STRUCTURED_LOGGING));
    }

    // #2169 R5: known_rule_docs() covers all eight obs/raft RULE_ID_*
    // constants, ids matching byte-for-byte and family set to "obs"/"raft"
    // correctly -- the structural guarantee the CONTRIBUTING.md doc
    // projection relies on.
    #[test]
    fn known_rule_docs_ids_match_obs_and_raft_consts() {
        let docs = known_rule_docs();
        let doc_ids: Vec<&str> = docs.iter().map(|d| d.id).collect();
        assert_eq!(
            doc_ids,
            vec![
                RULE_ID_OBS_STRUCTURED_LOGGING,
                RULE_ID_OBS_W3C_CONTEXT,
                RULE_ID_RAFT_PROPOSAL_ROUTING_TELEMETRY,
                RULE_ID_RAFT_LEADER_ROUTE_REPLICATION_LAG,
                RULE_ID_RAFT_HIGH_CARDINALITY_LABEL,
                RULE_ID_RAFT_TRACE_CONTEXT_CONTINUITY,
                RULE_ID_RAFT_FOLLOWER_LOCAL_MUTATION,
                RULE_ID_RAFT_LOSS_OF_LEADER_FAIL_OPEN,
            ]
        );
        for doc in &docs {
            assert!(
                doc.family == "obs" || doc.family == "raft",
                "unexpected family {} for rule id {}",
                doc.family,
                doc.id
            );
            assert!(!doc.description.is_empty());
        }
        assert_eq!(docs.iter().filter(|d| d.family == "obs").count(), 2);
        assert_eq!(docs.iter().filter(|d| d.family == "raft").count(), 6);
    }
}
// HANDWRITE-END
