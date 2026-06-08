---
id: sdd-tools-review-change-docs-workflow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change docs workflow

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_change_docs.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_change_docs.rs | function | pub | 44 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_change_docs.rs | function | pub | 150 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_change_docs.rs | function | pub | 97 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_change_docs.rs | function | pub | 19 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute sdd_workflow_review_change_docs.
///
/// Builds doc-reviewer prompt with review checklist and dispatches
/// sdd-doc-reviewer agent.
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);
    let prompt = build_review_docs_prompt(&change_id, group_id.as_deref(), project_root);

    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewChangeDocs);

    let extra = json!({
        "review_checklist": {
            "hard": [
                "Documented CLI commands produce correct output when executed",
                "All change-spec requirements are reflected in the guide",
                "No regression — existing documented features are still accurate"
            ],
            "soft": [
                "Audience-appropriate tone and detail level",
                "Includes practical examples for key workflows",
                "Logical flow and section organization"
            ]
        }
    });

    workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "review_change_docs",
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
  - path: projects/agentic-workflow/src/tools/review_change_docs.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_workflow"
    description: "Review-change-docs workflow prompt construction and dispatch metadata."
```
