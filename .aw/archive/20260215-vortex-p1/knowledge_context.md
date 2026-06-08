---
change_id: vortex-p1
type: knowledge_context
created_at: 2026-02-14T17:08:59.985925+00:00
updated_at: 2026-02-14T17:08:59.985925+00:00
iteration: 2
complexity: high
stage: knowledge
scanned_categories:
  - index
  - 30-claude
  - 40-mcp
  - 05-titan
  - spec-to-code
  - changelogs
  - grid
  - orbit
  - improve-quasar-maturity-changelog
  - 178-grid-db-refactor-changelog
---

# Knowledge Context

## Relevant Documents

- **40-mcp/index.md**
  - summary: MCP configuration overview covering dynamic tool loading per workflow stage. Documents stage-based tool set selection (Plan=22 tools, Implement=4-5, Review=3-4).
  - relevant sections: Stage-based MCP tool sets, Dynamic MCP configuration
- **40-mcp/http-server.md**
  - summary: HTTP MCP server architecture: multi-project support via X-Genesis-Project header, localhost:3000, JSON-RPC 2.0 over HTTP, dynamic registry reloading, project context switching.
  - relevant sections: Architecture, Transport Protocol, Client Configuration, Project Context Switching, Registry File
- **40-mcp/dynamic-config.md**
  - summary: Dynamic MCP configuration strategy for loading stage-specific tool sets at runtime.
  - relevant sections: Dynamic tool loading
- **spec-to-code/spec-model.md**
  - summary: 6 core spec types: API Spec, Sequence+, Flowchart+, Class+, ERD+, Requirement+. State+ listed as supplementary spec for systems with explicit phase transitions. Workflow/state machine archetype uses API Spec + State+ + Sequence+ + Flowchart+ + Requirement+.
  - relevant sections: System Archetypes, Supplementary Specs (State Plus), Requirement Plus, Sequence Plus
- **spec-to-code/code-generator-contract.md**
  - summary: Code generator contract defining how specs map to framework-specific code. Specs describe WHAT (language-agnostic), generators handle HOW (framework-specific).
  - relevant sections: Generator contract
- **orbit/bridge-internals.md**
  - summary: Orbit bridge internals documenting async event patterns for cross-boundary communication. Includes event loop architecture and async dispatch mechanisms.
  - relevant sections: Event loop architecture, Async patterns
- **index.md**
  - summary: Root knowledge base index. Lists three main categories: 30-claude (Claude Code), 40-mcp (MCP configuration), spec-to-code (pipeline architecture).
  - relevant sections: Contents
- **30-claude/skills.md**
  - summary: Claude Code Skills documentation: auto-triggered markdown instruction files with YAML metadata. Covers skill creation, progressive disclosure, forked context, tool restrictions.
  - relevant sections: How Skills Work, Skill Structure

## Patterns

- **MCP Tool Registration via HTTP Server** (source: 40-mcp/http-server.md)
  - MCP tools are registered via HTTP server with project isolation headers (X-Genesis-Project). Server runs on localhost:3000, uses JSON-RPC 2.0 over HTTP, with dynamic registry at ~/.genesis/registry.json.
- **State Machine Spec Archetype** (source: spec-to-code/spec-model.md)
  - Systems with explicit phase transitions use the workflow/state machine archetype: API Spec + State+ + Sequence+ + Flowchart+ + Requirement+. State+ is a supplementary spec type for phase transition modeling.
- **Distributed Slice CLI Registration** (source: 30-claude/skills.md)
  - The cclab project uses linkme distributed_slice for CLI module auto-registration. IonCli is an existing example that implements the CliModule trait and registers via #[distributed_slice(CLI_MODULES)].
- **ECS Sparse Set Architecture** (source: spec-to-code/spec-model.md)
  - Vortex ECS uses sparse set component storage with a World/System framework. Systems are the unit of game logic execution within the ECS tick loop.
- **Async Event Bridge Pattern** (source: orbit/bridge-internals.md)
  - Orbit implements async event patterns for cross-boundary communication, combining sync dispatch within the main loop and async listeners for external integration.

## Pitfalls

- Tokio worker thread stack overflow causes silent server crashes with no log output (documented in issue #182).
- Project file size constraint: files >= 1000 lines must be split; files >= 500 lines are candidates for splitting.
- WASM builds require rustup toolchain (not Homebrew rustc) due to missing wasm target in Homebrew installation.
- MCP server port 3000 conflicts are possible if another service occupies the port. cclab server list shows current status.
- Rayon parallel system execution in ECS requires care with shared mutable state across concurrent systems.
