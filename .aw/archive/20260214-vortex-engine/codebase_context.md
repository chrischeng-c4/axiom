---
change_id: vortex-engine
type: codebase_context
created_at: 2026-02-14T06:23:52.445389+00:00
updated_at: 2026-02-14T06:23:52.445389+00:00
iteration: 2
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - grep_search
  - list_directory
---

# Codebase Context

## Analyzed Files

- **crates/cclab-server/src/mcp/router.rs** — Unified MCP router that routes tool calls to specific registries (Genesis, Prism, Aurora).
  - symbols: `UnifiedMcpRouter`, `list_tools`, `call_tool`
- **crates/cclab-aurora/src/mcp/tools.rs** — Implementation of MCP tool schemas and registry for diagram/spec generation.
  - symbols: `AuroraTools`, `ToolSchema`
- **crates/cclab-aurora/src/mcp/handlers.rs** — Dispatcher for Aurora MCP tool calls.
  - symbols: `call_tool`
- **crates/cclab-nova/src/agents/mod.rs** — Definition of the core Agent trait for autonomous assistants.
  - symbols: `Agent`
- **crates/cclab-orbit/src/loop_impl.rs** — Implementation of a high-performance event loop for Python asyncio (Rust-backed).
  - symbols: `PyLoop`
- **crates/cclab-vortex/** — Directory structure for the planned vortex engine. Currently contains subdirectories (agent, core, ecs, mcp, render, td) but no Cargo.toml or source files.

## Prism Results

- **grep_search** (query: `mcp tool registration patterns`)
  - Tool registries are instantiated in UnifiedMcpRouter::new and called in list_tools and call_tool methods.
- **prism_symbols** (query: `Agent trait definition`)
  - The Agent trait provides async methods for running tasks with or without stream handlers.

## Dependency Graph

- cclab-server -> cclab-aurora (Tool integration pattern)
- cclab-server -> cclab-genesis (Tool integration pattern)
- cclab-server -> cclab-prism (Tool integration pattern)
