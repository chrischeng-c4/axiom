// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
// CODEGEN-BEGIN
//! `aw generator` request surface.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::project::{self, ProjectHealthReport};

#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub struct GeneratorArgs {
    /// Project name from .aw/config.toml.
    #[arg(long, global = true)]
    pub project: Option<String>,
    #[command(subcommand)]
    pub command: GeneratorCommand,
}

#[derive(Debug, Subcommand)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub enum GeneratorCommand {
    /// Inspect generator gaps that can be routed into WI/TD/CB.
    Check(GeneratorCheckArgs),
    /// Create a generator gap WI draft payload after takeover readiness passes.
    Request(GeneratorRequestArgs),
}

#[derive(Debug, Args, Clone)]
#[command(after_help = r#"Output schema (JSON default):
{
  "command": string,
  "project": string,
  "status": "ready" | "blocked",
  "health": {
    "capability_ready": bool,
    "managed_ready": bool,
    "semantic_ready": bool,
    "traceability_ready": bool,
    "takeover_ready": bool,
    "generator_request_ready": bool,
    "production_ready": bool,
    "workflow_lock_count": number
  },
  "gap_count": number,
  "gaps": [{ "id": string, "description": string }],
  "next_action": { "command": string, "reason": string }
}"#)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub struct GeneratorCheckArgs {
    /// DEPRECATED compatibility no-op. Generator emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Pretty-print the JSON output.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Args, Clone)]
