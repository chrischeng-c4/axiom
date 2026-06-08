---
id: sdd-tools-create-pre-clarifications-workflow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create pre clarifications workflow

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
/// Execute sdd_workflow_create_pre_clarifications.
///
/// Determines the next group to clarify, returns prompt to mainthread.
pub async fn execute_workflow_pre_clarifications(
    args: &Value,
    project_root: &Path,
) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    // If already past this phase, advance and return
    let sm = StateManager::load(&change_dir)?;
    if *sm.phase() != StatePhase::ChangeInited {
        let result = json!({
            "status": "phase_complete",
            "prompt": "Pre-clarifications already created. Advancing.",
            "next_actions": [
                workflow_common::next_action(interface, "sdd_run_change", json!({"change_id": change_id}))
            ]
        });
        return Ok(serde_json::to_string_pretty(&result)?);
    }
    drop(sm);

    let project_path = project_root.display();

    let prompt = format!(
        r#"# Task: Create Pre-Clarifications for Change '{change_id}'

## Files to Read

- `{project_path}/.aw/changes/{change_id}/user_input.md` — user's description

## Instructions

1. Read user_input.md
2. Identify key decisions and open questions
3. Use AskUserQuestion to ask the user for clarifications
4. When sufficient, run `score artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
score artifact create-pre-clarifications {change_id} .aw/changes/{change_id}/payloads/create-pre-clarifications.json
```"#,
    );

    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreatePreClarifications,
    );

    workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "create_pre_clarifications",
        prompt,
        executor,
        json!({}),
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
  - path: projects/agentic-workflow/src/tools/create_pre_clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_workflow_pre_clarifications"
    description: "Group-aware pre-clarifications workflow prompt builder and dispatch response."
```
