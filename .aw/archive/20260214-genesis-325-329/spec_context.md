---
change_id: genesis-325-329
type: spec_context
created_at: 2026-02-14T09:46:58.029761+00:00
updated_at: 2026-02-14T09:46:58.029761+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-aurora
  - cclab-prism
  - cclab-genesis
---

# Spec Context

## Relevant Specs

- **aurora-codegen-system** (group: cclab-aurora)
  - relevance: high
  - reason: Defines the current codegen architecture with pluggable generators (FastAPI/Express/Axum). These generators will be migrated to Prism.
  - key sections: R1 - Unified Internal Representation, R4 - Pluggable Generators, Aurora Codegen Data Flow
- **json-schema-core** (group: cclab-aurora)
  - relevance: high
  - reason: Core JSON Schema parser — forms the foundation of SpecIR. Will be extended to support Plus diagram semantics.
  - key sections: R1 - Version Support, R2 - Typed Structure
- **template-engine** (group: cclab-aurora)
  - relevance: high
  - reason: Tera template engine used by generators. Must migrate to Prism along with generators.
  - key sections: R1 - Tera Initialization, R3 - String Manipulation Filters
- **generator-fastapi** (group: cclab-aurora)
  - relevance: high
  - reason: FastAPI generator to be migrated from Aurora to Prism. Uses SchemaIR + TemplateEngine.
  - key sections: R1 - Input Mapping, R4 - Type Mapping, R6 - Deterministic Output
- **generator-express** (group: cclab-aurora)
  - relevance: high
  - reason: Express generator to be migrated from Aurora to Prism.
  - key sections: R1 - Template Set Resolution, R2 - Context Construction, R4 - Overwrite Policy
- **generator-axum** (group: cclab-aurora)
  - relevance: high
  - reason: Axum generator to be migrated from Aurora to Prism.
  - key sections: R1 - Generator Interface, R2 - Context Transformation, R3 - Model Generation
- **spec-validator** (group: cclab-aurora)
  - relevance: medium
  - reason: Validates JSON Schema completeness before code gen. Stays in Aurora as part of spec format layer.
  - key sections: R1 - Type Validation, R2 - Reference Validation
- **architecture** (group: cclab-aurora)
  - relevance: medium
  - reason: Aurora pipeline overview showing Input→Parser→Transformer→Generator→Output. The Generator→Output portion moves to Prism.
  - key sections: Generation Pipeline, Code-to-Diagram Flow
- **mermaid-plus-format** (group: cclab-aurora)
  - relevance: medium
  - reason: Plus format with YAML frontmatter and semantic metadata. SpecIR must capture these semantics.
- **implement-change** (group: cclab-genesis)
  - relevance: high
  - reason: The implement phase that needs integration with Prism codegen. Currently has per-task loop but no structured spec→code path.
  - key sections: Phase Routing Table, BeginImplementation prompt, ImplementTask prompt
- **analysis-tools** (group: cclab-prism)
  - relevance: low
  - reason: Analysis tools for requirements research. Not directly related but shows Prism's tool pattern.
- **prism-pdg-mcp-tools** (group: cclab-prism)
  - relevance: medium
  - reason: PDG tools show how Prism exposes code intelligence. New codegen tools should follow same MCP pattern.

## Dependencies

- aurora-codegen-system depends on json-schema-core, spec-validator, template-engine
- generator-fastapi depends on aurora-codegen-system, template-engine
- generator-express depends on aurora-codegen-system, template-engine
- generator-axum depends on aurora-codegen-system, template-engine
- implement-change consumes specs produced by plan phase, will need to consume SpecIR via Prism
- Prism already depends on cclab-aurora (Cargo.toml confirmed)

## Gaps

- No SpecIR type/trait defined yet — generators use raw JsonSchema/Spec directly
- Aurora generators don't consume Plus diagram semantics (Flowchart+, Class+, ERD+, Sequence+, Requirement+) — only JSON Schema/OpenAPI
- Prism's existing gen/ module has per-crate generators (Shield, Titan, etc.) but no unified SpecIR input contract
- Genesis implement phase prompt has no structured spec→code step — agents write code manually
- Knowledge docs spec-model.md and code-generator-contract.md define the target architecture but it's not yet implemented
- No spec exists for the SpecIR contract itself — needs to be created as part of this change
