---
change_id: prompt-registry
type: knowledge_context
created_at: 2026-02-08T17:43:22.612047+00:00
updated_at: 2026-02-08T17:43:22.612047+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - 30-claude
  - 40-mcp
  - changelogs
  - index
---

# Knowledge Context

## Relevant Documents

- **40-mcp/index.md**
  - summary: MCP configuration overview for genesis workflow stages. Documents which MCP tools are exposed per stage. Relevant because prompt templates reference MCP tool names.
  - relevant sections: Stage-tool mapping table

## Patterns

- **Inline prompt generation** (source: run_change flow files)
  - Each flow file generates prompt strings via format!() with change_id, project_path, and action-specific parameters. No shared template infrastructure exists.
- **Tool name constants** (source: helpers.rs)
  - GENESIS_AGENT_TOOL and GENESIS_RUN_CHANGE_TOOL are already extracted as constants. This pattern should extend to prompt fragments.

## Pitfalls

- Knowledge base has no documentation on prompt architecture or template patterns
- Previous prompt-registry spec (v1) is already done — this change extends it, not replaces it
