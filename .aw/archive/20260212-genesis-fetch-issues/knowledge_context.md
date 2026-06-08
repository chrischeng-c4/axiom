---
change_id: genesis-fetch-issues
type: knowledge_context
created_at: 2026-02-11T17:07:07.301252+00:00
updated_at: 2026-02-11T17:07:07.301252+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - 05-titan
  - 178-grid-db-refactor
  - 30-claude
  - 40-mcp
  - changelogs
  - grid
  - orbit
---

# Knowledge Context

## Relevant Documents

- **40-mcp/claude-mcp.md**
  - summary: Describes how Claude Code loads stage-specific MCP configurations via --mcp-config and --strict-mcp-config flags.
  - relevant sections: Runtime MCP Configuration, Genesis Integration Strategy
- **40-mcp/dynamic-config.md**
  - summary: Outlines the strategy for filtering the 22 available MCP tools into smaller, stage-specific sets (Plan, Implement, Review, Archive).
  - relevant sections: Tool Filtering by Stage, Tool Sets by Stage
- **40-mcp/http-server.md**
  - summary: Details the HTTP MCP server implementation which solves stdio buffering issues and supports multi-project isolation using headers.
  - relevant sections: Architecture, Key Features, Registry File
- **30-claude/skills.md**
  - summary: Explains Claude Code Agent Skills, which are markdown files providing specialized knowledge and instructions that are automatically triggered.
  - relevant sections: How Skills Work, Creating Your First Skill

## Patterns

- **Stage-Specific MCP Tool Filtering** (source: 40-mcp/dynamic-config.md)
  - Filters exposed MCP tools based on the current workflow stage (e.g., 4 tools for Implement vs 22 for Plan) to reduce LLM cognitive load and token usage.
- **HTTP MCP Transport** (source: 40-mcp/http-server.md)
  - Uses HTTP POST with JSON-RPC over port 3000 instead of stdio to avoid transport-level buffering issues in complex terminal environments.
- **Project Isolation via HTTP Headers** (source: 40-mcp/http-server.md)
  - Identifies projects using X-Genesis-Project and X-Genesis-Cwd headers, allowing a single global server instance to handle multiple repository contexts.
- **Global Project Registry** (source: 40-mcp/http-server.md)
  - Maintains a global registry at ~/.genesis/registry.json to track project paths and the daemon server PID.

## Pitfalls

- Stdio transport buffering issues in Pipe or similar terminal environments can cause the MCP connection to hang.
- Exposing the full set of 22 tools to all stages increases the risk of the LLM selecting an inappropriate or irrelevant tool.
- Failing to set the correct working directory context (X-Genesis-Cwd) in multi-project environments can lead to operations being performed in the wrong path.
