// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/cb_review.md#source
// CODEGEN-BEGIN
//! `aw td code-review` — third-of-four code-artifact CRRR verbs.
//!
//! Brief mode dispatches `score-cb-reviewer` with the list of slug-introduced
//! files and the filled HANDWRITE markers within them. Apply mode reads
//! `.aw/payloads/<slug>/cb_review.md`, validates the verdict, commits a
//! `Lifecycle-Stage: Cb-Review` trailer, advances phase to `cb_reviewed`, and
//! emits the next dispatch (td merge / code revise / code arbitrate).
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli

use std::collections::HashSet;
use std::path::Path;

use crate::issues::{IssueBackend, IssuePatch, LocalBackend};
use anyhow::{Context, Result};
use clap::Args;

use crate::cli::cb_fill::{branch_changed_files, enumerate_worktree_markers};
use crate::cli::remote_push::maybe_push_remote;
use crate::models::artifact_quality::{infer_artifact_kind_from_hint, ArtifactQualityProfile};
use crate::models::preflight::{
    default_preflight_gates, PreFlightGateReport, PreFlightGateSeverity, PreFlightGateStatus,
};

// Args for `aw td code-review <slug>`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
#[derive(Debug, Args)]
pub struct CbReviewArgs {
    /// Issue slug identifying the current checkout branch.
    pub slug: String,
    /// Apply mode: merge `.aw/payloads/<slug>/cb_review.md`, commit
    /// `Lifecycle-Stage: Cb-Review`, advance phase, dispatch next verb.
    #[arg(long)]
    pub apply: bool,
    /// Emit envelope as JSON.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Pretty-print the JSON envelope.
    #[arg(long)]
    pub pretty: bool,
}

// Top-level dispatch.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_review.md#source
pub async fn run_review(args: CbReviewArgs) -> Result<()> {
    if args.apply {
        run_review_apply(args).await
    } else {
        run_review_brief(args).await
    }
}

fn worktree_path(slug: &str) -> Result<std::path::PathBuf> {
    let project_root = crate::find_project_root()?;
    let payload_rel = cb_review_payload_rel(slug);
    crate::cli::td::td_activate_inplace_allowing_dirty_lifecycle_paths(
        &project_root,
        slug,
        &[payload_rel.as_str()],
    )?;
    let path = crate::cli::td::td_workspace_path(&project_root, slug);
    if !path.exists() {
        anyhow::bail!("workspace not found: {}", path.display());
    }
    Ok(path)
}

fn cb_review_payload_rel(slug: &str) -> String {
    format!(".aw/payloads/{}/cb_review.md", slug)
}

fn cb_review_apply_command(slug: &str) -> String {
    format!("aw td code-review {} --apply", slug)
}

fn cb_revise_command(slug: &str) -> String {
    format!("aw td code-revise {}", slug)
}

fn cb_arbitrate_command(slug: &str) -> String {
    format!("aw td code-arbitrate {}", slug)
}

fn td_merge_command(slug: &str, spec_path: &str) -> String {
    if spec_path.is_empty() {
        format!("aw td merge {}", slug)
    } else {
        format!("aw td merge {} --spec-path {}", slug, spec_path)
    }
}

fn next_for_review_apply(slug: &str, payload_path: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": cb_review_apply_command(slug),
        "reason": "complete the CB review payload and apply it",
        "requires_hitl": false,
        "payload_path": payload_path,
    })
}

fn next_for_td_merge(slug: &str, spec_path: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": td_merge_command(slug, spec_path),
        "reason": "CB review approved the implementation",
        "requires_hitl": false,
        "payload_path": null,
    })
}

fn next_for_cb_revise(slug: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": cb_revise_command(slug),
        "reason": "CB review requested implementation revision",
        "requires_hitl": false,
        "payload_path": null,
    })
}

fn next_for_cb_arbitrate(slug: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": cb_arbitrate_command(slug),
        "reason": "CB review needs arbitration after repeated revision",
        "requires_hitl": false,
        "payload_path": null,
    })
}

fn print_json(value: &serde_json::Value, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}

fn emit_error(slug: &str, message: &str, pretty: bool) -> Result<()> {
    let env = serde_json::json!({
        "action": "error",
        "slug": slug,
        "message": message,
        "next": {
            "kind": "none",
            "command": null,
            "reason": message,
            "requires_hitl": false,
            "payload_path": null,
        },
    });
    print_json(&env, pretty)
}

