---
change: sdd-codegen-and-fixes
group: specir-and-test-codegen
date: 2026-03-20
---

# Requirements

Implement two code generators for existing spec schemas in the generate subsystem. (1) SpecIR last-mile codegen (#932): the full pipeline exists (SpecIR 6 variants, SemanticType 12 ops, SpecBundle dependency graph, Tera template engine with filters) but no generator actually consumes SpecIR. Implement at least one target generator (Axum recommended) with real .j2/.tera template files. Priority spec types: schema (JSON Schema → Rust struct), rest-api (OpenAPI → route handler skeleton), state-machine (StatePlus → enum + match arms), db-model (ErdPlus → SQL DDL). Template test suite: known spec input → known correct output. Phase 2 (SpecBundle cross-section composition) optional for this change. (2) RequirementPlus test scaffold (#933): RequirementPlus already has BDD fields (given/when/then, test_type: unit/integration/e2e, verification) and requirement-element relationships (Satisfies/Verifies/Refines/Traces), but no generator produces test files. Implement a generator that maps ElementDef → test function skeleton with BDD structure (// Given / // When / // Then, assertion stubs derived from `then` field, traceability comment linking to requirement ID). Rust #[test] as the first target. Add coverage validation: parse RequirementPlus spec, check every Requirement has at least one Verifies → Element relationship, report uncovered requirements. Both generators share the same Tera-based template infrastructure and live in the generate subsystem of crate:sdd.
