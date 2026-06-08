//! Review tools for change-spec.
//!
//! - `sdd_workflow_review_change_spec` — returns review prompt for the current spec
//! - `sdd_artifact_review_change_spec` — writes inline `# Reviews` section + verdict

use super::common_change_spec::{self as common, SpecSubState};
#[cfg_attr(not(test), allow(unused_imports))]
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::state::StateManager;
use crate::tools::review_helpers;
use crate::tools::workflow_common;
use crate::tools::{get_optional_string, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_spec/definitions.md#source
// CODEGEN-BEGIN
/// MCP tool definition for sdd_workflow_review_change_spec
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_review_change_spec".to_string(),
        description: "Return review prompt for a change spec (whole-file review)".to_string(),
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

/// MCP tool definition for sdd_artifact_review_change_spec
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_review_change_spec".to_string(),
        description: "Write inline review for a change spec with `# Reviews` section".to_string(),
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
                    "pattern": "^[a-z0-9-]+$",
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
                },
                "problem_sections": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["overview", "requirements", "scenarios", "db-model", "dependency", "state-machine", "logic", "interaction", "mindmap", "rest-api", "rpc-api", "async-api", "cli", "schema", "config", "wireframe", "component", "design-token", "unit-test", "e2e-test", "changes", "doc"]
                    },
                    "description": "Sections that need revision (used by revise flow)"
                }
            }
        }),
    }
}
// CODEGEN-END
// ─── Workflow ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_spec/workflow.md#source
// CODEGEN-BEGIN
/// Execute sdd_workflow_review_change_spec.
///
/// Returns review prompt for the current spec in Review sub-state.
/// @spec projects/agentic-workflow/tech-design/core/tools/review_change_spec/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);

    match common::resolve_next_spec(&change_dir, &change_id)? {
        SpecSubState::Review { spec_id } => {
            build_review_prompt(&change_id, &spec_id, group_id.as_deref(), project_root).await
        }
        SpecSubState::AdvanceToImplementation => {
            // REQ: bug-create-change-spec-review-change-spec-routing-dead
            // All specs filled + complete + no pending review → advance to
            // implementation directly. Previously this fell into the `_` arm
            // and redirected to create_change_spec, which saw create_complete
            // and redirected back here ⇒ infinite loop.
            let result = json!({
                "status": "ok",
                "prompt": "All specs are create_complete with no pending review. Advancing to implementation.",
                "next_actions": [
                    workflow_common::next_action(
                        interface,
                        "sdd_workflow_create_change_implementation",
                        json!({"change_id": change_id}),
                    )
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
        _ => {
            // Still in Create/Revise/MainthreadMustFix — redirect back to create router
            let result = json!({
                "status": "ok",
                "prompt": "Spec is not in Review sub-state. Redirecting to router.",
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}
// CODEGEN-END
// ─── Artifact Review ─────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_spec/artifact.md#source
// CODEGEN-BEGIN
/// Execute sdd_artifact_review_change_spec.
///
/// Writes inline review section into `specs/{spec_id}.md`.
/// Updates `review_verdict` in frontmatter and appends `# Reviews` section.
/// For non-APPROVED verdicts, also stores `problem_sections` for revise flow.
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
    let problem_sections: Vec<String> = args
        .get("problem_sections")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let caller = get_optional_string(args, "caller").unwrap_or_else(|| "reviewer".to_string());

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    // Validate verdict
    if !["APPROVED", "REVIEWED", "REJECTED"].contains(&verdict.as_str()) {
        anyhow::bail!("verdict must be 'APPROVED', 'REVIEWED', or 'REJECTED'");
    }

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    // Use group-aware path lookup (checks groups/*/specs/ first, then specs/)
    let spec_path = common::find_spec_path(&change_dir, &spec_id);

    if !spec_path.exists() {
        anyhow::bail!(
            "Spec file not found: {}.md (searched groups and specs/)",
            spec_id
        );
    }

    // Read current content
    let content = std::fs::read_to_string(&spec_path)?;

    // Strip old review section
    let stripped = review_helpers::strip_review_section(&content);

    // Get iteration number from revision count
    let rev_key = format!("spec:{}", spec_id);
    let sm = StateManager::load(&change_dir)?;
    let iteration = sm.revision_count(&rev_key) as u64 + 1;
    drop(sm);

    // Build new review section
    let review_section = review_helpers::build_review_section(
        &verdict, &summary, &checklist, &issues, iteration, &caller, &change_id,
    );

    // Append review section
    let new_content = format!("{}\n\n{}", stripped, review_section);

    // Update frontmatter based on verdict
    let final_content = if verdict == "APPROVED" {
        // Clean state: remove review_verdict and iteration markers
        let cleaned = review_helpers::remove_frontmatter_field(&new_content, "review_verdict");
        let cleaned = review_helpers::remove_frontmatter_field(&cleaned, "review_iteration");
        let cleaned = review_helpers::remove_frontmatter_field(&cleaned, "problem_sections");
        review_helpers::remove_frontmatter_field(&cleaned, "filled_sections")
    } else {
        // Upsert review_verdict and problem_sections
        let updated =
            review_helpers::upsert_frontmatter_field(&new_content, "review_verdict", &verdict);
        let updated = review_helpers::upsert_frontmatter_field(
            &updated,
            "review_iteration",
            &iteration.to_string(),
        );
        // Remove filled_sections for fresh revise tracking
        let updated = review_helpers::remove_frontmatter_field(&updated, "filled_sections");
        if !problem_sections.is_empty() {
            let sections_str = format!("[{}]", problem_sections.join(", "));
            review_helpers::upsert_frontmatter_field(&updated, "problem_sections", &sections_str)
        } else {
            updated
        }
    };

    std::fs::write(&spec_path, &final_content)?;

    // Phase advance moved to `score workflow validate` (three-role-contract R8).
    // The artifact CLI only writes payload + files; SubagentStop hook advances phase.

    let artifacts_written = vec![format!("specs/{}.md", spec_id)];

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
        ]
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END
// ─── Prompt Builder ──────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_spec/alignment-report.md#source
// CODEGEN-BEGIN
/// Build alignment report section for spec review prompts.
///
/// Calls `spec_alignment::check()` on the change-spec file being reviewed.
/// Returns a markdown section to inject into the review prompt, or empty string on error.
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
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_spec/prompt.md#source
// CODEGEN-BEGIN
/// Build REVIEW prompt for a spec.
async fn build_review_prompt(
    change_id: &str,
    spec_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    // Build alignment report for injection into prompt
    let spec_abs_path = match group_id {
        Some(gid) => project_root.join(format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        )),
        None => project_root.join(format!(".aw/changes/{}/specs/{}.md", change_id, spec_id)),
    };
    let alignment_section = if spec_abs_path.exists() {
        build_alignment_report(&spec_abs_path)
    } else {
        String::new()
    };

    let prompt = format!(
        r#"# Task: Review Spec '{spec_id}' for Change '{change_id}'

{alignment_section}## Instructions

1. **Run automated validation**:
   `score workflow validate-spec-completeness {change_id} --spec-id {spec_id}`
2. **Read the spec**:
   `.aw/changes/{change_id}/specs/{spec_id}.md`
3. **Read the proposal** for context routing
4. **Evaluate against checklist**:
   - Overview is substantive (>= 50 chars)
   - Requirements are well-defined with IDs and descriptions
   - At least one scenario per requirement
   - Diagrams are relevant and correct (if present)
   - API specs are valid (if present)
   - Changes list covers all affected files
   - No duplicate section types in this spec file
   - Sections follow dependency order: data → behavior → interface → test → changes
5. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve. Report every problem NOW — do not hold back issues for a future round.
6. **Determine verdict**: APPROVED / REVIEWED / REJECTED
7. **Identify problem sections**: If not APPROVED, list which sections need work
8. Write the review

## Verdict Guidelines

- **APPROVED**: Passes all checklist items, spec is implementation-ready
- **REVIEWED**: Missing elements, unclear requirements, or insufficient scenarios
- **REJECTED**: Fundamental design problems, wrong approach

## CLI Commands

```
# Validate spec completeness
score workflow validate-spec-completeness {change_id} --spec-id {spec_id}

# Read spec
Read file: .aw/changes/{change_id}/specs/{spec_id}.md

# Write review (write payload JSON first, then run)
score artifact review-change-spec {change_id} .aw/changes/{change_id}/payloads/review-change-spec.json
```"#,
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewChangeSpec);

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("review_spec_{}", spec_id),
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_spec/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    fn setup_review_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeSpecCreated;
        sm.save().unwrap();

        // Write spec with create_complete (Review sub-state)
        let content = format!(
            "---\nid: {cid}-spec\ncreate_complete: true\n---\n\n\
             # Test Spec\n\n## Overview\n\nGood overview.\n\n\
             ## Requirements\n\n### R1: Test\n\nDescription.\n\n\
             ## Scenarios\n\n### Scenario: Test\n\n**WHEN** action\n**THEN** result\n",
            cid = change_id
        );
        std::fs::write(specs_dir.join(format!("{}-spec.md", change_id)), &content).unwrap();

        // Write proposal with spec listed (no review file = pending review)
        let proposal = format!(
            "---\nspec_plan:\n- id: {}-spec\n---\n\n# Proposal\n",
            change_id
        );
        std::fs::write(change_dir.join("proposal.md"), &proposal).unwrap();

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
        let prompt = read_prompt(&parsed, tmp.path(), "rev-wf", "review_spec_rev-wf-spec");
        assert!(prompt.contains("Review Spec"));
    }

    #[test]
    fn test_artifact_review_approved() {
        let tmp = setup_review_change("rev-approve");
        let change_dir = tmp.path().join(".aw/changes/rev-approve");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-approve",
            "spec_id": "rev-approve-spec",
            "verdict": "APPROVED",
            "summary": "Spec looks good. All requirements covered.",
            "checklist_results": [{"item": "Overview", "passed": true}],
            "issues": []
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Verify file has review section
        let spec_path = change_dir.join("specs/rev-approve-spec.md");
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("# Reviews"));
        assert!(content.contains("APPROVED"));
        // review_verdict should be removed for APPROVED
        assert!(!content.contains("review_verdict:"));

        // Phase advance moved to `score workflow validate` (three-role-contract R8);
        // execute_artifact no longer advances phase.
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeSpecCreated);
    }

    #[test]
    fn test_artifact_review_with_issues() {
        let tmp = setup_review_change("rev-issues");
        let change_dir = tmp.path().join(".aw/changes/rev-issues");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-issues",
            "spec_id": "rev-issues-spec",
            "verdict": "REVIEWED",
            "summary": "Requirements need more detail.",
            "issues": [
                {"severity": "HIGH", "description": "Missing requirement R2"}
            ],
            "problem_sections": ["requirements", "scenarios"]
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Verify file has review verdict and problem sections
        let spec_path = change_dir.join("specs/rev-issues-spec.md");
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("problem_sections: [requirements, scenarios]"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("Missing requirement R2"));
    }

    #[test]
    fn test_artifact_review_rejected() {
        let tmp = setup_review_change("rev-reject");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-reject",
            "spec_id": "rev-reject-spec",
            "verdict": "REJECTED",
            "summary": "Fundamental design problem.",
            "issues": [
                {"severity": "HIGH", "description": "Wrong approach entirely"}
            ],
            "problem_sections": ["overview", "requirements"]
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let spec_path = tmp
            .path()
            .join(".aw/changes/rev-reject/specs/rev-reject-spec.md");
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("review_verdict: REJECTED"));
    }

    #[test]
    fn test_artifact_review_invalid_verdict() {
        let tmp = setup_review_change("rev-invalid");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-invalid",
            "spec_id": "rev-invalid-spec",
            "verdict": "UNKNOWN",
            "summary": "test"
        });
        let result = execute_artifact(&args, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("verdict must be"));
    }

    // ─── Phase 3: Alignment Report Tests (R24) ────────────���────────────────

    #[test]
    fn test_review_spec_prompt_includes_alignment_violations() {
        // R24: Spec with FormatPriorityViolation → report contains table
        let tmp = TempDir::new().unwrap();
        let spec_path = tmp.path().join("fpv-spec.md");
        // Section annotated as config/json but missing code block → FormatPriorityViolation
        std::fs::write(
            &spec_path,
            "---\nid: fpv-spec\n---\n# Spec\n\n\
             ## Config\n<!-- type: config lang: json -->\n\nJust prose, no code block.\n",
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
            report.contains("format_priority_violation"),
            "Should contain format_priority_violation kind"
        );
    }

    #[test]
    fn test_review_spec_prompt_clean_spec() {
        // R24: Clean spec → "No alignment violations found."
        let tmp = TempDir::new().unwrap();
        let spec_path = tmp.path().join("clean-spec.md");
        std::fs::write(&spec_path, "---\nid: clean-spec\n---\n# Clean Spec\n").unwrap();

        let report = build_alignment_report(&spec_path);
        assert!(
            report.contains("## Alignment Report"),
            "Should contain alignment report heading"
        );
        assert!(
            report.contains("No alignment violations found."),
            "Clean spec should show no-violations message"
        );
    }

    /// REQ: bug-create-change-spec-review-change-spec-routing-dead
    ///
    /// When the spec is create_complete and there's NO proposal.md (so
    /// `pending_review_spec = None`), `resolve_next_spec` returns
    /// `AdvanceToImplementation`. Before the fix, `review_change_spec`'s
    /// `_` arm redirected back to `create_change_spec`, which then advised
    /// `review_change_spec` again → infinite loop. The fix adds an explicit
    /// `AdvanceToImplementation` arm that redirects forward to
    /// `create_change_implementation`.
    #[tokio::test]
    async fn test_review_workflow_advances_to_impl_when_complete_without_proposal() {
        let tmp = TempDir::new().unwrap();
        let change_id = "impl-advance";
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeSpecCreated;
        sm.save().unwrap();

        // Write spec WITH create_complete + APPROVED verdict and NO proposal.md.
        // With CRR mandatory review (change-spec.md#CRR1), AdvanceToImplementation
        // requires an APPROVED verdict on every complete spec. This simulates a
        // fully-reviewed spec so we can exercise the advance path.
        let content = format!(
            "---\nid: {cid}-spec\ncreate_complete: true\nreview_verdict: APPROVED\n---\n\n# Test Spec\n",
            cid = change_id
        );
        std::fs::write(specs_dir.join(format!("{}-spec.md", change_id)), &content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": change_id
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["status"], "ok");
        // Must advise create_change_implementation, NOT create_change_spec
        let next_cli = parsed["next_actions"][0]["cli"].as_str().unwrap();
        assert!(
            next_cli.contains("create-change-implementation")
                || next_cli.contains("create_change_implementation"),
            "Expected next action to be create-change-implementation, got: {}",
            next_cli
        );
        // Must NOT loop back to review or create_change_spec
        assert!(!next_cli.contains("review-change-spec"));
        assert!(!next_cli.contains("create-change-spec"));
    }
}
// CODEGEN-END
