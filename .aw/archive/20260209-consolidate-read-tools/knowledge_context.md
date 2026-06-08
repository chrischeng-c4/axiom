---
change_id: consolidate-read-tools
type: knowledge_context
created_at: 2026-02-09T07:09:12.688042+00:00
updated_at: 2026-02-09T07:09:12.688042+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - 40-mcp
  - 30-claude
  - index
---

# Knowledge Context

## Relevant Documents

- **40-mcp/index.md**
  - summary: MCP configuration overview. Explicitly identifies the problem: exposing all tools to every stage increases LLM cognitive load and wastes prompt tokens. Solution: dynamic MCP configuration with stage-specific tool sets.
  - relevant sections: Problem, Solution, Stage tool counts
- **40-mcp/dynamic-config.md**
  - summary: Dynamic MCP configuration per workflow stage. ToolRegistry has stage-specific filters (plan_tools, challenge_tools, implement_tools, review_tools, archive_tools).
  - relevant sections: Stage filtering

## Patterns

- **Stage-specific tool filtering** (source: 40-mcp/index.md)
  - ToolRegistry.new_for_stage() returns only tools needed per stage. Consolidating read tools reduces the total tool count, amplifying token savings across all stages.
- **Tool name references in prompts** (source: run_change modules)
  - All run_change modules embed MCP tool names as string literals in prompts (e.g. mcp__cclab-mcp__genesis_read_file). Consolidated tool must keep the same name to minimize prompt changes.

## Pitfalls

- Tool names are hardcoded in 30+ run_change prompt strings — must update all references
- Stage filters (challenge_tools, implement_tools, etc.) reference specific tool definitions — must update registrations
- genesis_read_file file parameter currently only handles change-scoped files — extending to knowledge/spec scope requires careful path resolution to prevent directory traversal
