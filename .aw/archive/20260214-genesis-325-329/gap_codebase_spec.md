---
change_id: genesis-325-329
type: gap_codebase_spec
created_at: 2026-02-14T09:52:56.684956+00:00
updated_at: 2026-02-14T09:52:56.684956+00:00
---

# Gap Analysis: Codebase vs Spec

## Code without matching spec

### HIGH severity

1. **Prism `spec/ir.rs` (SpecIR type)** — File exists in codebase but no spec defines the SpecIR contract, its fields, or its relationship to Aurora's schema types. This is the core type that #325 must create a spec for.

2. **Prism `gen/traits.rs` (CodeGenerator trait)** — The trait exists with `name()`, `can_generate()`, `generate()` but has no corresponding spec. The code-generator-contract.md knowledge doc describes the *desired* contract but there is no formal spec in `cclab/specs/cclab-prism/`.

3. **Prism per-crate generators (Shield, Titan, Nebula, Photon, Quasar, Serde, Axum, SQLx, Reqwest)** — 9 generators exist in `gen/python/` and `gen/rust/` with no specs. Their input/output contracts are undocumented.

### MEDIUM severity

4. **Aurora `diagrams/*/schema.rs` SemanticType fields** — Plus diagram schemas define SemanticType (flowchart_plus), Stereotype (class_plus), etc. These are documented in mermaid-plus-format spec but there is no spec for how these semantic fields map to SpecIR for code generation purposes.

5. **Prism `types/codegen.rs` (24KB)** — Code generation helpers exist but have no spec coverage.

6. **Genesis `implement.rs` prompt templates** — The implement-change spec documents the per-task loop, but the actual prompt templates in code contain no reference to structured code generation or Prism tools.

## Specs without matching implementation

### HIGH severity

1. **`code-generator-contract.md` Target Architecture** — Knowledge doc defines a target where generators consume all 6 spec types (API, Sequence+, Flowchart+, Class+, ERD+, Requirement+). Current implementation only consumes JSON Schema/OpenAPI.

2. **No SpecIR contract spec exists** — This is the central missing spec for #325.

### MEDIUM severity

3. **`aurora-codegen-system` R5 (Test Generation)** — Spec requires test generation alongside code. No test generation is implemented in current Aurora generators.

4. **`generator-fastapi` R6 (Deterministic Output)** — Spec requires byte-for-byte identical output. Not verified in current implementation.

## Summary

| Category | High | Medium | Low |
|----------|------|--------|-----|
| Code without spec | 3 | 3 | 0 |
| Spec without impl | 2 | 2 | 0 |
| **Total gaps** | **5** | **5** | **0** |"