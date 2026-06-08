---
id: sdd-tools-implementation-read-summary-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Read Implementation Summary Tool

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
/// Get the tool definition for read_implementation_summary
pub fn read_implementation_summary_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_read_implementation_summary".to_string(),
        description: "Get git diff summary and commit log for implementation review".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID to get implementation summary for"
                },
                "base_branch": {
                    "type": "string",
                    "description": "Base branch to compare against (default: 'main')",
                    "default": "main"
                }
            }
        }),
    }
}

/// Execute the read_implementation_summary tool
pub fn execute_read_implementation_summary(args: &Value, _project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let base_branch =
        get_optional_string(args, "base_branch").unwrap_or_else(|| "main".to_string());

    if !is_git_repo() {
        anyhow::bail!("Not in a git repository");
    }

    let mut output = String::new();
    output.push_str(&format!("# Implementation Summary for: {}\n\n", change_id));

    // Branch validation
    match validate_branch(&change_id) {
        Ok((current_branch, is_match)) => {
            output.push_str(&format!("**Current Branch**: `{}`\n", current_branch));
            if !is_match {
                output.push_str(&format!(
                    "⚠️ **Warning**: Expected branch `cclab/{}` but on `{}`\n",
                    change_id, current_branch
                ));
            }
            output.push_str("\n");
        }
        Err(e) => {
            output.push_str(&format!(
                "⚠️ **Warning**: Could not verify branch: {}\n\n",
                e
            ));
        }
    }

    // Commits ahead of base
    let commits_ahead =
        run_git_command(&["rev-list", "--count", &format!("{}..HEAD", base_branch)])?;
    output.push_str(&format!(
        "**Commits ahead of {}**: {}\n\n",
        base_branch, commits_ahead
    ));

    // Changed files (name-status)
    output.push_str("## Changed Files\n\n");
    let name_status = run_git_command(&["diff", "--name-status", &base_branch])?;
    if name_status.is_empty() {
        output.push_str("*No changes detected*\n\n");
    } else {
        output.push_str("```\n");
        output.push_str(&name_status);
        output.push_str("\n```\n\n");
    }

    // Diff statistics
    output.push_str("## Diff Statistics\n\n");
    let diff_stat = run_git_command(&["diff", "--stat", &base_branch])?;
    if diff_stat.is_empty() {
        output.push_str("*No changes*\n\n");
    } else {
        output.push_str("```\n");
        output.push_str(&diff_stat);
        output.push_str("\n```\n\n");
    }

    // Commit log
    output.push_str("## Commit Log\n\n");
    let commit_log = run_git_command(&["log", "--oneline", &format!("{}..HEAD", base_branch)])?;
    if commit_log.is_empty() {
        output.push_str("*No commits*\n\n");
    } else {
        output.push_str("```\n");
        output.push_str(&commit_log);
        output.push_str("\n```\n\n");
    }

    output.push_str("---\n\n");
    output.push_str(
        "💡 **Note**: For detailed code review, use the `Read` tool to examine specific files.\n",
    );

    Ok(output)
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
      - "read_implementation_summary_definition"
      - "execute_read_implementation_summary"
    description: "Tool definition and execution for git-based implementation summary generation."
```
