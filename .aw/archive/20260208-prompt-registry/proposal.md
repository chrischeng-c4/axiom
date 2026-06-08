---
id: prompt-registry
type: proposal
version: 1
created_at: 2026-02-08T15:38:20.701114+00:00
updated_at: 2026-02-08T15:38:20.701114+00:00
author: mcp
status: proposed
iteration: 1
summary: "Populate agent_prompt in run_change by inlining prompts and restructuring into run_change/ folder"
history:
  - timestamp: 2026-02-08T15:38:20.701114+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 12
  new_files: 4
affected_specs:
  - id: prompt-registry
    path: specs/prompt-registry.md
    depends: []
---

<proposal>

# Change: prompt-registry

## Summary

Populate agent_prompt in run_change by inlining prompts and restructuring into run_change/ folder

## Why

Currently genesis_run_change returns agent_prompt: null for all agent-delegated actions (explore_spec, create_proposal, review_proposal, etc.). This means the mainthread cannot delegate work to LLM agents via genesis_llm because there's no prompt to pass. The prompt templates already exist in src/prompts/*.md but are only used by the deprecated task.rs system. This change inlines those prompts into the stage modules and actually populates agent_prompt in the run_change response, completing the agent delegation architecture.

## What Changes

- Delete src/prompts/ directory — remove 13 files (mod.rs + 12 markdown templates)
- Convert run_change.rs to run_change/mod.rs folder structure with per-stage files
- Inline prompt templates as Rust const strings in each stage module
- Populate agent_prompt field in build_response for all agent-delegated actions
- Migrate task.rs and llm.rs prompt dependencies to new locations

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~4
- Affected specs:
  - `prompt-registry` (no dependencies)
- Affected code: `crates/cclab-genesis/src/prompts/ (delete entire directory)`, `crates/cclab-genesis/src/mcp/tools/run_change.rs -> run_change/mod.rs`, `crates/cclab-genesis/src/mcp/tools/run_change/decide.rs (new)`, `crates/cclab-genesis/src/mcp/tools/run_change/plan.rs (new)`, `crates/cclab-genesis/src/mcp/tools/run_change/implement.rs (new)`, `crates/cclab-genesis/src/mcp/tools/run_change/merge.rs (new)`, `crates/cclab-genesis/src/mcp/tools/decide_change.rs (delete)`, `crates/cclab-genesis/src/mcp/tools/plan_change.rs (delete)`, `crates/cclab-genesis/src/mcp/tools/impl_change.rs (delete)`, `crates/cclab-genesis/src/mcp/tools/merge_change.rs (delete)`, `crates/cclab-genesis/src/mcp/tools/task.rs (update imports)`, `crates/cclab-genesis/src/mcp/tools/llm.rs (update imports)`

</proposal>
