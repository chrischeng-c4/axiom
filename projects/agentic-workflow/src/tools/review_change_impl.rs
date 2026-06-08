// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_impl.md#source
// CODEGEN-BEGIN
//! Review tools for change-implementation.
//!
//! - `sdd_workflow_review_change_implementation` — returns review prompt for current spec
//! - `sdd_artifact_review_change_implementation` — writes inline `## Review: {spec_id}`

use super::common_change_impl::{self as common, ImplSubState};
#[cfg_attr(not(test), allow(unused_imports))]
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::state::StateManager;
use crate::tools::workflow_common;
use crate::tools::{get_optional_string, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_review_change_implementation".to_string(),
        description: "Return review prompt for a per-spec implementation review".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                }
            }
        }),
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_review_change_implementation".to_string(),
        description: "Write inline review for a spec in implementation.md".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "spec_id", "verdict", "summary"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                },
                "spec_id": {
                    "type": "string",
                    "description": "Spec ID being reviewed"
                },
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "REVIEWED", "REJECTED"],
                    "description": "Review verdict"
                },
                "summary": {
                    "type": "string",
                    "description": "Review summary"
                },
                "checklist_results": {
                    "type": "array",
                    "description": "Checklist results",
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
                    "description": "Issues found during review",
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
                }
            }
        }),
    }
}

// ─── Workflow ─────────────────────────────────────────────────────────────────

