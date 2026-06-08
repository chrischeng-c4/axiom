---
change_id: genesis-295-302
type: knowledge_context
created_at: 2026-02-13T07:41:16.983964+00:00
updated_at: 2026-02-13T07:41:16.983964+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - spec-to-code
  - mcp-patterns
  - workflow-orchestration
---

# Knowledge Context

## Relevant Documents

- **spec-to-code/spec-model.md**
  - summary: Defines the mapping between agnostic specifications (API, Sequence+, Flowchart+, etc.) and framework-specific code.
  - relevant sections: Spec Catalog, Plus Diagrams mapping rules
- **40-mcp/dynamic-config.md**
  - summary: Describes the strategy for filtering MCP tools based on the workflow stage to reduce cognitive load and token usage.
  - relevant sections: Tool Filtering by Stage, Tool Sets by Stage
- **cclab-genesis/verdict-unification.md**
  - summary: Standardizes all verdict enums to APPROVED, REVIEWED, and REJECTED to eliminate routing bugs.
  - relevant sections: R1 - Unify spec verdict names, Acceptance Criteria
- **cclab-genesis/run-change/README.md**
  - summary: Detailed guide for the run_change orchestrator, including the 8-location checklist for StatePhase updates and legacy v1 deprecations.
  - relevant sections: 8-Location Checklist, Legacy v1 removal

## Patterns

- **Tag-Union Validation Logic** (source: validation-refactor.md)
  - A spec's requirements are the union of requirements of all its tags. If a spec is tagged both 'api' and 'logic', it must satisfy both OpenAPI and Flowchart requirements.
- **Semantic Diagram Plus Pattern** (source: spec-system-evolution.md)
  - Use of Plus variants of Mermaid diagrams (Sequence+, Flowchart+, Class+, ERD+, Requirement+) to provide semantic metadata for code generation.
- **Unified Verdict Standard** (source: verdict-unification.md)
  - Consolidating verdict enums (PASS, NEEDS_REVISION, NEEDS_FIX, etc.) into a unified standard: APPROVED, REVIEWED, REJECTED.

## Pitfalls

- Routing bugs caused by inconsistent verdict naming across different tools (e.g. NEEDS_REVISION vs REVIEWED).
- Complexity and maintenance burden from keeping legacy v1 paths (e.g. 'explored', 'needs_followup' phases).
- Inability to represent multi-faceted components with a single SpecType enum.
