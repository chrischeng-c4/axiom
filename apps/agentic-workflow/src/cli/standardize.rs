// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
// CODEGEN-BEGIN
//! `aw standardize` — existing-project workflow guidance and bounded remediation.

use anyhow::{bail, Context, Result};
use clap::{Args, CommandFactory, Parser, Subcommand};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::process::Command;

#[path = "standardize_audit.rs"]
mod standardize_audit;

const SOURCE_EXTS: &[&str] = &[
    "rs", "py", "js", "jsx", "mjs", "cjs", "ts", "tsx", "go", "json", "css", "scss",
];
const PROJECT_CONTEXT_ARTIFACTS: &[&str] = &["llms.txt"];
const RUST_BINARY_ARTIFACTS: &[&str] = &["build.sh", "install.sh"];
const EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".aw",
    "target",
    "node_modules",
    "dist",
    "build",
    ".venv",
    "vendor",
    "vendors",
    "generated",
    "__generated__",
];
const DELETED_COMMAND_PATHS: &[&str] = &[
    "aw run-change",
    "aw workflow",
    "aw revise-artifact",
    "aw artifact",
    "aw validate-spec-structure",
    "aw check-alignment",
    "aw iss ",
    "aw issues",
    "aw chat agents",
    "aw handoff",
    "aw takeoff",
    // #918: `aw run` full removal (superseded by `aw wi run` / `aw capability
    // run`, #917) + #857 merge-era leftovers. `"aw cb "` keeps the trailing
    // space so it does not false-positive on prose like "the aw cb_filled
    // phase" while still catching a literal `aw cb ...` invocation.
    "aw run",
    "aw td merge",
    "aw td review",
    "aw td revise",
    "aw wi merge",
    "aw cb ",
    // #920 (epic #914 slice F): `aw standardize` is retired down to `audit`
    // only; the `managed`/`semantic`/`traceability` layer `report`/`next`/
    // `run` drivers are gone.
    "aw standardize managed",
    "aw standardize semantic",
    "aw standardize traceability",
    // #1273 (epic #1270 R5): `aw td code-claim` folded into
    // `aw td create --from-source`.
    "aw td code-claim",
];
const AW_EC_BEGIN_MARKER: &str = "AW-EC-BEGIN";

#[derive(Parser)]
#[command(name = "aw")]
// Promoted to `pub(crate)` so `crate::cli::chain::validate_aw_command_string`
// (#915) can reuse this as the single full clap tree to validate emitted
// next-command strings against, instead of redeclaring a second wrapper.
pub(crate) struct TraceabilityCli {
    #[command(subcommand)]
    command: crate::cli::commands::Commands,
}

#[derive(Debug, Args)]
// #920 (epic #914 slice F): `aw standardize` is retired down to `audit`
// only. `managed`/`semantic`/`traceability` `report`/`next`/`run` are gone --
// `aw health` absorbs the read (coverage/next.command already sourced from
// the same inventory/coverage library code below) and slice-E worker verbs
// (`aw td promote`, `aw td create --from-source`, `aw wi create`, ...) absorb the
// mutating remediation. The subcommand is required (not `Option`): with only
// `audit` left, a bare `aw standardize --project <p>` would just be a second,
// narrower copy of `aw health --project <p>` -- exactly the duplicated read
// surface this slice removes. Run `aw health --project <p>` instead.
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct StandardizeArgs {
    /// Project name from aw.toml.
    #[arg(long, global = true)]
    pub project: Option<String>,
    #[command(subcommand)]
    pub command: StandardizeCommand,
}

#[derive(Debug, Subcommand)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub enum StandardizeCommand {
    /// Audit-first preservation protocol for quality standardization.
    Audit(StandardizeAuditArgs),
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-standardize-audit-first-quality.md#schema
pub struct StandardizeAuditArgs {
    #[command(subcommand)]
    pub command: StandardizeAuditCommand,
}

#[derive(Debug, Subcommand)]
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-standardize-audit-first-quality.md#schema
pub enum StandardizeAuditCommand {
    /// Check whether the preservation audit baseline exists.
    Check(StandardizeAuditCheckArgs),
    /// Record a bounded preservation audit fixture for the project.
    Record(StandardizeAuditRecordArgs),
}

#[derive(Debug, Args, Clone)]
#[command(after_help = r#"Output schema (JSON default):
{
  "audit_required": bool,
  "audit_path": string,
  "surfaces_to_preserve": [string]
}"#)]
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-standardize-audit-first-quality.md#schema
pub struct StandardizeAuditCheckArgs {
    /// Override workspace scopes. Repeatable; supports simple glob prefixes like `projects/app/**`.
    #[arg(long = "scope")]
    pub scopes: Vec<String>,
    /// DEPRECATED compatibility no-op. Standardize emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human-readable output.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON output.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Args, Clone)]
#[command(after_help = r#"Output schema (JSON default):
{
  "project": string,
  "scope": string | null,
  "surfaces": [{ "kind": string, "name": string, "preserve": string }],
  "quality_debt": [string],
  "safe_levers": [{ "name": string, "risk": string }]
}"#)]
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-standardize-audit-first-quality.md#schema
pub struct StandardizeAuditRecordArgs {
    /// Override workspace scopes. Repeatable; supports simple glob prefixes like `projects/app/**`.
    #[arg(long = "scope")]
    pub scopes: Vec<String>,
    /// DEPRECATED compatibility no-op. Standardize emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human-readable output.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON output.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct StandardizationCoverage {
    pub scope: Vec<String>,
    pub total_files: usize,
    pub managed_files: usize,
    pub percent: f64,
    pub by_language: BTreeMap<String, usize>,
    pub by_marker: MarkerCounts,
    pub uncovered_files: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct MarkerCounts {
    pub codegen: usize,
    pub handwrite: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct RegenerabilityCoverage {
    pub scope: Vec<String>,
    pub total_files: usize,
    pub eligible_files: usize,
    pub codegen_files: usize,
    pub handwrite_files: usize,
    pub unmarked_files: usize,
    pub unsupported_codegen_files: Vec<String>,
    pub non_replayable_codegen_files: Vec<String>,
    pub snapshot_codegen_files: Vec<String>,
    pub codegen_drift_evaluated: bool,
    pub codegen_drift_files: Vec<String>,
    pub percent: f64,
    pub gap_files: Vec<String>,
    pub semantic_percent: f64,
    pub generator_primitive_gaps: usize,
    pub primitive_covered_files: usize,
    pub missing_generator_primitive_gaps: usize,
    pub insufficient_td_section_gaps: usize,
    pub human_decision_required_gaps: usize,
    pub next_gap: Option<SemanticGap>,
    pub authority_mode: crate::cli::regenerability_policy::RegenerabilityAuthority,
    pub required_for_production: bool,
    pub authority_reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct SemanticCoverage {
    pub scope: Vec<String>,
    pub total_files: usize,
    pub source_units: usize,
    pub source_symbols: usize,
    pub claim_files: usize,
    pub semantic_files: usize,
    pub semantically_covered_files: usize,
    pub percent: f64,
    pub source_ir: Vec<SourceUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_evidence_graph: Option<SourceEvidenceGraph>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend_ecosystem: Option<FrontendEcosystemAst>,
    pub coverage_map: Vec<CoverageMapEntry>,
    pub generator_primitive_gaps: Vec<GeneratorPrimitiveGap>,
    pub uncovered_files: Vec<String>,
    pub next_gap: Option<SemanticGap>,
    pub blocked_gap_count: usize,
    pub human_decision_required_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct TraceabilityCoverage {
    pub project: String,
    pub scope: Vec<String>,
    pub cap_path: String,
    pub total_td_files: usize,
    pub traceable_td_files: usize,
    pub traceability_percent: f64,
    pub internal_td_count: usize,
    pub orphan_td_count: usize,
    pub source_edge_count: usize,
    pub cb_edge_count: usize,
    pub command_traceability: CommandTraceabilityCoverage,
    pub blocker_count: usize,
    pub blockers: Vec<TraceabilityBlocker>,
    pub next_gap: Option<TraceabilityBlocker>,
}

/// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
impl TraceabilityCoverage {
    pub fn ready_fixture(project: &str) -> Self {
        Self {
            project: project.to_string(),
            scope: Vec::new(),
            cap_path: String::new(),
            total_td_files: 0,
            traceable_td_files: 0,
            traceability_percent: 100.0,
            internal_td_count: 0,
            orphan_td_count: 0,
            source_edge_count: 0,
            cb_edge_count: 0,
            command_traceability: CommandTraceabilityCoverage::ready_fixture(),
            blocker_count: 0,
            blockers: Vec::new(),
            next_gap: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct CommandTraceabilityCoverage {
    pub total_command_paths: usize,
    pub traceable_command_paths: usize,
    pub command_traceability_percent: f64,
    pub hidden_command_count: usize,
    pub orphan_command_count: usize,
    pub command_ref_count: usize,
    pub orphan_commands: Vec<String>,
    pub blockers: Vec<TraceabilityBlocker>,
    pub next_gap: Option<TraceabilityBlocker>,
}

/// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
impl CommandTraceabilityCoverage {
    pub fn ready_fixture() -> Self {
        Self {
            total_command_paths: 0,
            traceable_command_paths: 0,
            command_traceability_percent: 100.0,
            hidden_command_count: 0,
            orphan_command_count: 0,
            command_ref_count: 0,
            orphan_commands: Vec::new(),
            blockers: Vec::new(),
            next_gap: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct TraceabilityBlocker {
    pub kind: TraceabilityBlockerKind,
    pub target: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub enum TraceabilityBlockerKind {
    TdNoCapabilityRef,
    TdInvalidCapabilityRef,
    TdMissingPrimaryCapabilityRef,
    TdChangeMissingImplMode,
    TdChangeInvalidImplMode,
    TdChangeMissingSection,
    TdChangeInvalidSection,
    TdSectionNoImplementationEdge,
    InternalTdHasSourceEdge,
    SourceBlockNoTd,
    SourceBlockTdNoCapabilityRef,
    CbBlockTdNoCapabilityRef,
    CommandNoTdRef,
    CommandRefUnknownCommand,
    CommandRefTdNoCapabilityRef,
    HiddenCommandRegistered,
    ActiveDocUnknownCommandRef,
    ActiveDocDeletedCommandRef,
}

/// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
impl TraceabilityBlockerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TraceabilityBlockerKind::TdNoCapabilityRef => "td_no_capability_ref",
            TraceabilityBlockerKind::TdInvalidCapabilityRef => "td_invalid_capability_ref",
            TraceabilityBlockerKind::TdMissingPrimaryCapabilityRef => {
                "td_missing_primary_capability_ref"
            }
            TraceabilityBlockerKind::TdChangeMissingImplMode => "td_change_missing_impl_mode",
            TraceabilityBlockerKind::TdChangeInvalidImplMode => "td_change_invalid_impl_mode",
            TraceabilityBlockerKind::TdChangeMissingSection => "td_change_missing_section",
            TraceabilityBlockerKind::TdChangeInvalidSection => "td_change_invalid_section",
            TraceabilityBlockerKind::TdSectionNoImplementationEdge => {
                "td_section_no_implementation_edge"
            }
            TraceabilityBlockerKind::InternalTdHasSourceEdge => "internal_td_has_source_edge",
            TraceabilityBlockerKind::SourceBlockNoTd => "source_block_no_td",
            TraceabilityBlockerKind::SourceBlockTdNoCapabilityRef => {
                "source_block_td_no_capability_ref"
            }
            TraceabilityBlockerKind::CbBlockTdNoCapabilityRef => "cb_block_td_no_capability_ref",
            TraceabilityBlockerKind::CommandNoTdRef => "command_no_td_ref",
            TraceabilityBlockerKind::CommandRefUnknownCommand => "command_ref_unknown_command",
            TraceabilityBlockerKind::CommandRefTdNoCapabilityRef => {
                "command_ref_td_no_capability_ref"
            }
            TraceabilityBlockerKind::HiddenCommandRegistered => "hidden_command_registered",
            TraceabilityBlockerKind::ActiveDocUnknownCommandRef => "active_doc_unknown_command_ref",
            TraceabilityBlockerKind::ActiveDocDeletedCommandRef => "active_doc_deleted_command_ref",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct StackMigrationCoverage {
    pub project: String,
    pub workspaces: Vec<WorkspaceStackMigration>,
    pub migration_normalized_percent: f64,
    pub incomplete_workspace_count: usize,
    pub dependency_policy_blockers: Vec<String>,
    pub deployment_policy_blockers: Vec<String>,
    pub blockers: Vec<String>,
}

/// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub(crate) struct ProjectHealthStandardizeCoverage {
    pub managed: StandardizationCoverage,
    pub semantic: SemanticCoverage,
    pub traceability: TraceabilityCoverage,
    pub regenerable: RegenerabilityCoverage,
    pub stack_migration: StackMigrationCoverage,
    pub drift_marker: DriftMarkerCoverage,
}

/// Issue #1276: one `crate::generate::audit::audit_file_unified` finding
/// (CODEGEN drift, a CODEGEN item missing its `@spec` marker, or a
/// spec-claimed item living outside CODEGEN markers), projected for the
/// `aw health` drift/marker axis. Same shape as `RustAuditFinding` (which
/// this is built from) but public/`Serialize` so it can round-trip through
/// `ProjectHealthReport`'s JSON payload.
#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct DriftMarkerFinding {
    pub kind: StandardizeActionKind,
    pub target: String,
    pub reason: String,
}

/// Issue #1276: health-axis projection of the CODEGEN-drift / HANDWRITE
/// marker-gap audit engine. This reuses
/// `crate::generate::audit::audit_file_unified` via `collect_rust_audit_findings`
/// (already run once per `aw health` call inside `build_inventory`, at zero
/// additional scan cost) instead of the retired whole-tree walker a bare,
/// slug-less `aw td code-check` used to run (#844). `clean_files` counts
/// scanned `.rs` files with no drift/marker-gap/uncovered finding;
/// `drift_count`/`marker_gap_count`/`uncovered_count` mirror
/// `crate::generate::audit::UnifiedReport::status()`'s `"drift"`/
/// `"marker_gap"`/`"uncovered"` values.
#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct DriftMarkerCoverage {
    pub project: String,
    pub clean_files: usize,
    pub drift_count: usize,
    pub marker_gap_count: usize,
    pub uncovered_count: usize,
    pub findings: Vec<DriftMarkerFinding>,
}

// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
impl DriftMarkerCoverage {
    /// The single highest-priority finding to route `aw health`'s
    /// `next.command` from: a `RegenDrift` (real regen verb available) beats
    /// a `FoldShadow` (a promotable path) beats an `IssueMarkerGap` (no
    /// deterministic worker verb, falls back to the `aw health` pointer).
    pub fn top_finding(&self) -> Option<&DriftMarkerFinding> {
        self.findings
            .iter()
            .find(|f| f.kind == StandardizeActionKind::RegenDrift)
            .or_else(|| {
                self.findings
                    .iter()
                    .find(|f| f.kind == StandardizeActionKind::FoldShadow)
            })
            .or_else(|| {
                self.findings
                    .iter()
                    .find(|f| f.kind == StandardizeActionKind::IssueMarkerGap)
            })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct WorkspaceStackMigration {
    pub name: String,
    pub target: Option<String>,
    pub paths: Vec<String>,
    pub manifest_stacks: Vec<String>,
    pub source_stacks: Vec<String>,
    pub migration_state: String,
    pub persistence_annotations: usize,
    pub dependency_policies: Vec<DependencyPolicyFinding>,
    pub deployment_manifest_count: usize,
    pub deployment_facets: Vec<DeploymentFacetFinding>,
    pub unsupported_deployment_kinds: Vec<String>,
    pub normalized: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct DependencyPolicyFinding {
    pub dependency: String,
    pub classification: String,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct DeploymentFacetFinding {
    pub path: String,
    pub kind: String,
    pub api_version: String,
    pub facet: String,
    pub classification: String,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct SourceUnit {
    pub path: String,
    pub language: String,
    pub symbols: Vec<SourceSymbol>,
    pub imports: Vec<ImportEdge>,
    pub generator_primitives: Vec<String>,
    pub managed_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_evidence_node: Option<SourceEvidenceNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend_node: Option<FrontendSourceNode>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct SourceEvidenceGraph {
    pub domains: Vec<SourceEvidenceDomain>,
    pub source_nodes: Vec<SourceEvidenceNode>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct SourceEvidenceDomain {
    pub key: String,
    pub layers: Vec<String>,
    pub section_types: Vec<String>,
    pub source_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct SourceEvidenceNode {
    pub path: String,
    pub layer: String,
    pub ecosystem: String,
    pub role: String,
    pub section_type: String,
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct FrontendEcosystemAst {
    pub workspaces: Vec<FrontendWorkspaceNode>,
    pub source_nodes: Vec<FrontendSourceNode>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct FrontendWorkspaceNode {
    pub root: String,
    pub kind: String,
    pub package_name: Option<String>,
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct FrontendSourceNode {
    pub path: String,
    pub workspace_root: String,
    pub role: String,
    pub section_type: String,
    pub artifact_kind: String,
    pub route: Option<String>,
    pub component: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct SourceSymbol {
    pub name: String,
    pub kind: String,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct ImportEdge {
    pub path: String,
    pub items: Vec<String>,
    pub external: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct CoverageMapEntry {
    pub source_unit: String,
    pub td_section: Option<String>,
    pub generator_primitives: Vec<String>,
    pub ownership_state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct GeneratorPrimitiveGap {
    pub target: String,
    pub primitive: String,
    pub reason: String,
    pub human_decision_required: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct SemanticGap {
    pub target: String,
    pub primitive: String,
    pub reason: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub struct StandardizeAction {
    pub id: String,
    pub kind: StandardizeActionKind,
    pub target: String,
    pub executor: String,
    pub command: String,
    pub reason: String,
    pub requires_hitl: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub enum StandardizeActionKind {
    AuditRequired,
    RegenDrift,
    ProjectRootArtifact,
    PromoteHandwrite,
    SemanticGap,
    GeneratorPrimitiveGap,
    IssueMarkerGap,
    FixSpecRule,
    FoldShadow,
    ClaimCode,
    None,
    Blocked,
}

#[derive(Debug, Clone)]
struct Inventory {
    coverage: StandardizationCoverage,
    files: Vec<SourceFile>,
    // Issue #1276: now read in production by `project_health_standardize_coverage`
    // (via `drift_marker_coverage`) to build the `aw health` drift/marker
    // axis, in addition to the cfg(test) `choose_regenerable_action_with_project`
    // ("regenerable" maturity scan) coverage.
    rust_findings: Vec<RustAuditFinding>,
    #[allow(dead_code)]
    spec_violation: Option<SpecViolation>,
}

#[derive(Debug, Clone)]
struct SourceFile {
    rel: String,
    abs: PathBuf,
    language: String,
    markers: FileMarkers,
    handwrite_gaps: Vec<HandwriteGap>,
}

#[derive(Debug, Clone, Default)]
struct FileMarkers {
    codegen: bool,
    handwrite: bool,
}

// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
impl FileMarkers {
    fn managed(&self) -> bool {
        self.codegen || self.handwrite
    }
}

#[derive(Debug, Clone)]
// `message`/`needs_promotion` are read directly by
// `marker_parser_recognizes_comment_styles`, and all four fields are
// read by the cfg(test) `choose_regenerable_action_with_project`
// ("regenerable" maturity scan); no production driver reads them in a
// plain build since #920 retired the old `choose_action` dispatcher.
#[allow(dead_code)]
struct HandwriteGap {
    line_no: usize,
    tracker: String,
    message: String,
    needs_promotion: bool,
}

// Issue #1276: now read in production by `drift_marker_coverage` (the `aw
// health` drift/marker axis), in addition to the cfg(test)
// `choose_regenerable_action_with_project` ("regenerable" maturity scan).
#[derive(Debug, Clone)]
struct RustAuditFinding {
    kind: StandardizeActionKind,
    target: String,
    reason: String,
}

// Read only by the cfg(test) `choose_regenerable_action_with_project`
// (the "regenerable" maturity scan's own test coverage); see the note
// on `RustAuditFinding` above.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SpecViolation {
    target: String,
    reason: String,
}

// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub async fn run(args: StandardizeArgs) -> Result<()> {
    let project = args.project;
    match args.command {
        StandardizeCommand::Audit(a) => run_audit_stage(project.as_deref(), a).await,
    }
}

fn print_json<T: Serialize>(value: &T, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}

fn require_standardize_project<'a>(project: Option<&'a str>) -> Result<&'a str> {
    project.ok_or_else(|| anyhow::anyhow!("standardize requires --project <project>"))
}

async fn run_audit_stage(project: Option<&str>, args: StandardizeAuditArgs) -> Result<()> {
    match args.command {
        StandardizeAuditCommand::Check(a) => run_audit_check(project, a).await,
        StandardizeAuditCommand::Record(a) => run_audit_record(project, a).await,
    }
}

async fn run_audit_check(project: Option<&str>, args: StandardizeAuditCheckArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let project =
        resolve_standardize_project_name(&project_root, require_standardize_project(project)?)?;
    let decision = standardize_audit::evaluate_audit_decision(
        &project_root,
        &project,
        &args.scopes,
        StandardizeActionKind::PromoteHandwrite,
    );
    if !args.human {
        print_json(&decision, args.pretty || args.json)?;
    } else if decision.audit_required {
        println!("audit required: {}", decision.audit_path);
        println!("preserve: {}", decision.surfaces_to_preserve.join(", "));
    } else {
        println!("audit present: {}", decision.audit_path);
    }
    Ok(())
}

async fn run_audit_record(project: Option<&str>, args: StandardizeAuditRecordArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let project =
        resolve_standardize_project_name(&project_root, require_standardize_project(project)?)?;
    let audit = standardize_audit::fixture_audit(&project, &args.scopes);
    let path = standardize_audit::audit_path(&project_root, &project);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(&audit)?;
    fs::write(&path, format!("{body}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;
    if !args.human {
        print_json(&audit, args.pretty || args.json)?;
    } else {
        println!("recorded preservation audit: {}", path.display());
    }
    Ok(())
}

// Return managed/adoption coverage for one configured project without
// printing the standardize report.
// @spec apps/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_managed_coverage(project: &str) -> Result<StandardizationCoverage> {
    let project_root = crate::find_project_root()?;
    let inventory = build_inventory(&project_root, &[], Some(project), false)?;
    Ok(inventory.coverage)
}

// Touched-scope standardization verdict for one WI's candidate file set
// (typically a code-check terminal gate's branch-diff-union-Changes-paths
// scope; see `cb_fill::resolve_touched_scope`). `unmarked` names in-scope
// candidate files carrying neither a CODEGEN nor a HANDWRITE marker;
// `attr_gap` names candidate files whose HANDWRITE marker(s) are missing a
// required `gap`/`tracker`/`reason` attribute (see `detect_handwrite_gaps`).
// `baseline_percent` is the project's managed-coverage percent computed
// over the REST of the managed inventory, i.e. excluding every candidate
// path — this answers "was everything this WI did not touch already fully
// standardized", which is what activation policy gates on (issue #932).
pub(crate) struct TouchedScopeStandardization {
    pub baseline_percent: f64,
    pub unmarked: Vec<String>,
    pub attr_gap: Vec<String>,
}

// Compute the touched-scope standardization verdict for `candidate_paths`
// against one configured project's managed inventory. Only files that are
// both in-scope (managed-inventory scope: supported source language, not
// excluded) AND present in `candidate_paths` are evaluated; every other
// inventory file only contributes to `baseline_percent`. Files in
// `candidate_paths` that fall outside the managed-inventory scope (e.g. not
// a supported source language) are silently ignored — this gate never
// invents new file-scope rules beyond the existing managed-inventory
// classifier.
// @spec apps/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub(crate) fn project_touched_scope_standardization(
    project: &str,
    candidate_paths: &[String],
) -> Result<TouchedScopeStandardization> {
    let project_root = crate::find_project_root()?;
    let inventory = build_inventory(&project_root, &[], Some(project), false)?;
    let candidates: std::collections::BTreeSet<&str> =
        candidate_paths.iter().map(String::as_str).collect();

    let mut unmarked = Vec::new();
    let mut attr_gap = Vec::new();
    let mut baseline_total = inventory.coverage.total_files;
    let mut baseline_managed = inventory.coverage.managed_files;

    for file in &inventory.files {
        if !candidates.contains(file.rel.as_str()) {
            continue;
        }
        baseline_total = baseline_total.saturating_sub(1);
        if file.markers.managed() {
            baseline_managed = baseline_managed.saturating_sub(1);
        } else {
            unmarked.push(file.rel.clone());
        }
        if !file.handwrite_gaps.is_empty() {
            attr_gap.push(file.rel.clone());
        }
    }

    let baseline_percent = if baseline_total == 0 {
        100.0
    } else {
        (baseline_managed as f64 / baseline_total as f64) * 100.0
    };

    Ok(TouchedScopeStandardization {
        baseline_percent,
        unmarked,
        attr_gap,
    })
}

// Return full regenerability coverage for one configured project without
// printing the standardize report.
// @spec apps/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_regenerability_coverage(project: &str) -> Result<RegenerabilityCoverage> {
    project_regenerability_coverage_with_options(project, true)
}

// Return marker/semantic regenerability maturity without replaying generated
// artifacts. This keeps interactive health checks fast; deterministic drift
// verification remains available through full regenerability coverage.
// @spec apps/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_regenerability_fast_coverage(project: &str) -> Result<RegenerabilityCoverage> {
    project_regenerability_coverage_with_options(project, false)
}

fn project_regenerability_coverage_with_options(
    project: &str,
    verify_codegen_drift: bool,
) -> Result<RegenerabilityCoverage> {
    let project_root = crate::find_project_root()?;
    let inventory = build_inventory(&project_root, &[], Some(project), false)?;
    let semantic = build_semantic_coverage(&project_root, &inventory)?;
    build_regenerability_coverage_with_options(
        &project_root,
        &inventory,
        &semantic,
        Some(project),
        verify_codegen_drift,
    )
}

// Return semantic TD and generator primitive coverage for one configured project
// without printing the standardize report.
// @spec apps/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_semantic_coverage(project: &str) -> Result<SemanticCoverage> {
    let project_root = crate::find_project_root()?;
    let inventory = build_inventory(&project_root, &[], Some(project), false)?;
    build_semantic_coverage(&project_root, &inventory)
}

// Return TD/source/CB capability-closure coverage for one configured project
// without printing the traceability report.
// @spec apps/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_traceability_coverage(project: &str) -> Result<TraceabilityCoverage> {
    project_traceability_coverage_with_scopes(project, &[])
}

fn project_traceability_coverage_with_scopes(
    project: &str,
    scopes: &[String],
) -> Result<TraceabilityCoverage> {
    let project_root = crate::find_project_root()?;
    let project = resolve_standardize_project_name(&project_root, project)?;
    let inventory = if scopes.is_empty() {
        build_inventory(&project_root, &[], Some(&project), false)?
    } else {
        build_inventory(&project_root, scopes, None, false)?
    };
    build_traceability_coverage(&project_root, &project, &inventory)
}

// Return workspace-level stack and persistence-migration classification for one
// configured project without conflating it with byte-equivalent replay.
// @spec apps/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_stack_migration_coverage(project: &str) -> Result<StackMigrationCoverage> {
    let project_root = crate::find_project_root()?;
    let project = resolve_standardize_project_name(&project_root, project)?;
    build_stack_migration_coverage(&project_root, &project)
}

/// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#source
pub(crate) fn project_health_standardize_coverage(
    project: &str,
    verify_traceability: bool,
    verify_codegen_drift: bool,
) -> Result<ProjectHealthStandardizeCoverage> {
    let project_root = crate::find_project_root()?;
    let project = resolve_standardize_project_name(&project_root, project)?;
    let inventory = build_inventory(&project_root, &[], Some(&project), false)?;
    let semantic = build_semantic_coverage(&project_root, &inventory)?;
    let traceability = if verify_traceability {
        build_traceability_coverage(&project_root, &project, &inventory)?
    } else {
        let mut coverage = TraceabilityCoverage::ready_fixture(&project);
        coverage.scope = inventory.coverage.scope.clone();
        coverage.traceability_percent = 0.0;
        coverage.command_traceability.command_traceability_percent = 0.0;
        coverage
    };
    let regenerable = build_regenerability_coverage_with_options(
        &project_root,
        &inventory,
        &semantic,
        Some(&project),
        verify_codegen_drift,
    )?;
    let stack_migration =
        build_stack_migration_coverage_with_inventory(&project_root, &project, &inventory)?;
    let total_rust_files = inventory
        .files
        .iter()
        .filter(|f| f.language == "rust")
        .count();
    let drift_marker = drift_marker_coverage(&project, total_rust_files, &inventory.rust_findings);

    Ok(ProjectHealthStandardizeCoverage {
        managed: inventory.coverage,
        semantic,
        traceability,
        regenerable,
        stack_migration,
        drift_marker,
    })
}

/// Issue #1276: project `Inventory.rust_findings` (already computed once per
/// `aw health` call by `build_inventory` -> `collect_rust_audit_findings`,
/// which drives `crate::generate::audit::audit_file_unified` over every
/// scanned `.rs` file) into the `aw health` drift/marker axis. `clean_files`
/// is derived by set-difference (scanned rust files minus files carrying at
/// least one finding) rather than re-deriving `UnifiedReport::Clean` counts,
/// since `collect_rust_audit_findings` only projects the actionable
/// (`Drift`/`MarkerGap`/`Uncovered`) variants.
fn drift_marker_coverage(
    project: &str,
    total_rust_files: usize,
    rust_findings: &[RustAuditFinding],
) -> DriftMarkerCoverage {
    let mut drift_count = 0usize;
    let mut marker_gap_count = 0usize;
    let mut uncovered_count = 0usize;
    let mut flagged_targets: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let findings = rust_findings
        .iter()
        .map(|finding| {
            match finding.kind {
                StandardizeActionKind::RegenDrift => drift_count += 1,
                StandardizeActionKind::IssueMarkerGap => marker_gap_count += 1,
                StandardizeActionKind::FoldShadow => uncovered_count += 1,
                _ => {}
            }
            flagged_targets.insert(finding.target.as_str());
            DriftMarkerFinding {
                kind: finding.kind,
                target: finding.target.clone(),
                reason: finding.reason.clone(),
            }
        })
        .collect();
    let clean_files = total_rust_files.saturating_sub(flagged_targets.len());
    DriftMarkerCoverage {
        project: project.to_string(),
        clean_files,
        drift_count,
        marker_gap_count,
        uncovered_count,
        findings,
    }
}

/// #1276: mirrors `managed_health_worker_command`/`semantic_health_worker_command`/
/// `traceability_health_worker_command`'s "concrete verb where derivable,
/// else a read-only pointer" pattern. `RegenDrift` routes to the real
/// project-wide regen verb (the finding only names a file, not a WI slug);
/// `FoldShadow` (a spec-claimed item living outside CODEGEN markers) routes
/// to `aw td promote <path>` since the finding's target *is* a usable path;
/// `IssueMarkerGap` (a CODEGEN item missing its `@spec` marker) has no
/// deterministic worker verb derivable from a bare file path, so it falls
/// back to the `aw health` pointer, same as the semantic/traceability axes.
pub(crate) fn drift_marker_health_worker_command(
    project: &str,
    top_finding: Option<&DriftMarkerFinding>,
) -> String {
    match top_finding {
        Some(finding) if finding.kind == StandardizeActionKind::RegenDrift => {
            format!("aw td gen --force-regen --project {project}")
        }
        Some(finding) if finding.kind == StandardizeActionKind::FoldShadow => {
            format!("aw td promote {}", finding.target)
        }
        _ => format!("aw health --project {project} drift-marker --verbose"),
    }
}

/// #920 (epic #914 slice F): `aw standardize managed run --project <p>
/// --non-interactive --max-ticks 1` is retired. `project_health_next_command`
/// (project.rs) names the concrete slice-E worker verb for the top
/// managed-layer gap directly instead of looping back through a removed
/// layer-run driver.
///
/// Only the `claim_code` action kind (an untracked/unmarked source file,
/// `choose_action`'s own lowest-priority-but-cheapest-to-detect managed
/// finding) is derivable here without a live filesystem inventory scan:
/// `ProjectHealthReport` retains the first `uncovered_files` entry from the
/// managed-coverage computation that already ran to produce the health
/// report. `promote_handwrite`/`issue_marker_gap` need per-file HANDWRITE
/// marker attributes (`tracker`, `needs_promotion`) that live on
/// `Inventory::files[..].handwrite_gaps` and are not retained on
/// `ProjectHealthReport`; those (and any other managed gap kind) fall back
/// to the read-only `aw health --project <p> metrics --verbose` pointer,
/// which surfaces the managed axis detail an agent needs to pick the next
/// concrete `aw td create --from-source`/`aw td promote` invocation by hand.
pub(crate) fn managed_health_worker_command(
    project: &str,
    next_uncovered_file: Option<&str>,
) -> String {
    match next_uncovered_file {
        Some(target) => format!("aw td create --from-source {target} --project {project}"),
        None => format!("aw health --project {project} metrics --verbose"),
    }
}

/// #920: semantic-layer gap remediation (author a TD covering the target
/// file, or resolve a generator-primitive design decision) needs a WI slug
/// that `aw health` cannot fabricate from a bare file path, so this always
/// routes to the read-only `aw health` pointer; `reason` already names the
/// specific target via `project_health_next_reason`.
pub(crate) fn semantic_health_worker_command(project: &str) -> String {
    format!("aw health --project {project} metrics --verbose")
}

/// #920: traceability blockers (missing `capability_refs`, stale
/// `command_refs`, hidden commands, ...) are HITL decisions -- there is no
/// single deterministic worker verb that closes an arbitrary blocker kind,
/// so this routes to the read-only `aw health --project <p> traceability
/// --verbose` pointer, which lists the concrete blocker(s) to resolve by
/// hand.
pub(crate) fn traceability_health_worker_command(project: &str) -> String {
    format!("aw health --project {project} traceability --verbose")
}

fn build_inventory(
    project_root: &Path,
    explicit_scopes: &[String],
    project: Option<&str>,
    all: bool,
) -> Result<Inventory> {
    let scope = resolve_scopes(project_root, explicit_scopes, project, all)?;
    let mut files = collect_source_files(project_root, &scope)?;
    if let Some(project_name) = project {
        extend_project_root_artifact_files(project_root, project_name, &mut files)?;
    }
    let rust_findings = collect_rust_audit_findings(project_root, &files);
    let spec_violation = find_spec_violation(project_root, &scope)?;
    let root_artifact_gaps = if project.is_some() && explicit_scopes.is_empty() && !all {
        project
            .map(|project_name| missing_project_root_artifacts(project_root, project_name))
            .transpose()?
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut by_language = BTreeMap::new();
    let mut by_marker = MarkerCounts::default();
    let mut uncovered_files = Vec::new();
    let mut managed_files = 0usize;

    for file in &files {
        *by_language.entry(file.language.clone()).or_insert(0) += 1;
        if file.markers.codegen {
            by_marker.codegen += 1;
        }
        if file.markers.handwrite {
            by_marker.handwrite += 1;
        }
        if file.markers.managed() {
            managed_files += 1;
        } else {
            uncovered_files.push(file.rel.clone());
        }
    }
    for rel in &root_artifact_gaps {
        *by_language.entry("root-artifact".to_string()).or_insert(0) += 1;
        uncovered_files.push(rel.clone());
    }

    let total_files = files.len() + root_artifact_gaps.len();
    let percent = if total_files == 0 {
        100.0
    } else {
        (managed_files as f64 / total_files as f64) * 100.0
    };

    Ok(Inventory {
        coverage: StandardizationCoverage {
            scope,
            total_files,
            managed_files,
            percent,
            by_language,
            by_marker,
            uncovered_files,
        },
        files,
        rust_findings,
        spec_violation,
    })
}

fn build_semantic_coverage(project_root: &Path, inventory: &Inventory) -> Result<SemanticCoverage> {
    let td_index = collect_td_index(project_root, &inventory.coverage.scope)?;
    let configured = read_config_workspace_scopes(project_root).unwrap_or_default();
    let semantic_source_files = semantic_source_files(inventory);
    let source_ir = build_source_ir_for_files(&semantic_source_files);
    let source_evidence_graph = build_source_evidence_graph(&source_ir);
    let frontend_ecosystem = build_frontend_ecosystem_ast(inventory);
    let source_symbols = source_ir.iter().map(|unit| unit.symbols.len()).sum();
    let claim_files = td_index.values().filter(|td| td.is_claim).count();
    let semantic_td_files = td_index.values().filter(|td| !td.is_claim).count();
    let semantic_paths: BTreeSet<_> = td_index
        .values()
        .filter(|td| !td.is_claim)
        .flat_map(|td| td.source_paths.iter().cloned())
        .collect();

    let mut coverage_map = Vec::new();
    let mut generator_primitive_gaps = Vec::new();
    let mut uncovered_files = Vec::new();
    let mut semantically_covered_files = 0usize;
    let inventory_paths: BTreeSet<_> = semantic_source_files
        .iter()
        .map(|file| file.rel.clone())
        .collect();

    for td in td_index.values() {
        if td.is_claim {
            continue;
        }
        let Some(target) = td
            .source_paths
            .iter()
            .find(|path| inventory_paths.contains(*path))
        else {
            continue;
        };
        let td_content = fs::read_to_string(project_root.join(&td.path)).unwrap_or_default();
        let needs_migration = td.needs_migration
            || semantic_td_needs_generated_capability_ref_migration(
                project_root,
                &configured,
                target,
                &td_content,
            );
        if !needs_migration {
            continue;
        }
        generator_primitive_gaps.push(GeneratorPrimitiveGap {
            target: target.clone(),
            primitive: "semantic_td_legacy".to_string(),
            reason: format!(
                "semantic TD {} needs semantic evidence migration; rewrite to approved section-type TD with source evidence graph",
                td.path
            ),
            human_decision_required: false,
        });
    }

    for file in semantic_source_files.iter().copied() {
        let source_refs = source_spec_refs(&file.abs, project_root);
        let td_section = if semantic_paths.contains(&file.rel) {
            td_index
                .values()
                .find(|td| !td.is_claim && td.source_paths.contains(&file.rel))
                .map(|td| td.path.clone())
        } else {
            source_refs
                .into_iter()
                .find(|spec| td_index.get(spec).is_some_and(|td| !td.is_claim))
        };
        let covered = td_section.is_some();
        if covered {
            semantically_covered_files += 1;
        } else {
            uncovered_files.push(file.rel.clone());
            generator_primitive_gaps.push(GeneratorPrimitiveGap {
                target: file.rel.clone(),
                primitive: "semantic_td_missing".to_string(),
                reason: "source unit has no semantic TD coverage; claim TDs do not count as semantic coverage".to_string(),
                human_decision_required: false,
            });
        }

        let primitives = source_ir
            .iter()
            .find(|unit| unit.path == file.rel)
            .map(|unit| unit.generator_primitives.clone())
            .unwrap_or_default();
        if covered && file.markers.handwrite && has_executable_generator_promotion(file) {
            let primitive = primitives
                .first()
                .cloned()
                .unwrap_or_else(|| "generator_primitive_missing".to_string());
            generator_primitive_gaps.push(GeneratorPrimitiveGap {
                target: file.rel.clone(),
                primitive,
                reason:
                    "semantic TD coverage exists, but source still contains HANDWRITE ownership"
                        .to_string(),
                human_decision_required: false,
            });
        }

        coverage_map.push(CoverageMapEntry {
            source_unit: file.rel.clone(),
            td_section,
            generator_primitives: primitives,
            ownership_state: ownership_state(&file.markers).to_string(),
        });
    }

    generator_primitive_gaps.sort_by(|a, b| {
        a.human_decision_required
            .cmp(&b.human_decision_required)
            .then_with(|| {
                primitive_gap_priority(&a.primitive).cmp(&primitive_gap_priority(&b.primitive))
            })
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.primitive.cmp(&b.primitive))
    });
    generator_primitive_gaps.dedup();

    let next_gap = generator_primitive_gaps
        .iter()
        .find(|gap| !gap.human_decision_required)
        .map(|gap| SemanticGap {
            target: gap.target.clone(),
            primitive: gap.primitive.clone(),
            reason: gap.reason.clone(),
            action: semantic_gap_action(&gap.primitive).to_string(),
        });
    let blocked_gap_count = generator_primitive_gaps
        .iter()
        .filter(|gap| gap.human_decision_required)
        .count();
    let human_decision_required_count = blocked_gap_count;
    let total_semantic_source_files = semantic_source_files.len();
    let percent = if total_semantic_source_files == 0 {
        100.0
    } else {
        (semantically_covered_files as f64 / total_semantic_source_files as f64) * 100.0
    };

    Ok(SemanticCoverage {
        scope: inventory.coverage.scope.clone(),
        total_files: total_semantic_source_files,
        source_units: source_ir.len(),
        source_symbols,
        claim_files,
        semantic_files: semantic_td_files,
        semantically_covered_files,
        percent,
        source_ir,
        source_evidence_graph,
        frontend_ecosystem,
        coverage_map,
        generator_primitive_gaps,
        uncovered_files,
        next_gap,
        blocked_gap_count,
        human_decision_required_count,
    })
}

fn semantic_source_files(inventory: &Inventory) -> Vec<&SourceFile> {
    inventory
        .files
        .iter()
        .filter(|file| !is_aw_ec_generated_wrapper(file))
        .collect()
}

fn is_aw_ec_generated_wrapper(file: &SourceFile) -> bool {
    fs::read_to_string(&file.abs)
        .map(|content| content.contains(AW_EC_BEGIN_MARKER))
        .unwrap_or(false)
}

#[derive(Debug, Clone)]
struct TraceabilityTdRecord {
    path: String,
    internal: bool,
    has_valid_capability_refs: bool,
    has_capability_refs: bool,
    command_refs: BTreeSet<String>,
    source_paths: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct CommandInventoryEntry {
    path: String,
    hidden: bool,
    alias_of: Option<String>,
}

#[derive(Debug, Clone)]
struct TraceabilityChangeEntry {
    index: usize,
    path: Option<String>,
    section: Option<String>,
    impl_mode: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct TraceabilityTdFrontmatter {
    #[serde(default)]
    capability_scope: Option<String>,
    #[serde(default)]
    capability_refs: Vec<crate::cli::capability::TdCapabilityRef>,
    #[serde(default)]
    command_refs: Vec<TraceabilityCommandRef>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct TraceabilityCommandRef {
    command: String,
}

fn build_traceability_coverage(
    project_root: &Path,
    project: &str,
    inventory: &Inventory,
) -> Result<TraceabilityCoverage> {
    let command_inventory = if project == "agentic-workflow" {
        runtime_command_inventory()
    } else {
        BTreeMap::new()
    };
    build_traceability_coverage_with_command_inventory(
        project_root,
        project,
        inventory,
        &command_inventory,
    )
}

fn build_traceability_coverage_with_command_inventory(
    project_root: &Path,
    project: &str,
    inventory: &Inventory,
    command_inventory: &BTreeMap<String, CommandInventoryEntry>,
) -> Result<TraceabilityCoverage> {
    let (cap_path, capability_document) = match crate::cli::capability::resolve_capability_path(
        project_root,
        project,
        None,
    ) {
        Ok(cap_path) => {
            let cap_body = match fs::read_to_string(&cap_path) {
                Ok(body) => body,
                Err(err) => {
                    return Ok(traceability_capability_map_blocked(
                        project,
                        inventory,
                        cap_path.display().to_string(),
                        format!("capability document read failed: {err}"),
                    ));
                }
            };
            match crate::cli::capability::parse_capability_document(&cap_body, &cap_path) {
                Ok(document)
                    if document.capabilities.is_empty() && document.legacy_rows.is_empty() =>
                {
                    let reason = document.findings.first().cloned().unwrap_or_else(|| {
                            "no capability sections found; define README capability roots under ## Capabilities"
                                .to_string()
                        });
                    return Ok(traceability_capability_map_blocked(
                        project,
                        inventory,
                        cap_path.display().to_string(),
                        format!("capability document has no capability sections: {reason}"),
                    ));
                }
                Ok(document) => (cap_path, document),
                Err(err) => {
                    return Ok(traceability_capability_map_blocked(
                        project,
                        inventory,
                        cap_path.display().to_string(),
                        format!("capability document parse failed: {err}"),
                    ));
                }
            }
        }
        Err(err) => {
            return Ok(traceability_capability_map_blocked(
                project,
                inventory,
                String::new(),
                format!("capability path resolution failed: {err}"),
            ));
        }
    };
    let td_index = collect_td_index(project_root, &inventory.coverage.scope)?;
    let semantic = build_semantic_coverage(project_root, inventory)?;
    let mut records = BTreeMap::new();
    let mut blockers = Vec::new();

    for td in td_index.values() {
        let path = project_root.join(&td.path);
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                blockers.push(TraceabilityBlocker {
                    kind: TraceabilityBlockerKind::TdInvalidCapabilityRef,
                    target: td.path.clone(),
                    reason: format!("failed to read TD for traceability scan: {err}"),
                    source: None,
                });
                continue;
            }
        };
        let (record, mut td_blockers) = traceability_td_record(td, &content, &capability_document);
        blockers.append(&mut td_blockers);
        records.insert(record.path.clone(), record);
    }

    let inventory_paths: BTreeSet<_> = inventory
        .files
        .iter()
        .map(|file| file.rel.clone())
        .collect();
    let mut source_edges = BTreeSet::new();
    for record in records.values() {
        for source_path in &record.source_paths {
            if inventory_paths.contains(source_path) {
                source_edges.insert((source_path.clone(), record.path.clone()));
            }
        }
    }
    for entry in &semantic.coverage_map {
        if let Some(td_path) = &entry.td_section {
            source_edges.insert((entry.source_unit.clone(), td_path.clone()));
        } else {
            blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::SourceBlockNoTd,
                target: entry.source_unit.clone(),
                reason: "source unit has no semantic TD edge to close back to a capability"
                    .to_string(),
                source: Some(entry.source_unit.clone()),
            });
        }
    }

    let capability_owned_source_paths =
        traceability_capability_owned_source_paths(&source_edges, &records);

    for (source_path, td_path) in &source_edges {
        match records.get(td_path) {
            Some(record) if record.internal => blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::InternalTdHasSourceEdge,
                target: td_path.clone(),
                reason: "internal TD is referenced by production source".to_string(),
                source: Some(source_path.clone()),
            }),
            Some(record)
                if !record.has_valid_capability_refs
                    && capability_owned_source_paths.contains(source_path) => {}
            Some(record) if !record.has_valid_capability_refs => {
                blockers.push(TraceabilityBlocker {
                    kind: TraceabilityBlockerKind::SourceBlockTdNoCapabilityRef,
                    target: td_path.clone(),
                    reason:
                        "source unit resolves to a TD that does not resolve to a README capability"
                            .to_string(),
                    source: Some(source_path.clone()),
                });
            }
            Some(_) => {}
            None => blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::SourceBlockNoTd,
                target: source_path.clone(),
                reason: format!("source unit references unknown TD `{td_path}`"),
                source: Some(source_path.clone()),
            }),
        }
    }

    let mut cb_edges = BTreeSet::new();
    for file in &inventory.files {
        let Ok(content) = fs::read_to_string(&file.abs) else {
            continue;
        };
        if content.contains(AW_EC_BEGIN_MARKER) {
            continue;
        }
        for block in crate::generate::marker::parse_codegen_blocks(&content) {
            let source = format!("{}:{}", file.rel, block.begin_line + 1);
            let Some(td_path) = normalize_spec_ref_path(&block.spec_ref, project_root) else {
                blockers.push(TraceabilityBlocker {
                    kind: TraceabilityBlockerKind::SourceBlockNoTd,
                    target: source.clone(),
                    reason: "CODEGEN block has no SPEC-MANAGED TD reference".to_string(),
                    source: Some(file.rel.clone()),
                });
                continue;
            };
            if !records.contains_key(&td_path) {
                blockers.push(TraceabilityBlocker {
                    kind: TraceabilityBlockerKind::SourceBlockNoTd,
                    target: source.clone(),
                    reason: format!("CODEGEN block references unknown TD `{td_path}`"),
                    source: Some(file.rel.clone()),
                });
                continue;
            }
            cb_edges.insert((source, td_path));
        }
    }

    for (source, td_path) in &cb_edges {
        match records.get(td_path) {
            Some(record) if record.internal => blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::InternalTdHasSourceEdge,
                target: td_path.clone(),
                reason: "internal TD is referenced by a CODEGEN block".to_string(),
                source: Some(source.clone()),
            }),
            Some(record)
                if !record.has_valid_capability_refs
                    && capability_owned_source_paths
                        .contains(traceability_cb_edge_source_path(source)) => {}
            Some(record) if !record.has_valid_capability_refs => {
                blockers.push(TraceabilityBlocker {
                    kind: TraceabilityBlockerKind::CbBlockTdNoCapabilityRef,
                    target: td_path.clone(),
                    reason:
                        "CODEGEN block resolves to a TD that does not resolve to a README capability"
                            .to_string(),
                    source: Some(source.clone()),
                });
            }
            Some(_) => {}
            None => {}
        }
    }

    let command_traceability =
        build_command_traceability_coverage(project_root, &records, command_inventory);
    blockers.extend(command_traceability.blockers.iter().cloned());
    sort_traceability_blockers(&mut blockers);
    let blocked_td_paths: BTreeSet<_> = blockers
        .iter()
        .filter(|blocker| records.contains_key(&blocker.target))
        .map(|blocker| blocker.target.clone())
        .collect();
    let traceable_td_files = records
        .values()
        .filter(|record| {
            (record.internal || record.has_valid_capability_refs)
                && !blocked_td_paths.contains(&record.path)
        })
        .count();
    let internal_td_count = records.values().filter(|record| record.internal).count();
    let orphan_td_count = records
        .values()
        .filter(|record| !record.internal && !record.has_capability_refs)
        .count();
    let total_td_files = records.len();
    let traceability_percent = coverage_percent(traceable_td_files, total_td_files);

    Ok(TraceabilityCoverage {
        project: project.to_string(),
        scope: inventory.coverage.scope.clone(),
        cap_path: cap_path.display().to_string(),
        total_td_files,
        traceable_td_files,
        traceability_percent,
        internal_td_count,
        orphan_td_count,
        source_edge_count: source_edges.len(),
        cb_edge_count: cb_edges.len(),
        command_traceability,
        blocker_count: blockers.len(),
        next_gap: blockers.first().cloned(),
        blockers,
    })
}

fn traceability_capability_map_blocked(
    project: &str,
    inventory: &Inventory,
    cap_path: String,
    reason: String,
) -> TraceabilityCoverage {
    let target = if cap_path.is_empty() {
        project.to_string()
    } else {
        cap_path.clone()
    };
    let blocker = TraceabilityBlocker {
        kind: TraceabilityBlockerKind::TdInvalidCapabilityRef,
        target,
        reason,
        source: None,
    };
    TraceabilityCoverage {
        project: project.to_string(),
        scope: inventory.coverage.scope.clone(),
        cap_path,
        total_td_files: 0,
        traceable_td_files: 0,
        traceability_percent: 0.0,
        internal_td_count: 0,
        orphan_td_count: 0,
        source_edge_count: 0,
        cb_edge_count: 0,
        command_traceability: CommandTraceabilityCoverage::ready_fixture(),
        blocker_count: 1,
        blockers: vec![blocker.clone()],
        next_gap: Some(blocker),
    }
}

fn traceability_capability_owned_source_paths(
    source_edges: &BTreeSet<(String, String)>,
    records: &BTreeMap<String, TraceabilityTdRecord>,
) -> BTreeSet<String> {
    source_edges
        .iter()
        .filter_map(|(source_path, td_path)| {
            records
                .get(td_path)
                .is_some_and(|record| record.has_valid_capability_refs)
                .then(|| source_path.clone())
        })
        .collect()
}

fn traceability_cb_edge_source_path(source: &str) -> &str {
    source.rsplit_once(':').map_or(source, |(path, _)| path)
}

fn traceability_td_record(
    td: &TdCoverageRecord,
    content: &str,
    document: &crate::cli::capability::CapabilityDocument,
) -> (TraceabilityTdRecord, Vec<TraceabilityBlocker>) {
    let mut blockers = Vec::new();
    let frontmatter = match parse_traceability_frontmatter(content) {
        Ok(frontmatter) => frontmatter,
        Err(err) => {
            blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::TdInvalidCapabilityRef,
                target: td.path.clone(),
                reason: format!("invalid TD frontmatter while reading capability refs: {err}"),
                source: None,
            });
            None
        }
    };
    let capability_scope = frontmatter
        .as_ref()
        .and_then(|fm| fm.capability_scope.as_deref());
    let internal = capability_scope.is_some_and(|scope| scope == "internal");
    let has_capability_refs = frontmatter
        .as_ref()
        .is_some_and(|fm| !fm.capability_refs.is_empty());
    let command_refs = frontmatter
        .as_ref()
        .map(|fm| {
            fm.command_refs
                .iter()
                .map(|entry| normalize_command_ref(&entry.command))
                .filter(|command| !command.is_empty())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut has_valid_capability_refs = false;

    if has_capability_refs {
        match crate::cli::capability::validate_td_capability_refs_for_content(content, document) {
            Ok((_spec_id, refs, findings)) => {
                has_valid_capability_refs = !refs.is_empty() && findings.is_empty();
                for finding in findings {
                    let kind = if finding.contains("at least one primary ref") {
                        TraceabilityBlockerKind::TdMissingPrimaryCapabilityRef
                    } else {
                        TraceabilityBlockerKind::TdInvalidCapabilityRef
                    };
                    blockers.push(TraceabilityBlocker {
                        kind,
                        target: td.path.clone(),
                        reason: finding,
                        source: None,
                    });
                }
            }
            Err(err) => blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::TdInvalidCapabilityRef,
                target: td.path.clone(),
                reason: err.to_string(),
                source: None,
            }),
        }
    }
    blockers.extend(traceability_td_section_blockers(&td.path, content));

    (
        TraceabilityTdRecord {
            path: td.path.clone(),
            internal,
            has_valid_capability_refs,
            has_capability_refs,
            command_refs,
            source_paths: td.source_paths.clone(),
        },
        blockers,
    )
}

fn parse_traceability_frontmatter(content: &str) -> Result<Option<TraceabilityTdFrontmatter>> {
    let Some(frontmatter) = split_traceability_frontmatter(content) else {
        return Ok(None);
    };
    let parsed: TraceabilityTdFrontmatter =
        serde_yaml::from_str(frontmatter).context("invalid YAML frontmatter")?;
    Ok(Some(parsed))
}

fn split_traceability_frontmatter(content: &str) -> Option<&str> {
    let body = content.strip_prefix("---\n")?;
    let end = body.find("\n---")?;
    Some(&body[..end])
}

fn traceability_td_section_blockers(td_path: &str, content: &str) -> Vec<TraceabilityBlocker> {
    let authored_sections = traceability_authored_sections(content);
    let mut blockers = Vec::new();
    let mut implemented_sections = BTreeSet::new();

    match traceability_change_entries(content) {
        Ok(entries) => {
            for entry in entries {
                let hint = traceability_change_hint(&entry);
                let impl_mode_valid = match entry.impl_mode.as_deref() {
                    Some("codegen") | Some("hand-written") => true,
                    Some(mode) => {
                        blockers.push(TraceabilityBlocker {
                            kind: TraceabilityBlockerKind::TdChangeInvalidImplMode,
                            target: td_path.to_string(),
                            reason: format!(
                                "changes[{}].impl_mode `{mode}` is not one of codegen or hand-written",
                                entry.index
                            ),
                            source: Some(hint.clone()),
                        });
                        false
                    }
                    None => {
                        blockers.push(TraceabilityBlocker {
                            kind: TraceabilityBlockerKind::TdChangeMissingImplMode,
                            target: td_path.to_string(),
                            reason: format!(
                                "changes[{}] has no impl_mode, so AW cannot classify codegen vs HANDWRITE ownership",
                                entry.index
                            ),
                            source: Some(hint.clone()),
                        });
                        false
                    }
                };

                let Some(section) = entry.section.as_deref() else {
                    blockers.push(TraceabilityBlocker {
                        kind: TraceabilityBlockerKind::TdChangeMissingSection,
                        target: td_path.to_string(),
                        reason: format!(
                            "changes[{}] has no section, so no TD section type owns the codebase edge",
                            entry.index
                        ),
                        source: Some(hint),
                    });
                    continue;
                };
                let Some(section) = normalize_traceability_section_type(section) else {
                    blockers.push(TraceabilityBlocker {
                        kind: TraceabilityBlockerKind::TdChangeInvalidSection,
                        target: td_path.to_string(),
                        reason: format!(
                            "changes[{}].section `{section}` is not a known TD section type",
                            entry.index
                        ),
                        source: Some(hint),
                    });
                    continue;
                };
                if impl_mode_valid {
                    implemented_sections.insert(section);
                }
            }
        }
        Err(err) if !authored_sections.is_empty() => blockers.push(TraceabilityBlocker {
            kind: TraceabilityBlockerKind::TdSectionNoImplementationEdge,
            target: td_path.to_string(),
            reason: format!("TD has authored sections but no parseable changes[] map: {err}"),
            source: None,
        }),
        Err(_) => {}
    }

    for section in authored_sections {
        if !implemented_sections.contains(&section) {
            blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::TdSectionNoImplementationEdge,
                target: td_path.to_string(),
                reason: format!(
                    "section type `{section}` is not served by any changes[] entry with section + impl_mode"
                ),
                source: Some(format!("section:{section}")),
            });
        }
    }
    blockers
}

fn traceability_authored_sections(content: &str) -> BTreeSet<String> {
    crate::models::section::parse_all_section_annotations(content)
        .into_iter()
        .map(|(_, meta)| meta.section_type.as_str().to_string())
        .filter(|section| traceability_requires_implementation_edge(section))
        .collect()
}

fn traceability_requires_implementation_edge(section: &str) -> bool {
    !matches!(
        section,
        "changes" | "overview" | "doc" | "reference-context" | "review" | "changelog" | "e2e-test"
    )
}

fn traceability_change_entries(content: &str) -> Result<Vec<TraceabilityChangeEntry>> {
    let changes_sections = traceability_section_contents(content, "changes");
    if changes_sections.is_empty() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let mut first_error = None;
    for changes_content in changes_sections {
        match traceability_change_entries_from_section(&changes_content) {
            Ok(section_entries) => {
                for mut entry in section_entries {
                    entry.index = entries.len();
                    entries.push(entry);
                }
            }
            Err(err) if first_error.is_none() => {
                first_error = Some(err);
            }
            Err(_) => {}
        }
    }
    if entries.is_empty() {
        if let Some(err) = first_error {
            return Err(err);
        }
    }
    Ok(entries)
}

fn traceability_change_entries_from_section(
    changes_content: &str,
) -> Result<Vec<TraceabilityChangeEntry>> {
    let mut entries = Vec::new();
    let yaml_text = traceability_first_yaml_fence(changes_content)
        .context("changes section has no YAML fence")?;
    let value: serde_yaml::Value =
        serde_yaml::from_str(&yaml_text).context("changes section YAML is invalid")?;
    let seq = value
        .get("changes")
        .and_then(|v| v.as_sequence())
        .or_else(|| value.get("files").and_then(|v| v.as_sequence()))
        .or_else(|| value.as_sequence())
        .context("changes section YAML must contain `changes: [...]`")?;
    for item in seq {
        let Some(map) = item.as_mapping() else {
            continue;
        };
        entries.push(TraceabilityChangeEntry {
            index: 0,
            path: yaml_string(map, "path").or_else(|| yaml_string(map, "file")),
            section: yaml_string(map, "section"),
            impl_mode: yaml_string(map, "impl_mode"),
        });
    }
    Ok(entries)
}

fn yaml_string(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    map.get(serde_yaml::Value::String(key.to_string()))
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
}

fn traceability_change_hint(entry: &TraceabilityChangeEntry) -> String {
    entry
        .path
        .as_ref()
        .map(|path| format!("changes[{}]:{path}", entry.index))
        .unwrap_or_else(|| format!("changes[{}]", entry.index))
}

fn normalize_traceability_section_type(section: &str) -> Option<String> {
    if section == "source" {
        return Some("source".to_string());
    }
    if section == "exports" {
        return Some("exports".to_string());
    }
    section
        .parse::<crate::models::spec_rules::SectionType>()
        .ok()
        .map(|section_type| section_type.as_str().to_string())
}

fn traceability_section_contents(content: &str, target_section: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut contents = Vec::new();
    let mut fence_open: Option<String> = None;
    let mut i = 0;
    while i < lines.len() {
        if traceability_is_annotated_section_heading(lines[i])
            && traceability_section_heading_is_visible(fence_open.as_deref(), &lines, i)
        {
            let section_start = lines
                .get(i + 1)
                .and_then(|annotation_line| {
                    crate::models::section::parse_section_annotation_parts(annotation_line).map(
                        |raw| {
                            (
                                normalize_traceability_section_type(&raw.section_type)
                                    .unwrap_or(raw.section_type),
                                i + 2,
                            )
                        },
                    )
                })
                .or_else(|| {
                    (target_section == "changes"
                        && traceability_heading_declares_changes_section(lines[i]))
                    .then_some(("changes".to_string(), i + 1))
                });
            if let Some((normalized, start)) = section_start {
                if normalized == target_section {
                    let mut end = start;
                    let mut section_fence_open: Option<String> = None;
                    while end < lines.len() {
                        if let Some(open) = &section_fence_open {
                            if traceability_fence_closes(lines[end], open) {
                                section_fence_open = None;
                            }
                            end += 1;
                            continue;
                        }
                        if let Some(open) = traceability_fence_open_marker(lines[end]) {
                            section_fence_open = Some(open);
                            end += 1;
                            continue;
                        }
                        if traceability_is_markdown_heading_boundary(lines[end]) {
                            break;
                        }
                        end += 1;
                    }
                    contents.push(lines[start..end].join("\n"));
                    fence_open = None;
                    i = end;
                    continue;
                }
            }
        }
        if let Some(open) = &fence_open {
            if traceability_fence_closes(lines[i], open) {
                fence_open = None;
            }
        } else if let Some(open) = traceability_fence_open_marker(lines[i]) {
            fence_open = Some(open);
        }
        i += 1;
    }
    contents
}

fn traceability_is_annotated_section_heading(line: &str) -> bool {
    line.starts_with("## ") || line.starts_with("### ")
}

fn traceability_heading_declares_changes_section(line: &str) -> bool {
    line.trim_start_matches('#')
        .trim()
        .eq_ignore_ascii_case("changes")
}

fn traceability_is_markdown_heading_boundary(line: &str) -> bool {
    let marker_len = line.bytes().take_while(|byte| *byte == b'#').count();
    (1..=6).contains(&marker_len) && line.as_bytes().get(marker_len) == Some(&b' ')
}

fn traceability_section_heading_is_visible(
    fence_open: Option<&str>,
    lines: &[&str],
    line_index: usize,
) -> bool {
    let Some(open) = fence_open else {
        return true;
    };
    traceability_previous_nonempty_line(lines, line_index)
        .and_then(traceability_fence_open_marker)
        .is_some_and(|marker| {
            marker.as_bytes().first() == open.as_bytes().first() && marker.len() >= open.len()
        })
}

fn traceability_previous_nonempty_line<'a>(
    lines: &[&'a str],
    line_index: usize,
) -> Option<&'a str> {
    lines[..line_index]
        .iter()
        .rev()
        .copied()
        .find(|line| !line.trim().is_empty())
}

fn traceability_first_yaml_fence(content: &str) -> Option<String> {
    let mut in_yaml = false;
    let mut close_marker = String::new();
    let mut yaml = String::new();
    for line in content.lines() {
        if in_yaml {
            if traceability_fence_closes(line, &close_marker) {
                return Some(yaml);
            }
            yaml.push_str(line);
            yaml.push('\n');
            continue;
        }
        let trimmed = line.trim_start();
        for marker in ["```", "~~~"] {
            if let Some(rest) = trimmed.strip_prefix(marker) {
                let lang = rest.split_whitespace().next().unwrap_or("");
                if lang.eq_ignore_ascii_case("yaml") || lang.eq_ignore_ascii_case("json") {
                    in_yaml = true;
                    close_marker = marker.to_string();
                    yaml.clear();
                    break;
                }
            }
        }
    }
    None
}

fn traceability_fence_open_marker(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let first = trimmed.as_bytes().first().copied()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let count = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| **byte == first)
        .count();
    if count < 3 {
        return None;
    }
    Some(trimmed[..count].to_string())
}

fn traceability_fence_closes(line: &str, opener: &str) -> bool {
    let Some(marker) = traceability_fence_open_marker(line) else {
        return false;
    };
    marker.as_bytes().first() == opener.as_bytes().first()
        && marker.len() >= opener.len()
        && line.trim_start()[marker.len()..].trim().is_empty()
}

fn normalize_spec_ref_path(spec_ref: &str, project_root: &Path) -> Option<String> {
    let mut spec = spec_ref
        .split('#')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .replace('\\', "/");
    if spec.is_empty() {
        return None;
    }
    let project_root = project_root.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = spec.strip_prefix(&project_root) {
        spec = stripped.trim_start_matches('/').to_string();
    }
    Some(spec.trim_start_matches("./").to_string())
}

fn runtime_command_inventory() -> BTreeMap<String, CommandInventoryEntry> {
    let command = TraceabilityCli::command();
    let mut inventory = BTreeMap::new();
    collect_command_inventory("aw", &command, None, &mut inventory);
    inventory
}

fn collect_command_inventory(
    parent: &str,
    command: &clap::Command,
    alias_of: Option<&str>,
    inventory: &mut BTreeMap<String, CommandInventoryEntry>,
) {
    for subcommand in command.get_subcommands() {
        let path = format!("{parent} {}", subcommand.get_name());
        inventory.insert(
            path.clone(),
            CommandInventoryEntry {
                path: path.clone(),
                hidden: subcommand.is_hide_set(),
                alias_of: alias_of.map(ToString::to_string),
            },
        );
        collect_command_inventory(&path, subcommand, None, inventory);

        for alias in subcommand.get_all_aliases() {
            let alias_path = format!("{parent} {alias}");
            inventory.insert(
                alias_path.clone(),
                CommandInventoryEntry {
                    path: alias_path.clone(),
                    hidden: subcommand.is_hide_set(),
                    alias_of: Some(path.clone()),
                },
            );
            collect_command_inventory(&alias_path, subcommand, Some(&path), inventory);
        }
    }
}

fn build_command_traceability_coverage(
    project_root: &Path,
    records: &BTreeMap<String, TraceabilityTdRecord>,
    command_inventory: &BTreeMap<String, CommandInventoryEntry>,
) -> CommandTraceabilityCoverage {
    let mut blockers = Vec::new();
    let mut refs_by_command: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for record in records.values() {
        for command in &record.command_refs {
            refs_by_command
                .entry(command.clone())
                .or_default()
                .insert(record.path.clone());
            match command_inventory.get(command) {
                Some(_) if !record.has_valid_capability_refs => {
                    blockers.push(TraceabilityBlocker {
                        kind: TraceabilityBlockerKind::CommandRefTdNoCapabilityRef,
                        target: command.clone(),
                        reason:
                            "command_ref resolves to a TD that does not resolve to a README capability"
                                .to_string(),
                        source: Some(record.path.clone()),
                    });
                }
                Some(_) => {}
                None => blockers.push(TraceabilityBlocker {
                    kind: TraceabilityBlockerKind::CommandRefUnknownCommand,
                    target: command.clone(),
                    reason: "TD command_ref points to a command path not registered by clap"
                        .to_string(),
                    source: Some(record.path.clone()),
                }),
            }
        }
    }

    let public_commands: Vec<_> = command_inventory
        .values()
        .filter(|entry| !entry.hidden)
        .collect();
    let mut orphan_commands = Vec::new();
    for entry in command_inventory.values() {
        if entry.hidden {
            blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::HiddenCommandRegistered,
                target: entry.path.clone(),
                reason: "hidden command is still registered in the runtime command tree"
                    .to_string(),
                source: entry.alias_of.clone(),
            });
            continue;
        }
        if !refs_by_command.contains_key(&entry.path) {
            orphan_commands.push(entry.path.clone());
            blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::CommandNoTdRef,
                target: entry.path.clone(),
                reason: "public command path is not claimed by any TD command_refs entry"
                    .to_string(),
                source: entry.alias_of.clone(),
            });
        }
    }

