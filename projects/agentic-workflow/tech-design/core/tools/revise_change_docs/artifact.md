---
id: sdd-tools-revise-change-docs-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change docs artifact

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_change_docs.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_change_docs.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_change_docs.rs | function | pub | 157 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_change_docs.rs | function | pub | 96 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_change_docs.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute sdd_artifact_revise_change_docs.
///
/// Delegates to create_change_docs::execute_artifact() for the actual write.
/// Increments revision count in STATE.yaml task_revisions.
/// Updates phase to DocsRevised → next_action points to review.
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    // Delegate write to create artifact
    let result = create_change_docs::execute_artifact(args, project_root)?;

    // Increment docs revision count
    let change_id = get_required_string(args, "change_id")?;
    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);

    if let Ok(mut sm) = StateManager::load(&change_dir) {
        sm.increment_revision_count("docs");
        let _ = sm.save();
    }

    // Override phase to DocsRevised (create artifact set it to DocsCreated)
    workflow_common::update_phase(&change_dir, StatePhase::DocsRevised)?;

    // Parse result and override next_actions to point to review
    let mut parsed: Value = serde_json::from_str(&result)?;
    let interface = workflow_common::load_interface(project_root);
    let na = workflow_common::next_action(
        interface,
        "sdd_workflow_review_change_docs",
        json!({"change_id": change_id}),
    );

    if let Some(obj) = parsed.as_object_mut() {
        obj.insert("next_actions".to_string(), json!([na]));
        // Add revision count to response
        if let Ok(sm) = StateManager::load(&change_dir) {
            let rev_count = sm.revision_count("docs");
            obj.insert("revision_count".to_string(), json!(rev_count));
        }
    }

    Ok(serde_json::to_string_pretty(&parsed)?)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/revise_change_docs.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_artifact"
    description: "Revise-change-docs artifact delegation, revision counter, and response override."
```
