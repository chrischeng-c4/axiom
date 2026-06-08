---
change_id: sdd-merge
type: codebase_context
created_at: 2026-02-15T03:35:00.842557+00:00
updated_at: 2026-02-15T03:35:00.842557+00:00
iteration: 2
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_references
  - prism_impact
---

# Codebase Context

## Analyzed Files

- **crates/cclab-server/src/mcp/router.rs** — Unified MCP tool router, combines tools from multiple crates.
  - symbols: `UnifiedMcpRouter`, `call_prism_tool`, `call_genesis_tool`, `call_aurora_tool`, `call_tool_streaming`, `list_tools`
- **crates/cclab-server/src/registry.rs** — Project and server registry for unified server.
  - symbols: `Registry`, `ProjectInfo`, `ServerInfo`, `load`, `save`, `register_project`, `auto_register`
- **crates/cclab-genesis/src/mcp/registry.rs** — Redundant registry implementation in genesis crate.
  - symbols: `Registry`, `ProjectInfo`, `ServerInfo`, `load`, `save`, `register_project`
- **crates/cclab-cli/src/main.rs** — Main CLI entry point, delegates to sub-crates.
  - symbols: `Commands`, `main`, `run_mcp_server`, `run_prism_check`
- **crates/cclab-prism/src/mcp/tools.rs** — Defines Prism MCP tools, including PDG tools.
  - symbols: `ArgusTools`, `prism_pdg`, `prism_impact`, `prism_check`, `prism_symbols`
- **crates/cclab-aurora/src/lib.rs** — Diagram and spec generation library and MCP tools.
  - symbols: `AuroraError`, `Result`
- **crates/cclab-server/src/cli.rs** — Server-specific CLI commands, consumer of Registry::load.
  - symbols: `ServerCommands`, `run`, `start_server`, `ensure_server_running`
- **crates/cclab-server/src/http_server.rs** — HTTP server implementation for cclab-server.
  - symbols: `UnifiedAppState`, `start_server`, `handle_mcp_request`
- **crates/cclab-genesis/src/cli/server.rs** — Genesis-specific server CLI commands, consumer of Registry::load.
  - symbols: `ServerCommands`, `run`, `start_server`, `shutdown_server`
- **crates/cclab-genesis/src/mcp/http_server.rs** — HTTP server implementation for cclab-genesis.
  - symbols: `AppState`, `start_server`, `handle_mcp_request`

## Prism Results

- **prism_symbols** (query: `prism_symbols on analyzed files`)
  - Successfully extracted symbols for 10 key files, identifying core structures and Registry::load consumers.
- **prism_references** (query: `Registry::load references`)
  - Attempted cross-crate reference search; returned empty result. Dependency on Registry::load confirmed via grep_search instead.
- **prism_impact** (query: `prism_impact on PDG tools`)
  - Identified that PDG tools are missing from the unified MCP router in cclab-server/src/mcp/router.rs.

## Dependency Graph

- cclab-cli -> cclab-server
- cclab-cli -> cclab-genesis
- cclab-cli -> cclab-prism
- cclab-cli -> cclab-titan
- cclab-cli -> cclab-quasar
- cclab-cli -> cclab-orbit
- cclab-cli -> cclab-probe
- cclab-cli -> cclab-ion
- cclab-cli -> cclab-jet
- cclab-cli -> cclab-mamba
- cclab-server -> cclab-genesis
- cclab-server -> cclab-prism
- cclab-server -> cclab-aurora
- cclab-genesis -> cclab-prism
- cclab-genesis -> cclab-aurora
- cclab-prism -> cclab-aurora