    if !command_inventory.is_empty() {
        blockers.extend(active_doc_command_blockers(project_root, command_inventory));
    }
    sort_traceability_blockers(&mut blockers);
    orphan_commands.sort();
    orphan_commands.dedup();

    let traceable_command_paths = public_commands
        .iter()
        .filter(|entry| {
            refs_by_command.get(&entry.path).is_some_and(|td_paths| {
                td_paths.iter().any(|td_path| {
                    records
                        .get(td_path)
                        .is_some_and(|record| record.has_valid_capability_refs)
                })
            })
        })
        .count();
    let total_command_paths = public_commands.len();
    CommandTraceabilityCoverage {
        total_command_paths,
        traceable_command_paths,
        command_traceability_percent: coverage_percent(
            traceable_command_paths,
            total_command_paths,
        ),
        hidden_command_count: command_inventory
            .values()
            .filter(|entry| entry.hidden)
            .count(),
        orphan_command_count: orphan_commands.len(),
        command_ref_count: refs_by_command.values().map(BTreeSet::len).sum(),
        orphan_commands,
        next_gap: blockers.first().cloned(),
        blockers,
    }
}

fn active_doc_command_blockers(
    project_root: &Path,
    command_inventory: &BTreeMap<String, CommandInventoryEntry>,
) -> Vec<TraceabilityBlocker> {
    let mut blockers = Vec::new();
    for path in active_doc_paths(project_root) {
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let rel = rel_display(project_root, &path);
        for deleted in DELETED_COMMAND_PATHS {
            if content.contains(deleted) {
                blockers.push(TraceabilityBlocker {
                    kind: TraceabilityBlockerKind::ActiveDocDeletedCommandRef,
                    target: (*deleted).to_string(),
                    reason: "active docs or skills still reference a deleted command".to_string(),
                    source: Some(rel.clone()),
                });
            }
        }
        for command in extract_top_level_aw_command_refs(&content) {
            if DELETED_COMMAND_PATHS.contains(&command.as_str()) {
                continue;
            }
            if command_inventory
                .keys()
                .any(|known| known.starts_with(&command))
            {
                continue;
            }
            blockers.push(TraceabilityBlocker {
                kind: TraceabilityBlockerKind::ActiveDocUnknownCommandRef,
                target: command,
                reason: "active docs or skills reference a command not registered by clap"
                    .to_string(),
                source: Some(rel.clone()),
            });
        }
    }
    blockers
}

fn active_doc_paths(project_root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for rel in [
        "README.md",
        "AGENTS.md",
        "CLAUDE.md",
        "CONTRIBUTING.md",
        "apps/agentic-workflow/templates/cli/README.md",
        "apps/agentic-workflow/templates/cli/mainthread/CLAUDE.md.tmpl",
    ] {
        let path = project_root.join(rel);
        if path.exists() {
            paths.push(path);
        }
    }
    collect_markdown_paths(
        &project_root.join(".agents/skills"),
        &mut paths,
        Some("aw-"),
    );
    collect_markdown_paths(
        &project_root.join(".claude/skills"),
        &mut paths,
        Some("aw-"),
    );
    collect_markdown_paths(
        &project_root.join("apps/agentic-workflow/templates/cli/mainthread/skills"),
        &mut paths,
        Some("aw-"),
    );
    paths
}

fn collect_markdown_paths(root: &Path, paths: &mut Vec<PathBuf>, dir_prefix: Option<&str>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if dir_prefix.is_some_and(|prefix| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| !name.starts_with(prefix))
            }) {
                continue;
            }
            collect_markdown_paths(&path, paths, None);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            paths.push(path);
        }
    }
}

fn extract_top_level_aw_command_refs(content: &str) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    for line in content.lines() {
        if let Some(command) = extract_top_level_aw_command_ref_from_snippet(line) {
            commands.insert(command);
        }
        let mut rest = line;
        while let Some(start) = rest.find('`') {
            let after_start = &rest[start + 1..];
            let Some(end) = after_start.find('`') else {
                break;
            };
            if let Some(command) =
                extract_top_level_aw_command_ref_from_snippet(&after_start[..end])
            {
                commands.insert(command);
            }
            rest = &after_start[end + 1..];
        }
    }
    commands
}

fn extract_top_level_aw_command_ref_from_snippet(snippet: &str) -> Option<String> {
    let cleaned = snippet
        .trim_start()
        .trim_start_matches(['-', '*', '>', '|'])
        .trim_start()
        .trim_start_matches('$')
        .trim_start()
        .trim_start_matches("command:")
        .trim_start()
        .trim_matches(['`', '"', '\'']);
    let mut words = cleaned.split_whitespace();
    if words.next()? != "aw" {
        return None;
    }
    let raw_subcommand = words.next()?;
    if raw_subcommand.starts_with('<') {
        return None;
    }
    let subcommand = raw_subcommand.trim_matches(|ch: char| {
        !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' && ch != ':'
    });
    if subcommand.is_empty()
        || subcommand.starts_with('-')
        || subcommand.starts_with('<')
        || subcommand.contains(':')
    {
        return None;
    }
    Some(format!("aw {subcommand}"))
}

fn normalize_command_ref(command: &str) -> String {
    command.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sort_traceability_blockers(blockers: &mut Vec<TraceabilityBlocker>) {
    blockers.sort_by(|a, b| {
        traceability_blocker_priority(a.kind)
            .cmp(&traceability_blocker_priority(b.kind))
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.reason.cmp(&b.reason))
    });
    blockers.dedup();
}

