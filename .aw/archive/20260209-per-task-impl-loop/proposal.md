---
id: per-task-impl-loop
type: proposal
version: 1
created_at: 2026-02-09T09:56:31.729728+00:00
updated_at: 2026-02-09T09:56:31.729728+00:00
author: mcp
status: proposed
iteration: 1
summary: "Refactor implementation workflow to execute, review, and revise tasks individually in dependency order."
history:
  - timestamp: 2026-02-09T09:56:31.729728+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: major
  affected_files: 12
  new_files: 0
affected_specs:
  - id: impl-workflow-refactor
    path: specs/impl-workflow-refactor.md
    depends: [run-change, impl-change/workflow]
  - id: task-review-protocol
    path: specs/task-review-protocol.md
    depends: [state-management]
---

<proposal>

# Change: per-task-impl-loop

## Summary

Refactor implementation workflow to execute, review, and revise tasks individually in dependency order.

## Why

The current monolithic implementation phase treats all changes as a single unit, which risks infinite review loops and makes it difficult to resume work or isolate failures. By breaking execution into per-task loops, we ensure incremental stability, enable precise state recovery, and enforce granular quality control with specific revision limits for each task.

## What Changes

- Update STATE.yaml schema to track `current_task_id` and per-task revision counts.
- Implement `State` migration: Default new fields to None/Empty for legacy files; always write new schema version.
- Refactor `implement.rs` to iterate tasks using TaskGraph topological sort with **lexical task_id tie-breaking** for deterministic order.
- Implement per-task review/revise loop with strict **2-revision limit**.
- Define terminal behavior: Upon reaching revision limit, fail the task, halt execution, output failure report, and escalate to operator for manual intervention.
- Update `genesis_review_implementation` to support task-scoped review artifacts.
- Safeguards: Enforce clean working directory check for `in_place` workflow to prevent data loss.

## Impact

- **Scope**: major
- **Affected Files**: ~12
- **New Files**: ~0
- Affected specs:
  - `impl-workflow-refactor` → depends on: `run-change`, `impl-change/workflow`
  - `task-review-protocol` → depends on: `state-management`
- Affected code: `crates/cclab-genesis/src/mcp/tools/run_change/implement.rs`, `crates/cclab-genesis/src/state/manager.rs`, `crates/cclab-genesis/src/models/frontmatter.rs`, `crates/cclab-genesis/src/mcp/tools/implementation.rs`, `crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs`, `crates/cclab-genesis/src/cli/implement.rs`, `crates/cclab-genesis/src/orchestrator/prompts.rs`
- **Breaking Changes**: STATE.yaml schema extension (adds `current_task_id`, `task_revisions`). Legacy files load with defaults; saves migrate to new version.

</proposal>
