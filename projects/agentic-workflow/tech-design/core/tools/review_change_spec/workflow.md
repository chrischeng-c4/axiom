---
id: sdd-tools-review-change-spec-workflow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change spec workflow

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 48 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 186 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 129 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 24 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute sdd_workflow_review_change_spec.
///
/// Returns review prompt for the current spec in Review sub-state.
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);

    match common::resolve_next_spec(&change_dir, &change_id)? {
        SpecSubState::Review { spec_id } => {
            build_review_prompt(&change_id, &spec_id, group_id.as_deref(), project_root).await
        }
        SpecSubState::AdvanceToImplementation => {
            // REQ: bug-create-change-spec-review-change-spec-routing-dead
            // All specs filled + complete + no pending review → advance to
            // implementation directly. Previously this fell into the `_` arm
            // and redirected to create_change_spec, which saw create_complete
            // and redirected back here ⇒ infinite loop.
            let result = json!({
                "status": "ok",
                "prompt": "All specs are create_complete with no pending review. Advancing to implementation.",
                "next_actions": [
                    workflow_common::next_action(
                        interface,
                        "sdd_workflow_create_change_implementation",
                        json!({"change_id": change_id}),
                    )
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
        _ => {
            // Still in Create/Revise/MainthreadMustFix — redirect back to create router
            let result = json!({
                "status": "ok",
                "prompt": "Spec is not in Review sub-state. Redirecting to router.",
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
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
  - path: projects/agentic-workflow/src/tools/review_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_workflow"
    description: "Review-change-spec workflow routing and implementation advance handling."
```