fn traceability_blocker_priority(kind: TraceabilityBlockerKind) -> u8 {
    match kind {
        TraceabilityBlockerKind::HiddenCommandRegistered => 0,
        TraceabilityBlockerKind::ActiveDocDeletedCommandRef => 1,
        TraceabilityBlockerKind::CommandRefUnknownCommand => 2,
        TraceabilityBlockerKind::ActiveDocUnknownCommandRef => 3,
        TraceabilityBlockerKind::CommandRefTdNoCapabilityRef => 4,
        TraceabilityBlockerKind::CommandNoTdRef => 5,
        TraceabilityBlockerKind::TdInvalidCapabilityRef => 6,
        TraceabilityBlockerKind::TdMissingPrimaryCapabilityRef => 7,
        TraceabilityBlockerKind::TdChangeInvalidImplMode => 8,
        TraceabilityBlockerKind::TdChangeMissingImplMode => 9,
        TraceabilityBlockerKind::TdChangeInvalidSection => 10,
        TraceabilityBlockerKind::TdChangeMissingSection => 11,
        TraceabilityBlockerKind::TdSectionNoImplementationEdge => 12,
        TraceabilityBlockerKind::InternalTdHasSourceEdge => 13,
        TraceabilityBlockerKind::SourceBlockNoTd => 14,
        TraceabilityBlockerKind::SourceBlockTdNoCapabilityRef => 15,
        TraceabilityBlockerKind::CbBlockTdNoCapabilityRef => 16,
        TraceabilityBlockerKind::TdNoCapabilityRef => 17,
    }
}

fn coverage_percent(done: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        (done as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
fn build_regenerability_coverage(
    project_root: &Path,
    inventory: &Inventory,
    semantic: &SemanticCoverage,
) -> Result<RegenerabilityCoverage> {
    build_regenerability_coverage_with_project(project_root, inventory, semantic, None)
}

#[cfg(test)]
fn build_regenerability_coverage_with_project(
    project_root: &Path,
    inventory: &Inventory,
    semantic: &SemanticCoverage,
    replay_project: Option<&str>,
) -> Result<RegenerabilityCoverage> {
    build_regenerability_coverage_with_options(
        project_root,
        inventory,
        semantic,
        replay_project,
        true,
    )
}

fn build_regenerability_coverage_with_options(
    project_root: &Path,
    inventory: &Inventory,
    semantic: &SemanticCoverage,
    replay_project: Option<&str>,
    verify_codegen_drift: bool,
) -> Result<RegenerabilityCoverage> {
    let mut codegen_files = 0usize;
    let mut eligible_files = 0usize;
    let mut unmarked_files = 0usize;
    let mut gap_files = Vec::new();
    let codegen_drift_files = if verify_codegen_drift {
        collect_codegen_gap_files(project_root, inventory, replay_project)?
    } else {
        Vec::new()
    };
    let codegen_drift_set = codegen_drift_files.iter().collect::<BTreeSet<_>>();
    let non_replayable_codegen_files =
        collect_non_replayable_codegen_files(project_root, inventory)?;
    let non_replayable_codegen_set = non_replayable_codegen_files.iter().collect::<BTreeSet<_>>();
    let snapshot_codegen_files = collect_snapshot_codegen_files(project_root, inventory)?;
    let snapshot_codegen_set = snapshot_codegen_files.iter().collect::<BTreeSet<_>>();
    let semantic_by_source: BTreeMap<_, _> = semantic
        .coverage_map
        .iter()
        .map(|entry| (entry.source_unit.as_str(), entry))
        .collect();
    let mut primitive_covered_files = 0usize;

    for file in &inventory.files {
        eligible_files += 1;
        if !file.markers.managed() {
            unmarked_files += 1;
        }

        let unsupported_codegen =
            file.markers.codegen && !file.markers.handwrite && !codegen_replay_supported(file);
        let codegen_drift = codegen_drift_set.contains(&file.rel);
        let non_replayable_codegen = non_replayable_codegen_set.contains(&file.rel);
        let snapshot_codegen = snapshot_codegen_set.contains(&file.rel);
        let ast_codegen = file.markers.codegen
            && !file.markers.handwrite
            && !unsupported_codegen
            && !non_replayable_codegen
            && !snapshot_codegen
            && !codegen_drift;
        if ast_codegen {
            codegen_files += 1;
        }
        if file.markers.handwrite
            || !file.markers.managed()
            || unsupported_codegen
            || non_replayable_codegen
            || snapshot_codegen
            || codegen_drift
        {
            gap_files.push(file.rel.clone());
        }
        if semantic_by_source
            .get(file.rel.as_str())
            .is_some_and(|entry| {
                entry.td_section.is_some()
                    && !entry.generator_primitives.is_empty()
                    && !entry
                        .generator_primitives
                        .iter()
                        .any(|p| p == "source_unit")
            })
        {
            primitive_covered_files += 1;
        }
    }

    let total_files = inventory.files.len();
    let percent = if eligible_files == 0 {
        100.0
    } else {
        (codegen_files as f64 / eligible_files as f64) * 100.0
    };
    let handwrite_files = eligible_files
        .saturating_sub(codegen_files)
        .saturating_sub(unmarked_files);

    let mut missing_generator_primitive_gaps = 0usize;
    let mut insufficient_td_section_gaps = 0usize;
    let mut human_decision_required_gaps = 0usize;
    for gap in &semantic.generator_primitive_gaps {
        if gap.human_decision_required {
            human_decision_required_gaps += 1;
        } else if gap.primitive == "semantic_td_missing" {
            insufficient_td_section_gaps += 1;
        } else {
            missing_generator_primitive_gaps += 1;
        }
    }

    let authority =
        crate::cli::regenerability_policy::resolve_regenerability_policy(replay_project);

    Ok(RegenerabilityCoverage {
        scope: inventory.coverage.scope.clone(),
        total_files,
        eligible_files,
        codegen_files,
        handwrite_files,
        unmarked_files,
        unsupported_codegen_files: inventory
            .files
            .iter()
            .filter(|file| {
                file.markers.codegen && !file.markers.handwrite && !codegen_replay_supported(file)
            })
            .map(|file| file.rel.clone())
            .collect(),
        non_replayable_codegen_files,
        snapshot_codegen_files,
        codegen_drift_evaluated: verify_codegen_drift,
        codegen_drift_files,
        percent,
        gap_files,
        semantic_percent: semantic.percent,
        generator_primitive_gaps: semantic.generator_primitive_gaps.len(),
        primitive_covered_files,
        missing_generator_primitive_gaps,
        insufficient_td_section_gaps,
        human_decision_required_gaps,
        next_gap: semantic.next_gap.clone(),
        authority_mode: authority.authority,
        required_for_production: authority.required_for_production(),
        authority_reason: authority.reason,
    })
}

fn collect_snapshot_codegen_files(
    project_root: &Path,
    inventory: &Inventory,
) -> Result<Vec<String>> {
    let mut gap_files = BTreeSet::new();
    for file in &inventory.files {
        if !file.markers.codegen || file.markers.handwrite {
            continue;
        }
        if codegen_file_uses_snapshot_or_source_template(project_root, file)? {
            gap_files.insert(file.rel.clone());
        }
    }
    Ok(gap_files.into_iter().collect())
}

fn codegen_file_uses_snapshot_or_source_template(
    project_root: &Path,
    file: &SourceFile,
) -> Result<bool> {
    for source_ref in source_spec_refs_with_sections(&file.abs, project_root) {
        let spec_path = project_root.join(&source_ref.path);
        if !spec_path.exists() {
            continue;
        }
        let spec_content = fs::read_to_string(&spec_path)
            .with_context(|| format!("failed to read {}", spec_path.display()))?;

        if source_snapshot_mentions_path(&spec_content, &file.rel) {
            return Ok(true);
        }
        if source_ref
            .section
            .as_deref()
            .is_some_and(is_source_section_ref)
            && !source_section_has_type_marker(&spec_content, "type: rust-source-unit")
            && !source_section_has_type_marker(&spec_content, "type: text-source-unit")
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn is_source_section_ref(section: &str) -> bool {
    matches!(section, "source" | "rust-source-unit" | "text-source-unit")
}

fn source_snapshot_mentions_path(spec_content: &str, target_rel: &str) -> bool {
    spec_content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("<!-- source-snapshot:"))
        .filter_map(|tail| tail.strip_suffix("-->"))
        .flat_map(|body| body.split_whitespace())
        .filter_map(|part| part.strip_prefix("path="))
        .any(|path| path.trim_matches('"') == target_rel)
}

fn source_section_has_type_marker(spec_content: &str, marker: &str) -> bool {
    let mut in_source = false;
    for line in spec_content.lines() {
        if line.starts_with("## ") {
            let heading = line.trim_start_matches('#').trim();
            in_source = heading.eq_ignore_ascii_case("Source");
            continue;
        }
        if in_source && line.trim().contains(marker) {
            return true;
        }
    }
    false
}

fn collect_non_replayable_codegen_files(
    project_root: &Path,
    inventory: &Inventory,
) -> Result<Vec<String>> {
    let mut gap_files = BTreeSet::new();
    for file in &inventory.files {
        if !file.markers.codegen || file.markers.handwrite {
            continue;
        }
        if codegen_file_has_hand_written_td_entry(project_root, file)? {
            gap_files.insert(file.rel.clone());
        }
    }
    Ok(gap_files.into_iter().collect())
}

fn codegen_file_has_hand_written_td_entry(project_root: &Path, file: &SourceFile) -> Result<bool> {
    for source_ref in source_spec_refs_with_sections(&file.abs, project_root) {
        let spec_path = project_root.join(&source_ref.path);
        if !spec_path.exists() {
            continue;
        }
        let spec_content = fs::read_to_string(&spec_path)
            .with_context(|| format!("failed to read {}", spec_path.display()))?;
        if crate::generate::apply::extract_change_entries(&spec_content)
            .into_iter()
            .any(|entry| {
                entry.path == file.rel
                    && change_entry_matches_source_ref_section(
                        entry.section_id.as_deref(),
                        source_ref.section.as_deref(),
                    )
                    && entry.impl_mode == crate::generate::apply::ImplMode::HandWritten
            })
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn change_entry_matches_source_ref_section(
    entry_section: Option<&str>,
    source_ref_section: Option<&str>,
) -> bool {
    match (entry_section, source_ref_section) {
        (Some(entry), Some(source_ref)) => entry == source_ref,
        (None, Some(_)) => true,
        (_, None) => true,
    }
}

fn collect_codegen_gap_files(
    project_root: &Path,
    inventory: &Inventory,
    replay_project: Option<&str>,
) -> Result<Vec<String>> {
    let mut gap_files = collect_codegen_audit_gap_files(project_root, inventory)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    if let Some(project) = replay_project {
        gap_files.extend(collect_force_regen_replay_gap_files(project, inventory)?);
    }
    Ok(gap_files.into_iter().collect())
}

fn collect_codegen_audit_gap_files(
    project_root: &Path,
    inventory: &Inventory,
) -> Result<Vec<String>> {
    use crate::generate::audit::{audit_file_unified, build_spec_file_index, UnifiedReport};

    let spec_index =
        build_spec_file_index(project_root).context("failed to build TD spec index")?;
    let mut gap_files = BTreeSet::new();
    for file in &inventory.files {
        if !file.markers.codegen || file.markers.handwrite || !codegen_replay_supported(file) {
            continue;
        }
        if !file.abs.exists() {
            continue;
        }
        let reports = audit_file_unified(&file.abs, project_root, &spec_index)
            .with_context(|| format!("failed to audit {}", file.rel))?;
        if reports.iter().any(|finding| {
            matches!(
                finding,
                UnifiedReport::Drift { .. }
                    | UnifiedReport::MarkerGap { .. }
                    | UnifiedReport::Uncovered { .. }
                    | UnifiedReport::Unresolvable { .. }
            )
        }) {
            gap_files.insert(file.rel.clone());
        }
    }
    Ok(gap_files.into_iter().collect())
}

fn collect_force_regen_replay_gap_files(
    project: &str,
    inventory: &Inventory,
) -> Result<Vec<String>> {
    let inventory_paths = inventory
        .files
        .iter()
        .map(|file| file.rel.as_str())
        .collect::<BTreeSet<_>>();
    let summary = crate::cli::cb::project_force_regen_verify_summary(project)
        .with_context(|| format!("failed to verify project replay for {project}"))?;
    let gap_files = summary
        .failures
        .iter()
        .filter_map(|failure| extract_force_regen_replay_failure_path(failure))
        .filter(|path| inventory_paths.contains(path.as_str()))
        .collect::<BTreeSet<_>>();
    Ok(gap_files.into_iter().collect())
}

fn extract_force_regen_replay_failure_path(failure: &str) -> Option<String> {
    failure
        .strip_suffix(": differs after TD replay")
        .map(str::to_string)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TdCoverageRecord {
    path: String,
    is_claim: bool,
    needs_migration: bool,
    source_paths: BTreeSet<String>,
}

fn collect_td_index(
    project_root: &Path,
    scopes: &[String],
) -> Result<BTreeMap<String, TdCoverageRecord>> {
    let mut out = BTreeMap::new();
    for root in spec_roots_for_scopes(project_root, scopes)? {
        let mut files = Vec::new();
        if root.is_file() {
            files.push(root);
        } else if root.is_dir() {
            for entry in walkdir::WalkDir::new(&root)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|entry| entry.file_type().is_file())
            {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("md") {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
        for file in files {
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let rel = rel_display(project_root, &file);
            out.insert(rel.clone(), td_coverage_record(&rel, &content));
        }
    }
    Ok(out)
}

fn td_coverage_record(path: &str, content: &str) -> TdCoverageRecord {
    let is_semantic = content.contains("coverage_kind: semantic");
    let has_claim_marker = content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "type: claim" || trimmed == "type: \"claim\""
    });
    let has_legacy_handwrite_claim_marker = !is_semantic
        && traceability_change_entries(content)
            .ok()
            .is_some_and(|entries| {
                entries.iter().any(|entry| {
                    entry.path.is_some()
                        && matches!(entry.impl_mode.as_deref(), Some("hand-written"))
                })
            });
    let is_claim = has_claim_marker || has_legacy_handwrite_claim_marker;
    let needs_migration = semantic_td_needs_section_type_migration(content)
        || semantic_td_needs_source_evidence_graph_migration(content)
        || semantic_td_needs_impl_mode_migration(content);
    let mut source_paths = BTreeSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let value = trimmed
            .strip_prefix("- path:")
            .or_else(|| trimmed.strip_prefix("path:"));
        let Some(value) = value else {
            continue;
        };
        let source = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_end_matches(',');
        if !source.is_empty() && !source.contains('*') {
            source_paths.insert(source.to_string());
        }
    }
    TdCoverageRecord {
        path: path.to_string(),
        is_claim,
        needs_migration,
        source_paths,
    }
}

fn primitive_gap_priority(primitive: &str) -> u8 {
    match primitive {
        "semantic_td_missing" => 0,
        "semantic_td_legacy" => 1,
        _ => 2,
    }
}

fn semantic_td_needs_section_type_migration(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "type: semantic" || trimmed == "type: \"semantic\"" || trimmed == "## Source IR"
    })
}

fn semantic_td_needs_source_evidence_graph_migration(content: &str) -> bool {
    content.contains("coverage_kind: semantic")
        && content.contains("source_units:")
        && !content.contains("source_evidence_node:")
}

fn semantic_td_needs_impl_mode_migration(content: &str) -> bool {
    content.contains("coverage_kind: semantic")
        && content.contains("## Changes")
        && content.contains("changes:")
        && !content.contains("impl_mode:")
}

fn semantic_td_needs_generated_capability_ref_migration(
    project_root: &Path,
    configured: &[ConfiguredScope],
    target: &str,
    content: &str,
) -> bool {
    content.contains("coverage_kind: semantic")
        && !content.contains("capability_refs:")
        && !content.contains("capability_scope:")
        && semantic_capability_ref_for_group(project_root, configured, &semantic_group_key(target))
            .is_some()
}

fn source_spec_refs(abs: &Path, project_root: &Path) -> Vec<String> {
    source_spec_refs_with_sections(abs, project_root)
        .into_iter()
        .map(|source_ref| source_ref.path)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourceSpecRef {
    path: String,
    section: Option<String>,
}

fn source_spec_refs_with_sections(abs: &Path, project_root: &Path) -> Vec<SourceSpecRef> {
    let Ok(content) = fs::read_to_string(abs) else {
        return Vec::new();
    };
    let mut refs = Vec::new();
    for line in content.lines() {
        let Some((_, rest)) = line.split_once("SPEC-MANAGED:") else {
            continue;
        };
        if let Some(source_ref) = parse_source_spec_ref(rest, project_root) {
            refs.push(source_ref);
        }
    }
    refs.sort();
    refs.dedup();
    refs
}

fn parse_source_spec_ref(raw: &str, project_root: &Path) -> Option<SourceSpecRef> {
    let value = raw
        .trim()
        .trim_end_matches("-->")
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches('"')
        .trim_matches('\'');
    if value.is_empty() {
        return None;
    }
    let (path, section) = match value.split_once('#') {
        Some((path, section)) => (path, Some(section.trim().to_string())),
        None => (value, None),
    };
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    let normalized =
        if let Some(stripped) = path.strip_prefix(project_root.to_string_lossy().as_ref()) {
            stripped.trim_start_matches('/').to_string()
        } else {
            path.to_string()
        };
    Some(SourceSpecRef {
        path: normalized,
        section: section.filter(|value| !value.is_empty()),
    })
}

fn build_source_ir_for_files(files: &[&SourceFile]) -> Vec<SourceUnit> {
    let mut analyzer = crate::fillback::AstAnalyzer::new().ok();
    let mut units = Vec::new();
    for file in files.iter().copied() {
        let mut symbols = Vec::new();
        let mut imports = Vec::new();
        let content = fs::read_to_string(&file.abs).ok();
        if let (Some(analyzer), Some(content)) = (analyzer.as_mut(), content.as_deref()) {
            if let Ok(module) = analyzer.parse_file(&file.abs, content) {
                symbols = module
                    .symbols
                    .into_iter()
                    .map(|symbol| SourceSymbol {
                        name: symbol.name,
                        kind: symbol.kind.to_string(),
                        public: symbol.is_public,
                    })
                    .collect();
                imports = module
                    .imports
                    .into_iter()
                    .map(|import| ImportEdge {
                        path: import.path,
                        items: import.items,
                        external: import.is_external,
                    })
                    .collect();
            }
        }
        let generator_primitives = infer_generator_primitives(file, &symbols, content.as_deref());
        let frontend_node = frontend_source_node(file, content.as_deref());
        let source_evidence_node = build_source_evidence_node(
            file,
            &symbols,
            &generator_primitives,
            frontend_node.as_ref(),
        );
        units.push(SourceUnit {
            path: file.rel.clone(),
            language: file.language.clone(),
            symbols,
            imports,
            generator_primitives,
            managed_state: ownership_state(&file.markers).to_string(),
            source_evidence_node,
            frontend_node,
        });
    }
    units
}

fn build_source_evidence_graph(source_ir: &[SourceUnit]) -> Option<SourceEvidenceGraph> {
    let mut source_nodes: Vec<_> = source_ir
        .iter()
        .filter_map(|unit| unit.source_evidence_node.clone())
        .collect();
    if source_nodes.is_empty() {
        return None;
    }
    source_nodes.sort_by(|a, b| a.path.cmp(&b.path));

    let mut domain_map: BTreeMap<String, Vec<&SourceEvidenceNode>> = BTreeMap::new();
    for node in &source_nodes {
        domain_map
            .entry(node.domain.clone())
            .or_default()
            .push(node);
    }
    let domains = domain_map
        .into_iter()
        .map(|(key, nodes)| {
            let mut layers: Vec<String> = nodes.iter().map(|node| node.layer.clone()).collect();
            layers.sort();
            layers.dedup();
            let mut section_types: Vec<String> =
                nodes.iter().map(|node| node.section_type.clone()).collect();
            section_types.sort();
            section_types.dedup();
            SourceEvidenceDomain {
                key,
                layers,
                section_types,
                source_count: nodes.len(),
            }
        })
        .collect();

    Some(SourceEvidenceGraph {
        domains,
        source_nodes,
    })
}

fn build_source_evidence_node(
    file: &SourceFile,
    symbols: &[SourceSymbol],
    primitives: &[String],
    frontend_node: Option<&FrontendSourceNode>,
) -> Option<SourceEvidenceNode> {
    if let Some(node) = frontend_node {
        return Some(SourceEvidenceNode {
            path: file.rel.clone(),
            layer: "frontend".to_string(),
            ecosystem: frontend_ecosystem_label(file).to_string(),
            role: node.role.clone(),
            section_type: node.section_type.clone(),
            domain: semantic_group_key(&file.rel),
            workspace_root: Some(node.workspace_root.clone()),
            route: node.route.clone(),
            component: node.component.clone(),
        });
    }

    if is_operations_language(&file.language) {
        let section_type = if matches!(file.language.as_str(), "dockerfile" | "dockerignore") {
            "runtime-image"
        } else {
            "deployment"
        };
        return Some(SourceEvidenceNode {
            path: file.rel.clone(),
            layer: "operations".to_string(),
            ecosystem: file.language.clone(),
            role: operation_artifact_kind(file).to_string(),
            section_type: section_type.to_string(),
            domain: semantic_group_key(&file.rel),
            workspace_root: None,
            route: None,
            component: None,
        });
    }

    if file.language == "python" {
        let role = python_source_evidence_role(&file.rel, symbols, primitives);
        return Some(SourceEvidenceNode {
            path: file.rel.clone(),
            layer: if role == "test" { "test" } else { "backend" }.to_string(),
            ecosystem: "python".to_string(),
            section_type: python_source_evidence_section_type(&role).to_string(),
            role,
            domain: semantic_group_key(&file.rel),
            workspace_root: None,
            route: None,
            component: None,
        });
    }

    if file.language == "rust" || file.language == "go" {
        return Some(SourceEvidenceNode {
            path: file.rel.clone(),
            layer: "backend".to_string(),
            ecosystem: file.language.clone(),
            role: if primitives.iter().any(|primitive| primitive == "test_case") {
                "test".to_string()
            } else {
                "source".to_string()
            },
            section_type: if primitives.iter().any(|primitive| primitive == "test_case") {
                "unit-test".to_string()
            } else {
                "schema".to_string()
            },
            domain: semantic_group_key(&file.rel),
            workspace_root: None,
            route: None,
            component: None,
        });
    }

    Some(SourceEvidenceNode {
        path: file.rel.clone(),
        layer: "source".to_string(),
        ecosystem: file.language.clone(),
        role: "source".to_string(),
        section_type: "schema".to_string(),
        domain: semantic_group_key(&file.rel),
        workspace_root: None,
        route: None,
        component: None,
    })
}

fn frontend_ecosystem_label(file: &SourceFile) -> &'static str {
    match file.language.as_str() {
        "typescript" => "typescript-jsx",
        "javascript" => "javascript",
        "stylesheet" => "style",
        "json" => "config",
        _ => "frontend",
    }
}

fn python_source_evidence_role(
    rel: &str,
    symbols: &[SourceSymbol],
    primitives: &[String],
) -> String {
    if primitives
        .iter()
        .any(|primitive| primitive == "pytest_case")
        || rel
            .split('/')
            .any(|segment| segment == "tests" || segment == "test" || segment.starts_with("test_"))
    {
        "test".to_string()
    } else if primitives.iter().any(|primitive| {
        matches!(
            primitive.as_str(),
            "fastapi_decorator_route" | "fastapi_class_route"
        )
    }) {
        "api".to_string()
    } else if primitives.iter().any(|primitive| {
        matches!(
            primitive.as_str(),
            "pydantic_model" | "beanie_document" | "sqlalchemy_model" | "python_data_model"
        )
    }) {
        "schema".to_string()
    } else if symbols.iter().any(|symbol| symbol.kind == "function") {
        "service".to_string()
    } else {
        "source".to_string()
    }
}

fn python_source_evidence_section_type(role: &str) -> &'static str {
    match role {
        "test" => "unit-test",
        "api" | "service" => "logic",
        "schema" | "source" => "schema",
        _ => "schema",
    }
}

fn infer_generator_primitives(
    file: &SourceFile,
    symbols: &[SourceSymbol],
    content: Option<&str>,
) -> Vec<String> {
    let mut primitives = BTreeSet::new();
    if file.rel.contains("/tests/")
        || file.rel.contains("/test/")
        || file
            .rel
            .rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("test_"))
    {
        primitives.insert("test_case".to_string());
    }
    if file.language == "typescript" && file.rel.ends_with(".tsx") {
        primitives.insert("ts_component".to_string());
    }
    if matches!(file.language.as_str(), "dockerfile" | "dockerignore") {
        primitives.insert("runtime_image".to_string());
    }
    if file.language == "kustomize" {
        primitives.insert("kustomize_manifest".to_string());
    }
    if let Some(node) = frontend_source_node(file, content) {
        primitives.insert(format!("frontend_{}", node.artifact_kind));
        primitives.insert(format!(
            "td_section_{}",
            node.section_type.replace('-', "_")
        ));
    }
    if file.language == "python" {
        if let Some(content) = content {
            if content.contains("@api_router.")
                || content.contains("@router.")
                || content.contains("fastapi.APIRouter")
                || content.contains("APIRouter(")
            {
                primitives.insert("fastapi_decorator_route".to_string());
            }
            if content.contains("BaseAPIRoute") || content.contains(".to_api_router(") {
                primitives.insert("fastapi_class_route".to_string());
            }
            if content.contains("pydantic.BaseModel")
                || content.contains("BaseAPIRequestModel")
                || content.contains("BaseAPIResponseModel")
            {
                primitives.insert("pydantic_model".to_string());
            }
            if content.contains("beanie.Document") || content.contains("core_db_models.Document") {
                primitives.insert("beanie_document".to_string());
            }
            if content.contains("mapped_column(")
                || content.contains("sqlalchemy")
                || content.contains("alloydb_models.Base")
            {
                primitives.insert("sqlalchemy_model".to_string());
            }
            if content.contains("pytest")
                || content.contains("def test_")
                || content.contains("async def test_")
            {
                primitives.insert("pytest_case".to_string());
            }
        }
    }
    for symbol in symbols {
        match symbol.kind.as_str() {
            "struct" | "class" => {
                if file.language == "python" && symbol.name.ends_with("Service") {
                    primitives.insert("service_class".to_string());
                } else if file.language == "python" && symbol.name.ends_with("Mixin") {
                    primitives.insert("service_mixin".to_string());
                } else if file.language == "python" {
                    primitives.insert("python_data_model".to_string());
                } else {
                    primitives.insert("data_model".to_string());
                }
            }
            "enum" => {
                primitives.insert("enum_model".to_string());
            }
            "function" => {
                if symbol.name.starts_with("test_") {
                    primitives.insert("test_case".to_string());
                } else {
                    primitives.insert("service_method".to_string());
                }
            }
            "interface" | "type" => {
                primitives.insert("ts_type_surface".to_string());
            }
            "constant" => {
                primitives.insert("config_surface".to_string());
            }
            _ => {}
        }
    }
    if primitives.is_empty() {
        primitives.insert("source_unit".to_string());
    }
    primitives.into_iter().collect()
}

fn build_frontend_ecosystem_ast(inventory: &Inventory) -> Option<FrontendEcosystemAst> {
    let mut source_nodes = Vec::new();
    for file in &inventory.files {
        let content = fs::read_to_string(&file.abs).ok();
        if let Some(node) = frontend_source_node(file, content.as_deref()) {
            source_nodes.push(node);
        }
    }
    if source_nodes.is_empty() {
        return None;
    }

    let mut workspace_roots: BTreeSet<String> = source_nodes
        .iter()
        .map(|node| node.workspace_root.clone())
        .collect();
    for root in source_nodes.iter().filter_map(|node| {
        if node.artifact_kind == "workspace-manifest" {
            Some(node.workspace_root.clone())
        } else {
            None
        }
    }) {
        workspace_roots.insert(root);
    }

    let mut workspaces = Vec::new();
    for root in workspace_roots {
        workspaces.push(frontend_workspace_node(inventory, &root));
    }
    workspaces.sort_by(|a, b| a.root.cmp(&b.root));
    source_nodes.sort_by(|a, b| a.path.cmp(&b.path));

    Some(FrontendEcosystemAst {
        workspaces,
        source_nodes,
    })
}

fn frontend_workspace_node(inventory: &Inventory, root: &str) -> FrontendWorkspaceNode {
    let package_json_rel = format!("{}/package.json", root.trim_end_matches('/'));
    let package_name = inventory
        .files
        .iter()
        .find(|file| file.rel == package_json_rel)
        .and_then(|file| fs::read_to_string(&file.abs).ok())
        .and_then(|content| json_string_field(&content, "name"));
    let framework = detect_frontend_framework(inventory, root);
    FrontendWorkspaceNode {
        root: root.to_string(),
        kind: frontend_workspace_kind(root).to_string(),
        package_name,
        framework,
    }
}

fn frontend_source_node(file: &SourceFile, content: Option<&str>) -> Option<FrontendSourceNode> {
    if !is_frontend_ecosystem_candidate(file) {
        return None;
    }
    let workspace_root = frontend_workspace_root(&file.rel)?;
    let classification = classify_frontend_source(file, content)?;
    Some(FrontendSourceNode {
        path: file.rel.clone(),
        workspace_root,
        role: classification.role,
        section_type: classification.section_type,
        artifact_kind: classification.artifact_kind,
        route: frontend_route_path(&file.rel),
        component: frontend_component_name(&file.rel),
    })
}

#[derive(Debug, Clone)]
struct FrontendSourceClassification {
    role: String,
    section_type: String,
    artifact_kind: String,
}

fn classify_frontend_source(
    file: &SourceFile,
    content: Option<&str>,
) -> Option<FrontendSourceClassification> {
    let path = Path::new(&file.rel);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let lower_name = file_name.to_ascii_lowercase();
    let rel = file.rel.as_str();

    let (role, section_type, artifact_kind) = if matches!(
        lower_name.as_str(),
        "package.json" | "project.json" | "nx.json"
    ) {
        ("manifest", "manifest", "workspace-manifest")
    } else if is_frontend_config_path(path) {
        ("config", "config", "workspace-config")
    } else if file.language == "stylesheet" {
        ("style", "design-token", "style-surface")
    } else if is_frontend_test_path(rel) {
        ("test", "unit-test", "test")
    } else if lower_name.contains(".stories.") {
        ("story", "component", "storybook-story")
    } else if is_frontend_route_path(rel) {
        ("route", "wireframe", "route")
    } else if file.language == "typescript" && rel.ends_with(".tsx") {
        ("component", "component", "component")
    } else if is_frontend_type_surface(rel) {
        ("type-surface", "schema", "type-surface")
    } else if is_frontend_logic_surface(rel, content) {
        ("logic", "logic", "logic")
    } else if matches!(file.language.as_str(), "typescript" | "javascript") {
        ("source", "schema", "source-unit")
    } else {
        return None;
    };

    Some(FrontendSourceClassification {
        role: role.to_string(),
        section_type: section_type.to_string(),
        artifact_kind: artifact_kind.to_string(),
    })
}

fn is_frontend_ecosystem_candidate(file: &SourceFile) -> bool {
    if matches!(
        file.language.as_str(),
        "typescript" | "javascript" | "json" | "stylesheet"
    ) {
        return true;
    }
    let path = Path::new(&file.rel);
    is_frontend_config_path(path) || is_frontend_manifest_json_path(path)
}

fn is_frontend_manifest_json_path(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("package.json" | "project.json" | "nx.json")
    )
}

fn is_frontend_config_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if is_frontend_manifest_json_path(path) {
        return false;
    }
    let lower = file_name.to_ascii_lowercase();
    lower == "tsconfig.json"
        || lower == "tsconfig.base.json"
        || lower == "next-env.d.ts"
        || lower == ".eslintrc.json"
        || lower == ".stylelintrc.json"
        || lower == ".prettierrc.json"
        || lower == ".prettierrc.js"
        || lower == "jest.config.ts"
        || lower == "jest.config.js"
        || lower == "jest.preset.js"
        || lower == "jest.transform.js"
        || lower == "webpack.config.js"
        || lower == "next.config.js"
        || lower == "postcss.config.js"
        || lower == "postcss.config.mjs"
        || lower == "tailwind.config.js"
        || lower == "tailwind.config.ts"
        || lower == "vite.config.ts"
        || lower == "playwright.config.ts"
        || lower == "cypress.config.ts"
        || lower == "rollup.config.js"
}

fn frontend_workspace_root(rel: &str) -> Option<String> {
    let parts: Vec<&str> = rel.split('/').collect();
    for marker in ["apps", "libs", "packages"] {
        if let Some(idx) = parts.iter().position(|part| *part == marker) {
            if idx + 1 < parts.len() {
                return Some(parts[..=idx + 1].join("/"));
            }
        }
    }
    if let Some(idx) = parts.iter().position(|part| *part == "frontend") {
        return Some(parts[..=idx].join("/"));
    }
    rel.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .filter(|parent| !parent.is_empty())
}

fn frontend_workspace_kind(root: &str) -> &'static str {
    let parts: Vec<&str> = root.split('/').collect();
    if root.ends_with("-e2e") || parts.iter().any(|part| *part == "e2e") {
        "e2e"
    } else if parts.iter().any(|part| *part == "apps") {
        "app"
    } else if parts.iter().any(|part| *part == "libs") {
        "library"
    } else {
        "monorepo"
    }
}

fn detect_frontend_framework(inventory: &Inventory, root: &str) -> Option<String> {
    let root_prefix = format!("{}/", root.trim_end_matches('/'));
    if inventory
        .files
        .iter()
        .any(|file| file.rel.starts_with(&root_prefix) && file.rel.ends_with("next.config.js"))
        || inventory
            .files
            .iter()
            .any(|file| file.rel.starts_with(&root_prefix) && file.rel.contains("/src/app/"))
    {
        return Some("next".to_string());
    }
    let package_json_rel = format!("{}package.json", root_prefix);
    let package_content = inventory
        .files
        .iter()
        .find(|file| file.rel == package_json_rel)
        .and_then(|file| fs::read_to_string(&file.abs).ok());
    if package_content
        .as_deref()
        .is_some_and(|content| content.contains("\"react\"") || content.contains("\"react-dom\""))
        || inventory
            .files
            .iter()
            .any(|file| file.rel.starts_with(&root_prefix) && file.rel.ends_with(".tsx"))
    {
        return Some("react".to_string());
    }
    None
}

fn json_string_field(content: &str, field: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(content).ok()?;
    value.get(field)?.as_str().map(str::to_string)
}

fn is_frontend_test_path(rel: &str) -> bool {
    let file_name = rel.rsplit('/').next().unwrap_or(rel);
    file_name.contains(".test.")
        || file_name.contains(".spec.")
        || file_name.contains(".cy.")
        || rel.contains("/__tests__/")
        || rel.contains("/e2e/")
        || rel.contains("-e2e/")
}

fn is_frontend_route_path(rel: &str) -> bool {
    let file_name = rel.rsplit('/').next().unwrap_or(rel);
    matches!(
        file_name,
        "page.tsx" | "layout.tsx" | "template.tsx" | "error.tsx" | "not-found.tsx"
    ) || rel.contains("/src/app/")
        || rel.ends_with("router.config.tsx")
        || rel.ends_with("router.config.ts")
}

fn is_frontend_type_surface(rel: &str) -> bool {
    rel.ends_with(".d.ts")
        || rel.contains("/types/")
        || rel.contains("/models/")
        || rel.contains("/interfaces/")
        || rel.rsplit('/').next().is_some_and(|name| {
            name == "types.ts" || name == "models.ts" || name == "interfaces.ts"
        })
}

fn is_frontend_logic_surface(rel: &str, content: Option<&str>) -> bool {
    rel.contains("/hooks/")
        || rel.contains("/services/")
        || rel.contains("/utils/")
        || rel.contains("/view-models/")
        || rel.contains("/mocks/")
        || content.is_some_and(|content| {
            content.contains("function ")
                || content.contains("=>")
                || content.contains("useMemo(")
                || content.contains("useCallback(")
        })
}

fn frontend_route_path(rel: &str) -> Option<String> {
    if !is_frontend_route_path(rel) {
        return None;
    }
    if let Some((_, route)) = rel.split_once("/src/app/") {
        let route = route
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or(route)
            .trim_matches('/');
        if route.is_empty() {
            return Some("/".to_string());
        }
        return Some(format!("/{}", route));
    }
    rel.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .or_else(|| Some(rel.to_string()))
}

fn frontend_component_name(rel: &str) -> Option<String> {
    if !rel.ends_with(".tsx") {
        return None;
    }
    let path = Path::new(rel);
    let stem = path.file_stem().and_then(|name| name.to_str())?;
    let raw = if matches!(stem, "index" | "page" | "layout" | "template") {
        path.parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or(stem)
    } else {
        stem
    };
    Some(to_pascal_identifier(raw))
}

