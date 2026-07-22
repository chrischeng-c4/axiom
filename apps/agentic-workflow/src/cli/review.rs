// HANDWRITE-BEGIN gap="missing-generator:logic:0f85d405" tracker="pending-tracker" reason="New module: `ProjectProfile` model (kind_surface, primary_workload, state_ownership, replication, serving_role dimensions, each carrying an explicit Unknown/Ambiguous variant per R1), `resolve_project_profile(project)` walking the evidence-gathering decision flow described in `## Logic` (aw.toml `[capability.profile].traits` from the #1546 workload-profile trait registry, Dockerfile/k8s manifest presence, `Cargo.toml` dependency graph, source-module naming conventions), plus `ReviewArgs`/`ReviewReport` and `run_review(args)` wiring the read-only `aw review --project <project>` verb's stdout envelope (resolved profile + evidence, or an explicit ambiguous-profile finding) following the runnable-or-terminal stdout contract. Hand-written: evidence-signal classification across five reference profiles is domain judgment no existing generator primitive covers yet (gap: project-profile-evidence-classifier, tracker: #2165, reason: no generator primitive maps aw.toml/source-tree evidence signals to a project-profile classification decision tree yet)."
//! `aw review` -- read-only project-profile resolution + report skeleton.
//!
//! Foundation slice for the existing-project-standardization capability's
//! `project-profile-conformance-review` gap (issue #2165, epic follow-ups
//! #2166/#2167/#2169). Resolves an orthogonal `ProjectProfile` -- kind/
//! surface, primary workload, state ownership, replication/consensus, and
//! serving role -- from real evidence (a project's own `aw.toml`
//! `[capability.profile].traits`, its `Cargo.toml` dependency graph, and
//! source-tree naming-convention markers), deliberately without conflating
//! `service-archetype`/`CapabilityType` (see
//! `crate::cli::capability_type::CapabilityType`, the EC-dimension ceiling
//! for a *capability*, and the #1546 workload-profile-trait baseline
//! derivation in `crate::cli::capability`, which drives generated
//! `CONTRIBUTING.md` obligations). Every dimension can resolve `Unknown`,
//! and the whole result can resolve `Ambiguous`, rather than guessing --
//! see `## Logic` in
//! `apps/agentic-workflow/tech-design/config/aw-review-resolve-project-profile-cli-report-skeleton.md`
//! for the decision flow this module implements.
//!
//! Rule findings, observability/Raft telemetry checks, and the `aw:review`
//! skill are explicitly out of scope for this slice (child WIs #2166,
//! #2167, #2169) -- this module only resolves and reports the profile.
//!
//! @spec apps/agentic-workflow/tech-design/config/aw-review-resolve-project-profile-cli-report-skeleton.md

use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// `aw review --project <project>` -- read-only, emits the resolved/
/// effective project profile plus evidence (or an explicit
/// ambiguous-profile finding). Never mutates the filesystem or a tracker.
///
/// @spec apps/agentic-workflow/tech-design/config/aw-review-resolve-project-profile-cli-report-skeleton.md#logic
#[derive(Debug, Args, Clone)]
#[command(
    after_help = r#"Read-only. Resolves an orthogonal project-profile (kind/surface, primary
workload, state ownership, replication/consensus, serving role) from a
project's own `aw.toml` capability-profile traits, `Cargo.toml` dependency
graph, and source-tree naming-convention markers -- never a hardcoded
per-project-name lookup. A profile that cannot be determined from evidence
resolves `Ambiguous` (with the evidence collected so far attached) rather
than guessing.

Output schema (JSON, aw.cli.v1):
{
  "schema_version": "aw.cli.v1",
  "event": "result",
  "status": "done",
  "action": "review",
  "project": string,
  "outcome": "resolved" | "ambiguous",
  "profile": { "kind_surface", "primary_workload", "state_ownership", "replication", "serving_role" },
  "ambiguous_reason": string | null,
  "evidence": [ { "source": string, "detail": string } ],
  "completion": { "workflow_complete": true, "requires_hitl": false },
  "next": { "kind": "done", "reason": string }
}

Rule findings, observability/Raft telemetry checks, and adoption of the
`aw:review` skill are out of scope for this verb (separate child work,
#2166/#2167/#2169); this is the profile-resolution foundation only."#
)]
pub struct ReviewArgs {
    /// Configured project name from `[[projects]]` in `aw.toml`.
    #[arg(long)]
    pub project: String,
    /// Pretty-print the JSON envelope.
    #[arg(long)]
    pub pretty: bool,
}

