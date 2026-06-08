---
change_id: genesis-372
type: knowledge_context
created_at: 2026-02-14T16:52:58.819928+00:00
updated_at: 2026-02-14T16:52:58.819928+00:00
iteration: 2
complexity: high
stage: knowledge
scanned_categories:
  - spec-to-code
  - 40-mcp
  - genesis-325-329
  - changelogs
  - 30-claude
  - 05-titan
  - grid
  - orbit
---

# Knowledge Context

## Relevant Documents

- **spec-to-code/index.md**
  - summary: Overview of spec-to-code pipeline architecture. 6 core spec types (API, Sequence+, Flowchart+, Class+, ERD+, Requirement+) map to code artifacts. Specs are language/framework-agnostic.
  - relevant sections: Spec Catalog, System Archetypes
- **spec-to-code/spec-model.md**
  - summary: Detailed spec catalog with mapping rules for all 6 spec types. Each spec type defines what it maps to in code. Sequence+ defines module boundaries, Flowchart+ defines function bodies, Requirement+ defines test scaffolding.
  - relevant sections: Spec Catalog table, SemanticType mapping, Sequence Plus Flowchart Plus Boundary, N:M mapping rules
- **spec-to-code/code-generator-contract.md**
  - summary: Generator contract: specs describe WHAT, generators decide HOW. Inference rules for auto-detection of DI, auth, DB dependencies. Current generators only consume API Spec (OpenAPI/JSON Schema). No Plus diagram consumption exists.
  - relevant sections: Generator Responsibilities, Inference Rules, Existing Generators, Current Gap
- **genesis-325-329/gap_codebase_knowledge.md**
  - summary: Gap analysis from previous change (genesis-325-329). Identifies 6 gaps including: generators don't follow spec-agnostic principle, SemanticType not used for code emission, CodeGenerator trait lacks Plus semantics, two code generation approaches coexist (Aurora templates vs Prism direct), inconsistent tool granularity.
  - relevant sections: Convention violations HIGH, Convention violations MEDIUM
- **40-mcp/index.md**
  - summary: MCP configuration overview. Genesis uses dynamic MCP config to load stage-specific tool sets. Exposing all 22 tools to every stage wastes tokens. Stage-specific tool loading reduces cognitive load and prompt token usage.
  - relevant sections: Stage-specific tools table, Problem, Solution

## Patterns

- **Spec-agnostic principle** (source: spec-to-code/spec-model.md)
  - Established convention: specs describe WHAT (language/framework-agnostic), generators decide HOW (framework-specific). Separation of concerns between specification and generation.
- **N:M mapping** (source: spec-to-code/spec-model.md)
  - One diagram can produce multiple code artifacts. Sequence+ produces module structure + DI wiring. Requirement+ produces test classes + test functions. The relationship between specs and code output is many-to-many.
- **SemanticType tags** (source: spec-to-code/spec-model.md)
  - Flowchart+ nodes carry SemanticType (db_query, api_call, validation, etc.) that correspond to framework-specific code patterns. Currently defined in knowledge but not consumed by any generator.
- **Dynamic MCP tool loading** (source: 40-mcp/index.md)
  - Stage-specific tool sets reduce token waste. Different workflow stages load different subsets of available MCP tools.
- **Generator contract** (source: spec-to-code/code-generator-contract.md)
  - Generators consume spec input and produce framework-specific code. Currently only API Spec (OpenAPI/JSON Schema) is consumed by existing generators (FastAPI, Express, Axum).

## Pitfalls

- Two code generation approaches coexist: Aurora uses Tera templates in crates/cclab-aurora/src/engine/, Prism uses direct string generation in crates/cclab-prism/src/gen/
- Current generators only consume API Spec / JSON Schema — no generator consumes Plus diagram types (Sequence+, Flowchart+, Class+, ERD+, Requirement+)
- Inconsistent tool granularity: Aurora exposes 21 separate diagram tools, Prism uses 1 monolithic prism_generate_from_spec tool
- SemanticType mapping is defined in knowledge but not implemented in any generator
- File size limit constraint: files exceeding 1000 lines require splitting per project convention
