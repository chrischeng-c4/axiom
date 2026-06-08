// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review/preamble-definition.md#source
// CODEGEN-BEGIN
//! sdd_review_file MCP Tool
//!
//! Unified review tool for decide-stage contexts/gaps and plan-stage proposal/spec.
//! For in-scope artifacts (contexts, gaps, proposal, spec), writes reviews as
//! an inline `# Reviews` section inside the original artifact file.
//! For implementation reviews, writes separate `review_impl*.md` files.
//!
//! Auto-updates STATE.yaml phase after writing the review artifact.

use super::review_helpers::{
    build_review_section, remove_frontmatter_field, strip_review_section, upsert_frontmatter_field,
};
use super::{get_optional_string, get_required_string, ToolDefinition};
use crate::models::state::StatePhase;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Valid review file types
const VALID_FILES: &[&str] = &[
    // Decide stage (8)
    "context_clarifications",
    "spec_clarifications",
    "spec_context",
    "knowledge_context",
    "codebase_context",
    "gap_codebase_spec",
    "gap_codebase_knowledge",
    "gap_spec_knowledge",
    // Plan stage (2)
    "proposal",
    "spec",
    // Impl stage (1)
    "implementation",
];

/// @spec projects/agentic-workflow/tech-design/core/tools/review/preamble-definition.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_review_file".to_string(),
        description: "Write a structured review artifact (review_{file}.md). Covers decide-stage contexts/gaps and plan-stage proposal/spec.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "file", "verdict", "summary"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID (lowercase, hyphens allowed)"
                },
                "caller": {
                    "type": "string",
                    "enum": ["agent", "mainthread"],
                    "default": "mainthread",
                    "description": "Who is calling: agent (via sdd_delegate_agent) or mainthread. Controls whether next dispatch hint is included in response."
                },
                "file": {
                    "type": "string",
                    "enum": VALID_FILES,
                    "description": "Artifact type being reviewed"
                },
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "REVIEWED", "REJECTED"],
                    "description": "Review verdict"
                },
                "summary": {
                    "type": "string",
                    "minLength": 10,
                    "description": "Summary of review findings"
                },
                "checklist_results": {
                    "type": "array",
                    "description": "Checklist items with pass/fail",
                    "items": {
                        "type": "object",
                        "required": ["item", "passed"],
                        "properties": {
                            "item": { "type": "string" },
                            "passed": { "type": "boolean" },
                            "note": { "type": "string" }
                        }
                    }
                },
                "issues": {
                    "type": "array",
                    "default": [],
                    "description": "List of issues found",
                    "items": {
                        "type": "object",
                        "required": ["severity", "description"],
                        "properties": {
                            "severity": {
                                "type": "string",
                                "enum": ["HIGH", "MEDIUM", "LOW"]
                            },
                            "description": { "type": "string" },
                            "recommendation": { "type": "string" }
                        }
                    }
                },
                "spec_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Spec ID (required when file='spec')"
                },
                "task_id": {
                    "type": "string",
                    "description": "Task ID for per-task review (when file='implementation'). Generates review_impl_{task_id}.md instead of global review_impl.md"
                },
                "iteration": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 1,
                    "description": "Review iteration number"
                }
            }
        }),
    }
}
// CODEGEN-END
// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review/execute.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/tools/review/execute.md#source
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let caller = args
        .get("caller")
        .and_then(|v| v.as_str())
        .unwrap_or("mainthread");
    let change_id = get_required_string(args, "change_id")?;
    let file = get_required_string(args, "file")?;
    let verdict = get_required_string(args, "verdict")?;
    let summary = get_required_string(args, "summary")?;
    let spec_id = get_optional_string(args, "spec_id");
    let task_id = get_optional_string(args, "task_id");
    let iteration = args.get("iteration").and_then(|v| v.as_u64()).unwrap_or(1);

    // Validate change_id
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("change_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate file type
    if !VALID_FILES.contains(&file.as_str()) {
        anyhow::bail!("Invalid file '{}'. Valid: {}", file, VALID_FILES.join(", "));
    }

    // Validate verdict
    if !["APPROVED", "REVIEWED", "REJECTED", "PASS", "NEEDS_REVISION"].contains(&verdict.as_str()) {
        anyhow::bail!("verdict must be APPROVED, REVIEWED, or REJECTED");
    }

    // Normalize legacy verdict names to unified standard
    let verdict = match verdict.as_str() {
        "PASS" => "APPROVED".to_string(),
        "NEEDS_REVISION" => "REVIEWED".to_string(),
        _ => verdict,
    };

    // spec_id required for spec reviews
    if file == "spec" && spec_id.is_none() {
        anyhow::bail!("spec_id is required when file='spec'");
    }

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", change_id);
    }

    // Parse checklist and issues
    let checklist = args
        .get("checklist_results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let issues = args
        .get("issues")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Compute display name
    let display_name = if file == "spec" {
        format!("spec:{}", spec_id.as_deref().unwrap_or("unknown"))
    } else if file == "implementation" && task_id.is_some() {
        format!("implementation:task_{}", task_id.as_deref().unwrap())
    } else {
        file.clone()
    };

    // Write review: inline (# Reviews section appended to artifact)
    let filename = write_inline_review(
        &change_dir,
        &file,
        &verdict,
        &summary,
        &checklist,
        &issues,
        iteration,
        &display_name,
        &change_id,
        spec_id.as_deref(),
        task_id.as_deref(),
    )?;

    // Auto-update STATE.yaml phase based on file type
    let target_phase = match file.as_str() {
        "spec" => Some(StatePhase::ChangeSpecReviewed),
        "implementation" => Some(StatePhase::ChangeImplementationReviewed),
        // Legacy context/gap/proposal types: no phase update (handled by run_change)
        _ => None,
    };
    if let Some(phase) = target_phase {
        super::workflow_common::update_phase(&change_dir, phase)?;
    }

    // Count issues
    let high = issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("HIGH"))
        .count();
    let medium = issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("MEDIUM"))
        .count();
    let low = issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("LOW"))
        .count();

    let status = format!(
        "Review written: {}\nVerdict: {}\nIssues: {} high, {} medium, {} low",
        filename, verdict, high, medium, low
    );
    if caller == "agent" {
        Ok(status)
    } else {
        Ok(format!(
            "{}\n\n→ Next: call `sdd_run_change` to continue.",
            status
        ))
    }
}
// CODEGEN-END
// ---------------------------------------------------------------------------
// Inline review writer (contexts, gaps, proposal, spec)
// ---------------------------------------------------------------------------

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review/inline-writer.md#source
// CODEGEN-BEGIN
/// Write inline review into the original artifact file.
/// Returns the filename of the modified artifact.
fn write_inline_review(
    change_dir: &Path,
    file: &str,
    verdict: &str,
    summary: &str,
    checklist: &[Value],
    issues: &[Value],
    iteration: u64,
    display_name: &str,
    change_id: &str,
    spec_id: Option<&str>,
    task_id: Option<&str>,
) -> Result<String> {
    let target_path = if file == "spec" {
        change_dir
            .join("specs")
            .join(format!("{}.md", spec_id.unwrap()))
    } else if file == "implementation" {
        match task_id {
            Some(tid) => change_dir.join(format!("impl_{}.md", tid)),
            None => change_dir.join("impl.md"),
        }
    } else {
        change_dir.join(format!("{}.md", file))
    };

    if !target_path.exists() {
        anyhow::bail!("Artifact file not found: {}", target_path.display());
    }

    let original = std::fs::read_to_string(&target_path)?;

    // Strip existing # Reviews section
    let base = strip_review_section(&original);

    // Update frontmatter fields
    let updated = if verdict == "APPROVED" {
        let c = remove_frontmatter_field(&base, "review_verdict");
        remove_frontmatter_field(&c, "review_iteration")
    } else {
        let c = upsert_frontmatter_field(&base, "review_verdict", verdict);
        upsert_frontmatter_field(&c, "review_iteration", &iteration.to_string())
    };

    // Build final content
    let final_content = if verdict == "APPROVED" {
        format!("{}\n", updated)
    } else {
        let review = build_review_section(
            verdict,
            summary,
            checklist,
            issues,
            iteration,
            display_name,
            change_id,
        );
        format!("{}\n\n{}", updated, review)
    };

    std::fs::write(&target_path, &final_content)?;

    Ok(target_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string())
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change(change_id: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        let change_dir = project_root.join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), change_id);
        (temp_dir, project_root)
    }

    // -- Inline review tests --

    #[test]
    fn test_review_context_approved() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("spec_context.md"),
            "---\nid: test-change\ntype: spec_context\n---\n\n# Spec Context\n\nContent.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "spec_context",
            "verdict": "PASS",
            "summary": "All checklist items satisfied",
            "checklist_results": [
                {"item": "All spec groups scanned", "passed": true},
                {"item": "Each spec has relevance score", "passed": true}
            ]
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("spec_context.md"));
        assert!(result.contains("APPROVED")); // PASS normalized

        let content = std::fs::read_to_string(change_dir.join("spec_context.md")).unwrap();
        // APPROVED: no review fields in frontmatter, no # Reviews section
        assert!(!content.contains("review_verdict:"));
        assert!(!content.contains("# Reviews"));
        // Original content preserved
        assert!(content.contains("# Spec Context"));
        assert!(content.contains("Content."));
    }

    #[test]
    fn test_review_gap_reviewed() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("gap_codebase_spec.md"),
            "---\nid: test-change\ntype: gap\n---\n\n# Gap Analysis\n\nGap content.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "gap_codebase_spec",
            "verdict": "REVIEWED",
            "summary": "Missing file paths for two gaps",
            "issues": [
                {"severity": "HIGH", "description": "Gap #3 missing file path", "recommendation": "Add crate path"},
                {"severity": "MEDIUM", "description": "Severity not assigned for gap #5"}
            ]
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("gap_codebase_spec.md"));
        assert!(result.contains("REVIEWED"));
        assert!(result.contains("1 high"));
        assert!(result.contains("1 medium"));

        let content = std::fs::read_to_string(change_dir.join("gap_codebase_spec.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 1"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("**Verdict**: REVIEWED"));
        assert!(content.contains("**[HIGH]** Gap #3 missing file path"));
        assert!(content.contains("# Gap Analysis")); // original preserved
    }

    #[test]
    fn test_review_spec_requires_spec_id() {
        let (_tmp, root) = setup_change("test-change");
        let args = json!({
            "change_id": "test-change",
            "file": "spec",
            "verdict": "PASS",
            "summary": "Spec looks good overall"
        });
        let result = execute(&args, &root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("spec_id is required"));
    }

    #[test]
    fn test_review_spec_with_spec_id() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("per-task-loop.md"),
            "---\nid: per-task-loop\ntype: spec\n---\n\n# Spec\n\nContent.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "spec",
            "spec_id": "per-task-loop",
            "verdict": "PASS",
            "summary": "Spec requirements and scenarios are complete"
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("per-task-loop.md"));
    }

    #[test]
    fn test_review_invalid_file() {
        let (_tmp, root) = setup_change("test-change");
        let args = json!({
            "change_id": "test-change",
            "file": "invalid_type",
            "verdict": "PASS",
            "summary": "This should fail"
        });
        let result = execute(&args, &root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid file"));
    }

    #[test]
    fn test_review_proposal_with_iteration() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("proposal.md"),
            "---\nid: test-change\ntype: proposal\n---\n\n# Proposal\n\nContent.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "proposal",
            "verdict": "REVIEWED",
            "summary": "Proposal needs minor adjustments",
            "iteration": 2
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("proposal.md"));

        let content = std::fs::read_to_string(change_dir.join("proposal.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 2"));
        assert!(content.contains("Iteration 2"));
    }

    #[test]
    fn test_review_spec_frontmatter_has_review_fields() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("auth-flow.md"),
            "---\nid: auth-flow\ntype: spec\n---\n\n# Auth Flow\n\nContent.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "spec",
            "spec_id": "auth-flow",
            "verdict": "REVIEWED",
            "summary": "Missing error scenarios",
            "issues": [{"severity": "MEDIUM", "description": "No error case for expired token"}]
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("auth-flow.md"));

        let content = std::fs::read_to_string(specs_dir.join("auth-flow.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 1"));
        assert!(content.contains("# Auth Flow")); // original preserved
    }

    #[test]
    fn test_review_approved_clears_previous_review() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        // Artifact with existing inline review
        std::fs::write(
            change_dir.join("proposal.md"),
            "---\nid: test-change\ntype: proposal\nreview_verdict: REVIEWED\nreview_iteration: 1\n---\n\n# Proposal\n\nContent.\n\n# Reviews\n\n## Review (Iteration 1)\n\nOld review.\n",
        ).unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "proposal",
            "verdict": "APPROVED",
            "summary": "All good now",
            "iteration": 2
        });
        execute(&args, &root).unwrap();

        let content = std::fs::read_to_string(change_dir.join("proposal.md")).unwrap();
        assert!(!content.contains("review_verdict"));
        assert!(!content.contains("review_iteration"));
        assert!(!content.contains("# Reviews"));
        assert!(content.contains("# Proposal"));
        assert!(content.contains("Content."));
    }

    #[test]
    fn test_review_updates_previous_review() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        // Artifact with existing inline review
        std::fs::write(
            change_dir.join("proposal.md"),
            "---\nid: test-change\ntype: proposal\nreview_verdict: REVIEWED\nreview_iteration: 1\n---\n\n# Proposal\n\nContent.\n\n# Reviews\n\n## Review (Iteration 1)\n\nOld review.\n",
        ).unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "proposal",
            "verdict": "REVIEWED",
            "summary": "Still needs work",
            "iteration": 2
        });
        execute(&args, &root).unwrap();

        let content = std::fs::read_to_string(change_dir.join("proposal.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 2"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("Iteration 2"));
        assert!(!content.contains("Old review.")); // old review stripped
        assert!(content.contains("Still needs work"));
    }

    // -- Inline implementation review tests --

    #[test]
    fn test_review_implementation_global_inline() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        // Create impl.md (auto-generated by implementation workflow)
        std::fs::write(
            change_dir.join("impl.md"),
            "# Implementation Diff\n\n```diff\n+fn new_fn() {}\n```\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "implementation",
            "verdict": "PASS",
            "summary": "Implementation looks good"
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("impl.md"));
        assert!(result.contains("APPROVED"));

        let content = std::fs::read_to_string(change_dir.join("impl.md")).unwrap();
        // APPROVED: no review fields, no # Reviews section
        assert!(!content.contains("review_verdict"));
        assert!(!content.contains("# Reviews"));
        // Original diff preserved
        assert!(content.contains("# Implementation Diff"));
        assert!(content.contains("+fn new_fn()"));
    }

    #[test]
    fn test_review_implementation_task_inline() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "# Implementation Diff — Task 2.1\n\n```diff\n+fn task_fn() {}\n```\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "implementation",
            "task_id": "2.1",
            "verdict": "REVIEWED",
            "summary": "Task 2.1 needs fixes",
            "issues": [{"severity": "HIGH", "description": "Missing error handling"}]
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("impl_2.1.md"));
        assert!(result.contains("REVIEWED"));

        let content = std::fs::read_to_string(change_dir.join("impl_2.1.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 1"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("**[HIGH]** Missing error handling"));
        // Original diff preserved
        assert!(content.contains("# Implementation Diff"));
        assert!(content.contains("+fn task_fn()"));
    }

    #[test]
    fn test_review_implementation_independent_tasks_inline() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("impl_1.1.md"),
            "# Implementation Diff — Task 1.1\n\n```diff\n+fn a() {}\n```\n",
        )
        .unwrap();
        std::fs::write(
            change_dir.join("impl_1.2.md"),
            "# Implementation Diff — Task 1.2\n\n```diff\n+fn b() {}\n```\n",
        )
        .unwrap();

        // Review task 1.1 — APPROVED
        execute(
            &json!({
                "change_id": "test-change",
                "file": "implementation",
                "task_id": "1.1",
                "verdict": "PASS",
                "summary": "Task 1.1 looks good"
            }),
            &root,
        )
        .unwrap();

        // Review task 1.2 — REVIEWED
        execute(
            &json!({
                "change_id": "test-change",
                "file": "implementation",
                "task_id": "1.2",
                "verdict": "REVIEWED",
                "summary": "Task 1.2 needs fixes",
                "issues": [{"severity": "MEDIUM", "description": "Missing test"}]
            }),
            &root,
        )
        .unwrap();

        // Task 1.1: APPROVED → no review section
        let c1 = std::fs::read_to_string(change_dir.join("impl_1.1.md")).unwrap();
        assert!(!c1.contains("# Reviews"));
        assert!(c1.contains("+fn a()"));

        // Task 1.2: REVIEWED → has review section
        let c2 = std::fs::read_to_string(change_dir.join("impl_1.2.md")).unwrap();
        assert!(c2.contains("# Reviews"));
        assert!(c2.contains("review_verdict: REVIEWED"));
        assert!(c2.contains("+fn b()"));
    }
}
// CODEGEN-END
