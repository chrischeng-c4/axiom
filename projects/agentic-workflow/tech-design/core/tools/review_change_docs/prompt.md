---
id: sdd-tools-review-change-docs-prompt
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change docs prompt

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
/// Build doc-reviewer prompt.
fn build_review_docs_prompt(
    change_id: &str,
    group_id: Option<&str>,
    _project_root: &Path,
) -> String {
    let spec_path_prefix = match group_id {
        Some(gid) => format!(".aw/changes/{}/groups/{}/specs", change_id, gid),
        None => format!(".aw/changes/{}/specs", change_id),
    };

    format!(
        r#"# Task: Review Docs for Change '{change_id}'

## Instructions

1. Read all change specs in `{spec_path_prefix}/`
2. Read the generated guide files
3. Verify accuracy by executing documented CLI commands
4. Evaluate ALL checklist items below
5. Write review via the artifact CLI command

## Review Checklist

### Hard Checklist (MUST ALL PASS for APPROVED)

- [HARD] Documented CLI commands produce correct output when executed
- [HARD] All change-spec requirements are reflected in the guide
- [HARD] No regression — existing documented features are still accurate

### Soft Checklist (Issues -> REVIEWED verdict)

- Audience-appropriate tone and detail level
- Includes practical examples for key workflows
- Logical flow and section organization

## Verdict Guidelines

- **APPROVED**: All hard checklist items pass, docs are accurate and complete
- **REVIEWED**: Hard checklist passes but has fixable soft issues
- **REJECTED**: Any hard checklist item fails

## CLI Commands

```
# Read specs
Glob pattern: {spec_path_prefix}/*.md

# Write review (write payload JSON first, then run)
score artifact review-change-docs {change_id} .aw/changes/{change_id}/payloads/review-change-docs.json
```"#,
    )
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
      - "build_review_docs_prompt"
    description: "Docs review prompt builder with group-aware spec path hint."
```
