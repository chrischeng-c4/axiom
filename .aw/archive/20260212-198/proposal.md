---
id: 198
type: proposal
version: 1
created_at: 2026-02-12T08:21:35.334129+00:00
updated_at: 2026-02-12T08:21:35.334129+00:00
author: mcp
status: proposed
iteration: 1
summary: "Sync run-change/README.md action enum with sub-spec actions"
history:
  - timestamp: 2026-02-12T08:21:35.334129+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: patch
  affected_files: 1
  new_files: 0
affected_specs:
  - id: run-change
    path: specs/run-change.md
    depends: []
---

<proposal>

# Change: 198

## Summary

Sync run-change/README.md action enum with sub-spec actions

## Why

run-change/README.md OpenRPC action enum is missing 5 actions defined in implement-change.md and merge-change.md sub-specs (review_task, revise_task, task_terminal_failure, all_tasks_done, merge_complete), and contains orphan 'complete' action not defined by any sub-spec. This causes confusion about which actions are valid.

## What Changes

- Add missing actions to OpenRPC enum: review_task, revise_task, task_terminal_failure, all_tasks_done, merge_complete
- Remove orphan 'complete' action from enum
- Verify Prompt Sources table covers all new actions

## Impact

- **Scope**: patch
- **Affected Files**: ~1
- **New Files**: ~0
- Affected specs:
  - `run-change` (no dependencies)

</proposal>
