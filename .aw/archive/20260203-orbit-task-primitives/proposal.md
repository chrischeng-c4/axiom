---
id: orbit-task-primitives
type: proposal
version: 1
created_at: 2026-01-31T15:30:50.159004+00:00
updated_at: 2026-01-31T15:30:50.159004+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add cancellation depth, task registry, and completion tracking primitives for PyLoop tasks"
history:
  - timestamp: 2026-01-31T15:30:50.159004+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T15:30:56.308828+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T15:31:10.221464+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 6
  new_files: 2---

<proposal>

# Change: orbit-task-primitives

## Summary

Add cancellation depth, task registry, and completion tracking primitives for PyLoop tasks

## Why

PyLoop currently models cancellation as a boolean and lacks a centralized registry or completion notifications. This blocks `uncancel()` semantics, weak-ref task lookup, and efficient waiting for task completion (issues #55–#57). Adding primitives aligns behavior with Python 3.11 asyncio semantics and enables higher-level scheduling APIs to build on reliable cancellation and completion tracking.

## What Changes

- Introduce a cancellation depth counter on `Task` with `cancel()`, `uncancel()`, and `cancelled()` derived from depth > 0.
- Add a `TaskRegistry` in `PyLoop` that stores weak references and stable `TaskId` values for lookup and cleanup without leaks.
- Emit task completion events (task id + terminal state) from executors over an mpsc channel.
- Add `CompletionQueue` and `WaitSet` abstractions to collect and await task completions (static set, matching asyncio.wait semantics).
- Wire task creation and completion to register tasks, notify completion, and prune registry entries on finish.

## Impact

- **Scope**: minor
- **Affected Files**: ~6
- **New Files**: ~2
- Affected code: `crates/cclab-orbit/src/task.rs`, `crates/cclab-orbit/src/loop_impl.rs`, `crates/cclab-orbit/src/lib.rs`, `crates/cclab-orbit/src/error.rs`, `crates/cclab-orbit/src/registry.rs`, `crates/cclab-orbit/src/completion.rs`
- **Breaking Changes**: Behavioral change: `Task.cancelled()` becomes true only when cancellation depth > 0; add `Task.uncancel()` and update any depth-aware callers accordingly.

</proposal>
