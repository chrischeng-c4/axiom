---
change: lens-beyond-ide
group: rpc-mcp-wiring
date: 2026-03-13
---

# Requirements

Wire existing refactoring engine (src/refactoring/) and semantic search engine (src/search/) into the daemon RPC layer and MCP tools.

1. handler.rs: Add RPC methods `refactor` (dispatches to RefactoringRegistry) and `search` (dispatches to SearchEngine), plus `call_graph` for hierarchy visualization.
2. mcp/tools.rs: Register 3 new MCP tools — `lens_refactor`, `lens_search`, `lens_call_graph` — with proper parameter schemas and daemon-backed execution.
3. Protocol: Add request/response types in protocol.rs for refactor and search methods.
4. Integration: search index should build on daemon startup and update incrementally via existing file watcher.
