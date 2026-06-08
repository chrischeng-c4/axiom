---
id: sdd-tools-review-reference-context-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review reference context artifact

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
// ─── Artifact Review ─────────────────────────────────────────────────────────

/// Execute sdd_artifact_review_reference_context.
///
/// Writes inline review section into `groups/{group_id}/reference_context.md`.
/// Handles APPROVED and auto-approve logic.
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let group_id = get_required_string(args, "group_id")?;
    let verdict = get_required_string(args, "verdict")?;
    let summary = get_required_string(args, "summary")?;
    let checklist = args
        .get("checklist_results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let issues = args
        .get("issues")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let caller = get_optional_string(args, "caller").unwrap_or_else(|| "reviewer".to_string());

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    let group_dir = change_dir.join("groups").join(&group_id);

    if !group_dir.exists() {
        anyhow::bail!("Group directory not found: groups/{}", group_id);
    }

    let artifact_path = group_dir.join("reference_context.md");
    if !artifact_path.exists() {
        anyhow::bail!(
            "Reference context artifact not found: groups/{}/reference_context.md",
            group_id
        );
    }

    // Read current content
    let content = std::fs::read_to_string(&artifact_path)?;

    // Strip old review section
    let stripped = review_helpers::strip_review_section(&content);

    // Get iteration number from revision count
    let rev_key = format!("ref_ctx:{}", group_id);
    let sm = StateManager::load(&change_dir)?;
    let iteration = sm.revision_count(&rev_key) as u64 + 1;
    drop(sm);

    // Build new review section
    let review_section = review_helpers::build_review_section(
        &verdict, &summary, &checklist, &issues, iteration, &caller, &change_id,
    );

    // Append review section
    let new_content = format!("{}\n\n{}", stripped, review_section);

    // Upsert review_verdict in frontmatter
    let final_content =
        review_helpers::upsert_frontmatter_field(&new_content, "review_verdict", &verdict);

    std::fs::write(&artifact_path, &final_content)?;

    // Phase stays at PostClarificationsCreated (reference context absorbed by issue lifecycle)
    {
        let mut sm = StateManager::load(&change_dir)?;
        if matches!(sm.phase(), StatePhase::ChangeInited) {
            // No phase transition needed — reference context review is internal
            sm.save()?;
        }
    }

    // Handle APPROVED or auto-approve
    let should_mark_done =
        if verdict.eq_ignore_ascii_case("APPROVED") || verdict.eq_ignore_ascii_case("PASS") {
            true
        } else {
            // Check revision count for auto-approve
            let sm = StateManager::load(&change_dir)?;
            let rev_count = sm.revision_count(&rev_key);
            rev_count >= 1
        };

    if should_mark_done {
        common::mark_group_done(&change_dir, &group_id)?;
    }

    let artifacts_written = vec![format!("groups/{}/reference_context.md", group_id)];

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_create_reference_context", json!({"change_id": change_id}))
        ]
    });

    Ok(serde_json::to_string_pretty(&result)?)
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
      - "execute_artifact"
    description: "Artifact writer for inline reference-context review sections and group completion state."
```
