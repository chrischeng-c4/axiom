---
id: k8s-job-executor
type: proposal
version: 1
created_at: 2026-01-31T11:14:23.471066+00:00
updated_at: 2026-01-31T11:14:23.471066+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add K8s Job Executor for heavy/GPU/long-running tasks with non-blocking worker hand-off."
history:
  - timestamp: 2026-01-31T11:14:23.471066+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T11:18:34.357618+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T11:18:52.891975+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
  - timestamp: 2026-01-31T11:22:00.196078+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T11:22:13.717030+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 7
  new_files: 1
affected_specs:
  - id: workflow-state-machine
    path: specs/workflow-state-machine.md
    depends: []
  - id: workflows
    path: specs/workflows.md
    depends: []---

<proposal>

# Change: k8s-job-executor

## Summary

Add K8s Job Executor for heavy/GPU/long-running tasks with non-blocking worker hand-off.

## Why

Distributed task execution needs to handle heterogeneous workloads efficiently. Heavy tasks (ML training, data processing) require specific hardware (GPUs/TPUs) and are better isolated in dedicated K8s Jobs. Non-blocking spawning allows workers to manage large-scale resource-intensive tasks without starvation.

## What Changes

- Add 'k8s' feature and K8sJobExecutor to cclab-meteor.
- Update Task trait and TaskMessage to support executor markers and K8s configuration.
- Extend ResultBackend with metadata support (set_metadata/get_metadata) for workflow tracking.
- Implement chain continuation logic that can be triggered by external K8s jobs.
- Add 'run-once' command to CLI for K8s Job container execution.

## Impact

- **Scope**: minor
- **Affected Files**: ~7
- **New Files**: ~1
- Affected specs:
  - `workflow-state-machine` (no dependencies)
  - `workflows` (no dependencies)
- Affected code: `crates/cclab-meteor/src/task.rs`, `crates/cclab-meteor/src/message.rs`, `crates/cclab-meteor/src/worker/mod.rs`, `crates/cclab-meteor/src/backend/mod.rs`, `crates/cclab-meteor/src/backend/ion.rs`, `crates/cclab-meteor/src/backend/redis.rs`, `crates/cclab-meteor/src/workflow/chain.rs`

</proposal>
