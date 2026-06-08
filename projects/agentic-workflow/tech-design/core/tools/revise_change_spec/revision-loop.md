---
id: sdd-tools-revise-change-spec-revision-loop
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change spec revision loop

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
/// Handle the revise sub-state: re-enter fill loop for flagged sections.
async fn handle_revise_sub_state(
    change_id: &str,
    spec_id: &str,
    change_dir: &Path,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    // Use group-aware path lookup (checks groups/*/specs/ first, then specs/)
    let spec_path = common::find_spec_path(change_dir, spec_id);
    let content = std::fs::read_to_string(&spec_path)?;

    // Read problem_sections from frontmatter (set by review artifact tool)
    let problem_sections = read_problem_sections(&content);

    if problem_sections.is_empty() {
        // No specific sections flagged — revise overview + requirements by default
        let prompt = build_revise_prompt(
            change_id,
            spec_id,
            &["overview", "requirements"],
            group_id,
            project_root,
        )
        .await?;
        return Ok(prompt);
    }

    // Check which flagged sections are not yet re-filled
    let filled_sections = common::read_filled_sections(&content);
    let next_section = problem_sections
        .iter()
        .find(|s| !filled_sections.contains(s));

    if let Some(section) = next_section {
        create::build_fill_prompt(change_id, spec_id, section, group_id, project_root).await
    } else {
        // All problem sections re-filled — strip old review, mark complete
        let stripped = review_helpers::strip_review_section(&content);
        let marked = review_helpers::upsert_frontmatter_field(&stripped, "create_complete", "true");
        // Remove problem_sections and filled_sections (clean state)
        let cleaned = review_helpers::remove_frontmatter_field(&marked, "problem_sections");
        let cleaned = review_helpers::remove_frontmatter_field(&cleaned, "filled_sections");
        // Append fresh Reviews section
        let final_content = format!("{}\n\n# Reviews\n", cleaned.trim_end());
        std::fs::write(&spec_path, &final_content)?;

        // Update phase to revised
        workflow_common::update_phase(change_dir, StatePhase::ChangeSpecRevised)?;

        // Redirect back to workflow router (will go to review)
        let interface = workflow_common::load_interface(project_root);
        let result = json!({
            "status": "ok",
            "spec_id": spec_id,
            "message": "Revision complete. Flagged sections re-filled.",
            "next_actions": [
                workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
            ]
        });
        Ok(serde_json::to_string_pretty(&result)?)
    }
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
      - "handle_revise_sub_state"
    description: "Revision loop for flagged spec sections and completion routing back to review."
```
