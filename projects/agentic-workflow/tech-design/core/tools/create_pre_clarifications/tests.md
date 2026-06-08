---
id: sdd-tools-create-pre-clarifications-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create pre clarifications tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_pre_clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 287 | artifact_definition() -> ToolDefinition |
| `definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 17 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 70 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_append` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 199 | execute_append(args: &Value, project_root: &Path) -> Result<String> |
| `execute_artifact_pre_clarifications` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 418 | execute_artifact_pre_clarifications(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow_pre_clarifications` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 343 | execute_workflow_pre_clarifications(     args: &Value,     project_root: &Path, ) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 262 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_create_clarifications_valid() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "add-oauth");

        let args = json!({
            "change_id": "add-oauth",
            "questions": [
                {
                    "topic": "Auth Method",
                    "question": "Which OAuth providers?",
                    "answer": "Google and GitHub",
                    "rationale": "Most common enterprise providers"
                }
            ]
        });

        let result = execute(&args, project_root).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["change_id"], "add-oauth");
        assert_eq!(parsed["phase"], "change_inited");
        assert_eq!(parsed["questions_count"], 1);
        assert_eq!(parsed["next"], "sdd_run_change");
        assert!(parsed["artifacts"].as_array().unwrap().len() >= 1);

        let file_path = project_root.join(".aw/changes/add-oauth/pre_clarifications.md");
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("change: add-oauth"));
        assert!(content.contains("## Q1: Auth Method"));
        assert!(content.contains("**Question**: Which OAuth providers?"));
    }

    #[test]
    fn test_create_clarifications_invalid_change_id() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "../etc/passwd",
            "questions": [{"topic": "test", "question": "test", "answer": "test", "rationale": "test"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_clarifications_multiple_questions() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "multi-test");

        let args = json!({
            "change_id": "multi-test",
            "questions": [
                {
                    "topic": "First Topic",
                    "question": "First question?",
                    "answer": "First answer",
                    "rationale": "First rationale"
                },
                {
                    "topic": "Second Topic",
                    "question": "Second question?",
                    "answer": "Second answer",
                    "rationale": "Second rationale"
                }
            ]
        });

        let result = execute(&args, project_root).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["questions_count"], 2);

        let file_path = project_root.join(".aw/changes/multi-test/pre_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Q1: First Topic"));
        assert!(content.contains("## Q2: Second Topic"));
    }

    #[test]
    fn test_append_clarifications_mcp_per_issue() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "append-mcp-test");

        // First create initial clarifications
        let create_args = json!({
            "change_id": "append-mcp-test",
            "questions": [
                {
                    "topic": "Initial Topic",
                    "question": "Initial question?",
                    "answer": "Initial answer",
                    "rationale": "Initial rationale"
                }
            ]
        });
        execute(&create_args, project_root).unwrap();

        // Now append per-issue clarifications (DAG mode)
        let append_args = json!({
            "change_id": "append-mcp-test",
            "issue": 188,
            "questions": [
                {
                    "topic": "Auth Method",
                    "question": "Which auth?",
                    "answer": "JWT",
                    "rationale": "Standard"
                }
            ]
        });
        let result = execute_append(&append_args, project_root).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["change_id"], "append-mcp-test");
        assert_eq!(parsed["next"], "sdd_run_change");

        // Verify content
        let file_path = project_root.join(".aw/changes/append-mcp-test/pre_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Q1: Initial Topic"));
        assert!(content.contains("## Issue #188"));
        assert!(content.contains("### Q1: Auth Method"));
    }

    // --- Workflow + artifact tests (post-groups removal) ---

    fn setup_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        tmp
    }

    #[test]
    fn test_artifact_pre_clarifications_writes_file() {
        let tmp = setup_change("art-test");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "art-test",
            "answers": [
                {
                    "topic": "Scope",
                    "answer": "sdd"
                }
            ]
        });
        let result = execute_artifact_pre_clarifications(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // File written
        let file_path = tmp
            .path()
            .join(".aw/changes/art-test/pre_clarifications.md");
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("status: answered"));
        assert!(content.contains("### Q1: Scope"));

        // Phase advanced
        let change_dir = tmp.path().join(".aw/changes/art-test");
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeInited);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_pre_clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
      - "<module-trailer>"
    description: "Create-pre-clarifications regression tests."
```
