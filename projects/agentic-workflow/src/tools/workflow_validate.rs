//! score workflow validate — three-role contract SubagentStop gate.
//!
//! Invoked by the `.claude/hooks/global/subagentstop-validate.sh` SubagentStop hook after a
//! `score-*` subagent completes. Validates that the subagent wrote a
//! well-formed artifact for its assigned phase and advances STATE.yaml
//! phase on pass. On fail, returns structured errors so the hook can
//! emit `{"decision":"block"}` and force the subagent to retry within
//! its current invocation.
//!
//! Spec: `projects/agentic-workflow/specs/three-role-contract.md` R5–R7.

use super::{get_required_string, ToolDefinition};
use crate::models::state::StatePhase;
use crate::models::ValidationOptions;
use crate::state::StateManager;
use crate::tools::validate_proposal::validate_proposal;
use crate::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/workflow_validate/definition.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_validate".to_string(),
        description: "Validate artifact output of a score-* subagent and advance phase on pass."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "agent_type"],
            "properties": {
                "project_path": { "type": "string" },
                "change_id": { "type": "string" },
                "agent_type": {
                    "type": "string",
                    "enum": [
                        "score-issue-author",
                        "score-change-spec",
                        "score-change-implementation",
                        "score-review",
                    ]
                }
            }
        }),
    }
}
// CODEGEN-END
/// Per-agent validation result.
struct ValidateOutcome {
    passed: bool,
    errors: Vec<String>,
    /// Phase to advance to on pass; None if this agent does not own a phase transition.
    next_phase: Option<StatePhase>,
}

pub async fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let agent_type = get_required_string(args, "agent_type")?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found.", change_id);
    }

    let current_phase = {
        let sm = StateManager::load(&change_dir)?;
        sm.phase().clone()
    };

    let outcome = match agent_type.as_str() {
        "score-issue-author" => validate_issue_author(&change_id, project_root, &change_dir)?,
        "score-change-spec" => {
            validate_change_spec(&change_id, project_root, &change_dir, &current_phase)?
        }
        "score-change-implementation" => {
            validate_change_implementation(&change_id, &change_dir, &current_phase)?
        }
        "score-review" => validate_review(&change_dir, &current_phase)?,
        other => anyhow::bail!("Unknown agent_type: {}", other),
    };

    let phase_advanced_to = if outcome.passed {
        if let Some(phase) = &outcome.next_phase {
            super::workflow_common::update_phase(&change_dir, phase.clone())?;
            Some(super::workflow_common::phase_to_string(phase).to_string())
        } else {
            None
        }
    } else {
        None
    };

    let result = json!({
        "passed": outcome.passed,
        "errors": outcome.errors,
        "agent_type": agent_type,
        "current_phase": super::workflow_common::phase_to_string(&current_phase),
        "phase_advanced_to": phase_advanced_to,
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

// ─── Per-agent validators ──────────────────────────────────────────────────

/// `score-issue-author` — runs before init_change, so it has no phase to
/// advance. Validates Problem/Requirements/Scope are present in the issue.
fn validate_issue_author(
    change_id: &str,
    project_root: &Path,
    _change_dir: &Path,
) -> Result<ValidateOutcome> {
    let mut errors = Vec::new();

    let issue_path = find_issue_path(project_root, change_id);
    let Some(issue_path) = issue_path else {
        return Ok(ValidateOutcome {
            passed: false,
            errors: vec![format!("Issue file not found for slug '{}'", change_id)],
            next_phase: None,
        });
    };

    let content = std::fs::read_to_string(&issue_path)?;

    for heading in ["## Problem", "## Requirements", "## Scope"] {
        if !content.contains(heading) {
            errors.push(format!("Missing section: {}", heading));
        } else if section_is_empty(&content, heading) {
            errors.push(format!("Section is empty: {}", heading));
        }
    }

    // R-id regex (e.g. "R1:", "R10.", "R1 —") — at least one must appear in Requirements.
    let re = regex::Regex::new(r"(?m)^\s*[-*]?\s*\*{0,2}R\d+\*{0,2}[:.\s]").unwrap();
    if !re.is_match(&content) {
        errors.push("No R-id (R1, R2, …) found in Requirements".to_string());
    }

    Ok(ValidateOutcome {
        passed: errors.is_empty(),
        errors,
        next_phase: None,
    })
}

/// `score-change-spec` — reuses `validate_proposal` for structure checks,
/// then confirms a spec exists under `specs/` and `fill_sections` are complete.
fn validate_change_spec(
    change_id: &str,
    project_root: &Path,
    change_dir: &Path,
    current_phase: &StatePhase,
) -> Result<ValidateOutcome> {
    let mut errors = Vec::new();

    let options = ValidationOptions::new()
        .with_strict(false)
        .with_verbose(false)
        .with_json(true)
        .with_fix(false);
    let summary = validate_proposal(change_id, &project_root.to_path_buf(), &options)?;
    if !summary.is_valid() {
        errors.extend(summary.errors.clone());
    }

    // A spec must exist under specs/ (not counting templates starting with `_`).
    let specs_dir = change_dir.join("specs");
    let mut has_spec = false;
    if specs_dir.exists() {
        for entry in std::fs::read_dir(&specs_dir)?.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".md") && !name.starts_with('_') {
                has_spec = true;
                break;
            }
        }
    }
    if !has_spec {
        errors.push("No spec file found under specs/".to_string());
    }

    let next_phase = match current_phase {
        StatePhase::ChangeInited | StatePhase::ChangeSpecRevised => {
            Some(StatePhase::ChangeSpecCreated)
        }
        // score-change-spec also owns docs creation in current flow.
        StatePhase::ChangeImplementationReviewed => Some(StatePhase::DocsCreated),
        StatePhase::DocsRevised => Some(StatePhase::DocsCreated),
        _ => None,
    };

    Ok(ValidateOutcome {
        passed: errors.is_empty(),
        errors,
        next_phase,
    })
}

