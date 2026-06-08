---
change_id: genesis-372
type: gap_spec_knowledge
created_at: 2026-02-14T17:12:04.188159+00:00
updated_at: 2026-02-14T17:12:04.188159+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec-knowledge misalignments

### HIGH severity

1. **spec-ir-contract defines Rust enum but knowledge describes language-agnostic specs** — Spec `cclab-aurora/spec-ir-contract` defines SpecIR as a Rust enum (R1). Knowledge `spec-to-code/spec-model.md` establishes that specs are language/framework-agnostic. The Rust enum couples the IR to a specific language, contradicting the agnostic principle. (Spec: spec-ir-contract R1, Knowledge: spec-to-code/spec-model.md)

2. **No spec defines YAML manifest schema** — Clarifications for this change specify k8s/Kustomize-style YAML as the SpecIR format. No existing spec defines this YAML schema (apiVersion, kind, metadata, spec fields). Knowledge `spec-to-code/spec-model.md` documents 6 spec types that would need YAML representation. (Knowledge: spec-to-code/spec-model.md, spec-to-code/code-generator-contract.md)

### MEDIUM severity

3. **Spec responsibility boundary unclear between Aurora and Genesis** — Spec `cclab-aurora/aurora-codegen-system` places code generation in Aurora. Spec `cclab-genesis/genesis-implement-integration` places codegen orchestration in Genesis. Knowledge `40-mcp/index.md` describes dynamic tool loading per workflow stage. The Aurora-Genesis merge means spec responsibilities need to be reconciled. (Spec: aurora-codegen-system R1, genesis-implement-integration R1-R2, Knowledge: 40-mcp/index.md)

4. **Generator contract spec covers all 6 spec types but existing specs only define API Spec consumption** — Knowledge `spec-to-code/code-generator-contract.md` describes generators consuming all 6 spec types with detailed inference rules. Spec `cclab-prism/prism-codegen-unification` only specifies CodeGenerator trait accepting SpecIR generically without detailed per-type consumption rules. (Spec: prism-codegen-unification R2, Knowledge: code-generator-contract.md)

5. **Spec validator coverage gap** — Spec `cclab-aurora/spec-validator` defines validation for JSON Schema types (R1-R2). Knowledge `spec-to-code/spec-model.md` documents 6 spec types including Plus diagrams. No spec covers validation of Plus diagram types or YAML manifest validation. (Spec: spec-validator R1-R2, Knowledge: spec-to-code/spec-model.md)

### LOW severity

6. **Dynamic MCP tool loading not reflected in codegen specs** — Knowledge `40-mcp/index.md` describes stage-specific tool loading to reduce token waste. Neither spec-ir-contract nor prism-codegen-unification addresses how codegen tools should be loaded/configured per workflow stage. (Knowledge: 40-mcp/index.md, Spec: spec-ir-contract, prism-codegen-unification)

## Summary

| Category | High | Medium | Low |
|----------|------|--------|-----|
| Spec-knowledge misalignments | 2 | 3 | 1 |
| **Total gaps** | **2** | **3** | **1** |