---
change_id: consolidate-read-tools
type: spec_context
created_at: 2026-02-09T07:07:33.718010+00:00
updated_at: 2026-02-09T07:07:33.718010+00:00
iteration: 1
complexity: medium
stage: spec
scanned_groups:
  - cclab-genesis
---

# Spec Context

## Relevant Specs

- **run-change** (group: cclab-genesis)
  - relevance: high
  - reason: Defines the workflow that uses all read/list MCP tools. The Action-Phase-Agent mapping table references genesis_read_file, genesis_list_main_specs, genesis_read_main_spec, genesis_list_knowledge, genesis_read_knowledge. Consolidation changes which tools appear in run_change prompts.
  - key sections: Action Phase Agent Mapping, Decide Phase Transition Flowcharts, OpenRPC Specification
- **orchestrator** (group: cclab-genesis)
  - relevance: low
  - reason: Agent fallback system. Not directly affected — agents call MCP tools by name, so only tool names in prompts need updating.
  - key sections: Agent Fallback Flow

## Dependencies

- run-change depends on MCP tool registry (mod.rs)
- run-change prompts embed tool names like genesis_read_file

## Gaps

- run-change spec still references 37 phases (pre-gap-analysis), needs update to 48 phases
- run-change spec Action table references genesis_list_main_specs and genesis_read_main_spec which will be removed