fn cb_review_payload_template(
    round: u8,
    code_paths: &[String],
    unfilled_markers: &[String],
    spec_path: &str,
    artifact_quality_contracts: &[serde_json::Value],
) -> Result<String> {
    let code_paths_text = if code_paths.is_empty() {
        "- (none)\n".to_string()
    } else {
        code_paths
            .iter()
            .map(|path| format!("- {}\n", path))
            .collect::<String>()
    };
    let marker_text = if unfilled_markers.is_empty() {
        "- (none)\n".to_string()
    } else {
        unfilled_markers
            .iter()
            .map(|marker| format!("- [{}] (fill)\n", marker))
            .collect::<String>()
    };
    let hard_patterns = serde_json::to_string_pretty(&completeness_hard_patterns_json())?;
    let artifact_contracts = serde_json::to_string_pretty(artifact_quality_contracts)?;

    Ok(format!(
        "# CB Review {round}\n\n\
         **Verdict:** <approved|needs-revision>\n\n\
         ## Scope\n\n\
         Spec: {spec_path}\n\n\
         Code paths:\n{code_paths_text}\n\
         Unfilled markers:\n{marker_text}\n\
         ## Completeness Review\n\n\
         Instructions:\n{completeness_instructions}\
         Hard patterns:\n\n```json\n{hard_patterns}\n```\n\n\
         ## Artifact Quality Review\n\n\
         Instructions:\n{artifact_instructions}\
         Contracts:\n\n```json\n{artifact_contracts}\n```\n\n\
         ## Findings\n\n\
         For needs-revision, add one finding per marker or preflight gate:\n\
         - [marker-id] (fill)\n",
        completeness_instructions = completeness_review_instructions()
            .iter()
            .map(|item| format!("- {}\n", item))
            .collect::<String>(),
        artifact_instructions = artifact_quality_review_instructions()
            .iter()
            .map(|item| format!("- {}\n", item))
            .collect::<String>(),
    ))
}

fn initialize_cb_review_payload(
    worktree: &Path,
    slug: &str,
    template: &str,
) -> Result<(String, bool)> {
    let rel = cb_review_payload_rel(slug);
    let abs = worktree.join(&rel);
    if abs.exists() {
        return Ok((rel, false));
    }
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create payload directory {}", parent.display()))?;
    }
    std::fs::write(&abs, template)
        .with_context(|| format!("failed to write payload {}", abs.display()))?;
    Ok((rel, true))
}

