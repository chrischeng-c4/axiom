---
id: sdd-tools-revise-change-spec-frontmatter-and-prompt
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change spec frontmatter and prompt

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 135 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 92 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 22 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Read `problem_sections` from spec frontmatter.
fn read_problem_sections(content: &str) -> Vec<String> {
    if !content.starts_with("---\n") {
        return vec![];
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return vec![],
    };
    let fm = &content[4..closing];

    let mut in_field = false;
    let mut sections = Vec::new();
    for line in fm.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("problem_sections:") {
            in_field = true;
            let after = trimmed.trim_start_matches("problem_sections:").trim();
            if after.starts_with('[') && after.ends_with(']') {
                let inner = &after[1..after.len() - 1];
                for item in inner.split(',') {
                    let s = item.trim().trim_matches('"').trim_matches('\'');
                    if !s.is_empty() {
                        sections.push(s.to_string());
                    }
                }
                return sections;
            }
            continue;
        }
        if in_field {
            if trimmed.starts_with("- ") {
                let item = trimmed
                    .trim_start_matches("- ")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                sections.push(item.to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        }
    }
    sections
}

/// Build REVISE prompt for multiple flagged sections.
async fn build_revise_prompt(
    change_id: &str,
    spec_id: &str,
    problem_sections: &[&str],
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();
    let sections_list = problem_sections.join(", ");

    let prompt = format!(
        r#"# Task: Revise Spec '{spec_id}' for Change '{change_id}'

## Instructions

1. Read the spec and its review:
   `.aw/changes/{change_id}/specs/{spec_id}.md`
2. Address review issues in these sections: {sections_list}
3. Run `score artifact revise-change-spec` for each section that needs revision

## CLI Commands

```
# Read spec
Read file: .aw/changes/{change_id}/specs/{spec_id}.md

# Write revised section (write payload JSON first, then run)
score artifact revise-change-spec {change_id} .aw/changes/{change_id}/payloads/revise-change-spec.json
```"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseChangeSpec);

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("revise_spec_{}", spec_id),
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
  - path: projects/agentic-workflow/src/tools/revise_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "read_problem_sections"
      - "build_revise_prompt"
      - "<module-trailer>"
    description: "Problem-section frontmatter parsing and fallback revise prompt construction."
```