// ---------------------------------------------------------------------
// R1: orthogonal project-profile model. Deliberately does not reuse or
// conflate `service-archetype`/`CapabilityType` -- see module doc above.
// Every dimension carries an explicit `Unknown` (applicable but
// undetermined) or `NotApplicable` (this profile shape does not populate
// this dimension, e.g. a `Cli` profile has no workload/replication/
// serving-role) outcome; a guessed default is never produced.
// ---------------------------------------------------------------------

/// Whether the project exposes a served surface at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KindSurface {
    /// No served surface: a pure CLI/tool project.
    Cli,
    /// Exposes a served surface (HTTP/h2c/etc.).
    Service,
}

/// The workload shape a served surface runs as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryWorkload {
    /// `Cli` profiles have no workload dimension.
    NotApplicable,
    /// Stateless/externally-stated service workload.
    Deployment,
    /// Stateful, identity-carrying service workload.
    StatefulSet,
}

/// Who owns durable state for a `Deployment`/`StatefulSet` workload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateOwnership {
    NotApplicable,
    /// State lives in an external system (Pgpool-like proxy/pool).
    ExternalState,
    /// The project itself owns durable state.
    OwnedState,
    /// Applicable but not determinable from current evidence.
    Unknown,
}

/// Replication/consensus shape for a state-owning workload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationConsensus {
    NotApplicable,
    /// No replication (e.g. a stateless external-state Deployment).
    None,
    /// Primary-write/replica-read topology (Lumen-like).
    PrimaryReplica,
    /// Replicated ordered log with checkpoints (Tape-like).
    ReplicatedLog,
    /// Raft-coordinated consensus (Relay/Defer-like).
    RaftConsensus,
    Unknown,
}

/// Serving-role shape layered on top of the replication dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServingRole {
    NotApplicable,
    /// No distinguished write/read role split.
    Standard,
    /// Primary accepts writes, replicas serve reads.
    PrimaryWriteReplicaRead,
    /// All writes must reach the leader (raft leader-ingest).
    LeaderIngest,
    Unknown,
}

/// The five orthogonal dimensions resolved for one project.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectProfile {
    pub kind_surface: KindSurface,
    pub primary_workload: PrimaryWorkload,
    pub state_ownership: StateOwnership,
    pub replication: ReplicationConsensus,
    pub serving_role: ServingRole,
}

impl ProjectProfile {
    fn cli() -> Self {
        ProjectProfile {
            kind_surface: KindSurface::Cli,
            primary_workload: PrimaryWorkload::NotApplicable,
            state_ownership: StateOwnership::NotApplicable,
            replication: ReplicationConsensus::NotApplicable,
            serving_role: ServingRole::NotApplicable,
        }
    }

    fn deployment_external_state() -> Self {
        ProjectProfile {
            kind_surface: KindSurface::Service,
            primary_workload: PrimaryWorkload::Deployment,
            state_ownership: StateOwnership::ExternalState,
            replication: ReplicationConsensus::None,
            serving_role: ServingRole::Standard,
        }
    }

    fn deployment_unknown_state() -> Self {
        ProjectProfile {
            kind_surface: KindSurface::Service,
            primary_workload: PrimaryWorkload::Deployment,
            state_ownership: StateOwnership::Unknown,
            replication: ReplicationConsensus::Unknown,
            serving_role: ServingRole::Unknown,
        }
    }

    fn stateful_set_raft_consensus() -> Self {
        ProjectProfile {
            kind_surface: KindSurface::Service,
            primary_workload: PrimaryWorkload::StatefulSet,
            state_ownership: StateOwnership::OwnedState,
            replication: ReplicationConsensus::RaftConsensus,
            serving_role: ServingRole::LeaderIngest,
        }
    }

    fn stateful_set_replicated_log() -> Self {
        ProjectProfile {
            kind_surface: KindSurface::Service,
            primary_workload: PrimaryWorkload::StatefulSet,
            state_ownership: StateOwnership::OwnedState,
            replication: ReplicationConsensus::ReplicatedLog,
            serving_role: ServingRole::Standard,
        }
    }

    fn stateful_set_primary_replica() -> Self {
        ProjectProfile {
            kind_surface: KindSurface::Service,
            primary_workload: PrimaryWorkload::StatefulSet,
            state_ownership: StateOwnership::OwnedState,
            replication: ReplicationConsensus::PrimaryReplica,
            serving_role: ServingRole::PrimaryWriteReplicaRead,
        }
    }

    fn stateful_set_unknown_replication() -> Self {
        ProjectProfile {
            kind_surface: KindSurface::Service,
            primary_workload: PrimaryWorkload::StatefulSet,
            state_ownership: StateOwnership::OwnedState,
            replication: ReplicationConsensus::Unknown,
            serving_role: ServingRole::Unknown,
        }
    }
}

