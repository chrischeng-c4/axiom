---
id: sdd-tools-create-pre-clarifications-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create pre clarifications artifact

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_pre_clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 287 | artifact_definition() -> ToolDefinition |
| `definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 17 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 70 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_append` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 199 | execute_append(args: &Value, project_root: &Path) -> Result<String> |
| `execute_artifact_pre_clarifications` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 418 | execute_artifact_pre_clarifications(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow_pre_clarifications` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 343 | execute_workflow_pre_clarifications(     args: &Value,     project_root: &Path, ) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 262 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute sdd_artifact_create_pre_clarifications.
///
/// Reads existing `groups/{group_id}/pre_clarifications.md` (with question stubs),
/// merges answers, sets status to answered, updates `groups_progress`.
pub fn execute_artifact_pre_clarifications(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;

    // Accept both "answers" (new) and "questions" (legacy) field names
    let answers = args
        .get("answers")
        .or_else(|| args.get("questions"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing required array field: answers"))?;

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);

    if !change_dir.exists() {
        anyhow::bail!("Change directory not found: {}", change_dir.display());
    }

    let clarif_path = change_dir.join("pre_clarifications.md");
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut content = format!(
        "---\nchange: {}\ndate: {}\nstatus: answered\n---\n\n# Pre-Clarifications\n\n",
        change_id, today
    );

    for (i, qa) in answers.iter().enumerate() {
        let topic = qa
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("General");
        let question = qa.get("question").and_then(|v| v.as_str()).unwrap_or("");
        let answer = qa.get("answer").and_then(|v| v.as_str()).unwrap_or("");
        let rationale = qa.get("rationale").and_then(|v| v.as_str()).unwrap_or("");
        let follow_ups: Vec<String> = qa
            .get("follow_up_questions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        content.push_str(&format!("### Q{}: {}\n", i + 1, topic));
        if !question.is_empty() {
            content.push_str(&format!("- **Question**: {}\n", question));
        }
        content.push_str(&format!("- **Answer**: {}\n", answer));
        if !rationale.is_empty() {
            content.push_str(&format!("- **Rationale**: {}\n", rationale));
        }
        for fq in &follow_ups {
            content.push_str(&format!("- **Follow-up**: {}\n", fq));
        }
        content.push('\n');
    }

    std::fs::write(&clarif_path, &content)?;

    // Advance phase to PostClarificationsCreated (pre-clarifications + reference context
    // absorbed by issue lifecycle)
    let mut sm = StateManager::load(&change_dir)?;
    sm.set_phase(StatePhase::ChangeInited)?;
    sm.save()?;

    let artifacts_written = vec![
        "pre_clarifications.md".to_string(),
        "STATE.yaml".to_string(),
    ];

    let next_actions = json!([workflow_common::next_action(
        interface,
        "sdd_run_change",
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
  - path: projects/agentic-workflow/src/tools/create_pre_clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_artifact_pre_clarifications"
    description: "Group-aware pre-clarifications artifact writer and phase response."
```
