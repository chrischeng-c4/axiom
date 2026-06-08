---
id: sdd-tools-revise-change-docs-prompt
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change docs prompt

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
fn build_revise_docs_prompt(
    change_id: &str,
    review_content: &str,
    revision_count: u64,
    group_id: Option<&str>,
    _project_root: &Path,
) -> String {
    let spec_path_prefix = match group_id {
        Some(gid) => format!(".aw/changes/{}/groups/{}/specs", change_id, gid),
        None => format!(".aw/changes/{}/specs", change_id),
    };

    format!(
        r#"# Task: Revise Docs for Change '{change_id}' (Revision {rev_num})

## Instructions

1. Read the review feedback below
2. Read the current guide files and change specs
3. Address all issues identified in the review
4. Write revised sections via the artifact CLI command

## Review Feedback

{review_content}

## Guidelines

- Fix all accuracy issues identified in the review
- Address completeness gaps
- Improve audience fit where noted
- Re-verify CLI command accuracy after changes

## CLI Commands

```
# Read specs
Glob pattern: {spec_path_prefix}/*.md

# Read review
Read file: .aw/changes/{change_id}/docs_review/review.md

# Write revised docs (write payload JSON first, then run)
score artifact revise-change-docs {change_id} .aw/changes/{change_id}/payloads/revise-change-docs.json
```"#,
        rev_num = revision_count + 1,
    )
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
      - "build_revise_docs_prompt"
      - "<module-trailer>"
    description: "Prompt builder for docs revisions from review feedback."
```