// Brief mode: list slug-introduced files + filled markers, emit dispatch.
async fn run_review_brief(args: CbReviewArgs) -> Result<()> {
    let slug = args.slug.clone();
    let worktree = worktree_path(&slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let issue = backend
        .get(&slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    if phase != "cb_filled" && phase != "cb_revised" {
        emit_error(
            &slug,
            &format!(
                "phase '{}' is not eligible for cb review (expected cb_filled or cb_revised)",
                phase
            ),
            args.pretty,
        )?;
        std::process::exit(2);
    }

    let base_branch =
        std::env::var("SCORE_CB_FILL_BASE_BRANCH").unwrap_or_else(|_| "main".to_string());
    let changed: HashSet<String> = branch_changed_files(&worktree, &base_branch);

    // List filled markers in slug-introduced files (post-fill, so this should
    // be empty if the slug introduced markers and they were all filled). We
    // surface code_paths so the reviewer agent can read what changed.
    let all_markers = enumerate_worktree_markers(&worktree);
    let unfilled_in_slug: Vec<String> = all_markers
        .iter()
        .filter(|m| changed.contains(m.source_path.as_str()))
        .map(|m| m.id.clone())
        .collect();

    let spec_path = issue
        .implements
        .iter()
        .find(|s| s.ends_with(".md"))
        .cloned()
        .unwrap_or_default();

    let mut code_paths = changed.into_iter().collect::<Vec<_>>();
    code_paths.sort();
    let artifact_quality_contracts: Vec<serde_json::Value> = code_paths
        .iter()
        .map(|path| artifact_quality_review_contract(path))
        .collect();

    let round = issue.review_count.unwrap_or(0) + 1;
    let payload_template = cb_review_payload_template(
        round,
        &code_paths,
        &unfilled_in_slug,
        &spec_path,
        &artifact_quality_contracts,
    )?;
    let (payload_path, payload_created) =
        initialize_cb_review_payload(&worktree, &slug, &payload_template)?;
    let env = serde_json::json!({
        "action": "dispatch",
        "agent": null,
        "slug": slug,
        "next": next_for_review_apply(&slug, &payload_path),
        "payload_initialized": payload_created,
        "invoke": {
            "command": "aw td code-review",
            "args": {
                "slug": slug,
                "round": round,
                "code_paths": code_paths,
                "unfilled_markers": unfilled_in_slug,
                "spec_path": spec_path,
                "payload_path": payload_path,
            },
        },
    });
    print_json(&env, args.pretty)?;
    let _ = args.json;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Approved,
    NeedsRevision,
}

fn parse_verdict(text: &str) -> Option<Verdict> {
    if text.contains("**Verdict:** approved")
        || text.contains("**Verdict**: approved")
        || text.contains("Verdict: approved")
    {
        Some(Verdict::Approved)
    } else if text.contains("**Verdict:** needs-revision")
        || text.contains("**Verdict**: needs-revision")
        || text.contains("Verdict: needs-revision")
    {
        Some(Verdict::NeedsRevision)
    } else {
        None
    }
}

// Extract flagged marker IDs from review findings. Format is `- [<marker-id>]
// <finding>` mirroring the TD review shape.
///
// Public so `cb_revise` can read the same `cb_review.md` payload.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_review.md#source
pub fn extract_flagged_markers_from(text: &str) -> Vec<String> {
    extract_flagged_markers(text)
}

fn extract_flagged_markers(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        let rest = if let Some(r) = trimmed.strip_prefix("- [") {
            r
        } else if let Some(r) = trimmed.strip_prefix("* [") {
            r
        } else {
            continue;
        };
        if let Some(close) = rest.find(']') {
            let id = rest[..close].trim().to_string();
            if !id.is_empty() && !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-preflight-gates.md#cli
#[allow(dead_code)]
fn preflight_validation_findings(report: &PreFlightGateReport) -> Vec<String> {
    report
        .results
        .iter()
        .filter(|result| {
            result.severity == PreFlightGateSeverity::Hard
                && matches!(
                    result.status,
                    PreFlightGateStatus::Missing | PreFlightGateStatus::Failed
                )
        })
        .map(|result| {
            format!(
                "- [preflight:{}] missing required pre-flight evidence for {}",
                result.gate_id, report.artifact_ref
            )
        })
        .collect()
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-preflight-gates.md#cli
fn artifact_quality_review_contract(artifact_ref: &str) -> serde_json::Value {
    let artifact_kind = infer_artifact_kind_from_hint(artifact_ref);
    let profile = ArtifactQualityProfile::default_for_kind(artifact_kind);
    let gates = default_preflight_gates(artifact_kind);
    let missing_report = PreFlightGateReport::evaluate(artifact_ref, &gates, &[]);
    let missing_hard_findings = preflight_validation_findings(&missing_report);

    serde_json::json!({
        "artifact_ref": artifact_ref,
        "artifact_quality_profile": profile,
        "required_preflight_gates": gates,
        "missing_hard_findings": missing_hard_findings,
        "production_blockers": missing_report.production_blockers,
    })
}

fn artifact_quality_review_instructions() -> Vec<&'static str> {
    vec![
        "Apply each artifact_quality_profile to the matching code path before approving the CB.",
        "Accept hard preflight evidence only when it is machine-verifiable: command transcript, test result, TD unit-test/e2e-test section, CLI/API transcript, or screenshot artifact path with viewport and command source.",
        "For frontend/UI artifacts, require desktop and mobile viewport evidence, interaction smoke proof, accessibility/readability smoke proof, and placeholder-free primary-state evidence.",
        "If any hard preflight gate is missing or failed, use a needs-revision verdict and include the corresponding [preflight:<gate-id>] finding.",
    ]
}

const COMPLETENESS_PLACEHOLDER_PATTERNS: &[(&str, &str)] = &[
    ("todo: hand-write content", "placeholder_artifact"),
    ("todo implement", "placeholder_artifact"),
    ("todo: implement", "placeholder_artifact"),
    ("todo - implement", "placeholder_artifact"),
    ("rest omitted", "omitted_content"),
    ("similar pattern omitted", "omitted_content"),
    ("omitted for brevity", "omitted_content"),
    ("implementation omitted", "omitted_content"),
];

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletenessFinding {
    code: &'static str,
    artifact_ref: String,
    line: usize,
    pattern: String,
    message: String,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-completeness-placeholder-gate.md#logic
#[allow(dead_code)]
fn completeness_placeholder_findings(
    artifact_ref: &str,
    artifact_text: &str,
) -> Vec<CompletenessFinding> {
    let mut findings = Vec::new();

    for (idx, line) in artifact_text.lines().enumerate() {
        if is_completeness_allowed_line(line) {
            continue;
        }

        let trimmed = line.trim();
        if trimmed == "..." {
            findings.push(completeness_finding(
                "omitted_content",
                artifact_ref,
                idx + 1,
                "...",
            ));
            continue;
        }

        let lowered = trimmed.to_ascii_lowercase();
        for (pattern, code) in COMPLETENESS_PLACEHOLDER_PATTERNS {
            if lowered.contains(pattern) {
                findings.push(completeness_finding(code, artifact_ref, idx + 1, pattern));
                break;
            }
        }
    }

    findings
}

#[allow(dead_code)]
fn completeness_finding(
    code: &'static str,
    artifact_ref: &str,
    line: usize,
    pattern: &str,
) -> CompletenessFinding {
    CompletenessFinding {
        code,
        artifact_ref: artifact_ref.to_string(),
        line,
        pattern: pattern.to_string(),
        message: format!(
            "{} line {} contains incomplete placeholder pattern `{}`",
            artifact_ref, line, pattern
        ),
    }
}

#[allow(dead_code)]
fn is_completeness_allowed_line(line: &str) -> bool {
    let lowered = line.to_ascii_lowercase();
    completeness_allowed_markers()
        .iter()
        .any(|marker| lowered.contains(&marker.to_ascii_lowercase()))
}

fn completeness_allowed_markers() -> Vec<String> {
    vec![
        ["HANDWRITE", "BEGIN"].join("-"),
        ["HANDWRITE", "END"].join("-"),
        "generator-gap".to_string(),
        "future_work_allowed".to_string(),
    ]
}

fn completeness_review_instructions() -> Vec<&'static str> {
    vec![
        "Count expected deliverables from the TD changes section and compare them with produced artifacts.",
        "Reject artifacts that contain placeholder code, skeleton prose, ellipsis-only truncation, or omitted-content wording.",
        "Allow AW ownership markers only when they are explicit HANDWRITE markers, generator-gap annotations, or future_work_allowed notes.",
    ]
}

fn completeness_hard_patterns_json() -> Vec<serde_json::Value> {
    COMPLETENESS_PLACEHOLDER_PATTERNS
        .iter()
        .map(|(pattern, code)| {
            serde_json::json!({
                "pattern": pattern,
                "code": code,
            })
        })
        .chain(std::iter::once(serde_json::json!({
            "pattern": "...",
            "code": "omitted_content",
        })))
        .collect()
}

// Public wrapper for `cb_revise` to commit `Lifecycle-Stage: Cb-Revise`.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_review.md#source
pub fn stage_and_commit_for_revise(
    worktree: &Path,
    slug: &str,
    trailer: &str,
    detail: &str,
    paths: &[&str],
) -> Result<()> {
    stage_and_commit(worktree, slug, trailer, detail, paths)
}

fn stage_and_commit(
    worktree: &Path,
    slug: &str,
    trailer: &str,
    detail: &str,
    paths: &[&str],
) -> Result<()> {
    let git_bin =
        crate::git::find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found"))?;
    for p in paths {
        let path = Path::new(p);
        if path.is_absolute() && !path.starts_with(worktree) {
            continue;
        }
        let _ = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(worktree)
            .args(["add", p])
            .output();
    }
    let msg = format!(
        "{trailer_kebab}({slug}) — {detail}\n\nWork-Item: {slug}\nLifecycle-Stage: {trailer}\n",
        trailer_kebab = trailer.to_lowercase(),
        slug = slug,
        detail = detail,
        trailer = trailer,
    );
    let out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree)
        .args(["commit", "--allow-empty", "-m", &msg])
        .output()
        .context("git commit failed")?;
    if !out.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

// Apply mode: validate review payload, commit trailer, dispatch next verb.
async fn run_review_apply(args: CbReviewArgs) -> Result<()> {
    let slug = args.slug.clone();
    let worktree = worktree_path(&slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let issue = backend
        .get(&slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    let payload_rel = cb_review_payload_rel(&slug);
    let payload_abs = worktree.join(&payload_rel);
    let payload = std::fs::read_to_string(&payload_abs).with_context(|| {
        format!(
            "payload not readable at {}: write the review with verdict + flagged markers first",
            payload_abs.display()
        )
    })?;

    let verdict = match parse_verdict(&payload) {
        Some(v) => v,
        None => {
            emit_error(
                &slug,
                "review payload missing verdict line — expected '**Verdict:** approved' or '**Verdict:** needs-revision'",
                args.pretty,
            )?;
            std::process::exit(1);
        }
    };

    let flagged = extract_flagged_markers(&payload);
    if matches!(verdict, Verdict::NeedsRevision) && flagged.is_empty() {
        emit_error(
            &slug,
            "needs-revision verdict requires at least one [marker-id] finding",
            args.pretty,
        )?;
        std::process::exit(1);
    }

    // Snapshot the payload into the issue body's `# Reviews` section by
    // appending the file (parallels TD-side reviewer template).
    let issue_abs = backend.issue_path(&issue);
    let issue_path_s = issue_abs.to_string_lossy().into_owned();
    if issue_abs.exists() {
        let mut body = std::fs::read_to_string(&issue_abs)?;
        if !body.contains("# Reviews") {
            body.push_str("\n# Reviews\n");
        }
        let new_count = issue.review_count.unwrap_or(0) + 1;
        body.push_str(&format!("\n## Cb-Review {}\n\n", new_count));
        body.push_str(&payload);
        if !body.ends_with('\n') {
            body.push('\n');
        }
        std::fs::write(&issue_abs, body)?;
    }

    let new_count = issue.review_count.unwrap_or(0) + 1;
    let detail = match verdict {
        Verdict::Approved => format!("approved (cb-review #{})", new_count),
        Verdict::NeedsRevision => {
            format!("needs-revision (cb-review #{})", new_count)
        }
    };

    let patch = IssuePatch {
        phase: Some(crate::issues::types::td_phase::CB_REVIEWED.to_string()),
        review_count: Some(new_count),
        ..Default::default()
    };
    backend.update(&slug, &patch).await?;

    maybe_push_remote(&worktree, &issue_abs, &slug).await?;

    stage_and_commit(
        &worktree,
        &slug,
        crate::issues::types::lifecycle_trailer::CB_REVIEW,
        &detail,
        &[issue_path_s.as_str()],
    )?;

    let spec_path = issue
        .implements
        .iter()
        .find(|s| s.ends_with(".md"))
        .cloned()
        .unwrap_or_default();

    // Routing
    let env = match verdict {
        Verdict::Approved => serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": slug,
            "next": next_for_td_merge(&slug, &spec_path),
            "invoke": {
                "command": "aw td merge",
                "args": { "slug": slug, "spec_path": spec_path },
            },
        }),
        Verdict::NeedsRevision if new_count < 2 => serde_json::json!({
            "action": "dispatch",
            "agent": null,
            "slug": slug,
            "next": next_for_cb_revise(&slug),
            "invoke": {
                "command": "aw td code-revise",
                "args": { "slug": slug, "flagged_markers": flagged },
            },
        }),
        Verdict::NeedsRevision => serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": slug,
            "next": next_for_cb_arbitrate(&slug),
            "invoke": {
                "command": "aw td code-arbitrate",
                "args": { "slug": slug },
            },
        }),
    };
    print_json(&env, args.pretty)?;
    let _ = args.json;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::preflight::{PreFlightGateResult, PreFlightGateStatus};

    #[test]
    fn preflight_review_findings_report_missing_hard_evidence() {
        let report = PreFlightGateReport {
            artifact_ref: "projects/demo/src/lib.rs".to_string(),
            results: vec![PreFlightGateResult {
                gate_id: "code-artifact-test".to_string(),
                severity: PreFlightGateSeverity::Hard,
                status: PreFlightGateStatus::Missing,
                evidence_ref: None,
            }],
            production_blockers: vec![
                "pre-flight gate code-artifact-test missing test evidence".to_string()
            ],
            quality_warnings: Vec::new(),
        };

        assert_eq!(
            preflight_validation_findings(&report),
            vec![
                "- [preflight:code-artifact-test] missing required pre-flight evidence for projects/demo/src/lib.rs"
                    .to_string()
            ]
        );
    }

    #[test]
    fn frontend_artifact_quality_contract_requires_ui_evidence() {
        let contract = artifact_quality_review_contract("projects/demo/frontend/src/App.tsx");
        let text = serde_json::to_string(&contract).unwrap();

        assert_eq!(
            contract["artifact_quality_profile"]["artifact_kind"],
            "frontend_page"
        );
        assert!(text.contains("frontend-page-viewport-screenshots"));
        assert!(text.contains("frontend-page-interaction-smoke"));
        assert!(text.contains("frontend-page-accessibility-readability"));
        assert!(text.contains("- [preflight:frontend-page-viewport-screenshots]"));
    }

    #[test]
    fn cli_artifact_quality_contract_does_not_require_frontend_evidence() {
        let contract = artifact_quality_review_contract("projects/demo/src/cli/run.rs");
        let text = serde_json::to_string(&contract).unwrap();

        assert_eq!(
            contract["artifact_quality_profile"]["artifact_kind"],
            "cli_surface"
        );
        assert!(text.contains("cli-surface-transcript"));
        assert!(!text.contains("frontend-page-viewport-screenshots"));
    }

    #[test]
    fn completeness_placeholder_code_rejected() {
        let findings = completeness_placeholder_findings(
            "projects/demo/src/lib.rs",
            "fn run() {\n    // TODO implement parser\n}\n",
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "placeholder_artifact");
        assert_eq!(findings[0].line, 2);
        assert_eq!(findings[0].pattern, "todo implement");
    }

    #[test]
    fn completeness_placeholder_omitted_prose_rejected() {
        let findings = completeness_placeholder_findings(
            "projects/demo/README.md",
            "Repeat the same workflow for the remaining commands; similar pattern omitted.\n",
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "omitted_content");
        assert_eq!(findings[0].pattern, "similar pattern omitted");
    }

    #[test]
    fn completeness_placeholder_allowed_future_todo_passes() {
        let findings = completeness_placeholder_findings(
            "projects/demo/src/lib.rs",
            "// future_work_allowed TODO implement optional telemetry later\n",
        );

        assert!(findings.is_empty());
    }

    #[test]
    fn cb_review_next_command_omits_legacy_json() {
        let next = next_for_review_apply("4124", ".aw/payloads/4124/cb_review.md");

        assert_eq!(next["command"], "aw td code-review 4124 --apply");
        assert!(!next["command"].as_str().unwrap().contains("--json"));
        assert_eq!(next["payload_path"], ".aw/payloads/4124/cb_review.md");
    }

    #[test]
    fn cb_review_payload_template_requires_agent_edit() {
        let code_paths = vec!["src/demo.rs".to_string()];
        let markers = vec!["missing-demo".to_string()];
        let contracts = vec![serde_json::json!({"artifact_ref": "src/demo.rs"})];
        let template =
            cb_review_payload_template(1, &code_paths, &markers, "td/demo.md", &contracts).unwrap();

        assert!(template.contains("**Verdict:** <approved|needs-revision>"));
        assert!(template.contains("- [missing-demo] (fill)"));
        assert!(template.contains("```json"));
        assert_eq!(parse_verdict(&template), None);
    }

    #[test]
    fn initialize_cb_review_payload_preserves_existing_content() {
        let tmp = tempfile::tempdir().unwrap();

        let (rel, created) = initialize_cb_review_payload(tmp.path(), "4124", "first\n").unwrap();
        assert!(created);
        assert_eq!(rel, ".aw/payloads/4124/cb_review.md");

        let abs = tmp.path().join(&rel);
        std::fs::write(&abs, "custom\n").unwrap();
        let (_, created_again) =
            initialize_cb_review_payload(tmp.path(), "4124", "second\n").unwrap();
        assert!(!created_again);
        assert_eq!(std::fs::read_to_string(abs).unwrap(), "custom\n");
    }
}

// CODEGEN-END
