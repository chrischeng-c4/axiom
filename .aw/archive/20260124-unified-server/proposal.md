---
id: unified-server
type: proposal
version: 1
created_at: 2026-01-24T08:52:22.292168+00:00
updated_at: 2026-01-24T08:52:22.292168+00:00
author: mcp
status: proposed
iteration: 1
summary: "Merge Argus daemon into cclab-server for unified MCP + LSP functionality"
history:
  - timestamp: 2026-01-24T08:52:22.292168+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-24T08:53:33.146274+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_proposal"
    action: "created"
    duration_secs: 213.39
  - timestamp: 2026-01-24T08:54:03.035591+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 29.89
  - timestamp: 2026-01-24T08:56:45.822552+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 162.78
  - timestamp: 2026-01-24T08:57:42.539724+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 56.72
impact:
  scope: major
  affected_files: 15
  new_files: 2
affected_specs:
  - id: unified-server-architecture
    path: specs/unified-server-architecture.md
    depends: []---

<proposal>

# Change: unified-server

## Summary

Merge Argus daemon into cclab-server for unified MCP + LSP functionality

## Why

Currently, the Argus engine (Prism) requires a separate daemon process for each project, which is complex to manage and resource-intensive. cclab-server already provides a unified HTTP server for dashboard and MCP. Merging the analysis engine into cclab-server simplifies the architecture, provides a single entry point for all tools, and allows sharing analysis state between MCP and LSP.

## What Changes

- Refactor cclab-prism::RequestHandler to be the core analysis engine with in-memory document support.
- Update cclab-server::PrismHandlerPool to host RequestHandler instances directly.
- Integrate Argus LSP server into cclab-server, listening on a configurable TCP port.
- Add UnifiedLspRouter to cclab-server for multi-project LSP support.
- Update MCP handlers in cclab-server to call the local analysis engine.
- Update CLI to manage the unified server and deprecate standalone Argus daemon commands.

## Impact

- **Scope**: major
- **Affected Files**: ~15
- **New Files**: ~2
- Affected specs:
  - `unified-server-architecture` (no dependencies)
- Affected code: `crates/cclab-server/src/lib.rs`, `crates/cclab-server/src/http_server.rs`, `crates/cclab-server/src/prism_pool.rs`, `crates/cclab-server/src/mcp/router.rs`, `crates/cclab-prism/src/server/handler.rs`, `crates/cclab-prism/src/lsp/server.rs`, `crates/cclab-cli/src/main.rs`
- **Breaking Changes**: ArgusDaemon Unix socket communication is replaced by direct in-process calls within cclab-server. External tools connecting to ArgusDaemon sockets will need to transition to the unified server's HTTP/MCP or TCP/LSP endpoints.

</proposal>

<review iteration="2" reviewer="gemini-cli-agent" status="approved">
## Summary
The proposal to merge the Argus daemon into `cclab-server` is well-grounded and simplifies the architecture by moving from a multi-process daemon model to a unified server model. The plan accurately identifies the necessary refactoring in `cclab-prism` and integration points in `cclab-server`.

## Issues
No major issues found. The proposed TCP/LSP transport is standard and the task breakdown is logical.

## Verdict
The proposal is approved. It meets all quality criteria and provides a clear implementation path.

## Next Steps
Proceed to the implementation phase as outlined in the tasks.md file.
</review>

<review iteration="1" reviewer="codex" status="needs_revision">
## Summary
Tasks are broadly aligned with the proposal/spec, but there are a few gaps and mismatches that could lead to missed implementation work.

## Issues
1. CLI update target appears inconsistent with the proposal. The proposal calls out `crates/cclab-cli/src/main.rs`, but Task 3.6 modifies `crates/cclab-server/src/cli.rs`. If the CLI lives in `cclab-cli`, the current task list will miss required changes.
2. LSP port configuration is assigned to `crates/cclab-prism/src/core/config.rs` (Task 1.1), but R5 is a server-level requirement. If the server owns the listener, config likely belongs in `cclab-server` (and be wired into `http_server`). Otherwise the task list risks scattering config across crates without a clear source of truth.
3. Acceptance criteria include multi-project routing and diagnostics publication, but Task 4.1 only mentions “MCP tools and LSP requests.” The test task should explicitly cover multi-project routing and diagnostics to satisfy the spec scenarios.

## Verdict
Needs revision.

## Next Steps
Revise tasks to fix the CLI file target, clarify the LSP port configuration ownership/wiring, and expand the integration test task to cover multi-project routing and diagnostics.
</review>
