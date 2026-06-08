---
id: sdd-tools-create-pre-clarifications-append
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create pre clarifications append

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
/// Append per-issue clarifications to pre_clarifications.md (DAG mode).
///
/// Called internally by `artifact_write` when `issue` param is present.
pub fn execute_append(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let questions = get_required_array(args, "questions")?;
    let issue_num = args.get("issue").and_then(|v| v.as_u64());

    // Convert JSON array to QuestionAnswer structs
    let qa_list: Vec<QuestionAnswer> = questions
        .iter()
        .map(|qa| QuestionAnswer {
            topic: qa
                .get("topic")
                .and_then(|v| v.as_str())
                .unwrap_or("General")
                .to_string(),
            question: qa
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            answer: qa
                .get("answer")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            rationale: qa
                .get("rationale")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect();

    let qa_count = qa_list.len();

    let input = AppendClarificationsInput {
        change_id: change_id.clone(),
        issue: issue_num,
        questions: qa_list,
    };

    let _service_result = service_append(input, project_root)?;

    let result = json!({
        "status": "ok",
        "change_id": change_id,
        "artifacts": [
            format!(".aw/changes/{}/pre_clarifications.md", change_id),
            format!(".aw/changes/{}/STATE.yaml", change_id),
        ],
        "questions_count": qa_count,
        "next": "sdd_run_change"
    });
    Ok(result.to_string())
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
      - "execute_append"
    description: "Per-issue DAG pre-clarification append entrypoint."
```
