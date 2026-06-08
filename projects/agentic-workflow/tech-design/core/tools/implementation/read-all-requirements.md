---
id: sdd-tools-implementation-read-all-requirements-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Read All Requirements Tool

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
/// Get the tool definition for read_all_requirements
pub fn read_all_requirements_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_read_all_requirements".to_string(),
        description: "Read all requirement files (proposal, tasks, specs) for a change in one call"
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
                    "description": "The change ID to read requirements for"
                }
            }
        }),
    }
}

/// Execute the read_all_requirements tool
pub fn execute_read_all_requirements(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found at {}",
            change_id,
            change_dir.display()
        );
    }

    let mut output = String::new();
    output.push_str(&format!("# Requirements for Change: {}\n\n", change_id));

    // Read proposal.md (required)
    let proposal_path = change_dir.join("proposal.md");
    if !proposal_path.exists() {
        anyhow::bail!("proposal.md not found for change '{}'", change_id);
    }
    let proposal_content = std::fs::read_to_string(&proposal_path)?;
    output.push_str("## Proposal\n\n");
    output.push_str(&proposal_content);
    output.push_str("\n\n---\n\n");

    // Read tasks.md (required)
    let tasks_path = change_dir.join("tasks.md");
    if !tasks_path.exists() {
        anyhow::bail!("tasks.md not found for change '{}'", change_id);
    }
    let tasks_content = std::fs::read_to_string(&tasks_path)?;
    output.push_str("## Tasks\n\n");
    output.push_str(&tasks_content);
    output.push_str("\n\n---\n\n");

    // Read all specs (optional)
    let specs_dir = change_dir.join("specs");
    let mut spec_count = 0;
    if specs_dir.exists() {
        let mut spec_files = Vec::new();
        for entry in std::fs::read_dir(&specs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_stem() {
                    let name_str = name.to_string_lossy();
                    // Skip skeleton files
                    if !name_str.starts_with('_') {
                        spec_files.push((name_str.to_string(), path));
                    }
                }
            }
        }

        spec_files.sort_by(|a, b| a.0.cmp(&b.0));

        if !spec_files.is_empty() {
            output.push_str("## Specifications\n\n");
            for (name, path) in spec_files {
                let spec_content = std::fs::read_to_string(&path)?;
                output.push_str(&format!("### Spec: {}\n\n", name));
                output.push_str(&spec_content);
                output.push_str("\n\n");
                spec_count += 1;
            }
            output.push_str("---\n\n");
        }
    }

    // Summary
    output.push_str(&format!(
        "**Total**: 1 proposal, 1 tasks file, {} specification(s)\n",
        spec_count
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
      - "read_all_requirements_definition"
      - "execute_read_all_requirements"
    description: "Tool definition and execution for reading proposal, tasks, and spec requirements for a change."
```
