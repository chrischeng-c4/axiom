---
id: toolchain-reorg
type: proposal
version: 1
created_at: 2026-01-28T08:21:36.321692+00:00
updated_at: 2026-01-28T08:21:36.321692+00:00
author: mcp
status: proposed
iteration: 1
summary: "Reorganize Prism tool routing and add State Machine MCP tools"
history:
  - timestamp: 2026-01-28T08:21:36.321692+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-28T08:24:09.912699+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 248.29
  - timestamp: 2026-01-28T08:24:32.939401+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 23.02
impact:
  scope: minor
  affected_files: 5
  new_files: 0
affected_specs:
  - id: prism-mcp-refactor
    path: specs/prism-mcp-refactor.md
    depends: []
  - id: state-machine-tools
    path: specs/state-machine-tools.md
    depends: []---

<proposal>

# Change: toolchain-reorg

## Summary

Reorganize Prism tool routing and add State Machine MCP tools

## Why

To improve security by removing sensitive environment configuration tools from LLM access, and to optimize performance by handling pure functional tools (spec generation, state machines) locally in the server instead of routing them to the per-project daemon. This also completes the toolchain by exposing state machine capabilities via MCP.

## What Changes

- Split Prism tool routing into Daemon path (analysis) and MCP Handler path (local tools) in cclab-server router.
- Remove sensitive Python environment tools from MCP tool list.
- Add state machine generation and validation tools to Prism.
- Update RequestHandler and UnifiedMcpRouter to correctly handle all Prism tools.

## Impact

- **Scope**: minor
- **Affected Files**: ~5
- **New Files**: ~0
- Affected specs:
  - `prism-mcp-refactor` (no dependencies)
  - `state-machine-tools` (no dependencies)
- Affected code: `crates/cclab-server/src/mcp/router.rs`, `crates/cclab-prism/src/mcp/tools.rs`, `crates/cclab-prism/src/mcp/mod.rs`, `crates/cclab-prism/src/mcp/spec_handler.rs`, `crates/cclab-prism/src/server/handler.rs`

</proposal>
