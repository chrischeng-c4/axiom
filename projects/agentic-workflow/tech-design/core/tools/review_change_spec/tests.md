---
id: sdd-tools-review-change-spec-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change spec tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 48 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 186 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 129 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 24 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
    description: "Review-change-spec workflow, artifact, and alignment regression tests."
```