/// One cited evidence item backing a resolution (or the collected evidence
/// attached to an `Ambiguous` finding). `source` names where the signal
/// came from (an `aw.toml` field, a `Cargo.toml` dependency, or a
/// repo-relative source-file path); `detail` is the human-readable finding.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileEvidence {
    pub source: String,
    pub detail: String,
}

fn evidence(source: impl Into<String>, detail: impl Into<String>) -> ProfileEvidence {
    ProfileEvidence {
        source: source.into(),
        detail: detail.into(),
    }
}

/// The outcome of `resolve_project_profile`: either a resolved profile, or
/// an explicit ambiguous finding -- never a guessed default. Both variants
/// carry the full evidence trail that produced them.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ProfileResolution {
    Resolved {
        profile: ProjectProfile,
        evidence: Vec<ProfileEvidence>,
    },
    Ambiguous {
        reason: String,
        /// Best-effort partial profile: dimensions the evidence did
        /// determine keep their resolved value; contradictory or
        /// undetermined dimensions are `Unknown`.
        profile: ProjectProfile,
        evidence: Vec<ProfileEvidence>,
    },
}

// ---------------------------------------------------------------------
// Evidence gathering
// ---------------------------------------------------------------------

/// Known `[capability.profile].traits` ids relevant to profile resolution
/// (the #1546 workload-profile trait registry; see
/// `crate::cli::capability::known_capability_profile_traits` for the full
/// registry this is a resolution-relevant subset of).
const NETWORK_SURFACE_TRAITS: &[&str] = &["network_exposed", "service", "http2_api"];
const STATEFUL_WORKLOAD_TRAITS: &[&str] = &["stateful_storage", "primary_replicas"];
pub(crate) const PRIMARY_REPLICAS_TRAIT: &str = "primary_replicas";

/// `Cargo.toml` dependency names that indicate a served HTTP/h2c surface.
const SURFACE_DEPENDENCIES: &[&str] = &["service-http", "axum", "hyper", "tonic"];
/// `Cargo.toml` dependency names that indicate the project itself owns
/// durable state (a database client, a durable-storage lib, or a raft
/// replication engine -- raft implies the project replicates its own
/// state, which is owned state even before any StatefulSet-shaped
/// replication/serving-role signal is found).
const STATE_DEPENDENCIES: &[&str] = &[
    "sqlx",
    "tokio-postgres",
    "postgres",
    "rusqlite",
    "diesel",
    "storage-durable",
    "raft-core",
    "raft-runtime",
    "raft-host",
];
/// `Cargo.toml` dependency names for a raft replication engine (the
/// `raft-host` evidence source named in `## Logic`; this repo's shared lib
/// is named `raft-core`/`raft-runtime`, not literally `raft-host`).
pub(crate) const RAFT_DEPENDENCIES: &[&str] = &["raft-core", "raft-runtime", "raft-host"];

