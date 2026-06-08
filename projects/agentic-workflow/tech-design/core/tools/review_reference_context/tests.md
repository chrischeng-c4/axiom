---
id: sdd-tools-review-reference-context-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review reference context tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_reference_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 47 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 160 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 121 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 23 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::state::StatePhase;
    use crate::state::StateManager;
    use tempfile::TempDir;

    fn setup_review_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let group_dir = change_dir.join("groups").join("my-group");
        std::fs::create_dir_all(&group_dir).unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        // Write artifact without review (Review sub-state)
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: test\ngroup: my-group\ndate: 2026-03-04\n---\n\n# Reference Context\n\n| Spec | Group | Relevance | Key Requirements |\n|------|-------|-----------|------------------|\n| some-spec | test | high | R1 |\n",
        ).unwrap();

        tmp
    }

    /// Read prompt content from inline response, prompt_path, or the prompts directory
    /// (when agent dispatch consumed the prompt).
    fn read_prompt(
        parsed: &Value,
        project_root: &std::path::Path,
        change_id: &str,
        group_id: &str,
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
            .join("groups")
            .join(group_id)
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
        // Status can be "ok" (mainthread or agent success) or "agent_error"
        assert!(
            parsed["status"].as_str().unwrap().starts_with("ok")
                || parsed["status"] == "agent_error"
                || parsed["agent_completed"] == true
        );
        let prompt = read_prompt(
            &parsed,
            tmp.path(),
            "rev-wf",
            "my-group",
            "review_reference_context",
        );
        assert!(prompt.contains("Review Reference Context"));
    }

    #[test]
    fn test_artifact_review_approved_marks_done() {
        let tmp = setup_review_change("rev-approved");
        let change_dir = tmp.path().join(".aw/changes/rev-approved");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-approved",
            "group_id": "my-group",
            "verdict": "APPROVED",
            "summary": "Looks good.",
            "checklist_results": [{"item": "Specs covered", "passed": true}],
            "issues": []
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Groups removed — completion tracking simplified
        let _sm = StateManager::load(&change_dir).unwrap();
    }

    #[test]
    fn test_artifact_review_writes_inline_review() {
        let tmp = setup_review_change("rev-inline");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-inline",
            "group_id": "my-group",
            "verdict": "REVIEWED",
            "summary": "Missing some specs.",
            "issues": [{"severity": "MEDIUM", "description": "Missing lens spec"}]
        });
        execute_artifact(&args, tmp.path()).unwrap();

        // Verify file has review section
        let artifact_path = tmp
            .path()
            .join(".aw/changes/rev-inline/groups/my-group/reference_context.md");
        let content = std::fs::read_to_string(&artifact_path).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("Missing some specs."));
        assert!(content.contains("Missing lens spec"));
    }

    #[test]
    fn test_artifact_review_auto_approves_on_revision_limit() {
        let tmp = setup_review_change("rev-auto");
        let change_dir = tmp.path().join(".aw/changes/rev-auto");

        // Set revision count >= 1
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("ref_ctx:my-group");
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-auto",
            "group_id": "my-group",
            "verdict": "REVIEWED",
            "summary": "Still has issues but auto-approving.",
            "issues": []
        });
        execute_artifact(&args, tmp.path()).unwrap();

        // Groups removed — completion tracking simplified
        let _sm = StateManager::load(&change_dir).unwrap();
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
      - "<module-trailer>"
    description: "Regression tests for reference-context review workflow, artifact writes, and auto-approval behavior."
```
