---
change_id: genesis-325-329
type: gap_codebase_knowledge
created_at: 2026-02-14T09:58:57.178476+00:00
updated_at: 2026-02-14T09:58:57.178476+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention violations

### HIGH severity

1. **Generators don't follow spec-agnostic principle** — Knowledge doc `spec-to-code/spec-model.md` establishes that specs describe WHAT, generators decide HOW. Current Aurora generators (FastAPI, Express, Axum) directly couple to JSON Schema/OpenAPI input format rather than consuming a language-agnostic SpecIR. (Files: `crates/cclab-aurora/src/generators/*.rs`, Knowledge: `spec-to-code/spec-model.md`)

2. **SemanticType mapping not used for code emission** — Knowledge doc `spec-to-code/spec-model.md` defines a SemanticType-to-code mapping table (validation→input validation, db_query→database SELECT, etc.). While `crates/cclab-aurora/src/diagrams/flowchart_plus/generator.rs` uses `SemanticType::Start` for topological ordering during diagram generation, no generator (e.g., in `crates/cclab-aurora/src/generators/`) consumes these semantic tags to emit corresponding framework logic or validation code. (Knowledge: `spec-to-code/spec-model.md` Flowchart Plus SemanticType table)

### MEDIUM severity

3. **CodeGenerator trait lack of Plus semantics** — Knowledge doc `spec-to-code/code-generator-contract.md` describes generators consuming all 6 spec types (including Plus diagrams) with detailed inference rules. Prism's `CodeGenerator` trait in `crates/cclab-prism/src/gen/traits.rs` defines typed methods (e.g., `generate_data_models(&DataModelSpec)`) but the underlying SpecIR and trait signatures currently lack fields or methods to carry/process Sequence+, Flowchart+, or Requirement+ semantics. (Files: `crates/cclab-prism/src/gen/traits.rs`, Knowledge: `spec-to-code/code-generator-contract.md`)

4. **Two code generation approaches coexist** — Knowledge doc `spec-to-code/code-generator-contract.md` lists Aurora generators using Tera templates. Prism's `gen/` module uses direct string generation. Two divergent patterns for the same purpose. (Files: `crates/cclab-aurora/src/engine/`, `crates/cclab-prism/src/gen/`)

5. **Inconsistent tool granularity** — Pattern mismatch between Prism and Aurora MCP tools. Aurora follows a granular pattern exposing 21 separate diagram tools (e.g., `aurora_generate_flowchart_plus`, `aurora_generate_sequence_plus`), while Prism uses a monolithic `prism_generate_from_spec` tool that handles multiple tech stacks and spec types via arguments. This diverges from the granular tool pattern established in `cclab/knowledge/40-mcp/dynamic-config.md` for diagramming. (Files: `crates/cclab-prism/src/mcp/tools.rs`)

### LOW severity

6. **N:M Requirement+ test generation not implemented** — Knowledge doc `spec-to-code/code-generator-contract.md` describes detailed Requirement+→test mapping (N requirements → N test classes, M scenarios → M test functions). Automated search of `crates/cclab-prism/src/gen/` and `crates/cclab-aurora/src/generators/` confirms that no generator currently implements this verification logic or test scaffolding. (Knowledge: `spec-to-code/code-generator-contract.md` Requirement Plus Test Mapping)

## Summary

| Category | High | Medium | Low |
|----------|------|--------|-----|
| Convention violations | 2 | 3 | 1 |
| **Total gaps** | **2** | **3** | **1** |
