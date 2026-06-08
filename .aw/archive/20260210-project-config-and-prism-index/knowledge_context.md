---
change_id: project-config-and-prism-index
type: knowledge_context
created_at: 2026-02-10T06:07:03.210958+00:00
updated_at: 2026-02-10T06:07:03.210958+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - 05-titan
  - 30-claude
  - 40-mcp
  - changelogs
  - grid
  - orbit
---

# Knowledge Context

## Relevant Documents

- **40-mcp/index.md**
  - summary: Describes the problem of high tool count for LLMs and the solution of dynamic MCP configuration to reduce cognitive load.
  - relevant sections: Problem, Solution
- **40-mcp/dynamic-config.md**
  - summary: Detailes the strategy for loading stage-specific tool sets at runtime to improve efficiency.
  - relevant sections: Tool Filtering by Stage, Implementation Strategy
- **40-mcp/claude-mcp.md**
  - summary: Covers how Claude Code supports runtime MCP configuration via command-line flags and the integration strategy for Genesis.
  - relevant sections: Runtime MCP Configuration, Genesis Integration Strategy
- **main_spec:cclab-server/prism-init-spec.md**
  - summary: Defines automatic initialization and pre-indexing of projects to improve responsiveness and ensure persistence across restarts.
  - relevant sections: Overview, Requirements

## Patterns

- **Dynamic MCP Configuration** (source: 40-mcp/dynamic-config.md)
  - Tools are filtered and loaded based on the current workflow stage (Plan, Implement, Review, Archive) to optimize LLM performance.
- **Persistent Registry** (source: main_spec:cclab-server/prism-init-spec.md)
  - Project registries and configuration settings are persisted to disk (e.g., ~/.cclab/registry.json) to survive restarts.
- **Non-blocking Background Initialization** (source: main_spec:cclab-server/prism-init-spec.md)
  - Potentially long-running tasks like code indexing should be performed in the background to avoid blocking system startup or user requests.

## Pitfalls

- Exposing too many tools to an LLM increases cognitive load and token usage, making tool selection less reliable.
- Lazy initialization of code indexes can cause significant delays on the first user request; pre-indexing is preferred.
- Naively overwriting configuration files on startup can lead to loss of state (e.g., after a crash); merging or careful loading is required.
