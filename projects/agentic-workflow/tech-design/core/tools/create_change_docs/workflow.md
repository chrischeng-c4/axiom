---
id: sdd-tools-create-change-docs-workflow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change docs workflow

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_docs.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 45 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 199 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 94 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 20 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute sdd_workflow_create_change_docs.
///
/// Resolves matched doc targets from `[agentic_workflow.docs]` config, builds doc-writer
/// prompt, and dispatches sdd-doc-writer agent.
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    // Load docs config — if absent, skip docs phase
    let docs_config = match SddConfig::load_validated(project_root) {
        Ok(config) => config.docs,
        Err(_) => None,
    };

    let docs_config = match docs_config {
        Some(dc) => dc,
        None => {
            // No [agentic_workflow.docs] config — skip docs phase, advance to merge
            let na = workflow_common::next_action(
                interface,
                "sdd_workflow_create_change_merge",
                json!({"change_id": change_id}),
            );
            let result = json!({
                "status": "skip",
                "skip_reason": "No [agentic_workflow.docs] config section — docs phase skipped.",
                "next_actions": [na]
            });
            // Advance phase directly to ChangeMergeCreated
            workflow_common::update_phase(&change_dir, StatePhase::ChangeMergeCreated)?;
            return Ok(serde_json::to_string_pretty(&result)?);
        }
    };

    // Resolve matched targets: intersect config targets with change-affected crates
    let affected_crates = resolve_affected_crates(&change_dir);
    let matched_targets: Vec<Value> = docs_config
        .targets
        .iter()
        .filter(|t| affected_crates.contains(&t.crate_name))
        .map(|t| {
            json!({
                "crate": t.crate_name,
                "guide": t.guide,
                "sections": t.sections,
                "audience": t.audience,
            })
        })
        .collect();

    if matched_targets.is_empty() {
        // No crate intersection — skip docs phase
        let na = workflow_common::next_action(
            interface,
            "sdd_workflow_create_change_merge",
            json!({"change_id": change_id}),
        );
        let result = json!({
            "status": "skip",
            "skip_reason": "No matching crates between [agentic_workflow.docs] targets and change-affected crates.",
            "next_actions": [na]
        });
        workflow_common::update_phase(&change_dir, StatePhase::ChangeMergeCreated)?;
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    // Build doc-writer prompt
    let group_id = workflow_common::resolve_single_group_id(&change_dir);
    let prompt = build_create_docs_prompt(
        &change_id,
        &matched_targets,
        group_id.as_deref(),
        project_root,
    );

    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeDocs);

    let extra = json!({
        "targets": matched_targets,
    });

    workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "create_change_docs",
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_docs.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_workflow"
    description: "Create-change-docs workflow routing and docs target matching."
```
