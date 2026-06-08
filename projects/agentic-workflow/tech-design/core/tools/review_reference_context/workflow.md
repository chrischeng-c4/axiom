---
id: sdd-tools-review-reference-context-workflow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review reference context workflow

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_reference_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 47 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 160 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 121 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 23 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// ─── Workflow ─────────────────────────────────────────────────────────────────

/// Execute sdd_workflow_review_reference_context.
///
/// Returns review prompt for the current group in Review sub-state.
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    // Resolve current group — should be in Review sub-state
    match common::resolve_next_group(&change_dir)? {
        common::GroupSubState::Review { group_id } => {
            build_review_prompt(&change_id, &group_id, project_root).await
        }
        _ => {
            // Not in review sub-state — redirect back to router
            let result = json!({
                "status": "ok",
                "prompt": "Group is not in Review sub-state. Redirecting to router.",
                "group_id": null,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_create_reference_context", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_workflow"
    description: "Review-reference-context workflow routing for the current group sub-state."
```
