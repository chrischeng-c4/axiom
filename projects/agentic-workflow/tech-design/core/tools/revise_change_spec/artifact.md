---
id: sdd-tools-revise-change-spec-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change spec artifact

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 135 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 92 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 22 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute sdd_artifact_revise_change_spec.
///
/// Delegates to `create::execute_artifact()` — same write behavior.
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let result = create::execute_artifact(args, project_root)?;

    // Increment revision count so auto-approve (threshold >= 1) triggers on next review.
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    let rev_key = format!("spec:{}", spec_id);
    if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {
        sm.increment_revision_count(&rev_key);
        let _ = sm.save();
    }

    Ok(result)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/revise_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_artifact"
    description: "Revise-change-spec artifact delegation and revision counter update."
```
