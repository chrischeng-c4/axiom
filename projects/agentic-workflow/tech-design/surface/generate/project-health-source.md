---
id: project-health-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Project health generated surfaces implement standardization readiness reporting and gate evidence."
---

# Project Health Source Template

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/project.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CapabilityHealthReport` | projects/agentic-workflow/src/cli/project.rs | struct | pub | 131 |  |
| `ProjectHealthArgs` | projects/agentic-workflow/src/cli/project.rs | struct | pub | 40 |  |
| `ProjectHealthReport` | projects/agentic-workflow/src/cli/project.rs | struct | pub | 68 |  |
| `ProjectHealthStatus` | projects/agentic-workflow/src/cli/project.rs | enum | pub | 205 |  |
| `ProjectTestCommandReport` | projects/agentic-workflow/src/cli/project.rs | struct | pub | 235 |  |
| `ProjectTestCommandStatus` | projects/agentic-workflow/src/cli/project.rs | enum | pub | 248 |  |
| `ProjectTestGateReport` | projects/agentic-workflow/src/cli/project.rs | struct | pub | 212 |  |
| `ProjectTestGateStatus` | projects/agentic-workflow/src/cli/project.rs | enum | pub | 226 |  |
| `RegenerabilityAuthorityReport` | projects/agentic-workflow/src/cli/project.rs | struct | pub | 193 |  |
| `apply_preflight_gate_report` | projects/agentic-workflow/src/cli/project.rs | function | pub | 970 | apply_preflight_gate_report(&mut self, report: PreFlightGateReport) |
| `apply_workflow_locks_to_report` | projects/agentic-workflow/src/cli/project.rs | function | pub | 1726 | apply_workflow_locks_to_report(report: &mut ProjectHealthReport) -> Result<()> |
| `build_health_report` | projects/agentic-workflow/src/cli/project.rs | function | pub | 254 | build_health_report(project: &str) -> Result<ProjectHealthReport> |
| `build_health_report_with_options` | projects/agentic-workflow/src/cli/project.rs | function | pub | 259 | build_health_report_with_options(     project: &str,     verify_traceability: bool,     verify_cb: bool,     verify_cold: bool,     verify_tests: bool, ) -> Result<ProjectHealthReport> |
| `build_health_report_with_test_gates` | projects/agentic-workflow/src/cli/project.rs | function | pub | 304 | build_health_report_with_test_gates(     project: &str,     verify_traceability: bool,     verify_cb: bool,     verify_cold: bool,     test_gates: ProjectTestGateReport,     production_gates_evaluated: bool, ) -> Result<ProjectHealthReport> |
| `build_health_report_with_test_gates_and_capability_verified` | projects/agentic-workflow/src/cli/project.rs | function | pub | 324 | build_health_report_with_test_gates_and_capability_verified(     project: &str,     verify_traceability: bool,     verify_cb: bool,     verify_cold: bool,     test_gates: ProjectTestGateReport,     production_gates_evaluated: bool,     capability_verified_by_id: Option<BTreeMap<String, bool>>, ) -> Result<ProjectHealthReport> |
| `from_components` | projects/agentic-workflow/src/cli/project.rs | function | pub | 661 | from_components(         project: &str,         managed: StandardizationCoverage,         semantic: SemanticCoverage,         regenerable: RegenerabilityCoverage,         stack_migration: StackMigrationCoverage,         cb: CbVerifySummary,         cold_rebuilds: Vec<CbColdVerifySummary>,         test_gates: ProjectTestGateReport,     ) -> Self |
| `from_components_with_traceability` | projects/agentic-workflow/src/cli/project.rs | function | pub | 685 | from_components_with_traceability(         project: &str,         managed: StandardizationCoverage,         semantic: SemanticCoverage,         traceability: TraceabilityCoverage,         regenerable: RegenerabilityCoverage,         stack_migration: StackMigrationCoverage,         cb: CbVerifySummary,         cold_rebuilds: Vec<CbColdVerifySummary>,         test_gates: ProjectTestGateReport,     ) -> Self |
| `not_evaluated` | projects/agentic-workflow/src/cli/project.rs | function | pub | 1099 | not_evaluated(project: &str) -> Self |
| `passed_fixture` | projects/agentic-workflow/src/cli/project.rs | function | pub | 1114 | passed_fixture(command: &str) -> Self |
| `project_health_summary` | projects/agentic-workflow/src/cli/project.rs | function | pub | 1330 | project_health_summary(report: &ProjectHealthReport) -> serde_json::Value |
| `project_health_summary_with_payload_path` | projects/agentic-workflow/src/cli/project.rs | function | pub | 1345 | project_health_summary_with_payload_path(     report: &ProjectHealthReport,     payload_path: &str, ) -> serde_json::Value |
| `project_test_gate_report` | projects/agentic-workflow/src/cli/project.rs | function | pub | 1137 | project_test_gate_report(     project: &str,     project_root: &std::path::Path,     verify_tests: bool, ) -> Result<ProjectTestGateReport> |
| `run_health` | projects/agentic-workflow/src/cli/project.rs | function | pub | 1632 | run_health(args: ProjectHealthArgs) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/project.rs -->
````rust
use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use std::process::Command;
use std::time::Instant;

use crate::cli::cb::{CbCodegenOriginSummary, CbColdVerifySummary, CbVerifySummary};
use crate::cli::production::{
    evaluate_release_scope, inputs_from_sections, ProductionCapabilityReadiness, ProductionStatus,
};
use crate::cli::standardize::{
    RegenerabilityCoverage, SemanticCoverage, StackMigrationCoverage, StandardizationCoverage,
};

#[derive(Debug, Args, Clone)]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#cli
pub struct ProjectHealthArgs {
    // Configured project name from [[projects]] in .aw/config.toml.
    pub project: String,
    // Run expensive TD-only cold rebuild gates for verify_cold workspaces.
    #[arg(long)]
    pub verify_cold: bool,
    // Run configured workspace test commands as production release gates.
    #[arg(long)]
    pub verify_tests: bool,
    // Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub struct ProjectHealthReport {
    pub project: String,
    pub status: ProjectHealthStatus,
    pub production_ready: bool,
    pub production_status: ProductionStatus,
    pub production_scope: Vec<String>,
    pub production_blockers: Vec<String>,
    pub global_blockers: Vec<String>,
    pub scoped_capabilities: Vec<ProductionCapabilityReadiness>,
    pub test_gates: ProjectTestGateReport,
    pub managed_percent: f64,
    pub semantic_percent: f64,
    pub regenerable_percent: f64,
    pub codegen_percent: f64,
    pub full_codegen_percent: f64,
    pub codegen_eligible_files: usize,
    pub codegen_files: usize,
    pub fully_codegen_files: usize,
    pub cb_ownership: CbOwnershipSummary,
    pub codegen_origin: CbCodegenOriginSummary,
    pub td_ast_codegen_percent: f64,
    pub next_gap: Option<String>,
    pub blocked_gap_count: usize,
    pub human_decision_required_count: usize,
    pub handwrite_files: usize,
    pub unmarked_files: usize,
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
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
}

// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#logic
pub fn build_health_report(project: &str) -> Result<ProjectHealthReport> {
    build_health_report_with_options(project, false, false)
}

pub(crate) fn build_health_report_with_options(
    project: &str,
    verify_cold: bool,
    verify_tests: bool,
) -> Result<ProjectHealthReport> {
    let project_root = crate::find_project_root()?;
    let test_gates = project_test_gate_report(project, &project_root, verify_tests)?;
    build_health_report_with_test_gates(project, verify_cold, test_gates, verify_tests)
}

pub(crate) fn build_health_report_with_test_gates(
    project: &str,
    verify_cold: bool,
    test_gates: ProjectTestGateReport,
    production_gates_evaluated: bool,
) -> Result<ProjectHealthReport> {
    let managed = crate::cli::standardize::project_managed_coverage(project)?;
    let semantic = crate::cli::standardize::project_semantic_coverage(project)?;
    let regenerable = crate::cli::standardize::project_regenerability_coverage(project)?;
    let stack_migration = crate::cli::standardize::project_stack_migration_coverage(project)?;
    let cb = crate::cli::cb::project_force_regen_verify_summary(project)?;
    let cold_workspace_count =
        crate::cli::cb::project_force_regen_cold_verify_workspaces(project)?.len();
    let cold_rebuilds = if verify_cold {
        crate::cli::cb::project_force_regen_cold_verify_summary(project)?
    } else {
        Vec::new()
    };
    let mut report = ProjectHealthReport::from_components(
        project,
        managed,
        semantic,
        regenerable,
        stack_migration,
        cb,
        cold_rebuilds,
        test_gates,
    );
    if verify_cold && cold_workspace_count == 0 {
        report.cold_rebuild_evaluated = false;
        report.cold_rebuild_workspace_count = 0;
        report.cold_rebuild_clean = true;
        report.cold_rebuild_note = Some(format!(
            "not evaluated; project `{project}` has no workspace with `verify_cold = true`"
        ));
    } else if !verify_cold {
        report.cold_rebuild_evaluated = false;
        report.cold_rebuild_workspace_count = cold_workspace_count;
        report.cold_rebuild_clean = true;
        report.cold_rebuild_note = if cold_workspace_count == 0 {
            None
        } else {
            Some(format!(
                "skipped by default; run `aw health {project} --verify-cold` or workspace `aw cb gen ... --verify-cold`"
            ))
        };
    }
    apply_scoped_production_readiness(&mut report, production_gates_evaluated)?;
    Ok(report)
}

fn apply_scoped_production_readiness(
    report: &mut ProjectHealthReport,
    production_gates_evaluated: bool,
) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let production =
        match crate::cli::capability::resolve_capability_path(&project_root, &report.project, None)
        {
            Ok(cap_path) => match std::fs::read_to_string(&cap_path) {
                Ok(body) => {
                    match crate::cli::capability::parse_capability_document(&body, &cap_path) {
                        Ok(document) => evaluate_release_scope(
                            inputs_from_sections(
                                &document.capabilities,
                                &std::collections::BTreeMap::new(),
                            ),
                            report.blockers.clone(),
                            production_gates_evaluated,
                        ),
                        Err(err) => evaluate_release_scope(
                            Vec::new(),
                            vec![format!("capability document parse failed: {err}")],
                            production_gates_evaluated,
                        ),
                    }
                }
                Err(err) => evaluate_release_scope(
                    Vec::new(),
                    vec![format!("capability document read failed: {err}")],
                    production_gates_evaluated,
                ),
            },
            Err(err) => evaluate_release_scope(
                Vec::new(),
                vec![format!("capability path resolution failed: {err}")],
                production_gates_evaluated,
            ),
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
    report.status = if report.blockers.is_empty() {
        ProjectHealthStatus::Healthy
    } else {
        ProjectHealthStatus::Blocked
    };
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
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
        let mut blockers = Vec::new();
        let mut optional_regenerability_gaps = Vec::new();
        if !managed.uncovered_files.is_empty() {
            blockers.push(format!(
                "{} unmanaged source file(s)",
                managed.uncovered_files.len()
            ));
        }
        if regenerable.handwrite_files > 0 {
            optional_regenerability_gaps.push(format!(
                "{} file(s) still contain HANDWRITE gaps",
                regenerable.handwrite_files
            ));
        }
        if regenerable.unmarked_files > 0 {
            optional_regenerability_gaps.push(format!(
                "{} source file(s) have no ownership marker",
                regenerable.unmarked_files
            ));
        }
        if !regenerable.unsupported_codegen_files.is_empty() {
            optional_regenerability_gaps.push(format!(
                "{} CODEGEN file(s) are not replay-supported by current generators",
                regenerable.unsupported_codegen_files.len()
            ));
        }
        if !regenerable.codegen_drift_files.is_empty() {
            optional_regenerability_gaps.push(format!(
                "{} CODEGEN file(s) have audit/replay drift",
                regenerable.codegen_drift_files.len()
            ));
        }
        if !semantic.uncovered_files.is_empty() {
            blockers.push(format!(
                "semantic TD coverage incomplete: {}/{}",
                semantic.semantically_covered_files, semantic.total_files
            ));
        }
        if let Some(gap) = &semantic.next_gap {
            blockers.push(format!(
                "next semantic gap: {} {}",
                gap.target, gap.primitive
            ));
        }
        if !cb.clean {
            blockers.push(format!("cb verify has {} finding(s)", cb.failures.len()));
            blockers.extend(cb.failures.iter().cloned());
        }
        if cb.public_api_total > cb.public_api_covered {
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
        if !cold_rebuild_failures.is_empty() {
            blockers.push(format!(
                "cold rebuild failed: {} finding(s)",
                cold_rebuild_failures.len()
            ));
            blockers.extend(cold_rebuild_failures.iter().cloned());
        }
        if stack_migration.incomplete_workspace_count > 0 {
            blockers.push(format!(
                "stack migration classification incomplete: {}/{} workspace(s)",
                stack_migration.incomplete_workspace_count,
                stack_migration.workspaces.len()
            ));
        }
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

        let status = if blockers.is_empty() {
            ProjectHealthStatus::Healthy
        } else {
            ProjectHealthStatus::Blocked
        };
        let production_ready = blockers.is_empty();

        Self {
            project: project.to_string(),
            status,
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
            test_gates,
            managed_percent: managed.percent,
            semantic_percent: semantic.percent,
            regenerable_percent: regenerable.percent,
            next_gap: semantic
                .next_gap
                .as_ref()
                .map(|gap| format!("{} {}", gap.target, gap.primitive)),
            blocked_gap_count: semantic.blocked_gap_count,
            human_decision_required_count: semantic.human_decision_required_count,
            handwrite_files: regenerable.handwrite_files,
            unmarked_files: regenerable.unmarked_files,
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
            optional_regenerability_gaps,
            blockers,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/generate/project-health-source.md#source
impl ProjectTestGateReport {
    pub fn not_evaluated(project: &str) -> Self {
        Self {
            evaluated: false,
            status: ProjectTestGateStatus::NotEvaluated,
            note: Some(format!(
                "test gates not evaluated; run `aw health {project} --verify-tests`"
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
}

pub(crate) fn project_test_gate_report(
    project: &str,
    project_root: &std::path::Path,
    verify_tests: bool,
) -> Result<ProjectTestGateReport> {
    if !verify_tests {
        return Ok(ProjectTestGateReport::not_evaluated(project));
    }

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

fn run_project_test_command(
    workspace: &str,
    command: &str,
    project_root: &std::path::Path,
) -> Result<ProjectTestCommandReport> {
    let started = Instant::now();
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(project_root)
        .output()
        .with_context(|| format!("failed to execute test command `{command}`"))?;
    let duration_ms = started.elapsed().as_millis();
    let status = if output.status.success() {
        ProjectTestCommandStatus::Passed
    } else {
        ProjectTestCommandStatus::Failed
    };
    Ok(ProjectTestCommandReport {
        workspace: workspace.to_string(),
        command: command.to_string(),
        status,
        exit_code: output.status.code(),
        duration_ms,
        stdout_tail: tail_lossy(&output.stdout, 4000),
        stderr_tail: tail_lossy(&output.stderr, 4000),
    })
}

fn tail_lossy(bytes: &[u8], max_chars: usize) -> String {
    let text = String::from_utf8_lossy(bytes);
    let len = text.chars().count();
    if len <= max_chars {
        text.into_owned()
    } else {
        text.chars().skip(len - max_chars).collect()
    }
}

pub async fn run_health(args: ProjectHealthArgs) -> Result<()> {
    let mut report =
        build_health_report_with_options(&args.project, args.verify_cold, args.verify_tests)?;
    let project_root = crate::find_project_root()?;
    let locks = crate::cli::workflow_guard::issue_locks(&project_root).await?;
    report.workflow_lock_count = locks.len();
    if !locks.is_empty() {
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
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_health_report(&report);
    }
    if report.status == ProjectHealthStatus::Blocked {
        std::process::exit(1);
    }
    Ok(())
}

fn print_health_report(report: &ProjectHealthReport) {
    println!("project health: {} ({:?})", report.project, report.status);
    println!(
        "production_ready: {}",
        if report.production_ready { "yes" } else { "no" }
    );
    println!("production_status: {:?}", report.production_status);
    if !report.production_scope.is_empty() {
        println!("production_scope: {}", report.production_scope.join(", "));
    }
    println!(
        "coverage: managed {:.1}%, semantic {:.1}%, regenerable maturity {:.1}%",
        report.managed_percent, report.semantic_percent, report.regenerable_percent
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
        "cb_verify: {}",
        if report.cb_verify_clean {
            "clean"
        } else {
            "blocked"
        }
    );
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
    if !report.blockers.is_empty() {
        println!("blockers:");
        for blocker in &report.blockers {
            println!("  - {blocker}");
        }
    }
}

fn percent_of(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/project.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Source-template promotion for the project health command/report module.
      Replays the issue-2119 implementation without the temporary HANDWRITE wrapper.
```
