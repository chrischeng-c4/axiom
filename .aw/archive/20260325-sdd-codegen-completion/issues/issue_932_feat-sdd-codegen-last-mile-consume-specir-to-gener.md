---
number: 932
title: "feat(sdd): codegen last mile — consume SpecIR to generate code"
state: open
labels: [enhancement, P1, crate:sdd]
group: "core-codegen"
---

# #932 — feat(sdd): codegen last mile — consume SpecIR to generate code

## Summary

All codegen infrastructure layers are complete (SpecIR 6 variants, SemanticType 12 typed operations, SpecBundle dependency graph, Tera template engine with filters), but **no generator actually consumes them**. FastAPI/Express/Axum generators are stubs with inline fallback. The pipeline is built but the last segment is disconnected.

## Current State

| Layer | Status |
|-------|--------|
| Mermaid+ schemas (8 diagram types) | 100% ✅ |
| SemanticType vocabulary (12 operations) | 100% ✅ |
| Schema → Validate → Generate | 100% ✅ |
| SpecIR (6 variants + metadata) | 100% ✅ |
| SpecBundle (multi-spec dependency graph) | 100% ✅ |
| Tera template engine + filters | 100% ✅ |
| **SpecIR → Code generators** | **0% ❌** |
| **SpecBundle consumption by generators** | **0% ❌** |
| **Template files (.j2/.tera)** | **0% ❌** |

## Proposal

### Phase 1: Single-section codegen (per SemanticType → code statement)

Priority by translatability:
1. `schema` (JSON Schema) → Rust struct / Python dataclass / TS interface
2. `db-model` (ErdPlus) → SQL DDL + ORM model
3. `rest-api` (OpenAPI) → route handler skeleton
4. `cli` (YAML) → clap derive struct
5. `state-machine` (StatePlus) → enum + transition match arms
6. `config` (JSON Schema) → config struct

### Phase 2: Cross-section composition (SpecBundle → integrated output)

Define composition rules: "route handler = rest-api × schema × logic × interaction"
- Generator receives SpecBundle
- Traverses dependency graph
- Resolves cross-refs between sections
- Produces integrated code unit

### Phase 3: Gradual cutover in implementation phase

Hybrid mechanism where:
- Section types with verified templates → auto-generate
- Section types without templates → fallback to agent
- Mixed output integrated into single implementation

## Acceptance Criteria

- [ ] At least one generator (Axum recommended) produces compilable code from SpecIR
- [ ] Template files exist and are loaded by Tera engine
- [ ] Template test suite: known spec input → known correct output
