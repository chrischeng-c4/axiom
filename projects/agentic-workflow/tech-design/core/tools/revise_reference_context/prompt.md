---
id: sdd-tools-revise-reference-context-prompt
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise reference context prompt

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
// ─── Prompt Builder ──────────────────────────────────────────────────────────

/// Build REVISE prompt for a group's reference context.
async fn build_revise_prompt(
    change_id: &str,
    group_id: &str,
    project_root: &Path,
) -> Result<String> {
    let project_path = project_root.display();

    let prompt = format!(
        r#"# Task: Revise Reference Context for Group '{group_id}' (Change '{change_id}')

## Instructions

1. **Read artifact + review feedback**:
   `{project_path}/.aw/changes/{change_id}/groups/{group_id}/reference_context.md`
   Focus on the `# Reviews` section — list each issue to address.
2. **Read pre-clarifications** (confirm scope):
   `{project_path}/.aw/changes/{change_id}/groups/{group_id}/pre_clarifications.md`
3. **Address each issue one by one**: For each review issue:
   - Identify what needs to change (add spec? fix relevance? update key requirements?)
   - If a missing spec is mentioned, read it from `{project_path}/.aw/tech-design/`
   - Apply the fix to your specs array
4. **Self-verify**: Walk through each original review issue — is it resolved in the new specs array?
5. **Scope re-check**: Do the revised specs still cover all crates/areas from pre-clarifications?
6. Rewrite via artifact tool:

## CLI Commands

```
# Write revised artifact (write payload JSON first, then run)
score artifact revise-reference-context {change_id} .aw/changes/{change_id}/payloads/revise-reference-context.json
```"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseReferenceContext);

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "revise_reference_context",
        prompt,
        executor,
        json!({ "group_id": group_id }),
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
  - path: projects/agentic-workflow/src/tools/revise_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "build_revise_prompt"
    description: "Prompt builder for group reference-context revision."
```
