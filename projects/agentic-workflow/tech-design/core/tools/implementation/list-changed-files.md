---
id: sdd-tools-implementation-list-changed-files-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# List Changed Files Tool

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
/// Get the tool definition for list_changed_files
pub fn list_changed_files_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_list_changed_files".to_string(),
        description: "List changed files with detailed statistics (additions/deletions)"
            .to_string(),
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
                    "description": "The change ID to list files for"
                },
                "base_branch": {
                    "type": "string",
                    "description": "Base branch to compare against (default: 'main')",
                    "default": "main"
                },
                "filter": {
                    "type": "string",
                    "description": "Optional filter pattern (simple string match on file path)"
                }
            }
        }),
    }
}

/// Execute the list_changed_files tool
pub fn execute_list_changed_files(args: &Value, _project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let base_branch =
        get_optional_string(args, "base_branch").unwrap_or_else(|| "main".to_string());
    let filter = get_optional_string(args, "filter");

    if !is_git_repo() {
        anyhow::bail!("Not in a git repository");
    }

    let mut output = String::new();
    output.push_str(&format!("# Changed Files for: {}\n\n", change_id));

    if let Some(ref f) = filter {
        output.push_str(&format!("**Filter**: `{}`\n\n", f));
    }

    // Get numstat output
    let numstat = run_git_command(&["diff", "--numstat", &base_branch])?;

    if numstat.is_empty() || numstat.starts_with("⚠️") {
        output.push_str("*No changes detected*\n");
        return Ok(output);
    }

    // Parse numstat output
    #[derive(Debug)]
    struct FileStat {
        added: String,
        removed: String,
        path: String,
    }

    let mut files: Vec<FileStat> = Vec::new();
    let mut total_added = 0;
    let mut total_removed = 0;

    for line in numstat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 3 {
            continue;
        }

        let path = parts[2].to_string();

        // Apply filter if specified
        if let Some(ref f) = filter {
            if !path.contains(f) {
                continue;
            }
        }

        let added_str = parts[0].to_string();
        let removed_str = parts[1].to_string();

        // Parse numbers (handle binary files marked with '-')
        if added_str != "-" {
            if let Ok(n) = added_str.parse::<usize>() {
                total_added += n;
            }
        }
        if removed_str != "-" {
            if let Ok(n) = removed_str.parse::<usize>() {
                total_removed += n;
            }
        }

        files.push(FileStat {
            added: added_str,
            removed: removed_str,
            path,
        });
    }

    if files.is_empty() {
        output.push_str("*No matching files found*\n");
        return Ok(output);
    }

    // Format as markdown table
    output.push_str("| File | Status | +Lines | -Lines |\n");
    output.push_str("|------|--------|--------|--------|\n");

    for file in &files {
        let status = if file.added == "-" && file.removed == "-" {
            "Binary"
        } else if file.removed == "0" {
            "Added"
        } else if file.added == "0" {
            "Deleted"
        } else {
            "Modified"
        };

        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            file.path, status, file.added, file.removed
        ));
    }

    output.push_str("\n");
    output.push_str(&format!(
        "**Totals**: {} files, +{} lines, -{} lines\n",
        files.len(),
        total_added,
        total_removed
    ));

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
      - "list_changed_files_definition"
      - "execute_list_changed_files"
    description: "Tool definition and execution for changed-file listing with additions/deletions statistics."
```
