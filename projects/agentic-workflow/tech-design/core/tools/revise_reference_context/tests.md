---
id: sdd-tools-revise-reference-context-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise reference context tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_reference_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 45 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 148 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 108 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::state::StatePhase;
    use crate::state::StateManager;
    use tempfile::TempDir;

    fn setup_revise_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let group_dir = change_dir.join("groups").join("my-group");
        std::fs::create_dir_all(&group_dir).unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        // Write artifact with REVIEWED verdict (Revise sub-state)
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: test\ngroup: my-group\ndate: 2026-03-04\nreview_verdict: REVIEWED\n---\n\n# Reference Context\n\n| Spec | Group | Relevance | Key Requirements |\n|------|-------|-----------|------------------|\n| some-spec | test | high | R1 |\n\n# Reviews\n\n## Review: reviewer (Iteration 1)\n\n**Verdict**: REVIEWED\n\nMissing specs.\n",
        ).unwrap();

        tmp
    }

    /// Read prompt content from either inline response or prompt file.
    fn read_prompt(parsed: &Value, project_root: &std::path::Path) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        let rel = parsed["prompt_path"]
            .as_str()
            .expect("Expected prompt_path in response");
        let prompt_path = project_root.join(rel);
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
    }

    #[tokio::test]
    async fn test_revise_workflow_returns_prompt() {
        let tmp = setup_revise_change("rev-wf");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-wf"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["group_id"], "my-group");
        let prompt = read_prompt(&parsed, tmp.path());
        assert!(prompt.contains("Revise Reference Context"));
    }

    #[test]
    fn test_revise_artifact_delegates_to_create() {
        let tmp = setup_revise_change("rev-art");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-art",
            "group_id": "my-group",
            "specs": [
                {
                    "spec_id": "some-spec",
                    "spec_group": "test",
                    "relevance": "high",
                    "key_requirements": ["R1"]
                },
                {
                    "spec_id": "missing-spec",
                    "spec_group": "test",
                    "relevance": "medium",
                    "key_requirements": ["R2"]
                }
            ]
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Verify file was rewritten
        let artifact_path = tmp
            .path()
            .join(".aw/changes/rev-art/groups/my-group/reference_context.md");
        let content = std::fs::read_to_string(&artifact_path).unwrap();
        assert!(content.contains("missing-spec"));
        // Review section should be gone (overwritten)
        assert!(!content.contains("# Reviews"));

        // Revise artifact increments revision count so auto-approve triggers.
        let change_dir = tmp.path().join(".aw/changes/rev-art");
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.revision_count("ref_ctx:my-group"), 1);
    }

    #[tokio::test]
    async fn test_revise_workflow_redirects_when_not_in_revise_state() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/not-revise");
        let group_dir = change_dir.join("groups/my-group");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), "not-revise");
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        // No artifact = Create sub-state, not Revise
        std::fs::write(
            group_dir.join("pre_clarifications.md"),
            "---\nchange: test\ngroup: my-group\n---\n",
        )
        .unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "not-revise"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        // Should redirect to router
        let next = &parsed["next_actions"][0];
        assert_eq!(next["args"]["change_id"], "not-revise");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/revise_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
      - "<module-trailer>"
    description: "Regression tests for reference-context revise workflow routing, artifact delegation, and revision count behavior."
```
