---
change_id: genesis-325-329
type: gap_spec_knowledge
created_at: 2026-02-14T10:03:29.322118+00:00
updated_at: 2026-02-14T10:03:29.322118+00:00
---

# Gap Analysis: Spec vs Knowledge

## Contradictions

| Gap ID | Description | Severity |
| :--- | :--- | :--- |
| G1 | **Generation Strategy Conflict**: Aurora's generators (to be migrated) use the Tera template engine (`template-engine` spec), while Prism's existing `gen/` module uses direct string generation. The knowledge base identifies this as a pitfall but doesn't define which pattern should prevail in the unified architecture. | HIGH |
| G2 | **SpecIR Contract vs Raw Input**: `spec-model.md` and `code-generator-contract.md` define a unified SpecIR input, but the `generator-*` specs in the spec context still describe direct consumption of SchemaIR and TemplateEngine contexts. | MEDIUM |

## Missing Patterns

| Gap ID | Description | Severity |
| :--- | :--- | :--- |
| G3 | **Plus Diagram Semantics in Generator Specs**: The knowledge base (`spec-model.md`) defines detailed mapping rules for Sequence+, Flowchart+ (SemanticType), and Requirement+ (N:M mapping). However, the existing `generator-fastapi`, `generator-express`, and `generator-axum` specs do not include requirements for consuming these semantics. | HIGH |
| G4 | **Test Generator Specification**: `code-generator-contract.md` explicitly defines Requirement+ to test file mapping rules, but the spec context lacks any specification for a Test Generator or the integration of Requirement+ into the code generation pipeline. | HIGH |
| G5 | **Inference Rules Implementation**: The "Inference rules (auto-detect DI needs)" pattern in `code-generator-contract.md` is entirely absent from the `aurora-codegen-system` and `generator-*` technical designs. | MEDIUM |
| G6 | **MCP Tool Exposure for Generators**: The knowledge base (`40-mcp/http-server.md`) mandates one-tool-one-capability granularity for MCP. While the spec context identifies the migration of generators to Prism, it lacks a specification for exposing these new generation capabilities through the MCP tool layer in a way that respects documented granularity constraints. | MEDIUM |

## Boundary Misalignments

| Gap ID | Description | Severity |
| :--- | :--- | :--- |
| G7 | **Implement Phase Integration**: The `implement-change` spec (cclab-genesis) defines the implementation loop but has no defined boundary for where structured codegen via Prism takes over from manual agent coding. The knowledge base describes the "Target Architecture" for this integration, but it isn't reflected in the phase routing tables of the spec. | MEDIUM |
| G8 | **SpecIR Format Boundary**: `SpecIR` is required to handle both structured JSON (OpenAPI) and diagram YAML (Mermaid+). The knowledge base documents these as separate paths, but the spec context doesn't clarify if `SpecIR` is a unified union type or separate trait-based implementations. | LOW |
