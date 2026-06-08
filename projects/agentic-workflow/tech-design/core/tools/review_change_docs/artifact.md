---
id: sdd-tools-review-change-docs-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change docs artifact

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
/// Execute sdd_artifact_review_change_docs.
///
/// Writes verdict (APPROVED/REVIEWED/REJECTED) + review_notes.
/// Stores cli_verification_results. Updates STATE.yaml phase to DocsReviewed.
/// On APPROVED → next_action points to sdd_workflow_create_change_merge.
/// On REVIEWED/REJECTED → next_action points to sdd_workflow_revise_change_docs.
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let verdict = get_required_string(args, "verdict")?;
    let review_notes = get_required_string(args, "review_notes")?;
    let cli_results = args
        .get("cli_verification_results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let _caller = get_optional_string(args, "caller").unwrap_or_else(|| "doc-reviewer".to_string());

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    if !["APPROVED", "REVIEWED", "REJECTED"].contains(&verdict.as_str()) {
        anyhow::bail!("verdict must be 'APPROVED', 'REVIEWED', or 'REJECTED'");
    }

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);

    // Write review file
    let review_dir = change_dir.join("docs_review");
    std::fs::create_dir_all(&review_dir)?;

    let review_content = build_review_file(&verdict, &review_notes, &cli_results);
    let review_path = review_dir.join("review.md");
    std::fs::write(&review_path, &review_content)?;

    // Phase advance moved to `score workflow validate` (three-role-contract R8).

    // Check for auto-approve: if revision count >= 1 and verdict is REVIEWED, auto-approve
    let sm = StateManager::load(&change_dir)?;
    let rev_count = sm.revision_count("docs") as u64;
    drop(sm);

    let auto_approved = verdict != "APPROVED" && rev_count >= 1;

    let (next_tool, effective_verdict) = if verdict == "APPROVED" || auto_approved {
        (
            "sdd_workflow_create_change_merge",
            if auto_approved {
                "AUTO_APPROVED"
            } else {
                "APPROVED"
            },
        )
    } else {
        ("sdd_workflow_revise_change_docs", verdict.as_str())
    };

    let na = workflow_common::next_action(interface, next_tool, json!({"change_id": change_id}));

    let result = json!({
        "status": "ok",
        "verdict": effective_verdict,
        "review_path": format!(".aw/changes/{}/docs_review/review.md", change_id),
        "cli_verification_results": cli_results,
        "auto_approved": auto_approved,
        "revision_count": rev_count,
        "next_actions": [na]
    });
    Ok(serde_json::to_string_pretty(&result)?)
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
      - "<handwrite-gap:missing-generator:sdd-review-docs-artifact-flow>"
    description: "Review-change-docs artifact verdict persistence and routing."
```