#[command(after_help = r#"Output schema (JSON default):
{
  "command": string,
  "project": string,
  "gap_id": string,
  "status": "ready" | "blocked" | "not_found",
  "health": { "takeover_ready": bool, "generator_request_ready": bool, "production_ready": bool, ... },
  "gap": { "id": string, "description": string } | null,
  "blockers": [string],
  "payload_path": string | null,
  "next_action": { "command": string, "reason": string }
}"#)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub struct GeneratorRequestArgs {
    /// Gap id from `aw generator check --project <project>`.
    pub gap_id: String,
    /// DEPRECATED compatibility no-op. Generator emits JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Pretty-print the JSON output.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub struct GeneratorGap {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub struct GeneratorHealthSummary {
    pub capability_ready: bool,
    pub managed_ready: bool,
    pub semantic_ready: bool,
    pub traceability_ready: bool,
    pub takeover_ready: bool,
    pub generator_request_ready: bool,
    pub production_ready: bool,
    pub workflow_lock_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub struct GeneratorNextAction {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub struct GeneratorCheckReport {
    pub command: String,
    pub project: String,
    pub status: String,
    pub health: GeneratorHealthSummary,
    pub gap_count: usize,
    pub gaps: Vec<GeneratorGap>,
    pub next_action: GeneratorNextAction,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub struct GeneratorRequestReport {
    pub command: String,
    pub project: String,
    pub gap_id: String,
    pub status: String,
    pub health: GeneratorHealthSummary,
    pub gap: Option<GeneratorGap>,
    pub blockers: Vec<String>,
    pub payload_path: Option<String>,
    pub next_action: GeneratorNextAction,
}

/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
pub async fn run(args: GeneratorArgs) -> Result<()> {
    let project = args
        .project
        .ok_or_else(|| anyhow::anyhow!("generator requires --project <project>"))?;
    match args.command {
        GeneratorCommand::Check(args) => run_check(&project, args).await,
        GeneratorCommand::Request(args) => run_request(&project, args).await,
    }
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
async fn run_check(project: &str, args: GeneratorCheckArgs) -> Result<()> {
    let health = generator_health_report(project).await?;
    let report = build_check_report(project, &health);
    print_json(&report, args.pretty || args.json)?;
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
async fn run_request(project: &str, args: GeneratorRequestArgs) -> Result<()> {
    let health = generator_health_report(project).await?;
    let (report, should_fail) = build_request_report(project, &args.gap_id, &health)?;
    print_json(&report, args.pretty || args.json)?;
    if should_fail {
        std::process::exit(1);
    }
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
async fn generator_health_report(project: &str) -> Result<ProjectHealthReport> {
    let mut report =
        project::build_health_report_with_options(project, true, false, false, false, false)?;
    project::apply_workflow_locks_to_report(&mut report).await?;
    Ok(report)
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
fn build_check_report(project: &str, health: &ProjectHealthReport) -> GeneratorCheckReport {
    let gaps = generator_gaps(health);
    let status = if health.generator_request_ready {
        "ready"
    } else {
        "blocked"
    }
    .to_string();
    let next_action = if health.generator_request_ready {
        if let Some(gap) = gaps.first() {
            GeneratorNextAction {
                command: format!(
                    "aw generator request --project {} {}",
                    project,
                    shell_quote(&gap.id)
                ),
                reason: "request the next generator gap through WI/TD/CB".to_string(),
            }
        } else {
            GeneratorNextAction {
                command: String::new(),
                reason: "no generator gaps found".to_string(),
            }
        }
    } else {
        GeneratorNextAction {
            command: format!("aw health --project {} --verify-traceability", project),
            reason: "takeover readiness is required before generator gap requests".to_string(),
        }
    };
    GeneratorCheckReport {
        command: format!("aw generator check --project {project}"),
        project: project.to_string(),
        status,
        health: GeneratorHealthSummary::from(health),
        gap_count: gaps.len(),
        gaps,
        next_action,
    }
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
fn build_request_report(
    project: &str,
    gap_id: &str,
    health: &ProjectHealthReport,
) -> Result<(GeneratorRequestReport, bool)> {
    let gaps = generator_gaps(health);
    let gap = gaps.iter().find(|gap| gap.id == gap_id).cloned();
    if !health.generator_request_ready {
        return Ok((
            GeneratorRequestReport {
                command: format!(
                    "aw generator request --project {} {}",
                    project,
                    shell_quote(gap_id)
                ),
                project: project.to_string(),
                gap_id: gap_id.to_string(),
                status: "blocked".to_string(),
                health: GeneratorHealthSummary::from(health),
                gap,
                blockers: takeover_blockers(health),
                payload_path: None,
                next_action: GeneratorNextAction {
                    command: format!("aw health --project {} --verify-traceability", project),
                    reason: "resolve takeover blockers before requesting generator work"
                        .to_string(),
                },
            },
            true,
        ));
    }

    let Some(gap) = gap else {
        return Ok((
            GeneratorRequestReport {
                command: format!(
                    "aw generator request --project {} {}",
                    project,
                    shell_quote(gap_id)
                ),
                project: project.to_string(),
                gap_id: gap_id.to_string(),
                status: "not_found".to_string(),
                health: GeneratorHealthSummary::from(health),
                gap: None,
                blockers: vec![format!(
                    "generator gap `{gap_id}` was not found; run `aw generator check --project {project}`"
                )],
                payload_path: None,
                next_action: GeneratorNextAction {
                    command: format!("aw generator check --project {project}"),
                    reason: "refresh generator gap inventory".to_string(),
                },
            },
            true,
        ));
    };

    let payload_path = generator_request_payload_path(project, gap_id);
    write_request_payload(project, &gap, &payload_path)?;
    let payload = payload_path.to_string_lossy().replace('\\', "/");
    Ok((
        GeneratorRequestReport {
            command: format!(
                "aw generator request --project {} {}",
                project,
                shell_quote(gap_id)
            ),
            project: project.to_string(),
            gap_id: gap_id.to_string(),
            status: "created".to_string(),
            health: GeneratorHealthSummary::from(health),
            gap: Some(gap),
            blockers: Vec::new(),
            payload_path: Some(payload.clone()),
            next_action: GeneratorNextAction {
                command: format!(
                    "aw wi draft init --project {} --type change --title {} --body-file {}",
                    project,
                    shell_quote(&format!("Strengthen generator for {gap_id}")),
                    shell_quote(&payload)
                ),
                reason: "create a WI draft, then continue normal WI/TD/CB lifecycle".to_string(),
            },
        },
        false,
    ))
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
fn generator_gaps(health: &ProjectHealthReport) -> Vec<GeneratorGap> {
    let mut descriptions = health.optional_regenerability_gaps.clone();
    descriptions.extend(health.regenerability_authority.blockers.iter().cloned());
    descriptions.sort();
    descriptions.dedup();
    descriptions
        .into_iter()
        .map(|description| GeneratorGap {
            id: slug_for_path(&description),
            description,
        })
        .collect()
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
fn takeover_blockers(health: &ProjectHealthReport) -> Vec<String> {
    let mut blockers = Vec::new();
    if !health.capability_ready {
        blockers.push("capability readiness is incomplete".to_string());
    }
    if !health.managed_ready {
        blockers.push("managed ownership is incomplete".to_string());
    }
    if !health.semantic_ready {
        blockers.push("semantic TD coverage is incomplete".to_string());
    }
    if !health.traceability_ready {
        blockers.push("traceability closure is incomplete".to_string());
    }
    if health.workflow_lock_count > 0 {
        blockers.push(format!(
            "{} workflow lock(s) are unresolved",
            health.workflow_lock_count
        ));
    }
    if health.blocked_gap_count > 0 {
        blockers.push(format!(
            "{} blocked semantic gap(s)",
            health.blocked_gap_count
        ));
    }
    if health.human_decision_required_count > 0 {
        blockers.push(format!(
            "{} semantic gap(s) require user judgment",
            health.human_decision_required_count
        ));
    }
    if blockers.is_empty() && !health.generator_request_ready {
        blockers.push("generator request readiness is incomplete".to_string());
    }
    blockers
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
impl From<&ProjectHealthReport> for GeneratorHealthSummary {
    fn from(report: &ProjectHealthReport) -> Self {
        Self {
            capability_ready: report.capability_ready,
            managed_ready: report.managed_ready,
            semantic_ready: report.semantic_ready,
            traceability_ready: report.traceability_ready,
            takeover_ready: report.takeover_ready,
            generator_request_ready: report.generator_request_ready,
            production_ready: report.production_ready,
            workflow_lock_count: report.workflow_lock_count,
        }
    }
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
fn generator_request_payload_path(project: &str, gap_id: &str) -> PathBuf {
    crate::shared::workspace::aw_tmp_path()
        .join(slug_for_path(project))
        .join("generator")
        .join("requests")
        .join(format!("{}.md", slug_for_path(gap_id)))
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
fn write_request_payload(project: &str, gap: &GeneratorGap, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let body = format!(
        "# Generator Gap Request\n\n## Project\n{}\n\n## Gap ID\n{}\n\n## Gap\n{}\n\n## Required Flow\nUse normal AW lifecycle: WI -> TD -> CB. Do not add a generator-specific lifecycle command.\n",
        project, gap.id, gap.description
    );
    fs::write(path, body).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
fn print_json<T: Serialize>(value: &T, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
fn slug_for_path(path: &str) -> String {
    let mut out = String::new();
    for ch in path.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "gap".to_string()
    } else {
        trimmed.to_string()
    }
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/generator.md#source
mod tests {
    use super::*;
    use crate::cli::cb::{CbCodegenOriginSummary, CbColdVerifySummary, CbVerifySummary};
    use crate::cli::project::ProjectTestGateReport;
    use crate::cli::regenerability_policy::RegenerabilityAuthority;
    use crate::cli::standardize::{
        CommandTraceabilityCoverage, MarkerCounts, RegenerabilityCoverage, SemanticCoverage,
        StackMigrationCoverage, StandardizationCoverage, TraceabilityCoverage,
    };
    use std::collections::BTreeMap;

    fn health(managed_percent: f64, semantic_percent: f64) -> ProjectHealthReport {
        ProjectHealthReport::from_components_with_traceability(
            "demo",
            StandardizationCoverage {
                scope: vec!["src/**".to_string()],
                total_files: 1,
                managed_files: if managed_percent >= 100.0 { 1 } else { 0 },
                percent: managed_percent,
                by_language: BTreeMap::new(),
                by_marker: MarkerCounts {
                    codegen: 0,
                    handwrite: if managed_percent >= 100.0 { 1 } else { 0 },
                },
                uncovered_files: if managed_percent >= 100.0 {
                    Vec::new()
                } else {
                    vec!["src/lib.rs".to_string()]
                },
            },
            SemanticCoverage {
                scope: vec!["src/**".to_string()],
                total_files: 1,
                source_units: 1,
                source_symbols: 0,
                claim_files: 0,
                semantic_files: if semantic_percent >= 100.0 { 1 } else { 0 },
                semantically_covered_files: if semantic_percent >= 100.0 { 1 } else { 0 },
                percent: semantic_percent,
                source_ir: Vec::new(),
                source_evidence_graph: None,
                frontend_ecosystem: None,
                coverage_map: Vec::new(),
                generator_primitive_gaps: Vec::new(),
                uncovered_files: if semantic_percent >= 100.0 {
                    Vec::new()
                } else {
                    vec!["src/lib.rs".to_string()]
                },
                next_gap: None,
                blocked_gap_count: 0,
                human_decision_required_count: 0,
            },
            TraceabilityCoverage {
                project: "demo".to_string(),
                scope: vec!["src/**".to_string()],
                cap_path: "README.md".to_string(),
                total_td_files: 1,
                traceable_td_files: 1,
                traceability_percent: 100.0,
                internal_td_count: 0,
                orphan_td_count: 0,
                source_edge_count: 1,
                cb_edge_count: 1,
                command_traceability: CommandTraceabilityCoverage::ready_fixture(),
                blocker_count: 0,
                next_gap: None,
                blockers: Vec::new(),
            },
            RegenerabilityCoverage {
                scope: vec!["src/**".to_string()],
                total_files: 1,
                eligible_files: 1,
                codegen_files: 0,
                handwrite_files: 1,
                unmarked_files: 0,
                unsupported_codegen_files: Vec::new(),
                non_replayable_codegen_files: Vec::new(),
                snapshot_codegen_files: Vec::new(),
                codegen_drift_evaluated: false,
                codegen_drift_files: Vec::new(),
                percent: 0.0,
                gap_files: vec!["src/lib.rs".to_string()],
                semantic_percent: 100.0,
                generator_primitive_gaps: 1,
                primitive_covered_files: 0,
                missing_generator_primitive_gaps: 1,
                insufficient_td_section_gaps: 0,
                human_decision_required_gaps: 0,
                next_gap: None,
                authority_mode: RegenerabilityAuthority::ExternalAdvisory,
                required_for_production: false,
                authority_reason: "test fixture".to_string(),
            },
            StackMigrationCoverage {
                project: "demo".to_string(),
                workspaces: Vec::new(),
                migration_normalized_percent: 100.0,
                incomplete_workspace_count: 0,
                dependency_policy_blockers: Vec::new(),
                deployment_policy_blockers: Vec::new(),
                blockers: Vec::new(),
            },
            CbVerifySummary {
                clean: true,
                public_api_covered: 1,
                public_api_total: 1,
                semantic_review_required: 0,
                failures: Vec::new(),
            },
            vec![CbColdVerifySummary {
                workspace: Some("demo".to_string()),
                clean: true,
                spec_count: 1,
                source_root_count: 1,
                generated_files: 1,
                expected_files: 1,
                codegen_origin: CbCodegenOriginSummary {
                    target_files: 1,
                    td_ast_files: 1,
                    artifact_replay_files: 0,
                    source_template_files: 0,
                },
                failures: Vec::new(),
            }],
            ProjectTestGateReport::passed_fixture("true"),
        )
    }

    #[test]
    fn generator_request_blocks_before_takeover_ready() {
        let report = health(0.0, 100.0);
        let (request, should_fail) =
            build_request_report("demo", "missing-generator-src-lib-rs", &report).unwrap();
        assert!(should_fail);
        assert_eq!(request.status, "blocked");
        assert!(!request.health.generator_request_ready);
        assert!(request
            .blockers
            .iter()
            .any(|blocker| blocker.contains("managed ownership")));
    }

    #[test]
    fn generator_request_creates_payload_after_takeover_ready() {
        let report = health(100.0, 100.0);
        let check = build_check_report("demo", &report);
        assert_eq!(check.status, "ready");
        assert!(check.gap_count >= 1);
        let gap_id = check.gaps[0].id.clone();
        let (request, should_fail) = build_request_report("demo", &gap_id, &report).unwrap();
        assert!(!should_fail);
        assert_eq!(request.status, "created");
        assert!(request.payload_path.is_some());
        assert!(request
            .next_action
            .command
            .contains("aw wi draft init --project demo --type change"));
    }
}

// CODEGEN-END
