// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// CODEGEN-BEGIN
//! `aw capability` -- product capability map governance.

use crate::issues::{
    make_backend, resolve_default_backend, Issue, IssueFilter, IssueState, IssueType,
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use super::capability_type::CapabilityType;
use super::production::{
    evaluate_capability_scope, inputs_from_report_items, ProductionReadinessReport,
    ProductionStatus,
};
use super::project::{project_test_gate_report, ProjectTestGateReport, ProjectTestGateStatus};
use super::workflow_guard;

const CAPABILITY_MIGRATION_INSERT_MARKER: &str = "<!-- aw:capability-migration-insert -->";
const CAPABILITY_GATE_TIMEOUT_ENV: &str = "AW_CAPABILITY_GATE_TIMEOUT_SECS";
const DEFAULT_CAPABILITY_GATE_TIMEOUT_SECS: u64 = 30 * 60;

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct CapabilityArgs {
    /// Configured project name from [[projects]] in .aw/config.toml.
    #[arg(long, global = true)]
    pub project: Option<String>,
    #[command(subcommand)]
    pub command: CapabilityCommand,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Subcommand)]
pub enum CapabilityCommand {
    /// Report product capability completion for one configured project.
    Report(CapabilityReportArgs),
    /// Print the next deterministic capability action.
    Next(CapabilityNextArgs),
    /// Render inferred README capability roots as a pending-review local draft.
    Draft(CapabilityDraftArgs),
    /// Apply a human-reviewed capability draft to the project README.
    ApplyDraft(CapabilityApplyDraftArgs),
    /// Execute one bounded capability tick.
    Run(CapabilityRunArgs),
    /// Rewrite legacy/YAML capability maps to the canonical Markdown format.
    Migrate(CapabilityMigrateArgs),
    /// Validate capability README sections and TD capability refs.
    Check(CapabilityCheckArgs),
    /// Create an empty canonical capability README when the configured map is missing.
    Init(CapabilityInitArgs),
    /// Sweep all configured projects and summarize capability next actions.
    Sweep(CapabilitySweepArgs),
    /// Assign a capability's type, persisting it to the README contract.
    SetType(CapabilitySetTypeArgs),
    /// Assign a capability's status, persisting it to the README contract.
    SetStatus(CapabilitySetStatusArgs),
    /// Upsert an exposed capability surface into the README contract.
    SetSurface(CapabilitySetSurfaceArgs),
    /// Upsert an EC dimension into the README contract.
    SetEcDimension(CapabilitySetEcDimensionArgs),
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilityReportArgs {
    /// Capability map path. Defaults to [[projects]].cap_path or [[projects]].path/README.md.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,
    /// Run verification commands declared in README capability sections.
    #[arg(long)]
    pub verify: bool,
    /// DEPRECATED compatibility no-op. Capability reports emit JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human-readable report.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON report.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
#[command(after_help = r#"Output schema (JSON default):
{
  "schema_version": "aw.cli.v1",
  "status": "continue" | "blocked" | "done",
  "action": "capability",
  "project": string,
  "report_status": string,
  "completion": { "workflow_complete": bool, "requires_hitl": bool, "missing": [string] },
  "next": { "kind": "run_command" | "hitl" | "blocked" | "done" | "error", "command": string?, "reason": string },
  "next_action": object,
  "coverage": object
}"#)]
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub struct CapabilityNextArgs {
    /// Capability map path override.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,
    /// DEPRECATED compatibility no-op. Capability next emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human-readable next action.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON next action.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilityDraftArgs {
    /// Capability map path override.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,
    /// Write draft to this path instead of /tmp/aw/{project}/capability-map-drafts/.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// DEPRECATED compatibility no-op. Capability draft emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the generated draft path only.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON result.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilityApplyDraftArgs {
    /// Pending-review draft artifact path from `aw capability draft` or sweep.
    #[arg(long)]
    pub draft: PathBuf,
    /// Capability map path override.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,
    /// Required assertion that a human has reviewed and accepted this draft.
    #[arg(long)]
    pub reviewed: bool,
    /// DEPRECATED compatibility no-op. Capability apply-draft emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit a compact human-readable report.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON result.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
#[command(after_help = r#"Output schema (JSON default):
Capability run emits the same aw.cli.v1 summary as `aw capability next`, with run_results included after bounded ticks.
"#)]
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub struct CapabilityRunArgs {
    /// Capability map path override.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,
    /// Require bounded, non-interactive execution.
    #[arg(long)]
    pub non_interactive: bool,
    /// Maximum bounded ticks to run.
    #[arg(long, default_value_t = 1)]
    pub max_ticks: usize,
    /// DEPRECATED compatibility no-op. Capability run emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human-readable run report.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON run report.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilityMigrateArgs {
    /// Capability map path override.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,
    /// DEPRECATED compatibility no-op. Capability migrate emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human-readable migrate report.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON migrate report.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilityCheckArgs {
    /// Capability map path override.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,
    /// Run capability verification commands and configured project test gates.
    #[arg(long)]
    pub verify: bool,
    /// DEPRECATED compatibility no-op. Capability check emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human-readable check report.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON check report.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilityInitArgs {
    /// Capability map path override.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,
    /// H1 title for the generated README shell.
    #[arg(long)]
    pub title: Option<String>,
    /// Brief text for the generated README shell. Defaults to an unconfirmed placeholder.
    #[arg(long)]
    pub brief: Option<String>,
    /// DEPRECATED compatibility no-op. Capability init emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the generated README path only.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON result.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilitySweepArgs {
    /// Run capability verification commands and configured project test gates.
    #[arg(long)]
    pub verify: bool,
    /// Include issue inventory when computing next actions.
    #[arg(long = "include-issue-inventory")]
    pub include_issue_inventory: bool,
    /// Skip issue inventory for a fast structural sweep.
    #[arg(long = "skip-issue-inventory")]
    pub skip_issue_inventory: bool,
    /// Write pending-review capability-map drafts for define-map projects.
    #[arg(long = "write-drafts")]
    pub write_drafts: bool,
    /// Write pending-review WI planning artifacts for create-WI projects.
    #[arg(long = "write-wi-plans")]
    pub write_wi_plans: bool,
    /// Write an execution queue for non-HITL next actions not covered by draft/WI-plan queues.
    #[arg(long = "write-action-queue")]
    pub write_action_queue: bool,
    /// DEPRECATED compatibility no-op. Capability sweep emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit a compact human-readable sweep.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON sweep.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilitySetTypeArgs {
    /// Capability id to assign a type to (the README capability heading id).
    #[arg(long)]
    pub capability: String,
    /// Capability type: AgentFirst, Service, Devops, DeveloperTool, RuntimeTool, or SecurityTool.
    #[arg(long = "type")]
    pub r#type: String,
    /// Pretty-print the JSON result.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilitySetStatusArgs {
    /// Capability id to assign a status to (the README capability heading id).
    #[arg(long)]
    pub capability: String,
    /// Capability status: candidate, confirmed, auditing, blocked, verified, or retired.
    #[arg(long)]
    pub status: String,
    /// Pretty-print the JSON result.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilitySetSurfaceArgs {
    /// Capability id whose README contract should receive the surface.
    #[arg(long)]
    pub capability: String,
    /// Surface kind, for example CLI, HTTP, UI, WebAppE2E, or Agent.
    #[arg(long)]
    pub kind: String,
    /// Public command/route/entrypoint for this surface. Repeatable.
    #[arg(long = "command")]
    pub commands: Vec<String>,
    /// Short purpose statement for this surface.
    #[arg(long)]
    pub summary: String,
    /// Pretty-print the JSON result.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args, Clone)]
pub struct CapabilitySetEcDimensionArgs {
    /// Capability id whose README contract should receive the EC dimension.
    #[arg(long)]
    pub capability: String,
    /// EC dimension: behavior, efficiency, security, stability, or content.
    #[arg(long)]
    pub dimension: String,
    /// Tool/runner that verifies this dimension, for example rig, meter, guard, or jet e2e.
    #[arg(long)]
    pub runner: Option<String>,
    /// Short contract summary for this dimension.
    #[arg(long)]
    pub summary: Option<String>,
    /// Efficiency operating point to reserve for aw-generated cube backfill.
    #[arg(long = "operating-point")]
    pub operating_point: Option<String>,
    /// Efficiency cube reference to reserve for aw-generated cube backfill.
    #[arg(long)]
    pub cube: Option<String>,
    /// Pretty-print the JSON result.
    #[arg(long)]
    pub pretty: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Candidate,
    Confirmed,
    Auditing,
    Blocked,
    Verified,
    Retired,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl CapabilityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityStatus::Candidate => "candidate",
            CapabilityStatus::Confirmed => "confirmed",
            CapabilityStatus::Auditing => "auditing",
            CapabilityStatus::Blocked => "blocked",
            CapabilityStatus::Verified => "verified",
            CapabilityStatus::Retired => "retired",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityGapStatus {
    Open,
    InProgress,
    Blocked,
    Closed,
    Deferred,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl CapabilityGapStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityGapStatus::Open => "open",
            CapabilityGapStatus::InProgress => "in_progress",
            CapabilityGapStatus::Blocked => "blocked",
            CapabilityGapStatus::Closed => "closed",
            CapabilityGapStatus::Deferred => "deferred",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRefRole {
    Primary,
    Contributes,
    Affected,
    RegressionGuard,
    OutOfScope,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCoverage {
    Full,
    Partial,
    Enabling,
    Guardrail,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityMaturity {
    Smoke,
    Conformance,
    Corpus,
    Negative,
    Dogfood,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl CapabilityMaturity {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityMaturity::Smoke => "smoke",
            CapabilityMaturity::Conformance => "conformance",
            CapabilityMaturity::Corpus => "corpus",
            CapabilityMaturity::Negative => "negative",
            CapabilityMaturity::Dogfood => "dogfood",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityGap {
    pub id: String,
    pub status: CapabilityGapStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_wi: Option<String>,
    pub summary: String,
}

/// Original capability-index row summary. Keep this during README format
/// migration so AW does not rewrite human-confirmed readiness language.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityIndexSummary {
    pub implementation: String,
    pub verification: String,
    pub maturity: String,
    pub production: String,
    pub notes: String,
}

/// Original Work Root table row. Keep this as the stable TD/WI anchor during
/// README format migration; derived claim labels are not authoritative.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityWorkRoot {
    pub id: String,
    pub work_root: String,
    pub kind: String,
    pub wi: String,
    pub implementation: String,
    pub verification: String,
    pub maturity: String,
    pub gate_evidence: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityVerification {
    pub id: String,
    pub command: String,
    pub proves: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityClaimGate {
    pub id: String,
    pub command: String,
    pub proves: String,
}

fn default_required_for_verified() -> bool {
    true
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityClaim {
    pub id: String,
    #[serde(default)]
    pub user_story: String,
    #[serde(default = "default_required_for_verified")]
    pub required_for_verified: bool,
    pub maturity: CapabilityMaturity,
    #[serde(default)]
    pub oracle: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixtures: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub negative_cases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gates: Vec<CapabilityClaimGate>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityVerificationContract {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_maturity: Vec<CapabilityMaturity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claims: Vec<CapabilityClaim>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub full_regenerability_required: bool,
}

/// Exposed product interface for a capability. CLI is a surface, not a
/// capability type; surfaces usually feed the behavior EC dimension.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilitySurface {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub verification: String,
}

/// EC dimension key as declared by a capability contract.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityEcDimensionKind {
    Behavior,
    Efficiency,
    Security,
    Stability,
    Content,
}

impl CapabilityEcDimensionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityEcDimensionKind::Behavior => "behavior",
            CapabilityEcDimensionKind::Efficiency => "efficiency",
            CapabilityEcDimensionKind::Security => "security",
            CapabilityEcDimensionKind::Stability => "stability",
            CapabilityEcDimensionKind::Content => "content",
        }
    }
}

/// Hand-authored slot for an aw-generated efficiency section. The measured
/// pivot/cube data is backfilled by `aw ec`; README authors only declare which
/// slice to render and where the cube record lives.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityEfficiencyBackfillSlot {
    pub operating_point: String,
    pub cube: String,
}

/// Declared EC dimension content for a capability.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityEcDimension {
    pub dimension: CapabilityEcDimensionKind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub runner: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_for_production: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub efficiency_backfill: Option<CapabilityEfficiencyBackfillSlot>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityEvidence {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub td: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cb: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification: Vec<CapabilityVerification>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CapabilityYaml {
    pub id: String,
    pub status: CapabilityStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_type: Option<CapabilityType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surfaces: Vec<CapabilitySurface>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ec_dimensions: Vec<CapabilityEcDimension>,
    pub promise: String,
    pub current_state: String,
    #[serde(default)]
    pub gaps: Vec<CapabilityGap>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_contract: Option<CapabilityVerificationContract>,
    #[serde(default)]
    pub evidence: CapabilityEvidence,
    #[serde(default)]
    pub done_when: Vec<String>,
    #[serde(default)]
    pub out_of_scope: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilitySection {
    pub title: String,
    pub id: String,
    pub status: CapabilityStatus,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prelude: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub postlude: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_summary: Option<CapabilityIndexSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_type: Option<CapabilityType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surfaces: Vec<CapabilitySurface>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ec_dimensions: Vec<CapabilityEcDimension>,
    pub promise: String,
    pub current_state: String,
    pub gaps: Vec<CapabilityGap>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub work_roots: Vec<CapabilityWorkRoot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_contract: Option<CapabilityVerificationContract>,
    pub evidence: CapabilityEvidence,
    pub done_when: Vec<String>,
    pub out_of_scope: Vec<String>,
    #[serde(default)]
    pub release_scope: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    pub line: usize,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl CapabilitySection {
    fn from_yaml(title: String, line: usize, yaml: CapabilityYaml) -> Self {
        Self {
            title,
            id: yaml.id,
            status: yaml.status,
            prelude: String::new(),
            postlude: String::new(),
            index_summary: None,
            capability_type: yaml.capability_type,
            surfaces: yaml.surfaces,
            ec_dimensions: yaml.ec_dimensions,
            promise: yaml.promise,
            current_state: yaml.current_state,
            gaps: yaml.gaps,
            work_roots: Vec::new(),
            verification_contract: yaml.verification_contract,
            evidence: yaml.evidence,
            done_when: yaml.done_when,
            out_of_scope: yaml.out_of_scope,
            release_scope: false,
            dependencies: yaml.dependencies,
            line,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LegacyCapabilityRow {
    pub capability: String,
    pub current_state: String,
    pub gaps: String,
    pub active_wi: String,
    pub evidence: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityProseCandidate {
    pub id: String,
    pub title: String,
    pub line: usize,
    pub root_wi: Option<String>,
    pub summary: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityDocument {
    pub cap_path: PathBuf,
    pub format: CapabilityDocumentFormat,
    pub needs_canonicalization: bool,
    pub capabilities: Vec<CapabilitySection>,
    pub legacy_rows: Vec<LegacyCapabilityRow>,
    #[serde(skip)]
    pub prose_candidates: Vec<CapabilityProseCandidate>,
    pub findings: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDocumentFormat {
    Empty,
    MarkdownTables,
    YamlSections,
    LegacyTable,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl CapabilityDocument {
    pub fn is_legacy_only(&self) -> bool {
        self.capabilities.is_empty() && !self.legacy_rows.is_empty()
    }

    pub fn requires_format_migration(&self) -> bool {
        matches!(
            self.format,
            CapabilityDocumentFormat::YamlSections | CapabilityDocumentFormat::LegacyTable
        ) || self.needs_canonicalization
    }

    pub fn format_version(&self) -> u8 {
        match self.format {
            CapabilityDocumentFormat::Empty => 0,
            CapabilityDocumentFormat::MarkdownTables if self.needs_canonicalization => 1,
            CapabilityDocumentFormat::MarkdownTables => 2,
            CapabilityDocumentFormat::YamlSections | CapabilityDocumentFormat::LegacyTable => 1,
        }
    }

    pub fn capability_ids(&self) -> BTreeSet<String> {
        self.capabilities
            .iter()
            .map(|capability| capability.id.clone())
            .collect()
    }

    pub fn gap_ids_for(&self, capability_id: &str) -> BTreeSet<String> {
        self.capabilities
            .iter()
            .find(|capability| capability.id == capability_id)
            .map(|capability| capability.gaps.iter().map(|gap| gap.id.clone()).collect())
            .unwrap_or_default()
    }

    pub fn claim_ids_for(&self, capability_id: &str) -> BTreeSet<String> {
        self.capabilities
            .iter()
            .find(|capability| capability.id == capability_id)
            .and_then(|capability| capability.verification_contract.as_ref())
            .map(|contract| {
                contract
                    .claims
                    .iter()
                    .map(|claim| claim.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn capability_has_contract(&self, capability_id: &str) -> bool {
        self.capabilities
            .iter()
            .find(|capability| capability.id == capability_id)
            .and_then(|capability| capability.verification_contract.as_ref())
            .is_some()
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl CapabilityDocumentFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityDocumentFormat::Empty => "empty",
            CapabilityDocumentFormat::MarkdownTables => "markdown_tables",
            CapabilityDocumentFormat::YamlSections => "yaml_sections",
            CapabilityDocumentFormat::LegacyTable => "legacy_table",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TdCapabilityRef {
    pub id: String,
    pub role: CapabilityRefRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim: Option<String>,
    pub coverage: CapabilityCoverage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TdFrontmatter {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    capability_scope: Option<String>,
    #[serde(default)]
    capability_refs: Vec<TdCapabilityRef>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TdCapabilityEvidence {
    pub spec_path: String,
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_status: Option<String>,
    pub capability_id: String,
    pub role: CapabilityRefRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim: Option<String>,
    pub coverage: CapabilityCoverage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityActionKind {
    DefineCapabilityMap,
    FormatMigrationRequired,
    HumanConfirmRequired,
    CreateWi,
    AtomizeWi,
    RunTd,
    RunCb,
    RunVerify,
    UpdateCapabilityStatus,
    EnvBlocked,
    StaleProjectConfig,
    DefineVerificationContract,
    LinkClaimVerification,
    AssignCapabilityType,
    None,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityAction {
    pub kind: CapabilityActionKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<String>,
    pub target: String,
    pub command: String,
    pub reason: String,
    pub requires_hitl: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hitl_question: Option<HitlQuestion>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HitlQuestion {
    pub id: String,
    pub question: String,
    pub target: String,
    pub resume_command: String,
    pub tool_hint: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<HitlChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_prompt: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HitlChoice {
    pub id: String,
    pub label: String,
    pub description: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VerificationRuntimeResult {
    pub id: String,
    pub command: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proves: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityRunResult {
    pub tick: usize,
    pub kind: CapabilityActionKind,
    pub command: String,
    pub executed_command: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hitl_question: Option<HitlQuestion>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityMigrateReport {
    pub schema_version: &'static str,
    pub action: &'static str,
    pub project: String,
    pub cap_path: PathBuf,
    pub status: String,
    pub changed: bool,
    pub result: CapabilityRunResult,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityDraftReport {
    pub schema_version: &'static str,
    pub action: &'static str,
    pub project: String,
    pub cap_path: PathBuf,
    pub path: PathBuf,
    pub status: String,
    pub source: &'static str,
    pub candidate_count: usize,
    pub agent_review_required: bool,
    pub review_status: &'static str,
    pub apply_command: String,
    pub check_command: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityApplyDraftReport {
    pub schema_version: &'static str,
    pub action: &'static str,
    pub project: String,
    pub cap_path: PathBuf,
    pub draft_path: PathBuf,
    pub status: String,
    pub changed: bool,
    pub capability_count: usize,
    pub check_command: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityWiEvidence {
    pub reference: String,
    pub gap_id: String,
    pub issue_type: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_command: Option<String>,
    pub title: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CapabilityClaimReport {
    pub id: String,
    pub user_story: String,
    pub required_for_verified: bool,
    pub maturity: CapabilityMaturity,
    pub oracle: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixtures: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub negative_cases: Vec<String>,
    pub gates: Vec<VerificationRuntimeResult>,
    pub verified: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CapabilityReportItem {
    pub id: String,
    pub title: String,
    pub status: CapabilityStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_type: Option<CapabilityType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surfaces: Vec<CapabilitySurface>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ec_dimensions: Vec<CapabilityEcDimension>,
    pub promise: String,
    pub current_state: String,
    pub gaps: Vec<CapabilityGap>,
    pub td_refs: Vec<TdCapabilityEvidence>,
    pub wi_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wi_evidence: Vec<CapabilityWiEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claims: Vec<CapabilityClaimReport>,
    pub claim_count: usize,
    pub verified_claim_count: usize,
    pub claim_percent: f64,
    pub verification: Vec<VerificationRuntimeResult>,
    pub verified: bool,
    pub release_scope: bool,
    pub full_regenerability_required: bool,
    pub dependencies: Vec<String>,
    pub dependency_closure: Vec<String>,
    pub production_ready: bool,
    pub production_blockers: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CapabilityReport {
    pub action: &'static str,
    pub project: String,
    pub cap_path: PathBuf,
    pub format_version: u8,
    pub status: String,
    pub test_gates: ProjectTestGateReport,
    pub production_ready: bool,
    pub production_status: ProductionStatus,
    pub production_scope: Vec<String>,
    pub production_blockers: Vec<String>,
    pub capability_count: usize,
    pub verified_count: usize,
    pub percent: f64,
    pub claim_count: usize,
    pub verified_claim_count: usize,
    pub claim_percent: f64,
    pub capabilities: Vec<CapabilityReportItem>,
    pub blockers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub next_action: CapabilityAction,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run_results: Vec<CapabilityRunResult>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilitySweepReport {
    pub schema_version: &'static str,
    pub action: &'static str,
    pub status: &'static str,
    pub project_count: usize,
    pub verified_project_count: usize,
    pub verify: bool,
    pub include_issue_inventory: bool,
    pub write_drafts: bool,
    pub write_wi_plans: bool,
    pub write_action_queue: bool,
    pub groups: Vec<CapabilitySweepGroup>,
    pub projects: Vec<CapabilitySweepProject>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drafts: Vec<CapabilityDraftReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_index_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wi_plans: Vec<crate::cli::issues::CapabilityWiPlanReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wi_plan_index_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_queue: Vec<CapabilityActionQueueEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_queue_index_path: Option<PathBuf>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityActionQueueEntry {
    pub project: String,
    pub action_kind: &'static str,
    pub action_group: String,
    pub target: String,
    pub command: String,
    pub reason: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilitySweepGroup {
    pub status: String,
    pub next_action_kind: &'static str,
    pub next_action_group: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action_detail: Option<&'static str>,
    pub count: usize,
    pub projects: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilitySweepProject {
    pub project: String,
    pub cap_path: PathBuf,
    pub report_status: String,
    pub loop_status: &'static str,
    pub format_version: u8,
    pub capability_count: usize,
    pub verified_count: usize,
    pub claim_count: usize,
    pub verified_claim_count: usize,
    pub production_ready: bool,
    pub requires_hitl: bool,
    pub blocker_count: usize,
    pub warning_count: usize,
    pub test_gate_status: ProjectTestGateStatus,
    pub test_gate_passed_count: usize,
    pub test_gate_command_count: usize,
    pub next_action_kind: &'static str,
    pub next_action_group: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action_detail: Option<&'static str>,
    pub next_action: CapabilityAction,
}

#[derive(Deserialize, Default)]
struct CapabilityConfig {
    #[serde(default)]
    projects: Vec<CapabilityProjectRow>,
}

#[derive(Deserialize, Default)]
struct CapabilityProjectRow {
    name: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    td_path: Option<String>,
    #[serde(default)]
    cap_path: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub async fn run(args: CapabilityArgs) -> Result<()> {
    let selected_project = args.project;
    match args.command {
        CapabilityCommand::Sweep(sweep_args) => {
            run_capability_sweep(selected_project.as_deref(), sweep_args).await
        }
        CapabilityCommand::Report(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            let report =
                build_capability_report(&project, args.cap_path.as_deref(), args.verify, true)
                    .await?;
            print_report(&report, args.human, args.pretty || args.json)?;
            Ok(())
        }
        CapabilityCommand::Next(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            let report =
                build_capability_report(&project, args.cap_path.as_deref(), false, true).await?;
            if args.human {
                print_next_action(&report.next_action);
            } else if args.pretty || args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&capability_summary(&report, false))?
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&capability_summary(&report, false))?
                );
            }
            Ok(())
        }
        CapabilityCommand::Draft(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            draft_capability_map(&project, args)
        }
        CapabilityCommand::ApplyDraft(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            apply_capability_draft(&project, args)
        }
        CapabilityCommand::Run(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            run_capability_tick(&project, args).await
        }
        CapabilityCommand::Migrate(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            migrate_capability_format(&project, args)
        }
        CapabilityCommand::Check(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            let mut report =
                build_capability_report(&project, args.cap_path.as_deref(), args.verify, false)
                    .await?;
            let check_failed = normalize_capability_check_report(&mut report);
            print_report(&report, args.human, args.pretty || args.json)?;
            if check_failed {
                std::process::exit(1);
            }
            Ok(())
        }
        CapabilityCommand::Init(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            init_capability_readme(&project, args)
        }
        CapabilityCommand::SetType(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            set_capability_type(&project, args)
        }
        CapabilityCommand::SetStatus(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            set_capability_status(&project, args)
        }
        CapabilityCommand::SetSurface(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            set_capability_surface(&project, args)
        }
        CapabilityCommand::SetEcDimension(args) => {
            let project = required_capability_project(selected_project.as_deref())?;
            set_capability_ec_dimension(&project, args)
        }
    }
}

fn required_capability_project(project: Option<&str>) -> Result<String> {
    project
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("capability requires --project <project>"))
}

fn normalize_capability_check_report(report: &mut CapabilityReport) -> bool {
    let check_failed = !report.blockers.is_empty()
        || matches!(report.test_gates.status, ProjectTestGateStatus::Failed)
        || report.next_action.kind == CapabilityActionKind::FormatMigrationRequired;
    if !check_failed {
        report.status = "healthy".to_string();
        report.next_action = CapabilityAction {
            kind: CapabilityActionKind::None,
            capability_id: None,
            gap_id: None,
            claim_id: None,
            target: report.cap_path.display().to_string(),
            command: String::new(),
            reason: "capability format and TD refs are valid".to_string(),
            requires_hitl: false,
            hitl_question: None,
        };
    }
    check_failed
}

fn init_capability_readme(project: &str, args: CapabilityInitArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let cap_path = resolve_capability_path(&project_root, project, args.cap_path.as_deref())?;
    if cap_path.exists() {
        anyhow::bail!(
            "capability map already exists at {}; use `aw capability migrate/check` instead",
            cap_path.display()
        );
    }
    let Some(parent) = cap_path.parent() else {
        anyhow::bail!(
            "capability map path {} has no parent directory",
            cap_path.display()
        );
    };
    if !parent.exists() {
        anyhow::bail!(
            "capability map parent directory does not exist: {}",
            parent.display()
        );
    }
    let title = args
        .title
        .clone()
        .unwrap_or_else(|| humanize_project_title(project));
    let brief = args.brief.clone().unwrap_or_else(|| {
        format!(
            "Capability map placeholder for `{project}`. Define project-specific product promises after human confirmation."
        )
    });
    let body = render_empty_capability_readme(&title, &brief);
    std::fs::write(&cap_path, body)
        .with_context(|| format!("write capability map {}", cap_path.display()))?;
    let payload = serde_json::json!({
        "action": "capability_init",
        "status": "created",
        "project": project,
        "cap_path": cap_path.display().to_string(),
        "title": title,
        "brief_confirmed": args.brief.is_some(),
        "check_command": format!("aw capability check --project {project}"),
    });
    if args.human {
        println!("{}", cap_path.display());
    } else if args.pretty || args.json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("{}", serde_json::to_string(&payload)?);
    }
    Ok(())
}

fn render_empty_capability_readme(title: &str, brief: &str) -> String {
    format!(
        "# {title}\n\n## Brief\n\n{brief}\n\n## Capabilities\n\n### Capability Index\n\n| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |\n|---|---:|---|---|---|---|---|\n"
    )
}

fn apply_capability_draft(project: &str, args: CapabilityApplyDraftArgs) -> Result<()> {
    if !args.reviewed {
        anyhow::bail!("refusing to apply draft without --reviewed; human confirmation is required before README mutation");
    }
    let project_root = crate::find_project_root()?;
    let cap_path = resolve_capability_path(&project_root, project, args.cap_path.as_deref())?;
    let check_command =
        capability_check_command(project, args.cap_path.as_ref().map(|_| cap_path.as_path()));
    let draft_body = std::fs::read_to_string(&args.draft)
        .with_context(|| format!("failed to read capability draft {}", args.draft.display()))?;
    let registry = extract_reviewed_draft_registry(&draft_body)?;
    let body = std::fs::read_to_string(&cap_path)
        .with_context(|| format!("failed to read capability map {}", cap_path.display()))?;
    let next_body = apply_capability_registry_to_readme(&body, &registry, project)?;
    let document = parse_capability_document(&next_body, &cap_path)
        .with_context(|| "applied draft does not parse as a capability map")?;
    if document.capabilities.is_empty() {
        anyhow::bail!("applied draft produced no capability contracts");
    }
    let changed = next_body != body;
    if changed {
        std::fs::write(&cap_path, next_body)
            .with_context(|| format!("failed to write capability map {}", cap_path.display()))?;
    }
    let report = CapabilityApplyDraftReport {
        schema_version: "aw.cli.v1",
        action: "capability_apply_draft",
        project: project.to_string(),
        cap_path,
        draft_path: args.draft,
        status: if changed { "applied" } else { "unchanged" }.to_string(),
        changed,
        capability_count: document.capabilities.len(),
        check_command,
    };

    if args.human {
        println!(
            "capability apply-draft: {} ({}) changed={} capabilities={}",
            report.project, report.status, report.changed, report.capability_count
        );
        println!("check: {}", report.check_command);
    } else if args.pretty || args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", serde_json::to_string(&report)?);
    }
    Ok(())
}

fn extract_reviewed_draft_registry(draft_body: &str) -> Result<String> {
    let registry = extract_draft_registry(draft_body)?;
    if registry.contains("(confirm") {
        anyhow::bail!(
            "draft still contains `(confirm ...)` placeholders; review it before applying"
        );
    }
    Ok(registry)
}

fn extract_draft_registry(draft_body: &str) -> Result<String> {
    let marker = "## Draft Canonical README Section";
    let Some(after_marker) = draft_body.split_once(marker).map(|(_, tail)| tail) else {
        anyhow::bail!("draft is missing `## Draft Canonical README Section`");
    };
    let Some((_, after_open)) = after_marker.split_once("```md") else {
        anyhow::bail!("draft canonical section must be fenced as ```md");
    };
    let Some((registry, _)) = after_open.split_once("```") else {
        anyhow::bail!("draft canonical section fence is not closed");
    };
    let registry = registry.trim();
    if !registry.starts_with("## Capabilities") {
        anyhow::bail!("draft canonical section must start with `## Capabilities`");
    }
    Ok(format!("{registry}\n"))
}

fn apply_capability_registry_to_readme(
    original_body: &str,
    registry: &str,
    project: &str,
) -> Result<String> {
    let mut body = ensure_canonical_readme_scaffold(original_body.to_string(), project);
    body = replace_capabilities_section(&body, registry)?;
    if !body.ends_with('\n') {
        body.push('\n');
    }
    Ok(body)
}

fn replace_capabilities_section(body: &str, registry: &str) -> Result<String> {
    let lines = body.lines().collect::<Vec<_>>();
    let fenced = markdown_fenced_line_mask(&lines);
    let Some(start) = lines.iter().enumerate().find_map(|(idx, line)| {
        if fenced[idx] {
            return None;
        }
        parse_heading(line)
            .filter(|(level, title)| *level == 2 && title.eq_ignore_ascii_case("Capabilities"))
            .map(|_| idx)
    }) else {
        anyhow::bail!("README scaffold is missing `## Capabilities`");
    };
    let end = (start + 1..lines.len())
        .find(|idx| {
            !fenced[*idx]
                && parse_heading(lines[*idx])
                    .map(|(level, _)| level <= 2)
                    .unwrap_or(false)
        })
        .unwrap_or(lines.len());

    let mut out = String::new();
    if start > 0 {
        out.push_str(&lines[..start].join("\n"));
        out.push_str("\n\n");
    }
    out.push_str(registry.trim_end());
    if end < lines.len() {
        out.push_str("\n\n");
        out.push_str(&lines[end..].join("\n"));
    }
    Ok(collapse_markdown_blank_runs_outside_fences(&out))
}

fn draft_capability_map(project: &str, args: CapabilityDraftArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let cap_path = resolve_capability_path(&project_root, project, args.cap_path.as_deref())?;
    let report = build_capability_draft_report(
        project,
        &cap_path,
        args.output.as_deref(),
        args.cap_path.as_ref().map(|_| cap_path.as_path()),
    )?;
    if args.human {
        println!("{}", report.path.display());
    } else if args.pretty || args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", serde_json::to_string(&report)?);
    }
    Ok(())
}

fn build_capability_draft_report(
    project: &str,
    cap_path: &Path,
    output: Option<&Path>,
    cap_path_override: Option<&Path>,
) -> Result<CapabilityDraftReport> {
    let cap_body = std::fs::read_to_string(cap_path)
        .with_context(|| format!("failed to read capability map {}", cap_path.display()))?;
    let document = parse_capability_document(&cap_body, cap_path)
        .with_context(|| format!("failed to parse capability map {}", cap_path.display()))?;
    if document.requires_format_migration() || !document.legacy_rows.is_empty() {
        anyhow::bail!(
            "legacy capability content found in {}; use `aw capability migrate` before drafting new roots",
            cap_path.display()
        );
    }
    if !document.capabilities.is_empty() {
        anyhow::bail!(
            "canonical capability contracts already found in {}; use `aw capability report/check` instead",
            cap_path.display()
        );
    }

    let body = render_capability_map_draft(project, cap_path, &document.prose_candidates);
    let path = write_capability_draft_artifact(project, output, &body)?;
    let apply_command = capability_apply_draft_command(project, &path, cap_path_override);
    let check_command = capability_check_command(project, cap_path_override);
    Ok(CapabilityDraftReport {
        schema_version: "aw.cli.v1",
        action: "capability_draft",
        project: project.to_string(),
        cap_path: cap_path.to_path_buf(),
        path,
        status: "pending_review".to_string(),
        source: capability_draft_source(&document.prose_candidates),
        candidate_count: document.prose_candidates.len(),
        agent_review_required: true,
        review_status: "pending",
        apply_command,
        check_command,
    })
}

fn capability_check_command(project: &str, cap_path_override: Option<&Path>) -> String {
    let mut command = format!("aw capability check --project {project}");
    if let Some(path) = cap_path_override {
        command.push_str(" --cap-path ");
        command.push_str(&shell_quote(&path.display().to_string()));
    }
    command
}

fn capability_apply_draft_command(
    project: &str,
    draft_path: &Path,
    cap_path_override: Option<&Path>,
) -> String {
    let mut command = format!(
        "aw capability apply-draft --project {project} --draft {}",
        shell_quote(&draft_path.display().to_string())
    );
    if let Some(path) = cap_path_override {
        command.push_str(" --cap-path ");
        command.push_str(&shell_quote(&path.display().to_string()));
    }
    command.push_str(" --reviewed");
    command
}

fn capability_draft_source(candidates: &[CapabilityProseCandidate]) -> &'static str {
    if candidates.is_empty() {
        "empty_capability_map"
    } else {
        "prose_candidates"
    }
}

fn write_capability_draft_artifact(
    project: &str,
    output: Option<&Path>,
    body: &str,
) -> Result<PathBuf> {
    let path = if let Some(path) = output {
        path.to_path_buf()
    } else {
        let stamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let dir = PathBuf::from("/tmp")
            .join("aw")
            .join(project)
            .join("capability-map-drafts");
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create {}", dir.display()))?;
        dir.join(format!(
            "{stamp}-{}-capability-map-draft.md",
            slugify(project)
        ))
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    std::fs::write(&path, body)
        .with_context(|| format!("failed to write capability draft {}", path.display()))?;
    Ok(path)
}

fn render_capability_map_draft(
    project: &str,
    cap_path: &Path,
    candidates: &[CapabilityProseCandidate],
) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("kind: capability_map_draft\n");
    out.push_str(&format!("project: {}\n", markdown_yaml_string(project)));
    out.push_str(&format!(
        "cap_path: {}\n",
        markdown_yaml_string(&cap_path.display().to_string())
    ));
    out.push_str("status: pending_review\n");
    out.push_str(&format!(
        "source: {}\n",
        capability_draft_source(candidates)
    ));
    out.push_str(&format!("candidate_count: {}\n", candidates.len()));
    out.push_str("---\n\n");
    out.push_str(&format!(
        "# {} Capability Map Draft\n\n",
        project_display_name(project)
    ));
    out.push_str(
        "This artifact is inference only. Review, revise, or defer these roots before copying any canonical contract into README.\n\n",
    );
    out.push_str("## Candidate Roots\n\n");
    out.push_str("| Candidate | Proposed ID | Root WI | Source Line | Summary |\n");
    out.push_str("|---|---|---:|---:|---|\n");
    if candidates.is_empty() {
        out.push_str("| (define capability root) | `(confirm-id)` | - | - | README has no candidate capability roots; human must define product promises. |\n");
    } else {
        for candidate in candidates {
            out.push_str(&format!(
                "| {} | `{}` | {} | {} | {} |\n",
                markdown_cell(&candidate.title),
                markdown_cell(&candidate.id),
                markdown_cell(candidate.root_wi.as_deref().unwrap_or("-")),
                candidate.line,
                markdown_cell(candidate.summary.as_deref().unwrap_or("-")),
            ));
        }
    }
    out.push_str("\n## Human Review Checklist\n\n");
    if candidates.is_empty() {
        out.push_str("- Define the project capability root(s) before editing README.\n");
    } else {
        out.push_str("- Confirm, rename, split, merge, or defer each candidate root.\n");
    }
    out.push_str("- Fill capability Type, public Surfaces, and EC Dimensions only after the product promise is confirmed.\n");
    out.push_str("- Replace `(confirm ...)` placeholders before writing to README.\n");
    out.push_str("- Run `aw capability check --project ");
    out.push_str(project);
    out.push_str("` after updating README.\n\n");
    out.push_str("## Draft Canonical README Section\n\n");
    out.push_str("```md\n");
    out.push_str(&render_candidate_capability_registry(project, candidates));
    out.push_str("```\n");
    out
}

fn render_candidate_capability_registry(
    project: &str,
    candidates: &[CapabilityProseCandidate],
) -> String {
    let mut out = String::new();
    out.push_str("## Capabilities\n\n");
    out.push_str("### Capability Index\n\n");
    out.push_str(
        "| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |\n",
    );
    out.push_str("|---|---:|---|---|---|---|---|\n");
    if candidates.is_empty() {
        out.push_str(&format!(
            "| {} Capability | - | planned | planned | smoke | not_ready | candidate |\n",
            markdown_cell(&project_display_name(project))
        ));
    } else {
        for candidate in candidates {
            out.push_str(&format!(
                "| {} | {} | planned | planned | smoke | not_ready | candidate from README prose; confirm promise |\n",
                markdown_cell(&candidate.title),
                markdown_cell(candidate.root_wi.as_deref().unwrap_or("-")),
            ));
        }
    }
    out.push('\n');
    if candidates.is_empty() {
        out.push_str(&render_empty_candidate_capability_section(project));
    } else {
        for candidate in candidates {
            out.push_str(&render_candidate_capability_section(candidate));
        }
    }
    out
}

fn render_empty_candidate_capability_section(project: &str) -> String {
    let title = format!("{} Capability", project_display_name(project));
    let id = slugify(&title);
    format!(
        "### {title}\n\nID: {id}\nType: (confirm capability type: AgentFirst, Service, Devops, DeveloperTool, RuntimeTool, or SecurityTool)\nSurfaces:\n- (confirm public surface, e.g. CLI: `command` - short summary)\nEC Dimensions:\n- (confirm EC dimension, e.g. behavior: `runner command` - contract summary)\nRoot WI: -\nStatus: candidate\nRequired Verification: smoke\nPromise:\n(confirm product promise)\nGate Inventory:\n- (confirm gate inventory)\n\n| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |\n|---|---|---:|---|---|---|---|\n| {work_root} root | epic | - | planned | planned | smoke | (confirm gate/evidence) |\n\n",
        title = markdown_cell(&title),
        id = markdown_cell(&id),
        work_root = markdown_cell(&title),
    )
}

fn render_candidate_capability_section(candidate: &CapabilityProseCandidate) -> String {
    let root_wi = candidate.root_wi.as_deref().unwrap_or("-");
    let promise = candidate
        .summary
        .as_deref()
        .filter(|summary| !summary.trim().is_empty())
        .unwrap_or("(confirm promise)");
    format!(
        "### {title}\n\nID: {id}\nType: (confirm capability type: AgentFirst, Service, Devops, DeveloperTool, RuntimeTool, or SecurityTool)\nSurfaces:\n- (confirm public surface, e.g. CLI: `command` - short summary)\nEC Dimensions:\n- (confirm EC dimension, e.g. behavior: `runner command` - contract summary)\nRoot WI: {root_wi}\nStatus: candidate\nRequired Verification: smoke\nPromise:\n{promise}\nGate Inventory:\n- (confirm gate inventory)\n\n| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |\n|---|---|---:|---|---|---|---|\n| {work_root} root | epic | {root_wi} | planned | planned | smoke | (confirm gate/evidence) |\n\n",
        title = candidate.title.trim(),
        id = candidate.id.trim(),
        root_wi = markdown_cell(root_wi),
        promise = promise.trim(),
        work_root = markdown_cell(&candidate.title),
    )
}

fn markdown_yaml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn humanize_project_title(project: &str) -> String {
    project
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!(
                "{}{}",
                first.to_uppercase().collect::<String>(),
                chars.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

async fn run_capability_sweep(
    selected_project: Option<&str>,
    args: CapabilitySweepArgs,
) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let rows = capability_sweep_project_rows(&project_root, selected_project)?;
    let include_issue_inventory = args.include_issue_inventory || !args.skip_issue_inventory;
    let mut reports = Vec::new();
    for row in rows {
        let project = row.name.clone();
        let report =
            match build_capability_report(&project, None, args.verify, include_issue_inventory)
                .await
            {
                Ok(report) => report,
                Err(err) => {
                    let cap_path = capability_path_from_row(&project_root, &row)
                        .unwrap_or_else(|_| project_root.join("README.md"));
                    capability_map_read_failure_report(
                        &project,
                        cap_path,
                        ProjectTestGateReport::not_evaluated(&project),
                        format!("failed to build capability report: {err:#}"),
                    )
                }
            };
        reports.push(report);
    }
    let mut sweep = capability_sweep_report(&reports, args.verify, include_issue_inventory);
    if args.write_drafts {
        sweep.write_drafts = true;
        sweep.drafts = write_capability_sweep_drafts(&sweep.projects)?;
        sweep.draft_index_path = write_capability_sweep_draft_index(&sweep.drafts)?;
    }
    if args.write_wi_plans {
        sweep.write_wi_plans = true;
        sweep.wi_plans = write_capability_sweep_wi_plans(&sweep.projects).await?;
        sweep.wi_plan_index_path = write_capability_sweep_wi_plan_index(&sweep.wi_plans)?;
    }
    if args.write_action_queue {
        sweep.write_action_queue = true;
        sweep.action_queue = capability_sweep_action_queue(&sweep.projects);
        sweep.action_queue_index_path =
            write_capability_sweep_action_queue_index(&sweep.action_queue)?;
    }
    if args.human {
        print_capability_sweep(&sweep);
    } else if args.pretty || args.json {
        println!("{}", serde_json::to_string_pretty(&sweep)?);
    } else {
        println!("{}", serde_json::to_string(&sweep)?);
    }
    Ok(())
}

fn capability_sweep_project_rows(
    project_root: &Path,
    selected_project: Option<&str>,
) -> Result<Vec<CapabilityProjectRow>> {
    if let Some(project) = selected_project {
        return resolve_project_row(project_root, project).map(|row| vec![row]);
    }
    Ok(
        crate::services::project_registry::load_project_config_rows(project_root)?
            .into_iter()
            .map(|row| CapabilityProjectRow {
                name: row.name,
                path: Some(row.path),
                td_path: row.td_path,
                cap_path: row.cap_path,
            })
            .collect(),
    )
}

fn capability_sweep_report(
    reports: &[CapabilityReport],
    verify: bool,
    include_issue_inventory: bool,
) -> CapabilitySweepReport {
    let projects = reports
        .iter()
        .map(capability_sweep_project)
        .collect::<Vec<_>>();
    let verified_project_count = reports
        .iter()
        .filter(|report| capability_workflow_complete(report))
        .count();
    let status = if verified_project_count == reports.len() {
        "done"
    } else if reports
        .iter()
        .any(|report| capability_loop_status(report) == "blocked")
    {
        "blocked"
    } else {
        "continue"
    };
    CapabilitySweepReport {
        schema_version: "aw.capability.sweep.v1",
        action: "capability_sweep",
        status,
        project_count: reports.len(),
        verified_project_count,
        verify,
        include_issue_inventory,
        write_drafts: false,
        write_wi_plans: false,
        write_action_queue: false,
        groups: capability_sweep_groups(&projects),
        projects,
        drafts: Vec::new(),
        draft_index_path: None,
        wi_plans: Vec::new(),
        wi_plan_index_path: None,
        action_queue: Vec::new(),
        action_queue_index_path: None,
    }
}

fn write_capability_sweep_drafts(
    projects: &[CapabilitySweepProject],
) -> Result<Vec<CapabilityDraftReport>> {
    capability_sweep_draft_projects(projects)
        .into_iter()
        .map(|project| {
            build_capability_draft_report(&project.project, &project.cap_path, None, None)
        })
        .collect()
}

fn write_capability_sweep_draft_index(drafts: &[CapabilityDraftReport]) -> Result<Option<PathBuf>> {
    if drafts.is_empty() {
        return Ok(None);
    }
    let stamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let dir = PathBuf::from("/tmp")
        .join("aw")
        .join("capability-map-drafts");
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = dir.join(format!("{stamp}-capability-map-draft-index.md"));
    std::fs::write(&path, render_capability_sweep_draft_index(drafts))
        .with_context(|| format!("failed to write capability draft index {}", path.display()))?;
    Ok(Some(path))
}

fn render_capability_sweep_draft_index(drafts: &[CapabilityDraftReport]) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("kind: capability_map_draft_index\n");
    out.push_str("status: pending_review\n");
    out.push_str(&format!("draft_count: {}\n", drafts.len()));
    out.push_str("---\n\n");
    out.push_str("# Capability Map Draft Review Index\n\n");
    out.push_str("These artifacts are inference only. Review, revise, or defer each root before copying any canonical contract into README.\n\n");
    out.push_str("| Project | Source | Candidates | Draft | Apply After Review | Check |\n");
    out.push_str("|---|---|---:|---|---|---|\n");
    for draft in drafts {
        out.push_str(&format!(
            "| {} | {} | {} | {} | `{}` | `{}` |\n",
            markdown_cell(&draft.project),
            draft.source,
            draft.candidate_count,
            markdown_cell(&draft.path.display().to_string()),
            markdown_cell(&draft.apply_command),
            markdown_cell(&draft.check_command),
        ));
    }
    out.push_str("\n## Review Guardrails\n\n");
    out.push_str("- Do not edit README until the capability promise is confirmed.\n");
    out.push_str(
        "- Replace `(confirm ...)` placeholders before publishing a capability contract.\n",
    );
    out.push_str("- Run each listed check command after README edits.\n");
    out
}

async fn write_capability_sweep_wi_plans(
    projects: &[CapabilitySweepProject],
) -> Result<Vec<crate::cli::issues::CapabilityWiPlanReport>> {
    let mut plans = Vec::new();
    for project in capability_sweep_wi_plan_projects(projects) {
        let report =
            crate::cli::issues::build_capability_wi_plan_report(crate::cli::issues::PlanArgs {
                project: Some(project.project.clone()),
                title: None,
                cap_path: Some(project.cap_path.clone()),
                output: None,
                json: false,
                repo: None,
            })
            .await?;
        plans.push(report);
    }
    Ok(plans)
}

fn write_capability_sweep_wi_plan_index(
    plans: &[crate::cli::issues::CapabilityWiPlanReport],
) -> Result<Option<PathBuf>> {
    if plans.is_empty() {
        return Ok(None);
    }
    let stamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let dir = PathBuf::from("/tmp").join("aw").join("capability-wi-plans");
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = dir.join(format!("{stamp}-capability-wi-plan-index.md"));
    std::fs::write(&path, render_capability_sweep_wi_plan_index(plans)).with_context(|| {
        format!(
            "failed to write capability WI plan index {}",
            path.display()
        )
    })?;
    Ok(Some(path))
}

fn render_capability_sweep_wi_plan_index(
    plans: &[crate::cli::issues::CapabilityWiPlanReport],
) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("kind: capability_wi_plan_index\n");
    out.push_str("status: pending_review\n");
    out.push_str(&format!("plan_count: {}\n", plans.len()));
    out.push_str("---\n\n");
    out.push_str("# Capability WI Plan Review Index\n\n");
    out.push_str("These artifacts are local review inputs. Do not publish tracker changes until a human accepts the WI candidates.\n\n");
    out.push_str("| Project | Backend | Candidates | Plan | Re-run |\n");
    out.push_str("|---|---|---:|---|---|\n");
    for plan in plans {
        out.push_str(&format!(
            "| {} | {} | {} | {} | `{}` |\n",
            markdown_cell(&plan.project),
            markdown_cell(&plan.backend),
            plan.candidate_count,
            markdown_cell(&plan.path.display().to_string()),
            markdown_cell(&plan.plan_command),
        ));
    }
    out.push_str("\n## Review Guardrails\n\n");
    out.push_str("- Treat the capability README as the confirmed anchor.\n");
    out.push_str("- Review candidates before using `aw wi draft init` or `aw wi create`.\n");
    out.push_str("- When the issue backend is unavailable, keep the artifact local/review-only.\n");
    out
}

fn capability_sweep_action_queue(
    projects: &[CapabilitySweepProject],
) -> Vec<CapabilityActionQueueEntry> {
    projects
        .iter()
        .filter(|project| is_capability_executable_action(&project.next_action))
        .map(|project| CapabilityActionQueueEntry {
            project: project.project.clone(),
            action_kind: project.next_action_kind,
            action_group: project.next_action_group.clone(),
            target: project.next_action.target.clone(),
            command: project.next_action.command.clone(),
            reason: project.next_action.reason.clone(),
        })
        .collect()
}

fn is_capability_executable_action(action: &CapabilityAction) -> bool {
    let command = action.command.trim();
    !action.requires_hitl
        && !command.is_empty()
        && !is_capability_draft_action(action)
        && !is_capability_wi_plan_action(action)
}

fn write_capability_sweep_action_queue_index(
    entries: &[CapabilityActionQueueEntry],
) -> Result<Option<PathBuf>> {
    if entries.is_empty() {
        return Ok(None);
    }
    let stamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let dir = PathBuf::from("/tmp")
        .join("aw")
        .join("capability-action-queue");
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = dir.join(format!("{stamp}-capability-action-queue.md"));
    std::fs::write(&path, render_capability_sweep_action_queue_index(entries)).with_context(
        || {
            format!(
                "failed to write capability action queue index {}",
                path.display()
            )
        },
    )?;
    Ok(Some(path))
}

fn render_capability_sweep_action_queue_index(entries: &[CapabilityActionQueueEntry]) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("kind: capability_action_queue\n");
    out.push_str("status: pending_execution\n");
    out.push_str(&format!("action_count: {}\n", entries.len()));
    out.push_str("---\n\n");
    out.push_str("# Capability Action Queue\n\n");
    out.push_str("These commands are non-HITL next actions that are not covered by the draft or WI-plan review queues. Execute one at a time and re-run `aw capability sweep --human` after each material change.\n\n");
    out.push_str("| Project | Action | Target | Command | Reason |\n");
    out.push_str("|---|---|---|---|---|\n");
    for entry in entries {
        out.push_str(&format!(
            "| {} | {} | {} | `{}` | {} |\n",
            markdown_cell(&entry.project),
            markdown_cell(&entry.action_group),
            markdown_cell(&entry.target),
            markdown_cell(&entry.command),
            markdown_cell(&entry.reason),
        ));
    }
    out.push_str("\n## Execution Guardrails\n\n");
    out.push_str("- Execute one command at a time; do not batch lifecycle mutations blindly.\n");
    out.push_str(
        "- Re-run `aw capability sweep --human` after each command to refresh the queue.\n",
    );
    out.push_str("- If a command mutates README, TD, CB, or source, run the relevant capability check before publishing.\n");
    out
}

fn capability_sweep_draft_projects(
    projects: &[CapabilitySweepProject],
) -> Vec<&CapabilitySweepProject> {
    projects
        .iter()
        .filter(|project| is_capability_draft_action(&project.next_action))
        .collect()
}

fn is_capability_draft_action(action: &CapabilityAction) -> bool {
    action.kind == CapabilityActionKind::DefineCapabilityMap
        && action.command.trim().starts_with("aw capability draft")
}

fn capability_sweep_wi_plan_projects(
    projects: &[CapabilitySweepProject],
) -> Vec<&CapabilitySweepProject> {
    projects
        .iter()
        .filter(|project| is_capability_wi_plan_action(&project.next_action))
        .collect()
}

fn is_capability_wi_plan_action(action: &CapabilityAction) -> bool {
    action.kind == CapabilityActionKind::CreateWi && action.command.trim().starts_with("aw wi plan")
}

fn capability_sweep_project(report: &CapabilityReport) -> CapabilitySweepProject {
    let next_action_kind = capability_action_kind_label(report.next_action.kind);
    let next_action_detail = capability_action_detail_label(&report.next_action);
    let next_action_group = capability_action_group_label(next_action_kind, next_action_detail);
    CapabilitySweepProject {
        project: report.project.clone(),
        cap_path: report.cap_path.clone(),
        report_status: report.status.clone(),
        loop_status: capability_loop_status(report),
        format_version: report.format_version,
        capability_count: report.capability_count,
        verified_count: report.verified_count,
        claim_count: report.claim_count,
        verified_claim_count: report.verified_claim_count,
        production_ready: report.production_ready,
        requires_hitl: report.next_action.requires_hitl
            || report.next_action.kind == CapabilityActionKind::EnvBlocked,
        blocker_count: report.blockers.len(),
        warning_count: report.warnings.len(),
        test_gate_status: report.test_gates.status,
        test_gate_passed_count: report.test_gates.passed_count,
        test_gate_command_count: report.test_gates.command_count,
        next_action_kind,
        next_action_group,
        next_action_detail,
        next_action: report.next_action.clone(),
    }
}

fn capability_sweep_groups(projects: &[CapabilitySweepProject]) -> Vec<CapabilitySweepGroup> {
    let mut grouped =
        BTreeMap::<(String, &'static str, String, Option<&'static str>), Vec<String>>::new();
    for project in projects {
        grouped
            .entry((
                project.report_status.clone(),
                project.next_action_kind,
                project.next_action_group.clone(),
                project.next_action_detail,
            ))
            .or_default()
            .push(project.project.clone());
    }
    grouped
        .into_iter()
        .map(
            |((status, next_action_kind, next_action_group, next_action_detail), projects)| {
                CapabilitySweepGroup {
                    status,
                    next_action_kind,
                    next_action_group,
                    next_action_detail,
                    count: projects.len(),
                    projects,
                }
            },
        )
        .collect()
}

fn capability_action_group_label(kind: &'static str, detail: Option<&'static str>) -> String {
    if let Some(detail) = detail {
        format!("{kind}:{detail}")
    } else {
        kind.to_string()
    }
}

fn capability_action_detail_label(action: &CapabilityAction) -> Option<&'static str> {
    if action.kind != CapabilityActionKind::DefineCapabilityMap {
        return None;
    }
    let command = action.command.trim();
    if command.starts_with("aw capability draft ") || command == "aw capability draft" {
        Some("draft")
    } else if command.starts_with("aw capability init ") || command == "aw capability init" {
        Some("init")
    } else {
        Some("report")
    }
}

fn capability_action_kind_label(kind: CapabilityActionKind) -> &'static str {
    match kind {
        CapabilityActionKind::DefineCapabilityMap => "define_capability_map",
        CapabilityActionKind::FormatMigrationRequired => "format_migration_required",
        CapabilityActionKind::HumanConfirmRequired => "human_confirm_required",
        CapabilityActionKind::CreateWi => "create_wi",
        CapabilityActionKind::AtomizeWi => "atomize_wi",
        CapabilityActionKind::RunTd => "run_td",
        CapabilityActionKind::RunCb => "run_cb",
        CapabilityActionKind::RunVerify => "run_verify",
        CapabilityActionKind::UpdateCapabilityStatus => "update_capability_status",
        CapabilityActionKind::EnvBlocked => "env_blocked",
        CapabilityActionKind::StaleProjectConfig => "stale_project_config",
        CapabilityActionKind::DefineVerificationContract => "define_verification_contract",
        CapabilityActionKind::LinkClaimVerification => "link_claim_verification",
        CapabilityActionKind::AssignCapabilityType => "assign_capability_type",
        CapabilityActionKind::None => "none",
    }
}

fn print_capability_sweep(sweep: &CapabilitySweepReport) {
    println!(
        "capability sweep: {} [{}/{} projects complete]",
        sweep.status, sweep.verified_project_count, sweep.project_count
    );
    for group in &sweep.groups {
        println!(
            "{}:{} [{}] {}",
            group.status,
            group.next_action_group,
            group.count,
            group.projects.join(", ")
        );
    }
    if sweep.write_drafts {
        println!("drafts written [{}]", sweep.drafts.len());
        if let Some(path) = &sweep.draft_index_path {
            println!("draft index {}", path.display());
        }
        for draft in &sweep.drafts {
            println!(
                "draft:{} {} {}",
                draft.project,
                draft.source,
                draft.path.display()
            );
        }
    }
    if sweep.write_wi_plans {
        println!("WI plans written [{}]", sweep.wi_plans.len());
        if let Some(path) = &sweep.wi_plan_index_path {
            println!("WI plan index {}", path.display());
        }
        for plan in &sweep.wi_plans {
            println!(
                "WI plan:{} candidates={} {}",
                plan.project,
                plan.candidate_count,
                plan.path.display()
            );
        }
    }
    if sweep.write_action_queue {
        println!("action queue [{}]", sweep.action_queue.len());
        if let Some(path) = &sweep.action_queue_index_path {
            println!("action queue index {}", path.display());
        }
        for entry in &sweep.action_queue {
            println!(
                "action:{} {} {}",
                entry.project, entry.action_group, entry.command
            );
        }
    }
}

/// Persist a capability's type into the README capability contract. This is the
/// direct (non-interactive) resume path for the `assign_capability_type` HITL
/// question: an agent answers by running `aw capability set-type` with the
/// chosen type, which updates the README `Type:` field. The sidecar type file
/// remains readable as migration fallback, but new answers are written to the
/// README because it is the primary source an agent reads first.
fn set_capability_type(project: &str, args: CapabilitySetTypeArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let capability_type = crate::cli::capability_type::CapabilityType::from_cli_str(&args.r#type)?;
    let cap_path = resolve_capability_path(&project_root, project, None)?;
    let content = std::fs::read_to_string(&cap_path)
        .with_context(|| format!("read capability map {}", cap_path.display()))?;
    let updated = upsert_capability_type_in_readme(&content, &args.capability, capability_type)
        .with_context(|| format!("update capability type in {}", cap_path.display()))?;
    std::fs::write(&cap_path, updated)
        .with_context(|| format!("write capability map {}", cap_path.display()))?;
    let payload = serde_json::json!({
        "action": "set_capability_type",
        "project": project,
        "capability_id": args.capability,
        "capability_type": capability_type.as_str(),
        "required_ec_dimensions":
            crate::cli::capability_type::required_ec_dimensions(&capability_type),
        "cap_path": cap_path.display().to_string(),
    });
    if args.pretty {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("{}", serde_json::to_string(&payload)?);
    }
    Ok(())
}

fn upsert_capability_type_in_readme(
    content: &str,
    capability_id: &str,
    capability_type: CapabilityType,
) -> Result<String> {
    upsert_capability_contract_field_in_readme(
        content,
        capability_id,
        "Type",
        "type",
        capability_type.as_str(),
        &["id"],
    )
}

fn set_capability_status(project: &str, args: CapabilitySetStatusArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let status = parse_capability_status_arg(&args.status)?;
    let cap_path = resolve_capability_path(&project_root, project, None)?;
    let content = std::fs::read_to_string(&cap_path)
        .with_context(|| format!("read capability map {}", cap_path.display()))?;
    let updated = upsert_capability_status_in_readme(&content, &args.capability, status)
        .with_context(|| format!("update capability status in {}", cap_path.display()))?;
    std::fs::write(&cap_path, updated)
        .with_context(|| format!("write capability map {}", cap_path.display()))?;
    let payload = serde_json::json!({
        "action": "set_capability_status",
        "project": project,
        "capability_id": args.capability,
        "status": status.as_str(),
        "cap_path": cap_path.display().to_string(),
    });
    if args.pretty {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("{}", serde_json::to_string(&payload)?);
    }
    Ok(())
}

fn upsert_capability_status_in_readme(
    content: &str,
    capability_id: &str,
    status: CapabilityStatus,
) -> Result<String> {
    let updated = upsert_capability_contract_field_in_readme(
        content,
        capability_id,
        "Status",
        "status",
        status.as_str(),
        &["rootwi", "type", "id"],
    )?;
    Ok(update_capability_index_for_status(
        &updated,
        capability_id,
        status,
    ))
}

fn update_capability_index_for_status(
    content: &str,
    capability_id: &str,
    status: CapabilityStatus,
) -> String {
    let verification_update = capability_index_verification_for_status(status);
    let production_update = capability_index_production_for_status(status);
    if verification_update.is_none() && production_update.is_none() {
        return content.to_string();
    }
    let mut lines = content
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    let borrowed = lines.iter().map(|line| line.as_str()).collect::<Vec<_>>();
    let mut cursor = 0;
    let mut replacements = Vec::<(usize, String)>::new();
    while cursor < borrowed.len() {
        let Some((headers, rows, next_cursor)) = parse_markdown_table_at(&borrowed, cursor) else {
            cursor += 1;
            continue;
        };
        let capability_idx = find_table_column(&headers, &["capability", "id"]);
        let verification_idx = find_table_column(&headers, &["verification"]);
        let production_idx = find_table_column(&headers, &["production"]);
        if let Some(capability_idx) = capability_idx {
            for (row_offset, row) in rows.iter().enumerate() {
                let row_id = slugify(&table_cell(row, capability_idx));
                if row_id != capability_id {
                    continue;
                }
                let mut updated_row = row.clone();
                while updated_row.len() < headers.len() {
                    updated_row.push(String::new());
                }
                if let (Some(idx), Some(value)) = (verification_idx, verification_update) {
                    updated_row[idx] = value.to_string();
                }
                if let (Some(idx), Some(value)) = (production_idx, production_update) {
                    updated_row[idx] = value.to_string();
                }
                replacements.push((
                    cursor + 2 + row_offset,
                    format!(
                        "| {} |",
                        updated_row
                            .iter()
                            .map(|cell| markdown_cell(cell))
                            .collect::<Vec<_>>()
                            .join(" | ")
                    ),
                ));
            }
        }
        cursor = next_cursor;
    }
    drop(borrowed);
    for (line_idx, replacement) in replacements {
        if let Some(line) = lines.get_mut(line_idx) {
            *line = replacement;
        }
    }
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn capability_index_verification_for_status(status: CapabilityStatus) -> Option<&'static str> {
    match status {
        CapabilityStatus::Candidate | CapabilityStatus::Confirmed | CapabilityStatus::Auditing => {
            Some("planned")
        }
        CapabilityStatus::Blocked => Some("blocked"),
        CapabilityStatus::Verified => Some("verified"),
        CapabilityStatus::Retired => None,
    }
}

fn capability_index_production_for_status(status: CapabilityStatus) -> Option<&'static str> {
    match status {
        CapabilityStatus::Candidate | CapabilityStatus::Confirmed | CapabilityStatus::Auditing => {
            Some("not_ready")
        }
        CapabilityStatus::Blocked => Some("blocked"),
        CapabilityStatus::Verified => Some("ready"),
        CapabilityStatus::Retired => Some("retired"),
    }
}

fn set_capability_surface(project: &str, args: CapabilitySetSurfaceArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let cap_path = resolve_capability_path(&project_root, project, None)?;
    let content = std::fs::read_to_string(&cap_path)
        .with_context(|| format!("read capability map {}", cap_path.display()))?;
    let updated = upsert_capability_surface_in_readme(
        &content,
        &args.capability,
        CapabilitySurface {
            kind: normalize_surface_kind(&args.kind),
            commands: args.commands.clone(),
            summary: args.summary.clone(),
            verification: String::new(),
        },
    )
    .with_context(|| format!("update capability surface in {}", cap_path.display()))?;
    std::fs::write(&cap_path, updated)
        .with_context(|| format!("write capability map {}", cap_path.display()))?;
    let payload = serde_json::json!({
        "action": "set_capability_surface",
        "project": project,
        "capability_id": args.capability,
        "surface": {
            "kind": normalize_surface_kind(&args.kind),
            "commands": args.commands,
            "summary": args.summary,
        },
        "cap_path": cap_path.display().to_string(),
    });
    if args.pretty {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("{}", serde_json::to_string(&payload)?);
    }
    Ok(())
}

fn upsert_capability_surface_in_readme(
    content: &str,
    capability_id: &str,
    surface: CapabilitySurface,
) -> Result<String> {
    let document = parse_capability_document(content, Path::new("README.md"))?;
    let mut surfaces = document
        .capabilities
        .iter()
        .find(|capability| capability.id == capability_id)
        .map(|capability| capability.surfaces.clone())
        .ok_or_else(|| anyhow::anyhow!("capability `{capability_id}` not found"))?;
    let normalized_kind = normalize_table_token(&surface.kind);
    match surfaces
        .iter_mut()
        .find(|existing| normalize_table_token(&existing.kind) == normalized_kind)
    {
        Some(existing) => *existing = surface,
        None => surfaces.push(surface),
    }
    let value = render_surface_field_items(&surfaces).join("; ");
    upsert_capability_contract_field_in_readme(
        content,
        capability_id,
        "Surfaces",
        "surfaces",
        &value,
        &["type", "id"],
    )
}

fn set_capability_ec_dimension(project: &str, args: CapabilitySetEcDimensionArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let cap_path = resolve_capability_path(&project_root, project, None)?;
    let dimension = parse_ec_dimension_kind(&args.dimension)
        .ok_or_else(|| anyhow::anyhow!("unknown EC dimension `{}`", args.dimension))?;
    let content = std::fs::read_to_string(&cap_path)
        .with_context(|| format!("read capability map {}", cap_path.display()))?;
    let updated = upsert_capability_ec_dimension_in_readme(
        &content,
        &args.capability,
        CapabilityEcDimension {
            dimension,
            runner: args.runner.clone().unwrap_or_default(),
            summary: args.summary.clone().unwrap_or_default(),
            required_for_production: None,
            efficiency_backfill: if dimension == CapabilityEcDimensionKind::Efficiency {
                match (args.operating_point.as_ref(), args.cube.as_ref()) {
                    (None, None) => None,
                    _ => Some(CapabilityEfficiencyBackfillSlot {
                        operating_point: args.operating_point.clone().unwrap_or_default(),
                        cube: args.cube.clone().unwrap_or_default(),
                    }),
                }
            } else {
                None
            },
        },
    )
    .with_context(|| format!("update capability EC dimension in {}", cap_path.display()))?;
    std::fs::write(&cap_path, updated)
        .with_context(|| format!("write capability map {}", cap_path.display()))?;
    let payload = serde_json::json!({
        "action": "set_capability_ec_dimension",
        "project": project,
        "capability_id": args.capability,
        "dimension": dimension.as_str(),
        "runner": args.runner,
        "summary": args.summary,
        "operating_point": args.operating_point,
        "cube": args.cube,
        "cap_path": cap_path.display().to_string(),
    });
    if args.pretty {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("{}", serde_json::to_string(&payload)?);
    }
    Ok(())
}

fn upsert_capability_ec_dimension_in_readme(
    content: &str,
    capability_id: &str,
    dimension: CapabilityEcDimension,
) -> Result<String> {
    let document = parse_capability_document(content, Path::new("README.md"))?;
    let mut dimensions = document
        .capabilities
        .iter()
        .find(|capability| capability.id == capability_id)
        .map(|capability| capability.ec_dimensions.clone())
        .ok_or_else(|| anyhow::anyhow!("capability `{capability_id}` not found"))?;
    dimensions.retain(|existing| existing.dimension != dimension.dimension);
    let efficiency_backfill = dimension.efficiency_backfill.clone();
    dimensions.push(dimension);
    dimensions.sort_by_key(|dimension| dimension.dimension);
    let value = render_ec_dimension_field_items(&dimensions).join("; ");
    let mut updated = upsert_capability_contract_field_in_readme(
        content,
        capability_id,
        "EC Dimensions",
        "ecdimensions",
        &value,
        &["surfaces", "type", "id"],
    )?;
    if let Some(slot) = efficiency_backfill {
        if !slot.operating_point.trim().is_empty() {
            updated = upsert_capability_contract_field_in_readme(
                &updated,
                capability_id,
                "Efficiency Operating Point",
                "efficiencyoperatingpoint",
                &slot.operating_point,
                &["ecdimensions", "surfaces", "type", "id"],
            )?;
        }
        if !slot.cube.trim().is_empty() {
            updated = upsert_capability_contract_field_in_readme(
                &updated,
                capability_id,
                "Efficiency Cube",
                "efficiencycube",
                &slot.cube,
                &[
                    "efficiencyoperatingpoint",
                    "ecdimensions",
                    "surfaces",
                    "type",
                    "id",
                ],
            )?;
        }
        updated = upsert_capability_efficiency_backfill_section_in_readme(
            &updated,
            capability_id,
            &slot,
        )?;
    }
    Ok(updated)
}

fn upsert_capability_efficiency_backfill_section_in_readme(
    content: &str,
    capability_id: &str,
    slot: &CapabilityEfficiencyBackfillSlot,
) -> Result<String> {
    let mut lines = content.lines().map(str::to_string).collect::<Vec<_>>();
    let block_range = {
        let line_refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
        let fenced = markdown_fenced_line_mask(&line_refs);
        let mut idx = 0;
        let mut found = None;
        while idx < line_refs.len() {
            if fenced[idx] {
                idx += 1;
                continue;
            }
            let Some((level, _title)) = parse_heading(line_refs[idx]) else {
                idx += 1;
                continue;
            };
            if level < 2 {
                idx += 1;
                continue;
            }
            let block_end =
                next_capability_heading(&line_refs, idx + 1, level).unwrap_or(lines.len());
            if markdown_block_has_capability_id(
                &line_refs,
                &fenced,
                idx + 1,
                block_end,
                capability_id,
            ) {
                found = Some((idx + 1, block_end));
                break;
            }
            idx = block_end;
        }
        found
    };
    let Some((start, end)) = block_range else {
        anyhow::bail!("capability `{capability_id}` not found in README capability map")
    };
    upsert_efficiency_backfill_section_in_markdown_block(&mut lines, start, end, slot);
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

fn upsert_efficiency_backfill_section_in_markdown_block(
    lines: &mut Vec<String>,
    start: usize,
    end: usize,
    slot: &CapabilityEfficiencyBackfillSlot,
) {
    let section_range = {
        let line_refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
        let fenced = markdown_fenced_line_mask(&line_refs);
        let mut cursor = start;
        let mut found = None;
        while cursor < end {
            if fenced[cursor] {
                cursor += 1;
                continue;
            }
            let Some((_level, title)) = parse_heading(line_refs[cursor]) else {
                cursor += 1;
                continue;
            };
            let normalized = normalize_table_token(&title);
            if normalized != "efficiency" && !normalized.starts_with("efficiencygenerated") {
                cursor += 1;
                continue;
            }
            let section_end = next_heading(&line_refs, cursor + 1)
                .filter(|idx| *idx < end)
                .unwrap_or(end);
            found = Some((cursor, section_end));
            break;
        }
        found
    };
    if let Some((section_start, section_end)) = section_range {
        lines.splice(
            section_start..section_end,
            render_efficiency_backfill_section(slot),
        );
        return;
    }

    let mut section = render_efficiency_backfill_section(slot);
    if end > 0
        && lines
            .get(end.saturating_sub(1))
            .is_some_and(|line| !line.trim().is_empty())
    {
        section.insert(0, String::new());
    }
    lines.splice(end..end, section);
}

fn render_efficiency_backfill_section(slot: &CapabilityEfficiencyBackfillSlot) -> Vec<String> {
    vec![
        "#### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)".to_string(),
        String::new(),
        format!("Operating point: {}", slot.operating_point),
        format!("Cube: {}", slot.cube),
        String::new(),
    ]
}

fn upsert_capability_contract_field_in_readme(
    content: &str,
    capability_id: &str,
    field_label: &str,
    canonical_key: &str,
    value: &str,
    preferred_after: &[&str],
) -> Result<String> {
    let mut lines = content.lines().map(str::to_string).collect::<Vec<_>>();
    let line_refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
    let fenced = markdown_fenced_line_mask(&line_refs);
    let mut idx = 0;
    while idx < line_refs.len() {
        if fenced[idx] {
            idx += 1;
            continue;
        }
        let Some((level, _title)) = parse_heading(line_refs[idx]) else {
            idx += 1;
            continue;
        };
        if level < 2 {
            idx += 1;
            continue;
        }
        let block_end = next_capability_heading(&line_refs, idx + 1, level).unwrap_or(lines.len());
        if markdown_block_has_capability_id(&line_refs, &fenced, idx + 1, block_end, capability_id)
        {
            upsert_capability_field_in_markdown_block(
                &mut lines,
                &fenced,
                idx + 1,
                block_end,
                field_label,
                canonical_key,
                value,
                preferred_after,
            )?;
            let mut out = lines.join("\n");
            if content.ends_with('\n') {
                out.push('\n');
            }
            return Ok(out);
        }
        idx = block_end;
    }
    anyhow::bail!("capability `{capability_id}` not found in README capability map")
}

fn markdown_block_has_capability_id(
    lines: &[&str],
    fenced: &[bool],
    start: usize,
    end: usize,
    capability_id: &str,
) -> bool {
    let mut cursor = start;
    while cursor < end {
        if fenced[cursor] {
            cursor += 1;
            continue;
        }
        if parse_markdown_contract_field_line(lines[cursor].trim())
            .map(|(key, value)| key == "id" && value.trim() == capability_id)
            .unwrap_or(false)
        {
            return true;
        }
        if let Some((headers, rows, next_cursor)) = parse_markdown_table_at(lines, cursor) {
            if markdown_table_has_capability_id(&headers, &rows, capability_id) {
                return true;
            }
            cursor = next_cursor;
            continue;
        }
        cursor += 1;
    }
    false
}

fn markdown_table_has_capability_id(
    headers: &[String],
    rows: &[Vec<String>],
    capability_id: &str,
) -> bool {
    if let Some(indices) = markdown_contract_indices(headers) {
        return rows
            .iter()
            .any(|row| table_cell(row, indices.id).trim() == capability_id);
    }
    let Some(field_column) = find_table_column(headers, &["field", "property", "key"]) else {
        return false;
    };
    let Some(value_column) = find_table_column(headers, &["value"]) else {
        return false;
    };
    rows.iter().any(|row| {
        matches!(
            normalize_table_token(&table_cell(row, field_column)).as_str(),
            "id" | "capabilityid"
        ) && table_cell(row, value_column).trim() == capability_id
    })
}

fn upsert_capability_field_in_markdown_block(
    lines: &mut Vec<String>,
    fenced: &[bool],
    start: usize,
    end: usize,
    field_label: &str,
    canonical_key: &str,
    value: &str,
    preferred_after: &[&str],
) -> Result<()> {
    let field_line = format!("{field_label}: {value}");
    let mut insert_after = None;
    let mut cursor = start;
    while cursor < end {
        if fenced[cursor] {
            cursor += 1;
            continue;
        }
        if let Some((key, _value)) = parse_markdown_contract_field_line(lines[cursor].trim()) {
            if key == canonical_key {
                lines[cursor] = field_line;
                return Ok(());
            }
            if preferred_after.contains(&key.as_str()) {
                insert_after = Some(cursor);
            }
        }
        if let Some((headers, rows, next_cursor)) = parse_markdown_table_at(
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            cursor,
        ) {
            if let Some(inserted) = upsert_capability_field_in_contract_table(
                lines,
                cursor,
                &headers,
                &rows,
                field_label,
                canonical_key,
                value,
                preferred_after,
            ) {
                if inserted {
                    return Ok(());
                }
            }
            cursor = next_cursor;
            continue;
        }
        cursor += 1;
    }
    let insert_at = insert_after.map(|line| line + 1).unwrap_or(start);
    lines.insert(insert_at, field_line);
    Ok(())
}

fn upsert_capability_field_in_contract_table(
    lines: &mut Vec<String>,
    table_start: usize,
    headers: &[String],
    rows: &[Vec<String>],
    field_label: &str,
    canonical_key: &str,
    value: &str,
    preferred_after: &[&str],
) -> Option<bool> {
    if let (Some(field_column), Some(_value_column)) = (
        find_table_column(headers, &["field", "property", "key"]),
        find_table_column(headers, &["value"]),
    ) {
        let mut insert_after = None;
        for (row_offset, row) in rows.iter().enumerate() {
            let field = normalize_table_token(&table_cell(row, field_column));
            if field == canonical_key {
                let line_idx = table_start + 2 + row_offset;
                lines[line_idx] = format!("| {field_label} | {value} |");
                return Some(true);
            }
            if preferred_after.contains(&field.as_str()) {
                insert_after = Some(row_offset);
            }
        }
        if let Some(row_offset) = insert_after {
            lines.insert(
                table_start + 3 + row_offset,
                format!("| {field_label} | {value} |"),
            );
            return Some(true);
        }
        return Some(false);
    }
    if markdown_contract_indices(headers).is_some() {
        let insert_at = table_start + 2 + rows.len();
        lines.insert(insert_at, format!("{field_label}: {value}"));
        return Some(true);
    }
    None
}

async fn run_capability_tick(project: &str, args: CapabilityRunArgs) -> Result<()> {
    if !args.non_interactive {
        anyhow::bail!("aw capability run requires --non-interactive");
    }
    if args.max_ticks == 0 {
        anyhow::bail!("--max-ticks must be greater than zero");
    }

    let project_root = crate::find_project_root()?;
    let mut last_report =
        build_capability_report(project, args.cap_path.as_deref(), false, true).await?;
    let mut run_results = Vec::new();
    for tick in 1..=args.max_ticks {
        let action = last_report.next_action.clone();
        match action.kind {
            CapabilityActionKind::RunVerify => {
                let result = run_verification_command(&project_root, &action.command);
                if args.human {
                    eprintln!("capability verify: {} [{}]", result.command, result.status);
                }
                run_results.push(CapabilityRunResult {
                    tick,
                    kind: action.kind,
                    command: action.command,
                    executed_command: result.command,
                    status: result.status.clone(),
                    exit_code: result.exit_code,
                    stdout: None,
                    stderr: result.stderr.clone(),
                    hitl_question: None,
                });
                last_report =
                    build_capability_report(project, args.cap_path.as_deref(), true, true).await?;
                if result.status != "pass" {
                    break;
                }
            }
            CapabilityActionKind::FormatMigrationRequired => {
                let result = apply_capability_format_migration_tick(
                    tick,
                    &project_root,
                    project,
                    args.cap_path.as_deref(),
                    &action,
                );
                if args.human {
                    eprintln!(
                        "capability migration: {} [{}]",
                        result.executed_command, result.status
                    );
                }
                let status = result.status.clone();
                run_results.push(result);
                last_report =
                    build_capability_report(project, args.cap_path.as_deref(), false, true).await?;
                if status != "pass" {
                    break;
                }
            }
            CapabilityActionKind::None => break,
            _ if action.requires_hitl => {
                run_results.push(CapabilityRunResult {
                    tick,
                    kind: action.kind,
                    command: action.command,
                    executed_command: String::new(),
                    status: "skipped_hitl_required".to_string(),
                    exit_code: None,
                    stdout: None,
                    stderr: Some(action.reason),
                    hitl_question: action.hitl_question,
                });
                break;
            }
            _ => {
                let result = run_next_action_command(&project_root, &action, tick);
                if args.human {
                    eprintln!(
                        "capability run: {} [{}]",
                        result.executed_command, result.status
                    );
                }
                let prior_command = action.command.clone();
                let prior_kind = action.kind;
                let status = result.status.clone();
                run_results.push(result);
                last_report =
                    build_capability_report(project, args.cap_path.as_deref(), false, true).await?;
                if status != "pass"
                    || last_report.next_action.kind == prior_kind
                        && last_report.next_action.command == prior_command
                {
                    break;
                }
            }
        }
    }

    last_report.run_results = run_results;
    if args.human {
        print_report(&last_report, true, args.pretty || args.json)?;
    } else if args.pretty || args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&capability_summary(&last_report, true))?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(&capability_summary(&last_report, true))?
        );
    }
    Ok(())
}

fn migrate_capability_format(project: &str, args: CapabilityMigrateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let cap_path = resolve_capability_path(&project_root, project, args.cap_path.as_deref())?;
    let action = CapabilityAction {
        kind: CapabilityActionKind::FormatMigrationRequired,
        capability_id: None,
        gap_id: None,
        claim_id: None,
        target: cap_path.display().to_string(),
        command: format!("aw capability migrate --project {project}"),
        reason: "rewrite capability map to canonical Markdown format".to_string(),
        requires_hitl: false,
        hitl_question: None,
    };
    let result = apply_capability_format_migration_tick(
        1,
        &project_root,
        project,
        args.cap_path.as_deref(),
        &action,
    );
    let changed = result
        .stdout
        .as_deref()
        .is_some_and(|stdout| stdout.starts_with("migrated "));
    let status = if result.status == "pass" {
        if changed {
            "migrated"
        } else {
            "unchanged"
        }
    } else {
        "blocked"
    };
    let report = CapabilityMigrateReport {
        schema_version: "aw.cli.v1",
        action: "capability_migrate",
        project: project.to_string(),
        cap_path,
        status: status.to_string(),
        changed,
        result,
    };

    if args.human {
        println!(
            "capability migrate: {} ({}) changed={}",
            report.project, report.status, report.changed
        );
        if let Some(stdout) = &report.result.stdout {
            println!("{stdout}");
        }
        if let Some(stderr) = &report.result.stderr {
            println!("error: {stderr}");
        }
    } else if args.pretty || args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", serde_json::to_string(&report)?);
    }

    if report.result.status != "pass" {
        std::process::exit(1);
    }
    Ok(())
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub async fn build_capability_report(
    project: &str,
    cap_path_override: Option<&Path>,
    verify: bool,
    include_issue_inventory: bool,
) -> Result<CapabilityReport> {
    build_capability_report_inner(
        project,
        cap_path_override,
        verify,
        include_issue_inventory,
        None,
    )
    .await
}

pub(crate) async fn build_capability_report_for_capability(
    project: &str,
    cap_path_override: Option<&Path>,
    verify: bool,
    include_issue_inventory: bool,
    capability_id: &str,
) -> Result<CapabilityReport> {
    build_capability_report_inner(
        project,
        cap_path_override,
        verify,
        include_issue_inventory,
        Some(capability_id),
    )
    .await
}

async fn build_capability_report_inner(
    project: &str,
    cap_path_override: Option<&Path>,
    verify: bool,
    include_issue_inventory: bool,
    production_capability_scope: Option<&str>,
) -> Result<CapabilityReport> {
    let project_root = crate::find_project_root()?;
    if verify {
        eprintln!("aw capability verify: running configured project test gates for `{project}`");
    }
    let test_gates = project_test_gate_report(project, &project_root, verify)?;
    let cap_path = resolve_capability_path(&project_root, project, cap_path_override)?;
    let cap_body = match std::fs::read_to_string(&cap_path) {
        Ok(body) => body,
        Err(err) => {
            return Ok(capability_map_read_failure_report(
                project,
                cap_path,
                test_gates,
                format!("failed to read capability map: {err}"),
            ));
        }
    };
    let document = parse_capability_document_repairing_previous_migration(&cap_body, &cap_path)
        .with_context(|| format!("failed to parse capability map from {}", cap_path.display()))?;
    let mut blockers = document.findings.clone();
    let mut warnings = Vec::new();
    let capability_types = {
        // README pillar grouping / explicit Type fields are primary. The
        // sidecar exists only as migration fallback and must not override
        // README because the README is what an agent reads first.
        let mut t = crate::cli::capability_type::load_capability_types_from_readme(&cap_path)
            .unwrap_or_default();
        for (id, ty) in
            crate::cli::capability_type::load_capability_types(&project_root).unwrap_or_default()
        {
            t.entry(id).or_insert(ty);
        }
        t
    };

    let mut issues = Vec::new();
    if include_issue_inventory && !document.is_legacy_only() {
        match load_project_issues(&project_root, project).await {
            Ok(found) => issues = found,
            Err(err) => warnings.push(format!("issue inventory unavailable: {err}")),
        }
    }

    let td_refs = if !document.capabilities.is_empty() {
        match collect_td_capability_refs(&project_root, project, &document) {
            Ok(refs) => refs,
            Err(err) => {
                blockers.push(format!("td capability scan unavailable: {err}"));
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let mut report_items = Vec::new();
    let mut verification_cache = VerificationCommandCache::default();
    for capability in &document.capabilities {
        let refs = td_refs
            .iter()
            .filter(|td| td.capability_id == capability.id)
            .cloned()
            .collect::<Vec<_>>();
        let wi_evidence = capability_wi_evidence(capability, &issues);
        let wi_refs = wi_evidence
            .iter()
            .map(|evidence| evidence.reference.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let claims = capability_claim_reports_with_cache(
            capability,
            &project_root,
            verify,
            &mut verification_cache,
        );
        let verification = capability_verification_results_with_cache(
            capability,
            &project_root,
            &claims,
            verify,
            &mut verification_cache,
        );
        let verified = capability_verified(capability, &claims, &verification, verify);
        if verify {
            blockers.extend(
                verification
                    .iter()
                    .filter(|result| result.status != "pass")
                    .map(|result| {
                        format!(
                            "verification failed for {}: {}",
                            capability.id, result.command
                        )
                    }),
            );
        }
        report_items.push(CapabilityReportItem {
            id: capability.id.clone(),
            title: capability.title.clone(),
            status: capability.status,
            capability_type: capability
                .capability_type
                .or_else(|| capability_types.get(&capability.id).copied()),
            surfaces: capability.surfaces.clone(),
            ec_dimensions: derive_report_ec_dimensions(capability, &capability_types),
            promise: capability.promise.clone(),
            current_state: capability.current_state.clone(),
            gaps: capability.gaps.clone(),
            td_refs: refs,
            wi_refs,
            wi_evidence,
            claim_count: claims
                .iter()
                .filter(|claim| claim.required_for_verified)
                .count(),
            verified_claim_count: claims
                .iter()
                .filter(|claim| claim.required_for_verified && claim.verified)
                .count(),
            claim_percent: percent(
                claims
                    .iter()
                    .filter(|claim| claim.required_for_verified && claim.verified)
                    .count(),
                claims
                    .iter()
                    .filter(|claim| claim.required_for_verified)
                    .count(),
            ),
            claims,
            verification,
            verified,
            release_scope: capability.release_scope,
            full_regenerability_required: capability_full_regenerability_required(capability),
            dependencies: capability.dependencies.clone(),
            dependency_closure: Vec::new(),
            production_ready: false,
            production_blockers: Vec::new(),
        });
    }

    let capability_count = if document.is_legacy_only() {
        document.legacy_rows.len()
    } else {
        report_items
            .iter()
            .filter(|item| item.status != CapabilityStatus::Retired)
            .count()
    };
    let verified_count = report_items
        .iter()
        .filter(|item| item.status != CapabilityStatus::Retired && item.verified)
        .count();
    let capability_percent = percent(verified_count, capability_count);
    let claim_count = report_items
        .iter()
        .filter(|item| item.status != CapabilityStatus::Retired)
        .map(|item| item.claim_count)
        .sum::<usize>();
    let verified_claim_count = report_items
        .iter()
        .filter(|item| item.status != CapabilityStatus::Retired)
        .map(|item| item.verified_claim_count)
        .sum::<usize>();
    let claim_percent = percent(verified_claim_count, claim_count);
    let production_readiness = capability_production_readiness(
        project,
        &report_items,
        verify,
        &test_gates,
        &blockers,
        production_capability_scope,
    )?;

    let mut report = CapabilityReport {
        action: "capability",
        project: project.to_string(),
        cap_path,
        format_version: document.format_version(),
        status: "healthy".to_string(),
        test_gates,
        production_ready: production_readiness.production_ready,
        production_status: production_readiness.production_status,
        production_scope: production_readiness.production_scope.clone(),
        production_blockers: production_readiness.production_blockers.clone(),
        capability_count,
        verified_count,
        percent: capability_percent,
        claim_count,
        verified_claim_count,
        claim_percent,
        capabilities: report_items,
        blockers,
        warnings,
        next_action: CapabilityAction {
            kind: CapabilityActionKind::None,
            capability_id: None,
            gap_id: None,
            claim_id: None,
            target: project.to_string(),
            command: String::new(),
            reason: "all non-retired capabilities are verified".to_string(),
            requires_hitl: false,
            hitl_question: None,
        },
        run_results: Vec::new(),
    };
    apply_production_readiness_to_items(&mut report.capabilities, &production_readiness);
    report.next_action = choose_next_action(&report, &document, &capability_types);
    if !report.blockers.is_empty()
        || report.next_action.kind != CapabilityActionKind::None
        || verified_count < capability_count
    {
        report.status = "blocked".to_string();
    }
    Ok(report)
}

fn capability_map_read_failure_report(
    project: &str,
    cap_path: PathBuf,
    test_gates: ProjectTestGateReport,
    reason: String,
) -> CapabilityReport {
    if let Some(parent) = missing_capability_map_parent(&cap_path) {
        return capability_map_stale_project_config_report(
            project,
            cap_path,
            test_gates,
            format!(
                "{reason}; configured capability map parent directory does not exist: {}",
                parent.display()
            ),
        );
    }
    capability_map_read_blocked_report(project, cap_path, test_gates, reason)
}

fn missing_capability_map_parent(cap_path: &Path) -> Option<PathBuf> {
    cap_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .filter(|parent| !parent.exists())
        .map(Path::to_path_buf)
}

fn capability_map_read_blocked_report(
    project: &str,
    cap_path: PathBuf,
    test_gates: ProjectTestGateReport,
    reason: String,
) -> CapabilityReport {
    let target = cap_path.display().to_string();
    let next_action = CapabilityAction {
        kind: CapabilityActionKind::EnvBlocked,
        capability_id: None,
        gap_id: None,
        claim_id: None,
        target: target.clone(),
        command: format!("aw capability report --project {project}"),
        reason: reason.clone(),
        requires_hitl: true,
        hitl_question: Some(capability_map_config_hitl_question(
            project, &target, &reason,
        )),
    };
    CapabilityReport {
        action: "capability",
        project: project.to_string(),
        cap_path,
        format_version: 0,
        status: "blocked".to_string(),
        test_gates,
        production_ready: false,
        production_status: ProductionStatus::NotEvaluated,
        production_scope: Vec::new(),
        production_blockers: vec![reason.clone()],
        capability_count: 0,
        verified_count: 0,
        percent: 0.0,
        claim_count: 0,
        verified_claim_count: 0,
        claim_percent: 0.0,
        capabilities: Vec::new(),
        blockers: vec![reason],
        warnings: Vec::new(),
        next_action,
        run_results: Vec::new(),
    }
}

fn capability_map_stale_project_config_report(
    project: &str,
    cap_path: PathBuf,
    test_gates: ProjectTestGateReport,
    reason: String,
) -> CapabilityReport {
    let target = cap_path.display().to_string();
    let next_action = CapabilityAction {
        kind: CapabilityActionKind::StaleProjectConfig,
        capability_id: None,
        gap_id: None,
        claim_id: None,
        target: target.clone(),
        command: format!("aw capability report --project {project}"),
        reason: reason.clone(),
        requires_hitl: true,
        hitl_question: Some(capability_map_config_hitl_question(
            project, &target, &reason,
        )),
    };
    CapabilityReport {
        action: "capability",
        project: project.to_string(),
        cap_path,
        format_version: 0,
        status: "blocked".to_string(),
        test_gates,
        production_ready: false,
        production_status: ProductionStatus::NotEvaluated,
        production_scope: Vec::new(),
        production_blockers: vec![reason.clone()],
        capability_count: 0,
        verified_count: 0,
        percent: 0.0,
        claim_count: 0,
        verified_claim_count: 0,
        claim_percent: 0.0,
        capabilities: Vec::new(),
        blockers: vec![reason],
        warnings: Vec::new(),
        next_action,
        run_results: Vec::new(),
    }
}

fn capability_verified(
    capability: &CapabilitySection,
    claims: &[CapabilityClaimReport],
    verification: &[VerificationRuntimeResult],
    verify: bool,
) -> bool {
    if capability.status == CapabilityStatus::Retired {
        return true;
    }
    if !verify {
        return false;
    }
    if capability.status != CapabilityStatus::Verified {
        return false;
    }
    if capability.gaps.iter().any(|gap| {
        !matches!(
            gap.status,
            CapabilityGapStatus::Closed | CapabilityGapStatus::Deferred
        )
    }) {
        return false;
    }
    let required_claims = claims
        .iter()
        .filter(|claim| claim.required_for_verified)
        .collect::<Vec<_>>();
    if !required_claims.is_empty() && !required_claims.iter().all(|claim| claim.verified) {
        return false;
    }
    verification.iter().all(|result| result.status == "pass")
}

pub(crate) fn runtime_verified_by_id_from_sections(
    sections: &[CapabilitySection],
    project_root: &Path,
    verify: bool,
) -> BTreeMap<String, bool> {
    if !verify {
        return BTreeMap::new();
    }
    let mut verification_cache = VerificationCommandCache::default();
    sections
        .iter()
        .map(|capability| {
            let claims = capability_claim_reports_with_cache(
                capability,
                project_root,
                true,
                &mut verification_cache,
            );
            let verification = capability_verification_results_with_cache(
                capability,
                project_root,
                &claims,
                true,
                &mut verification_cache,
            );
            let verified = capability_verified(capability, &claims, &verification, true);
            (capability.id.clone(), verified)
        })
        .collect()
}

fn capability_full_regenerability_required(capability: &CapabilitySection) -> bool {
    capability
        .verification_contract
        .as_ref()
        .is_some_and(|contract| contract.full_regenerability_required)
}

fn capability_production_readiness(
    project: &str,
    items: &[CapabilityReportItem],
    verify: bool,
    test_gates: &ProjectTestGateReport,
    catalog_blockers: &[String],
    production_capability_scope: Option<&str>,
) -> Result<ProductionReadinessReport> {
    let mut global_blockers = catalog_blockers.to_vec();
    let production_gates_evaluated =
        verify && !matches!(test_gates.status, ProjectTestGateStatus::NotEvaluated);

    let regenerability_gap_count = if verify {
        let verified_by_id = items
            .iter()
            .map(|item| (item.id.clone(), item.verified))
            .collect::<BTreeMap<_, _>>();
        match super::project::build_health_report_with_test_gates_and_capability_verified(
            project,
            verify,
            verify,
            false,
            test_gates.clone(),
            production_gates_evaluated,
            Some(verified_by_id),
        ) {
            Ok(health) => {
                global_blockers.extend(health.blockers);
                health.regenerability_authority.gap_count
            }
            Err(err) => {
                global_blockers.push(format!("project production health unavailable: {err}"));
                0
            }
        }
    } else {
        global_blockers.push(
            test_gates
                .note
                .clone()
                .unwrap_or_else(|| "production gates not evaluated".to_string()),
        );
        0
    };

    let inputs = inputs_from_report_items(items);
    Ok(match production_capability_scope {
        Some(capability_id) if regenerability_gap_count == 0 => evaluate_capability_scope(
            inputs,
            capability_id,
            global_blockers,
            production_gates_evaluated,
        ),
        Some(capability_id) => {
            crate::cli::production::evaluate_capability_scope_with_regenerability(
                inputs,
                capability_id,
                global_blockers,
                production_gates_evaluated,
                regenerability_gap_count,
            )
        }
        None => crate::cli::production::evaluate_release_scope_with_regenerability(
            inputs,
            global_blockers,
            production_gates_evaluated,
            regenerability_gap_count,
        ),
    })
}

fn apply_production_readiness_to_items(
    items: &mut [CapabilityReportItem],
    production: &ProductionReadinessReport,
) {
    let readiness_by_id = production
        .capabilities
        .iter()
        .map(|readiness| (readiness.id.clone(), readiness))
        .collect::<BTreeMap<_, _>>();
    for item in items {
        if let Some(readiness) = readiness_by_id.get(&item.id) {
            item.dependency_closure = readiness.dependency_closure.clone();
            item.production_ready = readiness.production_ready;
            item.production_blockers = readiness.production_blockers.clone();
        }
    }
}

#[cfg(test)]
fn capability_claim_reports(
    capability: &CapabilitySection,
    project_root: &Path,
    verify: bool,
) -> Vec<CapabilityClaimReport> {
    let mut verification_cache = VerificationCommandCache::default();
    capability_claim_reports_with_cache(capability, project_root, verify, &mut verification_cache)
}

/// @spec .aw/tech-design/projects/agentic-workflow/specs/4124.md#logic
fn capability_claim_reports_with_cache(
    capability: &CapabilitySection,
    project_root: &Path,
    verify: bool,
    verification_cache: &mut VerificationCommandCache,
) -> Vec<CapabilityClaimReport> {
    let Some(contract) = capability.verification_contract.as_ref() else {
        return Vec::new();
    };
    contract
        .claims
        .iter()
        .map(|claim| {
            let gates = if verify {
                claim
                    .gates
                    .iter()
                    .map(|gate| {
                        verification_cache
                            .run(project_root, &gate.command)
                            .with_gate(&gate.id, &gate.proves)
                    })
                    .collect::<Vec<_>>()
            } else {
                claim
                    .gates
                    .iter()
                    .map(|gate| VerificationRuntimeResult {
                        id: gate.id.clone(),
                        command: gate.command.clone(),
                        status: "not_run".to_string(),
                        proves: Some(gate.proves.clone()),
                        exit_code: None,
                        stdout: None,
                        stderr: None,
                    })
                    .collect::<Vec<_>>()
            };
            let gate_verified =
                verify && !gates.is_empty() && gates.iter().all(|gate| gate.status == "pass");
            let fixture_verified = verify && claim_fixtures_verified(project_root, &claim.fixtures);
            let verified = if claim.required_for_verified {
                gate_verified || fixture_verified
            } else {
                true
            };
            CapabilityClaimReport {
                id: claim.id.clone(),
                user_story: claim.user_story.clone(),
                required_for_verified: claim.required_for_verified,
                maturity: claim.maturity,
                oracle: claim.oracle.clone(),
                fixtures: claim.fixtures.clone(),
                negative_cases: claim.negative_cases.clone(),
                gates,
                verified,
            }
        })
        .collect()
}

#[cfg(test)]
fn capability_verification_results(
    capability: &CapabilitySection,
    project_root: &Path,
    claims: &[CapabilityClaimReport],
    verify: bool,
) -> Vec<VerificationRuntimeResult> {
    let mut verification_cache = VerificationCommandCache::default();
    capability_verification_results_with_cache(
        capability,
        project_root,
        claims,
        verify,
        &mut verification_cache,
    )
}

/// @spec .aw/tech-design/projects/agentic-workflow/specs/4124.md#logic
fn capability_verification_results_with_cache(
    capability: &CapabilitySection,
    project_root: &Path,
    claims: &[CapabilityClaimReport],
    verify: bool,
    verification_cache: &mut VerificationCommandCache,
) -> Vec<VerificationRuntimeResult> {
    if capability.verification_contract.is_some() {
        return claims
            .iter()
            .flat_map(|claim| claim.gates.clone())
            .collect::<Vec<_>>();
    }
    if verify {
        return capability
            .evidence
            .verification
            .iter()
            .map(|gate| {
                verification_cache
                    .run(project_root, &gate.command)
                    .with_gate(&gate.id, &gate.proves)
            })
            .collect::<Vec<_>>();
    }
    Vec::new()
}

/// @spec .aw/tech-design/projects/agentic-workflow/specs/4124.md#logic
#[derive(Default)]
struct VerificationCommandCache {
    results: BTreeMap<String, VerificationRuntimeResult>,
}

/// @spec .aw/tech-design/projects/agentic-workflow/specs/4124.md#logic
impl VerificationCommandCache {
    fn run(&mut self, project_root: &Path, command: &str) -> VerificationRuntimeResult {
        if let Some(result) = self.results.get(command) {
            return result.clone();
        }
        eprintln!("aw capability verify: running `{command}`");
        let result = run_verification_command(project_root, command);
        self.results.insert(command.to_string(), result.clone());
        result
    }
}

fn claim_fixtures_verified(project_root: &Path, fixtures: &[String]) -> bool {
    let refs = fixture_refs(fixtures);
    !refs.is_empty()
        && refs
            .iter()
            .all(|reference| fixture_reference_exists(project_root, reference))
}

fn fixture_refs(fixtures: &[String]) -> Vec<String> {
    fixtures
        .iter()
        .flat_map(|fixture| fixture.split("<br>"))
        .flat_map(|fixture| fixture.lines())
        .map(|fixture| fixture.trim().trim_matches('`').to_string())
        .filter(|fixture| !is_empty_table_value(fixture))
        .collect()
}

fn fixture_reference_exists(project_root: &Path, reference: &str) -> bool {
    let path_part = reference.split('#').next().unwrap_or(reference).trim();
    if path_part.is_empty() {
        return false;
    }
    let path = Path::new(path_part);
    if path.is_absolute() {
        path.exists()
    } else {
        project_root.join(path).exists()
    }
}

fn percent(done: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (done as f64 / total as f64) * 100.0
    }
}

fn capability_wi_evidence(
    capability: &CapabilitySection,
    issues: &[Issue],
) -> Vec<CapabilityWiEvidence> {
    let mut evidence = Vec::new();
    for gap in &capability.gaps {
        let Some(active_wi) = gap.active_wi.as_deref() else {
            continue;
        };
        for number in extract_hash_numbers(active_wi) {
            if let Some(issue) = issues
                .iter()
                .find(|issue| issue.github_id.or(issue.gitlab_id) == Some(number))
            {
                let projection = workflow_guard::parse_projection(&issue.body);
                evidence.push(CapabilityWiEvidence {
                    reference: issue_ref(issue),
                    gap_id: gap.id.clone(),
                    issue_type: issue.issue_type.as_str().to_string(),
                    state: issue.state.as_str().to_string(),
                    phase: issue.phase.clone().or_else(|| {
                        projection
                            .as_ref()
                            .and_then(|projection| projection.active_phase.clone())
                    }),
                    expected_command: projection
                        .as_ref()
                        .and_then(|projection| projection.expected_command.clone()),
                    title: issue.title.clone(),
                });
            } else {
                evidence.push(CapabilityWiEvidence {
                    reference: format!("#{number}"),
                    gap_id: gap.id.clone(),
                    issue_type: "unknown".to_string(),
                    state: "unknown".to_string(),
                    phase: None,
                    expected_command: None,
                    title: String::new(),
                });
            }
        }
    }
    evidence
}

fn derive_report_ec_dimensions(
    capability: &CapabilitySection,
    capability_types: &BTreeMap<String, crate::cli::capability_type::CapabilityType>,
) -> Vec<CapabilityEcDimension> {
    let capability_type = capability
        .capability_type
        .or_else(|| capability_types.get(&capability.id).copied());
    let mut dimensions = capability
        .ec_dimensions
        .iter()
        .cloned()
        .map(|mut dimension| {
            dimension.required_for_production = capability_type.map(|ty| {
                crate::cli::capability_type::category_is_required_for_type(
                    &ty,
                    dimension.dimension.as_str(),
                ) && capability_declares_ec_dimension_content(capability, dimension.dimension)
            });
            dimension
        })
        .collect::<Vec<_>>();
    if let Some(capability_type) = capability_type {
        if !dimensions
            .iter()
            .any(|dimension| dimension.dimension == CapabilityEcDimensionKind::Behavior)
            && capability_declares_ec_dimension_content(
                capability,
                CapabilityEcDimensionKind::Behavior,
            )
        {
            dimensions.push(CapabilityEcDimension {
                dimension: CapabilityEcDimensionKind::Behavior,
                runner: String::new(),
                summary: "declared by behavior surfaces or verification contract".to_string(),
                required_for_production: Some(
                    crate::cli::capability_type::category_is_required_for_type(
                        &capability_type,
                        CapabilityEcDimensionKind::Behavior.as_str(),
                    ),
                ),
                efficiency_backfill: None,
            });
        }
    }
    dimensions
}

fn capability_declares_ec_dimension_content(
    capability: &CapabilitySection,
    dimension: CapabilityEcDimensionKind,
) -> bool {
    if capability
        .ec_dimensions
        .iter()
        .any(|declared| declared.dimension == dimension && ec_dimension_has_content(declared))
    {
        return true;
    }
    matches!(dimension, CapabilityEcDimensionKind::Behavior)
        && (!capability.surfaces.is_empty()
            || capability
                .verification_contract
                .as_ref()
                .is_some_and(verification_contract_has_content)
            || !capability.evidence.verification.is_empty())
}

fn ec_dimension_has_content(dimension: &CapabilityEcDimension) -> bool {
    !dimension.runner.trim().is_empty()
        || !dimension.summary.trim().is_empty()
        || dimension.efficiency_backfill.is_some()
}

fn verification_contract_has_content(contract: &CapabilityVerificationContract) -> bool {
    !contract.required_maturity.is_empty() || !contract.claims.is_empty()
}

fn choose_next_action(
    report: &CapabilityReport,
    document: &CapabilityDocument,
    capability_types: &BTreeMap<String, crate::cli::capability_type::CapabilityType>,
) -> CapabilityAction {
    if document.capabilities.is_empty() && document.legacy_rows.is_empty() {
        let reason = if document.prose_candidates.is_empty() {
            "README has no capability roots; human must define product promises before AW can migrate/check"
                .to_string()
        } else {
            "README has prose capability roots but no canonical capability contracts; human must confirm product promises before AW can migrate/check"
                .to_string()
        };
        return CapabilityAction {
            kind: CapabilityActionKind::DefineCapabilityMap,
            capability_id: None,
            gap_id: None,
            claim_id: None,
            target: report.cap_path.display().to_string(),
            command: format!("aw capability draft --project {}", report.project),
            reason: reason.clone(),
            requires_hitl: true,
            hitl_question: Some(capability_map_hitl_question(
                report,
                &reason,
                &document.prose_candidates,
            )),
        };
    }

    if document.requires_format_migration() {
        return CapabilityAction {
            kind: CapabilityActionKind::FormatMigrationRequired,
            capability_id: None,
            gap_id: None,
            claim_id: None,
            target: report.cap_path.display().to_string(),
            command: format!("aw capability migrate --project {}", report.project),
            reason: "README capability map needs canonical Markdown migration".to_string(),
            requires_hitl: false,
            hitl_question: None,
        };
    }

    for item in &report.capabilities {
        if item.status == CapabilityStatus::Candidate {
            let reason =
                "capability is still candidate and requires human confirmation".to_string();
            return CapabilityAction {
                kind: CapabilityActionKind::HumanConfirmRequired,
                capability_id: Some(item.id.clone()),
                gap_id: None,
                claim_id: None,
                target: report.cap_path.display().to_string(),
                command: format!("aw capability report --project {}", report.project),
                reason: reason.clone(),
                requires_hitl: true,
                hitl_question: Some(candidate_capability_hitl_question(report, item, &reason)),
            };
        }
    }

    for capability in &document.capabilities {
        if matches!(
            capability.status,
            CapabilityStatus::Confirmed
                | CapabilityStatus::Auditing
                | CapabilityStatus::Blocked
                | CapabilityStatus::Verified
        ) && capability.verification_contract.is_none()
        {
            return CapabilityAction {
                kind: CapabilityActionKind::DefineVerificationContract,
                capability_id: Some(capability.id.clone()),
                gap_id: None,
                claim_id: None,
                target: report.cap_path.display().to_string(),
                command: format!("aw capability check --project {}", report.project),
                reason: "non-candidate capability is missing verification_contract".to_string(),
                requires_hitl: true,
                hitl_question: Some(verification_contract_hitl_question(report, capability)),
            };
        }
    }

    // A capability's TYPE decides the ceiling for production-required EC
    // dimensions; README-declared dimension content opts into that ceiling.
    // If a real (non-candidate, non-retired) capability has no type assigned in
    // .aw/capability-types.toml, prompt for one so `aw ec` can derive the
    // production requirement for declared dimensions instead of falling back to
    // the YAML flag.
    for item in &report.capabilities {
        if matches!(
            item.status,
            CapabilityStatus::Candidate | CapabilityStatus::Retired
        ) {
            continue;
        }
        if !capability_types.contains_key(&item.id) {
            return CapabilityAction {
                kind: CapabilityActionKind::AssignCapabilityType,
                capability_id: Some(item.id.clone()),
                gap_id: None,
                claim_id: None,
                target: item.title.clone(),
                command: format!(
                    "aw capability set-type --project {} --capability {} --type <AgentFirst|Service|Devops|DeveloperTool|RuntimeTool|SecurityTool>",
                    report.project, item.id
                ),
                reason: "capability has no type assigned; required EC dimension ceiling cannot be derived"
                    .to_string(),
                requires_hitl: true,
                hitl_question: Some(capability_type_hitl_question(report, item)),
            };
        }
    }

    for item in &report.capabilities {
        for gap in &item.gaps {
            if matches!(
                gap.status,
                CapabilityGapStatus::Open
                    | CapabilityGapStatus::InProgress
                    | CapabilityGapStatus::Blocked
            ) {
                let active_issue = item
                    .wi_evidence
                    .iter()
                    .find(|evidence| evidence.gap_id == gap.id);
                let active_issue_is_epic =
                    active_issue.is_some_and(|issue| issue.issue_type == IssueType::Epic.as_str());
                let td_ref = td_ref_for_gap(item, &gap.id);
                let td_spec_path = td_ref.map(|td_ref| td_ref.spec_path.as_str());
                let td_review_status = td_ref.and_then(|td_ref| td_ref.review_status.as_deref());
                let (kind, command, reason) = match gap.active_wi.as_deref() {
                    Some(_active) if active_issue_is_epic => {
                        if let Some(action) = first_child_wi_action(report) {
                            return action;
                        }
                        if has_non_epic_wi_evidence(report) {
                            (
                                CapabilityActionKind::HumanConfirmRequired,
                                format!("aw capability report --project {} --verify", report.project),
                                "active WI is an epic and all known bounded child WIs are closed; aggregate readiness requires verification review before closing the top-level gap".to_string(),
                            )
                        } else {
                            (
                                CapabilityActionKind::AtomizeWi,
                                format!("aw wi atomize --project {}", report.project),
                                "active WI is an epic; atomize it before TD/CB lifecycle"
                                    .to_string(),
                            )
                        }
                    }
                    Some(active) if !active.trim().is_empty() => lifecycle_action_for_work_item(
                        report,
                        active.trim().trim_start_matches('#'),
                        active_issue,
                        td_spec_path,
                        td_review_status,
                        "active WI exists; continue WI -> TD -> CB lifecycle",
                    ),
                    _ => (
                        CapabilityActionKind::CreateWi,
                        format!("aw wi plan --project {}", report.project),
                        wi_plan_reason(report, "open capability gap has no active WI in README"),
                    ),
                };
                return CapabilityAction {
                    kind,
                    capability_id: Some(item.id.clone()),
                    gap_id: Some(gap.id.clone()),
                    claim_id: None,
                    target: item.title.clone(),
                    command: command.clone(),
                    reason: reason.clone(),
                    requires_hitl: kind == CapabilityActionKind::HumanConfirmRequired,
                    hitl_question: (kind == CapabilityActionKind::HumanConfirmRequired)
                        .then(|| epic_rollup_hitl_question(report, item, gap, &command, &reason)),
                };
            }
        }
    }

    for item in &report.capabilities {
        if item.status == CapabilityStatus::Retired {
            continue;
        }
        for claim in item
            .claims
            .iter()
            .filter(|claim| claim.required_for_verified)
        {
            let has_primary_td = item.td_refs.iter().any(|td| {
                td.role == CapabilityRefRole::Primary && td.claim.as_deref() == Some(&claim.id)
            });
            if !has_primary_td {
                return CapabilityAction {
                    kind: CapabilityActionKind::LinkClaimVerification,
                    capability_id: Some(item.id.clone()),
                    gap_id: None,
                    claim_id: Some(claim.id.clone()),
                    target: item.title.clone(),
                    command: format!("aw wi plan --project {}", report.project),
                    reason: wi_plan_reason(
                        report,
                        "required capability claim has no primary TD verification linkage",
                    ),
                    requires_hitl: false,
                    hitl_question: None,
                };
            }
        }
    }

    for item in &report.capabilities {
        if item.status == CapabilityStatus::Verified && !item.verified {
            if let Some((claim, gate)) = failing_claim_verification_gate(item) {
                return CapabilityAction {
                    kind: CapabilityActionKind::RunVerify,
                    capability_id: Some(item.id.clone()),
                    gap_id: None,
                    claim_id: Some(claim.id.clone()),
                    target: item.title.clone(),
                    command: gate.command.clone(),
                    reason: "required capability claim has a failing verification gate".to_string(),
                    requires_hitl: false,
                    hitl_question: None,
                };
            }
            if let Some(gate) = failing_capability_verification_gate(item) {
                return CapabilityAction {
                    kind: CapabilityActionKind::RunVerify,
                    capability_id: Some(item.id.clone()),
                    gap_id: None,
                    claim_id: None,
                    target: item.title.clone(),
                    command: gate.command.clone(),
                    reason: "capability has a failing verification gate".to_string(),
                    requires_hitl: false,
                    hitl_question: None,
                };
            }
        }
        if item.status == CapabilityStatus::Verified
            && !item.verified
            && runtime_verification_not_evaluated(item)
        {
            return CapabilityAction {
                kind: CapabilityActionKind::RunVerify,
                capability_id: Some(item.id.clone()),
                gap_id: None,
                claim_id: None,
                target: item.title.clone(),
                command: format!("aw capability report --project {} --verify", report.project),
                reason: "capability status is a catalog claim; runtime verification must be rerun for the current code".to_string(),
                requires_hitl: false,
                hitl_question: None,
            };
        }
        if item.status == CapabilityStatus::Verified
            && !item.verified
            && matches!(
                report.test_gates.status,
                ProjectTestGateStatus::NotEvaluated
            )
            && claim_inventory_verification_not_evaluated(item)
        {
            return CapabilityAction {
                kind: CapabilityActionKind::RunVerify,
                capability_id: Some(item.id.clone()),
                gap_id: None,
                claim_id: None,
                target: item.title.clone(),
                command: format!("aw capability report --project {} --verify", report.project),
                reason: "capability has fixture/inventory claim evidence that must be verified for the current code".to_string(),
                requires_hitl: false,
                hitl_question: None,
            };
        }
    }

    for item in &report.capabilities {
        if item.status == CapabilityStatus::Retired {
            continue;
        }
        if item.status != CapabilityStatus::Verified {
            if let Some(gate) = item.gaps.iter().find(|gap| {
                !matches!(
                    gap.status,
                    CapabilityGapStatus::Closed | CapabilityGapStatus::Deferred
                )
            }) {
                return CapabilityAction {
                    kind: CapabilityActionKind::RunTd,
                    capability_id: Some(item.id.clone()),
                    gap_id: Some(gate.id.clone()),
                    claim_id: None,
                    target: item.title.clone(),
                    command: format!("aw capability report --project {}", report.project),
                    reason: "capability still has open gaps".to_string(),
                    requires_hitl: false,
                    hitl_question: None,
                };
            }
            if let Some(gate) = item
                .verification
                .iter()
                .find(|result| result.status != "pass")
            {
                return CapabilityAction {
                    kind: CapabilityActionKind::RunVerify,
                    capability_id: Some(item.id.clone()),
                    gap_id: None,
                    claim_id: None,
                    target: item.title.clone(),
                    command: gate.command.clone(),
                    reason: "capability has a failing verification gate".to_string(),
                    requires_hitl: false,
                    hitl_question: None,
                };
            }
            let reason = "gaps are closed; verify and update capability status".to_string();
            return CapabilityAction {
                kind: CapabilityActionKind::UpdateCapabilityStatus,
                capability_id: Some(item.id.clone()),
                gap_id: None,
                claim_id: None,
                target: report.cap_path.display().to_string(),
                command: format!("aw capability report --project {} --verify", report.project),
                reason: reason.clone(),
                requires_hitl: true,
                hitl_question: Some(update_capability_status_hitl_question(
                    report, item, &reason,
                )),
            };
        }
    }

    CapabilityAction {
        kind: CapabilityActionKind::None,
        capability_id: None,
        gap_id: None,
        claim_id: None,
        target: report.project.clone(),
        command: String::new(),
        reason: "all non-retired capabilities are verified".to_string(),
        requires_hitl: false,
        hitl_question: None,
    }
}

fn wi_plan_reason(report: &CapabilityReport, base_reason: &str) -> String {
    if issue_inventory_unavailable(report) {
        format!(
            "{base_reason}; issue inventory unavailable, so `aw wi plan` must stay local/review-only before publishing tracker changes"
        )
    } else {
        base_reason.to_string()
    }
}

fn issue_inventory_unavailable(report: &CapabilityReport) -> bool {
    report
        .warnings
        .iter()
        .any(|warning| warning.starts_with("issue inventory unavailable:"))
}

fn runtime_verification_not_evaluated(item: &CapabilityReportItem) -> bool {
    let mut has_gate = false;
    for gate in item
        .claims
        .iter()
        .flat_map(|claim| claim.gates.iter())
        .chain(item.verification.iter())
    {
        has_gate = true;
        if gate.status != "not_run" {
            return false;
        }
    }
    has_gate
}

fn failing_claim_verification_gate(
    item: &CapabilityReportItem,
) -> Option<(&CapabilityClaimReport, &VerificationRuntimeResult)> {
    item.claims
        .iter()
        .filter(|claim| claim.required_for_verified)
        .find_map(|claim| {
            claim
                .gates
                .iter()
                .find(|gate| verification_gate_failed(gate))
                .map(|gate| (claim, gate))
        })
}

fn failing_capability_verification_gate(
    item: &CapabilityReportItem,
) -> Option<&VerificationRuntimeResult> {
    item.verification
        .iter()
        .find(|gate| verification_gate_failed(gate))
}

fn verification_gate_failed(gate: &VerificationRuntimeResult) -> bool {
    !matches!(gate.status.as_str(), "pass" | "not_run")
}

fn claim_inventory_verification_not_evaluated(item: &CapabilityReportItem) -> bool {
    item.claims.iter().any(|claim| {
        claim.required_for_verified
            && !claim.verified
            && claim.gates.is_empty()
            && !claim.fixtures.is_empty()
    })
}

fn hitl_choice(id: &str, label: &str, description: &str) -> HitlChoice {
    HitlChoice {
        id: id.to_string(),
        label: label.to_string(),
        description: description.to_string(),
    }
}

fn capability_hitl_question(
    id: String,
    question: String,
    target: String,
    reason: &str,
    project: &str,
    choices: Vec<HitlChoice>,
    default_choice: &str,
) -> HitlQuestion {
    HitlQuestion {
        id,
        question,
        target,
        resume_command: format!(
            "aw capability run --project {project} --non-interactive --max-ticks 1"
        ),
        tool_hint: "ask_user_question".to_string(),
        choices,
        default_choice: Some(default_choice.to_string()),
        freeform_prompt: Some(reason.to_string()),
    }
}

fn candidate_capability_hitl_question(
    report: &CapabilityReport,
    item: &CapabilityReportItem,
    reason: &str,
) -> HitlQuestion {
    capability_hitl_question(
        format!("capability:{}:confirm_candidate", item.id),
        format!(
            "Should capability `{}` be promoted from candidate to a confirmed product promise?",
            item.title
        ),
        item.title.clone(),
        reason,
        &report.project,
        vec![
            hitl_choice(
                "confirm_direction",
                "Confirm direction",
                "Treat this candidate as a real product capability and define its verification contract.",
            ),
            hitl_choice(
                "revise_promise",
                "Revise promise",
                "Keep the capability in review and adjust the promise, scope, or current-state text first.",
            ),
            hitl_choice(
                "defer_capability",
                "Defer capability",
                "Do not pursue this capability in the current completion loop.",
            ),
        ],
        "confirm_direction",
    )
}

fn capability_map_hitl_question(
    report: &CapabilityReport,
    reason: &str,
    candidates: &[CapabilityProseCandidate],
) -> HitlQuestion {
    let freeform_prompt = capability_map_freeform_prompt(reason, candidates);
    capability_hitl_question(
        "capability_map:define_roots".to_string(),
        format!(
            "What product capabilities should `{}` expose in its README capability map?",
            report.project
        ),
        report.cap_path.display().to_string(),
        reason,
        &report.project,
        vec![
            hitl_choice(
                "define_roots",
                "Define roots",
                "Provide the human-confirmed capability roots, promises, and external surfaces.",
            ),
            hitl_choice(
                "defer_project",
                "Defer project",
                "Do not add this project's capability map in the current completion loop.",
            ),
            hitl_choice(
                "fix_config",
                "Fix config",
                "Adjust project routing or cap_path before defining capabilities.",
            ),
        ],
        "define_roots",
    )
    .with_freeform_prompt(freeform_prompt)
}

impl HitlQuestion {
    fn with_freeform_prompt(mut self, prompt: String) -> Self {
        self.freeform_prompt = Some(prompt);
        self
    }
}

fn capability_map_freeform_prompt(reason: &str, candidates: &[CapabilityProseCandidate]) -> String {
    if candidates.is_empty() {
        return reason.to_string();
    }
    let mut prompt = String::from(reason);
    prompt.push_str("\n\nCandidate README capability roots detected for human confirmation:");
    for candidate in candidates.iter().take(8) {
        prompt.push_str("\n- ");
        prompt.push_str(&candidate.title);
        prompt.push_str(&format!(" (id: {}, line: {}", candidate.id, candidate.line));
        if let Some(root_wi) = candidate.root_wi.as_deref() {
            prompt.push_str(&format!(", WI: {}", root_wi));
        }
        prompt.push(')');
        if let Some(summary) = candidate.summary.as_deref() {
            prompt.push_str(": ");
            prompt.push_str(summary);
        }
    }
    if candidates.len() > 8 {
        prompt.push_str(&format!(
            "\n- ... {} additional candidate roots omitted",
            candidates.len() - 8
        ));
    }
    prompt.push_str(
        "\nThese candidates are inference only; confirm/revise/defer before writing canonical contracts.",
    );
    prompt
}

fn capability_map_config_hitl_question(project: &str, target: &str, reason: &str) -> HitlQuestion {
    capability_hitl_question(
        "capability_map:fix_config".to_string(),
        format!("How should `{project}` capability map routing be repaired?"),
        target.to_string(),
        reason,
        project,
        vec![
            hitl_choice(
                "fix_config",
                "Fix config",
                "Correct project path or cap_path before running capability checks.",
            ),
            hitl_choice(
                "create_readme",
                "Create README",
                "Create a README/capability map at the configured path after confirming the project exists.",
            ),
            hitl_choice(
                "defer_project",
                "Defer project",
                "Leave this configured project out of the current capability sweep.",
            ),
        ],
        "fix_config",
    )
}

fn verification_contract_hitl_question(
    report: &CapabilityReport,
    capability: &CapabilitySection,
) -> HitlQuestion {
    capability_hitl_question(
        format!("capability:{}:define_verification_contract", capability.id),
        format!(
            "What verification contract should make capability `{}` eligible for verified status?",
            capability.title
        ),
        capability.title.clone(),
        "non-candidate capability is missing verification_contract",
        &report.project,
        vec![
            hitl_choice(
                "define_contract",
                "Define contract",
                "Provide or approve concrete claims and gates for this capability.",
            ),
            hitl_choice(
                "revise_capability",
                "Revise capability",
                "Change the capability promise or status before defining a contract.",
            ),
            hitl_choice(
                "defer_capability",
                "Defer capability",
                "Mark this capability outside the current verified-completion target.",
            ),
        ],
        "define_contract",
    )
}

fn epic_rollup_hitl_question(
    report: &CapabilityReport,
    item: &CapabilityReportItem,
    gap: &CapabilityGap,
    command: &str,
    reason: &str,
) -> HitlQuestion {
    let mut question = capability_hitl_question(
        format!("capability:{}:{}:rollup_review", item.id, gap.id),
        format!(
            "Are the closed child work-items enough to close capability gap `{}` for `{}`?",
            gap.id, item.title
        ),
        item.title.clone(),
        reason,
        &report.project,
        vec![
            hitl_choice(
                "approve_rollup",
                "Approve rollup",
                "Accept the completed child work as sufficient evidence for the top-level gap.",
            ),
            hitl_choice(
                "atomize_more",
                "Atomize more",
                "Split or add bounded child work-items before closing the top-level gap.",
            ),
            hitl_choice(
                "revise_gap",
                "Revise gap",
                "Change the gap statement, refs, or evidence before proceeding.",
            ),
        ],
        "approve_rollup",
    );
    question.resume_command = command.to_string();
    question
}

fn update_capability_status_hitl_question(
    report: &CapabilityReport,
    item: &CapabilityReportItem,
    reason: &str,
) -> HitlQuestion {
    capability_hitl_question(
        format!("capability:{}:mark_verified", item.id),
        format!(
            "Should capability `{}` be marked verified based on closed gaps and passing gates?",
            item.title
        ),
        item.title.clone(),
        reason,
        &report.project,
        vec![
            hitl_choice(
                "mark_verified",
                "Mark verified",
                "Accept the current evidence and update the capability status to verified.",
            ),
            hitl_choice(
                "keep_auditing",
                "Keep auditing",
                "Leave the capability unverified and continue looking for missing work.",
            ),
            hitl_choice(
                "revise_evidence",
                "Revise evidence",
                "Adjust TD, WI, claim, or gate refs before status changes.",
            ),
        ],
        "mark_verified",
    )
}

fn capability_type_hitl_question(
    report: &CapabilityReport,
    item: &CapabilityReportItem,
) -> HitlQuestion {
    let mut question = capability_hitl_question(
        format!("capability:{}:assign_type", item.id),
        format!(
            "What capability type is `{}`? The type decides the production-required ceiling for declared EC dimensions (AgentFirst -> behavior; Service -> behavior+efficiency+security+stability; Devops -> behavior+stability; DeveloperTool/RuntimeTool -> behavior+efficiency+stability; SecurityTool -> behavior+security+stability).",
            item.title
        ),
        item.title.clone(),
        "capability has no type assigned in .aw/capability-types.toml; required EC dimension ceiling cannot be derived",
        &report.project,
        vec![
            hitl_choice(
                "agent_first",
                "AgentFirst",
                "Agent-facing capability. Only behavioral correctness is production-required.",
            ),
            hitl_choice(
                "service",
                "Service",
                "Externally-served capability. Behavior, efficiency, security, and stability are all production-required.",
            ),
            hitl_choice(
                "devops",
                "Devops",
                "Operational/devops capability. Behavior and stability are production-required.",
            ),
            hitl_choice(
                "developer_tool",
                "DeveloperTool",
                "Developer-facing toolchain capability. Behavior, efficiency, and stability are production-required.",
            ),
            hitl_choice(
                "runtime_tool",
                "RuntimeTool",
                "Runtime/tool execution capability. Behavior, efficiency, and stability are production-required.",
            ),
            hitl_choice(
                "security_tool",
                "SecurityTool",
                "Security evidence capability. Behavior, security, and stability are production-required.",
            ),
        ],
        "service",
    );
    question.resume_command = format!(
        "aw capability set-type --project {} --capability {} --type <AgentFirst|Service|Devops|DeveloperTool|RuntimeTool|SecurityTool> && aw capability run --project {} --non-interactive --max-ticks 1",
        report.project, item.id, report.project
    );
    question
}

fn has_non_epic_wi_evidence(report: &CapabilityReport) -> bool {
    report.capabilities.iter().any(|item| {
        item.wi_evidence.iter().any(|evidence| {
            evidence.issue_type != IssueType::Epic.as_str()
                && evidence.issue_type != "unknown"
                && !evidence.reference.trim().is_empty()
        })
    })
}

fn lifecycle_action_for_work_item(
    report: &CapabilityReport,
    work_item: &str,
    evidence: Option<&CapabilityWiEvidence>,
    td_spec_path: Option<&str>,
    td_review_status: Option<&str>,
    default_reason: &str,
) -> (CapabilityActionKind, String, String) {
    if let Some(command) = evidence
        .and_then(|evidence| evidence.expected_command.as_deref())
        .map(str::trim)
        .filter(|command| !command.is_empty())
    {
        return (
            action_kind_for_lifecycle_command(command),
            command.to_string(),
            "active WI has a workflow expected_command; follow lifecycle lock".to_string(),
        );
    }

    if lifecycle_issue_evidence_unresolved(evidence) {
        return (
            CapabilityActionKind::CreateWi,
            format!("aw wi plan --project {}", report.project),
            wi_plan_reason(
                report,
                "active WI reference is not present in project issue inventory; sync or recreate a bounded WI before TD/CB lifecycle",
            ),
        );
    }

    match evidence.and_then(|evidence| evidence.phase.as_deref()) {
        Some("td_reviewed") => (
            CapabilityActionKind::RunCb,
            cb_gen_command(work_item, td_spec_path),
            "active WI has reviewed TD; continue CB generation".to_string(),
        ),
        Some("cb_genned") | Some("cb_fill_in_progress") => (
            CapabilityActionKind::RunCb,
            format!("aw cb fill {work_item}"),
            "active WI has generated CB output; continue handwrite fill".to_string(),
        ),
        Some("cb_filled") => (
            CapabilityActionKind::RunCb,
            format!("aw cb review {work_item}"),
            "active WI has filled CB output; continue CB review".to_string(),
        ),
        Some("cb_reviewed") => (
            CapabilityActionKind::RunTd,
            format!("aw td merge {work_item}"),
            "active WI has reviewed CB output; merge TD/CB lifecycle".to_string(),
        ),
        Some("td_merged") => (
            CapabilityActionKind::RunVerify,
            format!("aw capability report --project {} --verify", report.project),
            "active WI has merged TD/CB lifecycle; verify capability readiness".to_string(),
        ),
        _ if td_review_status == Some("approved") && td_spec_path.is_some() => (
            CapabilityActionKind::RunCb,
            cb_gen_command(work_item, td_spec_path),
            "active WI has approved TD evidence; continue CB generation".to_string(),
        ),
        _ => (
            CapabilityActionKind::RunTd,
            format!("aw td create {work_item}"),
            default_reason.to_string(),
        ),
    }
}

fn lifecycle_issue_evidence_unresolved(evidence: Option<&CapabilityWiEvidence>) -> bool {
    match evidence {
        None => true,
        Some(evidence) => {
            evidence.issue_type == "unknown"
                || evidence.state == "unknown"
                || evidence.reference.trim().is_empty()
        }
    }
}

fn action_kind_for_lifecycle_command(command: &str) -> CapabilityActionKind {
    let command = command.trim();
    if command.starts_with("aw cb ") {
        CapabilityActionKind::RunCb
    } else if command.starts_with("aw capability report")
        || command.starts_with("aw capability check")
    {
        CapabilityActionKind::RunVerify
    } else if command.starts_with("aw td ") {
        CapabilityActionKind::RunTd
    } else {
        CapabilityActionKind::RunTd
    }
}

fn cb_gen_command(work_item: &str, td_spec_path: Option<&str>) -> String {
    match td_spec_path.map(str::trim).filter(|path| !path.is_empty()) {
        Some(path) => format!("aw cb gen {work_item} --spec-path {}", shell_quote(path)),
        None => format!("aw cb gen {work_item}"),
    }
}

fn td_ref_for_gap<'a>(
    item: &'a CapabilityReportItem,
    gap_id: &str,
) -> Option<&'a TdCapabilityEvidence> {
    item.td_refs
        .iter()
        .find(|td_ref| {
            td_ref.gap.as_deref() == Some(gap_id) && td_ref.role == CapabilityRefRole::Primary
        })
        .or_else(|| {
            item.td_refs
                .iter()
                .find(|td_ref| td_ref.gap.as_deref() == Some(gap_id))
        })
}

fn td_review_status_from_content(content: &str) -> Option<String> {
    let reviews = content.split_once("# Reviews")?.1;
    if reviews.contains("**Verdict:** approved") || reviews.contains("Verdict: approved") {
        Some("approved".to_string())
    } else if reviews.contains("**Verdict:** needs-revision")
        || reviews.contains("Verdict: needs-revision")
    {
        Some("needs_revision".to_string())
    } else {
        None
    }
}

fn first_child_wi_action(report: &CapabilityReport) -> Option<CapabilityAction> {
    for item in &report.capabilities {
        for gap in &item.gaps {
            if !matches!(
                gap.status,
                CapabilityGapStatus::Open
                    | CapabilityGapStatus::InProgress
                    | CapabilityGapStatus::Blocked
            ) {
                continue;
            }
            let Some(evidence) = item.wi_evidence.iter().find(|evidence| {
                evidence.gap_id == gap.id
                    && evidence.issue_type != IssueType::Epic.as_str()
                    && evidence.state == IssueState::Open.as_str()
            }) else {
                continue;
            };
            let work_item = evidence.reference.trim().trim_start_matches('#');
            if work_item.is_empty() {
                continue;
            }
            let td_ref = td_ref_for_gap(item, &gap.id);
            let (kind, command, reason) = lifecycle_action_for_work_item(
                report,
                work_item,
                Some(evidence),
                td_ref.map(|td_ref| td_ref.spec_path.as_str()),
                td_ref.and_then(|td_ref| td_ref.review_status.as_deref()),
                "bounded child WI exists; continue WI -> TD -> CB lifecycle",
            );
            return Some(CapabilityAction {
                kind,
                capability_id: Some(item.id.clone()),
                gap_id: Some(gap.id.clone()),
                claim_id: None,
                target: if evidence.title.is_empty() {
                    item.title.clone()
                } else {
                    evidence.title.clone()
                },
                command,
                reason,
                requires_hitl: false,
                hitl_question: None,
            });
        }
    }
    None
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn parse_capability_document(body: &str, cap_path: &Path) -> Result<CapabilityDocument> {
    let mut findings = Vec::new();
    let markdown_capabilities = parse_markdown_table_capability_sections(body)?;
    let needs_canonicalization = markdown_capability_document_needs_canonicalization(body);
    let yaml_capabilities = parse_h2_capability_sections(body)?;
    let legacy_rows = parse_legacy_capability_table(body);
    let prose_candidates = parse_capability_prose_candidates(body);
    let (format, mut capabilities) = if !markdown_capabilities.is_empty() {
        if !yaml_capabilities.is_empty() {
            findings.push(
                "YAML capability sections detected but ignored because Markdown capability tables are present"
                    .to_string(),
            );
        }
        (
            CapabilityDocumentFormat::MarkdownTables,
            markdown_capabilities,
        )
    } else if !yaml_capabilities.is_empty() {
        findings.push(
            "YAML capability sections detected; migrate README to Markdown capability tables"
                .to_string(),
        );
        (CapabilityDocumentFormat::YamlSections, yaml_capabilities)
    } else if !legacy_rows.is_empty() {
        (CapabilityDocumentFormat::LegacyTable, Vec::new())
    } else {
        if prose_candidates.is_empty() {
            findings.push(
                "no capability sections found; define README capability roots under ## Capabilities"
                    .to_string(),
            );
        } else {
            findings.push(
                "no canonical capability contracts found; confirm candidate README roots before migration/check"
                    .to_string(),
            );
        }
        (CapabilityDocumentFormat::Empty, Vec::new())
    };
    if capabilities.is_empty() && !legacy_rows.is_empty() {
        findings.push(
            "legacy capability table detected; migrate rows to Markdown capability sections"
                .to_string(),
        );
    }
    if capabilities.is_empty() && legacy_rows.is_empty() && !prose_candidates.is_empty() {
        let candidates = prose_candidates
            .iter()
            .take(8)
            .map(|candidate| {
                if let Some(root_wi) = candidate
                    .root_wi
                    .as_deref()
                    .filter(|root_wi| !candidate.title.contains(*root_wi))
                {
                    format!("{} ({})", candidate.title, root_wi)
                } else {
                    candidate.title.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        findings.push(format!(
            "candidate prose capability roots detected for HITL confirmation: {}",
            candidates
        ));
    }

    let index_summaries = parse_capability_index_summaries(body);
    for capability in &mut capabilities {
        if let Some(summary) = index_summaries
            .get(&capability.id)
            .or_else(|| index_summaries.get(&slugify(&capability.title)))
        {
            capability.release_scope =
                normalize_table_token(&summary.production).starts_with("ready");
            capability.index_summary = Some(summary.clone());
        } else {
            capability.release_scope = false;
        }
    }

    let mut ids = BTreeSet::new();
    for capability in &capabilities {
        if !ids.insert(capability.id.clone()) {
            anyhow::bail!("duplicate capability id `{}`", capability.id);
        }
        findings.extend(validate_capability_contract(capability)?);
        let mut gap_ids = BTreeSet::new();
        for gap in &capability.gaps {
            if !gap_ids.insert(gap.id.clone()) {
                anyhow::bail!(
                    "duplicate gap id `{}` in capability `{}`",
                    gap.id,
                    capability.id
                );
            }
        }
    }

    Ok(CapabilityDocument {
        cap_path: cap_path.to_path_buf(),
        format,
        needs_canonicalization,
        capabilities,
        legacy_rows,
        prose_candidates,
        findings,
    })
}

fn parse_capability_document_repairing_previous_migration(
    body: &str,
    cap_path: &Path,
) -> Result<CapabilityDocument> {
    match parse_capability_document(body, cap_path) {
        Ok(document) => Ok(document),
        Err(err) if err.to_string().contains("duplicate capability id") => {
            let Some(repaired) = strip_previous_canonical_capability_tail(body) else {
                return Err(err);
            };
            parse_capability_document(&repaired, cap_path)
        }
        Err(err) => Err(err),
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub(crate) fn render_capability_markdown_migration(
    original_body: &str,
    document: &CapabilityDocument,
    project: &str,
) -> String {
    let mut prefix = collapse_markdown_blank_runs_outside_fences(
        &strip_migrated_capability_sources(original_body),
    );
    prefix = prefix
        .replace(
            "Each `## Capability:` section is\nmachine-readable input for `aw capability`; summary tables are non-authoritative.",
            "Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.",
        )
        .replace(
            "Each `## Capability:` section is machine-readable input for `aw capability`; summary tables are non-authoritative.",
            "Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.",
        )
        .replace(
            "Any new Jet product claim starts by updating the relevant\n  `verification_contract` in this README.",
            "Any new Jet product claim starts by updating the relevant\n  capability table rows in this README.",
        );
    if prefix.trim().is_empty() {
        prefix = format!("# {}\n", project_display_name(project));
    }
    let (mut prefix, suffix) =
        if let Some((before, after)) = prefix.split_once(CAPABILITY_MIGRATION_INSERT_MARKER) {
            (before.to_string(), Some(after.to_string()))
        } else {
            (prefix, None)
        };
    prefix = ensure_canonical_readme_scaffold(prefix, project);
    let mut out = prefix.trim_end().to_string();
    out.push_str("\n\n");
    out.push_str(&render_capability_registry(document, project));
    if let Some(suffix) = suffix {
        let suffix = suffix.trim_start_matches('\n');
        if !suffix.trim().is_empty() {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
            out.push_str(suffix);
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn render_capability_registry(document: &CapabilityDocument, project: &str) -> String {
    let mut out = render_capability_index(document, project);
    if document.capabilities.is_empty() {
        for row in &document.legacy_rows {
            out.push_str(&render_legacy_capability_section(row));
        }
    } else {
        for capability in &document.capabilities {
            out.push_str(&render_markdown_capability_section(capability));
        }
    }
    out
}

fn ensure_canonical_readme_scaffold(mut prefix: String, project: &str) -> String {
    if !has_markdown_heading(&prefix, 1, None) {
        prefix = format!(
            "# {}\n\n{}",
            project_display_name(project),
            prefix.trim_start()
        );
    }
    if !has_markdown_heading(&prefix, 2, Some("Brief")) {
        prefix = insert_brief_heading_or_todo(prefix);
    }
    if !has_markdown_heading(&prefix, 2, Some("Capabilities")) {
        prefix.push_str(
            "\n\n## Capabilities\n\nMarkdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.\n",
        );
    }
    prefix
}

fn insert_brief_heading_or_todo(prefix: String) -> String {
    let lines = prefix.lines().collect::<Vec<_>>();
    let Some(h1_idx) = lines.iter().position(|line| {
        parse_heading(line)
            .map(|(level, _)| level == 1)
            .unwrap_or(false)
    }) else {
        let mut out = prefix.trim_end().to_string();
        out.push_str(
            "\n\n## Brief\n\n<!-- TODO: Add the human-confirmed project brief before publishing. -->\n",
        );
        return out;
    };
    let first_h2_idx = lines
        .iter()
        .enumerate()
        .skip(h1_idx + 1)
        .find_map(|(idx, line)| {
            parse_heading(line)
                .map(|(level, _)| level == 2)
                .unwrap_or(false)
                .then_some(idx)
        })
        .unwrap_or(lines.len());
    let mut lead_start = h1_idx + 1;
    while lead_start < first_h2_idx && lines[lead_start].trim().is_empty() {
        lead_start += 1;
    }
    let mut lead_end = first_h2_idx;
    while lead_end > lead_start && lines[lead_end - 1].trim().is_empty() {
        lead_end -= 1;
    }
    let lead = lines[lead_start..lead_end].join("\n");
    if lead.trim().is_empty() {
        let mut out = prefix.trim_end().to_string();
        out.push_str(
            "\n\n## Brief\n\n<!-- TODO: Add the human-confirmed project brief before publishing. -->\n",
        );
        return out;
    }

    let mut out = Vec::new();
    out.extend_from_slice(&lines[..=h1_idx]);
    out.push("");
    out.push("## Brief");
    out.push("");
    out.extend(lines[lead_start..lead_end].iter().copied());
    out.push("");
    out.extend(lines[first_h2_idx..].iter().copied());
    out.join("\n")
}

fn has_markdown_heading(body: &str, expected_level: usize, expected_title: Option<&str>) -> bool {
    body.lines().any(|line| {
        let Some((level, title)) = parse_heading(line) else {
            return false;
        };
        if level != expected_level {
            return false;
        }
        expected_title
            .map(|expected| title.eq_ignore_ascii_case(expected))
            .unwrap_or(true)
    })
}

fn strip_migrated_capability_sources(body: &str) -> String {
    let lines = body.lines().collect::<Vec<_>>();
    let fenced = markdown_fenced_line_mask(&lines);
    let mut out = Vec::new();
    let mut inserted_marker = false;
    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];
        if fenced[idx] {
            out.push(line);
            idx += 1;
            continue;
        }
        if is_capability_index_pillar_label(line) {
            if let Some(next_idx) = next_nonblank_markdown_line(&lines, idx + 1) {
                if !fenced[next_idx] {
                    if let Some((headers, _rows, _next_table_idx)) =
                        parse_markdown_table_at(&lines, next_idx)
                    {
                        if is_capability_index_table(&headers) {
                            idx += 1;
                            while idx < lines.len() && lines[idx].trim().is_empty() {
                                idx += 1;
                            }
                            continue;
                        }
                    }
                }
            }
        }
        if parse_markdown_table_row(line)
            .and_then(|cells| legacy_capability_column_indices(&cells))
            .is_some()
        {
            push_capability_insert_marker(&mut out, &mut inserted_marker);
            idx += 1;
            while idx < lines.len() {
                let Some(cells) = parse_markdown_table_row(lines[idx]) else {
                    break;
                };
                if !is_markdown_separator_row(&cells)
                    && legacy_capability_column_indices(&cells).is_none()
                    && cells.is_empty()
                {
                    break;
                }
                idx += 1;
            }
            continue;
        }

        if let Some((headers, _rows, next_idx)) = parse_markdown_table_at(&lines, idx) {
            if is_capability_index_table(&headers) {
                idx = next_idx;
                continue;
            }
        }

        if let Some((heading_level, title)) = parse_heading(line) {
            if (heading_level == 2 || heading_level == 3)
                && title.eq_ignore_ascii_case("Capability Index")
            {
                idx += 1;
                continue;
            }
            let block_end =
                next_heading_at_or_above(&lines, idx + 1, heading_level).unwrap_or(lines.len());
            let is_migrated_source = title.starts_with("Capability:")
                || (heading_level >= 2
                    && markdown_block_has_capability_contract(&lines, idx + 1, block_end));
            if is_migrated_source {
                push_capability_insert_marker(&mut out, &mut inserted_marker);
                idx = block_end;
                continue;
            }
        }

        out.push(line);
        idx += 1;
    }
    out.join("\n")
}

fn collapse_markdown_blank_runs_outside_fences(body: &str) -> String {
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut blank_run = 0usize;
    for line in body.lines() {
        let is_fence = is_markdown_fence_line(line);
        if is_fence {
            in_fence = !in_fence;
        }
        if !in_fence && line.trim().is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                out.push(line);
            }
            continue;
        }
        blank_run = 0;
        out.push(line);
    }
    out.join("\n")
}

fn push_capability_insert_marker<'a>(out: &mut Vec<&'a str>, inserted_marker: &mut bool) {
    if !*inserted_marker {
        out.push(CAPABILITY_MIGRATION_INSERT_MARKER);
        *inserted_marker = true;
    }
}

fn is_capability_index_table(headers: &[String]) -> bool {
    find_table_column(headers, &["capability"]).is_some()
        && find_table_column(headers, &["rootwi", "wi"]).is_some()
        && (find_table_column(headers, &["impl", "implementation"]).is_some()
            || find_table_column(headers, &["production"]).is_some())
}

fn is_capability_index_pillar_label(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("**Pillar")
}

fn next_nonblank_markdown_line(lines: &[&str], start: usize) -> Option<usize> {
    (start..lines.len()).find(|idx| !lines[*idx].trim().is_empty())
}

fn next_heading_at_or_above(lines: &[&str], start: usize, level: usize) -> Option<usize> {
    let fenced = markdown_fenced_line_mask(lines);
    (start..lines.len()).find(|idx| {
        !fenced[*idx]
            && parse_heading(lines[*idx])
                .map(|(candidate_level, _)| candidate_level <= level)
                .unwrap_or(false)
    })
}

fn render_capability_index(document: &CapabilityDocument, project: &str) -> String {
    let mut out = String::new();
    out.push_str("### Capability Index\n\n");
    out.push_str(
        "| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |\n",
    );
    out.push_str("|---|---:|---|---|---|---|---|\n");
    if document.capabilities.is_empty() {
        for row in &document.legacy_rows {
            out.push_str(&format!(
                "| {} | {} | planned | planned | smoke | not_ready | migrated from legacy table; confirm promise |\n",
                markdown_cell(&row.capability),
                markdown_cell(&row.active_wi),
            ));
        }
    } else {
        for capability in &document.capabilities {
            let fallback_maturity = capability_maturity_summary(capability);
            let implementation = capability
                .index_summary
                .as_ref()
                .map(|summary| summary.implementation.as_str())
                .unwrap_or_else(|| capability_impl_summary(capability));
            let verification = capability
                .index_summary
                .as_ref()
                .map(|summary| summary.verification.as_str())
                .unwrap_or_else(|| capability_verification_summary(capability));
            let maturity = capability
                .index_summary
                .as_ref()
                .map(|summary| summary.maturity.as_str())
                .unwrap_or(fallback_maturity.as_str());
            let production = capability
                .index_summary
                .as_ref()
                .map(|summary| summary.production.as_str())
                .unwrap_or_else(|| capability_production_summary(capability));
            let notes = capability
                .index_summary
                .as_ref()
                .map(|summary| summary.notes.as_str())
                .filter(|notes| !notes.trim().is_empty())
                .unwrap_or(&capability.promise);
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                markdown_cell(&capability.title),
                markdown_cell(&root_wi_for_capability(capability)),
                markdown_cell(implementation),
                markdown_cell(verification),
                markdown_cell(maturity),
                markdown_cell(production),
                markdown_cell(notes),
            ));
        }
    }
    if document.capabilities.is_empty() && document.legacy_rows.is_empty() {
        out.push_str(&format!(
            "| {} Capability | - | planned | planned | smoke | not_ready | candidate |\n",
            markdown_cell(&project_display_name(project))
        ));
    }
    out.push('\n');
    out
}

fn render_markdown_capability_section(capability: &CapabilitySection) -> String {
    let mut out = String::new();
    out.push_str(&format!("### {}\n\n", capability.title.trim()));
    if !capability.prelude.trim().is_empty() {
        out.push_str(capability.prelude.trim());
        out.push_str("\n\n");
    }
    out.push_str(&format!(
        "ID: {}\nRoot WI: {}\nStatus: {}\n",
        capability.id.trim(),
        root_wi_for_capability(capability),
        capability.status.as_str(),
    ));
    if let Some(capability_type) = capability.capability_type {
        out.push_str(&format!("Type: {}\n", capability_type.as_str()));
    }
    out.push_str(&format!(
        "Required Verification: {}\n",
        capability_maturity_summary(capability)
    ));
    out.push_str("Promise:\n");
    out.push_str(capability.promise.trim());
    out.push_str("\n");
    out.push_str("Gate Inventory:\n");
    for item in markdown_field_list_items(&capability_gate_inventory(capability)) {
        out.push_str(&format!("- {}\n", item));
    }
    if !capability.surfaces.is_empty() {
        out.push_str("Surfaces:\n");
        for item in render_surface_field_items(&capability.surfaces) {
            out.push_str(&format!("- {}\n", item));
        }
    }
    if !capability.ec_dimensions.is_empty() {
        out.push_str("EC Dimensions:\n");
        for item in render_ec_dimension_field_items(&capability.ec_dimensions) {
            out.push_str(&format!("- {}\n", item));
        }
    }
    if !capability.dependencies.is_empty() {
        out.push_str("Dependencies:\n");
        for dependency in &capability.dependencies {
            out.push_str(&format!("- {}\n", dependency));
        }
    }
    out.push('\n');
    out.push_str("| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |\n");
    out.push_str("|---|---|---:|---|---|---|---|\n");
    if !capability.work_roots.is_empty() {
        for row in &capability.work_roots {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                markdown_cell(&row.work_root),
                markdown_cell(&row.kind),
                markdown_cell(&row.wi),
                markdown_cell(&row.implementation),
                markdown_cell(&row.verification),
                markdown_cell(&row.maturity),
                markdown_cell(&row.gate_evidence),
            ));
        }
    } else if let Some(contract) = capability.verification_contract.as_ref() {
        for claim in &contract.claims {
            out.push_str(&format!(
                "| {} | epic | {} | {} | {} | {} | {} |\n",
                markdown_cell(&humanize_id(&claim.id)),
                markdown_cell(&root_wi_for_capability(capability)),
                markdown_cell(capability_impl_summary(capability)),
                markdown_cell(claim_verification_summary(capability, claim)),
                markdown_cell(claim.maturity.as_str()),
                markdown_cell(&claim_gate_evidence(claim)),
            ));
        }
    }
    if capability.work_roots.is_empty()
        && capability
            .verification_contract
            .as_ref()
            .map(|contract| contract.claims.is_empty())
            .unwrap_or(true)
    {
        for gap in &capability.gaps {
            out.push_str(&format!(
                "| {} | epic | {} | {} | {} | smoke | {} |\n",
                markdown_cell(&gap.summary),
                markdown_cell(gap.active_wi.as_deref().unwrap_or("-")),
                markdown_cell(gap_status_to_impl(gap.status)),
                markdown_cell(gap_status_to_verification(gap.status, capability.status)),
                markdown_cell(&capability_gate_inventory(capability)),
            ));
        }
    }
    if capability.gaps.is_empty()
        && capability
            .verification_contract
            .as_ref()
            .map(|contract| contract.claims.is_empty())
            .unwrap_or(true)
    {
        out.push_str(&format!(
            "| {} root | epic | {} | planned | planned | smoke | {} |\n",
            markdown_cell(&capability.title),
            markdown_cell(&root_wi_for_capability(capability)),
            markdown_cell(&capability_gate_inventory(capability)),
        ));
    }
    out.push('\n');
    if !capability.postlude.trim().is_empty() {
        out.push_str(capability.postlude.trim());
        out.push_str("\n\n");
    }
    if let Some(slot) = capability_efficiency_slot(capability) {
        out.push_str("#### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)\n\n");
        out.push_str(&format!(
            "Operating point: {}\n",
            markdown_cell(&slot.operating_point)
        ));
        out.push_str(&format!("Cube: {}\n\n", markdown_cell(&slot.cube)));
    }
    out
}

fn markdown_field_list_items(value: &str) -> Vec<String> {
    let items = value
        .split("<br>")
        .flat_map(|part| part.lines())
        .map(|part| part.trim().trim_start_matches("- ").trim().to_string())
        .filter(|part| !is_empty_table_value(part))
        .collect::<Vec<_>>();
    if items.is_empty() {
        vec!["-".to_string()]
    } else {
        items
    }
}

fn render_surface_field_items(surfaces: &[CapabilitySurface]) -> Vec<String> {
    surfaces
        .iter()
        .map(|surface| {
            let command_text = surface
                .commands
                .iter()
                .map(|command| format!("`{}`", command.trim()))
                .collect::<Vec<_>>()
                .join(" + ");
            let summary = surface.summary.trim();
            match (command_text.is_empty(), summary.is_empty()) {
                (true, true) => surface.kind.trim().to_string(),
                (false, true) => format!("{}: {}", surface.kind.trim(), command_text),
                (true, false) => format!("{}: {}", surface.kind.trim(), summary),
                (false, false) => {
                    format!("{}: {} - {}", surface.kind.trim(), command_text, summary)
                }
            }
        })
        .collect()
}

fn render_ec_dimension_field_items(dimensions: &[CapabilityEcDimension]) -> Vec<String> {
    dimensions
        .iter()
        .map(|dimension| {
            let runner = dimension.runner.trim();
            let summary = dimension.summary.trim();
            match (runner.is_empty(), summary.is_empty()) {
                (true, true) => dimension.dimension.as_str().to_string(),
                (false, true) => format!("{}: `{}`", dimension.dimension.as_str(), runner),
                (true, false) => format!("{}: {}", dimension.dimension.as_str(), summary),
                (false, false) => {
                    format!(
                        "{}: `{}` - {}",
                        dimension.dimension.as_str(),
                        runner,
                        summary
                    )
                }
            }
        })
        .collect()
}

fn capability_efficiency_slot(
    capability: &CapabilitySection,
) -> Option<&CapabilityEfficiencyBackfillSlot> {
    capability
        .ec_dimensions
        .iter()
        .find(|dimension| dimension.dimension == CapabilityEcDimensionKind::Efficiency)
        .and_then(|dimension| dimension.efficiency_backfill.as_ref())
}

fn render_legacy_capability_section(row: &LegacyCapabilityRow) -> String {
    let id = slugify(&row.capability);
    format!(
        "### {title}\n\nID: {id}\nRoot WI: {wi}\nStatus: candidate\nRequired Verification: smoke\nPromise:\n{promise}\nGate Inventory:\n- {evidence}\n\n| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |\n|---|---|---:|---|---|---|---|\n| {gap} | epic | {wi} | planned | planned | smoke | {evidence} |\n\n",
        title = markdown_cell(&row.capability),
        wi = markdown_cell(&row.active_wi),
        promise = markdown_cell(&row.current_state),
        evidence = markdown_cell(&row.evidence),
        gap = markdown_cell(&row.gaps),
    )
}

fn project_display_name(project: &str) -> String {
    project
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = first.to_ascii_uppercase().to_string();
                    out.push_str(chars.as_str());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn markdown_cell(value: &str) -> String {
    let trimmed = value.trim();
    if is_empty_table_value(trimmed) {
        "-".to_string()
    } else {
        trimmed.replace('|', "\\|").replace('\n', "<br>")
    }
}

fn root_wi_for_capability(capability: &CapabilitySection) -> String {
    capability
        .gaps
        .iter()
        .find_map(|gap| gap.active_wi.as_deref())
        .filter(|wi| !is_empty_table_value(wi))
        .unwrap_or("-")
        .to_string()
}

fn capability_impl_summary(capability: &CapabilitySection) -> &'static str {
    if capability.status == CapabilityStatus::Blocked {
        return "blocked";
    }
    if capability.status == CapabilityStatus::Verified
        || capability.gaps.iter().all(|gap| {
            matches!(
                gap.status,
                CapabilityGapStatus::Closed | CapabilityGapStatus::Deferred
            )
        })
    {
        "implemented"
    } else if capability
        .gaps
        .iter()
        .any(|gap| matches!(gap.status, CapabilityGapStatus::InProgress))
    {
        "partial"
    } else {
        "planned"
    }
}

fn capability_verification_summary(capability: &CapabilitySection) -> &'static str {
    match capability.status {
        CapabilityStatus::Verified => "verified",
        CapabilityStatus::Blocked => "blocked",
        CapabilityStatus::Auditing => "planned",
        CapabilityStatus::Confirmed => "planned",
        CapabilityStatus::Candidate => "planned",
        CapabilityStatus::Retired => "blocked",
    }
}

fn capability_production_summary(capability: &CapabilitySection) -> &'static str {
    if capability.release_scope {
        "ready"
    } else {
        "not_ready"
    }
}

fn parse_capability_index_summaries(body: &str) -> BTreeMap<String, CapabilityIndexSummary> {
    let lines = body.lines().collect::<Vec<_>>();
    let fenced = markdown_fenced_line_mask(&lines);
    let mut idx = 0;
    while idx < lines.len() {
        if fenced[idx] {
            idx += 1;
            continue;
        }
        let Some((level, title)) = parse_heading(lines[idx]) else {
            idx += 1;
            continue;
        };
        if !(level == 2 || level == 3) || !title.eq_ignore_ascii_case("Capability Index") {
            idx += 1;
            continue;
        }
        let mut cursor = idx + 1;
        let mut summaries = BTreeMap::new();
        while cursor < lines.len() {
            if fenced[cursor] {
                cursor += 1;
                continue;
            }
            if let Some((heading_level, _)) = parse_heading(lines[cursor]) {
                if heading_level <= level {
                    break;
                }
            }
            let Some((headers, rows, next_idx)) = parse_markdown_table_at(&lines, cursor) else {
                cursor += 1;
                continue;
            };
            let Some(capability_idx) = find_table_column(&headers, &["capability"]) else {
                cursor = next_idx;
                continue;
            };
            let implementation_idx = find_table_column(&headers, &["impl", "implementation"]);
            let verification_idx = find_table_column(&headers, &["verification"]);
            let maturity_idx = find_table_column(&headers, &["maturity"]);
            let production_idx = find_table_column(&headers, &["production"]);
            let notes_idx = find_table_column(&headers, &["notes", "note"]);
            for row in rows {
                let name = table_cell(&row, capability_idx);
                if is_empty_table_value(&name) {
                    continue;
                }
                let summary = CapabilityIndexSummary {
                    implementation: implementation_idx
                        .map(|idx| table_cell(&row, idx))
                        .unwrap_or_else(|| "-".to_string()),
                    verification: verification_idx
                        .map(|idx| table_cell(&row, idx))
                        .unwrap_or_else(|| "-".to_string()),
                    maturity: maturity_idx
                        .map(|idx| table_cell(&row, idx))
                        .unwrap_or_else(|| "-".to_string()),
                    production: production_idx
                        .map(|idx| table_cell(&row, idx))
                        .unwrap_or_else(|| "-".to_string()),
                    notes: notes_idx
                        .map(|idx| table_cell(&row, idx))
                        .unwrap_or_default(),
                };
                summaries.insert(slugify(&name), summary.clone());
                summaries.insert(normalize_table_token(&name), summary);
            }
            cursor = next_idx;
        }
        return summaries;
    }
    BTreeMap::new()
}

fn parse_capability_prose_candidates(body: &str) -> Vec<CapabilityProseCandidate> {
    let lines = body.lines().collect::<Vec<_>>();
    let fenced = markdown_fenced_line_mask(&lines);
    let mut candidates = Vec::new();
    let mut idx = 0;
    while idx < lines.len() {
        if fenced[idx] {
            idx += 1;
            continue;
        }
        let Some((level, title)) = parse_heading(lines[idx]) else {
            idx += 1;
            continue;
        };
        if level != 2 || !title.eq_ignore_ascii_case("Capabilities") {
            idx += 1;
            continue;
        }
        let capabilities_end = next_heading_at_or_above(&lines, idx + 1, 2).unwrap_or(lines.len());
        let mut cursor = idx + 1;
        while cursor < capabilities_end {
            if fenced[cursor] {
                cursor += 1;
                continue;
            }
            let Some((candidate_level, raw_candidate_title)) = parse_heading(lines[cursor]) else {
                cursor += 1;
                continue;
            };
            if candidate_level < 3 {
                cursor += 1;
                continue;
            }
            let block_end = next_heading_at_or_above(&lines, cursor + 1, candidate_level)
                .unwrap_or(capabilities_end)
                .min(capabilities_end);
            if capability_prose_candidate_title_is_ignored(&raw_candidate_title)
                || markdown_block_has_capability_contract(&lines, cursor + 1, block_end)
            {
                cursor = block_end;
                continue;
            }
            let candidate_title = clean_markdown_inline_links(&raw_candidate_title);
            let id = slugify(&candidate_title);
            if id.is_empty() {
                cursor = block_end;
                continue;
            }
            let root_wi = first_issue_ref(&raw_candidate_title)
                .or_else(|| first_issue_ref(&lines[cursor + 1..block_end].join("\n")));
            let summary = first_candidate_summary_line(&lines, cursor + 1, block_end);
            candidates.push(CapabilityProseCandidate {
                id,
                title: candidate_title,
                line: cursor + 1,
                root_wi,
                summary,
            });
            cursor = block_end;
        }
        idx = capabilities_end;
    }
    dedupe_capability_prose_candidates(candidates)
}

fn capability_prose_candidate_title_is_ignored(title: &str) -> bool {
    let normalized = normalize_table_token(title);
    normalized == "capabilityindex"
        || normalized == "efficiency"
        || normalized.ends_with("generated")
        || title.contains("GENERATED")
}

fn first_candidate_summary_line(lines: &[&str], start: usize, end: usize) -> Option<String> {
    let fenced = markdown_fenced_line_mask(lines);
    let mut parts = Vec::new();
    for idx in start..end {
        if fenced[idx] {
            continue;
        }
        let trimmed = lines[idx].trim();
        if trimmed.is_empty() {
            if !parts.is_empty() {
                break;
            }
            continue;
        }
        if trimmed.starts_with('|')
            || trimmed.starts_with("<!--")
            || parse_heading(trimmed).is_some()
            || parse_markdown_contract_field_line(trimmed).is_some()
        {
            if !parts.is_empty() {
                break;
            }
            continue;
        }
        let summary = candidate_summary_text(trimmed);
        if !summary.is_empty() {
            parts.push(summary);
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(truncate_candidate_summary(&parts.join(" ")))
    }
}

fn candidate_summary_text(line: &str) -> String {
    clean_markdown_inline_links(
        line.trim()
            .trim_start_matches("- ")
            .trim_start_matches("* ")
            .trim(),
    )
}

fn truncate_candidate_summary(text: &str) -> String {
    let text = text.trim();
    let max_chars = 280;
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out = text.chars().take(max_chars).collect::<String>();
    out = out.trim_end().to_string();
    out.push_str("...");
    out
}

fn clean_markdown_inline_links(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let Some(close_rel) = rest[open + 1..].find(']') else {
            break;
        };
        let close = open + 1 + close_rel;
        if !rest[close + 1..].starts_with('(') {
            out.push_str(&rest[..=close]);
            rest = &rest[close + 1..];
            continue;
        }
        let Some(paren_close_rel) = rest[close + 2..].find(')') else {
            break;
        };
        let paren_close = close + 2 + paren_close_rel;
        out.push_str(&rest[..open]);
        out.push_str(&rest[open + 1..close]);
        rest = &rest[paren_close + 1..];
    }
    out.push_str(rest);
    out.trim().to_string()
}

fn first_issue_ref(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] != b'#' {
            idx += 1;
            continue;
        }
        let start = idx + 1;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end > start {
            return Some(format!("#{}", &text[start..end]));
        }
        idx += 1;
    }
    None
}

fn dedupe_capability_prose_candidates(
    candidates: Vec<CapabilityProseCandidate>,
) -> Vec<CapabilityProseCandidate> {
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.id.clone()))
        .collect()
}

fn capability_maturity_summary(capability: &CapabilitySection) -> String {
    capability
        .verification_contract
        .as_ref()
        .map(|contract| {
            contract
                .required_maturity
                .iter()
                .map(|maturity| maturity.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "smoke".to_string())
}

fn capability_gate_inventory(capability: &CapabilitySection) -> String {
    if let Some(raw) = capability_raw_gate_inventory(capability) {
        return raw;
    }
    let mut refs = Vec::new();
    if let Some(contract) = capability.verification_contract.as_ref() {
        for claim in &contract.claims {
            refs.extend(claim.fixtures.iter().cloned());
        }
    }
    refs.extend(
        capability
            .evidence
            .verification
            .iter()
            .map(|gate| format!("`{}`", gate.command)),
    );
    if refs.is_empty() {
        "-".to_string()
    } else {
        refs.join("<br>")
    }
}

fn capability_raw_gate_inventory(capability: &CapabilitySection) -> Option<String> {
    let marker = "Gate inventory:";
    let (_before, after) = capability.current_state.split_once(marker)?;
    let raw = after.trim();
    if raw.is_empty() || is_empty_table_value(raw) {
        None
    } else {
        Some(raw.to_string())
    }
}

fn claim_verification_summary(
    capability: &CapabilitySection,
    claim: &CapabilityClaim,
) -> &'static str {
    if capability.status == CapabilityStatus::Verified
        && (!claim.gates.is_empty() || !claim.fixtures.is_empty())
    {
        "verified"
    } else if capability.status == CapabilityStatus::Blocked {
        "blocked"
    } else {
        "planned"
    }
}

fn claim_gate_evidence(claim: &CapabilityClaim) -> String {
    let mut refs = claim
        .gates
        .iter()
        .map(|gate| format!("`{}`", gate.command))
        .collect::<Vec<_>>();
    refs.extend(claim.fixtures.iter().cloned());
    if refs.is_empty() {
        claim.oracle.clone()
    } else {
        refs.join("<br>")
    }
}

fn gap_status_to_impl(status: CapabilityGapStatus) -> &'static str {
    match status {
        CapabilityGapStatus::Open => "planned",
        CapabilityGapStatus::InProgress => "partial",
        CapabilityGapStatus::Blocked => "blocked",
        CapabilityGapStatus::Closed => "implemented",
        CapabilityGapStatus::Deferred => "out_of_scope",
    }
}

fn gap_status_to_verification(
    status: CapabilityGapStatus,
    capability_status: CapabilityStatus,
) -> &'static str {
    match (status, capability_status) {
        (CapabilityGapStatus::Blocked, _) => "blocked",
        (_, CapabilityStatus::Verified) => "verified",
        (CapabilityGapStatus::Closed, _) => "passing",
        (CapabilityGapStatus::Deferred, _) => "blocked",
        _ => "planned",
    }
}

fn humanize_id(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = first.to_ascii_uppercase().to_string();
                    out.push_str(chars.as_str());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn validate_capability_contract(capability: &CapabilitySection) -> Result<Vec<String>> {
    let mut findings = Vec::new();
    let requires_contract = matches!(
        capability.status,
        CapabilityStatus::Confirmed
            | CapabilityStatus::Auditing
            | CapabilityStatus::Blocked
            | CapabilityStatus::Verified
    );
    findings.extend(validate_efficiency_backfill_slots(capability));
    let Some(contract) = capability.verification_contract.as_ref() else {
        if requires_contract {
            findings.push(format!(
                "capability `{}` status {:?} requires verification_contract",
                capability.id, capability.status
            ));
        }
        return Ok(findings);
    };

    if contract.required_maturity.is_empty() {
        findings.push(format!(
            "capability `{}` verification_contract.required_maturity must not be empty",
            capability.id
        ));
    }
    if contract.claims.is_empty() {
        findings.push(format!(
            "capability `{}` verification_contract.claims must not be empty",
            capability.id
        ));
    }

    let mut claim_ids = BTreeSet::new();
    for claim in &contract.claims {
        if claim.id.trim().is_empty() {
            findings.push(format!(
                "capability `{}` has a claim with an empty id",
                capability.id
            ));
            continue;
        }
        if !claim_ids.insert(claim.id.clone()) {
            anyhow::bail!(
                "duplicate claim id `{}` in capability `{}`",
                claim.id,
                capability.id
            );
        }
        if claim.required_for_verified && claim.user_story.trim().is_empty() {
            findings.push(format!(
                "claim `{}` in capability `{}` requires user_story",
                claim.id, capability.id
            ));
        }
        if claim.required_for_verified && claim.oracle.trim().is_empty() {
            findings.push(format!(
                "claim `{}` in capability `{}` requires oracle",
                claim.id, capability.id
            ));
        }
        if claim.required_for_verified && claim.gates.is_empty() && claim.fixtures.is_empty() {
            findings.push(format!(
                "claim `{}` in capability `{}` requires at least one gate or fixture/inventory reference",
                claim.id, capability.id
            ));
        }
        let mut gate_ids = BTreeSet::new();
        for gate in &claim.gates {
            if gate.id.trim().is_empty() {
                findings.push(format!(
                    "claim `{}` in capability `{}` has a gate with an empty id",
                    claim.id, capability.id
                ));
            } else if !gate_ids.insert(gate.id.clone()) {
                anyhow::bail!(
                    "duplicate gate id `{}` in claim `{}` for capability `{}`",
                    gate.id,
                    claim.id,
                    capability.id
                );
            }
            if gate.command.trim().is_empty() {
                findings.push(format!(
                    "gate `{}` in claim `{}` for capability `{}` requires command",
                    gate.id, claim.id, capability.id
                ));
            }
            if gate.proves.trim().is_empty() {
                findings.push(format!(
                    "gate `{}` in claim `{}` for capability `{}` requires proves",
                    gate.id, claim.id, capability.id
                ));
            }
        }
    }
    Ok(findings)
}

fn validate_efficiency_backfill_slots(capability: &CapabilitySection) -> Vec<String> {
    let mut findings = Vec::new();
    for dimension in &capability.ec_dimensions {
        if let Some(slot) = &dimension.efficiency_backfill {
            if slot.operating_point.trim().is_empty() {
                findings.push(format!(
                    "capability `{}` efficiency backfill slot requires operating_point",
                    capability.id
                ));
            }
            if slot.cube.trim().is_empty() {
                findings.push(format!(
                    "capability `{}` efficiency backfill slot requires cube",
                    capability.id
                ));
            }
        }
    }
    findings
}

fn markdown_capability_document_needs_canonicalization(body: &str) -> bool {
    let lines = body.lines().collect::<Vec<_>>();
    let fenced = markdown_fenced_line_mask(&lines);
    let mut idx = 0;
    while idx < lines.len() {
        if fenced[idx] {
            idx += 1;
            continue;
        }
        if let Some((level, title)) = parse_heading(lines[idx]) {
            if markdown_heading_is_noncanonical_efficiency_backfill(&lines, idx, &title) {
                return true;
            }
            if level == 2 && title.eq_ignore_ascii_case("Capability Index") {
                return true;
            }
            if level == 2 && title.starts_with("Capability:") {
                idx += 1;
                continue;
            }
            if level == 2 {
                let block_end =
                    next_capability_heading(&lines, idx + 1, level).unwrap_or(lines.len());
                if markdown_block_has_capability_contract(&lines, idx + 1, block_end) {
                    return true;
                }
            }
        }
        if let Some((headers, rows, next_idx)) = parse_markdown_table_at(&lines, idx) {
            if markdown_contract_indices(&headers).is_some()
                || markdown_field_value_contract_has_id(&headers, &rows)
            {
                return true;
            }
            idx = next_idx;
            continue;
        }
        idx += 1;
    }
    false
}

fn markdown_heading_is_noncanonical_efficiency_backfill(
    lines: &[&str],
    heading_idx: usize,
    title: &str,
) -> bool {
    let normalized = normalize_table_token(title);
    if normalized != "efficiency" {
        return false;
    }
    let section_end = next_heading(lines, heading_idx + 1).unwrap_or(lines.len());
    let values = parse_markdown_field_values(lines, heading_idx + 1, section_end);
    values.contains_key("operatingpoint")
        || values.contains_key("efficiencyoperatingpoint")
        || values.contains_key("cube")
        || values.contains_key("cuberef")
        || values.contains_key("efficiencycube")
}

fn markdown_block_has_capability_contract(lines: &[&str], start: usize, end: usize) -> bool {
    let fenced = markdown_fenced_line_mask(lines);
    let mut cursor = start;
    while cursor < end {
        if fenced[cursor] {
            cursor += 1;
            continue;
        }
        if parse_markdown_contract_field_line(lines[cursor].trim())
            .map(|(key, _)| key == "id")
            .unwrap_or(false)
        {
            return true;
        }
        if let Some((headers, rows, next_idx)) = parse_markdown_table_at(lines, cursor) {
            if markdown_contract_indices(&headers).is_some()
                || markdown_field_value_contract_has_id(&headers, &rows)
            {
                return true;
            }
            cursor = next_idx;
            continue;
        }
        cursor += 1;
    }
    false
}

fn markdown_field_value_contract_has_id(headers: &[String], rows: &[Vec<String>]) -> bool {
    let Some(field_column) = find_table_column(headers, &["field", "property", "key"]) else {
        return false;
    };
    let Some(_value_column) = find_table_column(headers, &["value"]) else {
        return false;
    };
    rows.iter().any(|row| {
        matches!(
            normalize_table_token(&table_cell(row, field_column)).as_str(),
            "id" | "capabilityid"
        )
    })
}

fn parse_markdown_table_capability_sections(body: &str) -> Result<Vec<CapabilitySection>> {
    let lines = body.lines().collect::<Vec<_>>();
    let fenced = markdown_fenced_line_mask(&lines);
    let mut sections = Vec::new();
    let mut idx = 0;
    while idx < lines.len() {
        if fenced[idx] {
            idx += 1;
            continue;
        }
        let Some((level, title)) = parse_heading(lines[idx]) else {
            idx += 1;
            continue;
        };
        if level < 2
            || title.eq_ignore_ascii_case("Capability Index")
            || title.starts_with("Capability:")
        {
            idx += 1;
            continue;
        }

        let block_end = next_capability_heading(&lines, idx + 1, level).unwrap_or(lines.len());
        if let Some(section) =
            parse_markdown_capability_block(&lines, idx, block_end, title.clone())?
        {
            sections.push(section);
            idx = block_end;
        } else {
            idx += 1;
        }
    }
    Ok(sections)
}

fn parse_markdown_capability_block(
    lines: &[&str],
    heading_idx: usize,
    block_end: usize,
    title: String,
) -> Result<Option<CapabilitySection>> {
    let mut contract = parse_markdown_field_capability_contract(lines, heading_idx, block_end);
    let mut work_roots = Vec::new();
    let mut surfaces = Vec::new();
    let mut ec_dimensions = Vec::new();
    let mut machine_table_spans = Vec::<(usize, usize)>::new();
    let fenced = markdown_fenced_line_mask(lines);
    let mut cursor = heading_idx + 1;
    while cursor < block_end {
        if fenced[cursor] {
            cursor += 1;
            continue;
        }
        let Some((headers, rows, next_idx)) = parse_markdown_table_at(lines, cursor) else {
            cursor += 1;
            continue;
        };
        if let Some(parsed_contract) =
            parse_markdown_capability_contract(&title, heading_idx, &headers, &rows)?
        {
            contract = Some(parsed_contract);
            machine_table_spans.push((cursor, next_idx));
        } else if markdown_work_root_indices(&headers).is_some() {
            work_roots.push((headers, rows));
            machine_table_spans.push((cursor, next_idx));
        } else if let Some(parsed_surfaces) = parse_markdown_surface_table(&headers, &rows) {
            surfaces.extend(parsed_surfaces);
            machine_table_spans.push((cursor, next_idx));
        } else if let Some(parsed_dimensions) = parse_markdown_ec_dimension_table(&headers, &rows) {
            ec_dimensions.extend(parsed_dimensions);
            machine_table_spans.push((cursor, next_idx));
        }
        cursor = next_idx;
    }

    let Some(contract) = contract else {
        return Ok(None);
    };
    let id = contract.id;
    let status = parse_capability_status_cell(&contract.status);
    let promise = contract.promise;
    let root_wi = contract.root_wi;
    let required_maturity = parse_maturity_list(&contract.required_verification);
    let full_regenerability_required =
        parse_full_regenerability_required(&contract.required_verification);
    let gate_inventory = contract.gate_inventory;
    let dependencies = parse_dependency_list(&contract.dependencies);
    let capability_type = parse_capability_type_cell(&contract.capability_type)?;
    surfaces.extend(parse_capability_surfaces(&contract.surfaces));
    ec_dimensions.extend(parse_capability_ec_dimensions(&contract.ec_dimensions));
    if let Some(slot) = parse_efficiency_slot_from_contract(
        &contract.efficiency_operating_point,
        &contract.efficiency_cube,
    ) {
        merge_efficiency_backfill_slot(&mut ec_dimensions, slot);
    }
    if let Some((slot, section_span)) =
        parse_efficiency_backfill_section(lines, heading_idx + 1, block_end)
    {
        merge_efficiency_backfill_slot(&mut ec_dimensions, slot);
        machine_table_spans.push(section_span);
    }
    ec_dimensions = dedupe_ec_dimensions(ec_dimensions);
    surfaces = dedupe_surfaces(surfaces);
    let (prelude, postlude) = markdown_capability_prose_around_machine_tables(
        lines,
        heading_idx,
        block_end,
        &machine_table_spans,
    );

    let mut gaps = Vec::new();
    let mut work_root_rows = Vec::new();
    let mut claims = Vec::new();
    let mut verification = Vec::new();
    for (headers, rows) in work_roots {
        let Some(work_indices) = markdown_work_root_indices(&headers) else {
            continue;
        };
        for row in rows {
            let work_root = table_cell(&row, work_indices.work_root);
            if is_empty_table_value(&work_root) {
                continue;
            }
            let kind = table_cell(&row, work_indices.kind);
            validate_work_root_kind(&title, &work_root, &kind)?;
            let wi = table_cell(&row, work_indices.wi);
            let implementation = table_cell(&row, work_indices.implementation);
            validate_work_root_impl(&title, &work_root, &implementation)?;
            let verification_state = table_cell(&row, work_indices.verification);
            validate_work_root_verification(&title, &work_root, &verification_state)?;
            let maturity_cell = table_cell(&row, work_indices.maturity);
            validate_work_root_maturity(&title, &work_root, &maturity_cell)?;
            let gate_evidence = table_cell(&row, work_indices.gate_evidence);
            let gap_id = slugify(&work_root);
            let active_wi = if is_empty_table_value(&wi) {
                None
            } else {
                Some(wi.clone())
            };
            gaps.push(CapabilityGap {
                id: gap_id.clone(),
                status: capability_gap_status_from_table(&implementation, &verification_state),
                active_wi,
                summary: work_root.clone(),
            });
            work_root_rows.push(CapabilityWorkRoot {
                id: gap_id.clone(),
                work_root: work_root.clone(),
                kind: kind.clone(),
                wi: wi.clone(),
                implementation: implementation.clone(),
                verification: verification_state.clone(),
                maturity: maturity_cell.clone(),
                gate_evidence: gate_evidence.clone(),
            });

            let maturity = parse_first_maturity(&maturity_cell)
                .or_else(|| required_maturity.first().copied())
                .unwrap_or(CapabilityMaturity::Smoke);
            let (gates, fixtures) =
                capability_claim_evidence_from_table(&gap_id, &work_root, &gate_evidence);
            verification.extend(gates.iter().map(|gate| CapabilityVerification {
                id: gate.id.clone(),
                command: gate.command.clone(),
                proves: gate.proves.clone(),
            }));
            if maturity_cell != "none" || !gates.is_empty() || !fixtures.is_empty() {
                claims.push(CapabilityClaim {
                    id: gap_id,
                    user_story: work_root,
                    required_for_verified: true,
                    maturity,
                    oracle: if is_empty_table_value(&gate_evidence) {
                        gate_inventory.clone()
                    } else {
                        gate_evidence.clone()
                    },
                    fixtures,
                    negative_cases: Vec::new(),
                    gates,
                });
            }
        }
    }

    if gaps.is_empty() && !is_empty_table_value(&root_wi) {
        gaps.push(CapabilityGap {
            id: format!("{}-root", id),
            status: CapabilityGapStatus::Open,
            active_wi: Some(root_wi.clone()),
            summary: format!("{} root work", title),
        });
    }

    let verification_contract =
        if !required_maturity.is_empty() || !claims.is_empty() || full_regenerability_required {
            Some(CapabilityVerificationContract {
                required_maturity,
                claims,
                full_regenerability_required,
            })
        } else {
            None
        };

    Ok(Some(CapabilitySection {
        title,
        id,
        status,
        prelude,
        postlude,
        index_summary: None,
        capability_type,
        surfaces,
        ec_dimensions,
        promise,
        current_state: if is_empty_table_value(&gate_inventory) {
            format!("Root WI: {root_wi}")
        } else {
            format!("Root WI: {root_wi}; Gate inventory: {gate_inventory}")
        },
        gaps,
        work_roots: work_root_rows,
        verification_contract,
        evidence: CapabilityEvidence {
            source: Vec::new(),
            td: Vec::new(),
            cb: Vec::new(),
            verification,
        },
        done_when: Vec::new(),
        out_of_scope: Vec::new(),
        release_scope: false,
        dependencies,
        line: heading_idx + 1,
    }))
}

struct MarkdownCapabilityContract {
    id: String,
    root_wi: String,
    status: String,
    capability_type: String,
    surfaces: String,
    ec_dimensions: String,
    promise: String,
    required_verification: String,
    gate_inventory: String,
    dependencies: String,
    efficiency_operating_point: String,
    efficiency_cube: String,
}

fn parse_markdown_field_capability_contract(
    lines: &[&str],
    heading_idx: usize,
    block_end: usize,
) -> Option<MarkdownCapabilityContract> {
    let mut values = BTreeMap::<String, String>::new();
    let mut current_key: Option<String> = None;
    let fenced = markdown_fenced_line_mask(lines);
    let mut cursor = heading_idx + 1;

    while cursor < block_end {
        if fenced[cursor] {
            cursor += 1;
            continue;
        }
        if parse_markdown_table_at(lines, cursor).is_some() {
            break;
        }
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            cursor += 1;
            continue;
        }

        if let Some((key, value)) = parse_markdown_contract_field_line(trimmed) {
            current_key = Some(key.clone());
            append_markdown_contract_field_value(&mut values, &key, &value);
        } else if let Some(key) = current_key.as_deref() {
            let value = clean_markdown_contract_continuation(trimmed);
            append_markdown_contract_field_value(&mut values, key, &value);
        }
        cursor += 1;
    }

    let id = values.remove("id")?;
    Some(MarkdownCapabilityContract {
        id,
        root_wi: values.remove("rootwi").unwrap_or_else(|| "-".to_string()),
        status: values
            .remove("status")
            .unwrap_or_else(|| "candidate".to_string()),
        capability_type: values.remove("type").unwrap_or_else(|| "-".to_string()),
        surfaces: values.remove("surfaces").unwrap_or_else(|| "-".to_string()),
        ec_dimensions: values
            .remove("ecdimensions")
            .unwrap_or_else(|| "-".to_string()),
        promise: values.remove("promise").unwrap_or_default(),
        required_verification: values
            .remove("requiredverification")
            .unwrap_or_else(|| "-".to_string()),
        gate_inventory: values
            .remove("gateinventory")
            .unwrap_or_else(|| "-".to_string()),
        dependencies: values
            .remove("dependencies")
            .unwrap_or_else(|| "-".to_string()),
        efficiency_operating_point: values
            .remove("efficiencyoperatingpoint")
            .unwrap_or_else(|| "-".to_string()),
        efficiency_cube: values
            .remove("efficiencycube")
            .unwrap_or_else(|| "-".to_string()),
    })
}

fn parse_markdown_contract_field_line(line: &str) -> Option<(String, String)> {
    let line = line.strip_prefix("- ").unwrap_or(line).trim();
    let (raw_key, raw_value) = line.split_once(':')?;
    let key = normalize_table_token(raw_key.trim().trim_matches('*'));
    let canonical_key = match key.as_str() {
        "id" | "capabilityid" => "id",
        "rootwi" | "wi" => "rootwi",
        "status" => "status",
        "type" | "capabilitytype" => "type",
        "surface" | "surfaces" | "capabilitysurface" | "capabilitysurfaces" => "surfaces",
        "clisurface" | "commands" | "command" => "surfaces",
        "ecdimensions" | "dimension" | "dimensions" | "requireddimensions" => "ecdimensions",
        "promise" => "promise",
        "requiredverification" | "maturity" => "requiredverification",
        "gateinventory" | "gateevidence" | "inventory" => "gateinventory",
        "dependencies" | "dependency" | "depends" | "dependson" => "dependencies",
        "efficiencyoperatingpoint" | "operatingpoint" => "efficiencyoperatingpoint",
        "efficiencycube" | "cuberef" | "cube" => "efficiencycube",
        _ => return None,
    };
    Some((
        canonical_key.to_string(),
        clean_markdown_contract_continuation(raw_value),
    ))
}

fn clean_markdown_contract_continuation(value: &str) -> String {
    value
        .trim()
        .strip_prefix("- ")
        .unwrap_or(value.trim())
        .trim()
        .to_string()
}

fn append_markdown_contract_field_value(
    values: &mut BTreeMap<String, String>,
    key: &str,
    value: &str,
) {
    if value.is_empty() {
        values.entry(key.to_string()).or_default();
        return;
    }
    let entry = values.entry(key.to_string()).or_default();
    if entry.is_empty() {
        entry.push_str(value);
    } else if key == "promise" {
        entry.push('\n');
        entry.push_str(value);
    } else {
        entry.push_str("<br>");
        entry.push_str(value);
    }
}

fn markdown_capability_prose_around_machine_tables(
    lines: &[&str],
    heading_idx: usize,
    block_end: usize,
    machine_table_spans: &[(usize, usize)],
) -> (String, String) {
    if machine_table_spans.is_empty() {
        return (String::new(), String::new());
    }
    let first_table_start = machine_table_spans
        .iter()
        .map(|(start, _)| *start)
        .min()
        .unwrap_or(block_end);
    let last_table_end = machine_table_spans
        .iter()
        .map(|(_, end)| *end)
        .max()
        .unwrap_or(heading_idx + 1);
    let prelude = join_markdown_prose_lines(&lines[heading_idx + 1..first_table_start]);
    let postlude = join_markdown_prose_lines(&lines[last_table_end..block_end]);
    (prelude, postlude)
}

fn join_markdown_prose_lines(lines: &[&str]) -> String {
    lines.join("\n").trim().to_string()
}

fn parse_markdown_capability_contract(
    title: &str,
    heading_idx: usize,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<Option<MarkdownCapabilityContract>> {
    if let Some(indices) = markdown_contract_indices(headers) {
        let Some(row) = rows.first() else {
            anyhow::bail!(
                "capability `{}` at line {} has an empty contract table",
                title,
                heading_idx + 1
            );
        };
        return Ok(Some(MarkdownCapabilityContract {
            id: table_cell(row, indices.id),
            root_wi: table_cell(row, indices.root_wi),
            status: table_cell(row, indices.status),
            capability_type: indices
                .capability_type
                .map(|idx| table_cell(row, idx))
                .unwrap_or_else(|| "-".to_string()),
            surfaces: indices
                .surfaces
                .map(|idx| table_cell(row, idx))
                .unwrap_or_else(|| "-".to_string()),
            ec_dimensions: indices
                .ec_dimensions
                .map(|idx| table_cell(row, idx))
                .unwrap_or_else(|| "-".to_string()),
            promise: table_cell(row, indices.promise),
            required_verification: table_cell(row, indices.required_verification),
            gate_inventory: table_cell(row, indices.gate_inventory),
            dependencies: indices
                .dependencies
                .map(|idx| table_cell(row, idx))
                .unwrap_or_else(|| "-".to_string()),
            efficiency_operating_point: indices
                .efficiency_operating_point
                .map(|idx| table_cell(row, idx))
                .unwrap_or_else(|| "-".to_string()),
            efficiency_cube: indices
                .efficiency_cube
                .map(|idx| table_cell(row, idx))
                .unwrap_or_else(|| "-".to_string()),
        }));
    }

    let Some(field_column) = find_table_column(headers, &["field", "property", "key"]) else {
        return Ok(None);
    };
    let Some(value_column) = find_table_column(headers, &["value"]) else {
        return Ok(None);
    };

    let value_for = |aliases: &[&str]| -> Option<String> {
        rows.iter().find_map(|row| {
            let field = normalize_table_token(&table_cell(row, field_column));
            aliases
                .contains(&field.as_str())
                .then(|| table_cell(row, value_column))
        })
    };
    let Some(id) = value_for(&["id", "capabilityid"]) else {
        return Ok(None);
    };

    Ok(Some(MarkdownCapabilityContract {
        id,
        root_wi: value_for(&["rootwi", "wi"]).unwrap_or_else(|| "-".to_string()),
        status: value_for(&["status"]).unwrap_or_else(|| "candidate".to_string()),
        capability_type: value_for(&["type", "capabilitytype"]).unwrap_or_else(|| "-".to_string()),
        surfaces: value_for(&[
            "surface",
            "surfaces",
            "capabilitysurface",
            "capabilitysurfaces",
            "clisurface",
            "commands",
            "command",
        ])
        .unwrap_or_else(|| "-".to_string()),
        ec_dimensions: value_for(&[
            "ecdimensions",
            "dimension",
            "dimensions",
            "requireddimensions",
        ])
        .unwrap_or_else(|| "-".to_string()),
        promise: value_for(&["promise"]).unwrap_or_default(),
        required_verification: value_for(&["requiredverification", "maturity"])
            .unwrap_or_else(|| "-".to_string()),
        gate_inventory: value_for(&["gateinventory", "gateevidence", "inventory"])
            .unwrap_or_else(|| "-".to_string()),
        dependencies: value_for(&["dependencies", "dependency", "depends", "dependson"])
            .unwrap_or_else(|| "-".to_string()),
        efficiency_operating_point: value_for(&["efficiencyoperatingpoint", "operatingpoint"])
            .unwrap_or_else(|| "-".to_string()),
        efficiency_cube: value_for(&["efficiencycube", "cuberef", "cube"])
            .unwrap_or_else(|| "-".to_string()),
    }))
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&level) || !trimmed.chars().nth(level).is_some_and(|ch| ch == ' ') {
        return None;
    }
    let title = trimmed[level..].trim().trim_matches('#').trim().to_string();
    Some((level, title))
}

fn markdown_fenced_line_mask(lines: &[&str]) -> Vec<bool> {
    let mut in_fence = false;
    let mut mask = Vec::with_capacity(lines.len());
    for line in lines {
        mask.push(in_fence);
        if is_markdown_fence_line(line) {
            in_fence = !in_fence;
        }
    }
    mask
}

fn is_markdown_fence_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

fn next_heading(lines: &[&str], start: usize) -> Option<usize> {
    let fenced = markdown_fenced_line_mask(lines);
    (start..lines.len()).find(|idx| !fenced[*idx] && parse_heading(lines[*idx]).is_some())
}

fn next_capability_heading(lines: &[&str], start: usize, current_level: usize) -> Option<usize> {
    let fenced = markdown_fenced_line_mask(lines);
    (start..lines.len()).find(|idx| {
        if fenced[*idx] {
            return false;
        }
        let Some((level, _title)) = parse_heading(lines[*idx]) else {
            return false;
        };
        level <= current_level || heading_has_capability_contract(lines, *idx)
    })
}

fn heading_has_capability_contract(lines: &[&str], heading_idx: usize) -> bool {
    let fenced = markdown_fenced_line_mask(lines);
    let block_end = next_heading(lines, heading_idx + 1).unwrap_or(lines.len());
    parse_markdown_field_capability_contract(lines, heading_idx, block_end).is_some() || {
        let mut cursor = heading_idx + 1;
        while cursor < block_end {
            if fenced[cursor] {
                cursor += 1;
                continue;
            }
            let Some((headers, _rows, next_idx)) = parse_markdown_table_at(lines, cursor) else {
                cursor += 1;
                continue;
            };
            if markdown_contract_indices(&headers).is_some() {
                return true;
            }
            if find_table_column(&headers, &["field", "property", "key"]).is_some()
                && find_table_column(&headers, &["value"]).is_some()
            {
                return true;
            }
            cursor = next_idx;
        }
        false
    }
}

fn parse_markdown_table_at(
    lines: &[&str],
    start: usize,
) -> Option<(Vec<String>, Vec<Vec<String>>, usize)> {
    let headers = parse_markdown_table_row(lines.get(start)?)?;
    let separator = parse_markdown_table_row(lines.get(start + 1)?)?;
    if !is_markdown_separator_row(&separator) {
        return None;
    }
    let mut rows = Vec::new();
    let mut cursor = start + 2;
    while cursor < lines.len() {
        let Some(cells) = parse_markdown_table_row(lines[cursor]) else {
            break;
        };
        if is_markdown_separator_row(&cells) {
            cursor += 1;
            continue;
        }
        rows.push(cells);
        cursor += 1;
    }
    Some((headers, rows, cursor))
}

struct MarkdownContractIndices {
    id: usize,
    root_wi: usize,
    status: usize,
    capability_type: Option<usize>,
    surfaces: Option<usize>,
    ec_dimensions: Option<usize>,
    promise: usize,
    required_verification: usize,
    gate_inventory: usize,
    dependencies: Option<usize>,
    efficiency_operating_point: Option<usize>,
    efficiency_cube: Option<usize>,
}

fn markdown_contract_indices(cells: &[String]) -> Option<MarkdownContractIndices> {
    Some(MarkdownContractIndices {
        id: find_table_column(cells, &["id"])?,
        root_wi: find_table_column(cells, &["rootwi", "wi"])?,
        status: find_table_column(cells, &["status"])?,
        capability_type: find_table_column(cells, &["type", "capabilitytype"]),
        surfaces: find_table_column(
            cells,
            &[
                "surface",
                "surfaces",
                "capabilitysurface",
                "capabilitysurfaces",
                "clisurface",
                "commands",
            ],
        ),
        ec_dimensions: find_table_column(
            cells,
            &[
                "ecdimensions",
                "dimension",
                "dimensions",
                "requireddimensions",
            ],
        ),
        promise: find_table_column(cells, &["promise"])?,
        required_verification: find_table_column(cells, &["requiredverification", "maturity"])?,
        gate_inventory: find_table_column(cells, &["gateinventory", "gateevidence", "inventory"])?,
        dependencies: find_table_column(cells, &["dependencies", "dependency", "depends"]),
        efficiency_operating_point: find_table_column(
            cells,
            &["efficiencyoperatingpoint", "operatingpoint"],
        ),
        efficiency_cube: find_table_column(cells, &["efficiencycube", "cuberef", "cube"]),
    })
}

fn parse_capability_type_cell(value: &str) -> Result<Option<CapabilityType>> {
    if is_empty_table_value(value) {
        return Ok(None);
    }
    CapabilityType::from_cli_str(value).map(Some)
}

fn parse_capability_surfaces(value: &str) -> Vec<CapabilitySurface> {
    split_surface_contract_list(value)
        .into_iter()
        .filter_map(|piece| {
            let piece = piece.trim().trim_start_matches("- ").trim();
            if is_empty_table_value(piece) {
                return None;
            }
            let (kind, summary) = piece
                .split_once(':')
                .map(|(kind, summary)| (kind.trim(), summary.trim()))
                .unwrap_or(("CLI", piece));
            let commands = extract_backtick_values(summary);
            let summary = clean_surface_summary(summary);
            Some(CapabilitySurface {
                kind: normalize_surface_kind(kind),
                commands,
                summary,
                verification: String::new(),
            })
        })
        .collect()
}

fn split_surface_contract_list(value: &str) -> Vec<String> {
    value
        .split("<br>")
        .flat_map(|part| part.split(';'))
        .flat_map(|part| part.split('\n'))
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn clean_surface_summary(summary: &str) -> String {
    let summary = summary.trim();
    if summary.starts_with('`') {
        if let Some((_commands, rest)) = summary.split_once(" - ") {
            return rest.trim().to_string();
        }
    }
    summary.to_string()
}

fn parse_capability_ec_dimensions(value: &str) -> Vec<CapabilityEcDimension> {
    split_ec_dimension_contract_list(value)
        .into_iter()
        .filter_map(|piece| {
            let piece = piece.trim().trim_start_matches("- ").trim();
            if is_empty_table_value(piece) {
                return None;
            }
            let (raw_dimension, summary) = piece
                .split_once(':')
                .map(|(dimension, summary)| (dimension.trim(), summary.trim()))
                .unwrap_or((piece, ""));
            let dimension = parse_ec_dimension_kind(raw_dimension)?;
            let runner = first_backtick_value(summary).unwrap_or_default();
            let summary = clean_runner_prefixed_summary(summary);
            Some(CapabilityEcDimension {
                dimension,
                runner,
                summary,
                required_for_production: None,
                efficiency_backfill: None,
            })
        })
        .collect()
}

fn split_ec_dimension_contract_list(value: &str) -> Vec<String> {
    value
        .split("<br>")
        .flat_map(|part| part.split(';'))
        .flat_map(|part| part.split('\n'))
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn clean_runner_prefixed_summary(summary: &str) -> String {
    let summary = summary.trim();
    if summary.starts_with('`') {
        if let Some((_runner, rest)) = summary.split_once(" - ") {
            return rest.trim().to_string();
        }
    }
    summary.to_string()
}

fn parse_markdown_surface_table(
    headers: &[String],
    rows: &[Vec<String>],
) -> Option<Vec<CapabilitySurface>> {
    let surface_idx = find_table_column(headers, &["surface", "kind"]);
    let command_idx = find_table_column(headers, &["command", "commands", "cli"]);
    let summary_idx = find_table_column(headers, &["summary", "owns", "purpose"]);
    let verification_idx = find_table_column(headers, &["verification", "gate", "evidence"]);
    if surface_idx.is_none() && command_idx.is_none() {
        return None;
    }
    Some(
        rows.iter()
            .filter_map(|row| {
                let kind = surface_idx
                    .map(|idx| table_cell(row, idx))
                    .filter(|value| !is_empty_table_value(value))
                    .unwrap_or_else(|| "CLI".to_string());
                let command_text = command_idx
                    .map(|idx| table_cell(row, idx))
                    .unwrap_or_else(|| "-".to_string());
                let summary = summary_idx
                    .map(|idx| table_cell(row, idx))
                    .unwrap_or_default();
                let verification = verification_idx
                    .map(|idx| table_cell(row, idx))
                    .unwrap_or_default();
                if is_empty_table_value(&command_text)
                    && summary.trim().is_empty()
                    && verification.trim().is_empty()
                {
                    return None;
                }
                Some(CapabilitySurface {
                    kind: normalize_surface_kind(&kind),
                    commands: extract_backtick_values(&command_text),
                    summary,
                    verification,
                })
            })
            .collect(),
    )
}

fn parse_markdown_ec_dimension_table(
    headers: &[String],
    rows: &[Vec<String>],
) -> Option<Vec<CapabilityEcDimension>> {
    let dimension_idx = find_table_column(headers, &["dimension", "ecdimension", "category"])?;
    let runner_idx = find_table_column(headers, &["runner", "tool", "command"]);
    let summary_idx = find_table_column(headers, &["summary", "evidence", "contract"]);
    Some(
        rows.iter()
            .filter_map(|row| {
                let dimension = parse_ec_dimension_kind(&table_cell(row, dimension_idx))?;
                let runner = runner_idx
                    .map(|idx| table_cell(row, idx))
                    .filter(|value| !is_empty_table_value(value))
                    .unwrap_or_default();
                let summary = summary_idx
                    .map(|idx| table_cell(row, idx))
                    .unwrap_or_default();
                Some(CapabilityEcDimension {
                    dimension,
                    runner,
                    summary,
                    required_for_production: None,
                    efficiency_backfill: None,
                })
            })
            .collect(),
    )
}

fn parse_efficiency_slot_from_contract(
    operating_point: &str,
    cube: &str,
) -> Option<CapabilityEfficiencyBackfillSlot> {
    if is_empty_table_value(operating_point) && is_empty_table_value(cube) {
        return None;
    }
    Some(CapabilityEfficiencyBackfillSlot {
        operating_point: if is_empty_table_value(operating_point) {
            String::new()
        } else {
            operating_point.trim().to_string()
        },
        cube: if is_empty_table_value(cube) {
            String::new()
        } else {
            cube.trim().to_string()
        },
    })
}

fn parse_efficiency_backfill_section(
    lines: &[&str],
    start: usize,
    block_end: usize,
) -> Option<(CapabilityEfficiencyBackfillSlot, (usize, usize))> {
    let (section_start, section_end) =
        find_efficiency_backfill_section_span(lines, start, block_end)?;
    let values = parse_markdown_field_values(lines, section_start + 1, section_end);
    let slot = parse_efficiency_slot_from_contract(
        values
            .get("operatingpoint")
            .or_else(|| values.get("efficiencyoperatingpoint"))
            .map(String::as_str)
            .unwrap_or("-"),
        values
            .get("cube")
            .or_else(|| values.get("cuberef"))
            .or_else(|| values.get("efficiencycube"))
            .map(String::as_str)
            .unwrap_or("-"),
    )?;
    Some((slot, (section_start, section_end)))
}

fn find_efficiency_backfill_section_span(
    lines: &[&str],
    start: usize,
    block_end: usize,
) -> Option<(usize, usize)> {
    let fenced = markdown_fenced_line_mask(lines);
    let mut cursor = start;
    while cursor < block_end {
        if fenced[cursor] {
            cursor += 1;
            continue;
        }
        let Some((_level, title)) = parse_heading(lines[cursor]) else {
            cursor += 1;
            continue;
        };
        let normalized = normalize_table_token(&title);
        if normalized != "efficiency" && !normalized.starts_with("efficiencygenerated") {
            cursor += 1;
            continue;
        }
        let section_end = next_heading(lines, cursor + 1)
            .filter(|idx| *idx < block_end)
            .unwrap_or(block_end);
        return Some((cursor, section_end));
    }
    None
}

fn parse_markdown_field_values(
    lines: &[&str],
    start: usize,
    end: usize,
) -> BTreeMap<String, String> {
    let mut values = BTreeMap::<String, String>::new();
    let mut current_key: Option<String> = None;
    let fenced = markdown_fenced_line_mask(lines);
    let mut cursor = start;
    while cursor < end {
        if fenced[cursor] {
            cursor += 1;
            continue;
        }
        if parse_markdown_table_at(lines, cursor).is_some() {
            break;
        }
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            cursor += 1;
            continue;
        }
        if let Some((raw_key, raw_value)) = trimmed
            .strip_prefix("- ")
            .unwrap_or(trimmed)
            .split_once(':')
        {
            let key = normalize_table_token(raw_key.trim().trim_matches('*'));
            current_key = Some(key.clone());
            append_markdown_contract_field_value(
                &mut values,
                &key,
                &clean_markdown_contract_continuation(raw_value),
            );
        } else if let Some(key) = current_key.as_deref() {
            append_markdown_contract_field_value(
                &mut values,
                key,
                &clean_markdown_contract_continuation(trimmed),
            );
        }
        cursor += 1;
    }
    values
}

fn merge_efficiency_backfill_slot(
    dimensions: &mut Vec<CapabilityEcDimension>,
    slot: CapabilityEfficiencyBackfillSlot,
) {
    if let Some(dimension) = dimensions
        .iter_mut()
        .find(|dimension| dimension.dimension == CapabilityEcDimensionKind::Efficiency)
    {
        dimension.efficiency_backfill = Some(slot);
        return;
    }
    dimensions.push(CapabilityEcDimension {
        dimension: CapabilityEcDimensionKind::Efficiency,
        runner: String::new(),
        summary: "aw-generated efficiency backfill slot".to_string(),
        required_for_production: None,
        efficiency_backfill: Some(slot),
    });
}

fn dedupe_ec_dimensions(dimensions: Vec<CapabilityEcDimension>) -> Vec<CapabilityEcDimension> {
    let mut by_dimension = BTreeMap::<CapabilityEcDimensionKind, CapabilityEcDimension>::new();
    for dimension in dimensions {
        by_dimension
            .entry(dimension.dimension)
            .and_modify(|existing| {
                if existing.runner.is_empty() {
                    existing.runner = dimension.runner.clone();
                }
                if existing.summary.is_empty() {
                    existing.summary = dimension.summary.clone();
                }
                if existing.required_for_production.is_none() {
                    existing.required_for_production = dimension.required_for_production;
                }
                if existing.efficiency_backfill.is_none() {
                    existing.efficiency_backfill = dimension.efficiency_backfill.clone();
                }
            })
            .or_insert(dimension);
    }
    by_dimension.into_values().collect()
}

fn dedupe_surfaces(surfaces: Vec<CapabilitySurface>) -> Vec<CapabilitySurface> {
    let mut seen = BTreeSet::new();
    surfaces
        .into_iter()
        .filter(|surface| {
            let key = format!(
                "{}:{}:{}",
                normalize_table_token(&surface.kind),
                surface.commands.join(","),
                surface.summary
            );
            seen.insert(key)
        })
        .collect()
}

fn normalize_surface_kind(value: &str) -> String {
    match normalize_table_token(value).as_str() {
        "cli" | "command" | "commands" => "CLI",
        "http" | "api" | "rest" => "HTTP",
        "sdk" => "SDK",
        "ui" | "webui" | "web" => "UI",
        "config" | "configuration" => "Config",
        "fileformat" | "file" | "format" => "FileFormat",
        _ => value.trim(),
    }
    .to_string()
}

fn parse_ec_dimension_kind(value: &str) -> Option<CapabilityEcDimensionKind> {
    match normalize_table_token(value).as_str() {
        "behavior" | "behaviour" | "functional" | "function" | "render" => {
            Some(CapabilityEcDimensionKind::Behavior)
        }
        "efficiency" | "performance" | "perf" => Some(CapabilityEcDimensionKind::Efficiency),
        "security" | "secure" => Some(CapabilityEcDimensionKind::Security),
        "stability" | "resilience" | "reliability" => Some(CapabilityEcDimensionKind::Stability),
        "content" | "docs" | "documentation" => Some(CapabilityEcDimensionKind::Content),
        _ => None,
    }
}

fn first_backtick_value(value: &str) -> Option<String> {
    extract_backtick_values(value).into_iter().next()
}

fn parse_dependency_list(cell: &str) -> Vec<String> {
    cell.split([',', '\n'])
        .flat_map(|part| part.split("<br>"))
        .map(|part| part.trim().trim_matches('`'))
        .filter(|part| !is_empty_table_value(part))
        .map(slugify)
        .filter(|part| !part.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

struct MarkdownWorkRootIndices {
    work_root: usize,
    kind: usize,
    wi: usize,
    implementation: usize,
    verification: usize,
    maturity: usize,
    gate_evidence: usize,
}

fn markdown_work_root_indices(cells: &[String]) -> Option<MarkdownWorkRootIndices> {
    Some(MarkdownWorkRootIndices {
        work_root: find_table_column(cells, &["workroot", "capability", "root"])?,
        kind: find_table_column(cells, &["kind", "type"])?,
        wi: find_table_column(cells, &["wi", "workitem"])?,
        implementation: find_table_column(cells, &["impl", "implementation"])?,
        verification: find_table_column(cells, &["verification"])?,
        maturity: find_table_column(cells, &["maturity"])?,
        gate_evidence: find_table_column(cells, &["gateevidence", "gate", "evidence"])?,
    })
}

fn parse_capability_status_cell(cell: &str) -> CapabilityStatus {
    match normalize_table_token(cell).as_str() {
        "confirmed" => CapabilityStatus::Confirmed,
        "auditing" => CapabilityStatus::Auditing,
        "blocked" => CapabilityStatus::Blocked,
        "verified" => CapabilityStatus::Verified,
        "retired" => CapabilityStatus::Retired,
        _ => CapabilityStatus::Candidate,
    }
}

fn parse_capability_status_arg(value: &str) -> Result<CapabilityStatus> {
    match normalize_table_token(value).as_str() {
        "candidate" => Ok(CapabilityStatus::Candidate),
        "confirmed" => Ok(CapabilityStatus::Confirmed),
        "auditing" => Ok(CapabilityStatus::Auditing),
        "blocked" => Ok(CapabilityStatus::Blocked),
        "verified" => Ok(CapabilityStatus::Verified),
        "retired" => Ok(CapabilityStatus::Retired),
        other => anyhow::bail!(
            "invalid capability status `{}`; expected candidate, confirmed, auditing, blocked, verified, or retired",
            other
        ),
    }
}

fn parse_maturity_list(cell: &str) -> Vec<CapabilityMaturity> {
    cell.split(',')
        .filter_map(parse_first_maturity)
        .collect::<Vec<_>>()
}

fn parse_full_regenerability_required(cell: &str) -> bool {
    cell.split(',').any(|token| {
        matches!(
            normalize_table_token(token).as_str(),
            "fullregenerability" | "regenerability" | "fullcodegen" | "codegen"
        )
    })
}

fn parse_first_maturity(cell: &str) -> Option<CapabilityMaturity> {
    match normalize_table_token(cell).as_str() {
        "smoke" => Some(CapabilityMaturity::Smoke),
        "conformance" => Some(CapabilityMaturity::Conformance),
        "corpus" => Some(CapabilityMaturity::Corpus),
        "negative" => Some(CapabilityMaturity::Negative),
        "dogfood" => Some(CapabilityMaturity::Dogfood),
        _ => None,
    }
}

fn validate_work_root_kind(capability: &str, work_root: &str, value: &str) -> Result<()> {
    let token = normalize_table_token(value);
    if token.is_empty() || token == "epic" || token == "subepic" || token == "change" {
        return Ok(());
    }
    anyhow::bail!(
        "capability `{}` work root `{}` has invalid Kind `{}`; expected epic, subepic, or change",
        capability,
        work_root,
        value
    )
}

fn validate_work_root_impl(capability: &str, work_root: &str, value: &str) -> Result<()> {
    let token = normalize_table_token(value);
    if matches!(
        token.as_str(),
        "planned" | "partial" | "implemented" | "blocked" | "outofscope"
    ) {
        return Ok(());
    }
    anyhow::bail!(
        "capability `{}` work root `{}` has invalid Impl `{}`; expected planned, partial, implemented, blocked, or out_of_scope",
        capability,
        work_root,
        value
    )
}

fn validate_work_root_verification(capability: &str, work_root: &str, value: &str) -> Result<()> {
    let token = normalize_table_token(value);
    if matches!(
        token.as_str(),
        "none" | "planned" | "failing" | "passing" | "verified" | "blocked"
    ) {
        return Ok(());
    }
    anyhow::bail!(
        "capability `{}` work root `{}` has invalid Verification `{}`; expected none, planned, failing, passing, verified, or blocked",
        capability,
        work_root,
        value
    )
}

fn validate_work_root_maturity(capability: &str, work_root: &str, value: &str) -> Result<()> {
    let token = normalize_table_token(value);
    if token == "none" || parse_first_maturity(value).is_some() {
        return Ok(());
    }
    anyhow::bail!(
        "capability `{}` work root `{}` has invalid Maturity `{}`; expected none, smoke, conformance, corpus, negative, or dogfood",
        capability,
        work_root,
        value
    )
}

fn capability_gap_status_from_table(
    implementation: &str,
    verification: &str,
) -> CapabilityGapStatus {
    let implementation = normalize_table_token(implementation);
    let verification = normalize_table_token(verification);
    if implementation == "outofscope" {
        CapabilityGapStatus::Deferred
    } else if implementation == "blocked" || verification == "blocked" {
        CapabilityGapStatus::Blocked
    } else if implementation == "implemented"
        && matches!(verification.as_str(), "passing" | "verified")
    {
        CapabilityGapStatus::Closed
    } else if implementation == "planned" && matches!(verification.as_str(), "none" | "planned") {
        CapabilityGapStatus::Open
    } else {
        CapabilityGapStatus::InProgress
    }
}

fn capability_claim_evidence_from_table(
    gap_id: &str,
    work_root: &str,
    gate_evidence: &str,
) -> (Vec<CapabilityClaimGate>, Vec<String>) {
    let mut gates = Vec::new();
    let mut fixtures = Vec::new();
    for piece in split_gate_evidence_pieces(gate_evidence) {
        let commands = extract_backtick_values(&piece);
        if commands.is_empty() {
            if !is_empty_table_value(&piece) {
                fixtures.push(piece);
            }
            continue;
        }
        for command in commands {
            let id = if gates.is_empty() {
                format!("{}-gate", gap_id)
            } else {
                format!("{}-gate-{}", gap_id, gates.len() + 1)
            };
            gates.push(CapabilityClaimGate {
                id,
                command,
                proves: work_root.to_string(),
            });
        }
    }
    if gates.is_empty() && fixtures.is_empty() && !is_empty_table_value(gate_evidence) {
        fixtures.push(gate_evidence.to_string());
    }
    (gates, fixtures)
}

fn split_gate_evidence_pieces(cell: &str) -> Vec<String> {
    cell.split("<br>")
        .flat_map(|part| part.lines())
        .map(|part| part.trim().trim_start_matches("- ").trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn extract_backtick_values(cell: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = cell;
    while let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        let value = after_start[..end].trim();
        if !value.is_empty() {
            values.push(value.to_string());
        }
        rest = &after_start[end + 1..];
    }
    values
}

fn normalize_table_token(cell: &str) -> String {
    cell.trim()
        .trim_matches('`')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>()
}

fn is_empty_table_value(cell: &str) -> bool {
    let trimmed = cell.trim();
    trimmed.is_empty() || matches!(trimmed, "-" | "n/a" | "N/A")
}

fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn parse_h2_capability_sections(body: &str) -> Result<Vec<CapabilitySection>> {
    let lines = body.lines().collect::<Vec<_>>();
    let mut sections = Vec::new();
    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx].trim();
        let Some(title) = line.strip_prefix("## Capability:") else {
            idx += 1;
            continue;
        };
        let title = title.trim().to_string();
        let line_number = idx + 1;
        let annotation = lines
            .get(idx + 1)
            .map(|line| line.trim())
            .unwrap_or_default();
        if annotation != "<!-- type: capability lang: yaml -->" {
            anyhow::bail!(
                "capability `{}` at line {} missing `<!-- type: capability lang: yaml -->` annotation",
                title,
                line_number
            );
        }
        let mut fence_start = None;
        let mut cursor = idx + 2;
        while cursor < lines.len() {
            let trimmed = lines[cursor].trim();
            if trimmed.starts_with("## ") {
                break;
            }
            if trimmed == "```yaml" {
                fence_start = Some(cursor + 1);
                break;
            }
            cursor += 1;
        }
        let Some(yaml_start) = fence_start else {
            anyhow::bail!(
                "capability `{}` at line {} missing YAML code fence",
                title,
                line_number
            );
        };
        let mut yaml_end = None;
        cursor = yaml_start;
        while cursor < lines.len() {
            if lines[cursor].trim() == "```" {
                yaml_end = Some(cursor);
                break;
            }
            cursor += 1;
        }
        let Some(yaml_end) = yaml_end else {
            anyhow::bail!(
                "capability `{}` at line {} has unterminated YAML code fence",
                title,
                line_number
            );
        };
        let yaml = lines[yaml_start..yaml_end].join("\n");
        let parsed: CapabilityYaml = serde_yaml::from_str(&yaml).with_context(|| {
            format!(
                "invalid capability YAML for `{}` at line {}",
                title, line_number
            )
        })?;
        sections.push(CapabilitySection::from_yaml(title, line_number, parsed));
        idx = yaml_end + 1;
    }
    Ok(sections)
}

fn parse_legacy_capability_table(body: &str) -> Vec<LegacyCapabilityRow> {
    let lines = body.lines().collect::<Vec<_>>();
    for (header_idx, line) in lines.iter().enumerate() {
        let Some(header_cells) = parse_markdown_table_row(line) else {
            continue;
        };
        let Some(indices) = legacy_capability_column_indices(&header_cells) else {
            continue;
        };
        let mut row_idx = header_idx + 1;
        if row_idx < lines.len() {
            if let Some(cells) = parse_markdown_table_row(lines[row_idx]) {
                if is_markdown_separator_row(&cells) {
                    row_idx += 1;
                }
            }
        }
        let mut rows = Vec::new();
        while row_idx < lines.len() {
            let Some(cells) = parse_markdown_table_row(lines[row_idx]) else {
                break;
            };
            if is_markdown_separator_row(&cells) {
                row_idx += 1;
                continue;
            }
            rows.push(LegacyCapabilityRow {
                capability: table_cell(&cells, indices.capability),
                current_state: table_cell(&cells, indices.current_state),
                gaps: table_cell(&cells, indices.gaps),
                active_wi: table_cell(&cells, indices.active_wi),
                evidence: table_cell(&cells, indices.evidence),
            });
            row_idx += 1;
        }
        return rows;
    }
    Vec::new()
}

struct LegacyCapabilityColumnIndices {
    capability: usize,
    current_state: usize,
    gaps: usize,
    active_wi: usize,
    evidence: usize,
}

fn legacy_capability_column_indices(cells: &[String]) -> Option<LegacyCapabilityColumnIndices> {
    Some(LegacyCapabilityColumnIndices {
        capability: find_table_column(cells, &["capability"])?,
        current_state: find_table_column(cells, &["currentstate", "state"])?,
        gaps: find_table_column(cells, &["gaps", "gap"])?,
        active_wi: find_table_column(cells, &["activewi", "activeworkitem", "activeworkitems"])?,
        evidence: find_table_column(cells, &["evidence", "progress", "proof"])?,
    })
}

fn parse_markdown_table_row(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed[1..].contains('|') {
        return None;
    }
    Some(
        trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().replace("\\|", "|"))
            .collect(),
    )
}

fn is_markdown_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            !trimmed.is_empty()
                && trimmed.chars().all(|c| matches!(c, '-' | ':' | ' '))
                && trimmed.chars().any(|c| c == '-')
        })
}

fn table_cell(cells: &[String], idx: usize) -> String {
    cells
        .get(idx)
        .map(|cell| cell.trim().to_string())
        .filter(|cell| !cell.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

fn find_table_column(cells: &[String], aliases: &[&str]) -> Option<usize> {
    cells.iter().position(|cell| {
        let normalized = cell
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect::<String>();
        aliases.iter().any(|alias| normalized == *alias)
    })
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn resolve_capability_path(
    project_root: &Path,
    project: &str,
    override_path: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(if path.is_absolute() {
            path.to_path_buf()
        } else {
            project_root.join(path)
        });
    }
    let row = resolve_project_row(project_root, project)?;
    capability_path_from_row(project_root, &row)
}

fn capability_path_from_row(project_root: &Path, row: &CapabilityProjectRow) -> Result<PathBuf> {
    let path = if let Some(cap_path) = row.cap_path.as_deref() {
        PathBuf::from(cap_path)
    } else if let Some(project_path) = row.path.as_deref() {
        PathBuf::from(project_path).join("README.md")
    } else {
        anyhow::bail!(
            "project '{}' must declare [[projects]].cap_path or [[projects]].path",
            row.name
        );
    };
    Ok(if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    })
}

fn resolve_td_path(project_root: &Path, project: &str) -> Result<PathBuf> {
    let resolved =
        crate::services::project_registry::resolve_td_root_from_config(project_root, project)
            .map_err(|err| anyhow::anyhow!("{}", err.message))?;
    Ok(PathBuf::from(resolved.root))
}

fn resolve_project_row(project_root: &Path, project: &str) -> Result<CapabilityProjectRow> {
    let row = crate::services::project_registry::resolve_project_config_row(project_root, project)?;
    Ok(CapabilityProjectRow {
        name: row.name,
        path: Some(row.path),
        td_path: row.td_path,
        cap_path: row.cap_path,
    })
}

async fn load_project_issues(project_root: &Path, project: &str) -> Result<Vec<Issue>> {
    let project_label = crate::cli::issues::resolve_project_label(project_root, project)
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;
    let (kind, repo, host) = resolve_default_backend(project_root)?;
    let backend =
        make_backend(&kind, project_root, repo, host).context("Failed to create backend")?;
    let filter = IssueFilter {
        state: None,
        issue_type: None,
        label: Some(project_label),
        author: None,
    };
    let mut issues = backend.list(&filter).await?;
    issues.sort_by(|a, b| issue_ref(a).cmp(&issue_ref(b)));
    Ok(issues)
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn collect_td_capability_refs(
    project_root: &Path,
    project: &str,
    document: &CapabilityDocument,
) -> Result<Vec<TdCapabilityEvidence>> {
    let td_root = resolve_td_path(project_root, project)?;
    if !td_root.exists() {
        return Ok(Vec::new());
    }
    let mut refs = Vec::new();
    let mut findings = Vec::new();
    for entry in walkdir::WalkDir::new(&td_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read TD {}", path.display()))?;
        match validate_td_capability_refs_for_content(&content, document) {
            Ok((spec_id, file_refs, file_findings)) => {
                findings.extend(file_findings.into_iter().map(|finding| {
                    format!(
                        "{}: {}",
                        path.strip_prefix(project_root).unwrap_or(path).display(),
                        finding
                    )
                }));
                refs.extend(file_refs.into_iter().map(|td_ref| {
                    TdCapabilityEvidence {
                        spec_path: path
                            .strip_prefix(project_root)
                            .unwrap_or(path)
                            .display()
                            .to_string(),
                        spec_id: spec_id.clone(),
                        review_status: td_review_status_from_content(&content),
                        capability_id: td_ref.id,
                        role: td_ref.role,
                        gap: td_ref.gap,
                        claim: td_ref.claim,
                        coverage: td_ref.coverage,
                        rationale: td_ref.rationale,
                    }
                }));
            }
            Err(err) => findings.push(format!(
                "{}: {}",
                path.strip_prefix(project_root).unwrap_or(path).display(),
                err
            )),
        }
    }
    if !findings.is_empty() {
        anyhow::bail!("{}", findings.join("\n"));
    }
    Ok(refs)
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn validate_td_capability_refs_for_content(
    content: &str,
    document: &CapabilityDocument,
) -> Result<(Option<String>, Vec<TdCapabilityRef>, Vec<String>)> {
    let Some((fm_str, _body)) = split_frontmatter(content) else {
        return Ok((None, Vec::new(), Vec::new()));
    };
    let fm: TdFrontmatter = serde_yaml::from_str(fm_str)
        .context("invalid TD frontmatter while reading capability_refs")?;
    if fm
        .capability_scope
        .as_deref()
        .is_some_and(|scope| scope == "internal")
    {
        return Ok((fm.id, Vec::new(), Vec::new()));
    }
    let mut findings = Vec::new();
    if fm.capability_refs.is_empty() {
        return Ok((fm.id, Vec::new(), findings));
    }
    let capability_ids = document.capability_ids();
    let primary_count = fm
        .capability_refs
        .iter()
        .filter(|td_ref| td_ref.role == CapabilityRefRole::Primary)
        .count();
    if primary_count == 0 {
        findings.push("capability_refs must include at least one primary ref".to_string());
    }
    if primary_count > 1
        && fm
            .capability_refs
            .iter()
            .filter(|td_ref| td_ref.role == CapabilityRefRole::Primary)
            .any(|td_ref| td_ref.gap.is_none())
    {
        findings.push(
            "multiple primary capability_refs require each primary ref to name a gap".to_string(),
        );
    }
    for td_ref in &fm.capability_refs {
        if !capability_ids.contains(&td_ref.id) {
            findings.push(format!("unknown capability id `{}`", td_ref.id));
            continue;
        }
        if let Some(gap) = &td_ref.gap {
            let gaps = document.gap_ids_for(&td_ref.id);
            if !gaps.contains(gap) {
                findings.push(format!(
                    "unknown gap id `{}` for capability `{}`",
                    gap, td_ref.id
                ));
            }
        }
        if document.capability_has_contract(&td_ref.id)
            && td_ref.role == CapabilityRefRole::Primary
            && td_ref.claim.is_none()
        {
            findings.push(format!(
                "primary capability ref for `{}` requires claim because the capability has verification_contract",
                td_ref.id
            ));
        }
        if let Some(claim) = &td_ref.claim {
            let claims = document.claim_ids_for(&td_ref.id);
            if !claims.contains(claim) {
                findings.push(format!(
                    "unknown claim id `{}` for capability `{}`",
                    claim, td_ref.id
                ));
            }
        }
    }
    Ok((fm.id, fm.capability_refs, findings))
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn validate_td_capability_refs_for_spec_path(
    project_root: &Path,
    spec_path: &Path,
    content: &str,
) -> Vec<String> {
    if !content.contains("capability_refs:") && !content.contains("capability_scope:") {
        return Vec::new();
    }
    if content.contains("capability_scope: internal") && !content.contains("capability_refs:") {
        return Vec::new();
    }

    let config_file = project_root.join(".aw").join("config.toml");
    let content_config = match std::fs::read_to_string(&config_file) {
        Ok(content) => content,
        Err(err) => {
            return vec![format!(
                "capability_refs declared but {} could not be read: {}",
                config_file.display(),
                err
            )]
        }
    };
    let parsed: CapabilityConfig = match toml::from_str(&content_config) {
        Ok(parsed) => parsed,
        Err(err) => {
            return vec![format!(
                "capability_refs declared but {} could not be parsed: {}",
                config_file.display(),
                err
            )]
        }
    };

    let spec_abs = canonical_or_join(project_root, spec_path);
    let Some(project_row) = parsed.projects.into_iter().find(|row| {
        let Some(source_path) = row.path.as_deref() else {
            return false;
        };
        let input = crate::services::project_registry::TdRootInput {
            name: row.name.clone(),
            td_path: row.td_path.clone(),
            source_path: source_path.to_string(),
        };
        let Ok(resolved) =
            crate::services::project_registry::resolve_td_root(&input, None, project_root)
        else {
            return false;
        };
        spec_abs.starts_with(PathBuf::from(resolved.root))
    }) else {
        return vec![
            "capability_refs declared but TD path does not match any configured project TD root"
                .to_string(),
        ];
    };

    let cap_path = match capability_path_from_row(project_root, &project_row) {
        Ok(path) => path,
        Err(err) => return vec![err.to_string()],
    };
    let cap_body = match std::fs::read_to_string(&cap_path) {
        Ok(body) => body,
        Err(err) => {
            return vec![format!(
                "capability_refs declared but capability map {} could not be read: {}",
                cap_path.display(),
                err
            )]
        }
    };
    let document = match parse_capability_document(&cap_body, &cap_path) {
        Ok(document) => document,
        Err(err) => {
            return vec![format!(
                "capability_refs declared but capability map {} is invalid: {}",
                cap_path.display(),
                err
            )]
        }
    };
    match validate_td_capability_refs_for_content(content, &document) {
        Ok((_spec_id, _refs, findings)) => findings,
        Err(err) => vec![err.to_string()],
    }
}

fn canonical_or_join(project_root: &Path, path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    };
    std::fs::canonicalize(&absolute).unwrap_or(absolute)
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_open = &trimmed[3..];
    let close = after_open.find("\n---")?;
    let fm = &after_open[..close];
    let body = &after_open[close + 4..];
    Some((fm.trim(), body))
}

fn issue_ref(issue: &Issue) -> String {
    if let Some(id) = issue.github_id.or(issue.gitlab_id) {
        format!("#{id}")
    } else {
        issue.slug.clone()
    }
}

fn extract_hash_numbers(text: &str) -> Vec<u64> {
    let mut numbers = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '#' {
            continue;
        }
        let mut digits = String::new();
        while let Some(next) = chars.peek() {
            if next.is_ascii_digit() {
                digits.push(*next);
                chars.next();
            } else {
                break;
            }
        }
        if let Ok(number) = digits.parse::<u64>() {
            numbers.push(number);
        }
    }
    numbers
}

fn run_verification_command(project_root: &Path, command: &str) -> VerificationRuntimeResult {
    run_verification_command_with_timeout(project_root, command, capability_gate_timeout())
}

fn capability_gate_timeout() -> Duration {
    std::env::var(CAPABILITY_GATE_TIMEOUT_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_CAPABILITY_GATE_TIMEOUT_SECS))
}

fn run_verification_command_with_timeout(
    project_root: &Path,
    command: &str,
    timeout: Duration,
) -> VerificationRuntimeResult {
    let started = Instant::now();
    let stdout_file = match tempfile::NamedTempFile::new() {
        Ok(file) => file,
        Err(err) => return verification_command_error(command, err),
    };
    let stderr_file = match tempfile::NamedTempFile::new() {
        Ok(file) => file,
        Err(err) => return verification_command_error(command, err),
    };
    let stdout = match stdout_file.reopen() {
        Ok(file) => file,
        Err(err) => return verification_command_error(command, err),
    };
    let stderr = match stderr_file.reopen() {
        Ok(file) => file,
        Err(err) => return verification_command_error(command, err),
    };

    let mut command_process = std::process::Command::new("sh");
    crate::cli::shell_env::apply_default_shell_env(&mut command_process);
    configure_capability_verification_process_group(&mut command_process);
    let mut child = match command_process
        .arg("-c")
        .arg(command)
        .current_dir(project_root)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
    {
        Ok(child) => child,
        Err(err) => return verification_command_error(command, err),
    };

    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {}
            Err(err) => return verification_command_error(command, err),
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            terminate_capability_verification_child(&mut child);
            break None;
        }
        thread::sleep(Duration::from_millis(250));
    };

    let stdout_bytes = match fs::read(stdout_file.path()) {
        Ok(bytes) => bytes,
        Err(err) => return verification_command_error(command, err),
    };
    let stderr_bytes = match fs::read(stderr_file.path()) {
        Ok(bytes) => bytes,
        Err(err) => return verification_command_error(command, err),
    };
    let stdout = output_excerpt(&stdout_bytes);
    let mut stderr = output_excerpt(&stderr_bytes);
    if timed_out {
        let timeout_note = format!(
            "aw capability gate timed out after {}s; set {CAPABILITY_GATE_TIMEOUT_ENV} to override",
            timeout.as_secs()
        );
        stderr = Some(match stderr {
            Some(existing) if !existing.trim().is_empty() => format!("{existing}\n{timeout_note}"),
            _ => timeout_note,
        });
    }
    let result_status = if timed_out {
        "timeout"
    } else if output_mentions_env_skip(stdout.as_deref())
        .or_else(|| output_mentions_env_skip(stderr.as_deref()))
        .is_some()
    {
        "env_blocked"
    } else if status
        .as_ref()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        "pass"
    } else {
        "fail"
    };
    VerificationRuntimeResult {
        id: String::new(),
        command: command.to_string(),
        status: result_status.to_string(),
        proves: None,
        exit_code: status.as_ref().and_then(|status| status.code()),
        stdout,
        stderr,
    }
}

fn verification_command_error(
    command: &str,
    err: impl std::fmt::Display,
) -> VerificationRuntimeResult {
    VerificationRuntimeResult {
        id: String::new(),
        command: command.to_string(),
        status: "error".to_string(),
        proves: None,
        exit_code: None,
        stdout: None,
        stderr: Some(err.to_string()),
    }
}

#[cfg(unix)]
fn configure_capability_verification_process_group(command: &mut std::process::Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_capability_verification_process_group(_command: &mut std::process::Command) {}

fn terminate_capability_verification_child(child: &mut std::process::Child) {
    #[cfg(unix)]
    unsafe {
        let pgid = child.id() as i32;
        if pgid > 0 {
            libc::kill(-pgid, libc::SIGTERM);
        }
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }

    let terminate_started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(_) => return,
        }
        if terminate_started.elapsed() >= Duration::from_secs(2) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    #[cfg(unix)]
    unsafe {
        let pgid = child.id() as i32;
        if pgid > 0 {
            libc::kill(-pgid, libc::SIGKILL);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn output_mentions_env_skip(output: Option<&str>) -> Option<()> {
    let output = output?.to_ascii_lowercase();
    [
        "skipping:",
        "skip: ",
        "skipped: ",
        "chromium unavailable",
        "wasm-pack unavailable",
        "node unavailable",
        "missing prerequisites",
    ]
    .iter()
    .any(|needle| output.contains(needle))
    .then_some(())
}

fn run_next_action_command(
    project_root: &Path,
    action: &CapabilityAction,
    tick: usize,
) -> CapabilityRunResult {
    if action.command.trim().is_empty() {
        return CapabilityRunResult {
            tick,
            kind: action.kind,
            command: action.command.clone(),
            executed_command: String::new(),
            status: "skipped_empty_command".to_string(),
            exit_code: None,
            stdout: None,
            stderr: Some(action.reason.clone()),
            hitl_question: None,
        };
    }
    let executed_command = command_for_current_aw_binary(&action.command);
    let mut command_process = std::process::Command::new("sh");
    crate::cli::shell_env::apply_default_shell_env(&mut command_process);
    let output = command_process
        .arg("-c")
        .arg(&executed_command)
        .current_dir(project_root)
        .output();
    match output {
        Ok(output) => CapabilityRunResult {
            tick,
            kind: action.kind,
            command: action.command.clone(),
            executed_command,
            status: if output.status.success() {
                "pass".to_string()
            } else {
                "fail".to_string()
            },
            exit_code: output.status.code(),
            stdout: output_excerpt(&output.stdout),
            stderr: output_excerpt(&output.stderr),
            hitl_question: None,
        },
        Err(err) => CapabilityRunResult {
            tick,
            kind: action.kind,
            command: action.command.clone(),
            executed_command,
            status: "error".to_string(),
            exit_code: None,
            stdout: None,
            stderr: Some(err.to_string()),
            hitl_question: None,
        },
    }
}

fn apply_capability_format_migration_tick(
    tick: usize,
    project_root: &Path,
    project: &str,
    cap_path: Option<&Path>,
    action: &CapabilityAction,
) -> CapabilityRunResult {
    let result = (|| -> Result<String> {
        let resolved = resolve_capability_path(project_root, project, cap_path)?;
        let body = std::fs::read_to_string(&resolved)
            .with_context(|| format!("failed to read {}", resolved.display()))?;
        let (body, document) = match parse_capability_document(&body, &resolved) {
            Ok(document) => (body, document),
            Err(err) if err.to_string().contains("duplicate capability id") => {
                let repaired = strip_previous_canonical_capability_tail(&body).ok_or(err)?;
                let document = parse_capability_document(&repaired, &resolved)?;
                (repaired, document)
            }
            Err(err) => return Err(err),
        };
        if !document.requires_format_migration() {
            return Ok(format!(
                "{} already uses canonical Markdown capability format",
                resolved.display()
            ));
        }
        let migrated = render_capability_markdown_migration(&body, &document, project);
        if migrated != body {
            std::fs::write(&resolved, migrated)
                .with_context(|| format!("failed to write {}", resolved.display()))?;
        }
        Ok(format!("migrated {}", resolved.display()))
    })();

    match result {
        Ok(stdout) => CapabilityRunResult {
            tick,
            kind: action.kind,
            command: action.command.clone(),
            executed_command: action.command.clone(),
            status: "pass".to_string(),
            exit_code: Some(0),
            stdout: Some(stdout),
            stderr: None,
            hitl_question: None,
        },
        Err(err) => CapabilityRunResult {
            tick,
            kind: action.kind,
            command: action.command.clone(),
            executed_command: action.command.clone(),
            status: "fail".to_string(),
            exit_code: Some(1),
            stdout: None,
            stderr: Some(err.to_string()),
            hitl_question: None,
        },
    }
}

fn strip_previous_canonical_capability_tail(body: &str) -> Option<String> {
    let marker = "\n### Capability Index\n";
    let idx = body.rfind(marker)?;
    let mut repaired = body[..idx].trim_end().to_string();
    repaired.push('\n');
    Some(repaired)
}

fn command_for_current_aw_binary(command: &str) -> String {
    let trimmed = command.trim();
    if trimmed == "aw" || trimmed.starts_with("aw ") {
        if let Ok(current_exe) = std::env::current_exe() {
            let suffix = trimmed.strip_prefix("aw").unwrap_or_default();
            return format!(
                "{}{}",
                shell_quote(&current_exe.display().to_string()),
                suffix
            );
        }
    }
    command.to_string()
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl VerificationRuntimeResult {
    fn with_gate(mut self, id: &str, proves: &str) -> Self {
        self.id = id.to_string();
        self.proves = Some(proves.to_string());
        self
    }
}

fn output_excerpt(output: &[u8]) -> Option<String> {
    const OUTPUT_EXCERPT_CHARS: usize = 2_000;
    const OUTPUT_HEAD_CHARS: usize = 800;
    const OUTPUT_TAIL_CHARS: usize = OUTPUT_EXCERPT_CHARS - OUTPUT_HEAD_CHARS;

    let text = String::from_utf8_lossy(output).trim().to_string();
    if text.is_empty() {
        None
    } else if text.chars().count() <= OUTPUT_EXCERPT_CHARS {
        Some(text)
    } else {
        let head = text.chars().take(OUTPUT_HEAD_CHARS).collect::<String>();
        let tail = text
            .chars()
            .rev()
            .take(OUTPUT_TAIL_CHARS)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        Some(format!("{head}\n...[truncated]...\n{tail}"))
    }
}

fn print_report(report: &CapabilityReport, human: bool, pretty: bool) -> Result<()> {
    if !human {
        if pretty {
            println!("{}", serde_json::to_string_pretty(report)?);
        } else {
            println!("{}", serde_json::to_string(report)?);
        }
        return Ok(());
    }
    println!(
        "capability: {} ({}) {:.1}% [{}/{} verified]",
        report.project,
        report.status,
        report.percent,
        report.verified_count,
        report.capability_count
    );
    for blocker in &report.blockers {
        println!("blocker: {blocker}");
    }
    for warning in &report.warnings {
        println!("warning: {warning}");
    }
    println!(
        "test gates: {:?} [{}/{} passed]",
        report.test_gates.status, report.test_gates.passed_count, report.test_gates.command_count
    );
    print_next_action(&report.next_action);
    Ok(())
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
fn capability_summary(report: &CapabilityReport, include_run_results: bool) -> serde_json::Value {
    let mut summary = serde_json::json!({
        "schema_version": "aw.cli.v1",
        "status": capability_loop_status(report),
        "action": report.action,
        "project": &report.project,
        "cap_path": report.cap_path.to_string_lossy(),
        "report_status": &report.status,
        "completion": capability_completion(report),
        "next": capability_next(report),
        "coverage": capability_coverage_summary(report),
        "next_action": &report.next_action,
    });
    if include_run_results && !report.run_results.is_empty() {
        summary
            .as_object_mut()
            .expect("capability summary is an object")
            .insert(
                "run_results".to_string(),
                serde_json::to_value(&report.run_results).expect("serialize run results"),
            );
    }
    summary
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
fn capability_loop_status(report: &CapabilityReport) -> &'static str {
    if capability_workflow_complete(report) {
        "done"
    } else if report.next_action.requires_hitl
        || report.next_action.kind == CapabilityActionKind::EnvBlocked
    {
        "blocked"
    } else {
        "continue"
    }
}

fn capability_workflow_complete(report: &CapabilityReport) -> bool {
    report.status == "healthy"
        && report.blockers.is_empty()
        && report.next_action.kind == CapabilityActionKind::None
        && report.verified_count == report.capability_count
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
fn capability_completion(report: &CapabilityReport) -> serde_json::Value {
    let workflow_complete = capability_workflow_complete(report);
    serde_json::json!({
        "root_complete": workflow_complete,
        "workflow_complete": workflow_complete,
        "requires_hitl": report.next_action.requires_hitl || report.next_action.kind == CapabilityActionKind::EnvBlocked,
        "criteria": [
            "capability format is valid",
            "non-retired capabilities are runtime verified",
            "required capability claims have TD verification linkage"
        ],
        "missing": capability_missing(report),
    })
}

fn capability_missing(report: &CapabilityReport) -> Vec<String> {
    if capability_workflow_complete(report) {
        return Vec::new();
    }
    let mut missing = Vec::new();
    let mut seen = BTreeSet::new();
    for blocker in &report.blockers {
        push_missing_once(&mut missing, &mut seen, blocker.clone());
    }
    for blocker in &report.production_blockers {
        push_missing_once(&mut missing, &mut seen, blocker.clone());
    }
    if !report.next_action.reason.is_empty() {
        push_missing_once(&mut missing, &mut seen, report.next_action.reason.clone());
    }
    if report.verified_count < report.capability_count {
        push_missing_once(
            &mut missing,
            &mut seen,
            format!(
                "{} of {} non-retired capabilities are not runtime verified",
                report.capability_count - report.verified_count,
                report.capability_count
            ),
        );
    }
    missing
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
fn push_missing_once(missing: &mut Vec<String>, seen: &mut BTreeSet<String>, value: String) {
    if seen.insert(value.clone()) {
        missing.push(value);
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
fn capability_next(report: &CapabilityReport) -> serde_json::Value {
    let action = &report.next_action;
    let command = (!action.command.trim().is_empty()).then_some(action.command.trim());
    let mut next = serde_json::Map::new();
    next.insert(
        "kind".to_string(),
        serde_json::Value::String(capability_next_kind(report, command.is_some()).to_string()),
    );
    if let Some(command) = command {
        next.insert(
            "command".to_string(),
            serde_json::Value::String(command.to_string()),
        );
    }
    next.insert(
        "reason".to_string(),
        serde_json::Value::String(action.reason.clone()),
    );
    serde_json::Value::Object(next)
}

fn capability_next_kind(report: &CapabilityReport, has_command: bool) -> &'static str {
    if capability_workflow_complete(report) {
        "done"
    } else if report.next_action.requires_hitl {
        "hitl"
    } else if report.next_action.kind == CapabilityActionKind::EnvBlocked {
        "blocked"
    } else if has_command {
        "run_command"
    } else {
        "error"
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
fn capability_coverage_summary(report: &CapabilityReport) -> serde_json::Value {
    serde_json::json!({
        "format_version": report.format_version,
        "capability_count": report.capability_count,
        "verified_count": report.verified_count,
        "percent": report.percent,
        "claim_count": report.claim_count,
        "verified_claim_count": report.verified_claim_count,
        "claim_percent": report.claim_percent,
        "blocker_count": report.blockers.len(),
        "warning_count": report.warnings.len(),
        "production_ready": report.production_ready,
        "production_status": &report.production_status,
        "test_gate_status": &report.test_gates.status,
    })
}

fn print_next_action(action: &CapabilityAction) {
    println!("next: {:?} {}", action.kind, action.target);
    if !action.command.is_empty() {
        println!("command: {}", action.command);
    }
    println!("reason: {}", action.reason);
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn capability_rows_for_wi_plan(
    document: &CapabilityDocument,
    td_refs: &[TdCapabilityEvidence],
) -> Result<
    Vec<(
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
    )>,
> {
    if document.is_legacy_only() {
        anyhow::bail!(
            "legacy capability table detected; migrate README to `## Capability:` sections before planning WIs"
        );
    }
    let mut rows = Vec::new();
    for capability in &document.capabilities {
        if let Some(contract) = capability.verification_contract.as_ref() {
            for claim in contract
                .claims
                .iter()
                .filter(|claim| claim.required_for_verified)
            {
                let has_primary_td = td_refs.iter().any(|td| {
                    td.capability_id == capability.id
                        && td.role == CapabilityRefRole::Primary
                        && td.claim.as_deref() == Some(&claim.id)
                });
                let first_gate = claim
                    .gates
                    .first()
                    .map(|gate| gate.command.clone())
                    .unwrap_or_else(|| {
                        "Add at least one concrete verification gate to this claim.".to_string()
                    });
                rows.push((
                    capability.title.clone(),
                    capability.current_state.clone(),
                    if has_primary_td {
                        "none".to_string()
                    } else {
                        format!("claim {}: {}", claim.id, claim.user_story)
                    },
                    active_wi_for_capability(capability),
                    format!(
                        "claim gate: {}; oracle: {}; maturity: {:?}",
                        first_gate, claim.oracle, claim.maturity
                    ),
                    Some(claim.id.clone()),
                    Some(claim.user_story.clone()),
                ));
            }
            continue;
        }
        rows.push({
            let gap_summary = capability
                .gaps
                .iter()
                .filter(|gap| {
                    !matches!(
                        gap.status,
                        CapabilityGapStatus::Closed | CapabilityGapStatus::Deferred
                    )
                })
                .map(|gap| gap.summary.clone())
                .collect::<Vec<_>>()
                .join("; ");
            let active_wi = capability
                .gaps
                .iter()
                .filter_map(|gap| gap.active_wi.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let evidence = summarize_evidence(&capability.evidence);
            (
                capability.title.clone(),
                capability.current_state.clone(),
                if gap_summary.is_empty() {
                    "none".to_string()
                } else {
                    gap_summary
                },
                if active_wi.is_empty() {
                    "none".to_string()
                } else {
                    active_wi
                },
                evidence,
                None,
                None,
            )
        });
    }
    Ok(rows)
}

fn active_wi_for_capability(capability: &CapabilitySection) -> String {
    let active_wi = capability
        .gaps
        .iter()
        .filter_map(|gap| gap.active_wi.clone())
        .collect::<Vec<_>>()
        .join(", ");
    if active_wi.is_empty() {
        "none".to_string()
    } else {
        active_wi
    }
}

fn summarize_evidence(evidence: &CapabilityEvidence) -> String {
    let mut parts = Vec::new();
    if !evidence.source.is_empty() {
        parts.push(format!("source: {}", evidence.source.join(", ")));
    }
    if !evidence.td.is_empty() {
        parts.push(format!("td: {}", evidence.td.join(", ")));
    }
    if !evidence.cb.is_empty() {
        parts.push(format!("cb: {}", evidence.cb.join(", ")));
    }
    if !evidence.verification.is_empty() {
        parts.push(format!(
            "verification: {}",
            evidence
                .verification
                .iter()
                .map(|gate| gate.id.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cap_doc(body: &str) -> CapabilityDocument {
        parse_capability_document(body, Path::new("README.md")).unwrap()
    }

    fn canonical_doc(body: &str) -> CapabilityDocument {
        let mut document = cap_doc(body);
        document.needs_canonicalization = false;
        document
    }

    /// A capability-type map that assigns `Service` to every capability id in
    /// the report and document, so the `assign_capability_type` HITL check is
    /// satisfied and these tests exercise their original next-action path.
    fn all_typed(
        report: &CapabilityReport,
        document: &CapabilityDocument,
    ) -> BTreeMap<String, crate::cli::capability_type::CapabilityType> {
        report
            .capabilities
            .iter()
            .map(|item| item.id.clone())
            .chain(document.capabilities.iter().map(|cap| cap.id.clone()))
            .map(|id| (id, crate::cli::capability_type::CapabilityType::Service))
            .collect()
    }

    fn sample_report(next_action: CapabilityAction) -> CapabilityReport {
        CapabilityReport {
            action: "capability",
            project: "jet".to_string(),
            cap_path: PathBuf::from("projects/jet/README.md"),
            format_version: 1,
            status: "blocked".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("jet"),
            production_ready: false,
            production_status: ProductionStatus::NotEvaluated,
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            capability_count: 1,
            verified_count: 0,
            percent: 0.0,
            claim_count: 0,
            verified_claim_count: 0,
            claim_percent: 0.0,
            capabilities: Vec::new(),
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_action,
            run_results: Vec::new(),
        }
    }

    fn sample_action(
        kind: CapabilityActionKind,
        command: &str,
        requires_hitl: bool,
    ) -> CapabilityAction {
        CapabilityAction {
            kind,
            capability_id: Some("package-manager".to_string()),
            gap_id: Some("package-manager-readiness".to_string()),
            claim_id: None,
            target: "Package Manager".to_string(),
            command: command.to_string(),
            reason: "active WI exists; continue WI -> TD -> CB lifecycle".to_string(),
            requires_hitl,
            hitl_question: None,
        }
    }

    fn sample_report_item_with_gap(active_wi: Option<&str>) -> CapabilityReportItem {
        CapabilityReportItem {
            id: "package-manager".to_string(),
            title: "Package Manager".to_string(),
            status: CapabilityStatus::Auditing,
            capability_type: None,
            surfaces: Vec::new(),
            ec_dimensions: Vec::new(),
            promise: "Replace package manager flows.".to_string(),
            current_state: "Install surface exists.".to_string(),
            gaps: vec![CapabilityGap {
                id: "package-manager-readiness".to_string(),
                status: CapabilityGapStatus::Open,
                active_wi: active_wi.map(str::to_string),
                summary: "Readiness audit pending.".to_string(),
            }],
            td_refs: Vec::new(),
            wi_refs: active_wi.into_iter().map(str::to_string).collect(),
            wi_evidence: Vec::new(),
            claims: Vec::new(),
            claim_count: 0,
            verified_claim_count: 0,
            claim_percent: 0.0,
            verification: Vec::new(),
            verified: false,
            release_scope: false,
            full_regenerability_required: false,
            dependencies: Vec::new(),
            dependency_closure: Vec::new(),
            production_ready: false,
            production_blockers: Vec::new(),
        }
    }

    #[test]
    fn empty_capability_map_is_actionable_document_state() {
        let doc = cap_doc("# Mamba\n\n## Capabilities\n\n");

        assert_eq!(doc.format, CapabilityDocumentFormat::Empty);
        assert_eq!(doc.format_version(), 0);
        assert!(doc.capabilities.is_empty());
        assert!(doc.legacy_rows.is_empty());
        assert!(doc.prose_candidates.is_empty());
        assert!(doc
            .findings
            .iter()
            .any(|finding| finding.contains("no capability sections found")));
    }

    #[test]
    fn prose_capability_headings_are_candidate_input_not_contracts() {
        let doc = cap_doc(
            r#"# Mamba

## Brief

Python runtime.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|

### C1. Py3.12 functional parity - Axis 1 ([#3331](https://github.com/chrischeng-c4/cclab/issues/3331))

Mamba can execute the Python 3.12 language and standard library surface with CPython-compatible behavior.

#### Gates

- CPython fixture corpus

### C2. Less CPU time AND less memory than CPython - Axis 2 ([#3880](https://github.com/chrischeng-c4/cclab/issues/3880))

Mamba should improve both runtime CPU and memory profile on selected workloads.
"#,
        );

        assert_eq!(doc.format, CapabilityDocumentFormat::Empty);
        assert_eq!(doc.format_version(), 0);
        assert!(doc.capabilities.is_empty());
        assert_eq!(doc.prose_candidates.len(), 2);
        assert_eq!(
            doc.prose_candidates[0].id,
            "c1-py3-12-functional-parity-axis-1-3331"
        );
        assert_eq!(
            doc.prose_candidates[0].title,
            "C1. Py3.12 functional parity - Axis 1 (#3331)"
        );
        assert_eq!(doc.prose_candidates[0].root_wi.as_deref(), Some("#3331"));
        assert!(doc.prose_candidates[0]
            .summary
            .as_deref()
            .unwrap()
            .contains("Python 3.12 language"));
        assert!(doc
            .findings
            .iter()
            .any(|finding| finding.contains("candidate prose capability roots detected")));
    }

    #[test]
    fn empty_capability_map_next_action_requires_hitl_definition() {
        let document = cap_doc("# Cue\n\n## Capabilities\n\n");
        let mut report = sample_report(sample_action(CapabilityActionKind::None, "", false));
        report.capability_count = 0;
        report.verified_count = 0;
        report.format_version = document.format_version();
        report.blockers = document.findings.clone();

        let action = choose_next_action(&report, &document, &BTreeMap::new());

        assert_eq!(action.kind, CapabilityActionKind::DefineCapabilityMap);
        assert_eq!(action.command, "aw capability draft --project jet");
        assert!(action.requires_hitl);
        assert_eq!(
            action
                .hitl_question
                .as_ref()
                .unwrap()
                .default_choice
                .as_deref(),
            Some("define_roots")
        );
    }

    #[test]
    fn prose_candidates_are_in_define_map_hitl_prompt() {
        let document = cap_doc(
            r#"# Mamba

## Capabilities

### C1. Py3.12 functional parity - Axis 1 (#3331)

Mamba can execute the Python 3.12 language and standard library surface.
"#,
        );
        let mut report = sample_report(sample_action(CapabilityActionKind::None, "", false));
        report.capability_count = 0;
        report.verified_count = 0;
        report.format_version = document.format_version();
        report.blockers = document.findings.clone();

        let action = choose_next_action(&report, &document, &BTreeMap::new());
        let prompt = action
            .hitl_question
            .as_ref()
            .and_then(|question| question.freeform_prompt.as_deref())
            .unwrap();

        assert_eq!(action.kind, CapabilityActionKind::DefineCapabilityMap);
        assert_eq!(action.command, "aw capability draft --project jet");
        assert!(prompt.contains("Candidate README capability roots detected"));
        assert!(prompt.contains("C1. Py3.12 functional parity"));
        assert!(prompt.contains("#3331"));
        assert!(prompt.contains("inference only"));
    }

    #[test]
    fn capability_map_draft_artifact_is_pending_review_not_readme_mutation() {
        let candidates = vec![CapabilityProseCandidate {
            id: "c1-py3-12-functional-parity-axis-1-3331".to_string(),
            title: "C1. Py3.12 functional parity - Axis 1 (#3331)".to_string(),
            line: 11,
            root_wi: Some("#3331".to_string()),
            summary: Some("Run real Python 3.12 programs without semantic divergence.".to_string()),
        }];

        let artifact = render_capability_map_draft(
            "mamba",
            Path::new("projects/mamba/README.md"),
            &candidates,
        );

        assert!(artifact.contains("kind: capability_map_draft"));
        assert!(artifact.contains("status: pending_review"));
        assert!(artifact.contains("source: prose_candidates"));
        assert!(artifact.contains("## Draft Canonical README Section"));
        assert!(artifact.contains("ID: c1-py3-12-functional-parity-axis-1-3331"));
        assert!(artifact.contains(
            "Type: (confirm capability type: AgentFirst, Service, Devops, DeveloperTool, RuntimeTool, or SecurityTool)"
        ));
        assert!(artifact.contains(
            "Surfaces:\n- (confirm public surface, e.g. CLI: `command` - short summary)"
        ));
        assert!(artifact.contains(
            "EC Dimensions:\n- (confirm EC dimension, e.g. behavior: `runner command` - contract summary)"
        ));
        assert!(artifact.contains("Root WI: #3331"));
        assert!(artifact.contains("Status: candidate"));
        assert!(artifact.contains("(confirm gate inventory)"));
        assert!(artifact.contains("This artifact is inference only"));
    }

    #[test]
    fn empty_capability_map_draft_artifact_is_definition_worksheet() {
        let artifact = render_capability_map_draft("cue", Path::new("projects/cue/README.md"), &[]);

        assert!(artifact.contains("kind: capability_map_draft"));
        assert!(artifact.contains("source: empty_capability_map"));
        assert!(artifact.contains("candidate_count: 0"));
        assert!(artifact.contains("README has no candidate capability roots"));
        assert!(artifact.contains("### Cue Capability"));
        assert!(artifact.contains("ID: cue-capability"));
        assert!(artifact.contains(
            "Type: (confirm capability type: AgentFirst, Service, Devops, DeveloperTool, RuntimeTool, or SecurityTool)"
        ));
        assert!(artifact.contains("Root WI: -"));
        assert!(artifact.contains(
            "EC Dimensions:\n- (confirm EC dimension, e.g. behavior: `runner command` - contract summary)"
        ));
        assert!(artifact.contains("Promise:\n(confirm product promise)"));
        assert!(artifact.contains("This artifact is inference only"));
    }

    #[test]
    fn apply_draft_rejects_unreviewed_placeholders() {
        let artifact = render_capability_map_draft("cue", Path::new("projects/cue/README.md"), &[]);

        let err = extract_reviewed_draft_registry(&artifact).unwrap_err();

        assert!(err.to_string().contains("placeholders"));
    }

    #[test]
    fn apply_reviewed_draft_replaces_capabilities_section() {
        let draft = r#"# Cue Capability Map Draft

## Draft Canonical README Section

```md
## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Workflow Control Plane | #3893 | planned | planned | smoke | not_ready | human confirmed |

### Workflow Control Plane

ID: workflow-control-plane
Type: DeveloperTool
Root WI: #3893
Status: confirmed
Required Verification: smoke
Promise:
Cue provides a team workflow control plane over AW Core concepts.
Gate Inventory:
- projects/cue/tests/workflow-control-plane.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Workflow control plane readiness | epic | #3893 | planned | planned | smoke | projects/cue/tests/workflow-control-plane.md |
```
"#;
        let registry = extract_reviewed_draft_registry(draft).unwrap();
        let original = r#"# Cue

Team workflow product.

## Capabilities

old placeholder

## Notes

Keep this section.
"#;

        let applied = apply_capability_registry_to_readme(original, &registry, "cue").unwrap();
        let doc = cap_doc(&applied);

        assert!(applied.contains("## Brief\n\nTeam workflow product."));
        assert!(applied.contains("## Notes\n\nKeep this section."));
        assert!(!applied.contains("old placeholder"));
        assert_eq!(doc.capabilities.len(), 1);
        assert_eq!(doc.capabilities[0].id, "workflow-control-plane");
        assert_eq!(
            doc.capabilities[0].capability_type,
            Some(CapabilityType::DeveloperTool)
        );
    }

    #[test]
    fn draft_commands_preserve_cap_path_override() {
        let cap_path = Path::new("/tmp/aw draft/cue README.md");
        let draft_path = Path::new("/tmp/aw draft/cue capability draft.md");

        let apply = capability_apply_draft_command("cue", draft_path, Some(cap_path));
        let check = capability_check_command("cue", Some(cap_path));

        assert_eq!(
            apply,
            "aw capability apply-draft --project cue --draft '/tmp/aw draft/cue capability draft.md' --cap-path '/tmp/aw draft/cue README.md' --reviewed"
        );
        assert_eq!(
            check,
            "aw capability check --project cue --cap-path '/tmp/aw draft/cue README.md'"
        );
    }

    #[test]
    fn legacy_format_next_action_uses_explicit_migrate_command() {
        let document = cap_doc(one_capability());
        let mut report = sample_report(sample_action(CapabilityActionKind::None, "", false));
        report.format_version = document.format_version();
        report.capabilities = Vec::new();
        report.capability_count = document.legacy_rows.len();
        report.blockers = document.findings.clone();

        let action = choose_next_action(&report, &document, &BTreeMap::new());

        assert_eq!(action.kind, CapabilityActionKind::FormatMigrationRequired);
        assert!(!action.requires_hitl);
        assert_eq!(action.command, "aw capability migrate --project jet");
    }

    #[test]
    fn create_wi_next_action_keeps_unavailable_inventory_review_only() {
        let document = canonical_doc(one_field_markdown_capability());
        let mut report = sample_report(sample_action(CapabilityActionKind::None, "", false));
        report.format_version = document.format_version();
        report.capabilities = vec![sample_report_item_with_gap(None)];
        report
            .warnings
            .push("issue inventory unavailable: gh auth missing".to_string());

        let types = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &types);

        assert_eq!(action.kind, CapabilityActionKind::CreateWi);
        assert_eq!(action.command, "aw wi plan --project jet");
        assert!(action.reason.contains("issue inventory unavailable"));
        assert!(action.reason.contains("local/review-only"));
        assert!(!action.requires_hitl);
    }

    #[test]
    fn noncanonical_markdown_reports_v1_until_migrated() {
        let document = cap_doc(one_markdown_capability());
        let mut report = sample_report(sample_action(CapabilityActionKind::None, "", false));
        report.format_version = document.format_version();
        report.capability_count = document.capabilities.len();

        let action = choose_next_action(&report, &document, &BTreeMap::new());

        assert_eq!(document.format, CapabilityDocumentFormat::MarkdownTables);
        assert!(document.requires_format_migration());
        assert_eq!(document.format_version(), 1);
        assert_eq!(action.kind, CapabilityActionKind::FormatMigrationRequired);
        assert_eq!(action.command, "aw capability migrate --project jet");
    }

    #[test]
    fn canonical_h3_field_style_markdown_reports_v2() {
        let body = r#"# demo

## Brief

Demo project.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Package Manager | #3779 | partial | planned | conformance | not_ready | install flow |

### Package Manager

ID: package-manager
Type: DeveloperTool
Surfaces: CLI: `jet install` - package install surface
Root WI: #3779
Status: auditing
Required Verification: smoke, conformance
Promise:
Replace package manager flows.
Gate Inventory:
- projects/jet/validation/pkg-manager.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |
"#;
        let document = cap_doc(body);

        assert_eq!(document.format, CapabilityDocumentFormat::MarkdownTables);
        assert!(!document.requires_format_migration());
        assert_eq!(document.format_version(), 2);
        assert_eq!(
            document.capabilities[0].capability_type,
            Some(CapabilityType::DeveloperTool)
        );
        assert_eq!(document.capabilities[0].surfaces[0].kind, "CLI");
    }

    #[test]
    fn unreadable_capability_map_report_requires_config_hitl() {
        let report = capability_map_read_blocked_report(
            "pg",
            PathBuf::from("projects/pgkit/README.md"),
            ProjectTestGateReport::not_evaluated("pg"),
            "failed to read capability map: No such file or directory".to_string(),
        );

        assert_eq!(report.status, "blocked");
        assert_eq!(report.next_action.kind, CapabilityActionKind::EnvBlocked);
        assert!(report.next_action.requires_hitl);
        assert_eq!(
            report
                .next_action
                .hitl_question
                .as_ref()
                .unwrap()
                .default_choice
                .as_deref(),
            Some("fix_config")
        );
        let summary = capability_summary(&report, false);
        assert_eq!(summary["status"].as_str(), Some("blocked"));
        assert_eq!(summary["next"]["kind"].as_str(), Some("hitl"));
    }

    #[test]
    fn missing_capability_parent_reports_stale_project_config() {
        let missing_capability_path = PathBuf::from("__missing_capability_parent__/README.md");
        let report = capability_map_read_failure_report(
            "cclab-array",
            missing_capability_path,
            ProjectTestGateReport::not_evaluated("cclab-array"),
            "failed to read capability map: No such file or directory".to_string(),
        );

        assert_eq!(report.status, "blocked");
        assert_eq!(
            report.next_action.kind,
            CapabilityActionKind::StaleProjectConfig
        );
        assert_eq!(
            capability_action_kind_label(report.next_action.kind),
            "stale_project_config"
        );
        assert!(report.next_action.requires_hitl);
        assert!(report
            .next_action
            .reason
            .contains("configured capability map parent directory does not exist"));
        let summary = capability_summary(&report, false);
        assert_eq!(summary["status"].as_str(), Some("blocked"));
        assert_eq!(summary["next"]["kind"].as_str(), Some("hitl"));
    }

    #[test]
    fn capability_summary_exposes_aw_cli_next_contract() {
        let report = sample_report(sample_action(
            CapabilityActionKind::RunTd,
            "aw td create 3779",
            false,
        ));

        let summary = capability_summary(&report, false);

        assert_eq!(summary["schema_version"].as_str(), Some("aw.cli.v1"));
        assert_eq!(summary["status"].as_str(), Some("continue"));
        assert_eq!(summary["report_status"].as_str(), Some("blocked"));
        assert_eq!(
            summary["completion"]["workflow_complete"].as_bool(),
            Some(false)
        );
        assert_eq!(summary["next"]["kind"].as_str(), Some("run_command"));
        assert_eq!(
            summary["next"]["command"].as_str(),
            Some("aw td create 3779")
        );
        assert!(summary.get("run_results").is_none());
    }

    #[test]
    fn capability_summary_marks_hitl_as_blocked() {
        let report = sample_report(sample_action(
            CapabilityActionKind::HumanConfirmRequired,
            "aw capability report --project jet --verify",
            true,
        ));

        let summary = capability_summary(&report, false);

        assert_eq!(summary["status"].as_str(), Some("blocked"));
        assert_eq!(summary["completion"]["requires_hitl"].as_bool(), Some(true));
        assert_eq!(summary["next"]["kind"].as_str(), Some("hitl"));
        assert_eq!(
            summary["next"]["command"].as_str(),
            Some("aw capability report --project jet --verify")
        );
    }

    #[test]
    fn capability_summary_keeps_issue_inventory_warning_out_of_missing() {
        let mut report = sample_report(sample_action(
            CapabilityActionKind::RunTd,
            "aw td create 3779",
            false,
        ));
        report
            .warnings
            .push("issue inventory unavailable: gh auth missing".to_string());

        let summary = capability_summary(&report, false);
        let missing = summary["completion"]["missing"].as_array().unwrap();

        assert_eq!(summary["coverage"]["blocker_count"].as_u64(), Some(0));
        assert_eq!(summary["coverage"]["warning_count"].as_u64(), Some(1));
        assert!(!missing.iter().any(|value| value
            .as_str()
            .is_some_and(|missing| missing.contains("issue inventory unavailable"))));
    }

    #[test]
    fn capability_sweep_groups_by_report_status_and_next_action() {
        let mut healthy = sample_report(sample_action(CapabilityActionKind::None, "", false));
        healthy.status = "healthy".to_string();
        healthy.verified_count = healthy.capability_count;

        let mut blocked = sample_report(sample_action(
            CapabilityActionKind::EnvBlocked,
            "aw capability report --project pg",
            true,
        ));
        blocked.project = "pg".to_string();

        let sweep = capability_sweep_report(&[healthy, blocked], false, false);

        assert_eq!(sweep.status, "blocked");
        assert_eq!(sweep.project_count, 2);
        assert_eq!(sweep.verified_project_count, 1);
        assert_eq!(
            sweep
                .groups
                .iter()
                .map(|group| format!(
                    "{}:{}:{}",
                    group.status, group.next_action_kind, group.count
                ))
                .collect::<Vec<_>>(),
            vec!["blocked:env_blocked:1", "healthy:none:1"]
        );
    }

    #[test]
    fn capability_sweep_splits_define_map_draft_from_report() {
        let mut draftable = sample_report(sample_action(
            CapabilityActionKind::DefineCapabilityMap,
            "aw capability draft --project mamba",
            false,
        ));
        draftable.project = "mamba".to_string();

        let mut inspect_only = sample_report(sample_action(
            CapabilityActionKind::DefineCapabilityMap,
            "aw capability report --project pg",
            true,
        ));
        inspect_only.project = "pg".to_string();

        let sweep = capability_sweep_report(&[draftable, inspect_only], false, false);

        assert_eq!(
            sweep
                .groups
                .iter()
                .map(|group| format!(
                    "{}:{}:{}",
                    group.status, group.next_action_group, group.count
                ))
                .collect::<Vec<_>>(),
            vec![
                "blocked:define_capability_map:draft:1",
                "blocked:define_capability_map:report:1"
            ]
        );
        assert_eq!(sweep.groups[0].next_action_kind, "define_capability_map");
        assert_eq!(sweep.groups[0].next_action_detail, Some("draft"));
        assert_eq!(sweep.groups[1].next_action_detail, Some("report"));

        let draft_projects = capability_sweep_draft_projects(&sweep.projects)
            .into_iter()
            .map(|project| project.project.as_str())
            .collect::<Vec<_>>();
        assert_eq!(draft_projects, vec!["mamba"]);
    }

    #[test]
    fn capability_sweep_selects_create_wi_projects_for_wi_plans() {
        let mut planable = sample_report(sample_action(
            CapabilityActionKind::CreateWi,
            "aw wi plan --project lumen",
            false,
        ));
        planable.project = "lumen".to_string();
        planable.cap_path = PathBuf::from("projects/lumen/README.md");

        let mut td_ready = sample_report(sample_action(
            CapabilityActionKind::RunTd,
            "aw td create 3783",
            false,
        ));
        td_ready.project = "jet".to_string();

        let sweep = capability_sweep_report(&[planable, td_ready], false, false);

        let plan_projects = capability_sweep_wi_plan_projects(&sweep.projects)
            .into_iter()
            .map(|project| {
                (
                    project.project.as_str(),
                    project.cap_path.display().to_string(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            plan_projects,
            vec![("lumen", "projects/lumen/README.md".to_string())]
        );
    }

    #[test]
    fn capability_sweep_action_queue_selects_executable_next_actions() {
        let mut td_ready = sample_report(sample_action(
            CapabilityActionKind::RunTd,
            "aw td create 3783",
            false,
        ));
        td_ready.project = "jet".to_string();

        let mut verify_ready = sample_report(sample_action(
            CapabilityActionKind::RunVerify,
            "aw capability report --project meter --verify",
            false,
        ));
        verify_ready.project = "meter".to_string();

        let mut planable = sample_report(sample_action(
            CapabilityActionKind::CreateWi,
            "aw wi plan --project lumen",
            false,
        ));
        planable.project = "lumen".to_string();

        let mut draftable = sample_report(sample_action(
            CapabilityActionKind::DefineCapabilityMap,
            "aw capability draft --project mamba",
            true,
        ));
        draftable.project = "mamba".to_string();

        let sweep =
            capability_sweep_report(&[td_ready, verify_ready, planable, draftable], false, false);

        let queue = capability_sweep_action_queue(&sweep.projects)
            .into_iter()
            .map(|entry| (entry.project, entry.action_group, entry.command))
            .collect::<Vec<_>>();

        assert_eq!(
            queue,
            vec![
                (
                    "jet".to_string(),
                    "run_td".to_string(),
                    "aw td create 3783".to_string()
                ),
                (
                    "meter".to_string(),
                    "run_verify".to_string(),
                    "aw capability report --project meter --verify".to_string()
                ),
            ]
        );
    }

    #[test]
    fn capability_sweep_draft_index_lists_review_queue() {
        let index = render_capability_sweep_draft_index(&[CapabilityDraftReport {
            schema_version: "aw.cli.v1",
            action: "capability_draft",
            project: "pg".to_string(),
            cap_path: PathBuf::from("projects/pg/README.md"),
            path: PathBuf::from("/tmp/aw/pg/capability-map-drafts/draft.md"),
            status: "pending_review".to_string(),
            source: "empty_capability_map",
            candidate_count: 0,
            agent_review_required: true,
            review_status: "pending",
            apply_command: "aw capability apply-draft --project pg --draft '/tmp/aw/pg/capability-map-drafts/draft.md' --reviewed".to_string(),
            check_command: "aw capability check --project pg".to_string(),
        }]);

        assert!(index.contains("kind: capability_map_draft_index"));
        assert!(index.contains("draft_count: 1"));
        assert!(index.contains("| pg | empty_capability_map | 0 |"));
        assert!(index.contains("/tmp/aw/pg/capability-map-drafts/draft.md"));
        assert!(index.contains(
            "`aw capability apply-draft --project pg --draft '/tmp/aw/pg/capability-map-drafts/draft.md' --reviewed`"
        ));
        assert!(index.contains("`aw capability check --project pg`"));
        assert!(index.contains("Do not edit README until the capability promise is confirmed."));
    }

    #[test]
    fn capability_sweep_wi_plan_index_lists_review_queue() {
        let index =
            render_capability_sweep_wi_plan_index(&[crate::cli::issues::CapabilityWiPlanReport {
                action: "planned",
                kind: "capability_plan",
                project: "lumen".to_string(),
                backend: "unavailable".to_string(),
                path: PathBuf::from("/tmp/aw/lumen/capability-plan/plan.md"),
                cap_path: PathBuf::from("projects/lumen/README.md"),
                capability_count: 17,
                planning_row_count: 54,
                issue_count: 0,
                candidate_count: 48,
                warnings: vec!["issue inventory unavailable: gh auth missing".to_string()],
                agent_review_required: true,
                review_status: "pending",
                plan_command: "aw wi plan --project lumen".to_string(),
            }]);

        assert!(index.contains("kind: capability_wi_plan_index"));
        assert!(index.contains("plan_count: 1"));
        assert!(index.contains("| lumen | unavailable | 48 |"));
        assert!(index.contains("/tmp/aw/lumen/capability-plan/plan.md"));
        assert!(index.contains("`aw wi plan --project lumen`"));
        assert!(index.contains("keep the artifact local/review-only"));
    }

    #[test]
    fn capability_sweep_action_queue_index_lists_executable_commands() {
        let index = render_capability_sweep_action_queue_index(&[
            CapabilityActionQueueEntry {
                project: "jet".to_string(),
                action_kind: "run_td",
                action_group: "run_td".to_string(),
                target: "WASM And Multi-Target Execution".to_string(),
                command: "aw td create 3783".to_string(),
                reason: "active WI exists; continue WI -> TD -> CB lifecycle".to_string(),
            },
            CapabilityActionQueueEntry {
                project: "meter".to_string(),
                action_kind: "run_verify",
                action_group: "run_verify".to_string(),
                target: "Runtime Resource Attribution".to_string(),
                command: "aw capability report --project meter --verify".to_string(),
                reason: "runtime verification must be rerun".to_string(),
            },
        ]);

        assert!(index.contains("kind: capability_action_queue"));
        assert!(index.contains("action_count: 2"));
        assert!(index
            .contains("| jet | run_td | WASM And Multi-Target Execution | `aw td create 3783` |"));
        assert!(index.contains("| meter | run_verify | Runtime Resource Attribution | `aw capability report --project meter --verify` |"));
        assert!(index.contains("Execute one command at a time"));
    }

    #[test]
    fn capability_sweep_project_exposes_stable_next_action_label() {
        let report = sample_report(sample_action(
            CapabilityActionKind::AssignCapabilityType,
            "aw capability set-type --project jet --capability package-manager --type DeveloperTool",
            true,
        ));

        let project = capability_sweep_project(&report);

        assert_eq!(project.project, "jet");
        assert_eq!(project.next_action_kind, "assign_capability_type");
        assert_eq!(project.next_action_group, "assign_capability_type");
        assert_eq!(project.next_action_detail, None);
        assert!(project.requires_hitl);
    }

    #[test]
    fn capability_init_renders_empty_canonical_readme_shell() {
        let body = render_empty_capability_readme(
            "Cclab Core",
            "Capability map placeholder for `cclab-core`.",
        );
        let doc = cap_doc(&body);

        assert!(body.starts_with("# Cclab Core\n\n## Brief\n\n"));
        assert!(body.contains("\n## Capabilities\n\n### Capability Index\n\n"));
        assert!(body.contains(
            "| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |"
        ));
        assert_eq!(doc.format, CapabilityDocumentFormat::Empty);
        assert!(doc.capabilities.is_empty());
    }

    #[test]
    fn capability_init_humanizes_project_slug_title() {
        assert_eq!(humanize_project_title("cclab-core"), "Cclab Core");
        assert_eq!(humanize_project_title("agentkit"), "Agentkit");
        assert_eq!(humanize_project_title("cclab_grid_wasm"), "Cclab Grid Wasm");
    }

    fn one_capability() -> &'static str {
        r##"# demo

## Capability: Package Manager
<!-- type: capability lang: yaml -->

```yaml
id: package-manager
status: auditing
promise: "Replace package manager flows."
current_state: "Install surface exists."
gaps:
  - id: package-manager-readiness
    status: open
    active_wi: "#3779"
    summary: "Readiness audit pending."
verification_contract:
  required_maturity: [smoke, conformance]
  claims:
    - id: lockfile-determinism
      user_story: "As a frontend dev, I want installs to be reproducible from the lockfile."
      maturity: conformance
      oracle: "npm/pnpm lockfile behavior"
      fixtures:
        - "projects/jet/fixtures/pkg-manager/lockfile"
      negative_cases:
        - "Integrity mismatch must fail with an actionable diagnostic."
      gates:
        - id: lockfile
          command: "cargo test -p jet pkg_manager::lockfile"
          proves: "lockfile behavior"
evidence:
  source:
    - "projects/jet/src/pkg_manager/**"
  verification:
    - id: lockfile
      command: "cargo test -p jet pkg_manager::lockfile"
      proves: "lockfile behavior"
done_when:
  - "audit closes"
out_of_scope:
  - "registry service"
```
"##
    }

    fn one_markdown_capability() -> &'static str {
        r#"# demo

## Package Manager

| Field | Value |
|---|---|
| ID | package-manager |
| Root WI | #3779 |
| Status | auditing |
| Promise | Replace package manager flows. |
| Required Verification | smoke, conformance |
| Gate Inventory | projects/jet/validation/pkg-manager.toml |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |
	"#
    }

    fn one_field_markdown_capability() -> &'static str {
        r#"# demo

## Package Manager

ID: package-manager
Root WI: #3779
Status: auditing
Required Verification: smoke, conformance
Promise:
Replace package manager flows.
Gate Inventory:
- projects/jet/validation/pkg-manager.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |
	"#
    }

    #[test]
    fn set_type_updates_field_style_readme_contract() {
        let updated = upsert_capability_type_in_readme(
            one_field_markdown_capability(),
            "package-manager",
            CapabilityType::DeveloperTool,
        )
        .unwrap();

        assert!(updated.contains("ID: package-manager\nType: DeveloperTool\nRoot WI: #3779"));
        let parsed = parse_capability_document(&updated, Path::new("README.md")).unwrap();
        assert_eq!(
            parsed.capabilities[0].capability_type,
            Some(CapabilityType::DeveloperTool)
        );
    }

    #[test]
    fn set_type_updates_field_value_readme_contract() {
        let updated = upsert_capability_type_in_readme(
            one_markdown_capability(),
            "package-manager",
            CapabilityType::RuntimeTool,
        )
        .unwrap();

        assert!(updated
            .contains("| ID | package-manager |\n| Type | RuntimeTool |\n| Root WI | #3779 |"));
        let parsed = parse_capability_document(&updated, Path::new("README.md")).unwrap();
        assert_eq!(
            parsed.capabilities[0].capability_type,
            Some(CapabilityType::RuntimeTool)
        );
    }

    #[test]
    fn set_status_updates_field_style_readme_contract() {
        let updated = upsert_capability_status_in_readme(
            one_field_markdown_capability(),
            "package-manager",
            CapabilityStatus::Retired,
        )
        .unwrap();

        assert!(updated.contains("Status: retired"));
        let parsed = parse_capability_document(&updated, Path::new("README.md")).unwrap();
        assert_eq!(parsed.capabilities[0].status, CapabilityStatus::Retired);
    }

    #[test]
    fn set_status_retired_updates_capability_index_production() {
        let body = r#"# demo

## Brief

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Package Manager | #3779 | implemented | verified | smoke | ready | install flow |

### Package Manager

ID: package-manager
Root WI: #3779
Status: verified
Required Verification: smoke
Promise:
Replace package manager flows.
Gate Inventory:
- `cargo test -p jet`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #3779 | implemented | verified | smoke | `cargo test -p jet` |
"#;
        let updated =
            upsert_capability_status_in_readme(body, "package-manager", CapabilityStatus::Retired)
                .unwrap();

        assert!(updated.contains(
            "| Package Manager | #3779 | implemented | verified | smoke | retired | install flow |"
        ));
        assert!(updated.contains("Status: retired"));
    }

    #[test]
    fn set_status_blocked_updates_capability_index_verification_and_production() {
        let body = r#"# demo

## Brief

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Dynamic Security Evidence | - | implemented | verified | smoke | ready | vat/rig/meter evidence adapters |

### Dynamic Security Evidence

ID: dynamic-security-evidence
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Compose dynamic security evidence.
Gate Inventory:
- `cargo test -p guard`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Dynamic security evidence | epic | - | implemented | verified | smoke | `cargo test -p guard` |
"#;
        let updated = upsert_capability_status_in_readme(
            body,
            "dynamic-security-evidence",
            CapabilityStatus::Blocked,
        )
        .unwrap();

        assert!(updated.contains(
            "| Dynamic Security Evidence | - | implemented | blocked | smoke | blocked | vat/rig/meter evidence adapters |"
        ));
        assert!(updated.contains("Status: blocked"));
    }

    #[test]
    fn set_surface_updates_field_style_readme_contract() {
        let with_type = upsert_capability_type_in_readme(
            one_field_markdown_capability(),
            "package-manager",
            CapabilityType::DeveloperTool,
        )
        .unwrap();
        let updated = upsert_capability_surface_in_readme(
            &with_type,
            "package-manager",
            CapabilitySurface {
                kind: "CLI".to_string(),
                commands: vec!["jet install".to_string(), "jet add react".to_string()],
                summary: "package-management command surface, including lockfile flows".to_string(),
                verification: String::new(),
            },
        )
        .unwrap();

        assert!(updated.contains(
            "Type: DeveloperTool\nSurfaces: CLI: `jet install` + `jet add react` - package-management command surface, including lockfile flows\nRoot WI: #3779"
        ));
        let parsed = parse_capability_document(&updated, Path::new("README.md")).unwrap();
        assert_eq!(parsed.capabilities[0].surfaces.len(), 1);
        assert_eq!(parsed.capabilities[0].surfaces[0].kind, "CLI");
        assert_eq!(
            parsed.capabilities[0].surfaces[0].commands,
            vec!["jet install", "jet add react"]
        );
        assert_eq!(
            parsed.capabilities[0].surfaces[0].summary,
            "package-management command surface, including lockfile flows"
        );
    }

    #[test]
    fn set_surface_replaces_existing_kind_without_reordering() {
        let with_type = upsert_capability_type_in_readme(
            one_field_markdown_capability(),
            "package-manager",
            CapabilityType::DeveloperTool,
        )
        .unwrap();
        let with_surfaces = with_type.replace(
            "Type: DeveloperTool\nRoot WI: #3779",
            "Type: DeveloperTool\nSurfaces: CLI: `jet install` - package install surface.; UI: `http://localhost:<port>` - browser client surface.\nRoot WI: #3779",
        );
        let updated = upsert_capability_surface_in_readme(
            &with_surfaces,
            "package-manager",
            CapabilitySurface {
                kind: "CLI".to_string(),
                commands: vec!["jet install".to_string(), "jet add react".to_string()],
                summary: "package-management command surface".to_string(),
                verification: String::new(),
            },
        )
        .unwrap();

        assert!(updated.contains(
            "Surfaces: CLI: `jet install` + `jet add react` - package-management command surface; UI: `http://localhost:<port>` - browser client surface."
        ));
        let parsed = parse_capability_document(&updated, Path::new("README.md")).unwrap();
        assert_eq!(parsed.capabilities[0].surfaces[0].kind, "CLI");
        assert_eq!(parsed.capabilities[0].surfaces[1].kind, "UI");
    }

    #[test]
    fn set_ec_dimension_updates_field_style_readme_contract_with_efficiency_slot() {
        let updated = upsert_capability_ec_dimension_in_readme(
            one_field_markdown_capability(),
            "package-manager",
            CapabilityEcDimension {
                dimension: CapabilityEcDimensionKind::Efficiency,
                runner: "rig".to_string(),
                summary: "load pin, latency regression, and RSS gate".to_string(),
                required_for_production: None,
                efficiency_backfill: Some(CapabilityEfficiencyBackfillSlot {
                    operating_point: "local-vat-search-p95".to_string(),
                    cube: "projects/demo/.aw/ec/efficiency/search.cube.json".to_string(),
                }),
            },
        )
        .unwrap();

        assert!(updated.contains(
            "EC Dimensions: efficiency: `rig` - load pin, latency regression, and RSS gate"
        ));
        assert!(updated.contains("Efficiency Operating Point: local-vat-search-p95"));
        assert!(
            updated.contains("Efficiency Cube: projects/demo/.aw/ec/efficiency/search.cube.json")
        );
        assert!(updated
            .contains("#### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)"));
        assert!(updated.contains("Operating point: local-vat-search-p95"));
        assert!(updated.contains("Cube: projects/demo/.aw/ec/efficiency/search.cube.json"));
        let parsed = parse_capability_document(&updated, Path::new("README.md")).unwrap();
        let efficiency = parsed.capabilities[0]
            .ec_dimensions
            .iter()
            .find(|dimension| dimension.dimension == CapabilityEcDimensionKind::Efficiency)
            .unwrap();
        assert_eq!(efficiency.runner, "rig");
        assert_eq!(
            efficiency.summary,
            "load pin, latency regression, and RSS gate"
        );
        assert_eq!(
            efficiency.efficiency_backfill.as_ref().unwrap().cube,
            "projects/demo/.aw/ec/efficiency/search.cube.json"
        );
    }

    #[test]
    fn markdown_capability_index_marks_release_scope_and_dependencies() {
        let body = r#"# demo

## Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Package Manager | #1 | implemented | verified | smoke | ready | current release |
| Shared Core | - | implemented | verified | smoke | not_ready | dependency only |

## Package Manager

| Field | Value |
|---|---|
| ID | package-manager |
| Root WI | #1 |
| Status | verified |
| Promise | Replace package manager flows. |
| Required Verification | smoke |
| Gate Inventory | projects/jet/validation/pkg-manager.toml |
| Dependencies | Shared Core |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #1 | implemented | verified | smoke | projects/jet/validation/pkg-manager.toml |

## Shared Core

| Field | Value |
|---|---|
| ID | shared-core |
| Root WI | - |
| Status | verified |
| Promise | Provide shared runtime. |
| Required Verification | smoke |
| Gate Inventory | projects/jet/validation/shared.toml |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Shared core readiness | epic | - | implemented | verified | smoke | projects/jet/validation/shared.toml |
"#;
        let doc = cap_doc(body);
        let package = doc
            .capabilities
            .iter()
            .find(|capability| capability.id == "package-manager")
            .unwrap();
        let shared = doc
            .capabilities
            .iter()
            .find(|capability| capability.id == "shared-core")
            .unwrap();

        assert!(package.release_scope);
        assert_eq!(package.dependencies, vec!["shared-core"]);
        assert!(!shared.release_scope);
    }

    fn without_contract(body: &str) -> String {
        body.replace(
            r#"verification_contract:
  required_maturity: [smoke, conformance]
  claims:
    - id: lockfile-determinism
      user_story: "As a frontend dev, I want installs to be reproducible from the lockfile."
      maturity: conformance
      oracle: "npm/pnpm lockfile behavior"
      fixtures:
        - "projects/jet/fixtures/pkg-manager/lockfile"
      negative_cases:
        - "Integrity mismatch must fail with an actionable diagnostic."
      gates:
        - id: lockfile
          command: "cargo test -p jet pkg_manager::lockfile"
          proves: "lockfile behavior"
"#,
            "",
        )
    }

    #[test]
    fn parse_one_valid_h2_capability_section() {
        let doc = cap_doc(one_capability());
        assert_eq!(doc.capabilities.len(), 1);
        assert_eq!(doc.format, CapabilityDocumentFormat::YamlSections);
        assert_eq!(doc.capabilities[0].id, "package-manager");
        assert_eq!(doc.capabilities[0].gaps[0].id, "package-manager-readiness");
    }

    #[test]
    fn parse_markdown_capability_tables() {
        let doc = cap_doc(one_markdown_capability());
        assert_eq!(doc.format, CapabilityDocumentFormat::MarkdownTables);
        assert_eq!(doc.capabilities.len(), 1);
        let capability = &doc.capabilities[0];
        assert_eq!(capability.id, "package-manager");
        assert_eq!(capability.status, CapabilityStatus::Auditing);
        assert_eq!(capability.gaps[0].active_wi.as_deref(), Some("#3779"));
        let contract = capability.verification_contract.as_ref().unwrap();
        assert_eq!(contract.required_maturity.len(), 2);
        assert_eq!(
            contract.claims[0].fixtures[0],
            "projects/jet/validation/pkg-manager.toml"
        );
    }

    #[test]
    fn parse_markdown_field_capability_contract() {
        let doc = cap_doc(one_field_markdown_capability());
        assert_eq!(doc.format, CapabilityDocumentFormat::MarkdownTables);
        assert_eq!(doc.capabilities.len(), 1);
        let capability = &doc.capabilities[0];
        assert_eq!(capability.id, "package-manager");
        assert_eq!(capability.status, CapabilityStatus::Auditing);
        assert_eq!(capability.promise, "Replace package manager flows.");
        assert_eq!(
            capability.current_state,
            "Root WI: #3779; Gate inventory: projects/jet/validation/pkg-manager.toml"
        );
    }

    #[test]
    fn parse_capability_contract_facets_for_78() {
        let body = r#"# demo

## Search

ID: search
Root WI: #4141
Status: auditing
Type: Service
Surfaces:
- HTTP: `GET /search` serves ranked external ids.
- CLI: `lumen serve` starts the service.
EC Dimensions:
- behavior: `rig run search-flow` validates API behavior.
- efficiency: `rig load search-perf` backfills latency and qps.
- security: `guard scan lumen-auth` validates auth boundaries.
Required Verification: smoke, conformance
Promise:
Serve search queries as ranked external ids only.
Gate Inventory:
- `cargo test -p lumen planner`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Query planner | epic | #4141 | implemented | passing | conformance | `cargo test -p lumen planner` |

### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)

Operating point: 1M docs, qps=100, metric=p99_ms, ratchet=0.8
Cube: projects/lumen/tests/perf-cube.json

| feature | lumen | pg |
|---|---|---|
| text_bm25 | 1.0 | 22.5x win |
"#;
        let doc = cap_doc(body);
        let capability = &doc.capabilities[0];

        assert_eq!(capability.capability_type, Some(CapabilityType::Service));
        assert_eq!(capability.surfaces.len(), 2);
        assert_eq!(capability.surfaces[0].kind, "HTTP");
        assert_eq!(capability.surfaces[0].commands, vec!["GET /search"]);
        assert_eq!(capability.surfaces[1].kind, "CLI");
        assert_eq!(capability.surfaces[1].commands, vec!["lumen serve"]);
        assert_eq!(capability.ec_dimensions.len(), 3);
        let efficiency = capability
            .ec_dimensions
            .iter()
            .find(|dimension| dimension.dimension == CapabilityEcDimensionKind::Efficiency)
            .unwrap();
        assert_eq!(efficiency.runner, "rig load search-perf");
        let slot = efficiency.efficiency_backfill.as_ref().unwrap();
        assert_eq!(
            slot.operating_point,
            "1M docs, qps=100, metric=p99_ms, ratchet=0.8"
        );
        assert_eq!(slot.cube, "projects/lumen/tests/perf-cube.json");
    }

    #[test]
    fn report_ec_required_for_production_uses_type_ceiling_for_declared_dimensions() {
        let body = r#"# demo

## Tooling

ID: tooling
Root WI: #78
Status: auditing
Type: DeveloperTool
Required Verification: smoke
Promise:
Expose a developer-facing toolchain.
Gate Inventory:
- `cargo test -p demo tooling`

| Dimension | Runner | Required |
|---|---|---|
| behavior | `rig run tooling-flow` | no |
| efficiency | `meter collect tooling` | no |
| security | `guard scan tooling` | yes |
"#;
        let doc = cap_doc(body);
        let capability = &doc.capabilities[0];

        assert!(capability
            .ec_dimensions
            .iter()
            .all(|dimension| dimension.required_for_production.is_none()));

        let derived = derive_report_ec_dimensions(capability, &BTreeMap::new());
        let required = derived
            .iter()
            .map(|dimension| {
                (
                    dimension.dimension,
                    dimension.required_for_production.unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        assert_eq!(
            required.get(&CapabilityEcDimensionKind::Behavior),
            Some(&true)
        );
        assert_eq!(
            required.get(&CapabilityEcDimensionKind::Efficiency),
            Some(&true)
        );
        assert_eq!(
            required.get(&CapabilityEcDimensionKind::Security),
            Some(&false)
        );
    }

    #[test]
    fn report_ec_dimensions_do_not_materialize_empty_type_dimensions() {
        let body = r#"# demo

## Search

ID: search
Root WI: #78
Status: auditing
Type: Service
Required Verification: smoke
Promise:
Expose a service capability.
Gate Inventory:
- `cargo test -p demo search`
"#;
        let doc = cap_doc(body);
        let capability = &doc.capabilities[0];

        assert!(capability.ec_dimensions.is_empty());

        let derived = derive_report_ec_dimensions(capability, &BTreeMap::new());

        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].dimension, CapabilityEcDimensionKind::Behavior);
        assert_eq!(derived[0].required_for_production, Some(true));
    }

    #[test]
    fn report_ec_dimensions_require_efficiency_only_when_slot_declares_content() {
        let body = r#"# demo

## Search

ID: search
Root WI: #78
Status: auditing
Type: Service
Efficiency Operating Point: 1M docs, qps=100, metric=p99_ms, ratchet=0.8
Efficiency Cube: projects/lumen/.aw/ec/efficiency/search.cube.json
Required Verification: smoke
Promise:
Expose a service search capability.
Gate Inventory:
- `cargo test -p demo search`
"#;
        let doc = cap_doc(body);
        let capability = &doc.capabilities[0];

        let derived = derive_report_ec_dimensions(capability, &BTreeMap::new());
        let required = derived
            .iter()
            .map(|dimension| {
                (
                    dimension.dimension,
                    dimension.required_for_production.unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        assert_eq!(derived.len(), 2);
        assert_eq!(
            required.get(&CapabilityEcDimensionKind::Behavior),
            Some(&true)
        );
        assert_eq!(
            required.get(&CapabilityEcDimensionKind::Efficiency),
            Some(&true)
        );
        assert!(!required.contains_key(&CapabilityEcDimensionKind::Security));
        assert!(!required.contains_key(&CapabilityEcDimensionKind::Stability));
    }

    #[test]
    fn markdown_required_verification_parses_full_regenerability_contract() {
        let body = one_markdown_capability().replace(
            "Required Verification | smoke, conformance",
            "Required Verification | smoke, full regenerability",
        );
        let doc = cap_doc(&body);
        let contract = doc.capabilities[0].verification_contract.as_ref().unwrap();

        assert!(contract.full_regenerability_required);
    }

    #[test]
    fn yaml_contract_parses_full_regenerability_required() {
        let body = one_capability().replace(
            "verification_contract:\n  required_maturity: [smoke, conformance]",
            "verification_contract:\n  required_maturity: [smoke, conformance]\n  full_regenerability_required: true",
        );
        let doc = cap_doc(&body);
        let contract = doc.capabilities[0].verification_contract.as_ref().unwrap();

        assert!(contract.full_regenerability_required);
    }

    #[test]
    fn parse_nested_markdown_capability_tables_as_separate_roots() {
        let body = r#"# jet

## Rust-Native Frontend Toolchain Replacement

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| rust-native-frontend-toolchain | #3778 | verified | Replace the frontend toolchain. | smoke, dogfood | `cargo test -p jet` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Full toolchain dogfood flow | epic | #3778 | implemented | verified | dogfood | `cargo test -p jet` |

### Package Manager

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| package-manager | #3779 | verified | Replace package manager flows. | smoke, conformance | `cargo test -p jet pkg_manager` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lockfile parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet pkg_manager::lockfile` |
"#;
        let doc = cap_doc(body);
        assert_eq!(doc.format, CapabilityDocumentFormat::MarkdownTables);
        assert_eq!(doc.capabilities.len(), 2);
        assert_eq!(doc.capabilities[0].id, "rust-native-frontend-toolchain");
        assert_eq!(
            doc.capabilities[0].gaps[0].id,
            "full-toolchain-dogfood-flow"
        );
        assert_eq!(doc.capabilities[1].id, "package-manager");
        assert_eq!(doc.capabilities[1].gaps[0].id, "lockfile-parity");
    }

    #[test]
    fn markdown_work_root_table_rejects_invalid_enums() {
        let body = one_markdown_capability().replace("| partial |", "| doing |");
        let err = parse_capability_document(&body, Path::new("README.md")).unwrap_err();
        assert!(err.to_string().contains("invalid Impl"));

        let body = one_markdown_capability().replace("| epic |", "| task |");
        let err = parse_capability_document(&body, Path::new("README.md")).unwrap_err();
        assert!(err.to_string().contains("invalid Kind"));
    }

    #[test]
    fn yaml_migration_renders_markdown_tables_without_yaml_sections() {
        let doc = cap_doc(one_capability());
        let migrated = render_capability_markdown_migration(one_capability(), &doc, "jet");
        assert!(migrated.contains("## Brief"));
        assert!(migrated.contains("## Capabilities"));
        assert!(migrated.contains("\n### Capability Index\n"));
        assert!(migrated.contains("\n### Package Manager\n"));
        assert!(migrated.contains("\nID: package-manager\n"));
        assert!(migrated.contains("\nPromise:\nReplace package manager flows."));
        assert!(!migrated.contains("| Field | Value |"));
        assert!(migrated.contains(
            "| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |"
        ));
        assert!(!migrated.contains("## Capability: Package Manager"));
        assert!(!migrated.contains("```yaml"));

        let reparsed = cap_doc(&migrated);
        assert_eq!(reparsed.format, CapabilityDocumentFormat::MarkdownTables);
        assert_eq!(reparsed.capabilities[0].id, "package-manager");
    }

    #[test]
    fn markdown_migration_preserves_project_brief_outside_capability_registry() {
        let body = format!(
            r#"# Jet

## Brief

Jet is a Rust-native frontend toolchain.

| Surface | Commands | Owns |
|---|---|---|
| Build | `jet build` | Production artifacts. |

## Capabilities

{}
"#,
            one_markdown_capability()
        );
        let doc = cap_doc(&body);
        let migrated = render_capability_markdown_migration(&body, &doc, "jet");

        assert!(migrated.contains("Jet is a Rust-native frontend toolchain."));
        assert!(migrated.contains("| Build | `jet build` | Production artifacts. |"));
        assert!(!migrated.contains("TODO: Add the human-confirmed project brief"));
        assert!(migrated.contains("\n### Capability Index\n"));
        assert!(migrated.contains("\n### Package Manager\n"));
    }

    #[test]
    fn markdown_migration_inserts_registry_at_original_capability_position() {
        let body = r#"# lumen

A K8s-native search specialist.

## Capabilities

Capability intro stays before the registry.

## Capability Index

The capability roots group into pillars.

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| search | - | implemented | auditing | conformance | not_ready | broad search evidence |

**Honest scope (do not over-claim):**

- Ingestion is the caller's own pub/sub.

### Search

ID: search
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Search returns external ids only.
Gate Inventory:
- projects/lumen/tests/planner_diff.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Query planner | epic | - | implemented | passing | conformance | projects/lumen/tests/planner_diff.rs |

## Benchmarks

Benchmark reference stays after the registry.
"#;
        let doc = cap_doc(body);
        let migrated = render_capability_markdown_migration(body, &doc, "lumen");

        assert!(migrated.contains("\n## Brief\n"));
        assert!(migrated.contains("## Brief\n\nA K8s-native search specialist."));
        assert!(!migrated.contains("## Brief\n\n\nA K8s-native search specialist."));
        assert!(migrated.contains("A K8s-native search specialist.\n\n## Capabilities"));
        assert!(migrated.contains("A K8s-native search specialist."));
        assert!(!migrated.contains("TODO: Add the human-confirmed project brief"));
        assert!(migrated.contains("The capability roots group into pillars."));
        assert!(migrated.contains("Ingestion is the caller's own pub/sub."));
        assert!(migrated.contains("Benchmark reference stays after the registry."));
        assert!(migrated.contains("\n### Capability Index\n"));
        assert!(migrated.contains("\n### Search\n"));
        assert!(
            migrated.find("\n### Search\n").unwrap() < migrated.find("\n## Benchmarks\n").unwrap()
        );
        assert!(!migrated.contains(CAPABILITY_MIGRATION_INSERT_MARKER));
    }

    #[test]
    fn markdown_migration_preserves_multiple_capability_index_tables() {
        let body = r#"# lumen

## Brief

Search specialist.

## Capabilities

## Capability Index

**Pillar — agent-first**

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| agentic-integration | 4143 | implemented | passing | conformance | ready | offline CLI contract |

**Pillar — serve / search**

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| search-lexical | - | partial | auditing | conformance | not_ready | WAND/block-max remains open |

### Lexical

ID: search-lexical
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
BM25 ranking over text.
Gate Inventory:
- projects/lumen/scripts/bench_vs_db.py

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| BM25 ranking | subepic | - | implemented | passing | conformance | projects/lumen/scripts/bench_vs_db.py |

### Agentic Integration

ID: agentic-integration
Root WI: 4143
Status: verified
Required Verification: conformance
Promise:
Offline CLI schema and topics.
Gate Inventory:
- projects/lumen/tests/spec_cli.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| `lumen spec` | epic | 4143 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
"#;
        let doc = cap_doc(body);
        let migrated = render_capability_markdown_migration(body, &doc, "lumen");

        assert!(migrated.contains("| Lexical | - | partial | auditing | conformance | not_ready | WAND/block-max remains open |"));
        assert!(migrated.contains("| Agentic Integration | 4143 | implemented | passing | conformance | ready | offline CLI contract |"));
        assert!(!migrated.contains("**Pillar — agent-first**"));
        assert!(!migrated.contains("**Pillar — serve / search**"));
    }

    #[test]
    fn markdown_migration_keeps_fenced_code_inside_capability_postlude() {
        let body = r#"# lumen

K8s-native search specialist.

## Capabilities

### Kubernetes-Native Deployment

ID: k8s-deployment
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Deploy declaratively.
Gate Inventory:
- projects/lumen/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| kustomize base | epic | - | implemented | passing | conformance | projects/lumen/k8s |

Deployment handoff:

```bash
# 1. build
kubectl apply -k k8s/overlays/myenv
```

### HTTP / REST Integration

ID: rest-integration
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
REST clients work.
Gate Inventory:
- projects/lumen/src

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| REST API | epic | - | implemented | passing | conformance | projects/lumen/src |
"#;
        let doc = cap_doc(body);
        let migrated = render_capability_markdown_migration(body, &doc, "lumen");
        let k8s = migrated
            .find("\n### Kubernetes-Native Deployment\n")
            .unwrap();
        let code = migrated.find("# 1. build").unwrap();
        let http = migrated.find("\n### HTTP / REST Integration\n").unwrap();

        assert!(k8s < code && code < http);
        assert!(migrated.contains(
            "```bash\n# 1. build\nkubectl apply -k k8s/overlays/myenv\n```\n\n### HTTP / REST Integration"
        ));
    }

    #[test]
    fn markdown_migration_canonicalizes_plain_efficiency_backfill_section() {
        let body = r#"# demo

## Brief

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Search | - | implemented | auditing | conformance | not_ready | search service |

### Search

ID: search
Type: Service
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Serve ranked external ids.
Gate Inventory:
- `cargo test -p lumen planner`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Query planner | epic | - | implemented | passing | conformance | `cargo test -p lumen planner` |

#### Efficiency

Operating point: 1M docs, qps=100, metric=p99_ms
Cube: projects/lumen/.aw/ec/efficiency/search.cube.json
"#;
        let doc = cap_doc(body);
        assert!(doc.requires_format_migration());

        let migrated = render_capability_markdown_migration(body, &doc, "lumen");

        assert!(!migrated.contains("\n#### Efficiency\n"));
        assert!(migrated.contains(
            "\n#### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)\n\nOperating point: 1M docs, qps=100, metric=p99_ms\nCube: projects/lumen/.aw/ec/efficiency/search.cube.json\n"
        ));
    }

    #[test]
    fn parse_nested_verification_contract() {
        let doc = cap_doc(one_capability());
        let contract = doc.capabilities[0].verification_contract.as_ref().unwrap();
        assert_eq!(contract.required_maturity.len(), 2);
        assert_eq!(contract.claims[0].id, "lockfile-determinism");
        assert_eq!(contract.claims[0].gates[0].id, "lockfile");
    }

    #[test]
    fn allow_candidate_without_verification_contract() {
        let body =
            without_contract(one_capability()).replace("status: auditing", "status: candidate");
        let doc = cap_doc(&body);
        assert!(!doc
            .findings
            .iter()
            .any(|finding| finding.contains("requires verification_contract")));
    }

    #[test]
    fn reject_auditing_without_verification_contract() {
        let body = without_contract(one_capability());
        let doc = cap_doc(&body);
        assert!(doc
            .findings
            .iter()
            .any(|finding| finding.contains("requires verification_contract")));
    }

    #[test]
    fn parse_multiple_capabilities_and_count() {
        let body = format!(
            "{}\n{}",
            one_capability(),
            one_capability().replace("package-manager", "bundler")
        );
        let doc = cap_doc(&body);
        assert_eq!(doc.capabilities.len(), 2);
    }

    #[test]
    fn reject_duplicate_capability_ids() {
        let body = format!("{}\n{}", one_capability(), one_capability());
        let err = parse_capability_document(&body, Path::new("README.md")).unwrap_err();
        assert!(err.to_string().contains("duplicate capability id"));
    }

    #[test]
    fn reject_invalid_capability_status() {
        let body = one_capability().replace("status: auditing", "status: maybe");
        let err = parse_capability_document(&body, Path::new("README.md")).unwrap_err();
        assert!(err.to_string().contains("invalid capability YAML"));
    }

    #[test]
    fn reject_invalid_claim_maturity() {
        let body = one_capability().replace("maturity: conformance", "maturity: maybe");
        let err = parse_capability_document(&body, Path::new("README.md")).unwrap_err();
        assert!(err.to_string().contains("invalid capability YAML"));
    }

    #[test]
    fn reject_duplicate_claim_ids() {
        let body = one_capability().replace(
            "evidence:\n",
            r#"    - id: lockfile-determinism
      user_story: "As a frontend dev, I want duplicate claims rejected."
      maturity: smoke
      oracle: "npm"
      gates:
        - id: duplicate-lockfile
          command: "cargo test -p jet pkg_manager::lockfile"
          proves: "duplicate claim rejection fixture"
evidence:
"#,
        );
        let err = parse_capability_document(&body, Path::new("README.md")).unwrap_err();
        assert!(err.to_string().contains("duplicate claim id"));
    }

    #[test]
    fn reject_required_claim_without_oracle() {
        let body = one_capability().replace("      oracle: \"npm/pnpm lockfile behavior\"\n", "");
        let doc = cap_doc(&body);
        assert!(doc
            .findings
            .iter()
            .any(|finding| finding.contains("requires oracle")));
    }

    #[test]
    fn reject_required_claim_without_gates() {
        let body = one_capability()
            .replace(
                r#"      fixtures:
        - "projects/jet/fixtures/pkg-manager/lockfile"
"#,
                "",
            )
            .replace(
                r#"      gates:
        - id: lockfile
          command: "cargo test -p jet pkg_manager::lockfile"
          proves: "lockfile behavior"
"#,
                "      gates: []\n",
            );
        let doc = cap_doc(&body);
        assert!(doc
            .findings
            .iter()
            .any(|finding| finding.contains("requires at least one gate")));
    }

    #[test]
    fn fixture_reference_can_verify_required_claim() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("evidence.md"), "verified evidence").unwrap();
        let body = r#"# Demo

## Capability One

| Field | Value |
|---|---|
| ID | capability-one |
| Root WI | - |
| Status | verified |
| Promise | Demonstrate fixture-backed verification. |
| Required Verification | smoke |
| Gate Inventory | evidence.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Fixture backed claim | epic | - | implemented | verified | smoke | evidence.md |
"#;
        let doc = cap_doc(body);
        let claims = capability_claim_reports(&doc.capabilities[0], tmp.path(), true);

        assert_eq!(claims.len(), 1);
        assert!(claims[0].verified);
    }

    #[test]
    fn missing_fixture_reference_does_not_verify_required_claim() {
        let tmp = tempfile::tempdir().unwrap();
        let body = r#"# Demo

## Capability One

| Field | Value |
|---|---|
| ID | capability-one |
| Root WI | - |
| Status | verified |
| Promise | Demonstrate fixture-backed verification. |
| Required Verification | smoke |
| Gate Inventory | missing.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Fixture backed claim | epic | - | implemented | verified | smoke | missing.md |
"#;
        let doc = cap_doc(body);
        let claims = capability_claim_reports(&doc.capabilities[0], tmp.path(), true);

        assert_eq!(claims.len(), 1);
        assert!(!claims[0].verified);
    }

    #[test]
    fn verification_env_skip_output_becomes_env_blocked() {
        let result =
            run_verification_command(Path::new("."), "printf 'skipping: chromium unavailable\\n'");
        assert_eq!(result.status, "env_blocked");
    }

    #[test]
    fn verification_command_timeout_reports_timeout() {
        let tmp = tempfile::tempdir().unwrap();
        let result = run_verification_command_with_timeout(
            tmp.path(),
            "sleep 2",
            Duration::from_millis(200),
        );

        assert_eq!(result.status, "timeout");
        assert_eq!(result.exit_code, None);
        assert!(result
            .stderr
            .as_deref()
            .unwrap_or("")
            .contains("aw capability gate timed out"));
    }

    #[test]
    fn duplicate_claim_gate_commands_run_once_per_report_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let body = r#"# Demo

## Capability: Demo
<!-- type: capability lang: yaml -->

```yaml
id: demo
status: auditing
promise: "Demonstrate cached claim gate verification."
current_state: "Test-only capability."
verification_contract:
  required_maturity: [smoke]
  claims:
    - id: first-claim
      user_story: "First claim"
      maturity: smoke
      oracle: "shell"
      required_for_verified: true
      gates:
        - id: first-gate
          command: "printf run >> runs.txt"
          proves: "first behavior"
    - id: second-claim
      user_story: "Second claim"
      maturity: smoke
      oracle: "shell"
      required_for_verified: true
      gates:
        - id: second-gate
          command: "printf run >> runs.txt"
          proves: "second behavior"
```
"#;
        let doc = cap_doc(body);

        let claims = capability_claim_reports(&doc.capabilities[0], tmp.path(), true);

        assert_eq!(claims.len(), 2);
        assert_eq!(claims[0].gates[0].id, "first-gate");
        assert_eq!(claims[0].gates[0].proves.as_deref(), Some("first behavior"));
        assert_eq!(claims[1].gates[0].id, "second-gate");
        assert_eq!(
            claims[1].gates[0].proves.as_deref(),
            Some("second behavior")
        );
        assert!(claims.iter().all(|claim| claim.verified));
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("runs.txt")).unwrap(),
            "run"
        );
    }

    #[test]
    fn duplicate_legacy_evidence_commands_run_once_per_report_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let body = r#"# Demo

## Capability: Demo
<!-- type: capability lang: yaml -->

```yaml
id: demo
status: candidate
promise: "Demonstrate cached legacy evidence verification."
current_state: "Test-only capability."
evidence:
  verification:
    - id: first-evidence
      command: "printf run >> runs.txt"
      proves: "first behavior"
    - id: second-evidence
      command: "printf run >> runs.txt"
      proves: "second behavior"
```
"#;
        let doc = cap_doc(body);

        let results = capability_verification_results(&doc.capabilities[0], tmp.path(), &[], true);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "first-evidence");
        assert_eq!(results[0].proves.as_deref(), Some("first behavior"));
        assert_eq!(results[1].id, "second-evidence");
        assert_eq!(results[1].proves.as_deref(), Some("second behavior"));
        assert!(results.iter().all(|result| result.status == "pass"));
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("runs.txt")).unwrap(),
            "run"
        );
    }

    #[test]
    fn detect_legacy_capability_table() {
        let body = r#"
## Capability Map

| Capability | Current State | Gaps | Active WI | Evidence |
|------------|---------------|------|-----------|----------|
| Package manager | exists | audit pending | #1 | source |
"#;
        let doc = cap_doc(body);
        assert!(doc.is_legacy_only());
        assert_eq!(doc.legacy_rows.len(), 1);
        assert_eq!(doc.findings.len(), 1);
    }

    #[test]
    fn validate_td_capability_refs_with_primary_ref() {
        let doc = cap_doc(one_capability());
        let td = r#"---
id: td-demo
fill_sections: [changes]
capability_refs:
  - id: package-manager
    role: primary
    gap: package-manager-readiness
    claim: lockfile-determinism
    coverage: partial
    rationale: "closes audit"
---

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes: []
```
"#;
        let (_, refs, findings) = validate_td_capability_refs_for_content(td, &doc).unwrap();
        assert_eq!(refs.len(), 1);
        assert!(findings.is_empty());
    }

    #[test]
    fn validate_td_capability_refs_rejects_unknown_capability() {
        let doc = cap_doc(one_capability());
        let td = r#"---
id: td-demo
fill_sections: [changes]
capability_refs:
  - id: nope
    role: primary
    coverage: partial
---
"#;
        let (_, _, findings) = validate_td_capability_refs_for_content(td, &doc).unwrap();
        assert!(findings.iter().any(|f| f.contains("unknown capability id")));
    }

    #[test]
    fn validate_td_capability_refs_rejects_unknown_gap() {
        let doc = cap_doc(one_capability());
        let td = r#"---
id: td-demo
fill_sections: [changes]
capability_refs:
  - id: package-manager
    role: primary
    gap: nope
    claim: lockfile-determinism
    coverage: partial
---
"#;
        let (_, _, findings) = validate_td_capability_refs_for_content(td, &doc).unwrap();
        assert!(findings.iter().any(|f| f.contains("unknown gap id")));
    }

    #[test]
    fn validate_td_capability_refs_requires_claim_for_primary_contract_ref() {
        let doc = cap_doc(one_capability());
        let td = r#"---
id: td-demo
fill_sections: [changes]
capability_refs:
  - id: package-manager
    role: primary
    gap: package-manager-readiness
    coverage: partial
---
"#;
        let (_, _, findings) = validate_td_capability_refs_for_content(td, &doc).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f
                    .contains("requires claim because the capability has verification_contract"))
        );
    }

    #[test]
    fn validate_td_capability_refs_rejects_unknown_claim() {
        let doc = cap_doc(one_capability());
        let td = r#"---
id: td-demo
fill_sections: [changes]
capability_refs:
  - id: package-manager
    role: primary
    gap: package-manager-readiness
    claim: nope
    coverage: partial
---
"#;
        let (_, _, findings) = validate_td_capability_refs_for_content(td, &doc).unwrap();
        assert!(findings.iter().any(|f| f.contains("unknown claim id")));
    }

    #[test]
    fn validate_td_capability_scope_internal_allows_no_refs() {
        let doc = cap_doc(one_capability());
        let td = r#"---
id: td-demo
fill_sections: [changes]
capability_scope: internal
---
"#;
        let (_, refs, findings) = validate_td_capability_refs_for_content(td, &doc).unwrap();
        assert!(refs.is_empty());
        assert!(findings.is_empty());
    }

    #[test]
    fn validate_td_capability_refs_accepts_multiple_refs() {
        let body = format!(
            "{}\n{}",
            one_capability(),
            one_capability().replace("package-manager", "bundler")
        );
        let doc = cap_doc(&body);
        let td = r#"---
id: td-demo
fill_sections: [changes]
capability_refs:
  - id: package-manager
    role: primary
    gap: package-manager-readiness
    claim: lockfile-determinism
    coverage: partial
  - id: bundler
    role: contributes
    gap: bundler-readiness
    coverage: partial
---
"#;
        let (_, refs, findings) = validate_td_capability_refs_for_content(td, &doc).unwrap();
        assert_eq!(refs.len(), 2);
        assert!(findings.is_empty());
    }

    #[test]
    fn next_action_defines_contract_when_non_candidate_lacks_one() {
        let body = r#"# demo

## Package Manager

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| package-manager | #3779 | auditing | Replace package manager flows. | - | - |
"#;
        let document = canonical_doc(&body);
        let report = CapabilityReport {
            action: "capability",
            project: "jet".to_string(),
            cap_path: PathBuf::from("projects/jet/README.md"),
            format_version: 1,
            status: "blocked".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("jet"),
            production_ready: false,
            production_status: ProductionStatus::NotEvaluated,
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            capability_count: 1,
            verified_count: 0,
            percent: 0.0,
            claim_count: 0,
            verified_claim_count: 0,
            claim_percent: 0.0,
            capabilities: vec![CapabilityReportItem {
                id: "package-manager".to_string(),
                title: "Package Manager".to_string(),
                status: CapabilityStatus::Auditing,
                capability_type: None,
                surfaces: Vec::new(),
                ec_dimensions: Vec::new(),
                promise: "Replace package manager flows.".to_string(),
                current_state: "Install surface exists.".to_string(),
                gaps: Vec::new(),
                td_refs: Vec::new(),
                wi_refs: Vec::new(),
                wi_evidence: Vec::new(),
                claims: Vec::new(),
                claim_count: 0,
                verified_claim_count: 0,
                claim_percent: 0.0,
                verification: Vec::new(),
                verified: false,
                release_scope: false,
                full_regenerability_required: false,
                dependencies: Vec::new(),
                dependency_closure: Vec::new(),
                production_ready: false,
                production_blockers: Vec::new(),
            }],
            blockers: document.findings.clone(),
            warnings: Vec::new(),
            next_action: CapabilityAction {
                kind: CapabilityActionKind::None,
                capability_id: None,
                gap_id: None,
                claim_id: None,
                target: "jet".to_string(),
                command: String::new(),
                reason: String::new(),
                requires_hitl: false,
                hitl_question: None,
            },
            run_results: Vec::new(),
        };

        let types = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &types);

        assert_eq!(
            action.kind,
            CapabilityActionKind::DefineVerificationContract
        );
        assert_eq!(action.capability_id.as_deref(), Some("package-manager"));
    }

    #[test]
    fn next_action_reruns_runtime_verification_for_catalog_verified_capability() {
        let body = one_markdown_capability()
            .replace("| Status | auditing |", "| Status | verified |")
            .replace(
                "| Package manager readiness | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |",
                "| Package manager readiness | epic | #3779 | implemented | verified | conformance | projects/jet/validation/pkg-manager.toml |",
            );
        let document = canonical_doc(&body);
        let report = CapabilityReport {
            action: "capability",
            project: "jet".to_string(),
            cap_path: PathBuf::from("projects/jet/README.md"),
            format_version: 1,
            status: "blocked".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("jet"),
            production_ready: false,
            production_status: ProductionStatus::NotEvaluated,
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            capability_count: 1,
            verified_count: 0,
            percent: 0.0,
            claim_count: 1,
            verified_claim_count: 0,
            claim_percent: 0.0,
            capabilities: vec![CapabilityReportItem {
                id: "package-manager".to_string(),
                title: "Package Manager".to_string(),
                status: CapabilityStatus::Verified,
                capability_type: None,
                surfaces: Vec::new(),
                ec_dimensions: Vec::new(),
                promise: "Replace package manager flows.".to_string(),
                current_state: "Install surface exists.".to_string(),
                gaps: Vec::new(),
                td_refs: vec![TdCapabilityEvidence {
                    spec_path: ".aw/tech-design/projects/jet/specs/3779.md".to_string(),
                    spec_id: Some("jet-package-manager-readiness-audit".to_string()),
                    review_status: None,
                    capability_id: "package-manager".to_string(),
                    role: CapabilityRefRole::Primary,
                    gap: Some("package-manager-readiness".to_string()),
                    claim: Some("lockfile-determinism".to_string()),
                    coverage: CapabilityCoverage::Partial,
                    rationale: None,
                }],
                wi_refs: Vec::new(),
                wi_evidence: Vec::new(),
                claims: vec![CapabilityClaimReport {
                    id: "lockfile-determinism".to_string(),
                    user_story: "reproducible lockfile".to_string(),
                    required_for_verified: true,
                    maturity: CapabilityMaturity::Conformance,
                    oracle: "cargo test".to_string(),
                    fixtures: Vec::new(),
                    negative_cases: Vec::new(),
                    gates: vec![VerificationRuntimeResult {
                        id: "lockfile".to_string(),
                        command: "cargo test -p jet pkg_manager::lockfile".to_string(),
                        status: "not_run".to_string(),
                        proves: Some("lockfile behavior".to_string()),
                        exit_code: None,
                        stdout: None,
                        stderr: None,
                    }],
                    verified: false,
                }],
                claim_count: 1,
                verified_claim_count: 0,
                claim_percent: 0.0,
                verification: Vec::new(),
                verified: false,
                release_scope: true,
                full_regenerability_required: false,
                dependencies: Vec::new(),
                dependency_closure: Vec::new(),
                production_ready: false,
                production_blockers: Vec::new(),
            }],
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_action: CapabilityAction {
                kind: CapabilityActionKind::None,
                capability_id: None,
                gap_id: None,
                claim_id: None,
                target: "jet".to_string(),
                command: String::new(),
                reason: String::new(),
                requires_hitl: false,
                hitl_question: None,
            },
            run_results: Vec::new(),
        };

        let types = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &types);

        assert_eq!(action.kind, CapabilityActionKind::RunVerify);
        assert_eq!(action.capability_id.as_deref(), Some("package-manager"));
        assert_eq!(
            action.command,
            "aw capability report --project jet --verify"
        );
        assert!(action.reason.contains("runtime verification"));
    }

    #[test]
    fn next_action_reruns_verification_for_fixture_inventory_claims() {
        let body = one_markdown_capability()
            .replace("| Status | auditing |", "| Status | verified |")
            .replace(
                "| Package manager readiness | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |",
                "| Package manager readiness | epic | #3779 | implemented | verified | conformance | projects/jet/validation/pkg-manager.toml |",
            );
        let document = canonical_doc(&body);
        let report = CapabilityReport {
            action: "capability",
            project: "jet".to_string(),
            cap_path: PathBuf::from("projects/jet/README.md"),
            format_version: 1,
            status: "blocked".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("jet"),
            production_ready: false,
            production_status: ProductionStatus::NotEvaluated,
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            capability_count: 1,
            verified_count: 0,
            percent: 0.0,
            claim_count: 1,
            verified_claim_count: 0,
            claim_percent: 0.0,
            capabilities: vec![CapabilityReportItem {
                id: "package-manager".to_string(),
                title: "Package Manager".to_string(),
                status: CapabilityStatus::Verified,
                capability_type: None,
                surfaces: Vec::new(),
                ec_dimensions: Vec::new(),
                promise: "Replace package manager flows.".to_string(),
                current_state: "Install surface exists.".to_string(),
                gaps: Vec::new(),
                td_refs: vec![TdCapabilityEvidence {
                    spec_path: ".aw/tech-design/projects/jet/specs/3779.md".to_string(),
                    spec_id: Some("jet-package-manager-readiness-audit".to_string()),
                    review_status: None,
                    capability_id: "package-manager".to_string(),
                    role: CapabilityRefRole::Primary,
                    gap: Some("package-manager-readiness".to_string()),
                    claim: Some("lockfile-determinism".to_string()),
                    coverage: CapabilityCoverage::Partial,
                    rationale: None,
                }],
                wi_refs: Vec::new(),
                wi_evidence: Vec::new(),
                claims: vec![CapabilityClaimReport {
                    id: "lockfile-determinism".to_string(),
                    user_story: "reproducible lockfile".to_string(),
                    required_for_verified: true,
                    maturity: CapabilityMaturity::Conformance,
                    oracle: "projects/jet/validation/pkg-manager.toml".to_string(),
                    fixtures: vec!["projects/jet/validation/pkg-manager.toml".to_string()],
                    negative_cases: Vec::new(),
                    gates: Vec::new(),
                    verified: false,
                }],
                claim_count: 1,
                verified_claim_count: 0,
                claim_percent: 0.0,
                verification: Vec::new(),
                verified: false,
                release_scope: true,
                full_regenerability_required: false,
                dependencies: Vec::new(),
                dependency_closure: Vec::new(),
                production_ready: false,
                production_blockers: Vec::new(),
            }],
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_action: CapabilityAction {
                kind: CapabilityActionKind::None,
                capability_id: None,
                gap_id: None,
                claim_id: None,
                target: "jet".to_string(),
                command: String::new(),
                reason: String::new(),
                requires_hitl: false,
                hitl_question: None,
            },
            run_results: Vec::new(),
        };

        let types = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &types);

        assert_eq!(action.kind, CapabilityActionKind::RunVerify);
        assert_eq!(action.capability_id.as_deref(), Some("package-manager"));
        assert_eq!(
            action.command,
            "aw capability report --project jet --verify"
        );
        assert!(action.reason.contains("fixture/inventory"));
    }

    #[test]
    fn next_action_reports_failing_runtime_verification_for_catalog_verified_capability() {
        let body = one_markdown_capability()
            .replace("| Status | auditing |", "| Status | verified |")
            .replace(
                "| Package manager readiness | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |",
                "| Package manager readiness | epic | #3779 | implemented | verified | conformance | projects/jet/validation/pkg-manager.toml |",
            );
        let document = canonical_doc(&body);
        let report = CapabilityReport {
            action: "capability",
            project: "jet".to_string(),
            cap_path: PathBuf::from("projects/jet/README.md"),
            format_version: 1,
            status: "blocked".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("jet"),
            production_ready: false,
            production_status: ProductionStatus::Blocked,
            production_scope: Vec::new(),
            production_blockers: vec![
                "capability `package-manager` catalog/claim verification is incomplete".to_string(),
            ],
            capability_count: 1,
            verified_count: 0,
            percent: 0.0,
            claim_count: 0,
            verified_claim_count: 0,
            claim_percent: 0.0,
            capabilities: vec![CapabilityReportItem {
                id: "package-manager".to_string(),
                title: "Package Manager".to_string(),
                status: CapabilityStatus::Verified,
                capability_type: None,
                surfaces: Vec::new(),
                ec_dimensions: Vec::new(),
                promise: "Replace package manager flows.".to_string(),
                current_state: "Install surface exists.".to_string(),
                gaps: Vec::new(),
                td_refs: Vec::new(),
                wi_refs: Vec::new(),
                wi_evidence: Vec::new(),
                claims: Vec::new(),
                claim_count: 0,
                verified_claim_count: 0,
                claim_percent: 0.0,
                verification: vec![VerificationRuntimeResult {
                    id: "lockfile".to_string(),
                    command: "cargo test -p jet pkg_manager::lockfile".to_string(),
                    status: "fail".to_string(),
                    proves: Some("lockfile behavior".to_string()),
                    exit_code: Some(101),
                    stdout: None,
                    stderr: Some("test failed".to_string()),
                }],
                verified: false,
                release_scope: true,
                full_regenerability_required: false,
                dependencies: Vec::new(),
                dependency_closure: Vec::new(),
                production_ready: false,
                production_blockers: Vec::new(),
            }],
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_action: CapabilityAction {
                kind: CapabilityActionKind::None,
                capability_id: None,
                gap_id: None,
                claim_id: None,
                target: "jet".to_string(),
                command: String::new(),
                reason: String::new(),
                requires_hitl: false,
                hitl_question: None,
            },
            run_results: Vec::new(),
        };

        let types = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &types);

        assert_eq!(action.kind, CapabilityActionKind::RunVerify);
        assert_eq!(action.capability_id.as_deref(), Some("package-manager"));
        assert_eq!(action.command, "cargo test -p jet pkg_manager::lockfile");
        assert!(action.reason.contains("failing verification gate"));
    }

    /// A real (non-candidate, non-retired) capability with NO type assigned in
    /// .aw/capability-types.toml triggers the assign-capability-type HITL; the
    /// same report with a type assigned proceeds past it.
    #[test]
    fn next_action_requires_capability_type_when_unset() {
        let body = one_markdown_capability()
            .replace("| Status | auditing |", "| Status | verified |")
            .replace(
                "| Package manager readiness | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |",
                "| Package manager readiness | epic | #3779 | implemented | verified | conformance | projects/jet/validation/pkg-manager.toml |",
            );
        let document = canonical_doc(&body);
        let report = CapabilityReport {
            action: "capability",
            project: "jet".to_string(),
            cap_path: PathBuf::from("projects/jet/README.md"),
            format_version: 1,
            status: "blocked".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("jet"),
            production_ready: false,
            production_status: ProductionStatus::Blocked,
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            capability_count: 1,
            verified_count: 1,
            percent: 100.0,
            claim_count: 0,
            verified_claim_count: 0,
            claim_percent: 0.0,
            capabilities: vec![CapabilityReportItem {
                id: "package-manager".to_string(),
                title: "Package Manager".to_string(),
                status: CapabilityStatus::Verified,
                capability_type: None,
                surfaces: Vec::new(),
                ec_dimensions: Vec::new(),
                promise: "Replace package manager flows.".to_string(),
                current_state: "Install surface exists.".to_string(),
                gaps: Vec::new(),
                td_refs: Vec::new(),
                wi_refs: Vec::new(),
                wi_evidence: Vec::new(),
                claims: Vec::new(),
                claim_count: 0,
                verified_claim_count: 0,
                claim_percent: 0.0,
                verification: Vec::new(),
                verified: true,
                release_scope: true,
                full_regenerability_required: false,
                dependencies: Vec::new(),
                dependency_closure: Vec::new(),
                production_ready: true,
                production_blockers: Vec::new(),
            }],
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_action: CapabilityAction {
                kind: CapabilityActionKind::None,
                capability_id: None,
                gap_id: None,
                claim_id: None,
                target: "jet".to_string(),
                command: String::new(),
                reason: String::new(),
                requires_hitl: false,
                hitl_question: None,
            },
            run_results: Vec::new(),
        };

        // No type assigned -> assign-capability-type HITL.
        let empty = BTreeMap::new();
        let action = choose_next_action(&report, &document, &empty);
        assert_eq!(action.kind, CapabilityActionKind::AssignCapabilityType);
        assert_eq!(action.capability_id.as_deref(), Some("package-manager"));
        assert!(action.requires_hitl);
        let question = action.hitl_question.expect("hitl question present");
        assert_eq!(question.id, "capability:package-manager:assign_type");
        assert_eq!(question.default_choice.as_deref(), Some("service"));
        let choice_ids: Vec<&str> = question.choices.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            choice_ids,
            vec![
                "agent_first",
                "service",
                "devops",
                "developer_tool",
                "runtime_tool",
                "security_tool"
            ]
        );

        // Type assigned -> the verified, gapless capability yields no action.
        let typed = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &typed);
        assert_ne!(action.kind, CapabilityActionKind::AssignCapabilityType);
        assert_eq!(action.kind, CapabilityActionKind::None);
    }

    #[test]
    fn next_action_ignores_retired_capability_status_rollup() {
        let body = r#"# meter

## Legacy Carried Internals

| Field | Value |
|---|---|
| ID | legacy-carried-internals |
| Root WI | - |
| Status | retired |
| Promise | Compatibility-only internals retained outside public scope. |
| Required Verification | smoke |
| Gate Inventory | `cargo test -p meter` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Legacy internals | epic | - | out_of_scope | verified | smoke | `cargo test -p meter` |
"#;
        let document = canonical_doc(body);
        let report = CapabilityReport {
            action: "capability",
            project: "meter".to_string(),
            cap_path: PathBuf::from("projects/meter/README.md"),
            format_version: 2,
            status: "healthy".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("meter"),
            production_ready: true,
            production_status: ProductionStatus::Ready,
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            capability_count: 0,
            verified_count: 0,
            percent: 100.0,
            claim_count: 1,
            verified_claim_count: 1,
            claim_percent: 100.0,
            capabilities: vec![CapabilityReportItem {
                id: "legacy-carried-internals".to_string(),
                title: "Legacy Carried Internals".to_string(),
                status: CapabilityStatus::Retired,
                capability_type: None,
                surfaces: Vec::new(),
                ec_dimensions: Vec::new(),
                promise: "Compatibility-only internals retained outside public scope.".to_string(),
                current_state: "Root WI: -; Gate inventory: `cargo test -p meter`".to_string(),
                gaps: vec![CapabilityGap {
                    id: "legacy-internals".to_string(),
                    status: CapabilityGapStatus::Deferred,
                    active_wi: None,
                    summary: "Legacy internals".to_string(),
                }],
                td_refs: Vec::new(),
                wi_refs: Vec::new(),
                wi_evidence: Vec::new(),
                claims: vec![CapabilityClaimReport {
                    id: "legacy-internals".to_string(),
                    user_story: "Legacy internals".to_string(),
                    required_for_verified: true,
                    maturity: CapabilityMaturity::Smoke,
                    oracle: "`cargo test -p meter`".to_string(),
                    fixtures: Vec::new(),
                    negative_cases: Vec::new(),
                    gates: vec![VerificationRuntimeResult {
                        id: "legacy-internals-gate".to_string(),
                        command: "cargo test -p meter".to_string(),
                        status: "pass".to_string(),
                        proves: Some("Legacy internals".to_string()),
                        exit_code: Some(0),
                        stdout: None,
                        stderr: None,
                    }],
                    verified: true,
                }],
                claim_count: 1,
                verified_claim_count: 1,
                claim_percent: 100.0,
                verification: Vec::new(),
                verified: true,
                release_scope: false,
                full_regenerability_required: false,
                dependencies: Vec::new(),
                dependency_closure: Vec::new(),
                production_ready: false,
                production_blockers: Vec::new(),
            }],
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_action: CapabilityAction {
                kind: CapabilityActionKind::None,
                capability_id: None,
                gap_id: None,
                claim_id: None,
                target: "meter".to_string(),
                command: String::new(),
                reason: String::new(),
                requires_hitl: false,
                hitl_question: None,
            },
            run_results: Vec::new(),
        };

        let types = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &types);

        assert_eq!(action.kind, CapabilityActionKind::None);
    }

    #[test]
    fn next_action_prefers_bounded_child_wi_when_epic_is_active() {
        let document = canonical_doc(one_markdown_capability());
        let report = CapabilityReport {
            action: "capability",
            project: "jet".to_string(),
            cap_path: PathBuf::from("projects/jet/README.md"),
            format_version: 1,
            status: "blocked".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("jet"),
            production_ready: false,
            production_status: ProductionStatus::NotEvaluated,
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            capability_count: 2,
            verified_count: 0,
            percent: 0.0,
            claim_count: 0,
            verified_claim_count: 0,
            claim_percent: 0.0,
            capabilities: vec![
                CapabilityReportItem {
                    id: "rust-native-frontend-toolchain".to_string(),
                    title: "Rust-Native Frontend Toolchain Replacement".to_string(),
                    status: CapabilityStatus::Auditing,
                    capability_type: None,
                    surfaces: Vec::new(),
                    ec_dimensions: Vec::new(),
                    promise: "replace frontend toolchain".to_string(),
                    current_state: "epic exists".to_string(),
                    gaps: vec![CapabilityGap {
                        id: "production-replacement-readiness".to_string(),
                        status: CapabilityGapStatus::Open,
                        active_wi: Some("#3778".to_string()),
                        summary: "audit pending".to_string(),
                    }],
                    td_refs: Vec::new(),
                    wi_refs: vec!["#3778".to_string()],
                    wi_evidence: vec![CapabilityWiEvidence {
                        reference: "#3778".to_string(),
                        gap_id: "production-replacement-readiness".to_string(),
                        issue_type: "epic".to_string(),
                        state: "open".to_string(),
                        phase: None,
                        expected_command: None,
                        title: "epic(jet): production replacement readiness".to_string(),
                    }],
                    claims: Vec::new(),
                    claim_count: 0,
                    verified_claim_count: 0,
                    claim_percent: 0.0,
                    verification: Vec::new(),
                    verified: false,
                    release_scope: false,
                    full_regenerability_required: false,
                    dependencies: Vec::new(),
                    dependency_closure: Vec::new(),
                    production_ready: false,
                    production_blockers: Vec::new(),
                },
                CapabilityReportItem {
                    id: "package-manager".to_string(),
                    title: "Package Manager".to_string(),
                    status: CapabilityStatus::Auditing,
                    capability_type: None,
                    surfaces: Vec::new(),
                    ec_dimensions: Vec::new(),
                    promise: "replace package manager flows".to_string(),
                    current_state: "surface exists".to_string(),
                    gaps: vec![CapabilityGap {
                        id: "package-manager-readiness".to_string(),
                        status: CapabilityGapStatus::Open,
                        active_wi: Some("#3779".to_string()),
                        summary: "audit pending".to_string(),
                    }],
                    td_refs: Vec::new(),
                    wi_refs: vec!["#3779".to_string()],
                    wi_evidence: vec![CapabilityWiEvidence {
                        reference: "#3779".to_string(),
                        gap_id: "package-manager-readiness".to_string(),
                        issue_type: "test".to_string(),
                        state: "open".to_string(),
                        phase: None,
                        expected_command: None,
                        title: "audit(jet): package manager production replacement readiness"
                            .to_string(),
                    }],
                    claims: Vec::new(),
                    claim_count: 0,
                    verified_claim_count: 0,
                    claim_percent: 0.0,
                    verification: Vec::new(),
                    verified: false,
                    release_scope: false,
                    full_regenerability_required: false,
                    dependencies: Vec::new(),
                    dependency_closure: Vec::new(),
                    production_ready: false,
                    production_blockers: Vec::new(),
                },
            ],
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_action: CapabilityAction {
                kind: CapabilityActionKind::None,
                capability_id: None,
                gap_id: None,
                claim_id: None,
                target: "jet".to_string(),
                command: String::new(),
                reason: String::new(),
                requires_hitl: false,
                hitl_question: None,
            },
            run_results: Vec::new(),
        };

        let types = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &types);

        assert_eq!(action.kind, CapabilityActionKind::RunTd);
        assert_eq!(action.capability_id.as_deref(), Some("package-manager"));
        assert_eq!(action.gap_id.as_deref(), Some("package-manager-readiness"));
        assert_eq!(action.command, "aw td create 3779");
    }

    #[test]
    fn next_action_requires_review_when_epic_children_are_closed() {
        let document = canonical_doc(one_markdown_capability());
        let report = CapabilityReport {
            action: "capability",
            project: "jet".to_string(),
            cap_path: PathBuf::from("projects/jet/README.md"),
            format_version: 1,
            status: "blocked".to_string(),
            test_gates: ProjectTestGateReport::not_evaluated("jet"),
            production_ready: false,
            production_status: ProductionStatus::NotEvaluated,
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            capability_count: 2,
            verified_count: 0,
            percent: 0.0,
            claim_count: 0,
            verified_claim_count: 0,
            claim_percent: 0.0,
            capabilities: vec![
                CapabilityReportItem {
                    id: "rust-native-frontend-toolchain".to_string(),
                    title: "Rust-Native Frontend Toolchain Replacement".to_string(),
                    status: CapabilityStatus::Auditing,
                    capability_type: None,
                    surfaces: Vec::new(),
                    ec_dimensions: Vec::new(),
                    promise: "replace frontend toolchain".to_string(),
                    current_state: "epic exists".to_string(),
                    gaps: vec![CapabilityGap {
                        id: "production-replacement-readiness".to_string(),
                        status: CapabilityGapStatus::Open,
                        active_wi: Some("#3778".to_string()),
                        summary: "audit pending".to_string(),
                    }],
                    td_refs: Vec::new(),
                    wi_refs: vec!["#3778".to_string()],
                    wi_evidence: vec![CapabilityWiEvidence {
                        reference: "#3778".to_string(),
                        gap_id: "production-replacement-readiness".to_string(),
                        issue_type: "epic".to_string(),
                        state: "open".to_string(),
                        phase: None,
                        expected_command: None,
                        title: "epic(jet): production replacement readiness".to_string(),
                    }],
                    claims: Vec::new(),
                    claim_count: 0,
                    verified_claim_count: 0,
                    claim_percent: 0.0,
                    verification: Vec::new(),
                    verified: false,
                    release_scope: false,
                    full_regenerability_required: false,
                    dependencies: Vec::new(),
                    dependency_closure: Vec::new(),
                    production_ready: false,
                    production_blockers: Vec::new(),
                },
                CapabilityReportItem {
                    id: "package-manager".to_string(),
                    title: "Package Manager".to_string(),
                    status: CapabilityStatus::Auditing,
                    capability_type: None,
                    surfaces: Vec::new(),
                    ec_dimensions: Vec::new(),
                    promise: "replace package manager flows".to_string(),
                    current_state: "surface exists".to_string(),
                    gaps: vec![CapabilityGap {
                        id: "package-manager-readiness".to_string(),
                        status: CapabilityGapStatus::Closed,
                        active_wi: Some("#3779".to_string()),
                        summary: "audit merged".to_string(),
                    }],
                    td_refs: Vec::new(),
                    wi_refs: vec!["#3779".to_string()],
                    wi_evidence: vec![CapabilityWiEvidence {
                        reference: "#3779".to_string(),
                        gap_id: "package-manager-readiness".to_string(),
                        issue_type: "test".to_string(),
                        state: "closed".to_string(),
                        phase: None,
                        expected_command: None,
                        title: "audit(jet): package manager production replacement readiness"
                            .to_string(),
                    }],
                    claims: Vec::new(),
                    claim_count: 0,
                    verified_claim_count: 0,
                    claim_percent: 0.0,
                    verification: Vec::new(),
                    verified: false,
                    release_scope: false,
                    full_regenerability_required: false,
                    dependencies: Vec::new(),
                    dependency_closure: Vec::new(),
                    production_ready: false,
                    production_blockers: Vec::new(),
                },
            ],
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_action: CapabilityAction {
                kind: CapabilityActionKind::None,
                capability_id: None,
                gap_id: None,
                claim_id: None,
                target: "jet".to_string(),
                command: String::new(),
                reason: String::new(),
                requires_hitl: false,
                hitl_question: None,
            },
            run_results: Vec::new(),
        };

        let types = all_typed(&report, &document);
        let action = choose_next_action(&report, &document, &types);

        assert_eq!(action.kind, CapabilityActionKind::HumanConfirmRequired);
        assert_eq!(
            action.capability_id.as_deref(),
            Some("rust-native-frontend-toolchain")
        );
        assert_eq!(
            action.gap_id.as_deref(),
            Some("production-replacement-readiness")
        );
        assert_eq!(
            action.command,
            "aw capability report --project jet --verify"
        );
        assert!(action.requires_hitl);
        let question = action.hitl_question.expect("epic rollup asks human");
        assert_eq!(question.tool_hint, "ask_user_question");
        assert_eq!(question.default_choice.as_deref(), Some("approve_rollup"));
    }

    #[test]
    fn lifecycle_action_for_reviewed_td_runs_cb_gen_with_spec_path() {
        let report = sample_report(sample_action(CapabilityActionKind::None, "", false));
        let evidence = CapabilityWiEvidence {
            reference: "#57".to_string(),
            gap_id: "generated-manual-ec-evidence-schema".to_string(),
            issue_type: "enhancement".to_string(),
            state: "open".to_string(),
            phase: Some("td_reviewed".to_string()),
            expected_command: None,
            title: "Promote generated manuals to first-class AW evidence artifacts".to_string(),
        };

        let (kind, command, reason) = lifecycle_action_for_work_item(
            &report,
            "57",
            Some(&evidence),
            Some("projects/agentic-workflow/tech-design/logic/manual.md"),
            None,
            "active WI exists; continue WI -> TD -> CB lifecycle",
        );

        assert_eq!(kind, CapabilityActionKind::RunCb);
        assert_eq!(
            command,
            "aw cb gen 57 --spec-path 'projects/agentic-workflow/tech-design/logic/manual.md'"
        );
        assert_eq!(reason, "active WI has reviewed TD; continue CB generation");
    }

    #[test]
    fn lifecycle_action_prefers_expected_command_from_workflow_projection() {
        let report = sample_report(sample_action(CapabilityActionKind::None, "", false));
        let evidence = CapabilityWiEvidence {
            reference: "#57".to_string(),
            gap_id: "generated-manual-ec-evidence-schema".to_string(),
            issue_type: "enhancement".to_string(),
            state: "open".to_string(),
            phase: Some("td_reviewed".to_string()),
            expected_command: Some("aw cb fill 57".to_string()),
            title: "Promote generated manuals to first-class AW evidence artifacts".to_string(),
        };

        let (kind, command, reason) = lifecycle_action_for_work_item(
            &report,
            "57",
            Some(&evidence),
            Some("projects/agentic-workflow/tech-design/logic/manual.md"),
            Some("approved"),
            "active WI exists; continue WI -> TD -> CB lifecycle",
        );

        assert_eq!(kind, CapabilityActionKind::RunCb);
        assert_eq!(command, "aw cb fill 57");
        assert_eq!(
            reason,
            "active WI has a workflow expected_command; follow lifecycle lock"
        );
    }

    #[test]
    fn lifecycle_action_routes_missing_issue_evidence_to_wi_plan_before_cb() {
        let report = sample_report(sample_action(CapabilityActionKind::None, "", false));

        let (kind, command, reason) = lifecycle_action_for_work_item(
            &report,
            "57",
            None,
            Some("projects/agentic-workflow/tech-design/logic/manual.md"),
            Some("approved"),
            "active WI exists; continue WI -> TD -> CB lifecycle",
        );

        assert_eq!(kind, CapabilityActionKind::CreateWi);
        assert_eq!(command, "aw wi plan --project jet");
        assert!(reason.contains("active WI reference is not present"));
    }

    #[test]
    fn lifecycle_action_routes_unknown_issue_evidence_to_wi_plan_before_cb() {
        let report = sample_report(sample_action(CapabilityActionKind::None, "", false));
        let evidence = CapabilityWiEvidence {
            reference: "#3783".to_string(),
            gap_id: "wasm-multi-target-readiness".to_string(),
            issue_type: "unknown".to_string(),
            state: "unknown".to_string(),
            phase: None,
            expected_command: None,
            title: String::new(),
        };

        let (kind, command, reason) = lifecycle_action_for_work_item(
            &report,
            "3783",
            Some(&evidence),
            Some(".aw/tech-design/projects/jet/specs/3783.md"),
            Some("approved"),
            "active WI exists; continue WI -> TD -> CB lifecycle",
        );

        assert_eq!(kind, CapabilityActionKind::CreateWi);
        assert_eq!(command, "aw wi plan --project jet");
        assert!(reason.contains("before TD/CB lifecycle"));
    }
}
// CODEGEN-END
