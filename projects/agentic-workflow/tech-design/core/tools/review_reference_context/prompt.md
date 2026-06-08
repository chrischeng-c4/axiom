---
id: sdd-tools-review-reference-context-prompt
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review reference context prompt

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
// ─── Prompt Builder ──────────────────────────────────────────────────────────

/// Build REVIEW prompt for a group's reference context.
async fn build_review_prompt(
    change_id: &str,
    group_id: &str,
    project_root: &Path,
) -> Result<String> {
    let project_path = project_root.display();

    let prompt = format!(
        r#"# Task: Review Reference Context for Group '{group_id}' (Change '{change_id}')

## Instructions

1. **Read pre-clarifications** (scope & requirements):
   `{project_path}/.aw/changes/{change_id}/groups/{group_id}/pre_clarifications.md`
2. **Read the reference context artifact**:
   `{project_path}/.aw/changes/{change_id}/groups/{group_id}/reference_context.md`
3. **Verify each spec entry**: For each spec listed in the artifact, read the actual spec under `{project_path}/.aw/tech-design/` to verify relevance and key requirements are accurate.
4. **Devil's advocate**: Actively check — what crates/areas from pre-clarifications have NO spec covering them?
5. **Evaluate checklist** (pass/fail each item independently):
   - All affected crates/areas from pre-clarifications are covered by at least one spec
   - Relevance scores are reasonable (high = directly implements, medium = related, low = background)
   - Key requirements listed per spec are accurate (match actual requirement IDs)
   - No irrelevant specs included
   - spec_plan: every entry has main_spec_ref set (not null)
   - spec_plan: sections are reasonable for the requirements
   - spec_plan: modify entries have valid source paths
   - spec_plan: main_spec_ref paths include a subfolder (not root-level under crate)
   - spec_plan: each spec file covers exactly one logical unit (not multiple unrelated concerns)
   - spec_plan: no spec file would require duplicate section types (split into separate files if needed)
   - spec_plan: spec paths mirror source structure (interfaces/, logic/, generate/)
6. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve kicks in. Do NOT hold back issues for a future round — every problem must be reported NOW. Scan the entire artifact exhaustively before writing the verdict.
7. **Separate observations from verdict**: First list all findings, then decide verdict based on evidence.
8. Write review verdict:

## CLI Commands

```
# Write review artifact (write payload JSON first, then run)
score artifact review-reference-context {change_id} .aw/changes/{change_id}/payloads/review-reference-context.json
```"#,
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewReferenceContext);

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "review_reference_context",
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
  - path: projects/agentic-workflow/src/tools/review_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "build_review_prompt"
    description: "Prompt builder for group reference-context review."
```
