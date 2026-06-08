---
id: sdd-tools-implementation-tests-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Implementation Tools Tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/implementation.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `create_merge_review_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 687 | create_merge_review_definition() -> ToolDefinition |
| `create_review_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 477 | create_review_definition() -> ToolDefinition |
| `execute_create_merge_review` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 781 | execute_create_merge_review(args: &Value, project_root: &Path) -> Result<String> |
| `execute_create_review` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 559 | execute_create_review(args: &Value, project_root: &Path) -> Result<String> |
| `execute_list_changed_files` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 356 | execute_list_changed_files(args: &Value, _project_root: &Path) -> Result<String> |
| `execute_read_all_requirements` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 117 | execute_read_all_requirements(args: &Value, project_root: &Path) -> Result<String> |
| `execute_read_implementation_summary` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 232 | execute_read_implementation_summary(args: &Value, _project_root: &Path) -> Result<String> |
| `list_changed_files_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 323 | list_changed_files_definition() -> ToolDefinition |
| `read_all_requirements_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 93 | read_all_requirements_definition() -> ToolDefinition |
| `read_implementation_summary_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 204 | read_implementation_summary_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_change_id_valid() {
        assert!(validate_change_id("test-change").is_ok());
        assert!(validate_change_id("feature-123").is_ok());
        assert!(validate_change_id("fix-bug-42").is_ok());
    }

    #[test]
    fn test_validate_change_id_invalid() {
        assert!(validate_change_id("../etc/passwd").is_err());
        assert!(validate_change_id("/absolute/path").is_err());
        assert!(validate_change_id("Test-Change").is_err()); // uppercase
        assert!(validate_change_id("test_change").is_err()); // underscore
        assert!(validate_change_id("test..change").is_err()); // double dot
    }

    #[test]
    fn test_read_all_requirements_basic() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory structure
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Create proposal.md
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Test Proposal\n\nThis is a test proposal.",
        )
        .unwrap();

        // Create tasks.md
        std::fs::write(change_dir.join("tasks.md"), "# Tasks\n\n- Task 1\n- Task 2").unwrap();

        // Create specs
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("feature-spec.md"),
            "# Feature Spec\n\nRequirements here.",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute_read_all_requirements(&args, project_root).unwrap();

        assert!(result.contains("# Requirements for Change: test-change"));
        assert!(result.contains("## Proposal"));
        assert!(result.contains("This is a test proposal"));
        assert!(result.contains("## Tasks"));
        assert!(result.contains("Task 1"));
        assert!(result.contains("## Specifications"));
        assert!(result.contains("### Spec: feature-spec"));
        assert!(result.contains("**Total**: 1 proposal, 1 tasks file, 1 specification(s)"));
    }

    #[test]
    fn test_read_all_requirements_no_specs() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory without specs
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
        std::fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute_read_all_requirements(&args, project_root).unwrap();

        assert!(result.contains("**Total**: 1 proposal, 1 tasks file, 0 specification(s)"));
    }

    #[test]
    fn test_read_all_requirements_missing_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory without proposal
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute_read_all_requirements(&args, project_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("proposal.md not found"));
    }

    #[test]
    fn test_read_all_requirements_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "nonexistent"
        });

        let result = execute_read_all_requirements(&args, project_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_is_git_repo() {
        // This test will pass or fail depending on whether we're in a git repo
        // Just verify it doesn't panic
        let _ = is_git_repo();
    }

    #[test]
    fn test_git_helpers_in_git_repo() {
        // Only run if we're in a git repo
        if is_git_repo() {
            let branch = get_current_branch();
            assert!(branch.is_ok());

            let status = run_git_command(&["status", "--short"]);
            assert!(status.is_ok());
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/implementation.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
    description: "Regression tests for implementation support helpers and requirement reading."
```
