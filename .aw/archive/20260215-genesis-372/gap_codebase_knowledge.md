---
change_id: genesis-372
type: gap_codebase_knowledge
created_at: 2026-02-14T17:10:43.204744+00:00
updated_at: 2026-02-14T17:10:43.204744+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention violations

### HIGH severity

1. **Generators don't follow spec-agnostic principle** — Knowledge `spec-to-code/spec-model.md` establishes specs describe WHAT, generators decide HOW. Aurora generators (`crates/cclab-aurora/src/generators/fastapi.rs`, `crates/cclab-aurora/src/generators/express.rs`, `crates/cclab-aurora/src/generators/axum.rs`) directly couple to JSON Schema/OpenAPI input rather than consuming language-agnostic SpecIR. (Knowledge: spec-to-code/spec-model.md)

2. **Two code generation approaches coexist** — Knowledge `spec-to-code/code-generator-contract.md` describes a unified generator contract. Aurora uses Tera templates (`crates/cclab-aurora/src/engine/`), Prism uses direct string generation (`crates/cclab-prism/src/gen/traits.rs`, `crates/cclab-prism/src/gen/registry.rs`). Two divergent patterns for the same purpose. (Knowledge: spec-to-code/code-generator-contract.md)

### MEDIUM severity

3. **SemanticType mapping not used for code emission** — Knowledge `spec-to-code/spec-model.md` defines SemanticType-to-code mapping (validation→input validation, db_query→database SELECT). Aurora flowchart generator (`crates/cclab-aurora/src/diagrams/flowchart_plus/generator.rs`) uses SemanticType::Start for ordering but no generator in Aurora (`crates/cclab-aurora/src/generators/`) or Prism (`crates/cclab-prism/src/gen/python/`, `crates/cclab-prism/src/gen/rust/`) consumes these semantic tags for code emission. (Knowledge: spec-to-code/spec-model.md, Flowchart Plus SemanticType table)

4. **CodeGenerator trait lacks Plus semantics** — Knowledge `spec-to-code/code-generator-contract.md` describes generators consuming all 6 spec types. Prism's CodeGenerator trait (`crates/cclab-prism/src/gen/traits.rs`) defines typed methods but current signatures lack fields to carry Sequence+, Flowchart+, or Requirement+ semantics. (Knowledge: code-generator-contract.md)

5. **Inconsistent tool granularity** — Aurora exposes 21 separate diagram MCP tools via `crates/cclab-aurora/src/mcp/`, Prism uses 1 monolithic `prism_generate_from_spec` tool in `crates/cclab-prism/src/mcp/tools.rs`. This diverges from the granular tool pattern documented in knowledge. (Files: `crates/cclab-aurora/src/mcp/`, `crates/cclab-prism/src/mcp/tools.rs`, Knowledge: 40-mcp/dynamic-config.md)

### LOW severity

6. **N:M Requirement+ test generation not implemented** — Knowledge `spec-to-code/code-generator-contract.md` describes Requirement+→test mapping. No generator in Aurora (`crates/cclab-aurora/src/generators/`) or Prism (`crates/cclab-prism/src/gen/`) implements test scaffolding from Requirement Plus diagrams. (Knowledge: code-generator-contract.md, Requirement Plus Test Mapping)

## Summary

| Category | High | Medium | Low |
|----------|------|--------|-----|
| Convention violations | 2 | 3 | 1 |
| **Total gaps** | **2** | **3** | **1** |