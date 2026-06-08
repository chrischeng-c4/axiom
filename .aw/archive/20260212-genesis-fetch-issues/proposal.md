---
id: genesis-fetch-issues
type: proposal
version: 1
created_at: 2026-02-12T02:38:16.836854+00:00
updated_at: 2026-02-12T02:38:16.836854+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement GitHub issue fetching via CLI, DAG-based topological workflow looping, and standardized response action fields."
history:
  - timestamp: 2026-02-12T02:38:16.836854+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 12
  new_files: 3
affected_specs:
  - id: fetch-issues
    path: specs/fetch-issues.md
    depends: []
  - id: run-change-dag-loop
    path: specs/run-change-dag-loop.md
    depends: [fetch-issues]
---

<proposal>

# Change: genesis-fetch-issues

## Summary

Implement GitHub issue fetching via CLI, DAG-based topological workflow looping, and standardized response action fields.

## Why

The current workflow lacks automated issue resolution and dependency tracking, requiring manual context gathering for complex changes. By fetching issues and dependencies directly from the platform, we can build a topological graph of requirements and clarifications. This change enables the per-issue clarification loop, ensuring that complex changes are decomposed and clarified systematically. It also standardizes the action response format by migrating legacy fields to the new next field architecture. Finally, it aligns the platform sync service with the documented HTTP MCP transport and project isolation patterns, improving robustness and multi-project support.

## What Changes

- Implement genesis_fetch_issues MCP tool using gh CLI for content and dependency extraction.
- Extend STATE.yaml schema with a dag section to track topological order and loop indices.
- Update run_change logic to handle topological iteration for clarify and context phases.
- Migrate action responses from mainthread_instruction to the standardized next field.
- Integrate PlatformSyncService with HTTP MCP headers and the global project registry.

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~3
- Affected specs:
  - `fetch-issues` (no dependencies)
  - `run-change-dag-loop` → depends on: `fetch-issues`
- Affected code: `crates/cclab-genesis/src/mcp/tools/run_change/`, `crates/cclab-genesis/src/services/platform_sync/`, `crates/cclab-genesis/src/models/change.rs`

</proposal>
