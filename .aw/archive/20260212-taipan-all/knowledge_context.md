---
change_id: taipan-all
type: knowledge_context
created_at: 2026-02-12T10:35:28.085933+00:00
updated_at: 2026-02-12T10:35:28.085933+00:00
iteration: 3
complexity: high
stage: knowledge
scanned_categories:
  - titan
  - mcp
  - grid
  - orbit
  - claude
  - changelogs
---

# Knowledge Context

## Relevant Documents

- **05-titan/architecture-guide.md**
  - summary: Guides the choice between Active Record (simplicity) and Data Mapper (complex business logic) in the Titan ORM.
  - relevant sections: Active Record Pattern, Data Mapper Pattern, When to Use Which?
- **40-mcp/index.md**
  - summary: Overview of MCP configuration and the strategy for dynamic tool loading based on workflow stage.
  - relevant sections: Dynamic Configuration Strategy, Overview
- **grid/formula-syntax.md**
  - summary: Detailed reference for Excel-compatible formula syntax and dynamic array behavior in cclab-grid.
  - relevant sections: Basic Syntax, Functions, Array Formulas
- **orbit/bridge-internals.md**
  - summary: In-depth technical guide on the Python-Rust bridge, focusing on thread safety, GIL handling, and memory management.
  - relevant sections: GIL Management, Waker Implementation, Error Propagation, Memory Ownership
- **30-claude/skills.md**
  - summary: Instructions for extending agent capabilities using the Skills framework.
  - relevant sections: How Skills Work, Creating Your First Skill, SKILL.md Configuration
- **40-mcp/dynamic-config.md**
  - summary: Technical implementation details for stage-specific MCP tool filtering to reduce token usage and cognitive load.
  - relevant sections: Tool Filtering by Stage, Tool Sets by Stage
- **changelogs/orbit-architecture.md**
  - summary: Records the addition of modular feature flags, custom memory allocation strategies, and platform-specific optimizations.
  - relevant sections: Feature Flags, Custom Slab Allocator, kqueue Configuration
- **changelogs/orbit-testing-safety.md**
  - summary: Documents the safety-first testing approach using fuzzing and Miri for undefined behavior detection.
  - relevant sections: Fuzz Testing Infrastructure, Miri CI Integration
- **changelogs/improve-titan-maturity.md**
  - summary: Upgrade cclab-titan maturity with dialect abstraction, robust transaction management, and enhanced validation capabilities.
  - relevant sections: Architectural Improvements, New Features, Technical Design Foundation

## Patterns

- **Active Record vs Data Mapper** (source: 05-titan/architecture-guide.md)
  - Flexible persistence patterns: Active Record for CRUD simplicity, Data Mapper for decoupling complex domain logic from the database.
- **Dialect Abstraction** (source: changelogs/improve-titan-maturity.md)
  - Decoupling SQL generation from specific databases (PostgreSQL, SQLite, MySQL) using a unified trait.
- **Dynamic MCP Configuration** (source: 40-mcp/dynamic-config.md)
  - Loading stage-specific MCP tool sets (Plan, Implement, Review, Archive) to optimize token usage and model performance.
- **Tokio-Asyncio Bridge** (source: orbit/bridge-internals.md)
  - Delegating Python asyncio operations to a background Rust Tokio runtime for high-performance I/O.
- **GIL Release Strategy** (source: orbit/bridge-internals.md)
  - Mandatory release of the GIL during blocking Rust operations or waits to allow Python concurrency and prevent deadlocks.
- **Agent Skills Framework** (source: 30-claude/skills.md)
  - Extending agent capabilities via metadata-enriched markdown files (SKILL.md) using a discovery and activation flow.
- **Modular Feature Flags** (source: changelogs/orbit-architecture.md)
  - Use of feature flags for modularity and platform-specific optimizations like kqueue on macOS/BSD.
- **Slab Allocator for Timers** (source: changelogs/orbit-architecture.md)
  - O(1) management of timer entries with ABA protection using generation-based key validation.

## Pitfalls

- GIL Deadlocks: Synchronous waits for cross-thread communication while holding the GIL trigger system-wide deadlocks.
- GIL Starvation: Non-adaptive Python callback batching can starve the main thread, causing latency spikes and unresponsive event loops.
- #SPILL! Errors: Data collision in the output range prevents dynamic array formulas from expanding, causing calculation failure.
- Skill Activation Failure: Ambiguous metadata in SKILL.md leads to incorrect tool selection or failure to trigger the intended skill.
- Reference Cycles: Cross-language reference loops (Python-to-Rust-to-Python) bypass standard garbage collection, resulting in unbounded memory growth.
