---
id: sdd-tools-create-reference-context-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create reference context artifact

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_reference_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_reference_context.rs | function | pub | 51 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_reference_context.rs | function | pub | 269 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_reference_context.rs | function | pub | 168 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_reference_context.rs | function | pub | 26 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// @spec projects/agentic-workflow/tech-design/core/logic/remaining-fixes.md#changes
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R1
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let group_id = get_required_string(args, "group_id")?;
    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    let group_dir = change_dir.join("groups").join(&group_id);

    if !group_dir.exists() {
        anyhow::bail!("Group directory not found: groups/{}", group_id);
    }

    let artifact_path = group_dir.join("reference_context.md");

    // Detect mode: section-loop vs legacy
    let section = get_optional_string(args, "section");
    let content_arg = get_optional_string(args, "content");

    if let (Some(section), Some(content)) = (section, content_arg) {
        // Section-loop mode
        return execute_artifact_section_loop(
            &change_id,
            &group_id,
            &section,
            &content,
            &artifact_path,
            &change_dir,
            interface,
        );
    }

    // Legacy mode: full specs array
    let specs = args
        .get("specs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing required array field: specs (or use section+content for section-loop mode)"))?;

    if specs.is_empty() {
        anyhow::bail!("specs array must not be empty");
    }

    let spec_plan = args.get("spec_plan").and_then(|v| v.as_array());

    let today = Local::now().format("%Y-%m-%d").to_string();
    let content = common::render_specs_markdown(&change_id, &group_id, &today, specs, spec_plan);
    std::fs::write(&artifact_path, &content)?;

    if let Some(plan_entries) = spec_plan {
        if !plan_entries.is_empty() {
            common::write_spec_plan_yaml(&group_dir, plan_entries)?;
        }
    }

    // Phase stays at PostClarificationsCreated (reference context absorbed by issue lifecycle)
    let mut sm = StateManager::load(&change_dir)?;
    if matches!(sm.phase(), StatePhase::ChangeInited) {
        // Phase stays at PostClarificationsCreated - reference context is now
        // an internal artifact within the issue lifecycle, not a separate phase.
        sm.save()?;
    }

    let artifacts_written = vec![format!("groups/{}/reference_context.md", group_id)];
    let next_actions = json!([workflow_common::next_action(
        interface,
        "sdd_workflow_create_reference_context",
        json!({"change_id": change_id})
    )]);

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "next_actions": next_actions
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:sdd-reference-context-artifact-flow>"
    description: "Create-reference-context artifact writer for legacy and section-loop modes."
```