#[derive(Debug, Default, Clone)]
pub(crate) struct MarkerHit {
    pub(crate) marker: &'static str,
    pub(crate) path: String,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SourceMarkers {
    pub(crate) checkpoint: bool,
    pub(crate) segment: bool,
    pub(crate) wal: bool,
    pub(crate) leader_ingest: bool,
    pub(crate) primary_replica_role: bool,
    pub(crate) hits: Vec<MarkerHit>,
}

#[derive(Debug, Default, Clone)]
struct ProjectEvidence {
    traits: Vec<String>,
    cargo_dependencies: Vec<String>,
    has_dockerfile_or_manifest: bool,
    markers: SourceMarkers,
}

fn normalize_trait(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

/// Read `<project_dir>/aw.toml`'s `[capability.profile].traits`, if any.
/// Absence is not an error -- most pure-CLI projects declare no traits at
/// all, which is itself evidence (see `resolve_project_profile_for_dir`).
pub(crate) fn read_project_traits(project_dir: &Path) -> Vec<String> {
    #[derive(Debug, Default, serde::Deserialize)]
    struct Document {
        #[serde(default)]
        capability: Capability,
    }
    #[derive(Debug, Default, serde::Deserialize)]
    struct Capability {
        #[serde(default)]
        profile: Profile,
    }
    #[derive(Debug, Default, serde::Deserialize)]
    struct Profile {
        #[serde(default)]
        traits: Vec<String>,
    }

    let path = project_dir.join("aw.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(doc) = toml::from_str::<Document>(&content) else {
        return Vec::new();
    };
    let mut traits: Vec<String> = doc
        .capability
        .profile
        .traits
        .iter()
        .map(|t| normalize_trait(t))
        .filter(|t| !t.is_empty())
        .collect();
    traits.sort();
    traits.dedup();
    traits
}

/// Read `<project_dir>/Cargo.toml`'s `[dependencies]`/`[dev-dependencies]`
/// keys. Missing `Cargo.toml` (e.g. a `target = "schemas"` project) is not
/// an error -- an empty dependency graph.
pub(crate) fn read_cargo_dependencies(project_dir: &Path) -> Vec<String> {
    let path = project_dir.join("Cargo.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(value) = content.parse::<toml::Value>() else {
        return Vec::new();
    };
    let mut deps = Vec::new();
    for table_name in ["dependencies", "dev-dependencies"] {
        if let Some(table) = value.get(table_name).and_then(toml::Value::as_table) {
            deps.extend(table.keys().map(|k| k.to_ascii_lowercase()));
        }
    }
    deps.sort();
    deps.dedup();
    deps
}

/// Coarse Dockerfile/k8s-manifest presence check: top-level `Dockerfile*`,
/// or a `k8s`/`deploy`/`kubernetes` directory containing any file. Several
/// service CLIs in this ecosystem render these on demand (`<cli> dockerfile
/// render` / `<cli> k8s ... render`) rather than committing them, so this
/// is auxiliary evidence, not the sole workload-kind signal (traits carry
/// that -- see `resolve_project_profile_for_dir`).
pub(crate) fn has_dockerfile_or_manifest(project_dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(project_dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("Dockerfile") {
            return true;
        }
        if matches!(name.as_ref(), "k8s" | "deploy" | "kubernetes")
            && entry.path().is_dir()
            && std::fs::read_dir(entry.path())
                .map(|mut it| it.next().is_some())
                .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

/// Scan `<project_dir>/src/**/*.rs` for the source-module naming-convention
/// markers named in `## Logic`: replicated-log/checkpoint markers
/// (`checkpoint`, `segment`, `wal`), leader-ingest markers (`leader` +
/// `ingest` co-occurring in one file), and primary/replica role markers
/// (`primary` + `replica` co-occurring in one file). Bounded to the `src/`
/// tree only (never `target/`, `tests/`, `benches/`) and to a scan-not-
/// parse substring pass -- this is domain judgment, not a generator
/// primitive (see the module-level HANDWRITE reason).
pub(crate) fn scan_source_markers(project_dir: &Path) -> SourceMarkers {
    let src_root = project_dir.join("src");
    let mut markers = SourceMarkers::default();
    if !src_root.is_dir() {
        return markers;
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
        let rel = path
            .strip_prefix(project_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();

        let has_checkpoint = lower.contains("checkpoint");
        let has_segment = lower.contains("segment");
        let has_wal = lower.contains("struct wal") || lower.contains("mod wal");
        let has_leader_ingest = lower.contains("leader_ingest")
            || lower.contains("leaderingest")
            || (lower.contains("leader") && lower.contains("ingest"));
        let has_primary_replica = lower.contains("primary") && lower.contains("replica");

        if has_checkpoint {
            markers.checkpoint = true;
            markers.hits.push(MarkerHit {
                marker: "checkpoint",
                path: rel.clone(),
            });
        }
        if has_segment {
            markers.segment = true;
            markers.hits.push(MarkerHit {
                marker: "segment",
                path: rel.clone(),
            });
        }
        if has_wal {
            markers.wal = true;
            markers.hits.push(MarkerHit {
                marker: "wal",
                path: rel.clone(),
            });
        }
        if has_leader_ingest {
            markers.leader_ingest = true;
            markers.hits.push(MarkerHit {
                marker: "leader_ingest",
                path: rel.clone(),
            });
        }
        if has_primary_replica {
            markers.primary_replica_role = true;
            markers.hits.push(MarkerHit {
                marker: "primary_replica_role",
                path: rel,
            });
        }
    }
    markers
}

fn gather_evidence(project_dir: &Path) -> ProjectEvidence {
    ProjectEvidence {
        traits: read_project_traits(project_dir),
        cargo_dependencies: read_cargo_dependencies(project_dir),
        has_dockerfile_or_manifest: has_dockerfile_or_manifest(project_dir),
        markers: scan_source_markers(project_dir),
    }
}

fn any_dependency(deps: &[String], candidates: &[&str]) -> Vec<String> {
    deps.iter()
        .filter(|d| candidates.contains(&d.as_str()))
        .cloned()
        .collect()
}

fn any_trait(traits: &[String], candidates: &[&str]) -> Vec<String> {
    traits
        .iter()
        .filter(|t| candidates.contains(&t.as_str()))
        .cloned()
        .collect()
}

fn marker_hits_for(markers: &SourceMarkers, marker: &str, limit: usize) -> Vec<String> {
    markers
        .hits
        .iter()
        .filter(|hit| hit.marker == marker)
        .map(|hit| hit.path.clone())
        .take(limit)
        .collect()
}

/// Preview limit for source-marker evidence citations, so a project with
/// many matching files doesn't produce an unbounded evidence list.
const MARKER_EVIDENCE_LIMIT: usize = 5;

// ---------------------------------------------------------------------
// Resolution: walks the `## Logic` decision flow.
// ---------------------------------------------------------------------

/// Resolve the project row for `project` under `project_root`'s `aw.toml`
/// registry, then resolve its profile from real project-tree evidence.
///
/// @spec apps/agentic-workflow/tech-design/config/aw-review-resolve-project-profile-cli-report-skeleton.md#logic
pub fn resolve_project_profile(project_root: &Path, project: &str) -> Result<ProfileResolution> {
    let project_dir = project_dir_for(project_root, project)?;
    Ok(resolve_project_profile_for_dir(&project_dir))
}

/// Resolve `project`'s configured directory under `project_root`'s `aw.toml`
/// registry, without resolving its profile. Shared by `resolve_project_profile`
/// and `run_review` (#2166) so both `review_rules::apply_conformance_rules`
/// and profile resolution walk the exact same project-root evidence tree.
pub(crate) fn project_dir_for(project_root: &Path, project: &str) -> Result<PathBuf> {
    let row = crate::services::project_registry::resolve_project_config_row(project_root, project)
        .with_context(|| format!("resolving project row for `{project}`"))?;
    let path = PathBuf::from(&row.path);
    Ok(if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    })
}

/// The evidence-gathering + decision-tree core, directly testable against a
/// constructed project directory fixture (no `aw.toml` project registry
/// needed). See `## Logic` in this module's owning TD for the flowchart
/// this function implements.
pub(crate) fn resolve_project_profile_for_dir(project_dir: &Path) -> ProfileResolution {
    let ev = gather_evidence(project_dir);
    let mut log = Vec::new();

    log.push(if ev.traits.is_empty() {
        evidence(
            "aw.toml:capability.profile.traits",
            "no traits declared (or no project-local aw.toml found)",
        )
    } else {
        evidence(
            "aw.toml:capability.profile.traits",
            format!("traits = [{}]", ev.traits.join(", ")),
        )
    });

    let surface_traits = any_trait(&ev.traits, NETWORK_SURFACE_TRAITS);
    let surface_deps = any_dependency(&ev.cargo_dependencies, SURFACE_DEPENDENCIES);
    let has_surface =
        !surface_traits.is_empty() || !surface_deps.is_empty() || ev.has_dockerfile_or_manifest;

    if !surface_traits.is_empty() {
        log.push(evidence(
            "aw.toml:capability.profile.traits",
            format!("served-surface traits: {}", surface_traits.join(", ")),
        ));
    }
    if !surface_deps.is_empty() {
        log.push(evidence(
            "Cargo.toml:dependencies",
            format!("served-surface dependencies: {}", surface_deps.join(", ")),
        ));
    }
    log.push(evidence(
        "filesystem:Dockerfile/k8s-manifest",
        if ev.has_dockerfile_or_manifest {
            "Dockerfile or k8s manifest present"
        } else {
            "no Dockerfile or k8s manifest found"
        },
    ));

    if !has_surface {
        return ProfileResolution::Resolved {
            profile: ProjectProfile::cli(),
            evidence: log,
        };
    }

    let stateful_traits = any_trait(&ev.traits, STATEFUL_WORKLOAD_TRAITS);
    let is_stateful_workload = !stateful_traits.is_empty();
    if !stateful_traits.is_empty() {
        log.push(evidence(
            "aw.toml:capability.profile.traits",
            format!("StatefulSet-shaped traits: {}", stateful_traits.join(", ")),
        ));
    }

    if !is_stateful_workload {
        // Deployment branch: does the project own durable state despite
        // being Deployment-shaped? A positive hit here is contradictory --
        // report Ambiguous with the evidence collected so far rather than
        // guessing which signal is stale.
        let state_deps = any_dependency(&ev.cargo_dependencies, STATE_DEPENDENCIES);
        if !state_deps.is_empty() {
            log.push(evidence(
                "Cargo.toml:dependencies",
                format!(
                    "owned-state dependencies on a Deployment-shaped project: {}",
                    state_deps.join(", ")
                ),
            ));
            return ProfileResolution::Ambiguous {
                reason: "Deployment workload with owned-state evidence (contradictory signals)"
                    .to_string(),
                profile: ProjectProfile::deployment_unknown_state(),
                evidence: log,
            };
        }
        log.push(evidence(
            "Cargo.toml:dependencies",
            "no owned-state dependency found",
        ));
        return ProfileResolution::Resolved {
            profile: ProjectProfile::deployment_external_state(),
            evidence: log,
        };
    }

    // StatefulSet branch.
    let raft_deps = any_dependency(&ev.cargo_dependencies, RAFT_DEPENDENCIES);
    let leader_ingest_files = marker_hits_for(&ev.markers, "leader_ingest", MARKER_EVIDENCE_LIMIT);
    if !raft_deps.is_empty() && !leader_ingest_files.is_empty() {
        log.push(evidence(
            "Cargo.toml:dependencies",
            format!("raft replication dependency: {}", raft_deps.join(", ")),
        ));
        log.push(evidence(
            "src/**/*.rs",
            format!(
                "leader-ingest markers in: {}",
                leader_ingest_files.join(", ")
            ),
        ));
        return ProfileResolution::Resolved {
            profile: ProjectProfile::stateful_set_raft_consensus(),
            evidence: log,
        };
    }

    let checkpoint_files = marker_hits_for(&ev.markers, "checkpoint", MARKER_EVIDENCE_LIMIT);
    let segment_files = marker_hits_for(&ev.markers, "segment", MARKER_EVIDENCE_LIMIT);
    let wal_files = marker_hits_for(&ev.markers, "wal", MARKER_EVIDENCE_LIMIT);
    let has_log_signal =
        !checkpoint_files.is_empty() || (!segment_files.is_empty() && !wal_files.is_empty());
    if has_log_signal {
        if !checkpoint_files.is_empty() {
            log.push(evidence(
                "src/**/*.rs",
                format!("checkpoint markers in: {}", checkpoint_files.join(", ")),
            ));
        }
        if !segment_files.is_empty() {
            log.push(evidence(
                "src/**/*.rs",
                format!("segment markers in: {}", segment_files.join(", ")),
            ));
        }
        if !wal_files.is_empty() {
            log.push(evidence(
                "src/**/*.rs",
                format!("wal markers in: {}", wal_files.join(", ")),
            ));
        }
        return ProfileResolution::Resolved {
            profile: ProjectProfile::stateful_set_replicated_log(),
            evidence: log,
        };
    }

    let primary_replicas_trait = ev.traits.iter().any(|t| t == PRIMARY_REPLICAS_TRAIT);
    let role_files = marker_hits_for(&ev.markers, "primary_replica_role", MARKER_EVIDENCE_LIMIT);
    if primary_replicas_trait && !role_files.is_empty() {
        log.push(evidence(
            "src/**/*.rs",
            format!("primary/replica role markers in: {}", role_files.join(", ")),
        ));
        return ProfileResolution::Resolved {
            profile: ProjectProfile::stateful_set_primary_replica(),
            evidence: log,
        };
    }

    log.push(evidence(
        "src/**/*.rs",
        "no recognized raft/log/replica-role signal found for this StatefulSet-shaped project",
    ));
    ProfileResolution::Ambiguous {
        reason: "StatefulSet workload with no recognized replication/serving signal".to_string(),
        profile: ProjectProfile::stateful_set_unknown_replication(),
        evidence: log,
    }
}

// ---------------------------------------------------------------------
// R3: CLI wiring.
// ---------------------------------------------------------------------

// <HANDWRITE gap="missing-generator:logic" tracker="#2166" reason="logic section in review.rs is hand-written pending codegen support">
/// The full report `run_review` resolves and renders: the #2165 profile
/// resolution plus this WI's (#2166) additive `findings` list from
/// `review_rules::apply_conformance_rules`.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewReport {
    pub project: String,
    #[serde(flatten)]
    pub resolution: ProfileResolution,
    /// Shared-service-kit adoption + profile negative-assertion findings
    /// (#2166). Always present, empty when there are none.
    pub findings: Vec<crate::cli::review_rules::Finding>,
}
// </HANDWRITE>

pub(crate) fn review_envelope(report: &ReviewReport) -> serde_json::Value {
    let (outcome, ambiguous_reason) = match &report.resolution {
        ProfileResolution::Resolved { .. } => ("resolved", None),
        ProfileResolution::Ambiguous { reason, .. } => ("ambiguous", Some(reason.clone())),
    };
    let (profile, evidence) = match &report.resolution {
        ProfileResolution::Resolved { profile, evidence } => (profile, evidence),
        ProfileResolution::Ambiguous {
            profile, evidence, ..
        } => (profile, evidence),
    };
    serde_json::json!({
        "schema_version": "aw.cli.v1",
        "event": "result",
        "status": "done",
        "action": "review",
        "project": &report.project,
        "outcome": outcome,
        "profile": profile,
        "ambiguous_reason": ambiguous_reason,
        "evidence": evidence,
        "findings": &report.findings,
        "completion": { "workflow_complete": true, "requires_hitl": false },
        "next": {
            "kind": "done",
            "reason": "profile resolution and rule findings reported; observability/Raft telemetry conformance and the `aw:review` skill/doc projection are separate follow-up work (#2167/#2169)",
        },
    })
}

/// Run `aw review --project <project>`. Read-only: resolves the project's
/// profile from evidence, applies the #2166 shared-service-kit adoption +
/// profile negative-assertion rule matrix, and prints the `aw.cli.v1`
/// result envelope.
///
/// @spec apps/agentic-workflow/tech-design/config/aw-review-resolve-project-profile-cli-report-skeleton.md#logic
/// @spec apps/agentic-workflow/tech-design/config/aw-review-shared-service-kit-adoption-rules-profile-negative-ass.md#logic
pub fn run_review(args: ReviewArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let project_dir = project_dir_for(&project_root, &args.project)?;
    let resolution = resolve_project_profile_for_dir(&project_dir);
    let findings = crate::cli::review_rules::apply_conformance_rules(&project_dir, &resolution);
    let report = ReviewReport {
        project: args.project,
        resolution,
        findings,
    };
    let envelope = review_envelope(&report);
    if args.pretty {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("{}", serde_json::to_string(&envelope)?);
    }
    Ok(())
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

    // R1: a pure-CLI project fixture (aw/jet/mamba-shaped tree with no
    // Dockerfile/k8s manifest/service surface) resolves to
    // Profile{kind_surface: Cli} with no workload/replication/serving-role
    // dimensions populated.
    #[test]
    fn test_resolve_cli_profile_fixture() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["clap", "anyhow"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn hello() {}\n");

        let resolution = resolve_project_profile_for_dir(tmp.path());
        match resolution {
            ProfileResolution::Resolved { profile, .. } => {
                assert_eq!(profile.kind_surface, KindSurface::Cli);
                assert_eq!(profile.primary_workload, PrimaryWorkload::NotApplicable);
                assert_eq!(profile.replication, ReplicationConsensus::NotApplicable);
                assert_eq!(profile.serving_role, ServingRole::NotApplicable);
            }
            other => panic!("expected Resolved(Cli), got {other:?}"),
        }
    }

    // R2 (Pgpool-like): a stateless/external-state Deployment fixture
    // (served surface, Deployment manifest/traits, no PVC/raft-host/db
    // dependency) resolves to Profile{workload: Deployment,
    // state: ExternalState}.
    #[test]
    fn test_resolve_deployment_external_state_fixture() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed", "kubernetes_native"]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http", "clap"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn pool() {}\n");

        let resolution = resolve_project_profile_for_dir(tmp.path());
        match resolution {
            ProfileResolution::Resolved { profile, .. } => {
                assert_eq!(profile.kind_surface, KindSurface::Service);
                assert_eq!(profile.primary_workload, PrimaryWorkload::Deployment);
                assert_eq!(profile.state_ownership, StateOwnership::ExternalState);
                assert_eq!(profile.replication, ReplicationConsensus::None);
            }
            other => panic!("expected Resolved(Deployment/ExternalState), got {other:?}"),
        }
    }

    // R3 (Lumen-like): a primary-write/replica-read fixture (StatefulSet
    // traits, primary_replicas trait, primary/replica role markers)
    // resolves to Profile{workload: StatefulSet, replication:
    // PrimaryReplica}.
    #[test]
    fn test_resolve_primary_replica_fixture() {
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
            &cargo_toml(&["service-http"]),
        );
        write(
            &tmp.path().join("src/routing.rs"),
            "// Routes writes to the Primary and reads to a Replica.\npub struct Replica;\n",
        );

        let resolution = resolve_project_profile_for_dir(tmp.path());
        match resolution {
            ProfileResolution::Resolved { profile, .. } => {
                assert_eq!(profile.primary_workload, PrimaryWorkload::StatefulSet);
                assert_eq!(profile.replication, ReplicationConsensus::PrimaryReplica);
                assert_eq!(profile.serving_role, ServingRole::PrimaryWriteReplicaRead);
            }
            other => panic!("expected Resolved(PrimaryReplica), got {other:?}"),
        }
    }

    // R5 (Relay/Defer-like): a leader-ingest/Raft-coordinated fixture
    // (StatefulSet traits, raft-host-shaped Cargo dependency, leader-ingest
    // source markers) resolves to Profile{workload: StatefulSet,
    // replication: RaftConsensus, serving_role: LeaderIngest}.
    #[test]
    fn test_resolve_raft_leader_ingest_fixture() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed", "stateful_storage"]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http", "raft-runtime"]),
        );
        write(
            &tmp.path().join("src/ingest.rs"),
            "// All publishes are forwarded to the leader (leader_ingest): only the\n// leader accepts writes.\npub fn leader_ingest() {}\n",
        );

        let resolution = resolve_project_profile_for_dir(tmp.path());
        match resolution {
            ProfileResolution::Resolved { profile, .. } => {
                assert_eq!(profile.primary_workload, PrimaryWorkload::StatefulSet);
                assert_eq!(profile.replication, ReplicationConsensus::RaftConsensus);
                assert_eq!(profile.serving_role, ServingRole::LeaderIngest);
            }
            other => panic!("expected Resolved(RaftConsensus/LeaderIngest), got {other:?}"),
        }
    }

    // R4 (Tape-like): a replicated-ordered-log-with-checkpoints fixture
    // (StatefulSet traits, segment/checkpoint source markers, no raft
    // dependency) resolves to Profile{workload: StatefulSet, replication:
    // ReplicatedLog}.
    #[test]
    fn test_resolve_replicated_log_fixture() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed", "stateful_storage"]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http"]),
        );
        write(
            &tmp.path().join("src/lib.rs"),
            "pub struct ConsumerCheckpoint;\npub fn put_checkpoint() {}\n",
        );

        let resolution = resolve_project_profile_for_dir(tmp.path());
        match resolution {
            ProfileResolution::Resolved { profile, .. } => {
                assert_eq!(profile.primary_workload, PrimaryWorkload::StatefulSet);
                assert_eq!(profile.replication, ReplicationConsensus::ReplicatedLog);
            }
            other => panic!("expected Resolved(ReplicatedLog), got {other:?}"),
        }
    }

    // R6: a fixture with contradictory or insufficient evidence (here: a
    // StatefulSet-shaped manifest with no recognized raft/log/replica-role
    // signal) resolves to an explicit Ambiguous finding carrying the
    // collected evidence, never a guessed default profile.
    #[test]
    fn test_resolve_ambiguous_profile_fixture() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed", "stateful_storage"]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = resolve_project_profile_for_dir(tmp.path());
        match resolution {
            ProfileResolution::Ambiguous {
                reason, evidence, ..
            } => {
                assert!(reason.contains("no recognized replication/serving signal"));
                assert!(!evidence.is_empty());
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    // A Deployment-shaped project (no StatefulSet-signaling traits) that
    // nonetheless carries owned-state Cargo evidence is contradictory, not
    // guessable -- also resolves Ambiguous.
    #[test]
    fn test_resolve_deployment_contradictory_state_is_ambiguous() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed", "kubernetes_native"]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http", "sqlx"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let resolution = resolve_project_profile_for_dir(tmp.path());
        match resolution {
            ProfileResolution::Ambiguous { reason, .. } => {
                assert!(reason.contains("contradictory"));
            }
            other => panic!("expected Ambiguous(contradictory), got {other:?}"),
        }
    }

    // R7: `aw review --project <project>` is read-only (no filesystem/
    // tracker mutation) and its stdout envelope satisfies this repo's
    // runnable-next-command-or-terminal-marker convention: every result
    // carries `completion.workflow_complete` and a `next.kind` terminal
    // marker (`"done"`).
    #[test]
    fn test_review_cli_read_only_stdout_contract() {
        let tmp = tempfile::tempdir().unwrap();
        write(&tmp.path().join("Cargo.toml"), &cargo_toml(&["clap"]));
        write(&tmp.path().join("src/lib.rs"), "pub fn hello() {}\n");
        let before = fs::read_dir(tmp.path()).unwrap().count();

        let report = ReviewReport {
            project: "fixture".to_string(),
            resolution: resolve_project_profile_for_dir(tmp.path()),
            findings: Vec::new(),
        };
        let envelope = review_envelope(&report);

        // Read-only: resolving the profile must not have written anything
        // to the fixture tree.
        assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), before);

        assert_eq!(envelope["schema_version"], "aw.cli.v1");
        assert_eq!(envelope["action"], "review");
        assert_eq!(envelope["completion"]["workflow_complete"], true);
        assert_eq!(envelope["next"]["kind"], "done");
        assert!(envelope["next"]["command"].is_null());
        assert_eq!(envelope["outcome"], "resolved");
        assert!(envelope["evidence"].is_array());
    }

    #[test]
    fn test_review_envelope_reports_ambiguous_outcome() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            &tmp.path().join("aw.toml"),
            &aw_toml(&["service", "network_exposed", "stateful_storage"]),
        );
        write(
            &tmp.path().join("Cargo.toml"),
            &cargo_toml(&["service-http"]),
        );
        write(&tmp.path().join("src/lib.rs"), "pub fn serve() {}\n");

        let report = ReviewReport {
            project: "fixture".to_string(),
            resolution: resolve_project_profile_for_dir(tmp.path()),
            findings: Vec::new(),
        };
        let envelope = review_envelope(&report);

        assert_eq!(envelope["outcome"], "ambiguous");
        assert!(envelope["ambiguous_reason"].is_string());
        assert_eq!(envelope["completion"]["workflow_complete"], true);
        assert_eq!(envelope["next"]["kind"], "done");
    }
}
// HANDWRITE-END
