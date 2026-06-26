// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::cli::cb::{CbCodegenOriginSummary, CbColdVerifySummary, CbVerifySummary};
use crate::cli::production::{
    evaluate_release_scope, evaluate_release_scope_with_regenerability, inputs_from_sections,
    ProductionCapabilityReadiness, ProductionStatus,
};
use crate::cli::regenerability_policy::{resolve_regenerability_policy, RegenerabilityAuthority};
use crate::cli::standardize::{
    RegenerabilityCoverage, SemanticCoverage, StackMigrationCoverage, StandardizationCoverage,
    TraceabilityCoverage,
};
use crate::models::preflight::PreFlightGateReport;
use crate::models::project::EcBinding;

// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#cli
#[derive(Debug, Args, Clone)]
#[command(after_help = r#"Default output is a low-token metrics envelope.
Use `aw health --project <project> full` for the previous detailed report, or a
focused section: metrics, capability, gates, tests, ec, cb, cold, traceability,
regenerable, api, stack, td-lock, claims, blockers.
Use `-v/--verbose` to include progress events.

Output schema (JSON default):
{
  "schema_version": "aw.cli.v1",
  "event": "result",
  "status": "continue" | "blocked" | "done",
  "action": "health",
  "project": string,
  "completion": { "workflow_complete": bool, "requires_hitl": bool, "missing": [string] },
  "next": { "kind": "run_command" | "hitl" | "blocked" | "done" | "error", "command": string?, "reason": string },
  "readiness": object,
  "axes": { "capability": object, "ec": object, "ec_gen": object, "td": object, "td_gen": object },
  "blockers": object,
  "payload_path": string
}"#)]
/// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#cli
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct ProjectHealthArgs {
    // Configured project name from [[projects]] in .aw/config.toml.
    #[arg(long)]
    pub project: String,
    // Optional focused view. Omit for low-token top-level health metrics.
    #[arg(value_enum)]
    pub section: Option<ProjectHealthSection>,
    // Run expensive TD/source/CB traceability closure verification.
    #[arg(long)]
    pub verify_traceability: bool,
    // Run expensive deterministic CB replay/drift verification.
    #[arg(long)]
    pub verify_cb: bool,
    // Run expensive TD-only cold rebuild gates for verify_cold workspaces.
    #[arg(long)]
    pub verify_cold: bool,
    // Run configured workspace test commands as production release gates.
    #[arg(long)]
    pub verify_tests: bool,
    // Run external-contract commands from the project aw.toml EC inventory.
    #[arg(long)]
    pub verify_ec: bool,
    // DEPRECATED compatibility no-op. Agents should invoke `aw health --project <project>`.
    #[arg(long, hide = true)]
    pub json: bool,
    // Emit the legacy human-readable health report.
    #[arg(long)]
    pub human: bool,
    // Pretty-print the JSON report.
    #[arg(long)]
    pub pretty: bool,
    // Emit progress events before the final result envelope.
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub enum ProjectHealthSection {
    Full,
    Metrics,
    Capability,
    Gates,
    Tests,
    Ec,
    Cb,
    Cold,
    Traceability,
    Regenerable,
    Api,
    Stack,
    TdLock,
    Claims,
    Blockers,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct ProjectHealthReport {
    pub project: String,
    pub status: ProjectHealthStatus,
    pub capability_ready: bool,
    pub managed_ready: bool,
    pub semantic_ready: bool,
    pub traceability_ready: bool,
    pub takeover_ready: bool,
    pub generator_request_ready: bool,
    pub production_ready: bool,
    pub production_status: ProductionStatus,
    pub production_scope: Vec<String>,
    pub production_blockers: Vec<String>,
    pub global_blockers: Vec<String>,
    pub scoped_capabilities: Vec<ProductionCapabilityReadiness>,
    pub capability: CapabilityHealthReport,
    pub test_gates: ProjectTestGateReport,
    pub ec: ProjectEcGateReport,
    pub claim_closure: ProjectClaimClosureReport,
    /// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-preflight-gates.md#schema
    pub preflight_gate_reports: Vec<PreFlightGateReport>,
    /// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-preflight-gates.md#schema
    pub optional_quality_warnings: Vec<String>,
    pub managed_percent: f64,
    pub semantic_percent: f64,
    pub codegen_percent: f64,
    pub codegen_eligible_files: usize,
    pub codegen_files: usize,
    pub cb_ownership: CbOwnershipSummary,
    pub codegen_origin: CbCodegenOriginSummary,
    pub traceability_evaluated: bool,
    pub traceability_note: Option<String>,
    pub traceability_percent: f64,
    pub traceability_blocker_count: usize,
    pub traceability_internal_td_count: usize,
    pub traceability_orphan_td_count: usize,
    pub command_traceability_percent: f64,
    pub command_traceability_blocker_count: usize,
    pub command_traceability_hidden_command_count: usize,
    pub command_traceability_orphan_command_count: usize,
    pub traceability: TraceabilityCoverage,
    pub next_gap: Option<String>,
    pub blocked_gap_count: usize,
    pub human_decision_required_count: usize,
    pub handwrite_files: usize,
    pub unmarked_files: usize,
    pub cb_verify_evaluated: bool,
    pub cb_verify_note: Option<String>,
    pub cb_verify_clean: bool,
    pub public_api_covered: usize,
    pub public_api_total: usize,
    pub semantic_review_required: usize,
    pub cold_rebuild_evaluated: bool,
    pub cold_rebuild_note: Option<String>,
    pub cold_rebuild_clean: bool,
    pub cold_rebuild_workspace_count: usize,
    pub cold_rebuild_failures: Vec<String>,
    pub cold_rebuilds: Vec<CbColdVerifySummary>,
    pub stack_migration_percent: f64,
    pub stack_migration_incomplete_workspaces: usize,
    pub stack_migration: StackMigrationCoverage,
    pub workflow_lock_count: usize,
    pub td_lock: crate::cli::td_lock::TdLockStatus,
    pub regenerability_authority: RegenerabilityAuthorityReport,
    pub optional_regenerability_gaps: Vec<String>,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct CbOwnershipSummary {
    pub eligible_files: usize,
    pub codegen_files: usize,
    pub handwrite_files: usize,
    pub unmarked_files: usize,
    pub codegen_percent: f64,
    pub handwrite_percent: f64,
    pub unmarked_percent: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct CapabilityHealthReport {
    pub evaluated: bool,
    pub production_evaluated: bool,
    pub note: Option<String>,
    pub cap_path: String,
    pub format: String,
    pub format_version: u8,
    pub capability_count: usize,
    pub release_scope_count: usize,
    pub root_runner_ready: bool,
    pub production_ready_count: usize,
    pub production_scope_count: usize,
    pub production_percent: f64,
    pub blocker_count: usize,
    pub blockers: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
impl CapabilityHealthReport {
    fn ready_fixture(project: &str) -> Self {
        Self {
            evaluated: true,
            production_evaluated: true,
            note: None,
            cap_path: format!("projects/{project}/README.md"),
            format: "markdown_tables".to_string(),
            format_version: 2,
            capability_count: 1,
            release_scope_count: 1,
            root_runner_ready: true,
            production_ready_count: 1,
            production_scope_count: 1,
            production_percent: 100.0,
            blocker_count: 0,
            blockers: Vec::new(),
        }
    }

    fn blocked(project: &str, cap_path: String, format: &str, blocker: String) -> Self {
        Self {
            evaluated: true,
            production_evaluated: false,
            note: Some(format!(
                "capability readiness blocked for project `{project}`"
            )),
            cap_path,
            format: format.to_string(),
            format_version: 0,
            capability_count: 0,
            release_scope_count: 0,
            root_runner_ready: false,
            production_ready_count: 0,
            production_scope_count: 0,
            production_percent: 0.0,
            blocker_count: 1,
            blockers: vec![blocker],
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct RegenerabilityAuthorityReport {
    pub authority: RegenerabilityAuthority,
    pub required_for_production: bool,
    pub gap_count: usize,
    pub reason: String,
    pub blockers: Vec<String>,
    pub advisory_gaps: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub enum ProjectHealthStatus {
    Healthy,
    Blocked,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct ProjectTestGateReport {
    pub evaluated: bool,
    pub status: ProjectTestGateStatus,
    pub note: Option<String>,
    pub command_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub commands: Vec<ProjectTestCommandReport>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub enum ProjectTestGateStatus {
    NotEvaluated,
    NotConfigured,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct ProjectTestCommandReport {
    pub workspace: String,
    pub command: String,
    pub status: ProjectTestCommandStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub enum ProjectTestCommandStatus {
    Passed,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
/// Project aw.toml external-contract gate report.
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct ProjectEcGateReport {
    pub evaluated: bool,
    pub check_clean: bool,
    pub verify_evaluated: bool,
    pub status: ProjectEcGateStatus,
    pub note: Option<String>,
    pub inventory_path: String,
    pub expected_case_count: usize,
    pub case_count: usize,
    pub expected_tool_manifest_count: usize,
    pub tool_manifest_count: usize,
    pub command_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub findings: Vec<String>,
    pub commands: Vec<ProjectEcCommandReport>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub enum ProjectEcGateStatus {
    NotEvaluated,
    NotConfigured,
    CheckFailed,
    NotVerified,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct ProjectEcCommandReport {
    pub case_id: String,
    pub command: String,
    pub status: ProjectTestCommandStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
/// Capability claim graph-closure report across caps, EC, TD, and artifact health.
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct ProjectClaimClosureReport {
    pub evaluated: bool,
    pub note: Option<String>,
    pub claim_total: usize,
    pub closed_claim_count: usize,
    pub claim_closure_percent: f64,
    pub claims_with_ec: usize,
    pub claims_with_passing_ec: usize,
    pub claims_with_primary_td: usize,
    pub claims_with_artifact_evidence: usize,
    pub blocker_count: usize,
    pub blockers: Vec<String>,
    pub claims: Vec<ProjectClaimClosureItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub struct ProjectClaimClosureItem {
    pub capability_id: String,
    pub claim_id: String,
    pub ec_case_ids: Vec<String>,
    pub passing_ec_case_ids: Vec<String>,
    pub primary_td_refs: Vec<String>,
    pub artifact_evidence: bool,
    pub status: ProjectClaimClosureStatus,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub enum ProjectClaimClosureStatus {
    Closed,
    Blocked,
}

impl ProjectClaimClosureReport {
    pub(crate) fn not_evaluated(project: &str) -> Self {
        Self {
            evaluated: false,
            note: Some(format!(
                "claim closure not evaluated; run `aw health --project {project} claims`"
            )),
            claim_total: 0,
            closed_claim_count: 0,
            claim_closure_percent: 100.0,
            claims_with_ec: 0,
            claims_with_passing_ec: 0,
            claims_with_primary_td: 0,
            claims_with_artifact_evidence: 0,
            blocker_count: 0,
            blockers: Vec::new(),
            claims: Vec::new(),
        }
    }

    fn from_blocker(project: &str, blocker: String) -> Self {
        Self {
            evaluated: true,
            note: Some(format!("claim closure blocked for project `{project}`")),
            claim_total: 0,
            closed_claim_count: 0,
            claim_closure_percent: 0.0,
            claims_with_ec: 0,
            claims_with_passing_ec: 0,
            claims_with_primary_td: 0,
            claims_with_artifact_evidence: 0,
            blocker_count: 1,
            blockers: vec![blocker],
            claims: Vec::new(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
impl ProjectEcGateReport {
    pub(crate) fn not_evaluated(project: &str) -> Self {
        Self {
            evaluated: false,
            check_clean: true,
            verify_evaluated: false,
            status: ProjectEcGateStatus::NotEvaluated,
            note: Some(format!(
                "EC not evaluated; run `aw health --project {project} --verify-ec`"
            )),
            inventory_path: format!("projects/{project}/aw.toml"),
            expected_case_count: 0,
            case_count: 0,
            expected_tool_manifest_count: 0,
            tool_manifest_count: 0,
            command_count: 0,
            passed_count: 0,
            failed_count: 0,
            findings: Vec::new(),
            commands: Vec::new(),
        }
    }

    fn from_check(summary: crate::cli::ec::EcCheckSummary) -> Self {
        let status = if !summary.clean {
            ProjectEcGateStatus::CheckFailed
        } else if summary.case_count == 0 && summary.tool_manifest_count == 0 {
            ProjectEcGateStatus::NotConfigured
        } else {
            ProjectEcGateStatus::NotVerified
        };
        let note = match status {
            ProjectEcGateStatus::NotConfigured => Some(
                "EC inventory has no cases; add external-contract e2e-test sections and run `aw ec gen --project <project>`"
                    .to_string(),
            ),
            ProjectEcGateStatus::CheckFailed => Some("EC inventory/check is blocked".to_string()),
            ProjectEcGateStatus::NotVerified => Some(format!(
                "EC commands not evaluated; run `aw health --project {} --verify-ec`",
                summary.project
            )),
            _ => None,
        };
        Self {
            evaluated: true,
            check_clean: summary.clean,
            verify_evaluated: false,
            status,
            note,
            inventory_path: summary.inventory_path,
            expected_case_count: summary.expected_case_count,
            case_count: summary.case_count,
            expected_tool_manifest_count: summary.expected_tool_manifest_count,
            tool_manifest_count: summary.tool_manifest_count,
            command_count: 0,
            passed_count: 0,
            failed_count: 0,
            findings: summary.findings,
            commands: Vec::new(),
        }
    }
}

// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#logic
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub fn build_health_report(project: &str) -> Result<ProjectHealthReport> {
    build_health_report_with_options(project, true, true, true, true, true)
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub(crate) fn build_health_report_with_options(
    project: &str,
    verify_traceability: bool,
    verify_cb: bool,
    verify_cold: bool,
    verify_tests: bool,
    verify_ec: bool,
) -> Result<ProjectHealthReport> {
    build_health_report_with_options_internal(
        project,
        verify_traceability,
        verify_cb,
        verify_cold,
        verify_tests,
        verify_ec,
        false,
    )
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn build_health_report_with_options_internal(
    project: &str,
    verify_traceability: bool,
    verify_cb: bool,
    verify_cold: bool,
    verify_tests: bool,
    verify_ec: bool,
    emit_progress: bool,
) -> Result<ProjectHealthReport> {
    let project_root = crate::find_project_root()?;
    let project = resolve_health_project_name(&project_root, project)?;
    let caps_ec_only = project_health_caps_ec_only(&project);
    let verify_traceability = verify_traceability && !caps_ec_only;
    let verify_cb = verify_cb && !caps_ec_only;
    let verify_cold = verify_cold && !caps_ec_only;
    let progress = HealthProgressSink::new(&project, emit_progress);
    progress.emit(0, "start", "starting project health verification", None);
    let test_gates = if caps_ec_only {
        ProjectTestGateReport::skipped_by_caps_ec_policy(&project)
    } else {
        project_test_gate_report_with_progress(&project, &project_root, verify_tests, &progress)?
    };
    build_health_report_with_test_gates_and_capability_verified_internal(
        &project,
        verify_traceability,
        verify_cb,
        verify_cold,
        test_gates,
        if caps_ec_only {
            verify_ec
        } else {
            verify_tests && verify_cb && verify_traceability && verify_ec
        },
        None,
        &progress,
    )
    .and_then(|mut report| {
        apply_ec_to_report(&mut report, verify_ec)?;
        apply_claim_closure_to_report(&mut report)?;
        Ok(report)
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
#[allow(dead_code)]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub(crate) fn build_health_report_with_test_gates(
    project: &str,
    verify_traceability: bool,
    verify_cb: bool,
    verify_cold: bool,
    test_gates: ProjectTestGateReport,
    production_gates_evaluated: bool,
) -> Result<ProjectHealthReport> {
    build_health_report_with_test_gates_and_capability_verified(
        project,
        verify_traceability,
        verify_cb,
        verify_cold,
        test_gates,
        production_gates_evaluated,
        None,
    )
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub(crate) fn build_health_report_with_test_gates_and_capability_verified(
    project: &str,
    verify_traceability: bool,
    verify_cb: bool,
    verify_cold: bool,
    test_gates: ProjectTestGateReport,
    production_gates_evaluated: bool,
    capability_verified_by_id: Option<BTreeMap<String, bool>>,
) -> Result<ProjectHealthReport> {
    let progress = HealthProgressSink::disabled(project);
    build_health_report_with_test_gates_and_capability_verified_internal(
        project,
        verify_traceability,
        verify_cb,
        verify_cold,
        test_gates,
        production_gates_evaluated,
        capability_verified_by_id,
        &progress,
    )
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn build_health_report_with_test_gates_and_capability_verified_internal(
    project: &str,
    verify_traceability: bool,
    verify_cb: bool,
    verify_cold: bool,
    test_gates: ProjectTestGateReport,
    production_gates_evaluated: bool,
    capability_verified_by_id: Option<BTreeMap<String, bool>>,
    progress: &HealthProgressSink<'_>,
) -> Result<ProjectHealthReport> {
    let project_root = crate::find_project_root()?;
    let project = resolve_health_project_name(&project_root, project)?;
    let caps_ec_only = project_health_caps_ec_only(&project);
    let verify_traceability = verify_traceability && !caps_ec_only;
    let verify_cb = verify_cb && !caps_ec_only;
    let verify_cold = verify_cold && !caps_ec_only;
    progress.emit(
        30,
        "traceability",
        "evaluating standardization and traceability coverage",
        None,
    );
    let standardize = crate::generate::apply::with_quiet_apply_diagnostics(|| {
        crate::cli::standardize::project_health_standardize_coverage(
            &project,
            verify_traceability,
            verify_cb,
        )
    })?;
    let traceability_note = if verify_traceability {
        None
    } else if caps_ec_only {
        Some(format!(
            "traceability is advisory for `{project}` self-health; capability and EC gates are authoritative"
        ))
    } else {
        Some(format!(
            "traceability not evaluated; run `aw health --project {project} full`"
        ))
    };
    let (cb, cb_verify_note) = if verify_cb {
        progress.emit(50, "cb", "running deterministic CB verification", None);
        (
            crate::generate::apply::with_quiet_apply_diagnostics(|| {
                crate::cli::cb::project_force_regen_verify_summary(&project)
            })?,
            None,
        )
    } else if caps_ec_only {
        (
            cb_verify_not_evaluated(),
            Some(format!(
                "cb verify is advisory for `{project}` self-health; capability and EC gates are authoritative"
            )),
        )
    } else {
        (
            cb_verify_not_evaluated(),
            Some(format!(
                "cb verify not evaluated; run `aw health --project {project} full`"
            )),
        )
    };
    let cold_workspace_count =
        crate::cli::cb::project_force_regen_cold_verify_workspaces(&project)?.len();
    let production_gates_evaluated = if caps_ec_only {
        production_gates_evaluated
    } else {
        production_gates_evaluated && (verify_cold || cold_workspace_count == 0)
    };
    let cold_rebuilds = if verify_cold {
        progress.emit(70, "cold", "running cold rebuild verification", None);
        crate::generate::apply::with_quiet_apply_diagnostics(|| {
            crate::cli::cb::project_force_regen_cold_verify_summary(&project)
        })?
    } else {
        Vec::new()
    };
    progress.emit(95, "summary", "building health readiness summary", None);
    let mut report = ProjectHealthReport::from_components_with_traceability(
        &project,
        standardize.managed,
        standardize.semantic,
        standardize.traceability,
        standardize.regenerable,
        standardize.stack_migration,
        cb,
        cold_rebuilds,
        test_gates,
    );
    report.traceability_evaluated = verify_traceability;
    report.traceability_note = traceability_note.clone();
    if !caps_ec_only {
        report
            .blockers
            .extend(crate::cli::standardize::project_root_artifact_blockers(
                &project,
            )?);
    }
    if !caps_ec_only {
        if let Some(note) = traceability_note {
            report.blockers.push(note);
        }
    }
    report.cb_verify_evaluated = verify_cb;
    report.cb_verify_note = cb_verify_note.clone();
    if !caps_ec_only {
        if let Some(note) = cb_verify_note {
            report.blockers.push(note);
        }
    }
    if verify_cold && cold_workspace_count == 0 {
        report.cold_rebuild_evaluated = false;
        report.cold_rebuild_workspace_count = 0;
        report.cold_rebuild_clean = false;
        let note = format!(
            "not evaluated; project `{project}` has no workspace with `verify_cold = true`"
        );
        report.cold_rebuild_note = Some(note.clone());
        report.blockers.push(note);
        report.status = ProjectHealthStatus::Blocked;
        report.production_ready = false;
        report.production_status = ProductionStatus::Blocked;
    } else if !verify_cold {
        report.cold_rebuild_evaluated = false;
        report.cold_rebuild_workspace_count = cold_workspace_count;
        report.cold_rebuild_clean = true;
        report.cold_rebuild_note = if cold_workspace_count == 0 {
            None
        } else if caps_ec_only {
            Some(format!(
                "cold rebuild is advisory for `{project}` self-health; capability and EC gates are authoritative"
            ))
        } else {
            Some(format!(
                "cold rebuild not evaluated; run `aw health --project {project} full`"
            ))
        };
        if let Some(note) = &report.cold_rebuild_note {
            if !caps_ec_only {
                report.blockers.push(note.clone());
            }
        }
    }
    apply_scoped_production_readiness(
        &mut report,
        production_gates_evaluated,
        capability_verified_by_id,
    )?;
    Ok(report)
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn resolve_health_project_name(project_root: &std::path::Path, requested: &str) -> Result<String> {
    Ok(
        crate::services::project_registry::load_project_config_rows(project_root)?
            .into_iter()
            .find(|project| project.matches(requested))
            .map(|project| project.name)
            .unwrap_or_else(|| requested.to_string()),
    )
}

pub(crate) fn project_health_caps_ec_only(project: &str) -> bool {
    matches!(project, "agentic-workflow" | "aw")
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn cb_verify_not_evaluated() -> CbVerifySummary {
    CbVerifySummary {
        clean: true,
        public_api_covered: 0,
        public_api_total: 0,
        semantic_review_required: 0,
        failures: Vec::new(),
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn apply_scoped_production_readiness(
    report: &mut ProjectHealthReport,
    production_gates_evaluated: bool,
    capability_verified_by_id: Option<BTreeMap<String, bool>>,
) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let mut capability_health;
    let production = match crate::cli::capability::resolve_capability_path(
        &project_root,
        &report.project,
        None,
    ) {
        Ok(cap_path) => match std::fs::read_to_string(&cap_path) {
            Ok(body) => match crate::cli::capability::parse_capability_document(&body, &cap_path) {
                Ok(document) => {
                    let mut capability_blockers = document.findings.clone();
                    capability_blockers.extend(
                        crate::cli::capability::capability_profile_blockers_for_document(
                            &project_root,
                            &report.project,
                            &document,
                        ),
                    );
                    capability_blockers.sort();
                    capability_blockers.dedup();
                    for blocker in &capability_blockers {
                        if !report.blockers.contains(blocker) {
                            report.blockers.push(blocker.clone());
                        }
                    }
                    let capability_count = if document.is_legacy_only() {
                        document.legacy_rows.len()
                    } else {
                        document
                            .capabilities
                            .iter()
                            .filter(|capability| {
                                capability.status
                                    != crate::cli::capability::CapabilityStatus::Retired
                            })
                            .count()
                    };
                    let release_scope_count = document
                        .capabilities
                        .iter()
                        .filter(|capability| {
                            capability.status != crate::cli::capability::CapabilityStatus::Retired
                                && capability.release_scope
                        })
                        .count();
                    let root_runner_ready = matches!(
                        document.format,
                        crate::cli::capability::CapabilityDocumentFormat::MarkdownTables
                    ) && capability_blockers.is_empty()
                        && !document.capabilities.is_empty();
                    capability_health = CapabilityHealthReport {
                        evaluated: true,
                        production_evaluated: production_gates_evaluated,
                        note: if production_gates_evaluated {
                            None
                        } else {
                            Some(format!(
                                        "capability production readiness not evaluated; run `aw health --project {} full`",
                                        report.project
                                    ))
                        },
                        cap_path: cap_path.display().to_string(),
                        format: document.format.as_str().to_string(),
                        format_version: document.format_version(),
                        capability_count,
                        release_scope_count,
                        root_runner_ready,
                        production_ready_count: 0,
                        production_scope_count: 0,
                        production_percent: 0.0,
                        blocker_count: capability_blockers.len(),
                        blockers: capability_blockers,
                    };
                    let verified_by_id = capability_verified_by_id.clone().unwrap_or_else(|| {
                        crate::cli::capability::runtime_verified_by_id_from_sections(
                            &document.capabilities,
                            &project_root,
                            production_gates_evaluated,
                        )
                    });
                    let regenerability_gap_count = if project_health_caps_ec_only(&report.project) {
                        0
                    } else {
                        report.regenerability_authority.gap_count
                    };
                    evaluate_release_scope_with_regenerability(
                        inputs_from_sections(&document.capabilities, &verified_by_id),
                        report.blockers.clone(),
                        production_gates_evaluated,
                        regenerability_gap_count,
                    )
                }
                Err(err) => {
                    let blocker = format!("capability document parse failed: {err}");
                    capability_health = CapabilityHealthReport::blocked(
                        &report.project,
                        cap_path.display().to_string(),
                        "unparseable",
                        blocker.clone(),
                    );
                    evaluate_release_scope(Vec::new(), vec![blocker], production_gates_evaluated)
                }
            },
            Err(err) => {
                let blocker = format!("capability document read failed: {err}");
                capability_health = CapabilityHealthReport::blocked(
                    &report.project,
                    cap_path.display().to_string(),
                    "missing",
                    blocker.clone(),
                );
                evaluate_release_scope(Vec::new(), vec![blocker], production_gates_evaluated)
            }
        },
        Err(err) => {
            let blocker = format!("capability path resolution failed: {err}");
            capability_health = CapabilityHealthReport::blocked(
                &report.project,
                String::new(),
                "unresolved",
                blocker.clone(),
            );
            evaluate_release_scope(Vec::new(), vec![blocker], production_gates_evaluated)
        }
    };

    for blocker in &production.production_blockers {
        if !report.blockers.contains(blocker) {
            report.blockers.push(blocker.clone());
        }
    }
    report.blockers.sort();
    report.blockers.dedup();
    report.production_ready = production.production_ready;
    report.production_status = production.production_status;
    report.production_scope = production.production_scope;
    report.production_blockers = production.production_blockers;
    report.global_blockers = production.global_blockers;
    report.scoped_capabilities = production.capabilities;
    capability_health.production_ready_count = report
        .scoped_capabilities
        .iter()
        .filter(|capability| capability.production_ready)
        .count();
    capability_health.production_scope_count = report.production_scope.len();
    capability_health.production_percent = percent_of(
        capability_health.production_ready_count,
        capability_health.production_scope_count,
    );
    report.capability = capability_health;
    report.status = if report.blockers.is_empty() {
        ProjectHealthStatus::Healthy
    } else {
        ProjectHealthStatus::Blocked
    };
    report.refresh_takeover_readiness();
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
impl ProjectHealthReport {
    // @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#logic
    pub fn from_components(
        project: &str,
        managed: StandardizationCoverage,
        semantic: SemanticCoverage,
        regenerable: RegenerabilityCoverage,
        stack_migration: StackMigrationCoverage,
        cb: CbVerifySummary,
        cold_rebuilds: Vec<CbColdVerifySummary>,
        test_gates: ProjectTestGateReport,
    ) -> Self {
        Self::from_components_with_traceability(
            project,
            managed,
            semantic,
            TraceabilityCoverage::ready_fixture(project),
            regenerable,
            stack_migration,
            cb,
            cold_rebuilds,
            test_gates,
        )
    }

    // @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#logic
    pub fn from_components_with_traceability(
        project: &str,
        managed: StandardizationCoverage,
        semantic: SemanticCoverage,
        traceability: TraceabilityCoverage,
        regenerable: RegenerabilityCoverage,
        stack_migration: StackMigrationCoverage,
        cb: CbVerifySummary,
        cold_rebuilds: Vec<CbColdVerifySummary>,
        test_gates: ProjectTestGateReport,
    ) -> Self {
        let caps_ec_only = project_health_caps_ec_only(project);
        let mut blockers = Vec::new();
        let mut regenerability_gaps = Vec::new();
        if !caps_ec_only && !managed.uncovered_files.is_empty() {
            blockers.push(format!(
                "{} unmanaged source file(s)",
                managed.uncovered_files.len()
            ));
        }
        if regenerable.handwrite_files > 0 {
            regenerability_gaps.push(format!(
                "{} file(s) are HANDWRITE because they are not TD AST-codegen",
                regenerable.handwrite_files
            ));
        }
        if regenerable.unmarked_files > 0 {
            regenerability_gaps.push(format!(
                "{} source file(s) have no ownership marker",
                regenerable.unmarked_files
            ));
        }
        if !regenerable.unsupported_codegen_files.is_empty() {
            regenerability_gaps.push(format!(
                "{} file(s) are wrapped as CODEGEN but are not AST-codegen replay-supported",
                regenerable.unsupported_codegen_files.len()
            ));
        }
        if !regenerable.non_replayable_codegen_files.is_empty() {
            regenerability_gaps.push(format!(
                "{} file(s) are wrapped as CODEGEN but backed by hand-written TD changes; mark HANDWRITE or implement AST codegen",
                regenerable.non_replayable_codegen_files.len()
            ));
        }
        if !regenerable.snapshot_codegen_files.is_empty() {
            regenerability_gaps.push(format!(
                "{} file(s) are wrapped as CODEGEN but use source-template/artifact replay instead of TD AST codegen",
                regenerable.snapshot_codegen_files.len()
            ));
        }
        if !regenerable.codegen_drift_files.is_empty() {
            regenerability_gaps.push(format!(
                "{} file(s) are wrapped as CODEGEN but have audit/replay drift",
                regenerable.codegen_drift_files.len()
            ));
        }
        if regenerable.missing_generator_primitive_gaps > 0 {
            regenerability_gaps.push(format!(
                "{} semantic gap(s) are missing generator primitives",
                regenerable.missing_generator_primitive_gaps
            ));
        }
        if regenerable.insufficient_td_section_gaps > 0 {
            regenerability_gaps.push(format!(
                "{} semantic gap(s) need stronger TD sections",
                regenerable.insufficient_td_section_gaps
            ));
        }
        if regenerable.human_decision_required_gaps > 0 {
            regenerability_gaps.push(format!(
                "{} semantic gap(s) require human generator-policy decisions",
                regenerable.human_decision_required_gaps
            ));
        }
        let regenerability_authority =
            regenerability_authority_report(project, &regenerable, regenerability_gaps);
        if !caps_ec_only && regenerability_authority.required_for_production {
            blockers.extend(regenerability_authority.blockers.iter().cloned());
        }
        let managed_ready = managed.percent >= 100.0 && managed.uncovered_files.is_empty();
        if !caps_ec_only && !semantic.uncovered_files.is_empty() {
            blockers.push(format!(
                "semantic TD coverage incomplete: {}/{}",
                semantic.semantically_covered_files, semantic.total_files
            ));
        }
        if let Some(gap) = &semantic.next_gap {
            if !caps_ec_only && semantic_gap_blocks_readiness(&gap.primitive) {
                blockers.push(format!(
                    "next semantic gap: {} {}",
                    gap.target, gap.primitive
                ));
            }
        }
        let semantic_ready = semantic.percent >= 100.0
            && semantic.uncovered_files.is_empty()
            && semantic
                .next_gap
                .as_ref()
                .is_none_or(|gap| !semantic_gap_blocks_readiness(&gap.primitive))
            && semantic.blocked_gap_count == 0
            && semantic.human_decision_required_count == 0;
        if !caps_ec_only && traceability.blocker_count > 0 {
            blockers.push(format!(
                "traceability closure incomplete: {} blocker(s)",
                traceability.blocker_count
            ));
            blockers.extend(traceability.blockers.iter().take(20).map(|blocker| {
                format!(
                    "traceability {}: {}{} ({})",
                    blocker.kind.as_str(),
                    blocker.target,
                    blocker
                        .source
                        .as_ref()
                        .map(|source| format!(" <- {source}"))
                        .unwrap_or_default(),
                    blocker.reason
                )
            }));
        }
        if !caps_ec_only && !cb.clean {
            blockers.push(format!("cb verify has {} finding(s)", cb.failures.len()));
            blockers.extend(cb.failures.iter().cloned());
        }
        if !caps_ec_only && cb.public_api_total > cb.public_api_covered {
            blockers.push(format!(
                "public API semantic coverage incomplete: {}/{}",
                cb.public_api_covered, cb.public_api_total
            ));
        }
        // Semantic review units are surfaced for agent sampling, but target-derived
        // source templates are not deterministic governance failures by themselves.
        let mut cold_rebuild_failures = Vec::new();
        for summary in &cold_rebuilds {
            let workspace = summary.workspace.as_deref().unwrap_or("<project>");
            for failure in &summary.failures {
                cold_rebuild_failures.push(format!("{workspace}: {failure}"));
            }
        }
        if !caps_ec_only && !cold_rebuild_failures.is_empty() {
            blockers.push(format!(
                "cold rebuild failed: {} finding(s)",
                cold_rebuild_failures.len()
            ));
            blockers.extend(cold_rebuild_failures.iter().cloned());
        }
        let codegen_origin = aggregate_codegen_origin(&cold_rebuilds);
        let cb_ownership = cb_ownership_summary(
            regenerable.eligible_files,
            regenerable.codegen_files,
            regenerable.handwrite_files,
            regenerable.unmarked_files,
        );
        if !caps_ec_only && stack_migration.incomplete_workspace_count > 0 {
            blockers.push(format!(
                "stack migration classification incomplete: {}/{} workspace(s)",
                stack_migration.incomplete_workspace_count,
                stack_migration.workspaces.len()
            ));
        }
        if !caps_ec_only {
            blockers.extend(stack_migration.blockers.iter().cloned());
            match test_gates.status {
                ProjectTestGateStatus::Passed => {}
                ProjectTestGateStatus::NotEvaluated => {
                    blockers.push(
                        test_gates
                            .note
                            .clone()
                            .unwrap_or_else(|| "test gates not evaluated".to_string()),
                    );
                }
                ProjectTestGateStatus::NotConfigured => {
                    blockers.push(
                        test_gates
                            .note
                            .clone()
                            .unwrap_or_else(|| "no workspace test_cmd configured".to_string()),
                    );
                }
                ProjectTestGateStatus::Failed => {
                    blockers.push(format!(
                        "test gates failed: {}/{} command(s)",
                        test_gates.failed_count, test_gates.command_count
                    ));
                    blockers.extend(
                        test_gates
                            .commands
                            .iter()
                            .filter(|cmd| cmd.status == ProjectTestCommandStatus::Failed)
                            .map(|cmd| {
                                format!(
                                    "{} `{}` failed with exit {:?}",
                                    cmd.workspace, cmd.command, cmd.exit_code
                                )
                            }),
                    );
                }
            }
        }

        let status = if blockers.is_empty() {
            ProjectHealthStatus::Healthy
        } else {
            ProjectHealthStatus::Blocked
        };
        let production_ready = blockers.is_empty();
        let traceability_ready = traceability.blocker_count == 0
            && traceability.next_gap.is_none()
            && traceability.command_traceability.blockers.is_empty()
            && traceability.command_traceability.next_gap.is_none()
            && traceability.command_traceability.hidden_command_count == 0
            && traceability.command_traceability.orphan_command_count == 0;
        let capability_ready = true;
        let takeover_ready = if caps_ec_only {
            capability_ready
        } else {
            capability_ready && managed_ready && semantic_ready && traceability_ready
        };
        let generator_request_ready = takeover_ready;

        Self {
            project: project.to_string(),
            status,
            capability_ready,
            managed_ready,
            semantic_ready,
            traceability_ready,
            takeover_ready,
            generator_request_ready,
            production_ready,
            production_status: if production_ready {
                ProductionStatus::Ready
            } else {
                ProductionStatus::Blocked
            },
            production_scope: Vec::new(),
            production_blockers: Vec::new(),
            global_blockers: Vec::new(),
            scoped_capabilities: Vec::new(),
            capability: CapabilityHealthReport::ready_fixture(project),
            test_gates,
            ec: ProjectEcGateReport::not_evaluated(project),
            claim_closure: ProjectClaimClosureReport::not_evaluated(project),
            preflight_gate_reports: Vec::new(),
            optional_quality_warnings: Vec::new(),
            managed_percent: managed.percent,
            semantic_percent: semantic.percent,
            codegen_percent: regenerable.percent,
            codegen_eligible_files: regenerable.eligible_files,
            codegen_files: regenerable.codegen_files,
            cb_ownership,
            codegen_origin,
            traceability_evaluated: true,
            traceability_note: None,
            traceability_percent: traceability.traceability_percent,
            traceability_blocker_count: traceability.blocker_count,
            traceability_internal_td_count: traceability.internal_td_count,
            traceability_orphan_td_count: traceability.orphan_td_count,
            command_traceability_percent: traceability
                .command_traceability
                .command_traceability_percent,
            command_traceability_blocker_count: traceability.command_traceability.blockers.len(),
            command_traceability_hidden_command_count: traceability
                .command_traceability
                .hidden_command_count,
            command_traceability_orphan_command_count: traceability
                .command_traceability
                .orphan_command_count,
            traceability: traceability.clone(),
            next_gap: traceability
                .command_traceability
                .next_gap
                .as_ref()
                .map(|gap| format!("{} {}", gap.kind.as_str(), gap.target))
                .or_else(|| {
                    semantic
                        .next_gap
                        .as_ref()
                        .map(|gap| format!("{} {}", gap.target, gap.primitive))
                })
                .or_else(|| {
                    traceability
                        .next_gap
                        .as_ref()
                        .map(|gap| format!("{} {}", gap.kind.as_str(), gap.target))
                })
                .or_else(|| {
                    regenerable
                        .next_gap
                        .as_ref()
                        .map(|gap| format!("{} {}", gap.target, gap.primitive))
                }),
            blocked_gap_count: semantic.blocked_gap_count,
            human_decision_required_count: semantic.human_decision_required_count,
            handwrite_files: regenerable.handwrite_files,
            unmarked_files: regenerable.unmarked_files,
            cb_verify_evaluated: true,
            cb_verify_note: None,
            cb_verify_clean: cb.clean,
            public_api_covered: cb.public_api_covered,
            public_api_total: cb.public_api_total,
            semantic_review_required: cb.semantic_review_required,
            cold_rebuild_evaluated: true,
            cold_rebuild_note: None,
            cold_rebuild_clean: cold_rebuild_failures.is_empty(),
            cold_rebuild_workspace_count: cold_rebuilds.len(),
            cold_rebuild_failures,
            cold_rebuilds,
            stack_migration_percent: stack_migration.migration_normalized_percent,
            stack_migration_incomplete_workspaces: stack_migration.incomplete_workspace_count,
            stack_migration,
            workflow_lock_count: 0,
            td_lock: crate::cli::td_lock::TdLockStatus::ready_fixture(project),
            regenerability_authority: regenerability_authority.clone(),
            optional_regenerability_gaps: regenerability_authority.advisory_gaps.clone(),
            blockers,
        }
    }

    /// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-preflight-gates.md#logic
    pub fn apply_preflight_gate_report(&mut self, report: PreFlightGateReport) {
        self.production_blockers
            .extend(report.production_blockers().iter().cloned());
        self.blockers
            .extend(report.production_blockers().iter().cloned());
        self.optional_quality_warnings
            .extend(report.quality_warnings().iter().cloned());
        self.preflight_gate_reports.push(report);

        self.production_blockers.sort();
        self.production_blockers.dedup();
        self.blockers.sort();
        self.blockers.dedup();
        self.optional_quality_warnings.sort();
        self.optional_quality_warnings.dedup();

        if self.blockers.is_empty() {
            self.status = ProjectHealthStatus::Healthy;
            self.production_ready = true;
            self.production_status = ProductionStatus::Ready;
        } else {
            self.status = ProjectHealthStatus::Blocked;
            self.production_ready = false;
            self.production_status = ProductionStatus::Blocked;
        }
    }

    fn refresh_takeover_readiness(&mut self) {
        self.capability_ready = self.capability.evaluated
            && self.capability.root_runner_ready
            && self.capability.capability_count > 0
            && self.capability.blocker_count == 0
            && self.capability.blockers.is_empty();
        self.traceability_ready = self.traceability_evaluated
            && self.traceability_blocker_count == 0
            && self.command_traceability_blocker_count == 0
            && self.command_traceability_hidden_command_count == 0
            && self.command_traceability_orphan_command_count == 0;
        if project_health_caps_ec_only(&self.project) {
            let ec_ready = self.ec.check_clean
                && (!self.ec.verify_evaluated
                    || matches!(self.ec.status, ProjectEcGateStatus::Passed));
            self.takeover_ready = self.capability_ready && ec_ready;
            self.generator_request_ready = self.takeover_ready;
            return;
        }
        self.takeover_ready = self.capability_ready
            && self.managed_ready
            && self.semantic_ready
            && self.traceability_ready
            && self.blocked_gap_count == 0
            && self.human_decision_required_count == 0
            && self.td_lock.clean
            && self.ec.check_clean
            && self.workflow_lock_count == 0;
        self.generator_request_ready = self.takeover_ready;
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn regenerability_authority_report(
    project: &str,
    coverage: &RegenerabilityCoverage,
    gaps: Vec<String>,
) -> RegenerabilityAuthorityReport {
    let policy = resolve_regenerability_policy(Some(project));
    let gap_count = regenerability_gap_count(coverage);
    let required_for_production = policy.required_for_production();
    let blockers = if required_for_production && gap_count > 0 {
        gaps.iter()
            .map(|gap| format!("regenerability required for production: {gap}"))
            .collect()
    } else {
        Vec::new()
    };
    let advisory_gaps = if required_for_production {
        Vec::new()
    } else {
        gaps
    };

    RegenerabilityAuthorityReport {
        authority: policy.authority,
        required_for_production,
        gap_count,
        reason: policy.reason,
        blockers,
        advisory_gaps,
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn regenerability_gap_count(coverage: &RegenerabilityCoverage) -> usize {
    coverage.gap_files.len()
        + coverage.missing_generator_primitive_gaps
        + coverage.insufficient_td_section_gaps
        + coverage.human_decision_required_gaps
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
struct HealthProgressSink<'a> {
    project: &'a str,
    started: Instant,
    enabled: bool,
}

impl<'a> HealthProgressSink<'a> {
    fn new(project: &'a str, enabled: bool) -> Self {
        Self {
            project,
            started: Instant::now(),
            enabled,
        }
    }

    fn disabled(project: &'a str) -> Self {
        Self::new(project, false)
    }

    fn emit(&self, percent: u8, phase: &str, message: &str, command: Option<&str>) {
        if !self.enabled {
            return;
        }
        let event = serde_json::json!({
            "schema_version": "aw.cli.v1",
            "event": "progress",
            "project": self.project,
            "percent": percent.min(100),
            "phase": phase,
            "message": message,
            "elapsed_ms": self.started.elapsed().as_millis(),
            "command": command,
        });
        println!("{event}");
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
impl ProjectTestGateReport {
    pub fn not_evaluated(project: &str) -> Self {
        Self {
            evaluated: false,
            status: ProjectTestGateStatus::NotEvaluated,
            note: Some(format!(
                "test gates not evaluated; run `aw health --project {project} full`"
            )),
            command_count: 0,
            passed_count: 0,
            failed_count: 0,
            skipped_count: 0,
            commands: Vec::new(),
        }
    }

    pub fn passed_fixture(command: &str) -> Self {
        Self {
            evaluated: true,
            status: ProjectTestGateStatus::Passed,
            note: None,
            command_count: 1,
            passed_count: 1,
            failed_count: 0,
            skipped_count: 0,
            commands: vec![ProjectTestCommandReport {
                workspace: "demo".to_string(),
                command: command.to_string(),
                status: ProjectTestCommandStatus::Passed,
                exit_code: Some(0),
                duration_ms: 0,
                stdout_tail: String::new(),
                stderr_tail: String::new(),
            }],
        }
    }

    fn skipped_by_caps_ec_policy(project: &str) -> Self {
        Self {
            evaluated: true,
            status: ProjectTestGateStatus::Passed,
            note: Some(format!(
                "workspace test gates are advisory for `{project}` self-health; capability and EC gates are authoritative"
            )),
            command_count: 0,
            passed_count: 0,
            failed_count: 0,
            skipped_count: 0,
            commands: Vec::new(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub(crate) fn project_test_gate_report(
    project: &str,
    project_root: &std::path::Path,
    verify_tests: bool,
) -> Result<ProjectTestGateReport> {
    let progress = HealthProgressSink::disabled(project);
    project_test_gate_report_with_progress(project, project_root, verify_tests, &progress)
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_test_gate_report_with_progress(
    project: &str,
    project_root: &std::path::Path,
    verify_tests: bool,
    progress: &HealthProgressSink<'_>,
) -> Result<ProjectTestGateReport> {
    if !verify_tests {
        return Ok(ProjectTestGateReport::not_evaluated(project));
    }
    if project_health_caps_ec_only(project) {
        return Ok(ProjectTestGateReport::skipped_by_caps_ec_policy(project));
    }
    progress.emit(10, "tests", "loading configured test gates", None);

    let projects = crate::services::project_registry::load_projects(project_root)?;
    let Some(row) = projects.iter().find(|row| row.name == project) else {
        return Ok(ProjectTestGateReport {
            evaluated: true,
            status: ProjectTestGateStatus::NotConfigured,
            note: Some(format!(
                "project `{project}` is not configured in .aw/config.toml"
            )),
            command_count: 0,
            passed_count: 0,
            failed_count: 0,
            skipped_count: 0,
            commands: Vec::new(),
        });
    };

    let configured: Vec<(String, String)> = row
        .workspaces
        .iter()
        .filter_map(|workspace| {
            let cmd = workspace.test_cmd.as_ref()?;
            Some((
                workspace.name.clone().unwrap_or_else(|| row.name.clone()),
                cmd.clone(),
            ))
        })
        .collect();

    if configured.is_empty() {
        return Ok(ProjectTestGateReport {
            evaluated: true,
            status: ProjectTestGateStatus::NotConfigured,
            note: Some(format!(
                "project `{project}` has no workspace `test_cmd`; configure tests in .aw/config.toml"
            )),
            command_count: 0,
            passed_count: 0,
            failed_count: 0,
            skipped_count: row.workspaces.len(),
            commands: Vec::new(),
        });
    }

    let mut commands = Vec::new();
    for (workspace, command) in configured {
        commands.push(run_project_test_command(
            &workspace,
            &command,
            project_root,
            progress,
        )?);
    }
    let passed_count = commands
        .iter()
        .filter(|cmd| cmd.status == ProjectTestCommandStatus::Passed)
        .count();
    let failed_count = commands.len() - passed_count;
    let status = if failed_count == 0 {
        ProjectTestGateStatus::Passed
    } else {
        ProjectTestGateStatus::Failed
    };

    Ok(ProjectTestGateReport {
        evaluated: true,
        status,
        note: None,
        command_count: commands.len(),
        passed_count,
        failed_count,
        skipped_count: row.workspaces.len().saturating_sub(commands.len()),
        commands,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn run_project_test_command(
    workspace: &str,
    command: &str,
    project_root: &std::path::Path,
    progress: &HealthProgressSink<'_>,
) -> Result<ProjectTestCommandReport> {
    run_project_test_command_with_timeout(
        workspace,
        command,
        project_root,
        progress,
        project_test_gate_timeout(),
    )
}

const PROJECT_TEST_GATE_TIMEOUT_ENV: &str = "AW_TEST_GATE_TIMEOUT_SECS";
const DEFAULT_PROJECT_TEST_GATE_TIMEOUT_SECS: u64 = 30 * 60;

fn project_test_gate_timeout() -> Duration {
    std::env::var(PROJECT_TEST_GATE_TIMEOUT_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_PROJECT_TEST_GATE_TIMEOUT_SECS))
}

fn run_project_test_command_with_timeout(
    workspace: &str,
    command: &str,
    project_root: &std::path::Path,
    progress: &HealthProgressSink<'_>,
    timeout: Duration,
) -> Result<ProjectTestCommandReport> {
    let started = Instant::now();
    progress.emit(
        15,
        "tests",
        &format!("running configured test gate for workspace `{workspace}`"),
        Some(command),
    );

    let stdout_file = tempfile::NamedTempFile::new()
        .with_context(|| format!("create stdout capture for test command `{command}`"))?;
    let stderr_file = tempfile::NamedTempFile::new()
        .with_context(|| format!("create stderr capture for test command `{command}`"))?;
    let stdout = stdout_file
        .reopen()
        .with_context(|| format!("open stdout capture for test command `{command}`"))?;
    let stderr = stderr_file
        .reopen()
        .with_context(|| format!("open stderr capture for test command `{command}`"))?;

    let mut command_process = crate::cli::shell_env::protected_shell_command(project_root, command);
    configure_test_gate_process_group(&mut command_process);
    let mut child = command_process
        .current_dir(project_root)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .with_context(|| format!("failed to execute test command `{command}`"))?;

    let mut next_progress = Duration::from_secs(10);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .with_context(|| format!("poll test command `{command}`"))?
        {
            break Some(status);
        }
        let elapsed = started.elapsed();
        if elapsed >= timeout {
            timed_out = true;
            progress.emit(
                80,
                "tests",
                &format!(
                    "test gate timed out for workspace `{workspace}` after {}s",
                    elapsed.as_secs()
                ),
                Some(command),
            );
            terminate_test_gate_child(&mut child);
            break None;
        }
        if elapsed >= next_progress {
            progress.emit(
                80,
                "tests",
                &format!(
                    "test gate still running for workspace `{workspace}` after {}s",
                    elapsed.as_secs()
                ),
                Some(command),
            );
            next_progress += Duration::from_secs(30);
        }
        thread::sleep(Duration::from_millis(250));
    };

    let duration_ms = started.elapsed().as_millis();
    let stdout = fs::read(stdout_file.path())
        .with_context(|| format!("read stdout capture for test command `{command}`"))?;
    let stderr = fs::read(stderr_file.path())
        .with_context(|| format!("read stderr capture for test command `{command}`"))?;
    let exit_code = status.as_ref().and_then(|status| status.code());
    let command_status = if timed_out {
        ProjectTestCommandStatus::TimedOut
    } else if status
        .as_ref()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        ProjectTestCommandStatus::Passed
    } else {
        ProjectTestCommandStatus::Failed
    };
    progress.emit(
        85,
        "tests",
        &format!("test gate finished for workspace `{workspace}` with status {command_status:?}"),
        Some(command),
    );
    let mut stderr_tail = tail_lossy(&stderr, 4000);
    if timed_out {
        if !stderr_tail.trim().is_empty() {
            stderr_tail.push('\n');
        }
        stderr_tail.push_str(&format!(
            "aw test gate timed out after {}s; set {PROJECT_TEST_GATE_TIMEOUT_ENV} to override",
            timeout.as_secs()
        ));
    }

    Ok(ProjectTestCommandReport {
        workspace: workspace.to_string(),
        command: command.to_string(),
        status: command_status,
        exit_code,
        duration_ms,
        stdout_tail: tail_lossy(&stdout, 4000),
        stderr_tail,
    })
}

#[cfg(unix)]
fn configure_test_gate_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_test_gate_process_group(_command: &mut Command) {}

fn terminate_test_gate_child(child: &mut std::process::Child) {
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

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn tail_lossy(bytes: &[u8], max_chars: usize) -> String {
    let text = String::from_utf8_lossy(bytes);
    let len = text.chars().count();
    if len <= max_chars {
        text.into_owned()
    } else {
        text.chars().skip(len - max_chars).collect()
    }
}

const HEALTH_SUMMARY_PREVIEW_LIMIT: usize = 20;
const HEALTH_COMPACT_PREVIEW_LIMIT: usize = 5;

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub fn project_health_compact_summary(report: &ProjectHealthReport) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "aw.cli.v1",
        "event": "result",
        "status": project_health_loop_status(report),
        "action": "health",
        "project": &report.project,
        "completion": project_health_compact_completion(report),
        "next": project_health_next(report),
        "readiness": project_health_compact_readiness(report),
        "axes": project_health_axes_summary(report),
        "blockers": project_health_compact_blockers(report),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub fn project_health_compact_summary_with_payload_path(
    report: &ProjectHealthReport,
    payload_path: &str,
) -> serde_json::Value {
    with_payload_path(project_health_compact_summary(report), payload_path)
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub fn project_health_section_summary(
    report: &ProjectHealthReport,
    section: ProjectHealthSection,
) -> serde_json::Value {
    if section == ProjectHealthSection::Full {
        return project_health_summary(report);
    }
    let payload = match section {
        ProjectHealthSection::Full => unreachable!(),
        ProjectHealthSection::Metrics => serde_json::json!({
            "readiness": project_health_compact_readiness(report),
            "axes": project_health_axes_summary(report),
        }),
        ProjectHealthSection::Capability => project_health_capability_summary(&report.capability),
        ProjectHealthSection::Gates => serde_json::json!({
            "tests": project_test_gate_summary(&report.test_gates),
            "ec": project_ec_gate_summary(&report.ec),
            "claim_closure": project_claim_closure_summary(&report.claim_closure),
            "cb_verify_evaluated": report.cb_verify_evaluated,
            "cb_verify_clean": report.cb_verify_clean,
            "cold_rebuild_evaluated": report.cold_rebuild_evaluated,
            "cold_rebuild_clean": report.cold_rebuild_clean,
            "td_lock": project_td_lock_summary(&report.td_lock),
            "workflow_lock_count": report.workflow_lock_count,
        }),
        ProjectHealthSection::Tests => project_test_gate_summary(&report.test_gates),
        ProjectHealthSection::Ec => project_ec_gate_summary(&report.ec),
        ProjectHealthSection::Claims => project_claim_closure_detail(&report.claim_closure),
        ProjectHealthSection::Cb => serde_json::json!({
            "cb_verify_evaluated": report.cb_verify_evaluated,
            "cb_verify_clean": report.cb_verify_clean,
            "cb_verify_note": &report.cb_verify_note,
            "cb_ownership": &report.cb_ownership,
            "codegen_origin": &report.codegen_origin,
            "codegen_percent": report.codegen_percent,
            "codegen_eligible_files": report.codegen_eligible_files,
            "codegen_files": report.codegen_files,
        }),
        ProjectHealthSection::Cold => serde_json::json!({
            "cold_rebuild_evaluated": report.cold_rebuild_evaluated,
            "cold_rebuild_clean": report.cold_rebuild_clean,
            "cold_rebuild_note": &report.cold_rebuild_note,
            "cold_rebuild_workspace_count": report.cold_rebuild_workspace_count,
            "cold_rebuild_failures": &report.cold_rebuild_failures,
            "cold_rebuilds": &report.cold_rebuilds,
        }),
        ProjectHealthSection::Traceability => serde_json::json!({
            "traceability_evaluated": report.traceability_evaluated,
            "traceability_note": &report.traceability_note,
            "traceability_percent": report.traceability_percent,
            "traceability_blocker_count": report.traceability_blocker_count,
            "traceability_internal_td_count": report.traceability_internal_td_count,
            "traceability_orphan_td_count": report.traceability_orphan_td_count,
            "command_traceability_percent": report.command_traceability_percent,
            "command_traceability_blocker_count": report.command_traceability_blocker_count,
            "command_traceability_hidden_command_count": report.command_traceability_hidden_command_count,
            "command_traceability_orphan_command_count": report.command_traceability_orphan_command_count,
            "traceability": &report.traceability,
        }),
        ProjectHealthSection::Regenerable => serde_json::json!({
            "codegen_percent": report.codegen_percent,
            "codegen_eligible_files": report.codegen_eligible_files,
            "codegen_files": report.codegen_files,
            "cb_ownership": &report.cb_ownership,
            "codegen_origin": &report.codegen_origin,
            "regenerability_authority": &report.regenerability_authority,
            "optional_regenerability_gaps": &report.optional_regenerability_gaps,
        }),
        ProjectHealthSection::Api => serde_json::json!({
            "public_api_covered": report.public_api_covered,
            "public_api_total": report.public_api_total,
            "semantic_review_required": report.semantic_review_required,
        }),
        ProjectHealthSection::Stack => serde_json::json!({
            "stack_migration_percent": report.stack_migration_percent,
            "stack_migration_incomplete_workspaces": report.stack_migration_incomplete_workspaces,
            "stack_migration": &report.stack_migration,
        }),
        ProjectHealthSection::TdLock => project_td_lock_summary(&report.td_lock),
        ProjectHealthSection::Blockers => serde_json::json!({
            "blocker_count": report.blockers.len(),
            "blockers": &report.blockers,
            "production_blocker_count": report.production_blockers.len(),
            "production_blockers": &report.production_blockers,
            "global_blocker_count": report.global_blockers.len(),
            "global_blockers": &report.global_blockers,
            "next_gap": &report.next_gap,
            "blocked_gap_count": report.blocked_gap_count,
            "human_decision_required_count": report.human_decision_required_count,
        }),
    };
    serde_json::json!({
        "schema_version": "aw.cli.v1",
        "event": "result",
        "status": project_health_loop_status(report),
        "action": "health",
        "project": &report.project,
        "section": section,
        "next": project_health_next(report),
        "data": payload,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub fn project_health_section_summary_with_payload_path(
    report: &ProjectHealthReport,
    section: ProjectHealthSection,
    payload_path: &str,
) -> serde_json::Value {
    with_payload_path(
        project_health_section_summary(report, section),
        payload_path,
    )
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub fn project_health_summary(report: &ProjectHealthReport) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "aw.cli.v1",
        "event": "result",
        "status": project_health_loop_status(report),
        "action": "health",
        "project": &report.project,
        "completion": project_health_completion(report),
        "next": project_health_next(report),
        "readiness": project_health_readiness_summary(report),
        "axes": project_health_axes_summary(report),
        "blockers": project_health_compact_blockers(report),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub fn project_health_summary_with_payload_path(
    report: &ProjectHealthReport,
    payload_path: &str,
) -> serde_json::Value {
    with_payload_path(project_health_summary(report), payload_path)
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn with_payload_path(mut summary: serde_json::Value, payload_path: &str) -> serde_json::Value {
    if let serde_json::Value::Object(map) = &mut summary {
        map.insert(
            "payload_path".to_string(),
            serde_json::Value::String(payload_path.to_string()),
        );
    }
    summary
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_compact_completion(report: &ProjectHealthReport) -> serde_json::Value {
    let missing = project_health_missing(report);
    serde_json::json!({
        "root_complete": report.production_ready,
        "workflow_complete": report.production_ready,
        "requires_hitl": project_health_requires_hitl(report),
        "missing_count": missing.len(),
        "missing_preview": preview_strings_limited(&missing, HEALTH_COMPACT_PREVIEW_LIMIT),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_compact_readiness(report: &ProjectHealthReport) -> serde_json::Value {
    serde_json::json!({
        "production_ready": report.production_ready,
        "production_status": &report.production_status,
        "takeover_ready": report.takeover_ready,
        "generator_request_ready": report.generator_request_ready,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_axes_summary(report: &ProjectHealthReport) -> serde_json::Value {
    serde_json::json!({
        "capability": project_health_capability_axis(report),
        "ec": project_health_ec_axis(report),
        "ec_gen": project_health_ec_gen_axis(report),
        "td": project_health_td_axis(report),
        "td_gen": project_health_td_gen_axis(report),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_capability_axis(report: &ProjectHealthReport) -> serde_json::Value {
    let blocker_count = report.capability.blocker_count + report.claim_closure.blocker_count;
    serde_json::json!({
        "status": if blocker_count == 0 && report.capability.production_percent >= 100.0 {
            "passed"
        } else {
            "blocked"
        },
        "production_percent": report.capability.production_percent,
        "claim_closure_percent": report.claim_closure.claim_closure_percent,
        "blocker_count": blocker_count,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_ec_axis(report: &ProjectHealthReport) -> serde_json::Value {
    serde_json::json!({
        "status": &report.ec.status,
        "verified": report.ec.verify_evaluated,
        "passed_commands": report.ec.passed_count,
        "command_count": report.ec.command_count,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_ec_gen_axis(report: &ProjectHealthReport) -> serde_json::Value {
    let generated_units = report.ec.case_count + report.ec.tool_manifest_count;
    let expected_units = report.ec.expected_case_count + report.ec.expected_tool_manifest_count;
    let missing_units = expected_units.saturating_sub(generated_units);
    let status = if !report.ec.evaluated {
        "not_evaluated"
    } else if expected_units == 0 {
        "not_configured"
    } else if report.ec.check_clean
        && report.ec.case_count == report.ec.expected_case_count
        && report.ec.tool_manifest_count == report.ec.expected_tool_manifest_count
    {
        "passed"
    } else {
        "blocked"
    };
    serde_json::json!({
        "status": status,
        "document_kind": "ec",
        "generated_units": generated_units,
        "expected_units": expected_units,
        "generated_percent": percent_of(generated_units, expected_units),
        "handwrite_units": 0,
        "missing_units": missing_units,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_td_axis(report: &ProjectHealthReport) -> serde_json::Value {
    if project_health_caps_ec_only(&report.project) {
        return serde_json::json!({
            "status": "advisory",
            "managed_percent": report.managed_percent,
            "semantic_percent": report.semantic_percent,
            "traceability_percent": report.traceability_percent,
            "td_lock_clean": report.td_lock.clean,
        });
    }
    let passed = report.managed_ready
        && report.semantic_ready
        && report.traceability_ready
        && report.td_lock.clean;
    serde_json::json!({
        "status": if passed { "passed" } else { "blocked" },
        "managed_percent": report.managed_percent,
        "semantic_percent": report.semantic_percent,
        "traceability_percent": report.traceability_percent,
        "td_lock_clean": report.td_lock.clean,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_td_gen_axis(report: &ProjectHealthReport) -> serde_json::Value {
    let generated_units = report.codegen_files;
    let expected_units = report.codegen_eligible_files;
    let required_for_production = report.regenerability_authority.required_for_production
        && !project_health_caps_ec_only(&report.project);
    let status = if project_health_caps_ec_only(&report.project) {
        "advisory"
    } else if required_for_production && report.regenerability_authority.gap_count > 0 {
        "blocked"
    } else if report.codegen_percent >= 100.0 {
        "passed"
    } else {
        "partial"
    };
    serde_json::json!({
        "status": status,
        "document_kind": "td",
        "generated_units": generated_units,
        "expected_units": expected_units,
        "generated_percent": percent_of(generated_units, expected_units),
        "handwrite_units": report.cb_ownership.handwrite_files,
        "missing_units": report.cb_ownership.unmarked_files,
        "required_for_production": required_for_production,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_compact_blockers(report: &ProjectHealthReport) -> serde_json::Value {
    let next_gap = if project_health_caps_ec_only(&report.project) {
        None
    } else {
        report.next_gap.as_ref()
    };
    let blocked_gap_count = if project_health_caps_ec_only(&report.project) {
        0
    } else {
        report.blocked_gap_count
    };
    let human_decision_required_count = if project_health_caps_ec_only(&report.project) {
        0
    } else {
        report.human_decision_required_count
    };
    serde_json::json!({
        "blocker_count": report.blockers.len(),
        "blockers_preview": preview_strings_limited(&report.blockers, HEALTH_COMPACT_PREVIEW_LIMIT),
        "production_blocker_count": report.production_blockers.len(),
        "production_blockers_preview": preview_strings_limited(&report.production_blockers, HEALTH_COMPACT_PREVIEW_LIMIT),
        "global_blocker_count": report.global_blockers.len(),
        "global_blockers_preview": preview_strings_limited(&report.global_blockers, HEALTH_COMPACT_PREVIEW_LIMIT),
        "next_gap": next_gap,
        "blocked_gap_count": blocked_gap_count,
        "human_decision_required_count": human_decision_required_count,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_capability_summary(report: &CapabilityHealthReport) -> serde_json::Value {
    serde_json::json!({
        "evaluated": report.evaluated,
        "production_evaluated": report.production_evaluated,
        "note": &report.note,
        "cap_path": &report.cap_path,
        "format": &report.format,
        "format_version": report.format_version,
        "capability_count": report.capability_count,
        "release_scope_count": report.release_scope_count,
        "root_runner_ready": report.root_runner_ready,
        "production_ready_count": report.production_ready_count,
        "production_scope_count": report.production_scope_count,
        "production_percent": report.production_percent,
        "blocker_count": report.blocker_count,
        "blockers_preview": preview_strings(&report.blockers),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_test_gate_summary(report: &ProjectTestGateReport) -> serde_json::Value {
    serde_json::json!({
        "evaluated": report.evaluated,
        "status": &report.status,
        "note": &report.note,
        "command_count": report.command_count,
        "passed_count": report.passed_count,
        "failed_count": report.failed_count,
        "skipped_count": report.skipped_count,
        "failed_commands_preview": report
            .commands
            .iter()
            .filter(|command| command.status == ProjectTestCommandStatus::Failed)
            .take(HEALTH_SUMMARY_PREVIEW_LIMIT)
            .map(project_test_command_summary)
            .collect::<Vec<_>>(),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_test_command_summary(command: &ProjectTestCommandReport) -> serde_json::Value {
    serde_json::json!({
        "workspace": &command.workspace,
        "command": &command.command,
        "status": &command.status,
        "exit_code": command.exit_code,
        "duration_ms": command.duration_ms,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_ec_gate_summary(report: &ProjectEcGateReport) -> serde_json::Value {
    serde_json::json!({
        "evaluated": report.evaluated,
        "check_clean": report.check_clean,
        "verify_evaluated": report.verify_evaluated,
        "status": &report.status,
        "note": &report.note,
        "inventory_path": &report.inventory_path,
        "expected_case_count": report.expected_case_count,
        "case_count": report.case_count,
        "expected_tool_manifest_count": report.expected_tool_manifest_count,
        "tool_manifest_count": report.tool_manifest_count,
        "command_count": report.command_count,
        "passed_count": report.passed_count,
        "failed_count": report.failed_count,
        "finding_count": report.findings.len(),
        "findings_preview": preview_strings(&report.findings),
        "failed_commands_preview": report
            .commands
            .iter()
            .filter(|command| command.status == ProjectTestCommandStatus::Failed)
            .take(HEALTH_SUMMARY_PREVIEW_LIMIT)
            .map(project_ec_command_summary)
            .collect::<Vec<_>>(),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_ec_command_summary(command: &ProjectEcCommandReport) -> serde_json::Value {
    serde_json::json!({
        "case_id": &command.case_id,
        "command": &command.command,
        "status": &command.status,
        "exit_code": command.exit_code,
        "duration_ms": command.duration_ms,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_claim_closure_summary(report: &ProjectClaimClosureReport) -> serde_json::Value {
    serde_json::json!({
        "evaluated": report.evaluated,
        "note": &report.note,
        "claim_total": report.claim_total,
        "closed_claim_count": report.closed_claim_count,
        "claim_closure_percent": report.claim_closure_percent,
        "claims_with_ec": report.claims_with_ec,
        "claims_with_passing_ec": report.claims_with_passing_ec,
        "claims_with_primary_td": report.claims_with_primary_td,
        "claims_with_artifact_evidence": report.claims_with_artifact_evidence,
        "blocker_count": report.blocker_count,
        "blockers_preview": preview_strings(&report.blockers),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_claim_closure_detail(report: &ProjectClaimClosureReport) -> serde_json::Value {
    serde_json::json!({
        "summary": project_claim_closure_summary(report),
        "claims": &report.claims,
        "blockers": &report.blockers,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_td_lock_summary(lock: &crate::cli::td_lock::TdLockStatus) -> serde_json::Value {
    serde_json::json!({
        "project": &lock.project,
        "td_path": &lock.td_path,
        "lock_path": &lock.lock_path,
        "status": &lock.status,
        "clean": lock.clean,
        "file_count": lock.file_count,
        "changed_count": lock.changed.len(),
        "changed_preview": preview_strings(&lock.changed),
        "added_count": lock.added.len(),
        "added_preview": preview_strings(&lock.added),
        "removed_count": lock.removed.len(),
        "removed_preview": preview_strings(&lock.removed),
        "message": &lock.message,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn preview_strings(values: &[String]) -> Vec<&str> {
    preview_strings_limited(values, HEALTH_SUMMARY_PREVIEW_LIMIT)
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn preview_strings_limited(values: &[String], limit: usize) -> Vec<&str> {
    values.iter().take(limit).map(String::as_str).collect()
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_loop_status(report: &ProjectHealthReport) -> &'static str {
    if report.production_ready {
        "done"
    } else if project_health_requires_hitl(report) || project_health_next_command(report).is_none()
    {
        "blocked"
    } else {
        "continue"
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_completion(report: &ProjectHealthReport) -> serde_json::Value {
    let criteria = if project_health_caps_ec_only(&report.project) {
        vec![
            "capability roots are defined and runtime verified",
            "capability claims map to production EC cases",
            "EC inventory/check is clean",
            "EC commands pass when verification is requested",
        ]
    } else {
        vec![
            "capability roots are defined and runtime verified",
            "managed, semantic, and traceability takeover gates are ready",
            "CB/cold/test/EC production gates are evaluated and clean",
            "capability claims have EC, TD, and artifact closure",
            "no workflow locks or artifact quality blockers remain",
        ]
    };
    serde_json::json!({
        "root_complete": report.production_ready,
        "workflow_complete": report.production_ready,
        "requires_hitl": project_health_requires_hitl(report),
        "criteria": criteria,
        "missing": project_health_missing(report),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_requires_hitl(report: &ProjectHealthReport) -> bool {
    if project_health_caps_ec_only(&report.project) {
        return false;
    }
    report.workflow_lock_count > 0 || report.human_decision_required_count > 0
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_missing(report: &ProjectHealthReport) -> Vec<String> {
    if report.production_ready {
        return Vec::new();
    }
    let mut missing = Vec::new();
    let mut seen = BTreeSet::new();
    for value in project_health_missing_evaluations(report) {
        push_project_health_missing(&mut missing, &mut seen, value);
    }
    for blocker in &report.blockers {
        push_project_health_missing(&mut missing, &mut seen, blocker.clone());
    }
    for blocker in &report.production_blockers {
        push_project_health_missing(&mut missing, &mut seen, blocker.clone());
    }
    if !project_health_caps_ec_only(&report.project) {
        if let Some(gap) = &report.next_gap {
            push_project_health_missing(&mut missing, &mut seen, format!("next gap: {gap}"));
        }
    }
    missing
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn push_project_health_missing(
    missing: &mut Vec<String>,
    seen: &mut BTreeSet<String>,
    value: String,
) {
    if seen.insert(value.clone()) {
        missing.push(value);
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_missing_evaluations(report: &ProjectHealthReport) -> Vec<String> {
    let mut missing = Vec::new();
    if !project_health_caps_ec_only(&report.project) {
        if !report.traceability_evaluated {
            missing.push(report.traceability_note.clone().unwrap_or_else(|| {
                format!(
                    "traceability not evaluated; run `aw health --project {} full`",
                    report.project
                )
            }));
        }
        if !report.cb_verify_evaluated {
            missing.push(report.cb_verify_note.clone().unwrap_or_else(|| {
                format!(
                    "cb verify not evaluated; run `aw health --project {} full`",
                    report.project
                )
            }));
        }
        if !report.cold_rebuild_evaluated
            && (report.cold_rebuild_workspace_count > 0 || report.cold_rebuild_note.is_some())
        {
            missing.push(report.cold_rebuild_note.clone().unwrap_or_else(|| {
                format!(
                    "cold rebuild not evaluated; run `aw health --project {} full`",
                    report.project
                )
            }));
        }
        if report.test_gates.status == ProjectTestGateStatus::NotEvaluated {
            missing.push(report.test_gates.note.clone().unwrap_or_else(|| {
                format!(
                    "test gates not evaluated; run `aw health --project {} full`",
                    report.project
                )
            }));
        }
    }
    if matches!(report.ec.status, ProjectEcGateStatus::NotVerified) {
        missing.push(report.ec.note.clone().unwrap_or_else(|| {
            format!(
                "EC commands not evaluated; run `aw health --project {} --verify-ec`",
                report.project
            )
        }));
    }
    if !report.claim_closure.evaluated {
        missing.push(report.claim_closure.note.clone().unwrap_or_else(|| {
            format!(
                "claim closure not evaluated; run `aw health --project {} claims`",
                report.project
            )
        }));
    }
    missing
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_next(report: &ProjectHealthReport) -> serde_json::Value {
    let command = project_health_next_command(report);
    let mut next = serde_json::Map::new();
    next.insert(
        "kind".to_string(),
        serde_json::Value::String(project_health_next_kind(report, command.is_some()).to_string()),
    );
    if let Some(command) = command {
        next.insert("command".to_string(), serde_json::Value::String(command));
    }
    next.insert(
        "reason".to_string(),
        serde_json::Value::String(project_health_next_reason(report)),
    );
    serde_json::Value::Object(next)
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_next_kind(report: &ProjectHealthReport, has_command: bool) -> &'static str {
    if report.production_ready {
        "done"
    } else if project_health_requires_hitl(report) {
        "hitl"
    } else if has_command {
        "run_command"
    } else {
        "blocked"
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_next_command(report: &ProjectHealthReport) -> Option<String> {
    let caps_ec_only = project_health_caps_ec_only(&report.project);
    if report.production_ready || (!caps_ec_only && report.workflow_lock_count > 0) {
        return None;
    }
    if !caps_ec_only && !report.td_lock.clean {
        return Some(
            if report.td_lock.status == crate::cli::td_lock::TdLockState::Missing {
                format!("aw td lock --project {}", report.project)
            } else {
                format!("aw td lock --project {} --show", report.project)
            },
        );
    }
    if !report.ec.check_clean {
        return Some(format!("aw ec gen --project {} --verify", report.project));
    }
    if matches!(report.ec.status, ProjectEcGateStatus::NotConfigured)
        && (report.ec.expected_case_count > 0 || report.ec.expected_tool_manifest_count > 0)
    {
        return Some(format!("aw ec gen --project {} --verify", report.project));
    }
    if matches!(report.ec.status, ProjectEcGateStatus::Failed) {
        return Some(format!(
            "aw health --project {} --verify-ec",
            report.project
        ));
    }
    if matches!(report.ec.status, ProjectEcGateStatus::NotVerified) {
        return Some(format!(
            "aw health --project {} --verify-ec",
            report.project
        ));
    }
    if report.claim_closure.blocker_count > 0 {
        return Some(format!("aw health --project {} claims", report.project));
    }
    if !report.capability_ready {
        if matches!(
            report.capability.format.as_str(),
            "missing" | "unparseable" | "unresolved"
        ) {
            return None;
        }
        return Some(format!(
            "aw capability run --project {} --non-interactive --max-ticks 1",
            report.project
        ));
    }
    if caps_ec_only {
        return report
            .blockers
            .first()
            .map(|_| format!("aw health --project {} claims", report.project));
    }
    if !report.managed_ready {
        return Some(format!(
            "aw standardize managed run --project {} --non-interactive --max-ticks 1",
            report.project
        ));
    }
    if !report.semantic_ready
        || report.stack_migration_incomplete_workspaces > 0
        || report.blocked_gap_count > 0
        || report.human_decision_required_count > 0
    {
        return Some(format!(
            "aw standardize semantic run --project {} --non-interactive --max-ticks 1",
            report.project
        ));
    }
    if !report.traceability_ready {
        return Some(format!(
            "aw standardize traceability run --project {} --non-interactive --max-ticks 1",
            report.project
        ));
    }
    if !project_health_missing_evaluations(report).is_empty() {
        return Some(format!("aw health --project {} full", report.project));
    }
    Some(format!("aw run --project {} --max-ticks 1", report.project))
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_next_reason(report: &ProjectHealthReport) -> String {
    let caps_ec_only = project_health_caps_ec_only(&report.project);
    if report.production_ready {
        return "project production readiness is complete".to_string();
    }
    if !caps_ec_only && report.workflow_lock_count > 0 {
        return report
            .blockers
            .iter()
            .find(|blocker| blocker.starts_with("workflow lock:"))
            .cloned()
            .unwrap_or_else(|| {
                "workflow lock requires current owner or HITL resolution".to_string()
            });
    }
    if !caps_ec_only && !report.td_lock.clean {
        return report.td_lock.message.clone();
    }
    if !report.ec.check_clean {
        return report
            .ec
            .findings
            .first()
            .cloned()
            .unwrap_or_else(|| "EC inventory/check is blocked".to_string());
    }
    if matches!(report.ec.status, ProjectEcGateStatus::NotConfigured)
        && (report.ec.expected_case_count > 0 || report.ec.expected_tool_manifest_count > 0)
    {
        return report
            .ec
            .note
            .clone()
            .unwrap_or_else(|| "EC inventory has no cases".to_string());
    }
    if matches!(report.ec.status, ProjectEcGateStatus::Failed) {
        return "external contract gate commands failed".to_string();
    }
    if matches!(report.ec.status, ProjectEcGateStatus::NotVerified) {
        return "external contract gate commands are not verified".to_string();
    }
    if report.claim_closure.blocker_count > 0 {
        return report
            .claim_closure
            .blockers
            .first()
            .cloned()
            .unwrap_or_else(|| "capability claim closure is incomplete".to_string());
    }
    if !report.capability_ready {
        if matches!(
            report.capability.format.as_str(),
            "missing" | "unparseable" | "unresolved"
        ) {
            return report
                .capability
                .blockers
                .first()
                .cloned()
                .unwrap_or_else(|| "capability roots must be defined in cap_path".to_string());
        }
        return "capability roots are incomplete; advance the capability workflow".to_string();
    }
    if caps_ec_only {
        return report.blockers.first().cloned().unwrap_or_else(|| {
            "Agentic Workflow self-health is governed by capability and EC gates".to_string()
        });
    }
    if !report.managed_ready {
        return "source ownership is incomplete; advance managed takeover".to_string();
    }
    if !report.semantic_ready
        || report.stack_migration_incomplete_workspaces > 0
        || report.blocked_gap_count > 0
        || report.human_decision_required_count > 0
    {
        return "semantic coverage or stack migration is incomplete; advance semantic takeover"
            .to_string();
    }
    if !report.traceability_ready {
        return "TD/source/command traceability is incomplete; advance traceability closure"
            .to_string();
    }
    let missing_evaluations = project_health_missing_evaluations(report);
    if !missing_evaluations.is_empty() {
        return format!(
            "production readiness needs full health verification: {}",
            missing_evaluations.join("; ")
        );
    }
    report.blockers.first().cloned().unwrap_or_else(|| {
        "project production readiness is blocked; return to project root".to_string()
    })
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn project_health_readiness_summary(report: &ProjectHealthReport) -> serde_json::Value {
    serde_json::json!({
        "production_ready": report.production_ready,
        "production_status": &report.production_status,
        "takeover_ready": report.takeover_ready,
        "generator_request_ready": report.generator_request_ready,
        "blocker_count": report.blockers.len(),
        "production_blocker_count": report.production_blockers.len(),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub async fn run_health(args: ProjectHealthArgs) -> Result<()> {
    let verification = effective_health_verification_flags(&args);
    let mut report = build_health_report_with_options_internal(
        &args.project,
        verification.traceability,
        verification.cb,
        verification.cold,
        verification.tests,
        verification.ec,
        args.verbose,
    )?;
    apply_td_lock_to_report(&mut report)?;
    apply_workflow_locks_to_report(&mut report).await?;
    let payload_path = write_health_payload(&report)?;
    if args.human {
        match args.section {
            Some(ProjectHealthSection::Full) => print_health_report(&report),
            Some(section) => print_health_section(&report, section),
            None => print_health_compact_report(&report),
        }
    } else if let Some(section) = args.section {
        let summary =
            project_health_section_summary_with_payload_path(&report, section, &payload_path);
        if args.pretty || args.json {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        } else {
            println!("{}", serde_json::to_string(&summary)?);
        }
    } else if args.pretty || args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&project_health_compact_summary_with_payload_path(
                &report,
                &payload_path,
            ))?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(&project_health_compact_summary_with_payload_path(
                &report,
                &payload_path,
            ))?
        );
    }
    if report.status == ProjectHealthStatus::Blocked {
        std::process::exit(1);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
struct HealthVerificationFlags {
    traceability: bool,
    cb: bool,
    cold: bool,
    tests: bool,
    ec: bool,
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn effective_health_verification_flags(args: &ProjectHealthArgs) -> HealthVerificationFlags {
    let targeted = args.verify_traceability
        || args.verify_cb
        || args.verify_cold
        || args.verify_tests
        || args.verify_ec;
    if targeted {
        HealthVerificationFlags {
            traceability: args.verify_traceability,
            cb: args.verify_cb,
            cold: args.verify_cold,
            tests: args.verify_tests,
            ec: args.verify_ec,
        }
    } else if let Some(section) = args.section {
        match section {
            ProjectHealthSection::Full => HealthVerificationFlags::all(),
            ProjectHealthSection::Tests => HealthVerificationFlags::tests(),
            ProjectHealthSection::Ec => HealthVerificationFlags::ec(),
            ProjectHealthSection::Cb | ProjectHealthSection::Api => HealthVerificationFlags::cb(),
            ProjectHealthSection::Cold => HealthVerificationFlags::cold(),
            ProjectHealthSection::Claims => HealthVerificationFlags {
                traceability: true,
                cb: false,
                cold: false,
                tests: false,
                ec: true,
            },
            ProjectHealthSection::Traceability
            | ProjectHealthSection::Regenerable
            | ProjectHealthSection::Stack => HealthVerificationFlags::traceability(),
            ProjectHealthSection::Metrics
            | ProjectHealthSection::Capability
            | ProjectHealthSection::Gates
            | ProjectHealthSection::TdLock
            | ProjectHealthSection::Blockers => HealthVerificationFlags::none(),
        }
    } else {
        HealthVerificationFlags::none()
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
impl HealthVerificationFlags {
    fn all() -> Self {
        Self {
            traceability: true,
            cb: true,
            cold: true,
            tests: true,
            ec: true,
        }
    }

    fn none() -> Self {
        Self {
            traceability: false,
            cb: false,
            cold: false,
            tests: false,
            ec: false,
        }
    }

    fn tests() -> Self {
        Self {
            tests: true,
            ..Self::none()
        }
    }

    fn ec() -> Self {
        Self {
            ec: true,
            ..Self::none()
        }
    }

    fn cb() -> Self {
        Self {
            cb: true,
            ..Self::none()
        }
    }

    fn cold() -> Self {
        Self {
            cold: true,
            ..Self::none()
        }
    }

    fn traceability() -> Self {
        Self {
            traceability: true,
            ..Self::none()
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn write_health_payload(report: &ProjectHealthReport) -> Result<String> {
    let dir = std::env::temp_dir()
        .join("aw")
        .join(sanitize_tmp_path_segment(&report.project))
        .join("health");
    fs::create_dir_all(&dir)
        .with_context(|| format!("create health payload dir {}", dir.display()))?;
    let path = dir.join("report.json");
    let bytes = serde_json::to_vec_pretty(report)?;
    fs::write(&path, bytes).with_context(|| format!("write health payload {}", path.display()))?;
    Ok(path.to_string_lossy().to_string())
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn sanitize_tmp_path_segment(value: &str) -> String {
    let mut segment = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            segment.push(ch);
        } else {
            segment.push('_');
        }
    }
    if segment.is_empty() {
        "project".to_string()
    } else {
        segment
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub(crate) fn apply_td_lock_to_report(report: &mut ProjectHealthReport) -> Result<()> {
    let status = crate::cli::td_lock::check_project_td_lock(&report.project)?;
    if !status.clean && !project_health_caps_ec_only(&report.project) {
        report.status = ProjectHealthStatus::Blocked;
        report
            .blockers
            .push(format!("td lock: {}", status.message.clone()));
        report.blockers.sort();
        report.blockers.dedup();
        report.production_ready = false;
        report.production_status = ProductionStatus::Blocked;
    }
    report.td_lock = status;
    report.refresh_takeover_readiness();
    Ok(())
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub(crate) fn apply_ec_to_report(report: &mut ProjectHealthReport, verify_ec: bool) -> Result<()> {
    let summary = crate::cli::ec::project_ec_check_summary(&report.project)?;
    let mut ec_report = ProjectEcGateReport::from_check(summary);

    if !ec_report.check_clean {
        block_health_report(
            report,
            format!("ec check: {} finding(s)", ec_report.findings.len()),
        );
        for finding in &ec_report.findings {
            block_health_report(report, format!("ec check: {finding}"));
        }
        report.ec = ec_report;
        report.refresh_takeover_readiness();
        return Ok(());
    }

    if verify_ec {
        if ec_report.case_count == 0 && ec_report.tool_manifest_count == 0 {
            let finding = "EC inventory has no cases; add external-contract e2e-test sections and run `aw ec gen --project <project>`"
                .to_string();
            ec_report.findings.push(finding.clone());
            ec_report.status = ProjectEcGateStatus::NotConfigured;
            ec_report.note = Some(finding.clone());
            block_health_report(report, format!("ec verify: {finding}"));
        } else {
            let Some((_inventory_path, manifest)) =
                crate::cli::ec::load_project_ec_manifest(&report.project)?
            else {
                let finding = format!(
                    "EC inventory not generated in aw.toml; run `aw ec gen --project {} --verify`",
                    report.project
                );
                ec_report.findings.push(finding.clone());
                ec_report.status = ProjectEcGateStatus::CheckFailed;
                ec_report.note = Some(finding.clone());
                block_health_report(report, format!("ec check: {finding}"));
                report.ec = ec_report;
                report.refresh_takeover_readiness();
                return Ok(());
            };
            let project_root = crate::find_project_root()?;
            // aw.toml case commands are authoritative for generated EC
            // inventory. Legacy category bindings remain as fallback for old
            // inventories that have not materialized per-case commands yet.
            let project_model = crate::services::project_registry::load_projects(&project_root)?
                .into_iter()
                .find(|project| project.name == report.project);
            let mut commands = Vec::new();
            let mut seen_commands = BTreeSet::new();
            for case in manifest
                .cases
                .iter()
                .filter(|case| case.required_for_production)
            {
                let started = Instant::now();
                let command = match resolve_project_ec_command(case, project_model.as_ref()) {
                    Ok(command) => command,
                    Err(err) => {
                        commands.push(project_ec_resolution_error_report(case, err, started));
                        continue;
                    }
                };
                if !command.trim().is_empty() && !seen_commands.insert(command.trim().to_string()) {
                    continue;
                }
                commands.push(run_project_ec_shell_command(
                    case.id.clone(),
                    command,
                    &project_root,
                    started,
                )?);
            }
            for tool in &manifest.tool_manifests {
                if !tool.command.trim().is_empty()
                    && !seen_commands.insert(tool.command.trim().to_string())
                {
                    continue;
                }
                commands.push(run_project_ec_tool_manifest_command(tool, &project_root)?);
            }
            let passed_count = commands
                .iter()
                .filter(|command| command.status == ProjectTestCommandStatus::Passed)
                .count();
            let failed_count = commands.len() - passed_count;
            ec_report.verify_evaluated = true;
            ec_report.command_count = commands.len();
            ec_report.passed_count = passed_count;
            ec_report.failed_count = failed_count;
            ec_report.commands = commands;
            if failed_count == 0 {
                ec_report.status = ProjectEcGateStatus::Passed;
                ec_report.note = None;
            } else {
                ec_report.status = ProjectEcGateStatus::Failed;
                ec_report.note = Some(format!(
                    "EC commands failed: {}/{} command(s)",
                    failed_count, ec_report.command_count
                ));
                block_health_report(
                    report,
                    format!(
                        "ec verify failed: {}/{} command(s)",
                        failed_count, ec_report.command_count
                    ),
                );
                for command in ec_report
                    .commands
                    .iter()
                    .filter(|command| command.status == ProjectTestCommandStatus::Failed)
                {
                    let failed_command = if command.command.trim().is_empty() {
                        command.case_id.as_str()
                    } else {
                        command.command.as_str()
                    };
                    block_health_report(
                        report,
                        format!(
                            "ec `{}` failed with exit {:?}",
                            failed_command, command.exit_code
                        ),
                    );
                }
            }
        }
    }

    report.ec = ec_report;
    report.refresh_takeover_readiness();
    Ok(())
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub(crate) fn apply_claim_closure_to_report(report: &mut ProjectHealthReport) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let closure = match build_project_claim_closure_report(&project_root, report) {
        Ok(closure) => closure,
        Err(err) => ProjectClaimClosureReport::from_blocker(
            &report.project,
            format!("claim closure unavailable: {err}"),
        ),
    };
    for blocker in &closure.blockers {
        if !report.blockers.contains(blocker) {
            block_health_report(report, blocker.clone());
        }
    }
    report.claim_closure = closure;
    report.refresh_takeover_readiness();
    Ok(())
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn build_project_claim_closure_report(
    project_root: &std::path::Path,
    report: &ProjectHealthReport,
) -> Result<ProjectClaimClosureReport> {
    let cap_path =
        crate::cli::capability::resolve_capability_path(project_root, &report.project, None)?;
    let cap_body = std::fs::read_to_string(&cap_path)
        .with_context(|| format!("failed to read capability map {}", cap_path.display()))?;
    let document = crate::cli::capability::parse_capability_document(&cap_body, &cap_path)
        .with_context(|| format!("failed to parse capability map from {}", cap_path.display()))?;
    let td_refs = crate::cli::capability::collect_td_capability_refs(
        project_root,
        &report.project,
        &document,
    )
    .with_context(|| "failed to scan TD capability_refs")?;
    let manifest =
        crate::cli::ec::load_project_ec_manifest(&report.project)?.map(|(_, manifest)| manifest);
    let caps_ec_only = project_health_caps_ec_only(&report.project);
    Ok(build_claim_closure_report(
        &report.project,
        &document,
        &td_refs,
        manifest.as_ref(),
        &report.ec,
        caps_ec_only
            || (report.managed_ready && report.semantic_ready && report.traceability_ready),
        !caps_ec_only,
    ))
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn build_claim_closure_report(
    _project: &str,
    document: &crate::cli::capability::CapabilityDocument,
    td_refs: &[crate::cli::capability::TdCapabilityEvidence],
    manifest: Option<&crate::cli::ec::EcManifest>,
    ec_report: &ProjectEcGateReport,
    artifact_evidence_ready: bool,
    require_td_artifact_evidence: bool,
) -> ProjectClaimClosureReport {
    let ec_cases = manifest
        .map(|manifest| manifest.cases.as_slice())
        .unwrap_or(&[]);
    let capability_ids = document.capability_ids();
    let mut global_blockers = Vec::new();

    for case in ec_cases.iter().filter(|case| case.required_for_production) {
        if case.capability_id.trim().is_empty() || case.capability_id == "unmapped" {
            push_unique_string(
                &mut global_blockers,
                format!(
                    "claim closure EC case `{}` is unmapped; production cases must name capability_id and claim_id",
                    case.id
                ),
            );
            continue;
        }
        if !capability_ids.contains(&case.capability_id) {
            push_unique_string(
                &mut global_blockers,
                format!(
                    "claim closure EC case `{}` references unknown capability `{}`",
                    case.id, case.capability_id
                ),
            );
            continue;
        }
        if case.claim_id.trim().is_empty()
            || !document
                .claim_ids_for(&case.capability_id)
                .contains(&case.claim_id)
        {
            push_unique_string(
                &mut global_blockers,
                format!(
                    "claim closure EC case `{}` references unknown claim `{}` for capability `{}`",
                    case.id, case.claim_id, case.capability_id
                ),
            );
        }
    }

    let passed_commands = ec_report
        .commands
        .iter()
        .filter(|command| command.status == ProjectTestCommandStatus::Passed)
        .map(|command| command.command.trim())
        .filter(|command| !command.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let mut passed_ec_case_ids = ec_report
        .commands
        .iter()
        .filter(|command| command.status == ProjectTestCommandStatus::Passed)
        .map(|command| command.case_id.clone())
        .collect::<BTreeSet<_>>();
    for case in ec_cases.iter().filter(|case| case.required_for_production) {
        let command = case.command.trim();
        if !command.is_empty() && passed_commands.contains(command) {
            passed_ec_case_ids.insert(case.id.clone());
        }
    }
    let mut claims = Vec::new();

    for capability in &document.capabilities {
        if capability.status == crate::cli::capability::CapabilityStatus::Retired {
            continue;
        }
        if capability.status != crate::cli::capability::CapabilityStatus::Verified {
            continue;
        }
        let Some(contract) = capability.verification_contract.as_ref() else {
            continue;
        };
        for claim in contract
            .claims
            .iter()
            .filter(|claim| claim.required_for_verified)
        {
            let ec_case_ids = ec_cases
                .iter()
                .filter(|case| {
                    case.required_for_production
                        && case.capability_id == capability.id
                        && case.claim_id == claim.id
                })
                .map(|case| case.id.clone())
                .collect::<Vec<_>>();
            let passing_ec_case_ids = if ec_report.verify_evaluated {
                ec_case_ids
                    .iter()
                    .filter(|case_id| passed_ec_case_ids.contains(*case_id))
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let primary_td_refs = td_refs
                .iter()
                .filter(|td_ref| {
                    td_ref.capability_id == capability.id
                        && td_ref.claim.as_deref() == Some(claim.id.as_str())
                        && td_ref.role == crate::cli::capability::CapabilityRefRole::Primary
                })
                .map(td_ref_display)
                .collect::<Vec<_>>();
            let artifact_evidence = if require_td_artifact_evidence {
                !primary_td_refs.is_empty() && artifact_evidence_ready
            } else {
                artifact_evidence_ready
            };
            let mut blockers = Vec::new();
            if ec_case_ids.is_empty() {
                blockers.push("missing required production EC case".to_string());
            }
            if !ec_report.verify_evaluated {
                blockers.push("EC verify not evaluated".to_string());
            } else if passing_ec_case_ids.is_empty() {
                blockers.push("no required EC case passed verification".to_string());
            }
            if require_td_artifact_evidence && primary_td_refs.is_empty() {
                blockers.push("missing primary TD capability_ref".to_string());
            }
            if require_td_artifact_evidence && !artifact_evidence {
                blockers.push(
                    "artifact evidence not closed by managed/semantic/traceability health"
                        .to_string(),
                );
            }
            let status = if blockers.is_empty() {
                ProjectClaimClosureStatus::Closed
            } else {
                ProjectClaimClosureStatus::Blocked
            };
            for blocker in &blockers {
                push_unique_string(
                    &mut global_blockers,
                    format!(
                        "claim closure `{}`:`{}`: {}",
                        capability.id, claim.id, blocker
                    ),
                );
            }
            claims.push(ProjectClaimClosureItem {
                capability_id: capability.id.clone(),
                claim_id: claim.id.clone(),
                ec_case_ids,
                passing_ec_case_ids,
                primary_td_refs,
                artifact_evidence,
                status,
                blockers,
            });
        }
    }

    let claim_total = claims.len();
    let closed_claim_count = claims
        .iter()
        .filter(|claim| claim.status == ProjectClaimClosureStatus::Closed)
        .count();
    let claims_with_ec = claims
        .iter()
        .filter(|claim| !claim.ec_case_ids.is_empty())
        .count();
    let claims_with_passing_ec = claims
        .iter()
        .filter(|claim| !claim.passing_ec_case_ids.is_empty())
        .count();
    let claims_with_primary_td = claims
        .iter()
        .filter(|claim| !claim.primary_td_refs.is_empty())
        .count();
    let claims_with_artifact_evidence = claims
        .iter()
        .filter(|claim| claim.artifact_evidence)
        .count();

    ProjectClaimClosureReport {
        evaluated: true,
        note: None,
        claim_total,
        closed_claim_count,
        claim_closure_percent: if claim_total == 0 {
            100.0
        } else {
            percent_of(closed_claim_count, claim_total)
        },
        claims_with_ec,
        claims_with_passing_ec,
        claims_with_primary_td,
        claims_with_artifact_evidence,
        blocker_count: global_blockers.len(),
        blockers: global_blockers,
        claims,
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn td_ref_display(td_ref: &crate::cli::capability::TdCapabilityEvidence) -> String {
    td_ref
        .spec_id
        .as_ref()
        .map(|spec_id| format!("{}#{}", td_ref.spec_path, spec_id))
        .unwrap_or_else(|| td_ref.spec_path.clone())
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn block_health_report(report: &mut ProjectHealthReport, blocker: String) {
    report.status = ProjectHealthStatus::Blocked;
    report.blockers.push(blocker.clone());
    report.production_blockers.push(blocker);
    report.blockers.sort();
    report.blockers.dedup();
    report.production_blockers.sort();
    report.production_blockers.dedup();
    report.production_ready = false;
    report.production_status = ProductionStatus::Blocked;
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
impl EcBinding {
    /// wi-13 R2: deterministic verify command for one EC tool binding. Current
    /// bindings use rig/meter/vat/guard. Arena remains accepted for legacy
    /// compatibility; new capability contracts should not default to it.
    /// A missing argument or an unknown tool is an error the dispatch surfaces
    /// as a Failed EC command, not a health-run abort.
    pub fn command(&self) -> Result<String> {
        if let Some(command) = self
            .command
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Ok(command.to_string());
        }
        match self.tool.as_str() {
            "arena" => {
                let spec = self
                    .spec
                    .as_deref()
                    .context("ec binding `arena` requires `spec`")?;
                Ok(format!("arena run --spec {spec}"))
            }
            "rig" => {
                let dir = self
                    .dir
                    .as_deref()
                    .context("ec binding `rig` requires `dir`")?;
                Ok(format!("rig run --dir {dir}"))
            }
            "meter" => {
                let target = self
                    .meter
                    .as_deref()
                    .context("ec binding `meter` requires `meter`")?;
                Ok(format!("meter run --target {target}"))
            }
            "vat" => {
                let runner = self
                    .dir
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                Ok(match runner {
                    Some(runner) => format!("vat run {runner}"),
                    None => "vat run".to_string(),
                })
            }
            "guard" => {
                let target = self
                    .dir
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or(".");
                Ok(format!("guard scan {target} --compact --no-persist"))
            }
            other => {
                anyhow::bail!(
                    "unknown ec binding tool `{other}` (expected rig|meter|vat|guard, or legacy arena)"
                )
            }
        }
    }
}

/// The command a case actually runs. Generated aw.toml case commands are
/// authoritative; category bindings are fallback for legacy inventories whose
/// cases do not carry a command yet.
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn resolve_project_ec_command(
    case: &crate::cli::ec::EcManifestCase,
    project: Option<&crate::models::project::Project>,
) -> Result<String> {
    if !case.command.trim().is_empty() {
        return Ok(case.command.clone());
    }
    match project.and_then(|project| project_ec_binding_for_category(project, &case.category)) {
        Some(binding) => binding.command(),
        None => Ok(case.command.clone()),
    }
}

fn project_ec_binding_for_category<'a>(
    project: &'a crate::models::project::Project,
    category: &str,
) -> Option<&'a EcBinding> {
    project.ec.get(category).or_else(|| match category {
        "efficiency" => project.ec.get("benchmark"),
        "benchmark" => project.ec.get("efficiency"),
        _ => None,
    })
}

fn project_ec_resolution_error_report(
    case: &crate::cli::ec::EcManifestCase,
    err: anyhow::Error,
    started: Instant,
) -> ProjectEcCommandReport {
    ProjectEcCommandReport {
        case_id: case.id.clone(),
        command: case.command.clone(),
        status: ProjectTestCommandStatus::Failed,
        exit_code: None,
        duration_ms: started.elapsed().as_millis(),
        stdout_tail: String::new(),
        stderr_tail: format!(
            "invalid ec binding for category `{}`: {err:#}",
            case.category
        ),
    }
}

fn run_project_ec_tool_manifest_command(
    tool: &crate::cli::ec::EcToolManifest,
    project_root: &std::path::Path,
) -> Result<ProjectEcCommandReport> {
    let started = Instant::now();
    if tool.command.trim().is_empty() {
        return Ok(ProjectEcCommandReport {
            case_id: format!("tool:{}", tool.id),
            command: String::new(),
            status: ProjectTestCommandStatus::Failed,
            exit_code: None,
            duration_ms: started.elapsed().as_millis(),
            stdout_tail: String::new(),
            stderr_tail: format!("tool-contract `{}` is missing command", tool.id),
        });
    }
    run_project_ec_shell_command(
        format!("tool:{}", tool.id),
        tool.command.clone(),
        project_root,
        started,
    )
}

fn run_project_ec_shell_command(
    case_id: String,
    command: String,
    project_root: &std::path::Path,
    started: Instant,
) -> Result<ProjectEcCommandReport> {
    let command = &command;
    let stdout_file = tempfile::NamedTempFile::new()
        .with_context(|| format!("create stdout capture for EC command `{command}`"))?;
    let stderr_file = tempfile::NamedTempFile::new()
        .with_context(|| format!("create stderr capture for EC command `{command}`"))?;
    let stdout = stdout_file
        .reopen()
        .with_context(|| format!("open stdout capture for EC command `{command}`"))?;
    let stderr = stderr_file
        .reopen()
        .with_context(|| format!("open stderr capture for EC command `{command}`"))?;

    let status = crate::cli::shell_env::protected_shell_command(project_root, command)
        .current_dir(project_root)
        .stdout(stdout)
        .stderr(stderr)
        .status()
        .with_context(|| format!("failed to execute EC command `{command}`"))?;
    let stdout = fs::read(stdout_file.path())
        .with_context(|| format!("read stdout capture for EC command `{command}`"))?;
    let stderr = fs::read(stderr_file.path())
        .with_context(|| format!("read stderr capture for EC command `{command}`"))?;
    Ok(ProjectEcCommandReport {
        case_id,
        command: command.clone(),
        status: if status.success() {
            ProjectTestCommandStatus::Passed
        } else {
            ProjectTestCommandStatus::Failed
        },
        exit_code: status.code(),
        duration_ms: started.elapsed().as_millis(),
        stdout_tail: tail_lossy(&stdout, 4000),
        stderr_tail: tail_lossy(&stderr, 4000),
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
pub(crate) async fn apply_workflow_locks_to_report(report: &mut ProjectHealthReport) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let locks = crate::cli::workflow_guard::issue_locks(&project_root).await?;
    report.workflow_lock_count = locks.len();
    if !locks.is_empty() && !project_health_caps_ec_only(&report.project) {
        report.status = ProjectHealthStatus::Blocked;
        for lock in locks {
            let expected = if lock.expected_command.is_empty() {
                "no expected command recorded".to_string()
            } else {
                format!("expects `{}`", lock.expected_command)
            };
            let blocker = lock
                .blocker_summary
                .map(|b| format!("; blocker: {b}"))
                .unwrap_or_default();
            report.blockers.push(format!(
                "workflow lock: {} owned by {} {}{}",
                lock.issue_id, lock.owner, expected, blocker
            ));
        }
        report.production_ready = false;
        report.production_status = ProductionStatus::Blocked;
    }
    report.refresh_takeover_readiness();
    Ok(())
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn print_health_compact_report(report: &ProjectHealthReport) {
    println!("project health: {} ({:?})", report.project, report.status);
    println!(
        "ready: production={}, takeover={}, generator={}",
        report.production_ready, report.takeover_ready, report.generator_request_ready
    );
    println!(
        "layers: capability={}, managed={} ({:.1}%), semantic={} ({:.1}%), traceability={} ({:.1}%)",
        report.capability_ready,
        report.managed_ready,
        report.managed_percent,
        report.semantic_ready,
        report.semantic_percent,
        report.traceability_ready,
        report.traceability_percent
    );
    println!(
        "gates: tests={:?}, ec={:?}, cb={}, cold={}, td_lock={:?}/{}",
        report.test_gates.status,
        report.ec.status,
        if report.cb_verify_clean {
            "clean"
        } else {
            "blocked"
        },
        if report.cold_rebuild_clean {
            "clean"
        } else {
            "blocked"
        },
        report.td_lock.status,
        if report.td_lock.clean {
            "clean"
        } else {
            "blocked"
        }
    );
    println!(
        "blockers: total={}, production={}, global={}",
        report.blockers.len(),
        report.production_blockers.len(),
        report.global_blockers.len()
    );
    for blocker in report.blockers.iter().take(HEALTH_COMPACT_PREVIEW_LIMIT) {
        println!("  - {blocker}");
    }
    let next = project_health_next(report);
    if let Some(command) = next.get("command").and_then(|value| value.as_str()) {
        println!("next: {command}");
    } else if let Some(reason) = next.get("reason").and_then(|value| value.as_str()) {
        println!("next: {reason}");
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn print_health_section(report: &ProjectHealthReport, section: ProjectHealthSection) {
    match serde_json::to_string_pretty(&project_health_section_summary(report, section)) {
        Ok(value) => println!("{value}"),
        Err(err) => println!("failed to render health section {section:?}: {err}"),
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn print_health_report(report: &ProjectHealthReport) {
    println!("project health: {} ({:?})", report.project, report.status);
    println!(
        "production_ready: {}",
        if report.production_ready { "yes" } else { "no" }
    );
    println!(
        "takeover_ready: {}",
        if report.takeover_ready { "yes" } else { "no" }
    );
    println!(
        "generator_request_ready: {}",
        if report.generator_request_ready {
            "yes"
        } else {
            "no"
        }
    );
    println!("production_status: {:?}", report.production_status);
    println!(
        "td_lock: {:?}, clean={}, files={}",
        report.td_lock.status, report.td_lock.clean, report.td_lock.file_count
    );
    if !report.td_lock.clean {
        println!("  blocker: {}", report.td_lock.message);
    }
    if !report.production_scope.is_empty() {
        println!("production_scope: {}", report.production_scope.join(", "));
    }
    println!(
        "capability: {} root(s), {} release-scope, format {}, root_runner_ready={}, production {}/{} ({:.1}%)",
        report.capability.capability_count,
        report.capability.release_scope_count,
        report.capability.format,
        report.capability.root_runner_ready,
        report.capability.production_ready_count,
        report.capability.production_scope_count,
        report.capability.production_percent
    );
    if !report.capability.production_evaluated {
        println!("  note: capability production readiness not evaluated");
    }
    if let Some(note) = &report.capability.note {
        println!("  next: {note}");
    }
    for blocker in &report.capability.blockers {
        println!("  blocker: {blocker}");
    }
    println!(
        "coverage: managed {:.1}%, semantic {:.1}%, traceability {:.1}%, codegen {:.1}% ({}/{})",
        report.managed_percent,
        report.semantic_percent,
        report.traceability_percent,
        report.codegen_percent,
        report.codegen_files,
        report.codegen_eligible_files
    );
    println!(
        "cb_ownership: codegen {}/{} ({:.1}%), handwrite {}/{} ({:.1}%), unmarked {}/{} ({:.1}%)",
        report.cb_ownership.codegen_files,
        report.cb_ownership.eligible_files,
        report.cb_ownership.codegen_percent,
        report.cb_ownership.handwrite_files,
        report.cb_ownership.eligible_files,
        report.cb_ownership.handwrite_percent,
        report.cb_ownership.unmarked_files,
        report.cb_ownership.eligible_files,
        report.cb_ownership.unmarked_percent
    );
    println!(
        "codegen_origin: td_ast {}/{} ({:.1}%), source_template {}/{} ({:.1}%), artifact_replay {}/{} ({:.1}%)",
        report.codegen_origin.td_ast_files,
        report.codegen_origin.target_files,
        percent_of(
            report.codegen_origin.td_ast_files,
            report.codegen_origin.target_files
        ),
        report.codegen_origin.source_template_files,
        report.codegen_origin.target_files,
        percent_of(
            report.codegen_origin.source_template_files,
            report.codegen_origin.target_files
        ),
        report.codegen_origin.artifact_replay_files,
        report.codegen_origin.target_files,
        percent_of(
            report.codegen_origin.artifact_replay_files,
            report.codegen_origin.target_files
        )
    );
    println!(
        "traceability: {} blocker(s), {} orphan TD, {} internal TD",
        report.traceability_blocker_count,
        report.traceability_orphan_td_count,
        report.traceability_internal_td_count
    );
    if !report.traceability_evaluated {
        println!("  note: traceability not evaluated");
    }
    if let Some(note) = &report.traceability_note {
        println!("  next: {note}");
    }
    println!(
        "command_traceability: {:.1}%, {} blocker(s), {} orphan command(s), {} hidden command(s)",
        report.command_traceability_percent,
        report.command_traceability_blocker_count,
        report.command_traceability_orphan_command_count,
        report.command_traceability_hidden_command_count
    );
    println!(
        "gaps: next {}, blocked {}, human_decision_required {}",
        report.next_gap.as_deref().unwrap_or("none"),
        report.blocked_gap_count,
        report.human_decision_required_count
    );
    println!(
        "markers: {} handwrite file(s), {} unmarked file(s)",
        report.handwrite_files, report.unmarked_files
    );
    println!(
        "regenerability_authority: {:?}; required_for_production={}; gap_count={}",
        report.regenerability_authority.authority,
        report.regenerability_authority.required_for_production,
        report.regenerability_authority.gap_count
    );
    println!("  reason: {}", report.regenerability_authority.reason);
    println!(
        "cb_verify: {}",
        if !report.cb_verify_evaluated {
            "not evaluated"
        } else if report.cb_verify_clean {
            "clean"
        } else {
            "blocked"
        }
    );
    if let Some(note) = &report.cb_verify_note {
        println!("  note: {note}");
    }
    println!(
        "public_api: {}/{} covered; semantic_review_required {}",
        report.public_api_covered, report.public_api_total, report.semantic_review_required
    );
    println!(
        "test_gates: {:?}; evaluated={}, commands {}/{} passed",
        report.test_gates.status,
        report.test_gates.evaluated,
        report.test_gates.passed_count,
        report.test_gates.command_count
    );
    if let Some(note) = &report.test_gates.note {
        println!("  note: {note}");
    }
    for command in &report.test_gates.commands {
        println!(
            "  - {}: `{}` [{:?}] exit {:?} ({} ms)",
            command.workspace,
            command.command,
            command.status,
            command.exit_code,
            command.duration_ms
        );
    }
    println!(
        "ec: {:?}; check_clean={}, verify_evaluated={}, cases {}/{}, tool_manifests {}/{}, commands {}/{} passed",
        report.ec.status,
        report.ec.check_clean,
        report.ec.verify_evaluated,
        report.ec.case_count,
        report.ec.expected_case_count,
        report.ec.tool_manifest_count,
        report.ec.expected_tool_manifest_count,
        report.ec.passed_count,
        report.ec.command_count
    );
    println!("  inventory: {}", report.ec.inventory_path);
    if let Some(note) = &report.ec.note {
        println!("  note: {note}");
    }
    for finding in &report.ec.findings {
        println!("  finding: {finding}");
    }
    for command in &report.ec.commands {
        println!(
            "  - {}: `{}` [{:?}] exit {:?} ({} ms)",
            command.case_id,
            command.command,
            command.status,
            command.exit_code,
            command.duration_ms
        );
    }
    println!(
        "cold_rebuild: {} workspace(s); {}",
        report.cold_rebuild_workspace_count,
        if !report.cold_rebuild_evaluated {
            "not evaluated"
        } else if report.cold_rebuild_clean {
            "clean"
        } else {
            "blocked"
        }
    );
    if let Some(note) = &report.cold_rebuild_note {
        println!("  note: {note}");
    }
    for summary in &report.cold_rebuilds {
        println!(
            "  - {}: files {}/{}, specs {}, source_roots {} [{}]",
            summary.workspace.as_deref().unwrap_or("<project>"),
            summary.generated_files,
            summary.expected_files,
            summary.spec_count,
            summary.source_root_count,
            if summary.clean { "clean" } else { "blocked" }
        );
        println!(
            "    codegen_origin: td_ast {}/{} ({:.1}%), source_template {}/{} ({:.1}%), artifact_replay {}/{} ({:.1}%)",
            summary.codegen_origin.td_ast_files,
            summary.codegen_origin.target_files,
            percent_of(
                summary.codegen_origin.td_ast_files,
                summary.codegen_origin.target_files
            ),
            summary.codegen_origin.source_template_files,
            summary.codegen_origin.target_files,
            percent_of(
                summary.codegen_origin.source_template_files,
                summary.codegen_origin.target_files
            ),
            summary.codegen_origin.artifact_replay_files,
            summary.codegen_origin.target_files,
            percent_of(
                summary.codegen_origin.artifact_replay_files,
                summary.codegen_origin.target_files
            )
        );
    }
    println!(
        "stack_migration: {:.1}% normalized; {} incomplete workspace(s)",
        report.stack_migration_percent, report.stack_migration_incomplete_workspaces
    );
    for workspace in &report.stack_migration.workspaces {
        println!(
            "  - {}: {} [{}], dependency policies {}, deployment manifests {}, deployment facets {}",
            workspace.name,
            workspace.migration_state,
            if workspace.normalized {
                "normalized"
            } else {
                "needs annotations"
            },
            workspace.dependency_policies.len(),
            workspace.deployment_manifest_count,
            workspace.deployment_facets.len()
        );
    }
    println!("workflow_locks: {}", report.workflow_lock_count);
    if !report.optional_regenerability_gaps.is_empty() {
        println!("optional_regenerability_gaps:");
        for gap in &report.optional_regenerability_gaps {
            println!("  - {gap}");
        }
    }
    if !report.optional_quality_warnings.is_empty() {
        println!("optional_quality_warnings:");
        for warning in &report.optional_quality_warnings {
            println!("  - {warning}");
        }
    }
    if !report.preflight_gate_reports.is_empty() {
        println!("preflight_gate_reports:");
        for preflight in &report.preflight_gate_reports {
            println!(
                "  - {}: {} result(s), {} blocker(s), {} warning(s)",
                preflight.artifact_ref,
                preflight.results.len(),
                preflight.production_blockers.len(),
                preflight.quality_warnings.len()
            );
            for blocker in &preflight.production_blockers {
                println!("    blocker: {blocker}");
            }
            for warning in &preflight.quality_warnings {
                println!("    warning: {warning}");
            }
        }
    }
    if !report.blockers.is_empty() {
        println!("blockers:");
        for blocker in &report.blockers {
            println!("  - {blocker}");
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn percent_of(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn semantic_gap_blocks_readiness(primitive: &str) -> bool {
    matches!(primitive, "semantic_td_missing" | "semantic_td_legacy")
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn aggregate_codegen_origin(cold_rebuilds: &[CbColdVerifySummary]) -> CbCodegenOriginSummary {
    let mut summary = CbCodegenOriginSummary::default();
    for cold in cold_rebuilds {
        summary.target_files += cold.codegen_origin.target_files;
        summary.td_ast_files += cold.codegen_origin.td_ast_files;
        summary.artifact_replay_files += cold.codegen_origin.artifact_replay_files;
        summary.source_template_files += cold.codegen_origin.source_template_files;
    }
    summary
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
fn cb_ownership_summary(
    eligible_files: usize,
    codegen_files: usize,
    handwrite_files: usize,
    unmarked_files: usize,
) -> CbOwnershipSummary {
    CbOwnershipSummary {
        eligible_files,
        codegen_files,
        handwrite_files,
        unmarked_files,
        codegen_percent: percent_of(codegen_files, eligible_files),
        handwrite_percent: percent_of(handwrite_files, eligible_files),
        unmarked_percent: percent_of(unmarked_files, eligible_files),
    }
}

#[cfg(test)]
/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
mod tests {
    use super::*;

    fn health_args(
        verify_traceability: bool,
        verify_cb: bool,
        verify_cold: bool,
        verify_tests: bool,
        verify_ec: bool,
    ) -> ProjectHealthArgs {
        ProjectHealthArgs {
            project: "demo".to_string(),
            section: None,
            verify_traceability,
            verify_cb,
            verify_cold,
            verify_tests,
            verify_ec,
            json: false,
            human: false,
            pretty: false,
            verbose: false,
        }
    }

    fn health_section_args(section: ProjectHealthSection) -> ProjectHealthArgs {
        ProjectHealthArgs {
            section: Some(section),
            ..health_args(false, false, false, false, false)
        }
    }

    fn ready_project_health_report(project: &str) -> ProjectHealthReport {
        let mut report = ProjectHealthReport::from_components(
            project,
            StandardizationCoverage {
                scope: Vec::new(),
                total_files: 0,
                managed_files: 0,
                percent: 100.0,
                by_language: BTreeMap::new(),
                by_marker: crate::cli::standardize::MarkerCounts::default(),
                uncovered_files: Vec::new(),
            },
            SemanticCoverage {
                scope: Vec::new(),
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
            },
            RegenerabilityCoverage {
                scope: Vec::new(),
                total_files: 0,
                eligible_files: 0,
                codegen_files: 0,
                handwrite_files: 0,
                unmarked_files: 0,
                unsupported_codegen_files: Vec::new(),
                non_replayable_codegen_files: Vec::new(),
                snapshot_codegen_files: Vec::new(),
                codegen_drift_evaluated: true,
                codegen_drift_files: Vec::new(),
                percent: 100.0,
                gap_files: Vec::new(),
                semantic_percent: 100.0,
                generator_primitive_gaps: 0,
                primitive_covered_files: 0,
                missing_generator_primitive_gaps: 0,
                insufficient_td_section_gaps: 0,
                human_decision_required_gaps: 0,
                next_gap: None,
                authority_mode: RegenerabilityAuthority::ExternalAdvisory,
                required_for_production: false,
                authority_reason: "fixture".to_string(),
            },
            StackMigrationCoverage {
                project: project.to_string(),
                workspaces: Vec::new(),
                migration_normalized_percent: 100.0,
                incomplete_workspace_count: 0,
                dependency_policy_blockers: Vec::new(),
                deployment_policy_blockers: Vec::new(),
                blockers: Vec::new(),
            },
            CbVerifySummary {
                clean: true,
                public_api_covered: 0,
                public_api_total: 0,
                semantic_review_required: 0,
                failures: Vec::new(),
            },
            Vec::new(),
            ProjectTestGateReport::passed_fixture("true"),
        );
        report.claim_closure.evaluated = true;
        report.claim_closure.note = None;
        report
    }

    #[test]
    fn health_without_verify_flags_defaults_to_metrics_only() {
        let flags =
            effective_health_verification_flags(&health_args(false, false, false, false, false));

        assert_eq!(
            flags,
            HealthVerificationFlags {
                traceability: false,
                cb: false,
                cold: false,
                tests: false,
                ec: false,
            }
        );
    }

    #[test]
    fn health_full_section_runs_full_verification() {
        let flags =
            effective_health_verification_flags(&health_section_args(ProjectHealthSection::Full));

        assert_eq!(
            flags,
            HealthVerificationFlags {
                traceability: true,
                cb: true,
                cold: true,
                tests: true,
                ec: true,
            }
        );
    }

    #[test]
    fn health_with_one_verify_flag_preserves_targeted_debug_mode() {
        let flags =
            effective_health_verification_flags(&health_args(false, false, false, true, false));

        assert_eq!(
            flags,
            HealthVerificationFlags {
                traceability: false,
                cb: false,
                cold: false,
                tests: true,
                ec: false,
            }
        );
    }

    #[test]
    fn health_verify_ec_is_targeted_debug_mode() {
        let flags =
            effective_health_verification_flags(&health_args(false, false, false, false, true));

        assert_eq!(
            flags,
            HealthVerificationFlags {
                traceability: false,
                cb: false,
                cold: false,
                tests: false,
                ec: true,
            }
        );
    }

    #[test]
    fn focused_capability_health_does_not_run_expensive_gates_by_default() {
        let flags = effective_health_verification_flags(&health_section_args(
            ProjectHealthSection::Capability,
        ));

        assert_eq!(
            flags,
            HealthVerificationFlags {
                traceability: false,
                cb: false,
                cold: false,
                tests: false,
                ec: false,
            }
        );
    }

    #[test]
    fn focused_tests_health_runs_only_test_gates_by_default() {
        let flags =
            effective_health_verification_flags(&health_section_args(ProjectHealthSection::Tests));

        assert_eq!(
            flags,
            HealthVerificationFlags {
                traceability: false,
                cb: false,
                cold: false,
                tests: true,
                ec: false,
            }
        );
    }

    #[test]
    fn project_test_gate_times_out_and_reports_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let progress = HealthProgressSink::disabled("demo");
        let report = run_project_test_command_with_timeout(
            "demo",
            "sleep 2",
            tmp.path(),
            &progress,
            Duration::from_millis(200),
        )
        .unwrap();

        assert_eq!(report.status, ProjectTestCommandStatus::TimedOut);
        assert_eq!(report.exit_code, None);
        assert!(report.stderr_tail.contains("aw test gate timed out"));
    }

    #[test]
    fn agentic_workflow_self_health_skips_workspace_test_gates() {
        let tmp = tempfile::tempdir().unwrap();
        let progress = HealthProgressSink::disabled("agentic-workflow");
        let report =
            project_test_gate_report_with_progress("agentic-workflow", tmp.path(), true, &progress)
                .unwrap();

        assert!(report.evaluated);
        assert_eq!(report.status, ProjectTestGateStatus::Passed);
        assert_eq!(report.command_count, 0);
        assert!(report
            .note
            .as_deref()
            .unwrap_or_default()
            .contains("capability and EC gates are authoritative"));
    }

    #[test]
    fn agentic_workflow_self_health_missing_ignores_advisory_gaps() {
        let mut report = ready_project_health_report("agentic-workflow");
        report.production_ready = false;
        report.status = ProjectHealthStatus::Blocked;
        report.production_status = ProductionStatus::Blocked;
        report.traceability_evaluated = false;
        report.cb_verify_evaluated = false;
        report.cold_rebuild_evaluated = false;
        report.cold_rebuild_workspace_count = 1;
        report.test_gates = ProjectTestGateReport::not_evaluated("agentic-workflow");
        report.next_gap = Some("src/lib.rs semantic_td_missing".to_string());
        report.blockers =
            vec!["claim closure `cap`:`claim`: missing required production EC case".to_string()];

        let missing = project_health_missing(&report);

        assert_eq!(missing, report.blockers);
    }

    #[test]
    fn agentic_workflow_self_health_next_ignores_td_lock() {
        let mut report = ready_project_health_report("agentic-workflow");
        report.production_ready = false;
        report.status = ProjectHealthStatus::Blocked;
        report.production_status = ProductionStatus::Blocked;
        report.td_lock = crate::cli::td_lock::TdLockStatus {
            project: "agentic-workflow".to_string(),
            td_path: "projects/agentic-workflow/tech-design".to_string(),
            lock_path: "projects/agentic-workflow/tech-design/td.lock".to_string(),
            status: crate::cli::td_lock::TdLockState::Stale,
            clean: false,
            current_digest: "sha256:new".to_string(),
            locked_digest: Some("sha256:old".to_string()),
            file_count: 1,
            changed: vec!["x.md".to_string()],
            added: Vec::new(),
            removed: Vec::new(),
            message: "td lock stale".to_string(),
        };
        report.ec = ProjectEcGateReport {
            evaluated: true,
            check_clean: false,
            verify_evaluated: false,
            status: ProjectEcGateStatus::CheckFailed,
            note: Some("EC inventory/check is blocked".to_string()),
            inventory_path: "projects/agentic-workflow/aw.toml".to_string(),
            expected_case_count: 1,
            case_count: 1,
            expected_tool_manifest_count: 0,
            tool_manifest_count: 0,
            command_count: 0,
            passed_count: 0,
            failed_count: 0,
            findings: vec!["EC generated content drifted".to_string()],
            commands: Vec::new(),
        };

        assert_eq!(
            project_health_next_command(&report).as_deref(),
            Some("aw ec gen --project agentic-workflow --verify")
        );
        assert_eq!(
            project_health_next_reason(&report),
            "EC generated content drifted"
        );
    }

    #[test]
    fn focused_claims_health_runs_traceability_and_ec_by_default() {
        let flags =
            effective_health_verification_flags(&health_section_args(ProjectHealthSection::Claims));

        assert_eq!(
            flags,
            HealthVerificationFlags {
                traceability: true,
                cb: false,
                cold: false,
                tests: false,
                ec: true,
            }
        );
    }

    fn claim_document(required_for_verified: bool) -> crate::cli::capability::CapabilityDocument {
        crate::cli::capability::CapabilityDocument {
            cap_path: std::path::PathBuf::from("projects/demo/README.md"),
            format: crate::cli::capability::CapabilityDocumentFormat::MarkdownTables,
            needs_canonicalization: false,
            capabilities: vec![crate::cli::capability::CapabilitySection {
                title: "Demo Capability".to_string(),
                id: "cap".to_string(),
                status: crate::cli::capability::CapabilityStatus::Verified,
                prelude: String::new(),
                postlude: String::new(),
                index_summary: None,
                capability_type: None,
                surfaces: Vec::new(),
                ec_dimensions: Vec::new(),
                promise: "promise".to_string(),
                current_state: "state".to_string(),
                gaps: Vec::new(),
                work_roots: Vec::new(),
                verification_contract: Some(
                    crate::cli::capability::CapabilityVerificationContract {
                        required_maturity: vec![crate::cli::capability::CapabilityMaturity::Smoke],
                        claims: vec![crate::cli::capability::CapabilityClaim {
                            id: "claim".to_string(),
                            user_story: "story".to_string(),
                            required_for_verified,
                            maturity: crate::cli::capability::CapabilityMaturity::Smoke,
                            oracle: "oracle".to_string(),
                            fixtures: Vec::new(),
                            negative_cases: Vec::new(),
                            gates: Vec::new(),
                        }],
                        full_regenerability_required: false,
                    },
                ),
                evidence: crate::cli::capability::CapabilityEvidence::default(),
                done_when: Vec::new(),
                out_of_scope: Vec::new(),
                release_scope: true,
                dependencies: Vec::new(),
                line: 1,
            }],
            legacy_rows: Vec::new(),
            prose_candidates: Vec::new(),
            findings: Vec::new(),
        }
    }

    fn td_claim_ref() -> crate::cli::capability::TdCapabilityEvidence {
        crate::cli::capability::TdCapabilityEvidence {
            spec_path: "projects/demo/tech-design/logic/claim.md".to_string(),
            spec_id: Some("demo-claim".to_string()),
            review_status: None,
            capability_id: "cap".to_string(),
            role: crate::cli::capability::CapabilityRefRole::Primary,
            gap: None,
            claim: Some("claim".to_string()),
            coverage: crate::cli::capability::CapabilityCoverage::Full,
            rationale: None,
        }
    }

    fn ec_manifest(cases: Vec<crate::cli::ec::EcManifestCase>) -> crate::cli::ec::EcManifest {
        crate::cli::ec::EcManifest {
            version: 1,
            project: "demo".to_string(),
            generated_from_td_digest: "digest".to_string(),
            cases,
            tool_manifests: Vec::new(),
        }
    }

    fn ec_report_for(case_id: &str, status: ProjectTestCommandStatus) -> ProjectEcGateReport {
        ProjectEcGateReport {
            evaluated: true,
            check_clean: true,
            verify_evaluated: true,
            status: if status == ProjectTestCommandStatus::Passed {
                ProjectEcGateStatus::Passed
            } else {
                ProjectEcGateStatus::Failed
            },
            note: None,
            inventory_path: "projects/demo/aw.toml".to_string(),
            expected_case_count: 1,
            case_count: 1,
            expected_tool_manifest_count: 0,
            tool_manifest_count: 0,
            command_count: 1,
            passed_count: if status == ProjectTestCommandStatus::Passed {
                1
            } else {
                0
            },
            failed_count: if status == ProjectTestCommandStatus::Failed {
                1
            } else {
                0
            },
            findings: Vec::new(),
            commands: vec![ProjectEcCommandReport {
                case_id: case_id.to_string(),
                command: "true".to_string(),
                status,
                exit_code: Some(0),
                duration_ms: 1,
                stdout_tail: String::new(),
                stderr_tail: String::new(),
            }],
        }
    }

    #[test]
    fn claim_closure_closes_when_required_edges_are_present() {
        let document = claim_document(true);
        let case = ec_case("behavior");
        let manifest = ec_manifest(vec![case]);
        let ec_report = ec_report_for("case-1", ProjectTestCommandStatus::Passed);

        let report = build_claim_closure_report(
            "demo",
            &document,
            &[td_claim_ref()],
            Some(&manifest),
            &ec_report,
            true,
            true,
        );

        assert_eq!(report.claim_total, 1);
        assert_eq!(report.closed_claim_count, 1);
        assert_eq!(report.claim_closure_percent, 100.0);
        assert!(report.blockers.is_empty());
        assert_eq!(report.claims[0].status, ProjectClaimClosureStatus::Closed);
    }

    #[test]
    fn claim_closure_closes_duplicate_command_sibling_cases() {
        let document = claim_document(true);
        let mut case = ec_case("behavior");
        case.id = "case-2".to_string();
        let manifest = ec_manifest(vec![case]);
        let mut ec_report = ec_report_for("case-1", ProjectTestCommandStatus::Passed);
        ec_report.commands[0].command = "cargo test -p demo".to_string();

        let report = build_claim_closure_report(
            "demo",
            &document,
            &[td_claim_ref()],
            Some(&manifest),
            &ec_report,
            true,
        );

        assert_eq!(report.closed_claim_count, 1);
        assert_eq!(report.claims[0].passing_ec_case_ids, vec!["case-2"]);
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn claim_closure_blocks_missing_ec_case() {
        let document = claim_document(true);
        let manifest = ec_manifest(Vec::new());
        let ec_report = ec_report_for("case-1", ProjectTestCommandStatus::Passed);

        let report = build_claim_closure_report(
            "demo",
            &document,
            &[td_claim_ref()],
            Some(&manifest),
            &ec_report,
            true,
            true,
        );

        assert_eq!(report.closed_claim_count, 0);
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker.contains("missing required production EC case")));
    }

    #[test]
    fn claim_closure_blocks_unmapped_production_ec_case() {
        let document = claim_document(true);
        let mut case = ec_case("behavior");
        case.capability_id = "unmapped".to_string();
        let manifest = ec_manifest(vec![case]);
        let ec_report = ec_report_for("case-1", ProjectTestCommandStatus::Passed);

        let report = build_claim_closure_report(
            "demo",
            &document,
            &[td_claim_ref()],
            Some(&manifest),
            &ec_report,
            true,
            true,
        );

        assert_eq!(report.closed_claim_count, 0);
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker.contains("is unmapped")));
    }

    #[test]
    fn claim_closure_blocks_missing_primary_td_ref() {
        let document = claim_document(true);
        let case = ec_case("behavior");
        let manifest = ec_manifest(vec![case]);
        let ec_report = ec_report_for("case-1", ProjectTestCommandStatus::Passed);

        let report = build_claim_closure_report(
            "demo",
            &document,
            &[],
            Some(&manifest),
            &ec_report,
            true,
            true,
        );

        assert_eq!(report.closed_claim_count, 0);
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker.contains("missing primary TD capability_ref")));
    }

    #[test]
    fn claim_closure_ignores_optional_claims() {
        let document = claim_document(false);
        let manifest = ec_manifest(Vec::new());
        let ec_report = ec_report_for("case-1", ProjectTestCommandStatus::Passed);

        let report = build_claim_closure_report(
            "demo",
            &document,
            &[],
            Some(&manifest),
            &ec_report,
            true,
            true,
        );

        assert_eq!(report.claim_total, 0);
        assert_eq!(report.claim_closure_percent, 100.0);
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn claim_closure_self_policy_only_requires_capability_and_ec() {
        let document = claim_document(true);
        let case = ec_case("behavior");
        let manifest = ec_manifest(vec![case]);
        let ec_report = ec_report_for("case-1", ProjectTestCommandStatus::Passed);

        let report = build_claim_closure_report(
            "agentic-workflow",
            &document,
            &[],
            Some(&manifest),
            &ec_report,
            true,
            false,
        );

        assert_eq!(report.closed_claim_count, 1);
        assert!(report.blockers.is_empty());
    }

    fn ec_case(category: &str) -> crate::cli::ec::EcManifestCase {
        crate::cli::ec::EcManifestCase {
            id: "case-1".into(),
            capability_id: "cap".into(),
            claim_id: "claim".into(),
            contract_id: "contract".into(),
            category: category.into(),
            td_ref: "td".into(),
            test_path: "tests/x.rs".into(),
            command: "cargo test -p demo".into(),
            required_for_production: true,
            assertions: vec![],
            evidence: vec![],
            evaluators: vec![],
        }
    }

    fn ec_project(ec: BTreeMap<String, EcBinding>) -> crate::models::project::Project {
        crate::models::project::Project {
            name: "demo".into(),
            path: "projects/demo".into(),
            tech_design_dir: None,
            ec,
            workspaces: vec![],
        }
    }

    /// wi-38 AC2: the builder emits deterministic tool shapes, including vat,
    /// guard, and legacy arena compatibility.
    #[test]
    fn ec_binding_command_builds_rig_meter_vat_guard_and_legacy_arena() {
        let arena = EcBinding {
            tool: "arena".into(),
            command: None,
            spec: Some("tests/arena/x.toml".into()),
            dir: None,
            meter: None,
        };
        assert_eq!(
            arena.command().unwrap(),
            "arena run --spec tests/arena/x.toml"
        );

        let rig = EcBinding {
            tool: "rig".into(),
            command: None,
            spec: None,
            dir: Some("tests/rig/scenarios".into()),
            meter: None,
        };
        assert_eq!(rig.command().unwrap(), "rig run --dir tests/rig/scenarios");

        let meter = EcBinding {
            tool: "meter".into(),
            command: None,
            spec: None,
            dir: None,
            meter: Some(".".into()),
        };
        assert_eq!(meter.command().unwrap(), "meter run --target .");

        let vat_default = EcBinding {
            tool: "vat".into(),
            command: None,
            spec: None,
            dir: None,
            meter: None,
        };
        assert_eq!(vat_default.command().unwrap(), "vat run");

        let vat_blank_runner = EcBinding {
            tool: "vat".into(),
            command: None,
            spec: None,
            dir: Some("   ".into()),
            meter: None,
        };
        assert_eq!(vat_blank_runner.command().unwrap(), "vat run");

        let vat_named_runner = EcBinding {
            tool: "vat".into(),
            command: None,
            spec: None,
            dir: Some("rig-load".into()),
            meter: None,
        };
        assert_eq!(vat_named_runner.command().unwrap(), "vat run rig-load");

        let guard_default = EcBinding {
            tool: "guard".into(),
            command: None,
            spec: None,
            dir: None,
            meter: None,
        };
        assert_eq!(
            guard_default.command().unwrap(),
            "guard scan . --compact --no-persist"
        );

        let guard_project = EcBinding {
            tool: "guard".into(),
            command: None,
            spec: None,
            dir: Some("projects/guard".into()),
            meter: None,
        };
        assert_eq!(
            guard_project.command().unwrap(),
            "guard scan projects/guard --compact --no-persist"
        );

        let guard_override = EcBinding {
            tool: "guard".into(),
            command: Some("target/debug/guard scan projects/guard --compact --no-persist".into()),
            spec: None,
            dir: Some("projects/guard".into()),
            meter: None,
        };
        assert_eq!(
            guard_override.command().unwrap(),
            "target/debug/guard scan projects/guard --compact --no-persist"
        );
    }

    /// wi-13 AC2: unknown tool and missing per-tool argument are errors.
    #[test]
    fn ec_binding_command_rejects_unknown_tool_and_missing_arg() {
        let unknown = EcBinding {
            tool: "valgrind".into(),
            command: None,
            spec: None,
            dir: None,
            meter: None,
        };
        assert!(unknown
            .command()
            .unwrap_err()
            .to_string()
            .contains("expected rig|meter|vat|guard, or legacy arena"));

        let armless = EcBinding {
            tool: "arena".into(),
            command: None,
            spec: None,
            dir: None,
            meter: None,
        };
        assert!(armless
            .command()
            .unwrap_err()
            .to_string()
            .contains("requires `spec`"));
    }

    /// Generated aw.toml case commands are authoritative. A bound category is
    /// used only when an old inventory has a blank per-case command.
    #[test]
    fn resolve_ec_command_prefers_case_command_and_falls_back_to_bound_category() {
        let mut ec = BTreeMap::new();
        ec.insert(
            "efficiency".to_string(),
            EcBinding {
                tool: "meter".into(),
                command: None,
                spec: None,
                dir: None,
                meter: Some("projects/lumen".into()),
            },
        );
        let project = ec_project(ec);

        let explicit = resolve_project_ec_command(&ec_case("efficiency"), Some(&project)).unwrap();
        assert_eq!(explicit, "cargo test -p demo");

        let mut blank_case = ec_case("efficiency");
        blank_case.command.clear();
        let bound = resolve_project_ec_command(&blank_case, Some(&project)).unwrap();
        assert_eq!(bound, "meter run --target projects/lumen");

        let unbound = resolve_project_ec_command(&ec_case("correctness"), Some(&project)).unwrap();
        assert_eq!(unbound, "cargo test -p demo");
    }

    #[test]
    fn resolve_ec_command_accepts_legacy_benchmark_binding_for_efficiency() {
        let mut ec = BTreeMap::new();
        ec.insert(
            "benchmark".to_string(),
            EcBinding {
                tool: "arena".into(),
                command: None,
                spec: Some("projects/arena/examples/lumen-vs-pg.toml".into()),
                dir: None,
                meter: None,
            },
        );
        let project = ec_project(ec);

        let mut case = ec_case("efficiency");
        case.command.clear();
        let command = resolve_project_ec_command(&case, Some(&project)).unwrap();
        assert_eq!(
            command,
            "arena run --spec projects/arena/examples/lumen-vs-pg.toml"
        );
    }

    #[test]
    fn resolve_ec_command_accepts_efficiency_binding_for_legacy_benchmark_case() {
        let mut ec = BTreeMap::new();
        ec.insert(
            "efficiency".to_string(),
            EcBinding {
                tool: "meter".into(),
                command: None,
                spec: None,
                dir: None,
                meter: Some("projects/lumen".into()),
            },
        );
        let project = ec_project(ec);

        let mut case = ec_case("benchmark");
        case.command.clear();
        let command = resolve_project_ec_command(&case, Some(&project)).unwrap();
        assert_eq!(command, "meter run --target projects/lumen");
    }

    /// wi-13 AC4: no `ec` map (or no project model at all) is today's
    /// behavior — pure generated-case-command verify-ec.
    #[test]
    fn resolve_ec_command_defaults_without_bindings() {
        let project = ec_project(BTreeMap::new());
        assert_eq!(
            resolve_project_ec_command(&ec_case("benchmark"), Some(&project)).unwrap(),
            "cargo test -p demo"
        );
        assert_eq!(
            resolve_project_ec_command(&ec_case("benchmark"), None).unwrap(),
            "cargo test -p demo"
        );
    }

    #[test]
    fn health_ec_tool_manifest_command_reports_as_ec_command() {
        let tmp = tempfile::tempdir().unwrap();
        let tool = crate::cli::ec::EcToolManifest {
            id: "demo-guard".to_string(),
            tool: "guard".to_string(),
            path: "projects/demo/guard.toml".to_string(),
            td_ref: "projects/demo/external-contracts/security/guard.md#demo guard".to_string(),
            content_digest: "sha256:demo".to_string(),
            command: "true".to_string(),
            category: "security".to_string(),
            generated_toml: String::new(),
        };

        let report = run_project_ec_tool_manifest_command(&tool, tmp.path()).unwrap();

        assert_eq!(report.case_id, "tool:demo-guard");
        assert_eq!(report.status, ProjectTestCommandStatus::Passed);
        assert_eq!(report.command, "true");
    }

    #[test]
    fn health_ec_tool_manifest_missing_command_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let tool = crate::cli::ec::EcToolManifest {
            id: "demo-guard".to_string(),
            tool: "guard".to_string(),
            path: "projects/demo/guard.toml".to_string(),
            td_ref: "projects/demo/external-contracts/security/guard.md#demo guard".to_string(),
            content_digest: "sha256:demo".to_string(),
            command: String::new(),
            category: "security".to_string(),
            generated_toml: String::new(),
        };

        let report = run_project_ec_tool_manifest_command(&tool, tmp.path()).unwrap();

        assert_eq!(report.case_id, "tool:demo-guard");
        assert_eq!(report.status, ProjectTestCommandStatus::Failed);
        assert!(report.stderr_tail.contains("missing command"));
    }

    /// wi-13 AC1: `[[projects]] ... ec.<category>` round-trips through the
    /// Project model; absence of the field serializes to nothing.
    #[test]
    fn project_ec_map_roundtrips_through_toml() {
        let doc = r#"
[[projects]]
name = "lumen"
path = "projects/lumen"
ec.efficiency = { tool = "meter", meter = "projects/lumen" }

[[projects.workspaces]]
paths = ["projects/lumen/**"]
target = "rust"
"#;
        let parsed: crate::models::project::ProjectsToml = toml::from_str(doc).unwrap();
        let project = &parsed.projects[0];
        assert_eq!(project.ec["efficiency"].tool, "meter");
        assert_eq!(
            project.ec["efficiency"].meter.as_deref(),
            Some("projects/lumen")
        );

        let reserialized = toml::to_string(&parsed).unwrap();
        assert!(
            reserialized.contains("[projects.ec.efficiency]")
                || reserialized.contains("ec.efficiency")
        );
        let reparsed: crate::models::project::ProjectsToml = toml::from_str(&reserialized).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
// CODEGEN-END