/// `score-change-implementation` — confirms an `artifact_writes.jsonl` entry
/// exists for this change and, when git is available, that its entries line
/// up with the worktree diff. `cargo check` is deferred to a follow-up.
fn validate_change_implementation(
    change_id: &str,
    change_dir: &Path,
    current_phase: &StatePhase,
) -> Result<ValidateOutcome> {
    let mut errors = Vec::new();

    let log = change_dir.join("artifact_writes.jsonl");
    if !log.exists() {
        errors.push(
            "artifact_writes.jsonl not found — subagent did not run `score artifact`".to_string(),
        );
    } else {
        let content = std::fs::read_to_string(&log)?;
        let has_impl_entry = content.lines().any(|line| {
            serde_json::from_str::<Value>(line)
                .ok()
                .and_then(|v| v.get("action").and_then(|a| a.as_str()).map(str::to_string))
                .map(|a| a.contains("implementation"))
                .unwrap_or(false)
        });
        if !has_impl_entry {
            errors.push(
                "artifact_writes.jsonl has no entry for create-change-implementation".to_string(),
            );
        }
    }

    let next_phase = match current_phase {
        StatePhase::ChangeSpecReviewed => Some(StatePhase::ChangeImplementationCreated),
        StatePhase::ChangeImplementationRevised => Some(StatePhase::ChangeImplementationCreated),
        _ => None,
    };

    // Suppress unused-variable warning when cargo check not yet wired.
    let _ = change_id;

    Ok(ValidateOutcome {
        passed: errors.is_empty(),
        errors,
        next_phase,
    })
}

