---
id: sdd-tools-review-change-spec-prompt
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change spec prompt

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
/// Build REVIEW prompt for a spec.
async fn build_review_prompt(
    change_id: &str,
    spec_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    // Build alignment report for injection into prompt
    let spec_abs_path = match group_id {
        Some(gid) => project_root.join(format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        )),
        None => project_root.join(format!(".aw/changes/{}/specs/{}.md", change_id, spec_id)),
    };
    let alignment_section = if spec_abs_path.exists() {
        build_alignment_report(&spec_abs_path)
    } else {
        String::new()
    };

    let prompt = format!(
        r#"# Task: Review Spec '{spec_id}' for Change '{change_id}'

{alignment_section}## Instructions

1. **Run automated validation**:
   `score workflow validate-spec-completeness {change_id} --spec-id {spec_id}`
2. **Read the spec**:
   `.aw/changes/{change_id}/specs/{spec_id}.md`
3. **Read the proposal** for context routing
4. **Evaluate against checklist**:
   - Overview is substantive (>= 50 chars)
   - Requirements are well-defined with IDs and descriptions
   - At least one scenario per requirement
   - Diagrams are relevant and correct (if present)
   - API specs are valid (if present)
   - Changes list covers all affected files
   - No duplicate section types in this spec file
   - Sections follow dependency order: data → behavior → interface → test → changes
5. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve. Report every problem NOW — do not hold back issues for a future round.
6. **Determine verdict**: APPROVED / REVIEWED / REJECTED
7. **Identify problem sections**: If not APPROVED, list which sections need work
8. Write the review

## Verdict Guidelines

- **APPROVED**: Passes all checklist items, spec is implementation-ready
- **REVIEWED**: Missing elements, unclear requirements, or insufficient scenarios
- **REJECTED**: Fundamental design problems, wrong approach

## CLI Commands

```
# Validate spec completeness
score workflow validate-spec-completeness {change_id} --spec-id {spec_id}

# Read spec
Read file: .aw/changes/{change_id}/specs/{spec_id}.md

# Write review (write payload JSON first, then run)
score artifact review-change-spec {change_id} .aw/changes/{change_id}/payloads/review-change-spec.json
```"#,
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewChangeSpec);

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("review_spec_{}", spec_id),
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
  - path: projects/agentic-workflow/src/tools/review_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "build_review_prompt"
    description: "Review prompt builder with alignment report and executor dispatch metadata."
```
