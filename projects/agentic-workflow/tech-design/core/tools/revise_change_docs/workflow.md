---
id: sdd-tools-revise-change-docs-workflow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change docs workflow

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
/// Execute sdd_workflow_revise_change_docs.
///
/// Builds doc-writer prompt with review feedback included and dispatches
/// sdd-doc-writer agent for revision.
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);

    // Read review feedback
    let review_path = change_dir.join("docs_review/review.md");
    let review_content = if review_path.exists() {
        std::fs::read_to_string(&review_path)?
    } else {
        "No review feedback found.".to_string()
    };

    // Get current revision count
    let sm = StateManager::load(&change_dir)?;
    let rev_count = sm.revision_count("docs") as u64;
    drop(sm);

    let prompt = build_revise_docs_prompt(
        &change_id,
        &review_content,
        rev_count,
        group_id.as_deref(),
        project_root,
    );

    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseChangeDocs);

    let extra = json!({
        "revision_count": rev_count + 1,
    });

    workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "revise_change_docs",
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
  - path: projects/agentic-workflow/src/tools/revise_change_docs.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_workflow"
    description: "Revise-change-docs workflow prompt construction and dispatch."
```
