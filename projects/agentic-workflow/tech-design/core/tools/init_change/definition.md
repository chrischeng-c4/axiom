---
id: sdd-init-change-tool-definition-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: capability-to-epic-planning
    claim: capability-to-epic-planning
    coverage: full
    rationale: "Issue initialization and reference-context tools support work-item planning and projection into bounded changes."
---

# Init Change Tool Definition

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/init_change.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `cleanup_stale_worktree` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 360 | cleanup_stale_worktree(project_root: &Path, slug: &str) -> Result<()> |
| `create_worktree` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 328 | create_worktree(project_root: &Path, slug: &str) -> Result<PathBuf> |
| `definition` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 25 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 561 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_standalone` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 74 | execute_standalone(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
/// MCP tool definition for sdd_init_change
/// @spec projects/agentic-workflow/tech-design/core/logic/structured-issue.md#changes
/// @spec projects/agentic-workflow/tech-design/core/logic/state-machine.md#changes
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_init_change".to_string(),
        description: "Initialize a new change directory and sync workflow state to the issue frontmatter. Returns next_actions for workflow chaining.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "description"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change identifier (lowercase alphanumeric + hyphens)"
                },
                "description": {
                    "type": "string",
                    "description": "User's description of the change"
                },
                "issues": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Issue references (e.g. [\"#188\", \"#189\"]). Triggers fetch_issues."
                },
                "git_workflow": {
                    "type": "string",
                    "enum": ["new_branch", "in_place"],
                    "description": "Git workflow chosen by user"
                },
                "branch": {
                    "type": "string",
                    "description": "Git branch name. Recorded in STATE.yaml."
                }
            }
        }),
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/init_change.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:standardize-gap-sdd-init-change-tool-definition>"
    description: "Init-change MCP tool definition JSON schema for the workflow init surface."
```