fn to_pascal_identifier(raw: &str) -> String {
    let mut out = String::new();
    let mut uppercase_next = true;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            if uppercase_next {
                out.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                out.push(ch);
            }
        } else {
            uppercase_next = true;
        }
    }
    if out.is_empty() {
        "Component".to_string()
    } else {
        out
    }
}

fn ownership_state(markers: &FileMarkers) -> &'static str {
    match (markers.codegen, markers.handwrite) {
        (true, true) => "mixed",
        (true, false) => "codegen",
        (false, true) => "handwrite",
        (false, false) => "unmanaged",
    }
}

fn semantic_gap_action(primitive: &str) -> &'static str {
    match primitive {
        "semantic_td_missing" => "draft_or_update_semantic_td_from_source_ir",
        "semantic_td_legacy" => "rewrite_semantic_td_to_section_types",
        _ => "extend_generator_primitive_or_keep_tracked_handwrite",
    }
}

fn resolve_scopes(
    project_root: &Path,
    explicit_scopes: &[String],
    project: Option<&str>,
    all: bool,
) -> Result<Vec<String>> {
    if !explicit_scopes.is_empty() {
        if project.is_some() || all {
            bail!("use either PROJECT, --all, or --scope; do not combine them");
        }
        return Ok(explicit_scopes.to_vec());
    }

    if project.is_some() && all {
        bail!("use either PROJECT or --all; do not combine them");
    }

    let configured = read_config_workspace_scopes(project_root)?;
    if let Some(project_name) = project {
        return resolve_project_scopes(project_root, &configured, project_name);
    }

    if !all {
        let project_names: BTreeSet<_> = configured
            .iter()
            .filter_map(|scope| scope.project_name.as_deref())
            .collect();
        if project_names.len() == 1 {
            if let Some(project_name) = project_names.iter().next() {
                return resolve_project_scopes(project_root, &configured, project_name);
            }
        } else if project_names.len() > 1 {
            bail!(
                "standardize requires --project <project> in multi-project repos (for example: aw standardize managed run --project sdd), or use --all/--scope. Available projects: {}",
                project_names.into_iter().collect::<Vec<_>>().join(", ")
            );
        }
    }

    resolve_all_scopes(project_root, &configured)
}

fn resolve_project_scopes(
    project_root: &Path,
    configured: &[ConfiguredScope],
    project_name: &str,
) -> Result<Vec<String>> {
    let mut scopes = Vec::new();
    let mut matched_config = false;
    let mut stale_config = false;
    let canonical_project_name = canonical_configured_project_name(configured, project_name);
    for configured_scope in configured
        .iter()
        .filter(|scope| configured_scope_matches_project(scope, project_name))
    {
        matched_config = true;
        if scope_walk_root(project_root, &configured_scope.scope).exists() {
            scopes.push(configured_scope.scope.clone());
        } else {
            stale_config = true;
        }
    }

    if scopes.is_empty() || stale_config {
        if let Ok(projects) = crate::services::project_discovery::discover_projects(project_root) {
            for project in projects {
                if project.name != canonical_project_name {
                    continue;
                }
                for workspace in project.workspaces {
                    scopes.extend(workspace.paths);
                }
            }
        }
    }

    if scopes.is_empty() {
        if matched_config {
            bail!(
                "standardize project `{}` has no existing workspace paths",
                project_name
            );
        }
        let available: BTreeSet<_> = configured
            .iter()
            .filter_map(|scope| scope.project_name.as_deref())
            .collect();
        if available.is_empty() {
            bail!("unknown standardize project `{}`", project_name);
        }
        bail!(
            "unknown standardize project `{}`. Available projects: {}",
            project_name,
            available.into_iter().collect::<Vec<_>>().join(", ")
        );
    }

    scopes.sort();
    scopes.dedup();
    Ok(scopes)
}

fn resolve_standardize_project_name(project_root: &Path, project_name: &str) -> Result<String> {
    let configured = read_config_workspace_scopes(project_root)?;
    Ok(canonical_configured_project_name(&configured, project_name).to_string())
}

fn canonical_configured_project_name<'a>(
    configured: &'a [ConfiguredScope],
    project_name: &'a str,
) -> &'a str {
    configured
        .iter()
        .find(|scope| configured_scope_matches_project(scope, project_name))
        .and_then(|scope| scope.project_name.as_deref())
        .unwrap_or(project_name)
}

fn configured_scope_matches_project(scope: &ConfiguredScope, project_name: &str) -> bool {
    scope.project_name.as_deref() == Some(project_name)
        || scope.aliases.iter().any(|alias| alias == project_name)
}

fn resolve_all_scopes(project_root: &Path, configured: &[ConfiguredScope]) -> Result<Vec<String>> {
    let mut scopes = Vec::new();
    let mut stale_project_names = BTreeSet::new();
    for configured_scope in configured {
        if scope_walk_root(project_root, &configured_scope.scope).exists() {
            scopes.push(configured_scope.scope.clone());
        } else if let Some(project_name) = configured_scope.project_name.as_deref() {
            stale_project_names.insert(project_name.to_string());
        }
    }

    if !stale_project_names.is_empty() {
        if let Ok(projects) = crate::services::project_discovery::discover_projects(project_root) {
            for project in projects {
                if !stale_project_names.contains(&project.name) {
                    continue;
                }
                for workspace in project.workspaces {
                    scopes.extend(workspace.paths);
                }
            }
        }
    }

    if scopes.is_empty() {
        if let Ok(projects) = crate::services::project_discovery::discover_projects(project_root) {
            for project in projects {
                for workspace in project.workspaces {
                    scopes.extend(workspace.paths);
                }
            }
        }
    }
    if scopes.is_empty() {
        scopes.push("**".to_string());
    }
    scopes.sort();
    scopes.dedup();
    Ok(scopes)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfiguredScope {
    project_name: Option<String>,
    aliases: Vec<String>,
    project_path: Option<String>,
    scope: String,
    td_path: Option<String>,
    cap_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfiguredWorkspace {
    project_name: Option<String>,
    name: Option<String>,
    paths: Vec<String>,
    target: Option<String>,
    test_cmd: Option<String>,
}

fn read_config_workspace_scopes(project_root: &Path) -> Result<Vec<ConfiguredScope>> {
    let path = project_root.join("aw.toml");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    let mut out = Vec::new();
    let Some(projects) = value.get("projects").and_then(|v| v.as_array()) else {
        return Ok(out);
    };
    for project in projects {
        let project_name = project
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let aliases = project
            .get("aliases")
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let project_path = project
            .get("path")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let td_path = project
            .get("td_path")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let cap_path = project
            .get("cap_path")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let Some(workspaces) = project.get("workspaces").and_then(|v| v.as_array()) else {
            continue;
        };
        for workspace in workspaces {
            let Some(paths) = workspace.get("paths").and_then(|v| v.as_array()) else {
                continue;
            };
            for p in paths {
                if let Some(s) = p.as_str() {
                    out.push(ConfiguredScope {
                        project_name: project_name.clone(),
                        aliases: aliases.clone(),
                        project_path: project_path.clone(),
                        scope: s.to_string(),
                        td_path: td_path.clone(),
                        cap_path: cap_path.clone(),
                    });
                }
            }
        }
    }
    Ok(out)
}

fn configured_td_path(scope: &ConfiguredScope) -> Option<String> {
    scope.td_path.clone().or_else(|| {
        scope.project_path.as_deref().map(|project_path| {
            crate::services::project_registry::default_project_td_path(project_path)
                .to_string_lossy()
                .into_owned()
        })
    })
}

fn configured_td_root(project_root: &Path, scope: &ConfiguredScope) -> Option<PathBuf> {
    configured_td_path(scope).map(|td_path| project_root.join(td_path))
}

fn read_config_workspaces(project_root: &Path) -> Result<Vec<ConfiguredWorkspace>> {
    let path = project_root.join("aw.toml");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    let mut out = Vec::new();
    let Some(projects) = value.get("projects").and_then(|v| v.as_array()) else {
        return Ok(out);
    };
    for project in projects {
        let project_name = project
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let Some(workspaces) = project.get("workspaces").and_then(|v| v.as_array()) else {
            continue;
        };
        for workspace in workspaces {
            let paths: Vec<String> = workspace
                .get("paths")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|p| p.as_str().map(str::to_string))
                .collect();
            if paths.is_empty() {
                continue;
            }
            out.push(ConfiguredWorkspace {
                project_name: project_name.clone(),
                name: workspace
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                paths,
                target: workspace
                    .get("target")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                test_cmd: workspace
                    .get("test_cmd")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            });
        }
    }
    Ok(out)
}

fn build_stack_migration_coverage(
    project_root: &Path,
    project: &str,
) -> Result<StackMigrationCoverage> {
    let inventory = build_inventory(project_root, &[], Some(project), false)?;
    build_stack_migration_coverage_with_inventory(project_root, project, &inventory)
}

fn build_stack_migration_coverage_with_inventory(
    project_root: &Path,
    project: &str,
    inventory: &Inventory,
) -> Result<StackMigrationCoverage> {
    let workspaces: Vec<_> = read_config_workspaces(project_root)?
        .into_iter()
        .filter(|workspace| workspace.project_name.as_deref() == Some(project))
        .collect();
    if workspaces.is_empty() {
        bail!(
            "standardize project `{}` has no configured workspaces",
            project
        );
    }

    let td_root = read_config_workspace_scopes(project_root)?
        .into_iter()
        .find(|scope| scope.project_name.as_deref() == Some(project))
        .and_then(|scope| configured_td_root(project_root, &scope))
        .unwrap_or_else(|| crate::shared::workspace::tech_design_path(project_root));
    let persistence_annotations = count_persistence_annotations(&td_root)?;

    let mut reports = Vec::new();
    let mut blockers = Vec::new();
    let mut dependency_policy_blockers = Vec::new();
    let mut deployment_policy_blockers = Vec::new();
    for workspace in workspaces {
        let report = build_workspace_stack_migration(
            project_root,
            &inventory,
            &workspace,
            persistence_annotations,
        )?;
        if !report.normalized {
            blockers.push(format!(
                "{} stack migration classification incomplete: {}",
                report.name,
                report.notes.join("; ")
            ));
        }
        for finding in report
            .dependency_policies
            .iter()
            .filter(|finding| finding.classification == "stale_or_unmaintained")
        {
            dependency_policy_blockers.push(format!(
                "{} dependency `{}`: {}",
                report.name, finding.dependency, finding.reason
            ));
        }
        for kind in &report.unsupported_deployment_kinds {
            deployment_policy_blockers.push(format!(
                "{} deployment kind `{}` needs generator/TD classification",
                report.name, kind
            ));
        }
        reports.push(report);
    }
    blockers.extend(dependency_policy_blockers.iter().cloned());
    blockers.extend(deployment_policy_blockers.iter().cloned());

    let normalized_count = reports
        .iter()
        .filter(|workspace| workspace.normalized)
        .count();
    let incomplete_workspace_count = reports.len().saturating_sub(normalized_count);
    let migration_normalized_percent = if reports.is_empty() {
        100.0
    } else {
        (normalized_count as f64 / reports.len() as f64) * 100.0
    };

    Ok(StackMigrationCoverage {
        project: project.to_string(),
        workspaces: reports,
        migration_normalized_percent,
        incomplete_workspace_count,
        dependency_policy_blockers,
        deployment_policy_blockers,
        blockers,
    })
}

fn build_workspace_stack_migration(
    project_root: &Path,
    inventory: &Inventory,
    workspace: &ConfiguredWorkspace,
    persistence_annotations: usize,
) -> Result<WorkspaceStackMigration> {
    let matcher = build_scope_matcher(&workspace.paths)?;
    let files: Vec<_> = inventory
        .files
        .iter()
        .filter(|file| matcher.is_match(&file.rel))
        .collect();
    let workspace_root = workspace_root_from_paths(&workspace.paths)
        .unwrap_or_else(|| workspace.paths.first().cloned().unwrap_or_default());
    let workspace_name = workspace
        .name
        .clone()
        .unwrap_or_else(|| workspace_root.trim_end_matches('/').to_string());

    let mut manifest_stacks = BTreeSet::new();
    let mut dependency_policies = BTreeMap::new();
    for manifest in workspace_manifest_candidates(project_root, &workspace.paths) {
        if let Ok(content) = fs::read_to_string(&manifest) {
            detect_manifest_stacks(&content, &mut manifest_stacks);
            detect_dependency_policies(&content, &mut dependency_policies);
        }
    }

    let mut source_stacks = BTreeSet::new();
    let mut deployment_facets = BTreeSet::new();
    let mut unsupported_deployment_kinds = BTreeSet::new();
    let mut deployment_manifest_count = 0usize;
    let mut migration_job_count = 0usize;
    let mut gpu_manifest_count = 0usize;
    let mut mongo_write_paths = 0usize;
    let mut mongo_read_fallbacks = 0usize;
    let mut objectid_uuid_bridges = 0usize;
    let is_python_workspace = workspace.target.as_deref() == Some("python");
    for file in files {
        let Ok(content) = fs::read_to_string(&file.abs) else {
            continue;
        };
        if is_python_workspace && file.language == "python" {
            detect_source_stacks(&content, &mut source_stacks);
        } else if matches!(file.language.as_str(), "typescript" | "javascript") {
            detect_frontend_source_stacks(&content, &mut source_stacks);
        } else if file.language == "kustomize" {
            let finding_count = detect_deployment_facets(
                &file.rel,
                &content,
                &mut deployment_facets,
                &mut unsupported_deployment_kinds,
            );
            if finding_count > 0 {
                deployment_manifest_count += 1;
            }
        }
        let lower = content.to_ascii_lowercase();
        if file.language == "kustomize" {
            if lower.contains("alembic") || lower.contains("beanie") || lower.contains("migration")
            {
                migration_job_count += 1;
            }
            if lower.contains("nvidia.com/gpu")
                || lower.contains("cloud.google.com/gke-accelerator")
            {
                gpu_manifest_count += 1;
            }
        }
        if is_python_workspace && file.language == "python" {
            if lower.contains("mongodb is never written")
                || lower.contains("read-only fallback")
                || lower.contains("read only fallback")
            {
                mongo_read_fallbacks += 1;
            }
            if lower.contains("objectid") && lower.contains("uuid") {
                objectid_uuid_bridges += 1;
            }
            if (lower.contains(".insert")
                || lower.contains(".save(")
                || lower.contains(".replace(")
                || lower.contains(".update("))
                && (lower.contains("beanie")
                    || lower.contains("mongodb")
                    || lower.contains("pymongo"))
                && !lower.contains("read-only fallback")
            {
                mongo_write_paths += 1;
            }
        }
    }

    let manifest_stacks: Vec<_> = manifest_stacks.into_iter().collect();
    let source_stacks: Vec<_> = source_stacks.into_iter().collect();
    let has_legacy = contains_any(&manifest_stacks, &["beanie", "mongo"])
        || contains_any(&source_stacks, &["beanie", "mongo"]);
    let has_target = contains_any(&manifest_stacks, &["sqlalchemy", "alloydb"])
        || contains_any(&source_stacks, &["sqlalchemy", "alloydb", "alembic"]);
    let migration_state = match (has_legacy, has_target) {
        (true, true) => "beanie_mongo_to_sqlalchemy_alloydb_in_progress",
        (true, false) => "legacy_beanie_mongo",
        (false, true) => "sqlalchemy_alloydb",
        (false, false) => "no_persistence_stack_detected",
    }
    .to_string();

    let mut notes = Vec::new();
    if is_python_workspace && has_legacy && has_target && persistence_annotations == 0 {
        notes.push("mixed persistence stack has no TD persistence annotations".to_string());
    }
    if mongo_write_paths > 0 {
        notes.push(format!(
            "{mongo_write_paths} Mongo/Beanie write-path source file(s) detected"
        ));
    }
    if mongo_read_fallbacks > 0 {
        notes.push(format!(
            "{mongo_read_fallbacks} Mongo read-only fallback source file(s) detected"
        ));
    }
    if objectid_uuid_bridges > 0 {
        notes.push(format!(
            "{objectid_uuid_bridges} ObjectId-to-UUID bridge source file(s) detected"
        ));
    }
    if deployment_manifest_count > 0 {
        notes.push(format!(
            "{deployment_manifest_count} Kustomize deployment manifest file(s) classified"
        ));
    }
    if migration_job_count > 0 {
        notes.push(format!(
            "{migration_job_count} migration/scheduler Kustomize job manifest file(s) detected"
        ));
    }
    if gpu_manifest_count > 0 {
        notes.push(format!(
            "{gpu_manifest_count} GPU scheduling Kustomize manifest file(s) detected"
        ));
    }
    if !unsupported_deployment_kinds.is_empty() {
        notes.push(format!(
            "{} unsupported Kubernetes/GKE kind(s) require TD/generator classification",
            unsupported_deployment_kinds.len()
        ));
    }
    if notes.is_empty() {
        notes.push("no migration classification blocker detected".to_string());
    }

    let normalized = (!is_python_workspace
        || !(has_legacy && has_target && persistence_annotations == 0))
        && unsupported_deployment_kinds.is_empty();

    Ok(WorkspaceStackMigration {
        name: workspace_name,
        target: workspace.target.clone(),
        paths: workspace.paths.clone(),
        manifest_stacks,
        source_stacks,
        migration_state,
        persistence_annotations,
        dependency_policies: dependency_policies.into_values().collect(),
        deployment_manifest_count,
        deployment_facets: deployment_facets.into_iter().collect(),
        unsupported_deployment_kinds: unsupported_deployment_kinds.into_iter().collect(),
        normalized,
        notes,
    })
}

fn workspace_root_from_paths(paths: &[String]) -> Option<String> {
    paths
        .iter()
        .filter_map(|path| path.split_once("/**").map(|(prefix, _)| prefix.to_string()))
        .min_by_key(|prefix| prefix.len())
}

fn workspace_manifest_candidates(project_root: &Path, paths: &[String]) -> Vec<PathBuf> {
    let mut out = BTreeSet::new();
    for path in paths {
        let root = path
            .split_once("/**")
            .map(|(prefix, _)| prefix)
            .unwrap_or(path.as_str())
            .trim_end_matches('/');
        out.insert(project_root.join(root).join("pyproject.toml"));
        out.insert(project_root.join(root).join("package.json"));
    }
    out.into_iter().collect()
}

fn detect_manifest_stacks(content: &str, out: &mut BTreeSet<String>) {
    let lower = content.to_ascii_lowercase();
    for (needle, stack) in [
        ("fastapi", "fastapi"),
        ("beanie", "beanie"),
        ("mongodb", "mongo"),
        ("pymongo", "mongo"),
        ("motor", "mongo"),
        ("sqlalchemy", "sqlalchemy"),
        ("asyncpg", "alloydb"),
        ("alembic", "alembic"),
        ("firebase-admin", "gcp:firebase"),
        ("google-cloud-aiplatform", "gcp:vertex-ai"),
        ("vertexai", "gcp:vertex-ai"),
        ("google-cloud-error-reporting", "gcp:error-reporting"),
        ("google-cloud-logging", "gcp:logging"),
        ("google-cloud-pubsub", "gcp:pubsub"),
        ("google-cloud-scheduler", "gcp:scheduler"),
        ("google-cloud-storage", "gcp:storage"),
        ("google-cloud-tasks", "gcp:cloud-tasks"),
        ("google-cloud-workflows", "gcp:workflows"),
        ("google-cloud-video-transcoder", "gcp:transcoder"),
        ("opentelemetry", "observability:opentelemetry"),
        ("react", "react"),
        ("vite", "vite"),
        ("tensorflow", "tensorflow"),
        ("torch", "pytorch"),
        ("transformers", "transformers"),
    ] {
        if lower.contains(needle) {
            out.insert(stack.to_string());
        }
    }
}

fn detect_source_stacks(content: &str, out: &mut BTreeSet<String>) {
    let lower = content.to_ascii_lowercase();
    for (needle, stack) in [
        ("fastapi", "fastapi"),
        ("beanie", "beanie"),
        ("mongodb", "mongo"),
        ("pymongo", "mongo"),
        ("asyncmongoclient", "mongo"),
        ("pydanticobjectid", "beanie"),
        ("sqlalchemy", "sqlalchemy"),
        ("mapped_column", "sqlalchemy"),
        ("alloydb", "alloydb"),
        ("alembic", "alembic"),
        ("objectid", "objectid"),
        ("uuid", "uuid"),
        ("google.cloud.aiplatform", "gcp:vertex-ai"),
        ("vertexai", "gcp:vertex-ai"),
        ("google.cloud.pubsub", "gcp:pubsub"),
        ("google.cloud.scheduler", "gcp:scheduler"),
        ("google.cloud.storage", "gcp:storage"),
        ("google.cloud.tasks", "gcp:cloud-tasks"),
        ("google.cloud.workflows", "gcp:workflows"),
        ("google.cloud.video", "gcp:transcoder"),
        ("firebase_admin", "gcp:firebase"),
        ("opentelemetry", "observability:opentelemetry"),
    ] {
        if lower.contains(needle) {
            out.insert(stack.to_string());
        }
    }
}

fn detect_frontend_source_stacks(content: &str, out: &mut BTreeSet<String>) {
    let lower = content.to_ascii_lowercase();
    for (needle, stack) in [("react", "react"), ("vue", "vue"), ("svelte", "svelte")] {
        if lower.contains(needle) {
            out.insert(stack.to_string());
        }
    }
}

fn detect_deployment_facets(
    path: &str,
    content: &str,
    out: &mut BTreeSet<DeploymentFacetFinding>,
    unsupported_kinds: &mut BTreeSet<String>,
) -> usize {
    let Ok(document) = serde_yaml::from_str::<serde_yaml::Value>(content) else {
        return 0;
    };
    let Some(kind) = yaml_string_field(&document, "kind") else {
        return 0;
    };
    let api_version = yaml_string_field(&document, "apiVersion").unwrap_or_default();
    let lower = content.to_ascii_lowercase();
    if let Some(rule) = deployment_facet_rule(&kind, &lower) {
        out.insert(DeploymentFacetFinding {
            path: path.to_string(),
            kind: kind.clone(),
            api_version: api_version.clone(),
            facet: rule.facet.to_string(),
            classification: rule.classification.to_string(),
            action: rule.action.to_string(),
            reason: rule.reason.to_string(),
        });
    } else {
        unsupported_kinds.insert(kind.clone());
        out.insert(DeploymentFacetFinding {
            path: path.to_string(),
            kind: kind.clone(),
            api_version: api_version.clone(),
            facet: "unsupported_kind".to_string(),
            classification: "generator_gap".to_string(),
            action: "add_td_section_annotation".to_string(),
            reason: "Kubernetes/GKE kind is not mapped to a deployment generator facet".to_string(),
        });
    }
    if lower.contains("nvidia.com/gpu") || lower.contains("cloud.google.com/gke-accelerator") {
        out.insert(DeploymentFacetFinding {
            path: path.to_string(),
            kind,
            api_version,
            facet: "gpu_scheduling".to_string(),
            classification: "ml_runtime_constraint".to_string(),
            action: "model_accelerator_requirements".to_string(),
            reason: "GPU resources, node selectors, and tolerations must be TD-owned for ML replay"
                .to_string(),
        });
    }
    1
}

fn yaml_string_field(document: &serde_yaml::Value, key: &str) -> Option<String> {
    let mapping = document.as_mapping()?;
    mapping
        .get(serde_yaml::Value::String(key.to_string()))?
        .as_str()
        .map(str::to_string)
}

struct DeploymentFacetRule {
    facet: &'static str,
    classification: &'static str,
    action: &'static str,
    reason: &'static str,
}

fn deployment_facet_rule(kind: &str, content_lower: &str) -> Option<DeploymentFacetRule> {
    let rule = match kind {
        "Kustomization" => DeploymentFacetRule {
            facet: "kustomize_composition",
            classification: "composition",
            action: "generate_kustomization",
            reason: "Kustomize resource composition and overlay wiring",
        },
        "Component" => DeploymentFacetRule {
            facet: "kustomize_component",
            classification: "composition",
            action: "generate_kustomize_component",
            reason: "Kustomize component resource composition and optional wiring",
        },
        "PrefixTransformer" => DeploymentFacetRule {
            facet: "kustomize_name_transformer",
            classification: "composition",
            action: "generate_name_transformer",
            reason: "Kustomize name prefix transformation for composed resources",
        },
        "Deployment" => DeploymentFacetRule {
            facet: "deployment_unit",
            classification: "core_kubernetes",
            action: "generate_deployment",
            reason: "runtime image, command, env, probes, resources, and replicas",
        },
        "Service" => DeploymentFacetRule {
            facet: "service_exposure",
            classification: "core_kubernetes",
            action: "generate_service",
            reason: "stable service name and port exposure",
        },
        "HorizontalPodAutoscaler" => DeploymentFacetRule {
            facet: "autoscaling_policy",
            classification: "core_kubernetes",
            action: "generate_hpa",
            reason: "replica and metric policy must follow runtime capacity model",
        },
        "CronJob" if content_lower.contains("migration") || content_lower.contains("alembic") => {
            DeploymentFacetRule {
                facet: "migration_job",
                classification: "migration_sensitive",
                action: "bind_to_persistence_migration_td",
                reason: "database migration execution is coupled to Beanie/Mongo to SQLAlchemy/AlloyDB rollout",
            }
        }
        "CronJob" => DeploymentFacetRule {
            facet: "scheduled_job",
            classification: "core_kubernetes",
            action: "generate_cronjob",
            reason: "scheduled operational task",
        },
        "ServiceAccount" => DeploymentFacetRule {
            facet: "identity_binding",
            classification: "security_boundary",
            action: "generate_service_account",
            reason: "workload identity and permission boundary",
        },
        "ConfigMap" => DeploymentFacetRule {
            facet: "runtime_config",
            classification: "configuration",
            action: "generate_configmap",
            reason: "environment and runtime configuration",
        },
        "Secret" => DeploymentFacetRule {
            facet: "secret_material_boundary",
            classification: "security_boundary",
            action: "externalize_secret_reference",
            reason: "secret material must be referenced, not embedded as generated source",
        },
        "HTTPRoute" => DeploymentFacetRule {
            facet: "gateway_route",
            classification: "gke_gateway_adapter",
            action: "generate_gateway_route",
            reason: "GKE Gateway API route binding",
        },
        "GCPBackendPolicy" => DeploymentFacetRule {
            facet: "gcp_backend_policy",
            classification: "gke_policy_adapter",
            action: "generate_gcp_backend_policy",
            reason: "GKE backend service policy and timeout semantics",
        },
        "HealthCheckPolicy" => DeploymentFacetRule {
            facet: "health_check_policy",
            classification: "gke_policy_adapter",
            action: "generate_health_check_policy",
            reason: "GKE health-check behavior",
        },
        "PodMonitoring" | "ServiceMonitor" => DeploymentFacetRule {
            facet: "observability_scrape",
            classification: "observability",
            action: "generate_monitoring_resource",
            reason: "metrics scrape and monitoring integration",
        },
        "StatefulSet" => DeploymentFacetRule {
            facet: "stateful_dependency",
            classification: "stateful_runtime",
            action: "generate_stateful_workload",
            reason: "stateful platform service deployment",
        },
        "PersistentVolumeClaim" => DeploymentFacetRule {
            facet: "stateful_storage",
            classification: "stateful_runtime",
            action: "generate_storage_claim",
            reason: "persistent storage binding",
        },
        _ => return None,
    };
    Some(rule)
}

fn detect_dependency_policies(content: &str, out: &mut BTreeMap<String, DependencyPolicyFinding>) {
    let lower = content.to_ascii_lowercase();
    for rule in dependency_policy_rules() {
        if lower.contains(rule.needle) {
            out.entry(rule.dependency.to_string())
                .or_insert_with(|| DependencyPolicyFinding {
                    dependency: rule.dependency.to_string(),
                    classification: rule.classification.to_string(),
                    action: rule.action.to_string(),
                    reason: rule.reason.to_string(),
                });
        }
    }
}

struct DependencyPolicyRule {
    needle: &'static str,
    dependency: &'static str,
    classification: &'static str,
    action: &'static str,
    reason: &'static str,
}

fn dependency_policy_rules() -> &'static [DependencyPolicyRule] {
    &[
        DependencyPolicyRule {
            needle: "fastapi",
            dependency: "fastapi",
            classification: "core_external",
            action: "keep",
            reason: "serving framework; short-term third-party default",
        },
        DependencyPolicyRule {
            needle: "pydantic",
            dependency: "pydantic",
            classification: "core_external",
            action: "keep",
            reason: "typed validation/model runtime; generator can target it",
        },
        DependencyPolicyRule {
            needle: "sqlalchemy",
            dependency: "sqlalchemy",
            classification: "core_external",
            action: "keep",
            reason: "AlloyDB/PostgreSQL ORM target during DB migration",
        },
        DependencyPolicyRule {
            needle: "alembic",
            dependency: "alembic",
            classification: "core_external",
            action: "keep",
            reason: "SQLAlchemy migration tool",
        },
        DependencyPolicyRule {
            needle: "beanie",
            dependency: "beanie",
            classification: "migration_sensitive",
            action: "classify_projection",
            reason: "Mongo document model should be annotated as legacy/current projection",
        },
        DependencyPolicyRule {
            needle: "bunnet",
            dependency: "bunnet",
            classification: "migration_sensitive",
            action: "classify_projection",
            reason: "Mongo document model should be annotated as legacy/current projection",
        },
        DependencyPolicyRule {
            needle: "pymongo",
            dependency: "pymongo",
            classification: "migration_sensitive",
            action: "classify_projection",
            reason: "direct Mongo access needs migration role and wrapper boundary",
        },
        DependencyPolicyRule {
            needle: "google-cloud-",
            dependency: "google-cloud-*",
            classification: "adapter_required",
            action: "wrap",
            reason: "GCP SDKs should be accessed through generated typed adapters",
        },
        DependencyPolicyRule {
            needle: "google-api-python-client",
            dependency: "google-api-python-client",
            classification: "adapter_required",
            action: "wrap",
            reason: "Google API clients should be isolated behind generated adapters",
        },
        DependencyPolicyRule {
            needle: "firebase-admin",
            dependency: "firebase-admin",
            classification: "adapter_required",
            action: "wrap",
            reason: "Firebase access should be isolated behind generated adapters",
        },
        DependencyPolicyRule {
            needle: "google-genai",
            dependency: "google-genai",
            classification: "adapter_required",
            action: "wrap",
            reason: "LLM provider SDK should be behind generated prompt/client contracts",
        },
        DependencyPolicyRule {
            needle: "openai",
            dependency: "openai",
            classification: "adapter_required",
            action: "wrap",
            reason: "LLM provider SDK should be behind generated prompt/client contracts",
        },
        DependencyPolicyRule {
            needle: "example-domain-client",
            dependency: "example-domain-client",
            classification: "adapter_required",
            action: "wrap",
            reason: "domain SDK should not leak into business logic",
        },
        DependencyPolicyRule {
            needle: "example-dataservice-sdk",
            dependency: "example-dataservice-sdk",
            classification: "adapter_required",
            action: "wrap",
            reason: "domain SDK should not leak into business logic",
        },
        DependencyPolicyRule {
            needle: "langchain",
            dependency: "langchain*",
            classification: "replace_after_mamba",
            action: "wrap_now_replace_later",
            reason: "orchestration/prompt glue is a cclab generator candidate",
        },
        DependencyPolicyRule {
            needle: "pydantic-settings",
            dependency: "pydantic-settings",
            classification: "replace_after_mamba",
            action: "wrap_now_replace_later",
            reason: "configuration glue is a cclab runtime/generator candidate",
        },
        DependencyPolicyRule {
            needle: "tenacity",
            dependency: "tenacity",
            classification: "replace_after_mamba",
            action: "wrap_now_replace_later",
            reason: "retry policy should become generated platform behavior",
        },
        DependencyPolicyRule {
            needle: "torch",
            dependency: "torch",
            classification: "core_external",
            action: "keep",
            reason: "ML runtime is not a short-term self-host target",
        },
        DependencyPolicyRule {
            needle: "transformers",
            dependency: "transformers",
            classification: "core_external",
            action: "keep",
            reason: "model runtime is not a short-term self-host target",
        },
        DependencyPolicyRule {
            needle: "sentence-transformers",
            dependency: "sentence-transformers",
            classification: "core_external",
            action: "keep",
            reason: "embedding/model runtime is not a short-term self-host target",
        },
        DependencyPolicyRule {
            needle: "scikit-learn",
            dependency: "scikit-learn",
            classification: "core_external",
            action: "keep",
            reason: "ML runtime is not a short-term self-host target",
        },
        DependencyPolicyRule {
            needle: "pandas",
            dependency: "pandas",
            classification: "core_external",
            action: "keep",
            reason: "data processing runtime is not a short-term self-host target",
        },
        DependencyPolicyRule {
            needle: "numpy",
            dependency: "numpy",
            classification: "core_external",
            action: "keep",
            reason: "numeric runtime is not a short-term self-host target",
        },
        DependencyPolicyRule {
            needle: "react",
            dependency: "react",
            classification: "core_external",
            action: "keep",
            reason: "frontend runtime; generator can target it short-term",
        },
        DependencyPolicyRule {
            needle: "next",
            dependency: "next",
            classification: "core_external",
            action: "keep",
            reason: "frontend app framework; generator can target it short-term",
        },
        DependencyPolicyRule {
            needle: "antd",
            dependency: "antd",
            classification: "adapter_required",
            action: "wrap",
            reason: "UI kit usage should be mediated by shared-ui components",
        },
        DependencyPolicyRule {
            needle: "lexical",
            dependency: "lexical",
            classification: "adapter_required",
            action: "wrap",
            reason: "rich text editor should be mediated by shared-ui components",
        },
        DependencyPolicyRule {
            needle: "monaco-editor",
            dependency: "monaco-editor",
            classification: "adapter_required",
            action: "wrap",
            reason: "editor integration should be mediated by generated/shared UI",
        },
        DependencyPolicyRule {
            needle: "chart.js",
            dependency: "chart.js",
            classification: "adapter_required",
            action: "wrap",
            reason: "charting should be mediated by generated/shared chart components",
        },
        DependencyPolicyRule {
            needle: "@ant-design/plots",
            dependency: "@ant-design/plots",
            classification: "adapter_required",
            action: "wrap",
            reason: "charting should be mediated by generated/shared chart components",
        },
        DependencyPolicyRule {
            needle: "axios",
            dependency: "axios",
            classification: "adapter_required",
            action: "wrap",
            reason: "HTTP access should be mediated by generated client contracts",
        },
        DependencyPolicyRule {
            needle: "material-design-lite",
            dependency: "material-design-lite",
            classification: "stale_or_unmaintained",
            action: "avoid_or_replace",
            reason: "legacy UI toolkit should not be a target dependency",
        },
        DependencyPolicyRule {
            needle: "braft-editor",
            dependency: "braft-editor",
            classification: "stale_or_unmaintained",
            action: "avoid_or_replace",
            reason: "legacy editor should not be a target dependency when Lexical exists",
        },
        DependencyPolicyRule {
            needle: "draft-js",
            dependency: "draft-js",
            classification: "stale_or_unmaintained",
            action: "avoid_or_replace",
            reason: "legacy editor should not be a target dependency when Lexical exists",
        },
        DependencyPolicyRule {
            needle: "next-images",
            dependency: "next-images",
            classification: "stale_or_unmaintained",
            action: "avoid_or_replace",
            reason: "legacy Next image plugin should not be a target dependency",
        },
        DependencyPolicyRule {
            needle: "react-qr-reader",
            dependency: "react-qr-reader",
            classification: "stale_or_unmaintained",
            action: "avoid_or_replace",
            reason: "beta QR reader dependency should be reviewed before generator adoption",
        },
        DependencyPolicyRule {
            needle: "xlsx",
            dependency: "xlsx",
            classification: "stale_or_unmaintained",
            action: "avoid_or_replace",
            reason: "spreadsheet parser has known ecosystem maintenance/security concerns",
        },
        DependencyPolicyRule {
            needle: "redis-om",
            dependency: "redis-om",
            classification: "incidental_review",
            action: "review_usage",
            reason: "Redis object mapper should be justified before platform standardization",
        },
        DependencyPolicyRule {
            needle: "jsonata-python",
            dependency: "jsonata-python",
            classification: "incidental_review",
            action: "review_usage",
            reason: "query/transformation DSL dependency should be justified",
        },
        DependencyPolicyRule {
            needle: "python-jose",
            dependency: "python-jose",
            classification: "incidental_review",
            action: "review_usage",
            reason: "security dependency should be reviewed against current alternatives",
        },
        DependencyPolicyRule {
            needle: "easyocr",
            dependency: "easyocr",
            classification: "incidental_review",
            action: "review_usage",
            reason: "OCR runtime should be tied to a specific product capability",
        },
    ]
}

fn contains_any(values: &[String], needles: &[&str]) -> bool {
    values
        .iter()
        .any(|value| needles.iter().any(|needle| value == needle))
}