/// Execute sdd_workflow_review_change_implementation.
///
/// Returns review prompt for the current spec in ReviewSpec sub-state.
/// @spec projects/agentic-workflow/tech-design/core/tools/review_change_impl.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);

    let (sub_state, _, _) = common::resolve_next_impl(&change_dir, &change_id)?;

    match sub_state {
        ImplSubState::ReviewSpec { spec_id } => {
            build_review_prompt(&change_id, &spec_id, group_id.as_deref(), project_root).await
        }
        _ => {
            // Not in review sub-state — redirect to create router
            let result = json!({
                "status": "ok",
                "prompt": "Not in review sub-state. Redirecting to create router.",
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

// ─── Artifact ─────────────────────────────────────────────────────────────────

/// Execute sdd_artifact_review_change_implementation.
///
/// Writes inline `## Review: {spec_id}` section into implementation.md.
/// Updates STATE.yaml phase to `ChangeImplementationReviewed`.
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
    let verdict = get_required_string(args, "verdict")?;
    let summary = get_required_string(args, "summary")?;
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
    let caller = get_optional_string(args, "caller").unwrap_or_else(|| "reviewer".to_string());

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    if !["APPROVED", "REVIEWED", "REJECTED"].contains(&verdict.as_str()) {
        anyhow::bail!("verdict must be 'APPROVED', 'REVIEWED', or 'REJECTED'");
    }

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    let impl_path = change_dir.join("implementation.md");

    if !impl_path.exists() {
        anyhow::bail!("implementation.md not found");
    }

    let content = std::fs::read_to_string(&impl_path)?;

    // Remove existing review for this spec (if re-reviewing after revise)
    let cleaned = strip_spec_review(&content, &spec_id);

    // Get iteration from revision count
    let sm = StateManager::load(&change_dir)?;
    let rev_key = format!("impl:{}", spec_id);
    let iteration = sm.revision_count(&rev_key) as u64 + 1;
    drop(sm);

    // Build the inline review section
    let review_md = build_inline_review(
        &spec_id, &verdict, &summary, &checklist, &issues, iteration, &caller, &change_id,
    );

    // Append review section
    let new_content = format!("{}\n\n{}", cleaned.trim_end(), review_md);
    std::fs::write(&impl_path, &new_content)?;

    // Phase advance moved to `score workflow validate` (three-role-contract R8).

    let result = json!({
        "status": "ok",
        "artifacts_written": ["implementation.md"],
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", json!({"change_id": change_id}))
        ]
    });
    Ok(serde_json::to_string_pretty(&result)?)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Strip an existing `## Review: {spec_id}` section from implementation.md.
///
/// Removes everything from `## Review: {spec_id}` up to the next `## Review:`
/// heading (or end of file).
fn strip_spec_review(content: &str, spec_id: &str) -> String {
    let marker = format!("## Review: {}", spec_id);
    let Some(start) = content.find(&marker) else {
        return content.to_string();
    };

    let after = &content[start + marker.len()..];
    let end = after
        .find("\n## Review:")
        .map(|pos| start + marker.len() + pos)
        .unwrap_or(content.len());

    let before = content[..start].trim_end();
    let remaining = &content[end..];

    if remaining.trim().is_empty() {
        before.to_string()
    } else {
        format!("{}\n\n{}", before, remaining.trim_start())
    }
}

/// Build an inline `## Review: {spec_id}` section.
fn build_inline_review(
    spec_id: &str,
    verdict: &str,
    summary: &str,
    checklist: &[Value],
    issues: &[Value],
    iteration: u64,
    caller: &str,
    change_id: &str,
) -> String {
    let mut md = String::new();
    md.push_str(&format!("## Review: {}\n\n", spec_id));
    md.push_str(&format!("verdict: {}\n", verdict));
    md.push_str(&format!("reviewer: {}\n", caller));
    md.push_str(&format!("iteration: {}\n", iteration));
    md.push_str(&format!("change_id: {}\n\n", change_id));

    md.push_str(&format!("**Summary**: {}\n\n", summary));

    if !checklist.is_empty() {
        md.push_str("### Checklist\n\n");
        for item in checklist {
            let name = item.get("item").and_then(|v| v.as_str()).unwrap_or("");
            let passed = item
                .get("passed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let note = item.get("note").and_then(|v| v.as_str());
            let icon = if passed { "PASS" } else { "FAIL" };
            md.push_str(&format!("- [{}] {}\n", icon, name));
            if let Some(n) = note {
                md.push_str(&format!("  - {}\n", n));
            }
        }
        md.push('\n');
    }

    if !issues.is_empty() {
        md.push_str("### Issues\n\n");
        for issue in issues {
            let severity = issue
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("MEDIUM");
            let desc = issue
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let rec = issue.get("recommendation").and_then(|v| v.as_str());
            md.push_str(&format!("- **[{}]** {}\n", severity, desc));
            if let Some(r) = rec {
                md.push_str(&format!("  - *Recommendation*: {}\n", r));
            }
        }
    }

    md
}

/// Build alignment report section for review prompts.
///
/// Calls `spec_alignment::check()` on the spec file. Returns a markdown
/// section to inject into the review prompt, or empty string on error.
fn build_alignment_report(spec_abs_path: &Path) -> String {
    let check_result =
        match std::panic::catch_unwind(|| crate::spec_alignment::check(spec_abs_path)) {
            Ok(result) => result,
            Err(_) => {
                tracing::warn!(
                    path = %spec_abs_path.display(),
                    "alignment check panicked — skipping injection"
                );
                return String::new();
            }
        };

    if check_result.total_violations == 0 {
        return "## Alignment Report\n\nNo alignment violations found.\n\n".to_string();
    }

    let mut table =
        String::from("## Alignment Report\n\n| File | Kind | Message |\n|------|------|---------|");
    for file_result in &check_result.files {
        for violation in &file_result.violations {
            table.push_str(&format!(
                "\n| {} | {} | {} |",
                file_result.path, violation.kind, violation.message
            ));
        }
    }
    table.push_str("\n\n");
    table
}

/// Build review prompt for a spec.
async fn build_review_prompt(
    change_id: &str,
    spec_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    // Group-aware spec path
    let spec_path = match group_id {
        Some(gid) => format!(".aw/changes/{change_id}/groups/{gid}/specs/{spec_id}.md"),
        None => format!(".aw/changes/{change_id}/specs/{spec_id}.md"),
    };

    // Build alignment report for injection into prompt
    let spec_abs_path = project_root.join(&spec_path);
    let alignment_section = if spec_abs_path.exists() {
        build_alignment_report(&spec_abs_path)
    } else {
        String::new()
    };

    let prompt = format!(
        r#"# Task: Review Implementation of Spec '{spec_id}' for Change '{change_id}'

## Pre-Review Step (MANDATORY)

Before evaluating any checklist items:
1. Read spec: `{spec_path}`
2. Find the `## Unit Test` and `## E2E Test` sections (if present) and note whether they exist and how many test cases they define.

{alignment_section}## Instructions

3. Read implementation diff: `.aw/changes/{change_id}/implementation.md`
4. List changed files via `score workflow list-changed-files {change_id}`
5. Review code changes against spec requirements
6. Evaluate ALL checklist items below
7. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve. Report every problem NOW.
8. Write review via the artifact CLI command

## Checklist

### Hard Checklist (MUST ALL PASS for APPROVED)

- [HARD] Code matches all spec requirements
- [HARD] If spec has `## Unit Test` or `## E2E Test` sections: diff contains corresponding Rust test coverage (`#[test]`, integration test, or runner fixture)
- [HARD] Existing tests still pass (no regressions introduced)

### Soft Checklist (Issues → REVIEWED verdict)

- Code quality and readability
- Error handling completeness
- Performance considerations
- Documentation where needed

## HARD REJECT RULE

**IF** the spec has a `## Unit Test` or `## E2E Test` section
**AND** the implementation diff contains zero matching unit/integration test additions
**THEN** verdict MUST be `REJECTED` — no exceptions, regardless of other checklist results.

This rule overrides all other considerations.

## Verdict Guidelines

- **APPROVED**: All hard checklist items pass, code matches spec, tests pass
- **REVIEWED**: Hard checklist passes but has fixable soft issues
- **REJECTED**: Any hard checklist item fails (including the hard reject rule above)

## CLI Commands

```
# Read spec and implementation
Read file: {spec_path}
Read file: .aw/changes/{change_id}/implementation.md

# List changed files
score workflow list-changed-files {change_id}

# Write review (write payload JSON first, then run)
score artifact review-change-implementation {change_id} .aw/changes/{change_id}/payloads/review-change-implementation.json
```"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::ReviewChangeImplementation,
    );

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("review_impl_{}", spec_id),
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_review_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeImplementationCreated;
        sm.save().unwrap();

        // Write a spec
        std::fs::write(
            specs_dir.join("spec-a.md"),
            "---\nid: spec-a\ntype: spec\n---\n# Spec A\n",
        )
        .unwrap();

        // Write implementation.md (no reviews yet)
        std::fs::write(
            change_dir.join("implementation.md"),
            "---\nid: impl\ntype: change_implementation\n---\n# Implementation\n\n## Diff\n\n```diff\n+code\n```\n",
        )
        .unwrap();

        tmp
    }

    /// Read prompt content from inline response, prompt_path, or the prompts directory
    /// (when agent dispatch consumed the prompt).
    fn read_prompt(
        parsed: &Value,
        project_root: &std::path::Path,
        change_id: &str,
        action: &str,
    ) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        if let Some(rel) = parsed["prompt_path"].as_str() {
            let prompt_path = project_root.join(rel);
            return std::fs::read_to_string(&prompt_path)
                .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path));
        }
        // Agent dispatch path — prompt was written to file before agent ran
        let prompt_path = project_root
            .join(".aw/changes")
            .join(change_id)
            .join("prompts")
            .join(format!("{}.md", action));
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No prompt at {:?} (agent dispatch path)", prompt_path))
    }

    #[tokio::test]
    async fn test_review_workflow_returns_prompt() {
        let tmp = setup_review_change("rev-wf");
        let _change_dir = tmp.path().join(".aw/changes/rev-wf");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-wf"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(
            parsed["status"].as_str().unwrap().starts_with("ok")
                || parsed["status"] == "agent_error"
                || parsed["agent_completed"] == true
        );
        let prompt = read_prompt(&parsed, tmp.path(), "rev-wf", "review_impl_spec-a");
        assert!(prompt.contains("Review Implementation"));
    }

    #[test]
    fn test_artifact_review_approved() {
        let tmp = setup_review_change("rev-app");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-app",
            "spec_id": "spec-a",
            "verdict": "APPROVED",
            "summary": "Implementation matches spec requirements.",
            "checklist_results": [{"item": "Tests pass", "passed": true}],
            "issues": []
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["artifacts_written"]
            .as_array()
            .unwrap()
            .contains(&json!("implementation.md")));

        // Verify content
        let impl_path = tmp.path().join(".aw/changes/rev-app/implementation.md");
        let content = std::fs::read_to_string(&impl_path).unwrap();
        assert!(content.contains("## Review: spec-a"));
        assert!(content.contains("verdict: APPROVED"));

        // Phase advance moved to `score workflow validate` (three-role-contract R8).
        let change_dir = tmp.path().join(".aw/changes/rev-app");
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeImplementationCreated);
    }

    #[test]
    fn test_artifact_review_with_issues() {
        let tmp = setup_review_change("rev-issues");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-issues",
            "spec_id": "spec-a",
            "verdict": "REVIEWED",
            "summary": "Some issues found.",
            "issues": [
                {"severity": "HIGH", "description": "Missing error handling"}
            ]
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let impl_path = tmp.path().join(".aw/changes/rev-issues/implementation.md");
        let content = std::fs::read_to_string(&impl_path).unwrap();
        assert!(content.contains("verdict: REVIEWED"));
        assert!(content.contains("Missing error handling"));
    }

    #[test]
    fn test_artifact_review_invalid_verdict() {
        let tmp = setup_review_change("rev-bad");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-bad",
            "spec_id": "spec-a",
            "verdict": "UNKNOWN",
            "summary": "test"
        });
        let result = execute_artifact(&args, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("verdict must be"));
    }

    #[test]
    fn test_strip_spec_review() {
        let content = "# Impl\n\n## Diff\n\n```diff\n+code\n```\n\n## Review: spec-a\n\nverdict: REVIEWED\nsummary: bad\n\n## Review: spec-b\n\nverdict: APPROVED\n";
        let stripped = strip_spec_review(content, "spec-a");
        assert!(!stripped.contains("## Review: spec-a"));
        assert!(stripped.contains("## Review: spec-b"));
    }

    #[test]
    fn test_strip_spec_review_last() {
        let content = "# Impl\n\n## Review: spec-a\n\nverdict: REVIEWED\n";
        let stripped = strip_spec_review(content, "spec-a");
        assert!(!stripped.contains("## Review: spec-a"));
        assert!(stripped.contains("# Impl"));
    }

    #[tokio::test]
    async fn test_review_checklist_includes_test_taxonomy_item() {
        let tmp = setup_review_change("rev-checklist");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-checklist"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let prompt = read_prompt(&parsed, tmp.path(), "rev-checklist", "review_impl_spec-a");
        assert!(
            prompt.contains("## Unit Test") && prompt.contains("## E2E Test"),
            "Prompt should reference canonical test sections"
        );
        assert!(
            prompt.contains("HARD REJECT RULE"),
            "Prompt should contain hard reject rule"
        );
        assert!(
            prompt.contains("[HARD]"),
            "Prompt should contain hard checklist items"
        );
    }

    #[tokio::test]
    async fn test_review_hard_reject_no_tests_in_diff() {
        // Verify that the prompt explicitly instructs the reviewer to REJECT
        // when Unit Test / E2E Test is present but no matching test in diff
        let tmp = setup_review_change("rev-reject");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-reject"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let prompt = read_prompt(&parsed, tmp.path(), "rev-reject", "review_impl_spec-a");
        // The hard reject rule must be in the prompt for the reviewer to enforce it
        assert!(
            prompt.contains("REJECTED") && prompt.contains("zero"),
            "Prompt must instruct reviewer to REJECT when no #[test] in diff"
        );
    }

    // ─── Phase 3: Alignment Report Tests (R23, R27) ────────────────────────

    #[test]
    fn test_review_impl_prompt_includes_alignment_violations() {
        // R23: Spec with DuplicateSection violation → report contains table
        let tmp = TempDir::new().unwrap();
        let spec_path = tmp.path().join("dup-spec.md");
        // Two identical ## Overview headings → DuplicateSection violation
        std::fs::write(
            &spec_path,
            "---\nid: dup-spec\n---\n# Spec\n\n\
             ## Overview\n<!-- type: overview lang: markdown -->\n\nSome text.\n\n\
             ## Overview\n<!-- type: overview lang: markdown -->\n\nMore text.\n",
        )
        .unwrap();

        let report = build_alignment_report(&spec_path);
        assert!(
            report.contains("## Alignment Report"),
            "Should contain alignment report heading"
        );
        assert!(
            report.contains("| File | Kind | Message |"),
            "Should contain table header"
        );
        assert!(
            report.contains("duplicate_section"),
            "Should contain duplicate_section violation kind"
        );
    }

    #[test]
    fn test_review_impl_prompt_clean_spec_no_violations() {
        // R23: Clean spec (no sections) → "No alignment violations found."
        let tmp = TempDir::new().unwrap();
        let spec_path = tmp.path().join("clean-spec.md");
        std::fs::write(&spec_path, "---\nid: clean-spec\n---\n# Clean Spec\n").unwrap();

        let report = build_alignment_report(&spec_path);
        assert!(
            report.contains("## Alignment Report"),
            "Should contain alignment report heading even when clean"
        );
        assert!(
            report.contains("No alignment violations found."),
            "Clean spec should show no-violations message"
        );
    }

    #[test]
    fn test_review_impl_prompt_alignment_error_graceful() {
        // R23 + R27: Non-existent spec file → build_review_prompt guards with
        // exists() check, so alignment section is empty. Test that
        // build_alignment_report does NOT panic for non-existent files.
        let non_existent = std::path::Path::new("/tmp/does-not-exist-alignment-test/spec.md");

        // build_alignment_report should not panic — check() returns IoError
        // violation for non-existent files, which is valid behavior.
        // The prompt builder skips calling this when file doesn't exist.
        let _report = build_alignment_report(non_existent);
        // If we get here, no panic occurred — test passes.
        // The actual graceful degradation is in build_review_prompt:
        // `if spec_abs_path.exists() { build_alignment_report(...) } else { String::new() }`
    }
}

// CODEGEN-END
