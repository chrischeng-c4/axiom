---
change_id: sdd-p1
type: knowledge_context
created_at: 2026-02-23T14:14:20.721543+00:00
updated_at: 2026-02-23T14:14:20.721543+00:00
iteration: 1
complexity: medium
stage: knowledge
scanned_categories:
  - mcp-config
  - mcp-server
  - orchestrator
  - spec-to-code
  - workflow-orchestration
---

# Knowledge Context

## Relevant Documents

- **knowledge:40-mcp/index.md**
  - summary: Overview of MCP configuration, dynamic tool filtering, and HTTP server for SDD.
- **knowledge:40-mcp/dynamic-config.md**
  - summary: Details the strategy for stage-specific tool sets (Plan: 22, Implement: 4, Review: 3).
- **knowledge:40-mcp/http-server.md**
  - summary: Describes the HTTP MCP server architecture for multi-project support and pipe safety.
- **knowledge:spec-to-code/code-generator-contract.md**
  - summary: Defines the responsibilities of generators in mapping specs to code and tests.
- **knowledge:genesis-372-impact.md**
  - summary: Migration strategy to YAML-based spec IR to eliminate token relay overhead.

## Patterns

- **Unified Artifact Management** (source: src/mcp/tools/mod.rs)
  - Unified artifact tools replace ~15 dedicated tools.
- **Dynamic MCP Configuration** (source: knowledge:40-mcp/dynamic-config.md)
  - Filter tools based on workflow stage (plan, implement, etc.).
- **HTTP MCP Server** (source: knowledge:40-mcp/http-server.md)
  - Global HTTP server on port 3000 with X-SDD-Project header for isolation.
- **Unified Workflow Tool (sdd_run_change)** (source: src/mcp/tools/run_change/mod.rs)
  - Pure state machine + prompt provider for workflow orchestration.
- **Spec-to-Code Generator Contract** (source: knowledge:spec-to-code/code-generator-contract.md)
  - Map agnostic specs to framework-specific code/tests.

## Pitfalls

- Stdout buffering in Pipe transport for large responses (use HTTP).
- LLM cognitive load and token waste from too many tools (use filtering).
- Token relay overhead for large generated specs (use YAML IR).
