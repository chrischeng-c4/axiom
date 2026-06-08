---
id: sdd-tools-clarifications-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools clarifications tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 15 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 68 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_append` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 312 | execute_append(args: &Value, project_root: &Path) -> Result<String> |
| `execute_post_clarifications` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 198 | execute_post_clarifications(     args: &Value,     project_root: &Path,     _action: &str, ) -> Result<String> |
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
        assert_eq!(parsed["phase"], "clarified");
        assert_eq!(parsed["questions_count"], 1);
        assert_eq!(parsed["next"], "sdd_run_change");
        assert!(parsed["artifacts"].as_array().unwrap().len() >= 1);

        let file_path = project_root.join(".aw/changes/add-oauth/context_clarifications.md");
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

        let file_path = project_root.join(".aw/changes/multi-test/context_clarifications.md");
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

        // Verify appended content in pre_clarifications.md (service_append writes here)
        let file_path = project_root.join(".aw/changes/append-mcp-test/pre_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Issue #188"));
        assert!(content.contains("### Q1: Auth Method"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
      - "<module-trailer>"
    description: "Clarifications tool regression tests."
```