fn count_persistence_annotations(td_root: &Path) -> Result<usize> {
    if !td_root.is_dir() {
        return Ok(0);
    }
    let mut count = 0usize;
    let mut stack = vec![td_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("failed to read TD directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            count += content.matches("persistence_migration:").count();
            count += content.matches("migration_role:").count();
            count += content.matches("projection:").count();
        }
    }
    Ok(count)
}

fn collect_source_files(project_root: &Path, scopes: &[String]) -> Result<Vec<SourceFile>> {
    let matcher = build_scope_matcher(scopes)?;
    let mut rels = BTreeSet::new();
    let mut out = Vec::new();

    for scope in scopes {
        let walk_root = scope_walk_root(project_root, scope);
        if !walk_root.exists() || is_excluded_path(&walk_root) {
            continue;
        }
        for entry in ignore::WalkBuilder::new(&walk_root)
            .follow_links(false)
            .hidden(false)
            .filter_entry(|e| !is_excluded_path(e.path()))
            .build()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }
            let path = entry.path();
            if is_minified_asset_source(path) || !is_supported_source(path) {
                continue;
            }
            let rel = path
                .strip_prefix(project_root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            if !matcher.is_match(&rel) || !rels.insert(rel.clone()) {
                continue;
            }
            let content = fs::read_to_string(path).unwrap_or_default();
            let markers = detect_markers(&content);
            let handwrite_gaps = detect_handwrite_gaps(&content);
            out.push(SourceFile {
                rel,
                abs: path.to_path_buf(),
                language: language_for_path(path).unwrap_or("unknown").to_string(),
                markers,
                handwrite_gaps,
            });
        }
    }

    Ok(out)
}

fn extend_project_root_artifact_files(
    project_root: &Path,
    project: &str,
    files: &mut Vec<SourceFile>,
) -> Result<()> {
    let Some(project_rel) = configured_project_path(project_root, project)? else {
        return Ok(());
    };
    let mut rels = files
        .iter()
        .map(|file| file.rel.clone())
        .collect::<BTreeSet<_>>();
    for name in required_project_root_artifact_names(project_root, project, &project_rel)? {
        let rel = format!("{}/{}", project_rel.trim_end_matches('/'), name);
        if !rels.insert(rel.clone()) {
            continue;
        }
        let abs = project_root.join(&rel);
        if !abs.is_file() {
            continue;
        }
        let content = fs::read_to_string(&abs).unwrap_or_default();
        files.push(SourceFile {
            rel,
            abs,
            language: project_root_artifact_language(name).to_string(),
            markers: detect_markers(&content),
            handwrite_gaps: detect_handwrite_gaps(&content),
        });
    }
    Ok(())
}

fn missing_project_root_artifacts(project_root: &Path, project: &str) -> Result<Vec<String>> {
    let Some(project_rel) = configured_project_path(project_root, project)? else {
        return Ok(Vec::new());
    };
    Ok(
        required_project_root_artifact_names(project_root, project, &project_rel)?
            .into_iter()
            .map(|name| format!("{}/{}", project_rel.trim_end_matches('/'), name))
            .filter(|rel| !project_root.join(rel).is_file())
            .collect(),
    )
}

/// Return production blockers for the project-root artifacts that AW itself
/// expects agents and build skills to consume.
/// @spec apps/agentic-workflow/tech-design/logic/manage-project-root-llms-and-build-install-artifacts.md#logic
pub(crate) fn project_root_artifact_blockers(project: &str) -> Result<Vec<String>> {
    let project_root = crate::find_project_root()?;
    project_root_artifact_blockers_at(&project_root, project)
}

fn project_root_artifact_blockers_at(project_root: &Path, project: &str) -> Result<Vec<String>> {
    let project = resolve_standardize_project_name(project_root, project)?;
    let Some(project_rel) = configured_project_path(project_root, &project)? else {
        return Ok(Vec::new());
    };
    let mut blockers = Vec::new();
    for name in required_project_root_artifact_names(project_root, &project, &project_rel)? {
        let rel = format!("{}/{}", project_rel.trim_end_matches('/'), name);
        let abs = project_root.join(&rel);
        if !abs.is_file() {
            blockers.push(format!("missing project root artifact `{rel}`"));
            continue;
        }
        if name == "llms.txt" {
            let content = fs::read_to_string(&abs).unwrap_or_default();
            let expected = render_project_llms_txt(project_root, &project)?;
            let markers = detect_markers(&content);
            if !markers.codegen {
                blockers.push(format!(
                    "project root artifact `{rel}` must be generated by AW from TD-first project context"
                ));
            } else if content != expected {
                blockers.push(format!(
                    "project root artifact `{rel}` is stale; run `aw standardize managed run --project {} --non-interactive --max-ticks 1`",
                    shell_quote(&project)
                ));
            }
        }
        if matches!(name, "build.sh" | "install.sh") && !is_executable_file(&abs) {
            blockers.push(format!("project root artifact `{rel}` is not executable"));
        }
        if name == "build.sh" {
            let content = fs::read_to_string(&abs).unwrap_or_default();
            if !content.contains("debug") || !content.contains("release") {
                blockers.push(format!(
                    "project root artifact `{rel}` must expose debug and release build modes"
                ));
            }
            if project_has_rust_binary(project_root, &project, &project_rel)?
                && !content.contains("--release")
                && !content.contains("target/release")
            {
                blockers.push(format!(
                    "project root artifact `{rel}` release mode must build or install the release profile"
                ));
            }
        }
    }
    Ok(blockers)
}

fn required_project_root_artifact_names(
    project_root: &Path,
    project: &str,
    project_rel: &str,
) -> Result<Vec<&'static str>> {
    let mut names = PROJECT_CONTEXT_ARTIFACTS.to_vec();
    if project_has_rust_binary(project_root, project, project_rel)? {
        names.extend(RUST_BINARY_ARTIFACTS.iter().copied());
    }
    names.sort();
    names.dedup();
    Ok(names)
}

fn configured_project_path(project_root: &Path, project: &str) -> Result<Option<String>> {
    let configured = read_config_workspace_scopes(project_root)?;
    Ok(configured
        .iter()
        .find(|scope| configured_scope_matches_project(scope, project))
        .and_then(|scope| scope.project_path.clone()))
}

fn configured_project_scope(project_root: &Path, project: &str) -> Result<Option<ConfiguredScope>> {
    let configured = read_config_workspace_scopes(project_root)?;
    Ok(configured
        .into_iter()
        .find(|scope| configured_scope_matches_project(scope, project)))
}

pub(crate) fn configured_project_name_for_path(
    project_root: &Path,
    target: &str,
) -> Result<Option<String>> {
    let configured = read_config_workspace_scopes(project_root)?;
    let target = target.replace('\\', "/");
    Ok(configured
        .into_iter()
        .filter_map(|scope| {
            let project_name = scope.project_name?;
            let project_path = scope.project_path?;
            if path_prefix_of(&project_path, &target) {
                Some((project_name, project_path.len()))
            } else {
                None
            }
        })
        .max_by_key(|(_, len)| *len)
        .map(|(project_name, _)| project_name))
}

pub(crate) fn render_project_llms_txt(project_root: &Path, project: &str) -> Result<String> {
    let project = resolve_standardize_project_name(project_root, project)?;
    let scope = configured_project_scope(project_root, &project)?
        .with_context(|| format!("project `{project}` is not configured"))?;
    let project_rel = scope
        .project_path
        .clone()
        .with_context(|| format!("project `{project}` has no configured path"))?;
    let td_path = configured_td_path(&scope).unwrap_or_else(|| {
        crate::services::project_registry::default_project_td_path(&project_rel)
            .to_string_lossy()
            .into_owned()
    });
    let cap_path = scope
        .cap_path
        .clone()
        .unwrap_or_else(|| format!("{}/README.md", project_rel.trim_end_matches('/')));
    let spec_ref = project_llms_semantic_spec_ref(project_root, &project, &project_rel, &td_path);
    let title = project_agent_context_title(&project);
    let required_artifacts =
        required_project_root_artifact_names(project_root, &project, &project_rel)?;
    let has_build = required_artifacts.contains(&"build.sh");
    let has_install = required_artifacts.contains(&"install.sh");
    let mut test_cmds = read_config_workspaces(project_root)?
        .into_iter()
        .filter(|workspace| workspace.project_name.as_deref() == Some(project.as_str()))
        .filter_map(|workspace| workspace.test_cmd)
        .filter(|cmd| cmd != "true")
        .collect::<Vec<_>>();
    test_cmds.sort();
    test_cmds.dedup();

    let mut out = String::new();
    out.push_str(&format!("<!-- SPEC-MANAGED: {spec_ref}#schema -->\n"));
    out.push_str("<!-- CODEGEN-BEGIN -->\n");
    out.push_str(&format!("# {title} Agent Context\n\n"));
    out.push_str(
        "> TD-first map for agents. Start from tech design and capability intent before implementation files.\n\n",
    );
    out.push_str("## Tech Design\n\n");
    out.push_str(&format!(
        "- [Tech Design]({}): implementation source of truth.\n",
        project_relative_link(&project_rel, &td_path)
    ));
    out.push_str(&format!("- Validate: `aw td check {td_path}`.\n\n"));
    out.push_str("## Capability Map\n\n");
    out.push_str(&format!(
        "- [README]({}): capability source of truth and product contract.\n\n",
        project_relative_link(&project_rel, &cap_path)
    ));
    out.push_str("## Agent Workflow\n\n");
    out.push_str(&format!(
        "- Capability next action: `aw capability next --project {}`.\n",
        shell_quote(&project)
    ));
    out.push_str(&format!(
        "- Readiness: `aw health --project {}`.\n",
        shell_quote(&project)
    ));
    out.push_str(&format!(
        "- Blockers: `aw health --project {} blockers`.\n\n",
        shell_quote(&project)
    ));
    out.push_str("## Commands\n\n");
    if has_build {
        out.push_str("- Build debug: `./build.sh debug`.\n");
        out.push_str("- Build release: `./build.sh release`.\n");
    }
    if has_install {
        out.push_str(&format!(
            "- Install: repo root `install.sh --project={}` dispatches to `./install.sh`.\n",
            project
        ));
    }
    for cmd in &test_cmds {
        out.push_str(&format!("- Test: `{cmd}`.\n"));
    }
    out.push('\n');
    out.push_str("## Root Artifacts\n\n");
    out.push_str("- [llms.txt](llms.txt): generated by AW from TD-first project context.\n");
    if has_build {
        out.push_str("- [build.sh](build.sh): debug/release build entrypoint.\n");
    }
    if has_install {
        out.push_str("- [install.sh](install.sh): project-local install entrypoint.\n");
    }
    out.push_str("<!-- CODEGEN-END -->\n");
    Ok(out)
}

fn project_llms_semantic_spec_ref(
    project_root: &Path,
    project: &str,
    project_rel: &str,
    td_path: &str,
) -> String {
    let configured = format!(
        "{}/semantic/{}-{}.md",
        td_path.trim_end_matches('/'),
        slug_for_path(project),
        slug_for_path(project_rel)
    );
    if project_root.join(&configured).is_file() {
        return configured;
    }
    let default_td_path = crate::services::project_registry::default_project_td_path(project_rel)
        .to_string_lossy()
        .into_owned();
    let fallback = format!(
        "{}/semantic/{}-{}.md",
        default_td_path.trim_end_matches('/'),
        slug_for_path(project),
        slug_for_path(project_rel)
    );
    if fallback != configured && project_root.join(&fallback).is_file() {
        return fallback;
    }
    configured
}

fn project_agent_context_title(project: &str) -> String {
    if project.contains('-') {
        project
            .split('-')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        project.to_string()
    }
}

fn project_relative_link(project_rel: &str, target_rel: &str) -> String {
    let project_rel = project_rel.trim_matches('/');
    let target_rel = target_rel.trim_start_matches("./").trim_matches('/');
    if target_rel.is_empty() || target_rel == project_rel {
        return ".".to_string();
    }
    let prefix = format!("{project_rel}/");
    if let Some(stripped) = target_rel.strip_prefix(&prefix) {
        return stripped.to_string();
    }
    let upward = project_rel
        .split('/')
        .filter(|part| !part.is_empty())
        .map(|_| "..")
        .collect::<Vec<_>>()
        .join("/");
    if upward.is_empty() {
        target_rel.to_string()
    } else {
        format!("{upward}/{target_rel}")
    }
}

fn project_has_rust_binary(project_root: &Path, project: &str, project_rel: &str) -> Result<bool> {
    let has_rust_workspace = read_config_workspaces(project_root)?
        .into_iter()
        .any(|workspace| {
            workspace.project_name.as_deref() == Some(project)
                && workspace.target.as_deref() == Some("rust")
        });
    if !has_rust_workspace {
        return Ok(false);
    }
    let root = project_root.join(project_rel);
    if !root.is_dir() {
        return Ok(false);
    }
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        if is_excluded_path(&dir) {
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml") {
                let manifest = fs::read_to_string(&path).unwrap_or_default();
                if manifest.contains("[[bin]]")
                    || path
                        .parent()
                        .is_some_and(|p| p.join("src/main.rs").is_file())
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn project_root_artifact_language(name: &str) -> &'static str {
    match name {
        "build.sh" | "install.sh" => "shell",
        "llms.txt" => "llms",
        _ => "root-artifact",
    }
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn build_scope_matcher(scopes: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for scope in scopes {
        if scope == "**" || scope == "." {
            builder.add(Glob::new("**")?);
            continue;
        }
        builder.add(Glob::new(scope)?);
        if !scope_has_glob(scope) {
            builder.add(Glob::new(&format!("{}/**", scope.trim_end_matches('/')))?);
        }
    }
    builder
        .build()
        .context("failed to build standardize scope matcher")
}

fn is_excluded_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    EXCLUDED_DIRS.contains(&name)
}

fn scope_has_glob(scope: &str) -> bool {
    scope.contains(['*', '?', '['])
}

fn scope_walk_root(project_root: &Path, scope: &str) -> PathBuf {
    let prefix = scope
        .split(['*', '?', '['])
        .next()
        .unwrap_or(scope)
        .trim_end_matches('/');
    let root = if prefix.is_empty() {
        PathBuf::from(".")
    } else if prefix.ends_with('/') {
        PathBuf::from(prefix.trim_end_matches('/'))
    } else if scope.contains('*') {
        PathBuf::from(prefix)
    } else {
        PathBuf::from(prefix)
    };
    project_root.join(root)
}

fn is_supported_source(path: &Path) -> bool {
    language_for_path(path).is_some()
}

fn is_minified_asset_source(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    file_name.ends_with(".min.js")
        && path
            .components()
            .any(|component| component.as_os_str() == "assets")
}

fn language_for_path(path: &Path) -> Option<&'static str> {
    if is_dockerfile_path(path) {
        return Some("dockerfile");
    }
    if is_dockerignore_path(path) {
        return Some("dockerignore");
    }
    if is_kustomize_yaml_path(path) {
        return Some("kustomize");
    }
    match path.extension().and_then(|e| e.to_str())? {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "ts" | "tsx" => Some("typescript"),
        "go" => Some("go"),
        "json" if is_frontend_manifest_json_path(path) || is_frontend_config_path(path) => {
            Some("json")
        }
        "css" | "scss" => Some("stylesheet"),
        _ => None,
    }
}

fn is_dockerfile_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    file_name == "Dockerfile"
        || file_name.starts_with("Dockerfile.")
        || file_name.ends_with(".dockerfile")
}

fn is_dockerignore_path(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some(".dockerignore")
}

fn is_kustomize_yaml_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let is_yaml = matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yaml" | "yml")
    );
    if !is_yaml {
        return false;
    }
    file_name == "kustomization.yaml"
        || file_name == "kustomization.yml"
        || path_has_component(path, "kustomize")
}

fn path_has_component(path: &Path, needle: &str) -> bool {
    path.components()
        .any(|component| component.as_os_str() == needle)
}

fn detect_markers(content: &str) -> FileMarkers {
    let mut markers = FileMarkers::default();
    let raw_string_lines = crate::generate::marker::rust_raw_string_line_mask(content);
    for (idx, line) in content.lines().enumerate() {
        if raw_string_lines.get(idx).copied().unwrap_or(false) {
            continue;
        }
        let Some(body) = marker_comment_body(line).or_else(|| structured_marker_body(line)) else {
            continue;
        };
        if body.starts_with("CODEGEN-BEGIN") {
            markers.codegen = true;
        }
        if body.starts_with("HANDWRITE-BEGIN") || body.starts_with("<HANDWRITE") {
            markers.handwrite = true;
        }
    }
    markers
}

fn detect_handwrite_gaps(content: &str) -> Vec<HandwriteGap> {
    let mut gaps = Vec::new();
    let raw_string_lines = crate::generate::marker::rust_raw_string_line_mask(content);
    for (idx, line) in content.lines().enumerate() {
        if raw_string_lines.get(idx).copied().unwrap_or(false) {
            continue;
        }
        let Some(body) = handwrite_marker_body(line).or_else(|| {
            structured_marker_body(line).filter(|body| is_handwrite_open_marker_line_body(body))
        }) else {
            continue;
        };
        let reason = extract_attr(body, "reason")
            .or_else(|| {
                body.split_once("reason:")
                    .map(|(_, rest)| rest.trim().to_string())
            })
            .unwrap_or_default();
        let tracker = extract_attr(body, "tracker").unwrap_or_default();
        let gap = extract_attr(body, "gap").unwrap_or_default();
        let needs_promotion = body.starts_with("HANDWRITE-BEGIN") && gap.trim().is_empty();
        let mut problems = Vec::new();
        if reason.trim().is_empty() {
            problems.push("missing reason");
        }
        if is_missing_tracker(&tracker) {
            problems.push("missing tracker");
        }
        if gap.trim().is_empty() && (body.starts_with("<HANDWRITE") || needs_promotion) {
            problems.push("missing gap");
        }
        if !problems.is_empty() {
            gaps.push(HandwriteGap {
                line_no: idx + 1,
                tracker,
                message: problems.join(", "),
                needs_promotion,
            });
        }
    }
    gaps
}

fn is_missing_tracker(tracker: &str) -> bool {
    matches!(
        tracker.trim(),
        "" | "pending-tracker" | "none" | "todo" | "tbd"
    )
}

fn strip_comment_lead(line: &str) -> &str {
    let s = line.trim_start();
    for prefix in ["///", "//!", "//", "#", "<!--"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            return rest.trim_start().trim_end_matches("-->").trim();
        }
    }
    s
}

fn marker_comment_body(line: &str) -> Option<&str> {
    let s = line.trim_start();
    let is_comment = ["///", "//!", "//", "#", "<!--"]
        .iter()
        .any(|prefix| s.starts_with(prefix));
    if !is_comment {
        return None;
    }
    Some(strip_comment_lead(s))
}

fn structured_marker_body(line: &str) -> Option<&str> {
    let s = line.trim_start();
    let key = s
        .strip_prefix("\"aw_ownership\"")
        .or_else(|| s.strip_prefix("aw_ownership"))?;
    let (_, raw_value) = key.split_once([':', '='])?;
    let value = raw_value
        .trim()
        .trim_end_matches(',')
        .trim()
        .trim_matches('"')
        .trim_matches('\'');
    if value.starts_with("CODEGEN-BEGIN")
        || value.starts_with("HANDWRITE-BEGIN")
        || value.starts_with("<HANDWRITE")
    {
        Some(value)
    } else {
        None
    }
}

fn handwrite_marker_body(line: &str) -> Option<&str> {
    let body = marker_comment_body(line)?;
    if is_handwrite_open_marker_line_body(body) {
        Some(body)
    } else {
        None
    }
}

fn is_handwrite_open_marker_line_body(body: &str) -> bool {
    let body = body.trim();
    body.starts_with("HANDWRITE-BEGIN") || (body.starts_with("<HANDWRITE") && body.ends_with('>'))
}

fn is_handwrite_close_marker_line_body(body: &str) -> bool {
    let body = body.trim();
    body == "HANDWRITE-END" || body == "</HANDWRITE>"
}

fn is_handwrite_marker_line_body(body: &str) -> bool {
    is_handwrite_open_marker_line_body(body) || is_handwrite_close_marker_line_body(body)
}

fn extract_attr(body: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    if let Some(start) = body.find(&needle).map(|idx| idx + needle.len()) {
        let rest = &body[start..];
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }

    let needle = format!("{name}=");
    let start = body.find(&needle)? + needle.len();
    let rest = &body[start..];
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let value = rest[..end].trim().trim_matches('"').trim_matches('\'');
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn collect_rust_audit_findings(project_root: &Path, files: &[SourceFile]) -> Vec<RustAuditFinding> {
    let index = match crate::generate::audit::build_spec_file_index(project_root) {
        Ok(index) => index,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for file in files.iter().filter(|f| f.language == "rust") {
        let Ok(reports) =
            crate::generate::audit::audit_file_unified(&file.abs, project_root, &index)
        else {
            continue;
        };
        for report in reports {
            match report {
                crate::generate::audit::UnifiedReport::Drift { file, spec_ref, .. } => {
                    out.push(RustAuditFinding {
                        kind: StandardizeActionKind::RegenDrift,
                        target: rel_display(project_root, &file),
                        reason: format!("CODEGEN block differs from {}", spec_ref),
                    });
                }
                crate::generate::audit::UnifiedReport::MarkerGap { file, line_no, .. } => {
                    out.push(RustAuditFinding {
                        kind: StandardizeActionKind::IssueMarkerGap,
                        target: rel_display(project_root, &file),
                        reason: format!("CODEGEN item at line {} lacks @spec marker", line_no),
                    });
                }
                crate::generate::audit::UnifiedReport::Uncovered { file, line_no, .. } => {
                    out.push(RustAuditFinding {
                        kind: StandardizeActionKind::FoldShadow,
                        target: rel_display(project_root, &file),
                        reason: format!(
                            "spec-claimed item at line {} lives outside markers",
                            line_no
                        ),
                    });
                }
                _ => {}
            }
        }
    }
    out
}

fn find_spec_violation(project_root: &Path, scopes: &[String]) -> Result<Option<SpecViolation>> {
    let spec_roots = spec_roots_for_scopes(project_root, scopes)?;
    if spec_roots.is_empty() {
        return Ok(None);
    }
    let mut files = BTreeSet::new();
    for root in spec_roots {
        if root.is_file() {
            if root.extension().and_then(|e| e.to_str()) == Some("md") {
                files.insert(root);
            }
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|e| e.to_str()) == Some("md")
            {
                files.insert(entry.path().to_path_buf());
            }
        }
    }
    let files: Vec<_> = files.into_iter().collect();
    if files.is_empty() {
        return Ok(None);
    }
    let report = crate::validate::run_rules(&files);
    let Some(finding) = report
        .findings
        .iter()
        .find(|f| f.severity == crate::validate::Severity::Error)
    else {
        return Ok(None);
    };
    Ok(Some(SpecViolation {
        target: rel_display(project_root, &finding.file),
        reason: finding.message.clone(),
    }))
}

fn spec_roots_for_scopes(project_root: &Path, scopes: &[String]) -> Result<Vec<PathBuf>> {
    let legacy_td_root = crate::shared::workspace::tech_design_path(project_root);
    let configured = read_config_workspace_scopes(project_root)?;
    if scopes.is_empty() || scopes.iter().any(|scope| scope_is_global(scope)) {
        let mut roots = BTreeSet::new();
        if legacy_td_root.exists() {
            roots.insert(legacy_td_root);
        }
        for configured_scope in &configured {
            if let Some(root) = configured_td_root(project_root, configured_scope) {
                if root.exists() {
                    roots.insert(root);
                }
            }
        }
        return Ok(roots.into_iter().collect());
    }

    let mut roots = BTreeSet::new();
    for scope in scopes {
        let mut matched_config = false;
        for configured_scope in &configured {
            if !scopes_overlap(scope, &configured_scope.scope) {
                continue;
            }
            matched_config = true;
            if let Some(root) = configured_td_root(project_root, configured_scope) {
                if root.exists() {
                    roots.insert(root);
                }
            }
        }
        if matched_config {
            continue;
        }
        if let Some(root) = fallback_spec_root_for_scope(project_root, scope) {
            if root.exists() {
                roots.insert(root);
            }
        }
    }
    Ok(roots.into_iter().collect())
}

fn scope_is_global(scope: &str) -> bool {
    let scope = scope.trim();
    scope.is_empty() || scope == "." || scope == "**"
}

fn scopes_overlap(left: &str, right: &str) -> bool {
    let left = scope_static_prefix(left);
    let right = scope_static_prefix(right);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    left == right || path_prefix_of(&left, &right) || path_prefix_of(&right, &left)
}

fn path_prefix_of(prefix: &str, path: &str) -> bool {
    path.strip_prefix(prefix)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

fn fallback_spec_root_for_scope(project_root: &Path, scope: &str) -> Option<PathBuf> {
    let prefix = scope_static_prefix(scope);
    if prefix.is_empty() {
        return Some(crate::shared::workspace::tech_design_path(project_root));
    }
    let mut rel = PathBuf::from(prefix);
    if rel.extension().and_then(|e| e.to_str()).is_some() {
        rel.set_extension("md");
    }
    Some(crate::shared::workspace::tech_design_path(project_root).join(rel))
}

fn scope_static_prefix(scope: &str) -> String {
    scope
        .replace('\\', "/")
        .trim()
        .trim_start_matches("./")
        .split(['*', '?', '['])
        .next()
        .unwrap_or(scope)
        .trim_end_matches('/')
        .to_string()
}

#[allow(dead_code)]
fn choose_codegen_action(inventory: &Inventory) -> StandardizeAction {
    if let Some(file) = inventory.files.iter().find(|f| !f.markers.managed()) {
        return action(
            StandardizeActionKind::ClaimCode,
            &file.rel,
            "cli",
            &format!(
                "aw standardize managed run --scope {} --max-ticks 1",
                shell_quote(&file.rel)
            ),
            "source file must be managed before the codegen-only pass",
            false,
        );
    }

    if let Some(file) = inventory.files.iter().find(|f| f.markers.handwrite) {
        return action(
            StandardizeActionKind::PromoteHandwrite,
            &file.rel,
            "mainthread",
            "mainthread: close the generator gap, rerun codegen, and replace HANDWRITE with CODEGEN",
            "HANDWRITE ownership remains; promote this region to CODEGEN",
            true,
        );
    }

    action(
        StandardizeActionKind::None,
        "",
        "none",
        "",
        "all in-scope source files are CODEGEN-owned",
        false,
    )
}

#[cfg(test)]
fn choose_semantic_action(coverage: &SemanticCoverage) -> StandardizeAction {
    choose_semantic_action_with_project(coverage, None)
}

// Kept test-only (issue #860 cleanup): the only live caller is the
// `#[cfg(test)]` wrapper `choose_semantic_action` above, exercised by
// `semantic_action_returns_next_deterministic_gap_as_cli_tick`.
#[cfg(test)]
fn choose_semantic_action_with_project(
    coverage: &SemanticCoverage,
    project: Option<&str>,
) -> StandardizeAction {
    if let Some(gap) = &coverage.next_gap {
        if matches!(
            gap.primitive.as_str(),
            "semantic_td_missing" | "semantic_td_legacy"
        ) {
            let scope = semantic_group_scope(&gap.target);
            return action(
                StandardizeActionKind::SemanticGap,
                &gap.target,
                "cli",
                &format!(
                    "aw standardize semantic run --scope {} --max-ticks 1",
                    shell_quote(&scope)
                ),
                &gap.reason,
                false,
            );
        }
        return action(
            StandardizeActionKind::GeneratorPrimitiveGap,
            &gap.target,
            if project.is_some() {
                "cli"
            } else {
                "mainthread"
            },
            &project
                .map(|project| format!("aw generator check --project {project}"))
                .unwrap_or_else(|| {
                    "mainthread: identify configured project, then run aw generator check --project <project>"
                        .to_string()
                }),
            &gap.reason,
            project.is_none(),
        );
    }
    if coverage.human_decision_required_count > 0 {
        return action(
            StandardizeActionKind::Blocked,
            "",
            "human",
            "human: resolve semantic grouping or generator primitive design decisions",
            "only human-decision semantic gaps remain",
            true,
        );
    }
    action(
        StandardizeActionKind::None,
        "",
        "none",
        "",
        "all in-scope source units have semantic TD coverage and no deterministic semantic/generator gap remains",
        false,
    )
}

#[cfg(test)]
fn choose_regenerable_action(
    project_root: &Path,
    inventory: &Inventory,
    semantic: &SemanticCoverage,
) -> StandardizeAction {
    choose_regenerable_action_with_project(project_root, inventory, semantic, None)
}

#[cfg(test)]
fn choose_regenerable_action_with_project(
    project_root: &Path,
    inventory: &Inventory,
    semantic: &SemanticCoverage,
    replay_project: Option<&str>,
) -> StandardizeAction {
    if let Some(finding) = inventory
        .rust_findings
        .iter()
        .find(|f| f.kind == StandardizeActionKind::RegenDrift)
    {
        return action(
            StandardizeActionKind::RegenDrift,
            &finding.target,
            "mainthread",
            "mainthread: regenerate affected CODEGEN block and rerun aw td code-check",
            &finding.reason,
            true,
        );
    }

    if let Some(finding) = inventory
        .rust_findings
        .iter()
        .find(|f| f.kind == StandardizeActionKind::IssueMarkerGap)
    {
        return action(
            StandardizeActionKind::IssueMarkerGap,
            &finding.target,
            "mainthread",
            "mainthread: add the missing @spec marker or revise the enclosing CODEGEN block",
            &finding.reason,
            true,
        );
    }

    if let Some(violation) = &inventory.spec_violation {
        return action(
            StandardizeActionKind::FixSpecRule,
            &violation.target,
            "mainthread",
            "mainthread: revise TD spec and rerun aw td check",
            &violation.reason,
            true,
        );
    }

    if let Some(finding) = inventory
        .rust_findings
        .iter()
        .find(|f| f.kind == StandardizeActionKind::FoldShadow)
    {
        return action(
            StandardizeActionKind::FoldShadow,
            &finding.target,
            "mainthread",
            "mainthread: convert the shadow region to CODEGEN or a tracked HANDWRITE gap",
            &finding.reason,
            true,
        );
    }

    if let Some(file) = inventory
        .files
        .iter()
        .find(|f| f.handwrite_gaps.iter().any(|gap| gap.needs_promotion))
    {
        let gap = file
            .handwrite_gaps
            .iter()
            .find(|gap| gap.needs_promotion)
            .expect("promotion gap");
        return action(
            StandardizeActionKind::PromoteHandwrite,
            &file.rel,
            "cli",
            &format!(
                "aw standardize managed run --scope {}",
                shell_quote(&file.rel)
            ),
            &format!(
                "legacy HANDWRITE marker at line {} needs managed metadata before CODEGEN promotion: {}",
                gap.line_no, gap.message
            ),
            false,
        );
    }

    if let Some(file) = inventory
        .files
        .iter()
        .find(|f| !f.handwrite_gaps.is_empty())
    {
        let gap = &file.handwrite_gaps[0];
        return action(
            StandardizeActionKind::IssueMarkerGap,
            &file.rel,
            "cli",
            &format!(
                "aw standardize managed run --scope {}",
                shell_quote(&file.rel)
            ),
            &format!(
                "HANDWRITE marker at line {} needs managed metadata before CODEGEN promotion: {}",
                gap.line_no, gap.message
            ),
            false,
        );
    }

    if let Some(file) = inventory.files.iter().find(|f| !f.markers.managed()) {
        return action(
            StandardizeActionKind::ClaimCode,
            &file.rel,
            "cli",
            &format!(
                "aw standardize managed run --scope {} --max-ticks 1",
                shell_quote(&file.rel)
            ),
            "source file is not managed yet; run the managed layer before regenerability",
            false,
        );
    }

    if let Some(file) = inventory
        .files
        .iter()
        .find(|f| f.markers.codegen && !f.markers.handwrite && !codegen_replay_supported(f))
    {
        return action(
            StandardizeActionKind::GeneratorPrimitiveGap,
            &file.rel,
            "mainthread",
            "mainthread: implement replay-capable generator support for this CODEGEN-owned file class, then rerun aw td gen --force-regen --project <project> --verify",
            &format!(
                "{} is CODEGEN-marked but its language/workspace is not replay-verifiable by current generators",
                file.language
            ),
            true,
        );
    }

    if inventory.files.iter().any(|file| file.markers.codegen) {
        match collect_codegen_gap_files(project_root, inventory, replay_project) {
            Ok(files) => {
                if let Some(file) = files.first() {
                    return action(
                        StandardizeActionKind::RegenDrift,
                        file,
                        "mainthread",
                        "mainthread: repair CODEGEN replay drift, then rerun aw td gen --force-regen --project <project> --verify",
                        "CODEGEN-owned file is not audit/replay clean",
                        true,
                    );
                }
            }
            Err(err) => {
                return action(
                    StandardizeActionKind::Blocked,
                    "cb-audit",
                    "mainthread",
                    "mainthread: fix cb audit failure before continuing regenerability",
                    &format!("failed to audit CODEGEN replay drift: {err}"),
                    true,
                );
            }
        }
    }

    if let Some(gap) = semantic
        .generator_primitive_gaps
        .iter()
        .find(|gap| gap.primitive == "semantic_td_missing" && !gap.human_decision_required)
    {
        let scope = semantic_group_scope(&gap.target);
        return action(
            StandardizeActionKind::SemanticGap,
            &gap.target,
            "cli",
            &format!(
                "aw standardize semantic run --scope {} --max-ticks 1",
                shell_quote(&scope)
            ),
            &gap.reason,
            false,
        );
    }

    if let Some(file) = inventory.files.iter().find(|f| f.markers.handwrite) {
        if let Some(gap) = semantic
            .generator_primitive_gaps
            .iter()
            .find(|gap| gap.target == file.rel && gap.primitive != "semantic_td_missing")
        {
            if file.language == "python"
                || is_operations_language(&file.language)
                || is_frontend_promotable_file(file)
                || is_rust_test_promotable_file(file)
                || is_rust_mixed_source_promotable_file(file)
                || is_rust_source_promotable_file(file)
            {
                let command = replay_project
                    .map(|project| format!("aw generator check --project {project}"))
                    .unwrap_or_else(|| {
                        "mainthread: identify configured project, then run aw generator check --project <project>"
                            .to_string()
                    });
                return action(
                    StandardizeActionKind::GeneratorPrimitiveGap,
                    &file.rel,
                    if replay_project.is_some() {
                        "cli"
                    } else {
                        "mainthread"
                    },
                    &command,
                    &gap.reason,
                    replay_project.is_none(),
                );
            }
            return action(
                StandardizeActionKind::GeneratorPrimitiveGap,
                &file.rel,
                "mainthread",
                "mainthread: extend the generator primitive from semantic coverage, rerun codegen, then replace HANDWRITE with CODEGEN",
                &gap.reason,
                true,
            );
        }
        return action(
            StandardizeActionKind::PromoteHandwrite,
            &file.rel,
            "mainthread",
            "mainthread: replace HANDWRITE with CODEGEN by extending the spec/generator, then rerun aw td code-check",
            "managed HANDWRITE remains; full regenerability requires CODEGEN ownership",
            true,
        );
    }

    action(
        StandardizeActionKind::None,
        "",
        "none",
        "",
        "all in-scope source files are fully CODEGEN-owned",
        false,
    )
}

fn action(
    kind: StandardizeActionKind,
    target: &str,
    executor: &str,
    command: &str,
    reason: &str,
    requires_hitl: bool,
) -> StandardizeAction {
    let id = if target.is_empty() {
        format!("{:?}", kind).to_ascii_lowercase()
    } else {
        format!("{:?}:{}", kind, target)
            .to_ascii_lowercase()
            .replace(['/', ' ', '\\'], "-")
    };
    StandardizeAction {
        id,
        kind,
        target: target.to_string(),
        executor: executor.to_string(),
        command: command.to_string(),
        reason: reason.to_string(),
        requires_hitl,
    }
}

#[allow(dead_code)]
fn write_starter_spec(
    project_root: &Path,
    rel: &str,
    configured: &[ConfiguredScope],
    symbols_yaml: &str,
) -> Result<PathBuf> {
    let spec_rel = starter_spec_rel_with_config(rel, configured);
    let spec_abs = project_root.join(&spec_rel);
    if let Some(parent) = spec_abs.parent() {
        fs::create_dir_all(parent)?;
    }
    if spec_abs.exists() {
        return Ok(PathBuf::from(spec_rel));
    }
    let id = slug_for_path(rel);
    let content = format!(
        "---\nid: {id}\nfill_sections: [changes]\n---\n\n# Standardized {rel}\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: {rel}\n    action: modify\n    impl_mode: hand-written\n    description: |\n      Existing source claimed by `aw standardize managed run`. The code is\n      wrapped in a tracked HANDWRITE block until deterministic generator\n      coverage can replace it with CODEGEN.\n{symbols_yaml}```\n",
    );
    fs::write(&spec_abs, content)?;
    Ok(PathBuf::from(spec_rel))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticCapabilityRef {
    id: String,
    claim: Option<String>,
}

fn semantic_capability_ref_for_group(
    project_root: &Path,
    configured: &[ConfiguredScope],
    group_key: &str,
) -> Option<SemanticCapabilityRef> {
    let document = semantic_capability_document_for_group(project_root, configured, group_key)?;
    let capability_ids = document.capability_ids();
    let mut candidates = semantic_capability_candidates(group_key)
        .into_iter()
        .filter(|id| capability_ids.contains(*id))
        .map(str::to_string)
        .collect::<Vec<_>>();
    for capability in &document.capabilities {
        if !candidates.contains(&capability.id) {
            candidates.push(capability.id.clone());
        }
    }

    for id in candidates {
        let Some(capability) = document
            .capabilities
            .iter()
            .find(|capability| capability.id == id)
        else {
            continue;
        };
        if capability.status == crate::cli::capability::CapabilityStatus::Retired {
            continue;
        }
        let claim = match &capability.verification_contract {
            Some(contract) => Some(contract.claims.first()?.id.clone()),
            None => None,
        };
        return Some(SemanticCapabilityRef {
            id: capability.id.clone(),
            claim,
        });
    }
    None
}

fn semantic_capability_document_for_group(
    project_root: &Path,
    configured: &[ConfiguredScope],
    group_key: &str,
) -> Option<crate::cli::capability::CapabilityDocument> {
    let scope = configured_scope_for_path(configured, group_key)?;
    let cap_abs = if let Some(cap_path) = &scope.cap_path {
        let path = Path::new(cap_path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            project_root.join(path)
        }
    } else {
        let project = scope.project_name.as_deref()?;
        crate::cli::capability::resolve_capability_path(project_root, project, None).ok()?
    };
    let body = fs::read_to_string(&cap_abs).ok()?;
    crate::cli::capability::parse_capability_document(&body, &cap_abs).ok()
}

fn configured_scope_for_path<'a>(
    configured: &'a [ConfiguredScope],
    rel: &str,
) -> Option<&'a ConfiguredScope> {
    configured
        .iter()
        .filter_map(|scope| {
            let prefix = scope_static_prefix(&scope.scope);
            if prefix.is_empty() || !path_prefix_of(&prefix, rel) {
                return None;
            }
            Some((prefix.len(), scope))
        })
        .max_by_key(|(len, _)| *len)
        .map(|(_, scope)| scope)
}

fn semantic_capability_candidates(group_key: &str) -> Vec<&'static str> {
    let lower = group_key.to_ascii_lowercase();
    let mut candidates = Vec::new();
    if lower.contains("/k8s") || lower.contains("/operator") {
        candidates.push("k8s-deployment");
    }
    if lower.contains("llm") || lower.contains("spec") {
        candidates.push("agentic-integration");
    }
    if lower.contains("auth") || lower.contains("tls") {
        candidates.push("security-auth");
    }
    if lower.contains("backup") || lower.contains("rdb") {
        candidates.push("backup-restore");
    }
    if lower.contains("metrics") || lower.contains("observability") {
        candidates.push("observability");
    }
    if lower.contains("bench") || lower.contains("perf") {
        candidates.push("ops-operability");
    }
    candidates.push("search");
    candidates
}

