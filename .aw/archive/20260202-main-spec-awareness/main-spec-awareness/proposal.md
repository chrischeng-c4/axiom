---
id: main-spec-awareness
type: proposal
version: 1
created_at: 2026-02-02T10:30:01.889196+00:00
updated_at: 2026-02-02T10:30:01.889196+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add main spec awareness to planning workflow with new MCP tools and frontmatter fields"
history:
  - timestamp: 2026-02-02T10:30:01.889196+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 5
  new_files: 1
affected_specs:
  - id: main-spec-integration
    path: specs/main-spec-integration.md
    depends: []
---

<proposal>

# Change: main-spec-awareness

## Summary

Add main spec awareness to planning workflow with new MCP tools and frontmatter fields

## Why

Currently, the planning workflow operates in isolation within `genesis/changes/<change_id>`, lacking visibility into the existing "source of truth" specs in `cclab/specs/`. This leads to duplication or inconsistency. This change enables the planner to read existing specs and link new specs to them (e.g., as updates or children), improving traceability and keeping the main specs as the single source of truth.

## What Changes

- Implement list_main_specs and read_main_spec MCP tools
- Add spec_group, main_spec_ref, and merge_strategy to SpecFrontmatter
- Update planning prompts to instruct agents to check cclab/specs/
- Register new tools in mcp/registry.rs

## Impact

- **Scope**: minor
- **Affected Files**: ~5
- **New Files**: ~1
- Affected specs:
  - `main-spec-integration` (no dependencies)
- Affected code: `crates/cclab-genesis/src/mcp/tools/`, `crates/cclab-genesis/src/mcp/registry.rs`, `crates/cclab-genesis/src/models/frontmatter.rs`, `crates/cclab-genesis/src/orchestrator/prompts.rs`

</proposal>
