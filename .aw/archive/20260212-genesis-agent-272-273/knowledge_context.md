---
change_id: genesis-agent-272-273
type: knowledge_context
created_at: 2026-02-12T11:27:31.387367+00:00
updated_at: 2026-02-12T11:27:31.387367+00:00
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
  - summary: Genesis MCP configuration overview. Documents dynamic MCP tool loading per workflow stage (plan/implement/review/archive). Relevant because delegate_agent spawns sub-agents that need appropriate MCP tool access.
  - relevant sections: Stage-specific tool sets, Tool filtering per stage
- **40-mcp/http-server.md**
  - summary: HTTP MCP server architecture on port 3000 with multi-project support. Relevant for understanding how sub-agents connect to MCP tools (stdio vs HTTP transport).
  - relevant sections: Client Configuration, Transport Protocol
- **30-claude/skills.md**
  - summary: Claude Code skills system. genesis_delegate_agent needs corresponding skill update when renamed. Skills use SKILL.md with allowed-tools restrictions.
  - relevant sections: SKILL.md Configuration, Restricting Tool Access, user-invocable

## Patterns

- **MCP tool response convention** (source: 40-mcp/index.md)
  - MCP tools return structured JSON with status, artifacts, phase, and next step hints. Responses are concise — artifact content is read separately via genesis_read_file.
- **Recursion prevention** (source: 40-mcp/index.md)
  - Sub-agents are blocked from calling genesis_agent via --disallowedTools (Claude), disabled_tools config (Codex), or excludeTools (Gemini).
- **Skill naming convention** (source: 30-claude/skills.md)
  - Skills use lowercase with hyphens. genesis_agent skill would need renaming to match genesis_delegate_agent.

## Pitfalls

- MCP response size: large responses trigger Claude Code context warnings (~10.6k tokens threshold)
- Sub-agent MCP access: spawned agents need correct MCP server config (headers, URL) to call genesis tools
- Skill rename: changing MCP tool name requires updating SKILL.md, skill directory name, and any references in CLAUDE.md
