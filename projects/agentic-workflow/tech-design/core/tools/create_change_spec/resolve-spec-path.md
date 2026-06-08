---
id: sdd-tools-create-change-spec-resolve-spec-path
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change spec resolve spec path

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 55 | artifact_definition() -> ToolDefinition |
| `build_fill_prompt` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 736 | build_fill_prompt(     change_id: &str,     spec_id: &str,     section: &str,     group_id: Option<&str>,     project_root: &Path, ) -> Result<String> |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 338 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 120 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Resolve spec file path by scanning groups/*/specs/ and top-level specs/.
/// Eliminates the need for callers to pass group_id — the state machine structure
/// is the source of truth.
fn resolve_spec_path(change_dir: &Path, spec_id: &str) -> Result<std::path::PathBuf> {
    let filename = format!("{}.md", spec_id);

    // 1. Check groups/*/specs/
    let groups_dir = change_dir.join("groups");
    if groups_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&groups_dir) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("specs").join(&filename);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
    }

    // 2. Check top-level specs/
    let top_level = change_dir.join("specs").join(&filename);
    if top_level.exists() {
        return Ok(top_level);
    }

    anyhow::bail!(
        "Spec file not found: {}.md\nSearched: {}/groups/*/specs/ and {}/specs/\n\
         Call `score workflow create-change-spec` first to generate the skeleton.",
        spec_id,
        change_dir.display(),
        change_dir.display()
    )
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "resolve_spec_path"
    description: "Spec-path resolution helper for grouped and top-level specs."
```
