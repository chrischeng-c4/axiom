---
change_id: genesis-325-329
type: knowledge_context
created_at: 2026-02-14T09:49:43.180323+00:00
updated_at: 2026-02-14T09:49:43.180323+00:00
iteration: 2
complexity: high
stage: knowledge
scanned_categories:
  - spec-to-code
  - 40-mcp
  - changelogs
  - index
---

# Knowledge Context

## Relevant Documents

- **spec-to-code/spec-model.md**
  - summary: Defines 6 core spec types (API, Sequence+, Flowchart+, Class+, ERD+, Requirement+) and 4 supplementary specs. Each spec type maps to specific code artifacts. System archetypes define which spec combinations apply. Includes detailed mapping rules for Sequence+ (module boundaries, function signatures, DI), Flowchart+ (SemanticType per node), and Requirement+ (N:M test generation).
  - relevant sections: Spec Catalog table, System Archetypes, Sequence Plus mapping rules, Flowchart Plus SemanticType table, Requirement Plus mapping rules, Spec Interactions diagram
- **spec-to-code/code-generator-contract.md**
  - summary: Defines the contract between specs and framework-specific code generators. Covers: input mapping (spec element to code output), inference rules (auto-detect DI needs from spec signals), Sequence+ to code at macro/micro levels, Requirement+ to test file generation. Documents current state: existing generators only consume API Spec (OpenAPI/JSON Schema), Plus diagram semantics are not consumed.
  - relevant sections: Generator Responsibilities table, Inference Rules table, Sequence Plus Code Mapping, Requirement Plus Test Mapping, Current Gap section, Target Architecture diagram
- **spec-to-code/index.md**
  - summary: Index page for the spec-to-code pipeline knowledge area. Links to spec-model.md and code-generator-contract.md.
- **40-mcp/http-server.md**
  - summary: HTTP MCP server architecture for multi-project support. Documents how MCP tools are exposed to clients via Streamable HTTP transport on localhost:3000 with project isolation via X-Genesis-Project header.
  - relevant sections: Transport Protocol, Client Configuration

## Patterns

- **Spec-agnostic design** (source: spec-to-code/spec-model.md)
  - Specs describe WHAT (language/framework agnostic). Generators decide HOW (framework-specific). This separation is the existing core principle in the knowledge base.
- **SemanticType-driven codegen** (source: spec-to-code/spec-model.md)
  - Flowchart+ nodes carry SemanticType (validation, db_query, api_call, transform, etc.) that maps directly to code constructs. These semantics are defined in Aurora's flowchart_plus schema.
- **N:M mapping for Requirement+** (source: spec-to-code/code-generator-contract.md)
  - One Requirement+ diagram produces N test classes (one per requirement) with M test functions (one per scenario). Module 'satisfies' links determine imports.
- **Pluggable Generator trait** (source: spec-to-code/code-generator-contract.md)
  - Generators implement CodeGenerator trait with name(), can_generate(spec), generate(spec, ctx). This pattern exists in Prism's gen/ module already.
- **MCP tool-per-capability granularity** (source: 40-mcp/http-server.md)
  - Each discrete capability in the system is exposed as a separate MCP tool (e.g. prism_generate_from_spec, prism_symbols). Existing tools follow this one-tool-one-capability pattern.

## Pitfalls

- Current generators only consume API Spec (JSON Schema/OpenAPI) — Plus diagram semantics (Flowchart+, Class+, ERD+, Sequence+, Requirement+) are not consumed by any generator today
- Aurora's generators use Tera template engine while Prism's existing gen/ module uses direct string generation — two different code generation approaches coexist
- SpecIR would handle both structured specs (OpenAPI JSON) and diagram specs (Mermaid+ YAML frontmatter) — two different input formats with different parsing paths
- Prism depends on cclab-aurora; adding SpecIR types to Aurora means Aurora's types become part of the cross-crate public API
- Genesis implement phase has per-task loop with manual agent coding as the existing path — structured codegen would be an additional path alongside it
