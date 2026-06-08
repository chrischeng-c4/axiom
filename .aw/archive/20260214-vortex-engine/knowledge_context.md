---
change_id: vortex-engine
type: knowledge_context
created_at: 2026-02-14T06:17:39.533586+00:00
updated_at: 2026-02-14T06:17:39.533586+00:00
iteration: 2
complexity: high
stage: knowledge
scanned_categories:
  - spec-to-code
  - 40-mcp
  - 05-titan
  - orbit
---

# Knowledge Context

## Relevant Documents

- **spec-to-code/index.md**
  - summary: Overview of the spec-to-code pipeline architecture.
  - relevant sections: Contents
- **spec-to-code/spec-model.md**
  - summary: Describes the 6 core spec types and how they map to code components.
  - relevant sections: Spec Catalog, System Archetypes, Spec Interactions
- **spec-to-code/code-generator-contract.md**
  - summary: Defines how generators should infer framework-specific code from agnostic specs.
  - relevant sections: Generator Responsibilities, Inference Rules
- **40-mcp/index.md**
  - summary: Strategy for stage-specific MCP tool loading to reduce cognitive load.
  - relevant sections: Dynamic Configuration
- **40-mcp/claude-mcp.md**
  - summary: Details on how Claude Code is configured with dynamic MCP servers.
  - relevant sections: Runtime MCP Configuration, Genesis Integration Strategy
- **05-titan/architecture-guide.md**
  - summary: Guidance on choosing between Active Record and Data Mapper patterns, preferring Data Mapper for complex systems.
  - relevant sections: Data Mapper Pattern, When to Use Which?
- **orbit/performance-tuning.md**
  - summary: Performance baseline and optimization strategies for the Orbit event loop.
  - relevant sections: Performance Comparison, Low-Latency Service, Profiling Guide

## Patterns

- **Spec-to-Code Mapping** (source: spec-to-code/spec-model.md)
  - Use 6 core spec types to describe architecture. Example: A 'Sequence+' participant 'AuthHandler' maps to a class in 'src/handlers/auth.py'.
- **Dynamic MCP Configuration** (source: 40-mcp/index.md)
  - Load stage-specific tool sets. Example: 'genesis/mcp-configs/implement.json' exposes only 4 tools (read_all_requirements, list_changed_files, etc.) to reduce LLM overhead.
- **Data Mapper Pattern** (source: 05-titan/architecture-guide.md)
  - Use 'Session' for coordinating persistence. Example: 'async with Session() as session: session.add(user); await session.commit()' keeps models pure.
- **Async Performance Tuning** (source: orbit/performance-tuning.md)
  - Optimize for latency. Example: Use 'loop.set_slow_callback_duration(0.05)' and 'orbit.install()' for high-throughput event processing.

## Pitfalls

- Exposing too many MCP tools to the LLM increases cognitive load and wastes tokens.
- GIL contention in multi-language environments can degrade performance if not handled by releasing GIL for long-running Rust tasks.
- Inefficient callbacks can block the async event loop; use debug mode to detect slow callbacks.