fn operation_artifact_kind(file: &SourceFile) -> &'static str {
    match file.language.as_str() {
        "dockerfile" => "dockerfile",
        "dockerignore" => "dockerignore",
        "kustomize" => {
            if file.rel.ends_with("kustomization.yaml") || file.rel.ends_with("kustomization.yml") {
                "kustomization"
            } else {
                "kubernetes-manifest"
            }
        }
        _ => "source",
    }
}

fn is_frontend_promotable_file(file: &SourceFile) -> bool {
    matches!(
        file.language.as_str(),
        "javascript" | "json" | "stylesheet" | "typescript"
    ) && frontend_workspace_root(&file.rel).is_some()
}

fn is_rust_test_promotable_file(file: &SourceFile) -> bool {
    file.language == "rust"
        && file
            .rel
            .split('/')
            .any(|segment| segment == "tests" || segment == "test")
}

fn is_rust_source_promotable_file(file: &SourceFile) -> bool {
    file.language == "rust" && !file.markers.codegen && !is_rust_test_promotable_file(file)
}

fn is_rust_mixed_source_promotable_file(file: &SourceFile) -> bool {
    file.language == "rust"
        && file.markers.codegen
        && file.markers.handwrite
        && !is_rust_test_promotable_file(file)
}

fn has_executable_generator_promotion(file: &SourceFile) -> bool {
    file.language == "python"
        || is_operations_language(&file.language)
        || is_frontend_promotable_file(file)
        || is_rust_test_promotable_file(file)
        || is_rust_mixed_source_promotable_file(file)
        || is_rust_source_promotable_file(file)
}

fn codegen_replay_supported(file: &SourceFile) -> bool {
    if matches!(
        file.language.as_str(),
        "dockerfile"
            | "javascript"
            | "json"
            | "llms"
            | "python"
            | "rust"
            | "shell"
            | "stylesheet"
            | "toml"
            | "typescript"
            | "yaml"
    ) {
        return true;
    }
    let path = Path::new(&file.rel);
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(
            "cjs"
                | "css"
                | "js"
                | "json"
                | "jsx"
                | "mjs"
                | "py"
                | "scss"
                | "sh"
                | "toml"
                | "ts"
                | "tsx"
                | "yaml"
                | "yml"
        )
    ) || path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(name, ".dockerignore" | "llms.txt")
                || name == "Dockerfile"
                || name.starts_with("Dockerfile.")
                || name.ends_with(".dockerfile")
        })
}

fn promote_handwrite_blocks_to_codegen(
    path: &Path,
    content: &str,
    fallback_spec_ref: &str,
) -> String {
    let trailing_newline = content.ends_with('\n');
    let comment = comment_prefix(path);
    let mut out: Vec<String> = Vec::new();
    let mut in_promoted_block = false;

    for line in content.lines() {
        let body = marker_comment_body(line);
        let is_open = body.is_some_and(is_handwrite_open_marker_line_body);
        let is_close = body.is_some_and(is_handwrite_close_marker_line_body);

        if is_open {
            let previous_has_spec = out
                .iter()
                .rev()
                .find_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    Some(
                        marker_comment_body(line)
                            .is_some_and(|body| body.starts_with("SPEC-MANAGED:")),
                    )
                })
                .unwrap_or(false);
            if !previous_has_spec {
                out.push(format!("{comment}SPEC-MANAGED: {fallback_spec_ref}"));
            }
            out.push(format!("{comment}CODEGEN-BEGIN"));
            in_promoted_block = true;
            continue;
        }

        if in_promoted_block && is_close {
            out.push(format!("{comment}CODEGEN-END"));
            in_promoted_block = false;
            continue;
        }

        out.push(line.to_string());
    }

    let mut rendered = out.join("\n");
    if trailing_newline {
        rendered.push('\n');
    }
    rendered
}

fn is_operations_language(language: &str) -> bool {
    matches!(language, "dockerfile" | "dockerignore" | "kustomize")
}

// Resolve the TD output path for a source claim, consulting
// `[[projects]].td_path` in `aw.toml` first and falling back to
// `<project.path>/tech-design`. This lets per-project `td_path` config
// (e.g. `examples/fixture_platform/**` → `examples/fixture_platform/tech_design`)
// keep legacy/external roots while convention-first projects write TDs inside
// the project tree.
#[allow(dead_code)]
fn starter_spec_rel_with_config(rel: &str, configured: &[ConfiguredScope]) -> String {
    let best = configured
        .iter()
        .filter_map(|cs| {
            let td = configured_td_path(cs)?;
            let prefix = scope_static_prefix(&cs.scope);
            if prefix.is_empty() {
                return None;
            }
            if path_prefix_of(&prefix, rel) {
                Some((prefix, td))
            } else {
                None
            }
        })
        .max_by_key(|(p, _)| p.len());

    if let Some((prefix, td_path)) = best {
        let rest = rel
            .strip_prefix(&prefix)
            .unwrap_or(rel)
            .trim_start_matches('/');
        let bucketed = bucket_under_allowed_top_dir(rest);
        let joined = if bucketed.is_empty() {
            format!("{}.md", td_path.trim_end_matches('/'))
        } else {
            format!("{}/{bucketed}.md", td_path.trim_end_matches('/'))
        };
        return replace_source_extension(&joined);
    }

    starter_spec_rel_for_source(rel)
}

fn semantic_spec_rel_with_config(rel: &str, configured: &[ConfiguredScope]) -> String {
    let best = configured
        .iter()
        .filter_map(|cs| {
            let td = configured_td_path(cs)?;
            let prefix = scope_static_prefix(&cs.scope);
            if prefix.is_empty() || !path_prefix_of(&prefix, rel) {
                return None;
            }
            Some((prefix, td))
        })
        .max_by_key(|(p, _)| p.len());
    let group_key = semantic_group_key(rel);
    let file_name = format!("{}.md", semantic_spec_slug(&group_key, configured));

    if let Some((_, td_path)) = best {
        return format!("{}/semantic/{file_name}", td_path.trim_end_matches('/'));
    }
    format!(".aw/tech-design/semantic/{file_name}")
}

fn semantic_spec_slug(group_key: &str, configured: &[ConfiguredScope]) -> String {
    let cleaned = semantic_group_display_key(group_key, configured);
    let slug = slug_for_path(&cleaned);
    if slug.is_empty() {
        slug_for_path(group_key)
    } else {
        slug
    }
}

fn semantic_group_display_key(group_key: &str, configured: &[ConfiguredScope]) -> String {
    let stripped = configured
        .iter()
        .filter_map(|cs| {
            let prefix = scope_static_prefix(&cs.scope);
            if prefix.is_empty() || !path_prefix_of(&prefix, group_key) {
                None
            } else {
                Some(prefix)
            }
        })
        .max_by_key(|prefix| prefix.len())
        .and_then(|prefix| {
            group_key
                .strip_prefix(&prefix)
                .map(|rest| rest.trim_start_matches('/').to_string())
        })
        .unwrap_or_else(|| group_key.to_string());

    let cleaned =
        clean_semantic_group_display_key(&stripped).unwrap_or_else(|| group_key.to_string());
    if let Some(workspace) = best_configured_scope_leaf(group_key, configured) {
        if !cleaned
            .strip_prefix(&workspace)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
        {
            return format!("{workspace}/{cleaned}");
        }
    }
    cleaned
}

fn best_configured_scope_leaf(group_key: &str, configured: &[ConfiguredScope]) -> Option<String> {
    configured
        .iter()
        .filter_map(|cs| {
            let prefix = scope_static_prefix(&cs.scope);
            if prefix.is_empty() || !path_prefix_of(&prefix, group_key) {
                None
            } else {
                Some(prefix)
            }
        })
        .max_by_key(|prefix| prefix.len())
        .and_then(|prefix| {
            prefix
                .rsplit('/')
                .find(|part| !part.is_empty())
                .map(str::to_string)
        })
}

fn clean_semantic_group_display_key(rest: &str) -> Option<String> {
    let trimmed = rest.trim_matches('/');
    if trimmed.is_empty() {
        return None;
    }

    for prefix in ["backend/src/features/", "src/features/"] {
        if let Some(value) = trimmed.strip_prefix(prefix) {
            return non_empty_string(value);
        }
    }
    for prefix in ["backend/tests/features/", "tests/features/"] {
        if let Some(value) = trimmed.strip_prefix(prefix) {
            return non_empty_string(&format!("tests/{value}"));
        }
    }
    for prefix in ["backend/src/", "src/"] {
        if let Some(value) = trimmed.strip_prefix(prefix) {
            return non_empty_string(value);
        }
    }
    for prefix in ["backend/tests/", "tests/"] {
        if let Some(value) = trimmed.strip_prefix(prefix) {
            return non_empty_string(&format!("tests/{value}"));
        }
    }

    Some(trimmed.to_string())
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim_matches('/');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn semantic_group_key(rel: &str) -> String {
    let path = Path::new(rel);
    if is_dockerfile_path(path) || is_dockerignore_path(path) {
        if let Some(parent) = rel.rsplit_once('/').map(|(parent, _)| parent) {
            if !parent.is_empty() {
                return format!("{parent}/runtime-image");
            }
        }
        return "runtime-image".to_string();
    }
    if let Some(group) = frontend_domain_group_key(rel) {
        return group;
    }
    if is_frontend_manifest_json_path(path) || is_frontend_config_path(path) {
        if let Some(root) = frontend_workspace_root(rel) {
            return root;
        }
    }
    if is_frontend_test_path(rel) || is_frontend_route_path(rel) {
        if let Some(parent) = rel.rsplit_once('/').map(|(parent, _)| parent) {
            if !parent.is_empty() {
                return parent.to_string();
            }
        }
    }

    let parts: Vec<&str> = rel.split('/').collect();
    if let Some(idx) = parts.iter().position(|part| *part == "features") {
        let end = (idx + 3).min(parts.len().saturating_sub(1));
        if end > idx {
            return parts[..end].join("/");
        }
    }
    if let Some(idx) = parts.iter().position(|part| *part == "tests") {
        let end = (idx + 3).min(parts.len().saturating_sub(1));
        if end > idx {
            return parts[..end].join("/");
        }
    }
    if let Some(parent) = rel.rsplit_once('/').map(|(parent, _)| parent) {
        if !parent.is_empty() {
            return parent.to_string();
        }
    }
    rel.to_string()
}

// Kept test-only (issue #860 cleanup): the only live callers are
// `choose_semantic_action_with_project` (test-only) and the kept
// `choose_regenerable_action_with_project` scanner.
#[cfg(test)]
fn semantic_group_scope(rel: &str) -> String {
    let group_key = semantic_group_key(rel);
    if let Some(parent) = group_key.strip_suffix("/runtime-image") {
        return format!("{parent}/**");
    }
    if group_key == "runtime-image" {
        return rel.to_string();
    }
    if group_key == rel {
        rel.to_string()
    } else {
        format!("{}/**", group_key)
    }
}

fn frontend_domain_group_key(rel: &str) -> Option<String> {
    let parts: Vec<&str> = rel.split('/').collect();
    if !parts.iter().any(|part| *part == "frontend") || frontend_workspace_root(rel).is_none() {
        return None;
    }
    let src_idx = parts.iter().position(|part| *part == "src")?;
    let marker_idx = parts
        .iter()
        .enumerate()
        .skip(src_idx + 1)
        .find_map(|(idx, part)| {
            matches!(
                *part,
                "features"
                    | "shared"
                    | "components"
                    | "api"
                    | "helpers"
                    | "models"
                    | "schemas"
                    | "constants"
                    | "utils"
            )
            .then_some(idx)
        })?;

    match parts[marker_idx] {
        "features" | "shared" | "components" | "api" | "helpers" => {
            if marker_idx + 1 < parts.len().saturating_sub(1) {
                Some(parts[..=marker_idx + 1].join("/"))
            } else {
                Some(parts[..=marker_idx].join("/"))
            }
        }
        "models" | "schemas" | "constants" | "utils" => Some(parts[..=marker_idx].join("/")),
        _ => None,
    }
}

// Bucket a workspace-relative source path under one of R6b's allowed
// top-level spec subdirs (interfaces, logic, config, tools, skills,
// generate, validate). Keeps the source's directory structure inside
// the bucket so the mapping is deterministic and reversible.
///
// Rules (first match wins):
//   * already starts with an allowed dir → keep as-is
//   * any `tests/` or `test/` segment → bucket under `validate/`
//   * default → bucket under `logic/`
fn bucket_under_allowed_top_dir(rest: &str) -> String {
    let allowed = [
        "interfaces",
        "logic",
        "config",
        "tools",
        "skills",
        "generate",
        "validate",
    ];
    let first = rest.split('/').next().unwrap_or("");
    if allowed.contains(&first) {
        return rest.to_string();
    }
    let is_test_path = rest
        .split('/')
        .any(|seg| seg == "tests" || seg == "test" || seg.starts_with("test_"));
    let bucket = if is_test_path { "validate" } else { "logic" };
    if rest.is_empty() {
        bucket.to_string()
    } else {
        format!("{bucket}/{rest}")
    }
}

// Parse the source at `abs` with the SDD AST analyzer and render a YAML
// fragment listing the extracted symbols. Returns an empty string when the
// file cannot be parsed or has no symbols — callers embed the result inside
// the `changes[].` block of the starter TD as additional structural metadata.
#[allow(dead_code)]
fn render_ast_symbols_yaml(abs: &Path) -> String {
    let Ok(content) = fs::read_to_string(abs) else {
        return String::new();
    };
    let Ok(mut analyzer) = crate::fillback::AstAnalyzer::new() else {
        return String::new();
    };
    let Ok(module) = analyzer.parse_file(abs, &content) else {
        return String::new();
    };
    if module.symbols.is_empty() {
        return String::new();
    }
    let mut out = String::from("    language: ");
    out.push_str(module.language.display_name());
    out.push('\n');
    if !module.imports.is_empty() {
        out.push_str("    imports:\n");
        for imp in &module.imports {
            out.push_str(&format!("      - path: {}\n", yaml_safe(&imp.path)));
            if !imp.items.is_empty() {
                let mut seen = BTreeSet::new();
                let mut deduped: Vec<&String> = Vec::with_capacity(imp.items.len());
                for item in &imp.items {
                    if seen.insert(item.as_str()) {
                        deduped.push(item);
                    }
                }
                let items_yaml = deduped
                    .iter()
                    .map(|i| yaml_safe(i))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("        items: [{items_yaml}]\n"));
            }
            out.push_str(&format!("        external: {}\n", imp.is_external));
        }
    }
    out.push_str("    symbols:\n");
    for sym in &module.symbols {
        out.push_str(&format!(
            "      - name: {}\n        kind: {}\n        line: {}\n        public: {}\n",
            yaml_safe(&sym.name),
            sym.kind,
            sym.line,
            sym.is_public
        ));
        if let Some(sig) = &sym.signature {
            if sig.contains('\n') {
                out.push_str("        signature: |\n");
                for line in sig.split('\n') {
                    out.push_str("          ");
                    out.push_str(line);
                    out.push('\n');
                }
            } else {
                out.push_str(&format!("        signature: {}\n", yaml_safe(sig)));
            }
        }
        if let Some(doc) = &sym.doc {
            if let Some(first) = doc.lines().find(|l| !l.trim().is_empty()) {
                let trimmed = first.trim();
                let snippet: String = trimmed.chars().take(200).collect();
                out.push_str(&format!("        doc: {}\n", yaml_safe(&snippet)));
            }
        }
    }
    out
}

// Quote a string for safe embedding as a YAML scalar value. Conservative:
// always wraps in double quotes and escapes `\` and `"`. The starter-TD
// embedding only carries identifier-ish strings so this is sufficient.
fn yaml_safe(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn starter_spec_rel_for_source(rel: &str) -> String {
    let parts: Vec<&str> = rel.split('/').collect();
    if parts.len() >= 4 && matches!(parts[0], "crates" | "projects" | "apps") {
        let root = parts[0];
        let crate_name = parts[1];
        let kind = parts[2];
        let rest = parts[3..].join("/");
        let spec_base = starter_spec_base(root, crate_name);
        let mapped = match kind {
            "tests" => Some(format!("{spec_base}/validate/{rest}.md")),
            "examples" => Some(format!("{spec_base}/validate/examples/{rest}.md")),
            "packages" => Some(format!("{spec_base}/interfaces/packages/{rest}.md")),
            "src" => starter_spec_rel_for_crate_src(root, crate_name, &parts[3..]),
            _ => None,
        };
        if let Some(path) = mapped {
            return replace_source_extension(&path);
        }
    }

    replace_source_extension(&format!(".aw/tech-design/{rel}.md"))
}

fn starter_spec_base(root: &str, crate_name: &str) -> String {
    if matches!(root, "projects" | "apps") && crate_name == "agentic-workflow" {
        "apps/agentic-workflow/tech-design/core".to_string()
    } else {
        format!(".aw/tech-design/{root}/{crate_name}")
    }
}

fn starter_spec_rel_for_crate_src(root: &str, crate_name: &str, rest: &[&str]) -> Option<String> {
    let module = rest.first()?;
    if matches!(root, "projects" | "apps") && crate_name == "agentic-workflow" && *module == "cli" {
        let rel = if rest.len() > 1 {
            rest[1..].join("/")
        } else {
            "mod".to_string()
        };
        return Some(format!(
            "apps/agentic-workflow/tech-design/surface/interfaces/src/{rel}.md"
        ));
    }
    let spec_base = starter_spec_base(root, crate_name);
    if rest.len() == 1 {
        let spec_root = if module.starts_with("test_") {
            "validate"
        } else {
            "logic"
        };
        return Some(format!("{spec_base}/{spec_root}/{}.md", rest.join("/")));
    }
    let path = match *module {
        "generate" => format!("{spec_base}/generate/{}.md", rest[1..].join("/")),
        "tools" => format!("{spec_base}/tools/{}.md", rest[1..].join("/")),
        "config" => format!("{spec_base}/config/{}.md", rest[1..].join("/")),
        "skills" => format!("{spec_base}/skills/{}.md", rest[1..].join("/")),
        "validate" => format!("{spec_base}/validate/{}.md", rest[1..].join("/")),
        "validator" => format!("{spec_base}/validate/{}.md", rest.join("/")),
        "runtime" | "context_builder" => format!("{spec_base}/logic/{}.md", rest.join("/")),
        _ => format!("{spec_base}/interfaces/{}.md", rest.join("/")),
    };
    Some(path)
}

fn replace_source_extension(path: &str) -> String {
    for ext in SOURCE_EXTS {
        let suffix = format!(".{ext}.md");
        if let Some(prefix) = path.strip_suffix(&suffix) {
            return format!("{prefix}.md");
        }
    }
    path.to_string()
}

/// Outcome of a real HANDWRITE→CODEGEN marker flip — distinct from
/// `promote_handwrite` above, which repairs legacy marker *attributes*
/// (a missing `gap`) and never changes the marker keyword itself.
pub(crate) struct PromoteOutcome {
    pub(crate) changed_paths: Vec<PathBuf>,
    pub(crate) message: String,
}

/// Result of attempting the flip in "soft" (non-erroring) mode — used by
/// `aw standardize managed run`'s `PromoteHandwrite` action, which must
/// never fail the tick loop just because a marker isn't closure-ready yet.
enum PromoteAttempt {
    Promoted(PromoteOutcome),
    NotReady(String),
}

/// Resolve a `aw td promote` target (a repo-relative path, or a HANDWRITE
/// marker tracker id / default gap-slug) to the owning source file's
/// repo-relative path.
pub(crate) fn resolve_promote_target(project_root: &Path, target: &str) -> Result<String> {
    let inventory = build_inventory(project_root, &[], None, true)?;
    if inventory.files.iter().any(|f| f.rel == target) {
        return Ok(target.to_string());
    }
    for file in &inventory.files {
        if slug_for_path(&file.rel) == target {
            return Ok(file.rel.clone());
        }
        let content = fs::read_to_string(&file.abs).unwrap_or_default();
        if handwrite_block_open_attrs(&content)
            .iter()
            .any(|(_, _, tracker)| tracker == target)
        {
            return Ok(file.rel.clone());
        }
    }
    bail!(
        "no HANDWRITE marker found for target `{target}` (expected a repo-relative path or a marker tracker id)"
    );
}

/// Default `<mirror>.md#anchor` spec reference for a promoted file's fresh
/// SPEC-MANAGED header, derived the same way
/// `promote_rust_mixed_source_generator_primitive` derives one for its own
/// HANDWRITE-block flips.
pub(crate) fn default_promote_spec_ref(project_root: &Path, rel: &str) -> String {
    let configured = read_config_workspace_scopes(project_root).unwrap_or_default();
    format!("{}#schema", semantic_spec_rel_with_config(rel, &configured))
}

fn handwrite_block_open_attrs(content: &str) -> Vec<(usize, String, String)> {
    let raw_string_lines = crate::generate::marker::rust_raw_string_line_mask(content);
    let mut out = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if raw_string_lines.get(idx).copied().unwrap_or(false) {
            continue;
        }
        let Some(body) = handwrite_marker_body(line) else {
            continue;
        };
        let gap = extract_attr(body, "gap").unwrap_or_default();
        let tracker = extract_attr(body, "tracker").unwrap_or_default();
        out.push((idx, gap, tracker));
    }
    out
}

/// Non-marker, non-spec-header lines — used to prove a HANDWRITE→CODEGEN
/// flip is byte-equivalence-safe: everything outside the marker delimiter
/// lines (the actual HANDWRITE payload) must be byte-identical before and
/// after the flip.
fn non_marker_lines(content: &str) -> Vec<&str> {
    content
        .lines()
        .filter(|line| {
            let body = marker_comment_body(line);
            let is_marker = body.is_some_and(|b| {
                is_handwrite_marker_line_body(b) || b == "CODEGEN-BEGIN" || b == "CODEGEN-END"
            });
            let is_injected_spec = body.is_some_and(|b| b.starts_with("SPEC-MANAGED:"));
            !(is_marker || is_injected_spec)
        })
        .collect()
}

/// Core promotion logic shared by `aw td promote` and (in soft mode)
/// `aw standardize managed run`'s `PromoteHandwrite` action: flip every
/// HANDWRITE block in `rel` to CODEGEN, but only when every block already
/// carries a complete `gap` + durable `tracker` (closure-ready — the
/// `issue_marker_gap` action attaches those first) and the flip is proven
/// byte-equivalence-safe (every line outside the marker delimiters is
/// unchanged). Otherwise reports not-ready without touching the file.
fn try_promote_handwrite_marker_to_codegen(
    project_root: &Path,
    rel: &str,
    fallback_spec_ref: &str,
) -> Result<PromoteAttempt> {
    let abs = project_root.join(rel);
    let content =
        fs::read_to_string(&abs).with_context(|| format!("failed to read {}", abs.display()))?;
    let blocks = handwrite_block_open_attrs(&content);
    if blocks.is_empty() {
        return Ok(PromoteAttempt::NotReady(format!(
            "`{rel}` has no HANDWRITE marker to promote"
        )));
    }
    for (line_idx, gap, tracker) in &blocks {
        if gap.trim().is_empty() {
            return Ok(PromoteAttempt::NotReady(format!(
                "`{rel}` line {}: HANDWRITE marker is missing a `gap` attribute — run `aw standardize managed run` (issue_marker_gap) before promoting",
                line_idx + 1
            )));
        }
        if is_missing_tracker(tracker) {
            return Ok(PromoteAttempt::NotReady(format!(
                "`{rel}` line {}: HANDWRITE marker has no durable tracker — run `aw standardize managed run` (issue_marker_gap) before promoting",
                line_idx + 1
            )));
        }
    }

    let promoted = promote_handwrite_blocks_to_codegen(&abs, &content, fallback_spec_ref);
    if promoted == content {
        return Ok(PromoteAttempt::Promoted(PromoteOutcome {
            changed_paths: Vec::new(),
            message: "no HANDWRITE block required promotion".to_string(),
        }));
    }
    if non_marker_lines(&content) != non_marker_lines(&promoted) {
        return Ok(PromoteAttempt::NotReady(format!(
            "`{rel}` promotion is not byte-equivalence-safe — the payload outside HANDWRITE markers would change; refusing to flip"
        )));
    }

    fs::write(&abs, &promoted)?;
    Ok(PromoteAttempt::Promoted(PromoteOutcome {
        changed_paths: vec![PathBuf::from(rel)],
        message: format!("promoted {} HANDWRITE block(s) to CODEGEN", blocks.len()),
    }))
}

/// `aw td promote` entry point: the strict form of
/// `try_promote_handwrite_marker_to_codegen` — a not-ready result is a hard
/// error, since the caller explicitly asked to promote this one target.
pub(crate) fn promote_handwrite_marker_to_codegen(
    project_root: &Path,
    rel: &str,
    fallback_spec_ref: &str,
) -> Result<PromoteOutcome> {
    match try_promote_handwrite_marker_to_codegen(project_root, rel, fallback_spec_ref)? {
        PromoteAttempt::Promoted(outcome) => Ok(outcome),
        PromoteAttempt::NotReady(reason) => bail!(reason),
    }
}

fn comment_prefix(path: &Path) -> &'static str {
    if is_dockerfile_path(path) || is_dockerignore_path(path) || is_kustomize_yaml_path(path) {
        return "# ";
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("py" | "yaml" | "yml") => "# ",
        _ => "// ",
    }
}

// Rule B (epic #914): batch/whole-repo standardization runs under one
// bootstrap WI root, not as anonymous ticks. `wi` carries that root's id
// (from `StandardizeRunArgs::wi`) so every per-action commit is
// attributable back to it; `None` keeps prior (unattributed) behavior.

fn rel_display(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// `pub(crate)` (not just file-private) so `aw td promote` (td.rs) can derive
/// the same stable slug the standardize path uses for its own action ids —
/// one slug scheme, not a duplicate.
pub(crate) fn slug_for_path(path: &str) -> String {
    let mut out = String::new();
    for ch in path.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._-".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// Initialise a bare-minimum git repo in `root` so `commit_action`
    /// (issue #919, Rule B) and `git::commit_scoped_paths` have something
    /// real to commit into. Mirrors `merge_target.rs`'s test-local
    /// `init_repo` helper.
    fn init_git_repo(root: &Path) {
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["init", "--initial-branch=main"])
            .output()
            .unwrap();
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .unwrap();
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["config", "user.name", "Test"])
            .output()
            .unwrap();
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["commit", "--allow-empty", "-m", "init"])
            .output()
            .unwrap();
    }

    fn git_log_body(root: &Path) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["log", "-1", "--format=%B"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    #[cfg(unix)]
    fn make_executable(root: &Path, rel: &str) {
        use std::os::unix::fs::PermissionsExt;
        let path = root.join(rel);
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }

    #[cfg(not(unix))]
    fn make_executable(_root: &Path, _rel: &str) {}

    fn write_traceability_config(root: &Path, workspace_scope: &str) {
        write(
            root,
            "aw.toml",
            &format!(
                r#"
[[projects]]
name = "demo"
path = "."
td_path = "tech-design/demo"
cap_path = "README.md"
label = "app:demo"

[[projects.workspaces]]
name = "demo"
paths = ["{workspace_scope}"]
target = "python"
test_cmd = "true"
"#
            ),
        );
    }

    fn write_traceability_readme(root: &Path) {
        write(
            root,
            "README.md",
            r#"# demo

## Demo Capability

| Field | Value |
|---|---|
| ID | demo-capability |
| Root WI | - |
| Status | verified |
| Promise | Provide demo behavior. |
| Required Verification | smoke |
| Gate Inventory | - |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo closure | epic | - | implemented | verified | smoke | true |
"#,
        );
    }

    fn valid_traceability_td() -> &'static str {
        r#"---
id: demo-td
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo TD

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: demo-logic
nodes: []
edges: []
---
flowchart TD
  A --> B
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: src/app.py
    action: modify
    section: logic
    impl_mode: hand-written
```
"#
    }

    fn valid_traceability_td_with_command_ref(command: &str) -> String {
        valid_traceability_td().replacen(
            "---\n\n# Demo TD",
            &format!("command_refs:\n  - command: {command}\n---\n\n# Demo TD"),
            1,
        )
    }

    fn source_referencing_demo_td() -> &'static str {
        "# SPEC-MANAGED: tech-design/demo/app.md#changes\n# CODEGEN-BEGIN\ndef handle():\n    return 1\n# CODEGEN-END\n"
    }

    fn traceability_coverage_for(root: &Path) -> TraceabilityCoverage {
        let inventory = build_inventory(root, &[], Some("demo"), false).unwrap();
        build_traceability_coverage_with_command_inventory(
            root,
            "demo",
            &inventory,
            &BTreeMap::new(),
        )
        .unwrap()
    }

    fn traceability_coverage_for_commands(
        root: &Path,
        command_paths: &[(&str, bool)],
    ) -> TraceabilityCoverage {
        let inventory = build_inventory(root, &[], Some("demo"), false).unwrap();
        let command_inventory = command_paths
            .iter()
            .map(|(path, hidden)| {
                (
                    (*path).to_string(),
                    CommandInventoryEntry {
                        path: (*path).to_string(),
                        hidden: *hidden,
                        alias_of: None,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        build_traceability_coverage_with_command_inventory(
            root,
            "demo",
            &inventory,
            &command_inventory,
        )
        .unwrap()
    }

    fn empty_semantic(scope: Vec<String>) -> SemanticCoverage {
        SemanticCoverage {
            scope,
            total_files: 0,
            source_units: 0,
            source_symbols: 0,
            claim_files: 0,
            semantic_files: 0,
            semantically_covered_files: 0,
            percent: 100.0,
            source_ir: Vec::new(),
            source_evidence_graph: None,
            frontend_ecosystem: None,
            coverage_map: Vec::new(),
            generator_primitive_gaps: Vec::new(),
            uncovered_files: Vec::new(),
            next_gap: None,
            blocked_gap_count: 0,
            human_decision_required_count: 0,
        }
    }

    #[test]
    fn scanner_covers_supported_languages_and_exclusions() {
        let tmp = TempDir::new().unwrap();
        for rel in [
            "src/lib.rs",
            "src/app.py",
            "src/index.js",
            "src/view.jsx",
            "src/mod.mjs",
            "src/cjs.cjs",
            "src/main.ts",
            "src/page.tsx",
            "src/server.go",
        ] {
            write(tmp.path(), rel, "fn main() {}\n");
        }
        write(tmp.path(), "node_modules/pkg/index.ts", "export {}\n");
        write(tmp.path(), "src/assets/highlight.min.js", "minified();\n");
        let files = collect_source_files(
            tmp.path(),
            &["src/**".to_string(), "node_modules/**".to_string()],
        )
        .unwrap();
        assert_eq!(files.len(), 9);
        assert!(!files.iter().any(|f| f.rel.ends_with(".min.js")));
        let langs: BTreeSet<_> = files.iter().map(|f| f.language.as_str()).collect();
        assert!(langs.contains("rust"));
        assert!(langs.contains("python"));
        assert!(langs.contains("javascript"));
        assert!(langs.contains("typescript"));
        assert!(langs.contains("go"));
    }

    #[test]
    fn scanner_covers_operations_artifacts_without_all_yaml() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "backend/Dockerfile", "FROM python:3.12\n");
        write(tmp.path(), "backend/.dockerignore", ".venv\n");
        write(
            tmp.path(),
            "backend/kustomize/bases/api/kustomization.yaml",
            "resources:\n  - deployment.yaml\n",
        );
        write(
            tmp.path(),
            "backend/kustomize/bases/api/deployment.yaml",
            "apiVersion: apps/v1\nkind: Deployment\n",
        );
        write(
            tmp.path(),
            "backend/config/settings.yaml",
            "not: kustomize\n",
        );

        let files = collect_source_files(tmp.path(), &["backend/**".to_string()]).unwrap();
        let paths: BTreeSet<_> = files.iter().map(|file| file.rel.as_str()).collect();

        assert!(paths.contains("backend/Dockerfile"));
        assert!(paths.contains("backend/.dockerignore"));
        assert!(paths.contains("backend/kustomize/bases/api/kustomization.yaml"));
        assert!(paths.contains("backend/kustomize/bases/api/deployment.yaml"));
        assert!(!paths.contains("backend/config/settings.yaml"));

        let langs: BTreeSet<_> = files.iter().map(|f| f.language.as_str()).collect();
        assert!(langs.contains("dockerfile"));
        assert!(langs.contains("dockerignore"));
        assert!(langs.contains("kustomize"));
    }

    #[test]
    fn managed_inventory_includes_project_root_artifacts() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "tool"
path = "projects/tool"
label = "app:tool"

[[projects.workspaces]]
name = "tool"
paths = ["projects/tool/**"]
target = "rust"
"#,
        );
        write(
            tmp.path(),
            "projects/tool/Cargo.toml",
            "[package]\nname = \"tool\"\n\n[[bin]]\nname = \"tool\"\npath = \"src/main.rs\"\n",
        );
        write(
            tmp.path(),
            "projects/tool/src/main.rs",
            "// <HANDWRITE gap=\"g\" tracker=\"#4158\" reason=\"fixture\">\nfn main() {}\n// </HANDWRITE>\n",
        );
        let llms = render_project_llms_txt(tmp.path(), "tool").unwrap();
        write(tmp.path(), "projects/tool/llms.txt", &llms);
        write(
            tmp.path(),
            "projects/tool/build.sh",
            "# <HANDWRITE gap=\"project-root-build\" tracker=\"#4158\" reason=\"fixture\">\ncase \"${1:-}\" in debug) cargo build -p tool ;; release) cargo build --release -p tool ;; esac\n# </HANDWRITE>\n",
        );
        write(
            tmp.path(),
            "projects/tool/install.sh",
            "# <HANDWRITE gap=\"project-root-install\" tracker=\"#4158\" reason=\"fixture\">\ninstall -m 755 target/release/tool \"$HOME/.cargo/bin/tool\"\n# </HANDWRITE>\n",
        );
        make_executable(tmp.path(), "projects/tool/build.sh");
        make_executable(tmp.path(), "projects/tool/install.sh");

        let inventory = build_inventory(tmp.path(), &[], Some("tool"), false).unwrap();
        let rels = inventory
            .files
            .iter()
            .map(|file| file.rel.as_str())
            .collect::<BTreeSet<_>>();

        assert!(rels.contains("projects/tool/llms.txt"));
        assert!(rels.contains("projects/tool/build.sh"));
        assert!(rels.contains("projects/tool/install.sh"));
        assert!(inventory.coverage.uncovered_files.is_empty());
        assert_eq!(inventory.coverage.percent, 100.0);
        assert!(project_root_artifact_blockers_at(tmp.path(), "tool")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn project_root_llms_generator_is_td_first() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "tool"
path = "projects/tool"
td_path = "projects/tool/tech-design"
cap_path = "projects/tool/README.md"
label = "app:tool"

[[projects.workspaces]]
name = "tool"
paths = ["projects/tool/**"]
target = "rust"
test_cmd = "cargo test -p tool"
"#,
        );
        write(
            tmp.path(),
            "projects/tool/Cargo.toml",
            "[package]\nname = \"tool\"\n\n[[bin]]\nname = \"tool\"\npath = \"src/main.rs\"\n",
        );

        let llms = render_project_llms_txt(tmp.path(), "tool").unwrap();

        let td_pos = llms.find("## Tech Design").unwrap();
        let cap_pos = llms.find("## Capability Map").unwrap();
        assert!(td_pos < cap_pos);
        assert!(llms.contains("<!-- CODEGEN-BEGIN -->"));
        assert!(llms.contains("[Tech Design](tech-design)"));
        assert!(llms.contains("`aw td check projects/tool/tech-design`"));
        assert!(llms.contains("`aw capability next --project tool`"));
        assert!(llms.contains("`aw health --project tool`"));
        assert!(llms.contains("`aw health --project tool blockers`"));
        assert!(!llms.contains("`aw run"));
        assert!(!llms.contains("`aw standardize managed"));
        assert!(llms.contains("`./build.sh debug`"));
        assert!(llms.contains("`./build.sh release`"));
        assert!(llms.contains("`cargo test -p tool`"));
        assert!(!llms.contains("src/"));
        assert!(!llms.contains("managed_percent"));
        assert!(!llms.contains("open WI"));
        assert!(!llms.contains("CB"));
        assert!(!llms.contains("HANDWRITE"));

        let llms_file = SourceFile {
            rel: "projects/tool/llms.txt".into(),
            abs: tmp.path().join("projects/tool/llms.txt"),
            language: "llms".into(),
            markers: FileMarkers {
                codegen: true,
                handwrite: false,
            },
            handwrite_gaps: vec![],
        };
        assert!(codegen_replay_supported(&llms_file));
    }

    #[test]
    fn codegen_replay_supports_dockerfile_variants() {
        for rel in [
            "projects/lumen/Dockerfile",
            "projects/lumen/Dockerfile.release",
            "projects/lumen/Dockerfile.bench",
            "projects/lumen/service.dockerfile",
        ] {
            let file = SourceFile {
                rel: rel.into(),
                abs: PathBuf::from(rel),
                language: "dockerfile".into(),
                markers: FileMarkers {
                    codegen: true,
                    handwrite: false,
                },
                handwrite_gaps: vec![],
            };
            assert!(
                codegen_replay_supported(&file),
                "codegen replay should accept {rel}"
            );
        }
    }

    #[test]
    fn project_root_llms_blocker_rejects_handwrite_and_stale_content() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "tool"
