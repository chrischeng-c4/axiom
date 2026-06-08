---
id: consolidate-read-tools
type: proposal
version: 1
created_at: 2026-02-09T07:12:38.620388+00:00
updated_at: 2026-02-09T07:12:38.620388+00:00
author: mcp
status: proposed
iteration: 1
summary: "Consolidate 6 genesis read/list MCP tools into unified genesis_read_file with scope prefixes"
history:
  - timestamp: 2026-02-09T07:12:38.620388+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 10
  new_files: 0
affected_specs:
  - id: consolidate-read-tools
    path: specs/consolidate-read-tools.md
    depends: []
---

<proposal>

# Change: consolidate-read-tools

## Summary

Consolidate 6 genesis read/list MCP tools into unified genesis_read_file with scope prefixes

## Why

The genesis MCP tool registry currently exposes 9 separate read/list tools (genesis_read_file, genesis_list_specs, genesis_read_knowledge, genesis_list_knowledge, genesis_read_main_spec, genesis_list_main_specs, genesis_read_all_requirements, genesis_read_implementation_summary, genesis_list_changed_files). Each tool definition sent to agents costs ~200-500 tokens in the MCP tool listing. Since every agent invocation receives the full tool listing, this overhead compounds across the entire workflow.

Six of these tools are fundamentally file-reading operations that differ only in their target directory scope (change artifacts, knowledge base, main specs). By consolidating them into genesis_read_file with scope prefixes in the file parameter, we reduce the tool count by 6, saving ~1500 tokens per agent invocation. This aligns with the documented goal in 40-mcp/index.md: "exposing all tools to every stage increases LLM cognitive load and wastes prompt tokens."

Git-related tools (read_implementation_summary, list_changed_files) and write tools (write_knowledge, write_main_spec) are kept separate since they have fundamentally different semantics and parameters.

## What Changes

- Extend genesis_read_file file parameter to support scope prefixes: knowledge:path, main_spec:group/id, list:knowledge, list:main_specs, list:specs, requirements
- Extend file_service.rs read_file() to route scope-prefixed requests to existing service functions (knowledge_service, main_spec listing, requirements aggregation)
- Update read.rs tool definition schema to document the new file parameter syntax
- Remove 6 tool registrations from ToolRegistry (all_tools_vec, stage filters, call_tool dispatch)
- Update run_change prompt strings in explore_spec.rs and explore_knowledge.rs to use genesis_read_file with scope prefixes instead of removed tool names
- Delete knowledge.rs and main_spec.rs tool modules (services remain)

## Impact

- **Scope**: minor
- **Affected Files**: ~10
- **New Files**: ~0
- Affected specs:
  - `consolidate-read-tools` (no dependencies)
- Affected code: `crates/cclab-genesis/src/mcp/tools/read.rs`, `crates/cclab-genesis/src/services/file_service.rs`, `crates/cclab-genesis/src/mcp/tools/mod.rs`, `crates/cclab-genesis/src/mcp/tools/knowledge.rs (remove)`, `crates/cclab-genesis/src/mcp/tools/main_spec.rs (remove)`, `crates/cclab-genesis/src/mcp/tools/implementation.rs (partial)`, `crates/cclab-genesis/src/mcp/tools/run_change/explore_spec.rs`, `crates/cclab-genesis/src/mcp/tools/run_change/explore_knowledge.rs`
- **Breaking Changes**: MCP tool names genesis_read_knowledge, genesis_list_knowledge, genesis_read_main_spec, genesis_list_main_specs, genesis_read_all_requirements, genesis_list_specs will no longer exist. Agents must use genesis_read_file with scope prefixes instead.

</proposal>
