---
id: 197
type: proposal
version: 1
created_at: 2026-02-12T08:17:55.746413+00:00
updated_at: 2026-02-12T08:17:55.746413+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add error recovery documentation to delegate-agent.md and run-change/README.md"
history:
  - timestamp: 2026-02-12T08:17:55.746413+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: patch
  affected_files: 2
  new_files: 0
affected_specs:
  - id: delegate-agent
    path: specs/delegate-agent.md
    depends: []
  - id: run-change
    path: specs/run-change.md
    depends: []
---

<proposal>

# Change: 197

## Summary

Add error recovery documentation to delegate-agent.md and run-change/README.md

## Why

Genesis specs assume happy path only. Missing documentation for: agent tool call failure recovery, genesis_agent verification failed next steps, cyclic dependency fallback, partial state recovery, and concurrent STATE.yaml access. This causes confusion when agents fail mid-task and mainthread doesn't know how to recover.

## What Changes

- Add Error Recovery section to delegate-agent.md covering: retry policy for transient failures, agent fallback chain, verification failure escalation path
- Add Error Recovery section to run-change/README.md covering: partial state recovery, cyclic dependency fallback, user intervention hooks, concurrent STATE.yaml access

## Impact

- **Scope**: patch
- **Affected Files**: ~2
- **New Files**: ~0
- Affected specs:
  - `delegate-agent` (no dependencies)
  - `run-change` (no dependencies)

</proposal>
