---
id: sdd-tools-create-change-docs-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change docs artifact

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_docs.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 45 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 199 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 94 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 20 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute sdd_artifact_create_change_docs.
///
/// Writes sections_content map to guide_path file. Merges new sections into
/// existing guide (preserves unchanged sections). Updates STATE.yaml phase
/// to DocsCreated.
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let target_crate = get_required_string(args, "target_crate")?;
    let guide_path = get_required_string(args, "guide_path")?;
    let summary = get_required_string(args, "summary")?;
    let sections_content = args
        .get("sections_content")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing required object field: sections_content"))?;

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let _change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    let abs_guide_path = project_root.join(&guide_path);

    // Ensure parent directory exists
    if let Some(parent) = abs_guide_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Read existing guide content (if any)
    let existing_content = if abs_guide_path.exists() {
        std::fs::read_to_string(&abs_guide_path)?
    } else {
        String::new()
    };

    // Merge sections: replace existing sections, append new ones
    let merged = merge_guide_sections(&existing_content, sections_content);
    std::fs::write(&abs_guide_path, &merged)?;

    let sections_updated: Vec<String> = sections_content.keys().cloned().collect();

    // Phase advance moved to `score workflow validate` (three-role-contract R8).

    let na = workflow_common::next_action(
        interface,
        "sdd_workflow_review_change_docs",
        json!({"change_id": change_id}),
    );

    let result = json!({
        "status": "ok",
        "artifacts_written": [guide_path],
        "guide_path": guide_path,
        "target_crate": target_crate,
        "sections_updated": sections_updated,
        "summary": summary,
        "next_actions": [na]
    });
    Ok(serde_json::to_string_pretty(&result)?)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_docs.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:sdd-docs-artifact-write-flow>"
    description: "Create-change-docs artifact write flow and next-action response."
```