path = "projects/tool"
label = "app:tool"

[[projects.workspaces]]
name = "tool"
paths = ["projects/tool/**"]
target = "python"
test_cmd = "true"
"#,
        );
        write(
            tmp.path(),
            "projects/tool/llms.txt",
            "<!-- <HANDWRITE gap=\"project-root-llms\" tracker=\"#4158\" reason=\"fixture\"> -->\n# tool\n<!-- </HANDWRITE> -->\n",
        );

        let blockers = project_root_artifact_blockers_at(tmp.path(), "tool").unwrap();
        assert!(blockers
            .iter()
            .any(|blocker| blocker.contains("must be generated")));

        let mut generated = render_project_llms_txt(tmp.path(), "tool").unwrap();
        generated.push_str("\n");
        write(tmp.path(), "projects/tool/llms.txt", &generated);
        let blockers = project_root_artifact_blockers_at(tmp.path(), "tool").unwrap();
        assert!(blockers.iter().any(|blocker| blocker.contains("is stale")));

        let generated = render_project_llms_txt(tmp.path(), "tool").unwrap();
        write(tmp.path(), "projects/tool/llms.txt", &generated);
        assert!(project_root_artifact_blockers_at(tmp.path(), "tool")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn project_root_llms_spec_ref_uses_existing_project_local_semantic_td() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "tool"
path = "projects/tool"
td_path = "projects/tool/tech-design"
label = "app:tool"

[[projects.workspaces]]
name = "tool"
paths = ["projects/tool/**"]
target = "python"
test_cmd = "true"
"#,
        );
        write(
            tmp.path(),
            "projects/tool/tech-design/semantic/tool-projects-tool.md",
            "---\nid: semantic-tool-projects-tool\ncapability_refs:\n  - id: tool\n    role: primary\n---\n",
        );

        let generated = render_project_llms_txt(tmp.path(), "tool").unwrap();
        assert!(generated.contains(
            "<!-- SPEC-MANAGED: projects/tool/tech-design/semantic/tool-projects-tool.md#schema -->"
        ));
    }

    #[test]
    fn rust_binary_project_missing_root_artifacts_blocks_managed_coverage() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "tool"
path = "projects/tool"
label = "app:tool"

[[projects.workspaces]]
name = "tool"
paths = ["projects/tool/**"]
target = "rust"
"#,
        );
        write(
            tmp.path(),
            "projects/tool/Cargo.toml",
            "[package]\nname = \"tool\"\n\n[[bin]]\nname = \"tool\"\npath = \"src/main.rs\"\n",
        );
        write(
            tmp.path(),
            "projects/tool/src/main.rs",
            "// <HANDWRITE gap=\"g\" tracker=\"#4158\" reason=\"fixture\">\nfn main() {}\n// </HANDWRITE>\n",
        );

        let inventory = build_inventory(tmp.path(), &[], Some("tool"), false).unwrap();

        assert_eq!(inventory.coverage.total_files, 4);
        assert_eq!(inventory.coverage.managed_files, 1);
        assert_eq!(
            inventory.coverage.uncovered_files,
            vec![
                "projects/tool/build.sh".to_string(),
                "projects/tool/install.sh".to_string(),
                "projects/tool/llms.txt".to_string(),
            ]
        );
        let blockers = project_root_artifact_blockers_at(tmp.path(), "tool").unwrap();
        assert!(blockers
            .iter()
            .any(|blocker| blocker.contains("projects/tool/build.sh")));
        assert!(blockers
            .iter()
            .any(|blocker| blocker.contains("projects/tool/install.sh")));
        assert!(blockers
            .iter()
            .any(|blocker| blocker.contains("projects/tool/llms.txt")));
    }

    #[test]
    fn deployment_facet_detection_classifies_kustomize_and_ml_constraints() {
        let mut facets = BTreeSet::new();
        let mut unsupported = BTreeSet::new();
        detect_deployment_facets(
            "backend/kustomize/jobs/migration.yaml",
            "apiVersion: batch/v1\nkind: CronJob\nmetadata:\n  name: alembic-migration\n",
            &mut facets,
            &mut unsupported,
        );
        detect_deployment_facets(
            "ml/kustomize/bases/ml/deployment.yaml",
            "apiVersion: apps/v1\nkind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - resources:\n            limits:\n              nvidia.com/gpu: 1\n",
            &mut facets,
            &mut unsupported,
        );
        detect_deployment_facets(
            "backend/kustomize/components/apis/name-prefix.yaml",
            "apiVersion: builtin\nkind: PrefixTransformer\nmetadata:\n  name: prefix\n",
            &mut facets,
            &mut unsupported,
        );
        detect_deployment_facets(
            "backend/kustomize/components/observability/kustomization.yaml",
            "apiVersion: kustomize.config.k8s.io/v1alpha1\nkind: Component\nmetadata:\n  name: observability\n",
            &mut facets,
            &mut unsupported,
        );

        let facet_names: BTreeSet<_> = facets
            .iter()
            .map(|finding| finding.facet.as_str())
            .collect();
        assert!(facet_names.contains("migration_job"));
        assert!(facet_names.contains("deployment_unit"));
        assert!(facet_names.contains("gpu_scheduling"));
        assert!(facet_names.contains("kustomize_name_transformer"));
        assert!(facet_names.contains("kustomize_component"));
        assert!(unsupported.is_empty());
    }

    #[test]
    fn resolve_scopes_replaces_stale_config_scope_with_discovered_project() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "agentic-workflow"
path = "apps/agentic-workflow"

[[projects.workspaces]]
paths = ["apps/agentic-workflow/**"]
target = "rust"
"#,
        );
        write(
            tmp.path(),
            "apps/agentic-workflow/Cargo.toml",
            "[package]\nname = \"agentic-workflow\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        );
        write(
            tmp.path(),
            "apps/agentic-workflow/src/lib.rs",
            "pub fn ok() {}\n",
        );

        let scopes = resolve_scopes(tmp.path(), &[], Some("agentic-workflow"), false).unwrap();

        assert!(scopes.contains(&"apps/agentic-workflow/**".to_string()));
        assert!(!scopes.contains(&"crates/agentic-workflow/**".to_string()));
    }

    #[test]
    fn resolve_scopes_requires_project_for_multi_project_config() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "agentic-workflow"

[[projects.workspaces]]
paths = ["apps/agentic-workflow/**"]

[[projects]]
name = "jet"

[[projects.workspaces]]
paths = ["apps/jet/**"]
"#,
        );
        write(
            tmp.path(),
            "apps/agentic-workflow/src/lib.rs",
            "pub fn agentic_workflow() {}\n",
        );
        write(tmp.path(), "apps/jet/src/lib.rs", "pub fn jet() {}\n");

        let err = resolve_scopes(tmp.path(), &[], None, false).unwrap_err();

        assert!(err.to_string().contains("requires --project <project>"));
        assert!(err.to_string().contains("agentic-workflow"));
        assert!(err.to_string().contains("jet"));
    }

    #[test]
    fn resolve_scopes_selects_positional_project_from_config() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "agentic-workflow"

[[projects.workspaces]]
paths = ["apps/agentic-workflow/**"]

[[projects]]
name = "jet"

[[projects.workspaces]]
paths = ["apps/jet/**"]
"#,
        );
        write(
            tmp.path(),
            "apps/agentic-workflow/src/lib.rs",
            "pub fn agentic_workflow() {}\n",
        );
        write(tmp.path(), "apps/jet/src/lib.rs", "pub fn jet() {}\n");

        let scopes = resolve_scopes(tmp.path(), &[], Some("agentic-workflow"), false).unwrap();

        assert_eq!(scopes, vec!["apps/agentic-workflow/**"]);
    }

    #[test]
    fn resolve_scopes_accepts_project_alias_from_config() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "agentic-workflow"
aliases = ["aw"]

[[projects.workspaces]]
paths = ["apps/agentic-workflow/**"]
"#,
        );
        write(
            tmp.path(),
            "apps/agentic-workflow/src/lib.rs",
            "pub fn agentic_workflow() {}\n",
        );

        let scopes = resolve_scopes(tmp.path(), &[], Some("aw"), false).unwrap();

        assert_eq!(scopes, vec!["apps/agentic-workflow/**"]);
        assert_eq!(
            resolve_standardize_project_name(tmp.path(), "aw").unwrap(),
            "agentic-workflow"
        );
    }

    #[test]
    fn spec_roots_for_scopes_uses_project_tech_design_without_td_path() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "cap"
path = "apps/cap"

[[projects.workspaces]]
paths = ["apps/cap/**"]
"#,
        );
        write(
            tmp.path(),
            "apps/cap/tech-design/semantic/cap-src.md",
            "# Semantic TD\n",
        );

        let roots = spec_roots_for_scopes(tmp.path(), &["apps/cap/**".into()]).unwrap();

        assert_eq!(roots, vec![tmp.path().join("apps/cap/tech-design")]);
    }

    #[test]
    fn semantic_spec_rel_with_config_defaults_to_project_tech_design() {
        let configured = vec![ConfiguredScope {
            project_name: Some("cap".into()),
            aliases: Vec::new(),
            project_path: Some("apps/cap".into()),
            scope: "apps/cap/**".into(),
            td_path: None,
            cap_path: None,
        }];

        let spec_rel = semantic_spec_rel_with_config("apps/cap/src/cli.rs", &configured);

        assert_eq!(spec_rel, "apps/cap/tech-design/semantic/cap-src.md");
    }

    #[test]
    fn marker_parser_recognizes_comment_styles() {
        let content =
            "// CODEGEN-BEGIN\n<!-- <HANDWRITE gap=\"g\" tracker=\"t\" reason=\"r\"> -->\n";
        let markers = detect_markers(content);
        assert!(markers.codegen);
        assert!(markers.handwrite);
        assert!(detect_handwrite_gaps(content).is_empty());

        let code = r#"let marker = "HANDWRITE-BEGIN";"#;
        assert!(!detect_markers(code).handwrite);
        assert!(detect_handwrite_gaps(code).is_empty());

        let raw_fixture =
            "const FIXTURE: &str = r#\"\n// CODEGEN-BEGIN\n// HANDWRITE-BEGIN reason: fixture\n\"#;\n";
        let raw_markers = detect_markers(raw_fixture);
        assert!(!raw_markers.codegen);
        assert!(!raw_markers.handwrite);

        let gap = "// HANDWRITE-BEGIN reason: legacy\n";
        let gaps = detect_handwrite_gaps(gap);
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].message.contains("missing tracker"));
        assert!(gaps[0].needs_promotion);

        let json_marker = r##"
{
  "aw_ownership": "HANDWRITE-BEGIN gap=json-manifest tracker=#4041 reason: valid JSON marker"
}
"##;
        let json_markers = detect_markers(json_marker);
        assert!(json_markers.handwrite);
        assert!(detect_handwrite_gaps(json_marker).is_empty());

        let toml_marker =
            "aw_ownership = \"CODEGEN-BEGIN tracker=\\\"#4041\\\" reason=\\\"valid TOML marker\\\"\"\n";
        let toml_markers = detect_markers(toml_marker);
        assert!(toml_markers.codegen);
    }

    #[test]
    fn ordinary_comment_is_not_a_marker() {
        let markers = detect_markers("# generated by an external tool\n");

        assert!(!markers.managed());
    }

    #[test]
    fn force_regen_replay_failure_path_is_extracted() {
        assert_eq!(
            extract_force_regen_replay_failure_path(
                "apps/jet/src/dev_server/proxy.rs: differs after TD replay"
            ),
            Some("apps/jet/src/dev_server/proxy.rs".to_string())
        );
        assert_eq!(
            extract_force_regen_replay_failure_path("public API summary failed"),
            None
        );
    }

    #[test]
    fn regenerability_counts_handwrite_vendor_as_gap() {
        let inv = Inventory {
            coverage: StandardizationCoverage {
                scope: vec!["src/**".into()],
                total_files: 2,
                managed_files: 2,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 1,
                    handwrite: 1,
                },
                uncovered_files: vec![],
            },
            files: vec![
                SourceFile {
                    rel: "src/generated.rs".into(),
                    abs: PathBuf::from("src/generated.rs"),
                    language: "rust".into(),
                    markers: FileMarkers {
                        codegen: true,
                        ..Default::default()
                    },
                    handwrite_gaps: vec![],
                },
                SourceFile {
                    rel: "src/vendor.min.js".into(),
                    abs: PathBuf::from("src/vendor.min.js"),
                    language: "javascript".into(),
                    markers: FileMarkers {
                        handwrite: true,
                        ..Default::default()
                    },
                    handwrite_gaps: vec![],
                },
            ],
            rust_findings: vec![],
            spec_violation: None,
        };

        let semantic = empty_semantic(inv.coverage.scope.clone());
        let coverage = build_regenerability_coverage(Path::new("."), &inv, &semantic).unwrap();
        assert_eq!(coverage.total_files, 2);
        assert_eq!(coverage.eligible_files, 2);
        assert_eq!(coverage.codegen_files, 1);
        assert_eq!(coverage.handwrite_files, 1);
        assert_eq!(coverage.gap_files, vec!["src/vendor.min.js"]);
        assert_eq!(coverage.percent, 50.0);

        let action = choose_regenerable_action(Path::new("."), &inv, &semantic);
        assert_eq!(action.kind, StandardizeActionKind::PromoteHandwrite);
    }

    #[test]
    fn regenerability_rejects_codegen_marker_backed_by_hand_written_td_change() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "src/lib.rs",
            "// SPEC-MANAGED: projects/demo/tech-design/semantic/demo-src.md#schema\n// CODEGEN-BEGIN\npub fn demo() {}\n// CODEGEN-END\n",
        );
        write(
            tmp.path(),
            "projects/demo/tech-design/semantic/demo-src.md",
            r#"---
id: demo-src
---

# Demo Source

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: src/lib.rs
    action: modify
    section: schema
    impl_mode: hand-written
```
"#,
        );
        let inv = Inventory {
            coverage: StandardizationCoverage {
                scope: vec!["src/**".into()],
                total_files: 1,
                managed_files: 1,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 1,
                    handwrite: 0,
                },
                uncovered_files: vec![],
            },
            files: vec![SourceFile {
                rel: "src/lib.rs".into(),
                abs: tmp.path().join("src/lib.rs"),
                language: "rust".into(),
                markers: FileMarkers {
                    codegen: true,
                    ..Default::default()
                },
                handwrite_gaps: vec![],
            }],
            rust_findings: vec![],
            spec_violation: None,
        };

        let semantic = empty_semantic(inv.coverage.scope.clone());
        let coverage =
            build_regenerability_coverage_with_options(tmp.path(), &inv, &semantic, None, false)
                .unwrap();

        assert_eq!(coverage.codegen_files, 0);
        assert_eq!(
            coverage.non_replayable_codegen_files,
            vec!["src/lib.rs".to_string()]
        );
        assert_eq!(coverage.gap_files, vec!["src/lib.rs"]);
        assert_eq!(coverage.percent, 0.0);
    }

    #[test]
    fn regenerability_rejects_source_template_snapshot_codegen() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "src/lib.rs",
            "// SPEC-MANAGED: projects/demo/tech-design/semantic/demo-src.md#source\n// CODEGEN-BEGIN\npub fn demo() {}\n// CODEGEN-END\n",
        );
        write(
            tmp.path(),
            "projects/demo/tech-design/semantic/demo-src.md",
            r#"---
id: demo-src
---

# Demo Source

## Source
<!-- type: source lang: rust -->

```rust
pub fn demo() {}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: src/lib.rs
    action: modify
    section: source
    impl_mode: codegen
```
"#,
        );
        let inv = Inventory {
            coverage: StandardizationCoverage {
                scope: vec!["src/**".into()],
                total_files: 1,
                managed_files: 1,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 1,
                    handwrite: 0,
                },
                uncovered_files: vec![],
            },
            files: vec![SourceFile {
                rel: "src/lib.rs".into(),
                abs: tmp.path().join("src/lib.rs"),
                language: "rust".into(),
                markers: FileMarkers {
                    codegen: true,
                    ..Default::default()
                },
                handwrite_gaps: vec![],
            }],
            rust_findings: vec![],
            spec_violation: None,
        };

        let semantic = empty_semantic(inv.coverage.scope.clone());
        let coverage =
            build_regenerability_coverage_with_options(tmp.path(), &inv, &semantic, None, false)
                .unwrap();

        assert_eq!(coverage.codegen_files, 0);
        assert_eq!(
            coverage.snapshot_codegen_files,
            vec!["src/lib.rs".to_string()]
        );
        assert_eq!(coverage.gap_files, vec!["src/lib.rs"]);
        assert_eq!(coverage.percent, 0.0);
    }

    #[test]
    fn regenerability_counts_rust_source_unit_as_lossless_regenerable_codegen() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "src/lib.rs",
            "// SPEC-MANAGED: projects/demo/tech-design/semantic/demo-src.md#source\n// CODEGEN-BEGIN\npub fn demo() {}\n// CODEGEN-END\n",
        );
        write(
            tmp.path(),
            "projects/demo/tech-design/semantic/demo-src.md",
            r#"---
id: demo-src
---

# Demo Source

## Source
<!-- type: rust-source-unit lang: rust -->

```rust
pub fn demo() {}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: src/lib.rs
    action: modify
    section: source
    impl_mode: codegen
```
"#,
        );
        let inv = Inventory {
            coverage: StandardizationCoverage {
                scope: vec!["src/**".into()],
                total_files: 1,
                managed_files: 1,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 1,
                    handwrite: 0,
                },
                uncovered_files: vec![],
            },
            files: vec![SourceFile {
                rel: "src/lib.rs".into(),
                abs: tmp.path().join("src/lib.rs"),
                language: "rust".into(),
                markers: FileMarkers {
                    codegen: true,
                    ..Default::default()
                },
                handwrite_gaps: vec![],
            }],
            rust_findings: vec![],
            spec_violation: None,
        };

        let semantic = empty_semantic(inv.coverage.scope.clone());
        let coverage =
            build_regenerability_coverage_with_options(tmp.path(), &inv, &semantic, None, false)
                .unwrap();

        assert_eq!(coverage.codegen_files, 1);
        assert!(coverage.snapshot_codegen_files.is_empty());
        assert!(coverage.gap_files.is_empty());
        assert_eq!(coverage.percent, 100.0);
    }

    #[test]
    fn regenerability_counts_text_source_unit_as_td_owned_codegen() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "build.sh",
            "# SPEC-MANAGED: projects/demo/tech-design/semantic/demo-build.md#text-source-unit\n# CODEGEN-BEGIN\n#!/usr/bin/env bash\nset -euo pipefail\n# CODEGEN-END\n",
        );
        write(
            tmp.path(),
            "projects/demo/tech-design/semantic/demo-build.md",
            r#"---
id: demo-build
---

# Demo Build

## Source
<!-- type: text-source-unit lang: bash -->

```bash
#!/usr/bin/env bash
set -euo pipefail
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: build.sh
    action: modify
    section: text-source-unit
    impl_mode: codegen
```
"#,
        );
        let inv = Inventory {
            coverage: StandardizationCoverage {
                scope: vec!["**".into()],
                total_files: 1,
                managed_files: 1,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 1,
                    handwrite: 0,
                },
                uncovered_files: vec![],
            },
            files: vec![SourceFile {
                rel: "build.sh".into(),
                abs: tmp.path().join("build.sh"),
                language: "shell".into(),
                markers: FileMarkers {
                    codegen: true,
                    ..Default::default()
                },
                handwrite_gaps: vec![],
            }],
            rust_findings: vec![],
            spec_violation: None,
        };

        let semantic = empty_semantic(inv.coverage.scope.clone());
        let coverage =
            build_regenerability_coverage_with_options(tmp.path(), &inv, &semantic, None, false)
                .unwrap();

        assert_eq!(coverage.codegen_files, 1);
        assert!(coverage.snapshot_codegen_files.is_empty());
        assert!(coverage.gap_files.is_empty());
        assert_eq!(coverage.percent, 100.0);
    }

    #[test]
    fn regenerability_counts_frontend_codegen_as_replay_supported() {
        let inv = Inventory {
            coverage: StandardizationCoverage {
                scope: vec!["examples/fixture_platform/frontend/**".into()],
                total_files: 1,
                managed_files: 1,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 1,
                    handwrite: 0,
                },
                uncovered_files: vec![],
            },
            files: vec![SourceFile {
                rel: "examples/fixture_platform/frontend/apps/demo/src/app.tsx".into(),
                abs: PathBuf::from("examples/fixture_platform/frontend/apps/demo/src/app.tsx"),
                language: "typescript".into(),
                markers: FileMarkers {
                    codegen: true,
                    ..Default::default()
                },
                handwrite_gaps: vec![],
            }],
            rust_findings: vec![],
            spec_violation: None,
        };

        let semantic = empty_semantic(inv.coverage.scope.clone());
        let coverage = build_regenerability_coverage(Path::new("."), &inv, &semantic).unwrap();
        assert_eq!(coverage.codegen_files, 1);
        assert!(coverage.unsupported_codegen_files.is_empty());
        assert!(coverage.gap_files.is_empty());
        assert_eq!(coverage.percent, 100.0);

        let action = choose_regenerable_action(Path::new("."), &inv, &semantic);
        assert_eq!(action.kind, StandardizeActionKind::None);
        assert!(!action.requires_hitl);
    }

    #[test]
    fn regenerable_next_routes_missing_semantics_to_semantic_layer() {
        let inv = Inventory {
            coverage: StandardizationCoverage {
                scope: vec!["src/**".into()],
                total_files: 1,
                managed_files: 1,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 0,
                    handwrite: 1,
                },
                uncovered_files: vec![],
            },
            files: vec![SourceFile {
                rel: "src/app.py".into(),
                abs: PathBuf::from("src/app.py"),
                language: "python".into(),
                markers: FileMarkers {
                    handwrite: true,
                    ..Default::default()
                },
                handwrite_gaps: vec![],
            }],
            rust_findings: vec![],
            spec_violation: None,
        };
        let mut semantic = empty_semantic(inv.coverage.scope.clone());
        semantic.percent = 0.0;
        semantic
            .generator_primitive_gaps
            .push(GeneratorPrimitiveGap {
                target: "src/app.py".into(),
                primitive: "semantic_td_missing".into(),
                reason: "missing semantic TD".into(),
                human_decision_required: false,
            });

        let coverage = build_regenerability_coverage(Path::new("."), &inv, &semantic).unwrap();
        assert_eq!(coverage.insufficient_td_section_gaps, 1);
        assert_eq!(coverage.missing_generator_primitive_gaps, 0);

        let action = choose_regenerable_action(Path::new("."), &inv, &semantic);
        assert_eq!(action.kind, StandardizeActionKind::SemanticGap);
        assert!(action.command.contains("aw standardize semantic run"));
    }

    #[test]
    fn source_ir_detects_fixture_platform_python_primitives() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "src/features/workspace/data_table/admin_api_endpoints.py",
            "# <HANDWRITE gap=\"g\" tracker=\"t\" reason=\"r\">\nfrom fastapi import APIRouter\napi_router = APIRouter()\n@api_router.get(\"\")\nasync def list_rows():\n    return []\n# </HANDWRITE>\n",
        );
        write(
            tmp.path(),
            "src/features/workspace/bases/api_routes.py",
            "# <HANDWRITE gap=\"g\" tracker=\"t\" reason=\"r\">\nfrom src import bases\nclass MyRoute(bases.BaseAPIRoute):\n    pass\n# </HANDWRITE>\n",
        );

        let inventory = build_inventory(tmp.path(), &["src/**".into()], None, false).unwrap();
        let source_ir = build_source_ir_for_files(&inventory.files.iter().collect::<Vec<_>>());
        let primitives: BTreeSet<_> = source_ir
            .iter()
            .flat_map(|unit| unit.generator_primitives.iter().cloned())
            .collect();

        assert!(primitives.contains("fastapi_decorator_route"));
        assert!(primitives.contains("fastapi_class_route"));
    }

    #[test]
    fn semantic_coverage_excludes_claim_tds() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "src/app.py",
            "# <HANDWRITE gap=\"g\" tracker=\"t\" reason=\"r\">\ndef handle():\n    return 1\n# </HANDWRITE>\n",
        );
        write(
            tmp.path(),
            "tech-design/src/app.md",
            "---\nid: src-app\n---\n\n## Changes\n```yaml\nchanges:\n  - path: src/app.py\n    action: modify\n    impl_mode: hand-written\n```\n",
        );

        let inventory = build_inventory(tmp.path(), &["**".into()], None, false).unwrap();
        let coverage = build_semantic_coverage(tmp.path(), &inventory).unwrap();

        assert_eq!(coverage.claim_files, 1);
        assert_eq!(coverage.semantic_files, 0);
        assert_eq!(coverage.semantically_covered_files, 0);
        assert_eq!(coverage.uncovered_files, vec!["src/app.py"]);
        assert_eq!(
            coverage.next_gap.as_ref().map(|gap| gap.primitive.as_str()),
            Some("semantic_td_missing")
        );
    }

    #[test]
    fn semantic_coverage_maps_source_to_semantic_td() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "src/app.py",
            "# SPEC-MANAGED: tech-design/features/app-api.md#changes\n# CODEGEN-BEGIN\ndef handle():\n    return 1\n# CODEGEN-END\n",
        );
        write(
            tmp.path(),
            "tech-design/features/app-api.md",
            "---\nid: app-api\n---\n\n## Changes\n```yaml\nchanges:\n  - path: src/app.py\n    action: modify\n    impl_mode: generated\n```\n",
        );
        write(
            tmp.path(),
            "tech-design/src/app.md",
            "---\nid: src-app\n---\n\n## Changes\n```yaml\nchanges:\n  - path: src/app.py\n    action: modify\n    impl_mode: hand-written\n```\n",
        );

        let inventory = build_inventory(tmp.path(), &["**".into()], None, false).unwrap();
        let coverage = build_semantic_coverage(tmp.path(), &inventory).unwrap();

        assert_eq!(coverage.claim_files, 1);
        assert_eq!(coverage.semantic_files, 1);
        assert_eq!(coverage.semantically_covered_files, 1);
        assert!(coverage.uncovered_files.is_empty());
        assert_eq!(coverage.percent, 100.0);
        assert_eq!(
            coverage.coverage_map[0].td_section.as_deref(),
            Some("tech-design/features/app-api.md")
        );
    }

    #[test]
    fn semantic_coverage_excludes_aw_ec_generated_wrappers() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "src/app.rs",
            "// SPEC-MANAGED: tech-design/features/app-api.md#changes\n// CODEGEN-BEGIN\npub fn handle() -> i32 { 1 }\n// CODEGEN-END\n",
        );
        write(
            tmp.path(),
            "tests/behavior_app_contract.rs",
            "// SPEC-MANAGED: tech-design/features/external-contracts.md#app-contract\n// CODEGEN-BEGIN\n// AW-EC-BEGIN\n// @ec app-contract\n#[test]\n#[ignore = \"generated EC wrapper\"]\nfn app_contract() {}\n// AW-EC-END\n// CODEGEN-END\n",
        );
        write(
            tmp.path(),
            "tech-design/features/app-api.md",
            "---\nid: app-api\nfill_sections: [changes]\n---\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\ncoverage_kind: semantic\nchanges:\n  - path: src/app.rs\n    action: modify\n    impl_mode: generated\n```\n",
        );

        let inventory = build_inventory(tmp.path(), &["**".into()], None, false).unwrap();
        assert_eq!(inventory.files.len(), 2);

        let coverage = build_semantic_coverage(tmp.path(), &inventory).unwrap();

        assert_eq!(coverage.total_files, 1);
        assert_eq!(coverage.source_units, 1);
        assert_eq!(coverage.semantically_covered_files, 1);
        assert_eq!(coverage.percent, 100.0);
        assert!(coverage.uncovered_files.is_empty());
        assert!(coverage
            .coverage_map
            .iter()
            .all(|entry| entry.source_unit != "tests/behavior_app_contract.rs"));
        assert!(coverage
            .source_ir
            .iter()
            .all(|unit| unit.path != "tests/behavior_app_contract.rs"));
    }

    #[test]
    fn semantic_coverage_prioritizes_missing_td_before_generator_gap() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "src/a.py",
            "# <HANDWRITE gap=\"g\" tracker=\"t\" reason=\"r\">\ndef covered():\n    return 1\n# </HANDWRITE>\n",
        );
        write(
            tmp.path(),
            "src/z.py",
            "# <HANDWRITE gap=\"g\" tracker=\"t\" reason=\"r\">\ndef uncovered():\n    return 2\n# </HANDWRITE>\n",
        );
        write(
            tmp.path(),
            "tech-design/features/a.md",
            "---\nid: a\ntype: semantic\n---\n\n## Changes\n```yaml\nchanges:\n  - path: src/a.py\n    action: modify\n```\n",
        );

        let inventory = build_inventory(tmp.path(), &["**".into()], None, false).unwrap();
        let coverage = build_semantic_coverage(tmp.path(), &inventory).unwrap();

        assert_eq!(
            coverage.next_gap.as_ref().map(|gap| gap.target.as_str()),
            Some("src/z.py")
        );
        assert_eq!(
            coverage.next_gap.as_ref().map(|gap| gap.primitive.as_str()),
            Some("semantic_td_missing")
        );
    }

    #[test]
    fn semantic_td_without_source_evidence_node_needs_migration() {
        let td = "## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nsemantic_domain:\n  coverage_kind: semantic\n  evidence:\n    source_units:\n      - path: src/app.py\n        language: python\n```\n";

        assert!(semantic_td_needs_source_evidence_graph_migration(td));

        let migrated = "## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nsemantic_domain:\n  coverage_kind: semantic\n  evidence:\n    source_units:\n      - path: src/app.py\n        language: python\n        source_evidence_node:\n          layer: backend\n```\n";
        assert!(!semantic_td_needs_source_evidence_graph_migration(migrated));
    }

    #[test]
    fn semantic_td_without_impl_mode_needs_migration() {
        let td = "## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\ncoverage_kind: semantic\nchanges:\n  - path: src/app.py\n    action: modify\n    section: schema\n```\n";
        assert!(semantic_td_needs_impl_mode_migration(td));

        let migrated = "## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\ncoverage_kind: semantic\nchanges:\n  - path: src/app.py\n    action: modify\n    section: schema\n    impl_mode: hand-written\n```\n";
        assert!(!semantic_td_needs_impl_mode_migration(migrated));
    }

    #[test]
    fn semantic_td_with_handwritten_impl_mode_is_not_claim() {
        let td = "## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\ncoverage_kind: semantic\nchanges:\n  - path: src/app.py\n    action: modify\n    section: schema\n    impl_mode: hand-written\n```\n";
        let record = td_coverage_record(".aw/tech-design/semantic/app.md", td);
        assert!(
            !record.is_claim,
            "semantic TDs use impl_mode to suppress codegen and must still count as semantic coverage"
        );
    }

    #[test]
    fn metadata_only_handwritten_impl_mode_is_not_claim() {
        let td = "## Traceability Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - action: annotate\n    section: schema\n    impl_mode: hand-written\n```\n";
        let record = td_coverage_record(".aw/tech-design/features/app.md", td);
        assert!(
            !record.is_claim,
            "metadata-only traceability entries do not claim source ownership"
        );
    }

    #[test]
    fn traceability_td_with_valid_capability_ref_passes() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            valid_traceability_td(),
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert_eq!(coverage.blocker_count, 0);
        assert_eq!(coverage.total_td_files, 1);
        assert_eq!(coverage.traceable_td_files, 1);
        assert_eq!(coverage.traceability_percent, 100.0);
    }

    #[test]
    fn traceability_missing_capability_sections_returns_blocker() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write(tmp.path(), "src/app.py", "def handle():\n    return 1\n");
        write(
            tmp.path(),
            "README.md",
            "# demo\n\nGeneral product docs only.\n",
        );

        let coverage = traceability_coverage_for(tmp.path());

        let blocker = coverage.next_gap.as_ref().unwrap();
        assert_eq!(coverage.blocker_count, 1);
        assert_eq!(coverage.traceability_percent, 0.0);
        assert_eq!(
            blocker.kind,
            TraceabilityBlockerKind::TdInvalidCapabilityRef
        );
        assert!(blocker
            .reason
            .contains("capability document has no capability sections"));
        assert!(blocker.reason.contains("no capability sections found"));
    }

    #[test]
    fn traceability_command_ref_to_valid_capability_td_passes() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            &valid_traceability_td_with_command_ref("aw demo run"),
        );

        let coverage = traceability_coverage_for_commands(tmp.path(), &[("aw demo run", false)]);

        assert_eq!(coverage.command_traceability.blockers, Vec::new());
        assert_eq!(coverage.command_traceability.total_command_paths, 1);
        assert_eq!(coverage.command_traceability.traceable_command_paths, 1);
        assert_eq!(
            coverage.command_traceability.command_traceability_percent,
            100.0
        );
    }

    #[test]
    fn traceability_public_command_without_td_ref_fails() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            valid_traceability_td(),
        );

        let coverage = traceability_coverage_for_commands(tmp.path(), &[("aw demo run", false)]);

        assert!(coverage
            .command_traceability
            .blockers
            .iter()
            .any(|blocker| {
                blocker.kind == TraceabilityBlockerKind::CommandNoTdRef
                    && blocker.target == "aw demo run"
            }));
    }

    #[test]
    fn traceability_unknown_command_ref_fails() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            &valid_traceability_td_with_command_ref("aw missing command"),
        );

        let coverage = traceability_coverage_for_commands(tmp.path(), &[]);

        assert!(coverage
            .command_traceability
            .blockers
            .iter()
            .any(|blocker| {
                blocker.kind == TraceabilityBlockerKind::CommandRefUnknownCommand
                    && blocker.target == "aw missing command"
            }));
    }

    #[test]
    fn traceability_command_ref_td_without_capability_ref_fails() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-td
