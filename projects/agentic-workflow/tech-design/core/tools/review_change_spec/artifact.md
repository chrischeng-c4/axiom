---
id: sdd-tools-review-change-spec-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change spec artifact

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
/// Execute sdd_artifact_review_change_spec.
///
/// Writes inline review section into `specs/{spec_id}.md`.
/// Updates `review_verdict` in frontmatter and appends `# Reviews` section.
/// For non-APPROVED verdicts, also stores `problem_sections` for revise flow.
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
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
    let problem_sections: Vec<String> = args
        .get("problem_sections")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let caller = get_optional_string(args, "caller").unwrap_or_else(|| "reviewer".to_string());

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    // Validate verdict
    if !["APPROVED", "REVIEWED", "REJECTED"].contains(&verdict.as_str()) {
        anyhow::bail!("verdict must be 'APPROVED', 'REVIEWED', or 'REJECTED'");
    }

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    // Use group-aware path lookup (checks groups/*/specs/ first, then specs/)
    let spec_path = common::find_spec_path(&change_dir, &spec_id);

    if !spec_path.exists() {
        anyhow::bail!(
            "Spec file not found: {}.md (searched groups and specs/)",
            spec_id
        );
    }

    // Read current content
    let content = std::fs::read_to_string(&spec_path)?;

    // Strip old review section
    let stripped = review_helpers::strip_review_section(&content);

    // Get iteration number from revision count
    let rev_key = format!("spec:{}", spec_id);
    let sm = StateManager::load(&change_dir)?;
    let iteration = sm.revision_count(&rev_key) as u64 + 1;
    drop(sm);

    // Build new review section
    let review_section = review_helpers::build_review_section(
        &verdict, &summary, &checklist, &issues, iteration, &caller, &change_id,
    );

    // Append review section
    let new_content = format!("{}\n\n{}", stripped, review_section);

    // Update frontmatter based on verdict
    let final_content = if verdict == "APPROVED" {
        // Clean state: remove review_verdict and iteration markers
        let cleaned = review_helpers::remove_frontmatter_field(&new_content, "review_verdict");
        let cleaned = review_helpers::remove_frontmatter_field(&cleaned, "review_iteration");
        let cleaned = review_helpers::remove_frontmatter_field(&cleaned, "problem_sections");
        review_helpers::remove_frontmatter_field(&cleaned, "filled_sections")
    } else {
        // Upsert review_verdict and problem_sections
        let updated =
            review_helpers::upsert_frontmatter_field(&new_content, "review_verdict", &verdict);
        let updated = review_helpers::upsert_frontmatter_field(
            &updated,
            "review_iteration",
            &iteration.to_string(),
        );
        // Remove filled_sections for fresh revise tracking
        let updated = review_helpers::remove_frontmatter_field(&updated, "filled_sections");
        if !problem_sections.is_empty() {
            let sections_str = format!("[{}]", problem_sections.join(", "));
            review_helpers::upsert_frontmatter_field(&updated, "problem_sections", &sections_str)
        } else {
            updated
        }
    };

    std::fs::write(&spec_path, &final_content)?;

    // Phase advance moved to `score workflow validate` (three-role-contract R8).
    // The artifact CLI only writes payload + files; SubagentStop hook advances phase.

    let artifacts_written = vec![format!("specs/{}.md", spec_id)];

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
        ]
    });

    Ok(serde_json::to_string_pretty(&result)?)
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
      - "<handwrite-gap:missing-generator:sdd-review-spec-artifact-flow>"
    description: "Review-change-spec artifact review persistence and frontmatter updates."
```
