---
id: impl-merge-mcp-tools
type: proposal
version: 1
created_at: 2026-02-05T15:50:04.131985+00:00
updated_at: 2026-02-05T15:50:04.131985+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add genesis_impl_change and genesis_merge_change MCP workflow tools"
history:
  - timestamp: 2026-02-05T15:50:04.131985+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 3
  new_files: 2
affected_specs:
  - id: impl-change-tool
    path: specs/impl-change-tool.md
    depends: []
  - id: merge-change-tool
    path: specs/merge-change-tool.md
    depends: [impl-change-tool]
---

<proposal>

# Change: impl-merge-mcp-tools

## Summary

Add genesis_impl_change and genesis_merge_change MCP workflow tools

## Why

The Genesis SDD workflow has four phases: decide, plan, impl, and merge. Currently, only genesis_decide_change and genesis_plan_change MCP tools exist. Issues #168 and #169 require implementing the missing genesis_impl_change and genesis_merge_change MCP tools to complete the full workflow orchestration via MCP. These tools enable AI agents to orchestrate implementation and merge workflows through the standard MCP protocol, providing state-aware guidance for mainthread execution.

## What Changes

- Add genesis_impl_change MCP tool for orchestrating implementation workflow (Planned → Implementing → Testing → CodeReviewing → Implemented)
- Add genesis_merge_change MCP tool for orchestrating merge workflow (Implemented → Merging → Archived)
- Register both tools in mcp/tools/mod.rs with definitions and call_tool handlers
- Use new AgentsConfig API with WorkflowArtifact for agent configuration

## Impact

- **Scope**: minor
- **Affected Files**: ~3
- **New Files**: ~2
- Affected specs:
  - `impl-change-tool` (no dependencies)
  - `merge-change-tool` → depends on: `impl-change-tool`
- Affected code: `crates/cclab-genesis/src/mcp/tools/impl_change.rs`, `crates/cclab-genesis/src/mcp/tools/merge_change.rs`, `crates/cclab-genesis/src/mcp/tools/mod.rs`

</proposal>
