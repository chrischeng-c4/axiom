---
change: sdd-codegen-and-fixes
group: specir-and-test-codegen
date: 2026-03-20
written_by: artifact_cli
review_verdict: REVISED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-sdd/README | overview | medium | Spec index, phase flow diagram, separation of interfaces and logic layers, link to generate/README.md |
| cclab-sdd/generate/codegen-system | codegen-architecture | high | Unified internal representation based on JSON Schema, Template-based generation engine (Tera/MiniJinja), Pluggable generators for different frameworks/targets, Pre-generation spec validation, Test generation support via TestGenerator component |
| cclab-sdd/generate/template-engine | generators | high | Tera initialization with recursive template discovery (.j2/.tera files), render(template_name, context) contract returning Result<String, TemplateError>, filter registration for case conversions, TemplateError variants (NotFound, ParseError, ContextMismatch). Directly required by Q2 (include_str! embedding) and wired into both Axum and FastAPI generators |
| cclab-sdd/generate/spec-ir-contract | spec-ir | high | SpecIR enum variants for all 6 spec types, SpecIR metadata (source file path, spec group, spec ID, tags), From<T> implementations from Generate types, Public API export from cclab-sdd lib.rs with Serialize+Deserialize for MCP transport, SpecBundle for multi-spec input (Vec<SpecIR> + dependency graph) — DEFERRED per Q3, Phase 2 only |
| cclab-sdd/generate/spec-ir-schema | spec-ir | high | Standard envelope structure (apiVersion, kind, metadata, spec), Kind registry (Api, FlowchartPlus, SequencePlus, ClassPlus, ErdPlus, RequirementPlus), Strict YAML serialization with unknown field rejection, Language-agnostic interface for code generators |
| cclab-sdd/generate/generator-axum | generators | high | Generator interface accepting Spec + output directory, Context transformation to Rust-specific types (i32, String, Option<T>, Vec<T>), Model generation with Serde derives, Router and handler generation with Axum config, Error handling with standardized GeneratorError |
| cclab-sdd/generate/generator-fastapi | generators | high | Input mapping from schema IR to FastAPI context, Template rendering with TemplateEngine, Standard FastAPI project layout generation, Type mapping to Pydantic (str, int, float, bool, List, BaseModel), Deterministic output ordering, structured error reporting |
| cclab-sdd/generate/test-generation | generators | high | Test artifact generation (fixtures, client helpers, test cases) for FastAPI, Express, Axum — in scope per Q5. R2 probe adapter — DEFERRED per Q5; deliverable is file scaffold generation only, no probe integration. Deterministic test file naming and content ordering, Structured error reporting (generator type, template name, human-readable cause) |
| cclab-sdd/generate/requirement-plus-enhancement | diagram-types | high | SysML v1.6 type support (functionalRequirement, interfaceRequirement, performanceRequirement, etc.), Risk levels (Low, Medium, High) and verification methods (Analysis, Inspection, Test, Demonstration), Relationship types (satisfies, verifies, refines, traces, contains, copies, derives), YAML frontmatter validation for all new types |
| cclab-sdd/generate/code-generator-contract | codegen-architecture | high | Generator responsibilities (framework-specific code from agnostic specs), Spec-to-code mappings (API schema → Models, Sequence+ → Function signatures, Requirement+ → Tests), Inference rules (auto-infer patterns from spec semantics), SequencePlus macro/micro level code generation, RequirementPlus N:M test generation (N Requirements → N Test Classes, N Scenarios → N Functions) |
| cclab-sdd/generate/README | overview | medium | Code generation and template library organization, Spec IR contract and schema specifications, Code generator contract definition, Generator implementations (Axum, FastAPI, Express), Integration architecture |

# Reviews

## Review: Revision (Iteration 2)

**Change ID**: sdd-codegen-and-fixes

**Verdict**: REVISED

### Summary

Addressed 5 of 7 deficiencies from Iteration 1:
- ✅ **[HIGH]** Added cclab-sdd/generate/template-engine.md (required by both Axum and FastAPI, directly addresses Q2)
- ✅ **[HIGH]** Removed cclab-sdd/sdd-cli (irrelevant to #932/#933; covers workflow CLI, not codegen)
- ✅ **[MEDIUM]** Flagged R2 probe adapter as DEFERRED per Q5 in test-generation key requirements
- ✅ **[MEDIUM]** Updated spec-ir-contract R5 to show SpecBundle scope is Phase 2, deferred per Q3
- ✅ **[LOW]** Downgraded cclab-sdd/README to MEDIUM and assigned logical group names to all specs

Remaining gaps require Phase 7 (change_impl) follow-up:
- **[MEDIUM]** BDD given/when/then field mapping (code-generator-contract mentions test N:M mapping at architectural level but lacks explicit BDD field → function body rules). Add to code-generator-contract as enhancement or create separate spec_plan entry for Phase 7 to formalize BDD field semantics.

### Checklist

- ✅ All affected crates/areas from pre-clarifications are covered by at least one spec
  - template-engine.md now included (covers Tera init, render contract, filters, error handling)
  - Test generation artifacts now explicitly scoped (in-scope per Q5, probe adapter deferred)
  - Codegen contract covers Requirement+ → Test N:M mapping at architectural level (BDD details deferred to Phase 7)

- ✅ Relevance scores are now accurate
  - Removed sdd-cli (HIGH was incorrect; workflow CLI unrelated to code generation from SpecIR)
  - Downgraded README to MEDIUM (background/orientation, not implementation-critical)
  - All generator and core codegen specs remain HIGH (directly implement #932/#933)

- ✅ Key requirements are accurate and scoped
  - spec-ir-contract: "SpecBundle for multi-spec input — DEFERRED per Q3, Phase 2 only" (corrected from "Serialization support")
  - test-generation: "R2 probe adapter — DEFERRED per Q5; deliverable is file scaffold only" (flagged deferral per Q5)
  - template-engine: R1-R4 requirements match spec (Tera init, render, filters, error handling), directly supports Q2 (include_str! embedding)

- ✅ No irrelevant specs included
  - Removed sdd-cli (covered SDD workflow phase CLI, not code generation from SpecIR)
  - All remaining specs directly bear on #932 (consuming SpecIR in generators) or #933 (test scaffolding from RequirementPlus)

- ✅ Logical groups assigned
  - **generators**: template-engine, generator-axum, generator-fastapi, test-generation
  - **codegen-architecture**: codegen-system, code-generator-contract
  - **spec-ir**: spec-ir-contract, spec-ir-schema
  - **diagram-types**: requirement-plus-enhancement
  - **overview**: README, generate/README

### Outstanding Items (Phase 7)

- Formalize BDD field mapping (given/when/then → // Given / // When / assert!(...) stubs) in code-generator-contract.md or create new spec for RequirementPlus test codegen
- Add coverage validation logic (warn when requirements have no Verifies relationship) to either codegen contract or test-generation spec
