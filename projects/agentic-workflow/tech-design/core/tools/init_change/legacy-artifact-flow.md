---
id: sdd-init-change-legacy-artifact-flow-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: capability-to-epic-planning
    claim: capability-to-epic-planning
    coverage: full
    rationale: "Issue initialization and reference-context tools support work-item planning and projection into bounded changes."
---

# Init Change Legacy Artifact Flow

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
/// @spec projects/agentic-workflow/tech-design/core/logic/structured-issue.md#changes
/// @spec projects/agentic-workflow/tech-design/core/logic/state-machine.md#requirements
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = super::get_required_string(args, "change_id")?;
    let payload = args.get("payload").cloned().unwrap_or_else(|| json!({}));

    let description = payload
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("payload.description is required for change/create"))?
        .to_string();

    let issue_refs: Option<Vec<String>> =
        payload
            .get("issue_refs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

    let git_workflow = payload
        .get("git_workflow")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    create_change_internal(
        project_root,
        &change_id,
        &description,
        issue_refs.as_deref(),
        git_workflow.as_deref(),
        None,
    )?;

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R7
    // R7: user_input.md is no longer generated. STATE.yaml is no longer the
    // primary store — workflow fields live in the issue frontmatter.
    let mut result = json!({
        "status": "ok",
        "change_id": change_id,
        "artifact": "change",
        "action": "create",
        "artifacts_written": [],
        "phase": "clarified",
    });

    if issue_refs.as_ref().is_some_and(|r| !r.is_empty()) {
        result["has_issues"] = json!(true);
    }

    Ok(serde_json::to_string_pretty(&result)?)
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
      - "<handwrite-tracker:standardize-gap-sdd-init-change-legacy-artifact-flow>"
    description: "Legacy artifact create entry point for sdd_write_artifact change/create compatibility."
```
