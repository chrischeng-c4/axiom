---
id: sdd-tools-workflow-common-phase-and-executor
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools workflow common phase and executor

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
/// Update STATE.yaml phase after successful agent execution.
///
/// StateManager::save() now dual-writes all workflow fields (phase, branch,
/// iteration, task tracking) to issue frontmatter automatically.
// REQ: R1 — Issue frontmatter absorbs STATE.yaml
// REQ: R6 — StateManager reads/writes issue frontmatter
pub fn update_phase(change_dir: &Path, phase: StatePhase) -> Result<()> {
    let mut state_manager = StateManager::load(change_dir)?;
    state_manager.set_phase(phase)?;
    state_manager.save()?;
    Ok(())
}

/// Get the executor chain for a WorkflowArtifact.
///
/// Score uses a **fixed hybrid mapping**: heavy tasks (reference context, spec,
/// implementation, docs) run as Claude Code subagents — context-isolated,
/// parallelizable. Lightweight tasks (restructure, clarifications, merge) run
/// in the mainthread session because subagent startup overhead isn't worth it.
///
/// The executor string is `subagent:<agent_type>` — model is NOT specified here.
/// Model selection lives in the agent definition (`.claude/agents/<type>.md`
/// frontmatter `model:` field). This separation means model changes don't
/// require code changes.
///
/// This is **not configurable**. There is no `workflow.mode` knob, no alternate
/// preset. The host agent (Claude Code) is the only supported runner — score
/// does not spawn subprocesses (no `claude-agent:*`, `gemini:*`, `codex:*`).
pub fn get_executor_chain(_project_root: &Path, artifact: WorkflowArtifact) -> Vec<String> {
    let executor = match artifact {
        // Lightweight phases — stay in mainthread (subagent overhead > benefit)
        WorkflowArtifact::RestructureInput
        | WorkflowArtifact::CreatePreClarifications
        | WorkflowArtifact::CreatePostClarifications
        | WorkflowArtifact::ReviseReferenceContext
        | WorkflowArtifact::CreateChangeMerge => "mainthread",

        // Reference context building — Explore agent (purpose-built for code reading)
        WorkflowArtifact::CreateReferenceContext => "subagent:Explore",

        // Review phases — quality gate
        WorkflowArtifact::ReviewReferenceContext
        | WorkflowArtifact::ReviewChangeSpec
        | WorkflowArtifact::ReviewChangeImplementation
        | WorkflowArtifact::ReviewChangeDocs => "subagent:score-review",

        // Spec creation / revision
        WorkflowArtifact::CreateChangeSpec
        | WorkflowArtifact::ReviseChangeSpec
        | WorkflowArtifact::CreateChangeDocs
        | WorkflowArtifact::ReviseChangeDocs => "subagent:score-change-spec",

        // Implementation creation / revision
        WorkflowArtifact::CreateChangeImplementation
        | WorkflowArtifact::ReviseChangeImplementation => "subagent:score-change-implementation",
    };
    vec![executor.to_string()]
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
      - "update_phase"
      - "get_executor_chain"
    description: "Workflow phase update and fixed executor-chain mapping."
```
