---
id: sdd-tools-revise-reference-context-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise reference context artifact

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_reference_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 45 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 148 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 108 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// ─── Artifact Revise ─────────────────────────────────────────────────────────

/// Execute sdd_artifact_revise_reference_context.
///
/// Delegates to `create::execute_artifact()` for writing, then increments revision count.
/// This ensures auto-approve triggers regardless of whether the revise was done by an agent
/// or by mainthread (the workflow_common agent-dispatch post-hook only covers the agent path).
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let result = create::execute_artifact(args, project_root)?;

    // Increment revision count so auto-approve (threshold >= 1) can trigger on next review.
    let change_id = get_required_string(args, "change_id")?;
    let group_id = get_required_string(args, "group_id")?;
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    let rev_key = format!("ref_ctx:{}", group_id);
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
  - path: projects/agentic-workflow/src/tools/revise_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_artifact"
    description: "Artifact writer delegation for revised reference context plus revision-count tracking."
```