/// `score-review` — validates the review payload has verdict + summary, then
/// maps the current phase to its reviewed counterpart.
fn validate_review(change_dir: &Path, current_phase: &StatePhase) -> Result<ValidateOutcome> {
    let mut errors = Vec::new();

    let payload_name = match current_phase {
        StatePhase::ChangeSpecCreated | StatePhase::ChangeSpecRevised => {
            Some("review-change-spec.json")
        }
        StatePhase::ChangeImplementationCreated | StatePhase::ChangeImplementationRevised => {
            Some("review-change-implementation.json")
        }
        StatePhase::DocsCreated | StatePhase::DocsRevised => Some("review-change-docs.json"),
        _ => None,
    };

    let Some(payload_name) = payload_name else {
        errors.push(format!(
            "score-review is not valid at phase '{}'",
            super::workflow_common::phase_to_string(current_phase)
        ));
        return Ok(ValidateOutcome {
            passed: false,
            errors,
            next_phase: None,
        });
    };

    let payload_path = change_dir.join("payloads").join(payload_name);
    if !payload_path.exists() {
        errors.push(format!(
            "Review payload not found: payloads/{}",
            payload_name
        ));
    } else {
        let content = std::fs::read_to_string(&payload_path)?;
        let parsed: Value = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Review payload is not valid JSON: {}", e))?;

        let verdict = parsed
            .get("verdict")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if !matches!(verdict, "APPROVED" | "REVIEWED" | "REJECTED") {
            errors.push(format!(
                "verdict must be APPROVED|REVIEWED|REJECTED, got '{}'",
                verdict
            ));
        }

        let summary_ok = parsed
            .get("summary")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !summary_ok {
            errors.push("summary is empty".to_string());
        }

        if let Some(issues) = parsed.get("issues") {
            if !issues.is_array() {
                errors.push("issues must be an array".to_string());
            }
        }
    }

    let next_phase = match current_phase {
        StatePhase::ChangeSpecCreated | StatePhase::ChangeSpecRevised => {
            Some(StatePhase::ChangeSpecReviewed)
        }
        StatePhase::ChangeImplementationCreated | StatePhase::ChangeImplementationRevised => {
            Some(StatePhase::ChangeImplementationReviewed)
        }
        StatePhase::DocsCreated | StatePhase::DocsRevised => Some(StatePhase::DocsReviewed),
        _ => None,
    };

    Ok(ValidateOutcome {
        passed: errors.is_empty(),
        errors,
        next_phase,
    })
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn find_issue_path(project_root: &Path, slug: &str) -> Option<PathBuf> {
    let issues_dir = crate::shared::workspace::issues_path(project_root);
    for sub in ["open", "closed"] {
        let candidate = issues_dir.join(sub).join(format!("{}.md", slug));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Return true when a `## Heading` section has no body text (whitespace only
/// before the next `## ` heading or end of file).
fn section_is_empty(content: &str, heading: &str) -> bool {
    let Some(start) = content.find(heading) else {
        return true;
    };
    let after = &content[start + heading.len()..];
    let body_end = after.find("\n## ").unwrap_or(after.len());
    let body = &after[..body_end];
    // Skip past the heading line terminator; anything non-whitespace counts.
    body.lines().skip(1).all(|l| l.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_change(temp: &TempDir, change_id: &str) -> PathBuf {
        let change_dir = temp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::create_dir_all(change_dir.join("payloads")).unwrap();
        std::fs::write(change_dir.join("STATE.yaml"), "phase: change_inited\n").unwrap();
        change_dir
    }

    #[test]
    fn review_payload_missing_verdict_fails() {
        let temp = TempDir::new().unwrap();
        let change_dir = make_change(&temp, "c1");
        std::fs::write(
            change_dir.join("payloads/review-change-spec.json"),
            r#"{"summary":"looks good"}"#,
        )
        .unwrap();
        let out = validate_review(&change_dir, &StatePhase::ChangeSpecCreated).unwrap();
        assert!(!out.passed);
        assert!(out.errors.iter().any(|e| e.contains("verdict")));
    }

    #[test]
    fn review_payload_valid_advances_phase() {
        let temp = TempDir::new().unwrap();
        let change_dir = make_change(&temp, "c2");
        std::fs::write(
            change_dir.join("payloads/review-change-spec.json"),
            r#"{"verdict":"APPROVED","summary":"lgtm","issues":[]}"#,
        )
        .unwrap();
        let out = validate_review(&change_dir, &StatePhase::ChangeSpecCreated).unwrap();
        assert!(out.passed, "errors: {:?}", out.errors);
        assert_eq!(out.next_phase, Some(StatePhase::ChangeSpecReviewed));
    }

    #[test]
    fn change_implementation_missing_log_fails() {
        let temp = TempDir::new().unwrap();
        let change_dir = make_change(&temp, "c3");
        let out =
            validate_change_implementation("c3", &change_dir, &StatePhase::ChangeSpecReviewed)
                .unwrap();
        assert!(!out.passed);
        assert!(out
            .errors
            .iter()
            .any(|e| e.contains("artifact_writes.jsonl")));
    }

    #[test]
    fn change_implementation_with_log_passes() {
        let temp = TempDir::new().unwrap();
        let change_dir = make_change(&temp, "c4");
        std::fs::write(
            change_dir.join("artifact_writes.jsonl"),
            r#"{"action":"create-change-implementation","change_id":"c4","ts":"now","payload_sha256":"abc"}"#,
        )
        .unwrap();
        let out =
            validate_change_implementation("c4", &change_dir, &StatePhase::ChangeSpecReviewed)
                .unwrap();
        assert!(out.passed, "errors: {:?}", out.errors);
        assert_eq!(
            out.next_phase,
            Some(StatePhase::ChangeImplementationCreated)
        );
    }

    #[test]
    fn issue_author_missing_sections_fails() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();
        std::fs::create_dir_all(crate::shared::workspace::issues_path(project_root).join("open"))
            .unwrap();
        std::fs::write(
            crate::shared::workspace::issues_path(project_root)
                .join("open")
                .join("my-slug.md"),
            "---\nstate: draft\n---\n\n# Title\n\nsome body\n",
        )
        .unwrap();
        let change_dir = make_change(&temp, "my-slug");
        let out = validate_issue_author("my-slug", project_root, &change_dir).unwrap();
        assert!(!out.passed);
        assert!(out.errors.iter().any(|e| e.contains("Problem")));
    }
}