command_refs:
  - command: aw demo run
---

# Demo TD
"#,
        );

        let coverage = traceability_coverage_for_commands(tmp.path(), &[("aw demo run", false)]);

        assert!(coverage
            .command_traceability
            .blockers
            .iter()
            .any(|blocker| {
                blocker.kind == TraceabilityBlockerKind::CommandRefTdNoCapabilityRef
                    && blocker.target == "aw demo run"
            }));
    }

    #[test]
    fn traceability_hidden_command_registered_fails() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            &valid_traceability_td_with_command_ref("aw hidden run"),
        );

        let coverage = traceability_coverage_for_commands(tmp.path(), &[("aw hidden run", true)]);

        assert!(coverage
            .command_traceability
            .blockers
            .iter()
            .any(|blocker| {
                blocker.kind == TraceabilityBlockerKind::HiddenCommandRegistered
                    && blocker.target == "aw hidden run"
            }));
    }

    #[test]
    fn traceability_command_blocker_is_prioritized() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            "---\nid: demo-td\n---\n\n# Demo TD\n",
        );

        let coverage = traceability_coverage_for_commands(tmp.path(), &[("aw demo run", false)]);

        assert_eq!(
            coverage.next_gap.as_ref().map(|gap| gap.kind),
            Some(TraceabilityBlockerKind::CommandNoTdRef)
        );
    }

    #[test]
    fn traceability_runtime_command_inventory_lists_nested_paths() {
        let inventory = runtime_command_inventory();

        for path in [
            "aw wi",
            "aw wi draft",
            "aw wi draft init",
            "aw td",
            "aw td check",
            "aw td gen",
            "aw td code-check",
            "aw standardize",
            // #920 (epic #914 slice F): `aw standardize traceability` (and
            // its `report`/`next`/`run` stage subcommands) are retired --
            // only `aw standardize audit` (and its `check`/`record`
            // subcommands) remain live.
            "aw standardize audit",
            "aw standardize audit check",
        ] {
            assert!(
                inventory.contains_key(path),
                "{path} missing from inventory"
            );
        }
    }

    #[test]
    fn traceability_non_command_project_ignores_aw_active_doc_refs() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(
            tmp.path(),
            "AGENTS.md",
            "Use `aw capability report --project demo` for capability checks.\n",
        );
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            valid_traceability_td(),
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert_eq!(coverage.blocker_count, 0);
        assert!(!coverage
            .command_traceability
            .blockers
            .iter()
            .any(|blocker| blocker.kind == TraceabilityBlockerKind::ActiveDocUnknownCommandRef));
    }

    #[test]
    fn active_doc_paths_scans_readme_md() {
        // The root README is the project and shared-library inventory and must
        // be scanned alongside AGENTS.md/CLAUDE.md/CONTRIBUTING.md, so a stale
        // command reference there is caught too.
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "README.md",
            "The project inventory used to say `aw run --project demo`.\n",
        );

        let inventory = runtime_command_inventory();
        let blockers = active_doc_command_blockers(tmp.path(), &inventory);
        assert!(
            blockers.iter().any(
                |b| b.kind == TraceabilityBlockerKind::ActiveDocDeletedCommandRef
                    && b.source.as_deref() == Some("README.md")
            ),
            "expected README.md to be scanned for deleted-command refs, got {blockers:?}"
        );
    }

    #[test]
    fn deleted_command_paths_918_hits_flag_active_doc_deleted_command_ref() {
        // #918: `aw run` full removal + #857 merge-era leftovers. Each of
        // these literals must be caught when a scanned active doc still
        // references the removed verb.
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "AGENTS.md",
            "Kick off the loop with `aw run --project demo`.\n\
             Legacy artifact steps used `aw td merge`, `aw td review`, and `aw td revise`.\n\
             Do not use `aw wi merge` or `aw cb fill` here.\n",
        );

        let inventory = runtime_command_inventory();
        let blockers = active_doc_command_blockers(tmp.path(), &inventory);
        let deleted_targets: BTreeSet<&str> = blockers
            .iter()
            .filter(|b| b.kind == TraceabilityBlockerKind::ActiveDocDeletedCommandRef)
            .map(|b| b.target.as_str())
            .collect();

        for expected in [
            "aw run",
            "aw td merge",
            "aw td review",
            "aw td revise",
            "aw wi merge",
            "aw cb ",
        ] {
            assert!(
                deleted_targets.contains(expected),
                "expected {expected:?} to be flagged as a deleted-command ref, got {deleted_targets:?}"
            );
        }
    }

    #[test]
    fn deleted_command_paths_918_does_not_false_positive_on_new_runner_verbs() {
        // `"aw run"` and `"aw cb "` must not match inside the new #917
        // canonical runner shells or inside ordinary `cb_*` phase prose.
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "AGENTS.md",
            "Use `aw wi run 42` or `aw capability run demo:foo --project demo`.\n\
             The lifecycle transitions through the cb_filled phase before\n\
             `aw td code-check` closes the loop.\n",
        );

        let inventory = runtime_command_inventory();
        let blockers = active_doc_command_blockers(tmp.path(), &inventory);
        let deleted_targets: Vec<&str> = blockers
            .iter()
            .filter(|b| b.kind == TraceabilityBlockerKind::ActiveDocDeletedCommandRef)
            .map(|b| b.target.as_str())
            .collect();

        assert!(
            deleted_targets.is_empty(),
            "expected no deleted-command hits, got {deleted_targets:?}"
        );
    }

    #[test]
    fn deleted_command_paths_920_hits_flag_active_doc_deleted_standardize_layer_ref() {
        // #920 (epic #914 slice F): `aw standardize` is retired down to
        // `audit` only. Each of the removed layer drivers must still be
        // caught when a scanned active doc references them.
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "AGENTS.md",
            "Run `aw standardize managed run --project demo` to drive coverage.\n\
             Follow with `aw standardize semantic next` and\n\
             `aw standardize traceability report --project demo`.\n",
        );

        let inventory = runtime_command_inventory();
        let blockers = active_doc_command_blockers(tmp.path(), &inventory);
        let deleted_targets: BTreeSet<&str> = blockers
            .iter()
            .filter(|b| b.kind == TraceabilityBlockerKind::ActiveDocDeletedCommandRef)
            .map(|b| b.target.as_str())
            .collect();

        for expected in [
            "aw standardize managed",
            "aw standardize semantic",
            "aw standardize traceability",
        ] {
            assert!(
                deleted_targets.contains(expected),
                "expected {expected:?} to be flagged as a deleted-command ref, got {deleted_targets:?}"
            );
        }
    }

    #[test]
    fn deleted_command_paths_920_does_not_false_positive_on_standardize_audit() {
        // `aw standardize audit` (and a bare `aw standardize --project <p>`
        // reference) must not match the retired `managed`/`semantic`/
        // `traceability` layer-driver literals.
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "AGENTS.md",
            "Run `aw standardize audit check --project demo` first, then\n\
             `aw standardize audit record --project demo`. A bare\n\
             `aw standardize --project demo` requires an explicit subcommand.\n",
        );

        let inventory = runtime_command_inventory();
        let blockers = active_doc_command_blockers(tmp.path(), &inventory);
        let deleted_targets: Vec<&str> = blockers
            .iter()
            .filter(|b| b.kind == TraceabilityBlockerKind::ActiveDocDeletedCommandRef)
            .map(|b| b.target.as_str())
            .collect();

        assert!(
            deleted_targets.is_empty(),
            "expected no deleted-command hits, got {deleted_targets:?}"
        );
    }

    #[test]
    fn traceability_td_without_capability_refs_is_orphan_metric_only() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            "---\nid: demo-td\n---\n\n# Demo TD\n",
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert_eq!(coverage.orphan_td_count, 1);
        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdNoCapabilityRef
                && blocker.target == "tech-design/demo/app.md"
        }));
    }

    #[test]
    fn traceability_unknown_capability_ref_fails() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-td
capability_refs:
  - id: missing-capability
    role: primary
    coverage: full
---

# Demo TD
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdInvalidCapabilityRef
                && blocker.reason.contains("unknown capability id")
        }));
    }

    #[test]
    fn traceability_internal_td_without_source_edge_passes() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/internal.md",
            "---\nid: internal\ncapability_scope: internal\n---\n\n# Internal TD\n",
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert_eq!(coverage.blocker_count, 0);
        assert_eq!(coverage.internal_td_count, 1);
    }

    #[test]
    fn traceability_internal_td_with_source_edge_fails() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: internal
capability_scope: internal
---

# Internal TD

## Changes
```yaml
coverage_kind: semantic
changes:
  - path: src/app.py
    action: modify
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::InternalTdHasSourceEdge
                && blocker.target == "tech-design/demo/app.md"
        }));
    }

    #[test]
    fn traceability_section_without_implementation_edge_fails() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-td
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo TD

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions: {}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes: []
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdSectionNoImplementationEdge
                && blocker.source.as_deref() == Some("section:schema")
        }));
    }

    #[test]
    fn traceability_e2e_test_section_is_owned_by_ec_gate() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/external-contracts.md",
            r#"---
id: demo-external-contracts
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo EC TD

## Demo Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: demo-contract
    command: cargo test -p demo demo_contract
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdSectionNoImplementationEdge
                && blocker.source.as_deref() == Some("section:e2e-test")
        }));
    }

    #[test]
    fn traceability_excludes_aw_ec_generated_wrapper_cb_edges() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "tests/**");
        write_traceability_readme(tmp.path());
        write(
            tmp.path(),
            "tests/behavior_demo_contract.rs",
            "// SPEC-MANAGED: external-contracts/behavior/demo.toml#demo-contract\n// CODEGEN-BEGIN\n// AW-EC-BEGIN\n// @ec demo-contract\n#[test]\n#[ignore = \"generated EC wrapper\"]\nfn demo_contract() {}\n// AW-EC-END\n// CODEGEN-END\n",
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(coverage.blockers.iter().all(|blocker| {
            blocker.source.as_deref() != Some("tests/behavior_demo_contract.rs")
        }));
    }

    #[test]
    fn traceability_change_missing_impl_mode_fails() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-td
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo TD

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions: {}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/app.py
    action: modify
    section: schema
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdChangeMissingImplMode
                && blocker.source.as_deref() == Some("changes[0]:src/app.py")
        }));
    }

    #[test]
    fn traceability_accepts_legacy_source_section_change_edge() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-td
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo TD

## Source
<!-- type: source lang: rust -->

```rust
pub fn demo() {}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/app.rs
    action: modify
    section: source
    impl_mode: codegen
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdChangeInvalidSection
                && blocker.source.as_deref() == Some("changes[0]:src/app.rs")
        }));
        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdSectionNoImplementationEdge
                && blocker.source.as_deref() == Some("section:source")
        }));
    }

    #[test]
    fn traceability_accepts_rust_source_unit_change_edge() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-td
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo TD

## Source
<!-- type: rust-source-unit lang: rust -->

```rust
pub fn demo() {}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/app.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdChangeInvalidSection
                && blocker.source.as_deref() == Some("changes[0]:src/app.rs")
        }));
        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdSectionNoImplementationEdge
                && blocker.source.as_deref() == Some("section:rust-source-unit")
        }));
    }

    #[test]
    fn traceability_accepts_text_source_unit_change_edge() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-td
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo TD

## Source
<!-- type: text-source-unit lang: bash -->

```bash
#!/usr/bin/env bash
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: build.sh
    action: modify
    section: text-source-unit
    impl_mode: codegen
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdChangeInvalidSection
                && blocker.source.as_deref() == Some("changes[0]:build.sh")
        }));
        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdSectionNoImplementationEdge
                && blocker.source.as_deref() == Some("section:text-source-unit")
        }));
    }

    #[test]
    fn traceability_accepts_generator_exports_change_edge() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "docs/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "docs/note.txt", "not a source file\n");
        write(
            tmp.path(),
            "tech-design/demo/module.md",
            r#"---
id: demo-td
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo TD

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/mod.rs
    action: modify
    section: exports
    impl_mode: codegen
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::TdChangeInvalidSection
                && blocker.source.as_deref() == Some("changes[0]:src/mod.rs")
        }));
    }

    #[test]
    fn traceability_change_entries_merges_multiple_changes_sections() {
        let entries = traceability_change_entries(
            r##"# Demo

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/app.rs
    action: modify
    section: source
    impl_mode: codegen
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

Approved.

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: schema
    impl_mode: hand-written
```
"##,
        )
        .expect("changes entries");

        let sections = entries
            .iter()
            .filter_map(|entry| entry.section.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(sections, vec!["source", "schema"]);
    }

    #[test]
    fn traceability_change_entries_merges_after_long_source_fence() {
        let entries = traceability_change_entries(
            r###"# Demo

## Source
<!-- type: source lang: rust -->

````rust
pub fn demo() {
    let fixture = r#"
## Nested Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/fixture.rs
    action: modify
```
"#;
}
````

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/app.rs
    action: modify
    section: source
    impl_mode: codegen
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

Approved.

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: schema
    impl_mode: hand-written
```
"###,
        )
        .expect("changes entries");

        let sections = entries
            .iter()
            .filter_map(|entry| entry.section.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(sections, vec!["source", "schema"]);
        let paths = entries
            .iter()
            .filter_map(|entry| entry.path.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(paths, vec!["src/app.rs"]);
    }

    #[test]
    fn traceability_change_entries_uses_later_metadata_when_legacy_changes_is_malformed() {
        let entries = traceability_change_entries(
            r##"# Demo

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions: {}
```

## Changes
<!-- type: changes lang: yaml -->

legacy prose without a YAML fence

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: schema
    impl_mode: hand-written
```
"##,
        )
        .expect("changes entries");

        let sections = entries
            .iter()
            .filter_map(|entry| entry.section.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(sections, vec!["schema"]);
    }

    #[test]
    fn traceability_change_entries_keeps_yaml_comment_lines_inside_fence() {
        let entries = traceability_change_entries(
            r##"# Demo

## Changes
<!-- type: changes lang: yaml -->

```yaml
# This YAML comment is not a markdown heading.
changes:
  - path: src/app.rs
    action: modify
    section: source
    impl_mode: codegen
```
"##,
        )
        .expect("changes entries");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path.as_deref(), Some("src/app.rs"));
    }

    #[test]
    fn traceability_change_entries_reads_real_metadata_edge() {
        let content = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tech-design/core/interfaces/agents/code_agent/mod.md"),
        )
        .expect("read real TD fixture");
        let entries = traceability_change_entries(&content).expect("changes entries");

        assert!(entries.iter().any(|entry| {
            entry.section.as_deref() == Some("schema")
                && entry.impl_mode.as_deref() == Some("hand-written")
        }));
    }

    #[test]
    fn traceability_change_entries_reads_state_machine_repair_edges() {
        let content = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tech-design/core/logic/state-machine.md"),
        )
        .expect("read state-machine TD fixture");
        let entries = traceability_change_entries(&content).expect("changes entries");

        assert!(
            entries.iter().any(|entry| {
                entry.section.as_deref() == Some("async-api")
                    && entry.impl_mode.as_deref() == Some("hand-written")
            }),
            "entries: {entries:?}"
        );
    }

    #[test]
    fn traceability_source_and_cb_edges_to_td_without_capability_refs_fail() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-td
---

# Demo TD

## Changes
```yaml
coverage_kind: semantic
changes:
  - path: src/app.py
    action: modify
```
"#,
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::SourceBlockTdNoCapabilityRef
                && blocker.source.as_deref() == Some("src/app.py")
        }));
        assert!(coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::CbBlockTdNoCapabilityRef
                && blocker.source.as_deref() == Some("src/app.py:2")
        }));
    }

    #[test]
    fn traceability_capability_owned_source_path_closes_legacy_source_refs() {
        let tmp = TempDir::new().unwrap();
        write_traceability_config(tmp.path(), "src/**");
        write_traceability_readme(tmp.path());
        write(tmp.path(), "src/app.py", source_referencing_demo_td());
        write(
            tmp.path(),
            "tech-design/demo/app.md",
            r#"---
id: demo-legacy-td
---

# Demo Legacy TD

## Changes
```yaml
coverage_kind: semantic
changes:
  - path: src/app.py
    action: modify
```
"#,
        );
        write(
            tmp.path(),
            "tech-design/demo/aggregate.md",
            valid_traceability_td(),
        );

        let coverage = traceability_coverage_for(tmp.path());

        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::SourceBlockTdNoCapabilityRef
                && blocker.source.as_deref() == Some("src/app.py")
        }));
        assert!(!coverage.blockers.iter().any(|blocker| {
            blocker.kind == TraceabilityBlockerKind::CbBlockTdNoCapabilityRef
                && blocker.source.as_deref() == Some("src/app.py:2")
        }));
    }

    #[test]
    fn semantic_action_returns_next_deterministic_gap_as_cli_tick() {
        let mut coverage = empty_semantic(vec!["src/**".into()]);
        coverage
            .generator_primitive_gaps
            .push(GeneratorPrimitiveGap {
                target: "src/app.py".into(),
                primitive: "semantic_td_missing".into(),
                reason: "missing semantic TD".into(),
                human_decision_required: false,
            });
        coverage.next_gap = Some(SemanticGap {
            target: "src/app.py".into(),
            primitive: "semantic_td_missing".into(),
            reason: "missing semantic TD".into(),
            action: "draft_or_update_semantic_td_from_source_ir".into(),
        });

        let action = choose_semantic_action(&coverage);

        assert_eq!(action.kind, StandardizeActionKind::SemanticGap);
        assert!(!action.requires_hitl);
        assert_eq!(action.executor, "cli");
        assert!(action.command.contains("aw standardize semantic run"));
        assert!(action.command.contains("src/**"));
        assert!(!action.command.contains("src/app.py --max-ticks"));

        coverage.next_gap = Some(SemanticGap {
            target:
                "examples/fixture_platform/backend/.gitlab/ci/scripts/get_latest_alembic_version.py"
                    .into(),
            primitive: "semantic_td_missing".into(),
            reason: "missing semantic TD".into(),
            action: "draft_or_update_semantic_td_from_source_ir".into(),
        });
        let action = choose_semantic_action(&coverage);
        assert!(action
            .command
            .contains("examples/fixture_platform/backend/.gitlab/ci/scripts/**"));
        assert!(!action
            .command
            .contains("get_latest_alembic_version.py --max-ticks"));
    }

    #[test]
    fn semantic_coverage_flags_missing_generated_capability_ref_for_refresh() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "aw.toml",
            r#"
[[projects]]
name = "fixture_platform"
path = "examples/fixture_platform"
td_path = "examples/fixture_platform/tech_design"
cap_path = "README.md"

[[projects.workspaces]]
name = "backend"
paths = ["examples/fixture_platform/backend/**"]
target = "python"
"#,
        );
        write_traceability_readme(tmp.path());
        write(
            tmp.path(),
            "examples/fixture_platform/backend/scripts/load_fixture.py",
            "print('load')\n",
        );
        write(
            tmp.path(),
            "examples/fixture_platform/tech_design/semantic/backend-scripts.md",
            "---\nid: semantic-backend-scripts\nfill_sections: [schema, unit-test, changes]\n---\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nsemantic_domain:\n  coverage_kind: semantic\n  evidence:\n    source_units:\n      - path: \"examples/fixture_platform/backend/scripts/load_fixture.py\"\n        language: python\n        source_evidence_node:\n          layer: backend\n          ecosystem: python\n          role: source\n          section_type: schema\n          domain: examples/fixture_platform/backend/scripts\n```\n\n## Unit Test\n<!-- type: unit-test lang: mermaid -->\n\n```mermaid\n---\nid: unit-test\n---\nrequirementDiagram\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\ncoverage_kind: semantic\nchanges:\n  - path: \"examples/fixture_platform/backend/scripts/load_fixture.py\"\n    action: modify\n    section: schema\n    impl_mode: hand-written\n  - action: annotate\n    section: unit-test\n    impl_mode: hand-written\n```\n",
        );
        let inventory = build_inventory(
            tmp.path(),
            &["examples/fixture_platform/backend/**".into()],
            None,
            false,
        )
        .unwrap();

        let coverage = build_semantic_coverage(tmp.path(), &inventory).unwrap();

        assert!(coverage.generator_primitive_gaps.iter().any(|gap| {
            gap.target == "examples/fixture_platform/backend/scripts/load_fixture.py"
                && gap.primitive == "semantic_td_legacy"
        }));
    }

    #[test]
    fn source_evidence_node_falls_back_for_unclassified_source_languages() {
        let file = SourceFile {
            rel: "projects/lumen/build.sh".into(),
            abs: PathBuf::from("projects/lumen/build.sh"),
            language: "shell".into(),
            markers: FileMarkers::default(),
            handwrite_gaps: Vec::new(),
        };

        let node = build_source_evidence_node(&file, &[], &["source_unit".to_string()], None)
            .expect("all in-scope source units should have evidence nodes");

        assert_eq!(node.path, "projects/lumen/build.sh");
        assert_eq!(node.layer, "source");
        assert_eq!(node.ecosystem, "shell");
        assert_eq!(node.section_type, "schema");
    }

    #[test]
    fn semantic_coverage_does_not_queue_unsupported_handwrite_promotion() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "projects/lumen/build.sh",
            "# <HANDWRITE gap=\"standardize:shell\" tracker=\"shell\" reason=\"r\">\n#!/usr/bin/env bash\ncargo build -p lumen\n# </HANDWRITE>\n",
        );
        write(
            tmp.path(),
            "projects/lumen/tech-design/semantic/lumen-projects-lumen.md",
            "---\nid: semantic-lumen-projects-lumen\nfill_sections: [schema, unit-test, changes]\n---\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nsemantic_domain:\n  coverage_kind: semantic\n  evidence:\n    source_units:\n      - path: \"projects/lumen/build.sh\"\n        language: shell\n        source_evidence_node:\n          layer: source\n          ecosystem: shell\n          role: source\n          section_type: schema\n          domain: projects/lumen\n```\n\n## Unit Test\n<!-- type: unit-test lang: mermaid -->\n\n```mermaid\n---\nid: unit-test\n---\nrequirementDiagram\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\ncoverage_kind: semantic\nchanges:\n  - path: \"projects/lumen/build.sh\"\n    action: modify\n    section: schema\n    impl_mode: hand-written\n  - action: annotate\n    section: unit-test\n    impl_mode: hand-written\n```\n",
        );

        let inventory = build_inventory(tmp.path(), &["projects/lumen/**".into()], None, false)
            .expect("inventory should build");
        let coverage =
            build_semantic_coverage(tmp.path(), &inventory).expect("semantic coverage builds");

        assert_eq!(coverage.percent, 100.0);
        assert!(coverage
            .generator_primitive_gaps
            .iter()
            .all(|gap| { gap.target != "projects/lumen/build.sh" }));
    }

    #[test]
    fn frontend_semantic_group_key_uses_domain_roots_for_feature_libraries() {
        assert_eq!(
            semantic_group_key(
                "examples/fixture_platform/frontend/libs/fixture-platform-lib/src/features/data-table/views/table.tsx"
            ),
            "examples/fixture_platform/frontend/libs/fixture-platform-lib/src/features/data-table"
        );
        assert_eq!(
            semantic_group_key(
                "examples/fixture_platform/frontend/libs/fixture-platform-lib/src/features/data-table/models/table.ts"
            ),
            "examples/fixture_platform/frontend/libs/fixture-platform-lib/src/features/data-table"
        );
        assert_eq!(
            semantic_group_key(
                "examples/fixture_platform/frontend/libs/fixture-platform-lib/src/shared/workspace-permission/view-models/hooks/use-permission.ts"
            ),
            "examples/fixture_platform/frontend/libs/fixture-platform-lib/src/shared/workspace-permission"
        );
        assert_eq!(
            semantic_group_key(
                "examples/fixture_platform/frontend/libs/shared-ui-form-inputs/src/lib/form-input/form-input.test.tsx"
            ),
            "examples/fixture_platform/frontend/libs/shared-ui-form-inputs/src/lib/form-input"
        );
    }

    #[test]
    fn source_evidence_graph_groups_backend_frontend_and_operations_nodes() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "examples/fixture_platform/backend/src/features/workspace/folder/api.py",
            "from fastapi import APIRouter\nrouter = APIRouter()\n@router.get('/folders')\ndef list_folders():\n    return []\n",
        );
        write(
            tmp.path(),
            "examples/fixture_platform/frontend/apps/demo/src/app.tsx",
            "export function App() {\n  return <main />;\n}\n",
        );
        write(
            tmp.path(),
            "examples/fixture_platform/backend/Dockerfile",
            "FROM python:3.12\n",
        );
        let inventory = build_inventory(
            tmp.path(),
            &["examples/fixture_platform/**".into()],
            None,
            false,
        )
        .unwrap();
        let source_ir = build_source_ir_for_files(&inventory.files.iter().collect::<Vec<_>>());
        let source_evidence_graph =
            build_source_evidence_graph(&source_ir).expect("source evidence graph");

        assert!(source_evidence_graph.source_nodes.iter().any(|node| {
            node.path.ends_with("api.py") && node.layer == "backend" && node.role == "api"
        }));
        assert!(source_evidence_graph.source_nodes.iter().any(|node| {
            node.path.ends_with("app.tsx")
                && node.layer == "frontend"
                && node.section_type == "component"
        }));
        assert!(source_evidence_graph.source_nodes.iter().any(|node| {
            node.path.ends_with("Dockerfile")
                && node.layer == "operations"
                && node.section_type == "runtime-image"
        }));
        assert!(source_evidence_graph
            .domains
            .iter()
            .any(|domain| domain.layers.iter().any(|layer| layer == "backend")));
        assert!(source_ir
            .iter()
            .all(|unit| unit.source_evidence_node.is_some()));
    }

    #[test]
    fn regenerable_routes_rust_test_helpers_to_cli_promotion() {
        let inv = Inventory {
            coverage: StandardizationCoverage {
                scope: vec!["apps/agentic-workflow/**".into()],
                total_files: 1,
                managed_files: 1,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 0,
                    handwrite: 1,
                },
                uncovered_files: vec![],
            },
            files: vec![SourceFile {
                rel: "apps/agentic-workflow/tests/cli/tests/td_dirty_gate_test.rs".into(),
                abs: PathBuf::from("apps/agentic-workflow/tests/cli/tests/td_dirty_gate_test.rs"),
                language: "rust".into(),
                markers: FileMarkers {
                    handwrite: true,
                    ..Default::default()
                },
                handwrite_gaps: vec![],
            }],
            rust_findings: vec![],
            spec_violation: None,
        };
        let mut semantic = empty_semantic(inv.coverage.scope.clone());
        semantic
            .generator_primitive_gaps
            .push(GeneratorPrimitiveGap {
                target: "apps/agentic-workflow/tests/cli/tests/td_dirty_gate_test.rs".into(),
                primitive: "service_method".into(),
                reason:
                    "semantic TD coverage exists, but source still contains HANDWRITE ownership"
                        .into(),
                human_decision_required: false,
            });

        let action =
            choose_regenerable_action_with_project(Path::new("."), &inv, &semantic, Some("demo"));

        assert_eq!(action.kind, StandardizeActionKind::GeneratorPrimitiveGap);
        assert_eq!(action.executor, "cli");
        assert!(!action.requires_hitl);
        assert!(action.command.contains("aw generator check --project demo"));
    }

    #[test]
    fn bucket_under_allowed_top_dir_routes_tests_to_validate() {
        assert_eq!(
            bucket_under_allowed_top_dir("backend/tests/conftest.py"),
            "validate/backend/tests/conftest.py"
        );
        assert_eq!(
            bucket_under_allowed_top_dir("backend/test_foo.py"),
            "validate/backend/test_foo.py"
        );
        assert_eq!(
            bucket_under_allowed_top_dir("backend/src/bases.py"),
            "logic/backend/src/bases.py"
        );
        assert_eq!(
            bucket_under_allowed_top_dir("logic/already_bucketed.py"),
            "logic/already_bucketed.py"
        );
        assert_eq!(bucket_under_allowed_top_dir(""), "logic");
    }

    #[test]
    fn claim_code_routes_crate_src_modules_to_allowed_spec_roots() {
        assert_eq!(
            starter_spec_rel_for_source("apps/agentic-workflow/src/ui/mod.rs"),
            "apps/agentic-workflow/tech-design/core/interfaces/ui/mod.md"
        );
        assert_eq!(
            starter_spec_rel_for_source("apps/agentic-workflow/src/generate/schema/parser.rs"),
            "apps/agentic-workflow/tech-design/core/generate/schema/parser.md"
        );
        assert_eq!(
            starter_spec_rel_for_source("apps/agentic-workflow/src/tools/analyze/rust_lang.rs"),
            "apps/agentic-workflow/tech-design/core/tools/analyze/rust_lang.md"
        );
        assert_eq!(
            starter_spec_rel_for_source("apps/agentic-workflow/src/validator/mod.rs"),
            "apps/agentic-workflow/tech-design/core/validate/validator/mod.md"
        );
        assert_eq!(
            starter_spec_rel_for_source("apps/agentic-workflow/src/runtime/envelope.rs"),
            "apps/agentic-workflow/tech-design/core/logic/runtime/envelope.md"
        );
        assert_eq!(
            starter_spec_rel_for_source("apps/agentic-workflow/src/test_util.rs"),
            "apps/agentic-workflow/tech-design/core/validate/test_util.md"
        );
        assert_eq!(
            starter_spec_rel_for_source("apps/agentic-workflow/src/lib.rs"),
            "apps/agentic-workflow/tech-design/core/logic/lib.md"
        );
    }

    #[test]
    fn test_render_emits_doc_when_present() {
        let tmp = TempDir::new().unwrap();
        let abs = tmp.path().join("mod.py");
        fs::write(
            &abs,
            "def greet():\n    \"\"\"Hello.\n\n    World.\n    \"\"\"\n    pass\n",
        )
        .unwrap();
        let out = render_ast_symbols_yaml(&abs);
        assert!(out.contains("doc: \"Hello.\""), "render output:\n{out}");
    }

    #[test]
    fn test_render_emits_imports_block() {
        let tmp = TempDir::new().unwrap();
        let abs = tmp.path().join("mod.py");
        fs::write(
            &abs,
            "import os\nfrom collections import OrderedDict\n\ndef f():\n    pass\n",
        )
        .unwrap();
        let out = render_ast_symbols_yaml(&abs);
        assert!(out.contains("    imports:\n"), "render output:\n{out}");
        assert!(out.contains("path: \"os\""), "render output:\n{out}");
        assert!(
            out.contains("path: \"collections\""),
            "render output:\n{out}"
        );
        assert!(
            out.contains("external: true") || out.contains("external: false"),
            "render output:\n{out}"
        );
    }

    #[test]
    fn test_render_multiline_signature_uses_block_literal() {
        let tmp = TempDir::new().unwrap();
        let abs = tmp.path().join("mod.py");
        fs::write(&abs, "@deco\ndef foo():\n    \"\"\"x\"\"\"\n    pass\n").unwrap();
        let out = render_ast_symbols_yaml(&abs);
        assert!(
            out.contains("signature: |\n"),
            "expected block literal in output:\n{out}"
        );
        assert!(
            out.contains("          @deco\n"),
            "expected indented decorator line in output:\n{out}"
        );
    }

    #[test]
    fn test_render_omits_imports_when_empty() {
        let tmp = TempDir::new().unwrap();
        let abs = tmp.path().join("mod.py");
        fs::write(&abs, "def f():\n    pass\n").unwrap();
        let out = render_ast_symbols_yaml(&abs);
        assert!(
            !out.contains("imports:"),
            "expected no imports block in output:\n{out}"
        );
    }

    #[test]
    fn test_render_single_line_signature_is_scalar() {
        let tmp = TempDir::new().unwrap();
        let abs = tmp.path().join("mod.py");
        fs::write(&abs, "def f():\n    pass\n").unwrap();
        let out = render_ast_symbols_yaml(&abs);
        assert!(
            !out.contains("signature: |"),
            "expected scalar signature in output:\n{out}"
        );
    }

    // #1276 AC1/AC3a: the `aw health` drift/marker axis reports correct
    // counts on a fixture with one drift finding + one marker-gap finding
    // (plus a third, unrelated-kind finding that must NOT be double-counted
    // into either bucket).
    #[test]
    fn drift_marker_coverage_counts_drift_and_marker_gap_findings() {
        let findings = vec![
            RustAuditFinding {
                kind: StandardizeActionKind::RegenDrift,
                target: "src/cli/drift_file.rs".to_string(),
                reason: "CODEGEN block content drifted from spec source".to_string(),
            },
            RustAuditFinding {
                kind: StandardizeActionKind::IssueMarkerGap,
                target: "src/cli/marker_gap_file.rs".to_string(),
                reason: "CODEGEN item missing @spec marker".to_string(),
            },
            RustAuditFinding {
                kind: StandardizeActionKind::FoldShadow,
                target: "src/cli/shadow_file.rs".to_string(),
                reason: "spec-claimed item living outside CODEGEN markers".to_string(),
            },
        ];
        let coverage = drift_marker_coverage("demo", 5, &findings);

        assert_eq!(coverage.project, "demo");
        assert_eq!(coverage.drift_count, 1);
        assert_eq!(coverage.marker_gap_count, 1);
        assert_eq!(coverage.uncovered_count, 1);
        // 5 scanned rust files, 3 carry a finding (one each) -> 2 clean.
        assert_eq!(coverage.clean_files, 2);
        assert_eq!(coverage.findings.len(), 3);
        assert!(coverage
            .findings
            .iter()
            .any(|f| f.kind == StandardizeActionKind::RegenDrift
                && f.target == "src/cli/drift_file.rs"));
        assert!(coverage
            .findings
            .iter()
            .any(|f| f.kind == StandardizeActionKind::IssueMarkerGap
                && f.target == "src/cli/marker_gap_file.rs"));
    }

    #[test]
    fn drift_marker_coverage_is_clean_with_no_findings() {
        let coverage = drift_marker_coverage("demo", 3, &[]);
        assert_eq!(coverage.clean_files, 3);
        assert_eq!(coverage.drift_count, 0);
        assert_eq!(coverage.marker_gap_count, 0);
        assert_eq!(coverage.uncovered_count, 0);
        assert!(coverage.findings.is_empty());
        assert!(coverage.top_finding().is_none());
    }

    // #1276 AC1: `next.command` routing -- `RegenDrift` (a real regen verb
    // is derivable) outranks `FoldShadow` (routes to `aw td promote
    // <path>`), which outranks a bare `IssueMarkerGap` (no deterministic
    // worker verb from a file path alone, falls back to the health pointer).
    #[test]
    fn drift_marker_health_worker_command_routes_by_finding_kind() {
        let regen = DriftMarkerFinding {
            kind: StandardizeActionKind::RegenDrift,
            target: "src/cli/drift_file.rs".to_string(),
            reason: "drift".to_string(),
        };
        assert_eq!(
            drift_marker_health_worker_command("demo", Some(&regen)),
            "aw td gen --force-regen --project demo"
        );

        let shadow = DriftMarkerFinding {
            kind: StandardizeActionKind::FoldShadow,
            target: "src/cli/shadow_file.rs".to_string(),
            reason: "shadow".to_string(),
        };
        assert_eq!(
            drift_marker_health_worker_command("demo", Some(&shadow)),
            "aw td promote src/cli/shadow_file.rs"
        );

        let marker_gap = DriftMarkerFinding {
            kind: StandardizeActionKind::IssueMarkerGap,
            target: "src/cli/marker_gap_file.rs".to_string(),
            reason: "gap".to_string(),
        };
        assert_eq!(
            drift_marker_health_worker_command("demo", Some(&marker_gap)),
            "aw health --project demo drift-marker --verbose"
        );

        assert_eq!(
            drift_marker_health_worker_command("demo", None),
            "aw health --project demo drift-marker --verbose"
        );
    }

    // #1276 AC1: `top_finding` priority order directly, independent of the
    // `next.command` string it feeds.
    #[test]
    fn drift_marker_coverage_top_finding_prefers_regen_drift_over_others() {
        let coverage = DriftMarkerCoverage {
            project: "demo".to_string(),
            clean_files: 0,
            drift_count: 1,
            marker_gap_count: 1,
            uncovered_count: 1,
            findings: vec![
                DriftMarkerFinding {
                    kind: StandardizeActionKind::IssueMarkerGap,
                    target: "b.rs".to_string(),
                    reason: "gap".to_string(),
                },
                DriftMarkerFinding {
                    kind: StandardizeActionKind::FoldShadow,
                    target: "c.rs".to_string(),
                    reason: "shadow".to_string(),
                },
                DriftMarkerFinding {
                    kind: StandardizeActionKind::RegenDrift,
                    target: "a.rs".to_string(),
                    reason: "drift".to_string(),
                },
            ],
        };
        let top = coverage.top_finding().expect("top finding");
        assert_eq!(top.kind, StandardizeActionKind::RegenDrift);
        assert_eq!(top.target, "a.rs");
    }
}
// CODEGEN-END
