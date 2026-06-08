---
id: sdd-tools-workflow-common-prompt-response
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools workflow common prompt response

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/workflow_common.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `build_group_issues_hint` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 442 | build_group_issues_hint(change_dir: &Path, group_id: &str) -> String |
| `build_workflow_response` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 395 | build_workflow_response(     change_dir: &Path,     change_id: &str,     action: &str,     prompt: String,     executor: Vec<String>,     extra_fields: Value,     _interface: SddInterface,     _project_root: &Path, ) -> Result<String> |
| `get_executor_chain` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 273 | get_executor_chain(_project_root: &Path, artifact: WorkflowArtifact) -> Vec<String> |
| `has_uncommitted_diff` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 209 | has_uncommitted_diff(project_root: &Path, rel_path: &str) -> Result<bool> |
| `is_git_project` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 174 | is_git_project(project_root: &Path) -> bool |
| `is_git_tracked` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 192 | is_git_tracked(project_root: &Path, rel_path: &str) -> Result<bool> |
| `list_group_ids` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 325 | list_group_ids(groups_dir: &Path) -> Result<Vec<String>> |
| `load_interface` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 309 | load_interface(project_root: &Path) -> SddInterface |
| `next_action` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 319 | next_action(interface: SddInterface, tool: &str, args: Value) -> Value |
| `phase_to_string` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 144 | phase_to_string(phase: &StatePhase) -> &'static str |
| `resolve_active_change_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 76 | resolve_active_change_id(project_root: &Path) -> Result<String> |
| `resolve_change_dir` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 120 | resolve_change_dir(project_root: &Path, change_id: &str) -> PathBuf |
| `resolve_single_group_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 346 | resolve_single_group_id(change_dir: &Path) -> Option<String> |
| `update_phase` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 250 | update_phase(change_dir: &Path, phase: StatePhase) -> Result<()> |
| `validate_change_dir` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 39 | validate_change_dir(change_dir: &Path, project_root: &Path) -> Result<()> |
| `validate_change_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 18 | validate_change_id(change_id: &str) -> Result<()> |
| `write_prompt_file` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 364 | write_prompt_file(     change_dir: &Path,     group_id: Option<&str>,     action: &str,     prompt: &str, ) -> Result<PathBuf> |
## Source
<!-- type: source lang: rust -->

````rust
/// Write a prompt to a file in the change's `prompts/` directory.
///
/// When `group_id` is `Some`, writes to `groups/{gid}/prompts/` instead of
/// the change-level `prompts/` directory.
///
/// Returns the path to the written file.
pub fn write_prompt_file(
    change_dir: &Path,
    group_id: Option<&str>,
    action: &str,
    prompt: &str,
) -> Result<PathBuf> {
    let prompts_dir = match group_id {
        Some(gid) => change_dir.join("groups").join(gid).join("prompts"),
        None => change_dir.join("prompts"),
    };
    std::fs::create_dir_all(&prompts_dir)?;
    let path = prompts_dir.join(format!("{}.md", action));
    std::fs::write(&path, prompt)?;
    Ok(path)
}

/// Build a workflow response. Always writes prompt to file for clean tool responses.
///
/// Score is **not a runner** — it's a state machine that writes prompts and
/// returns metadata. The host agent (Claude Code mainthread or subagent)
/// handles actual LLM execution:
///
/// - `mainthread` executor → host agent reads the prompt file and executes inline
/// - `subagent:{type}` → host agent invokes Agent tool (model from agent definition)
///
/// No subprocess dispatch, no runtime agent selection. The executor was
/// decided by `get_executor_chain()` per-artifact (see that function for the
/// hardcoded hybrid mapping).
///
/// `extra_fields` are merged into the top-level response (e.g., `spec_id`, `group_id`).
pub async fn build_workflow_response(
    change_dir: &Path,
    change_id: &str,
    action: &str,
    prompt: String,
    executor: Vec<String>,
    extra_fields: Value,
    _interface: SddInterface,
    _project_root: &Path,
) -> Result<String> {
    // Extract group_id from extra_fields for group-scoped prompt path
    let group_id = extra_fields.get("group_id").and_then(|v| v.as_str());

    // Always write prompt to file
    write_prompt_file(change_dir, group_id, action, &prompt)?;
    let rel_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/prompts/{}.md",
            change_id, gid, action
        ),
        None => format!(".aw/changes/{}/prompts/{}.md", change_id, action),
    };

    // All executors (mainthread + subagent:*) return prompt_path + executor.
    // The host agent decides how to run it. This is the only dispatch path.
    let mut result = json!({
        "status": "ok",
        "prompt_path": rel_path,
        "executor": executor,
        "next_actions": []
    });

    // Merge extra fields into top-level
    if let Some(obj) = extra_fields.as_object() {
        for (k, v) in obj {
            result[k] = v.clone();
        }
    }

    Ok(serde_json::to_string_pretty(&result)?)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/workflow_common.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "write_prompt_file"
      - "build_workflow_response"
    description: "Prompt-file writing and workflow response construction."
```
