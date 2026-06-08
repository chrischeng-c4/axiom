---
change: sdd-codegen-testgen
group: specir-codegen
date: 2026-03-19
written_by: claude
review_verdict: REVISED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-sdd/README.md | workflow-overview | low | SDD workflow phases and state machine, Phase flow from Init → Restructure → PreClarify → RefContext → PostClarify → Spec → Impl → Merge. Background reference for SDD framework understanding. |
| cclab-sdd/logic/change-spec.md | spec-lifecycle | low | Change spec lifecycle and per-spec CRR cycles. Background understanding of how SDD specs are structured and filled, but not actionable for codegen implementation. |
| cclab-sdd/logic/implement-task.md | implementation-logic | medium | Codegen routing logic for specs with json_schema or api_spec, prism_generate_from_spec integration point. Spec execution order via Kahn's algorithm (topological sort). |
| cclab-probe/README.md | testing-framework | medium | Test framework for validating generated code, cclab probe CLI for test discovery and execution, Coverage collection and reporting. Integration point for testing generated Python code. |
| cclab-probe/00-architecture.md | testing-framework | medium | Heavy Rust, Thin Python architecture pattern, Rust CLI module registration via CliModule + linkme, pyo3 bindings and embed layer for Python execution. Architecture pattern applicable to code generation system. |
| cclab-probe/10-components.md | testing-framework | medium | Component responsibilities and integration patterns, PyO3 bridge layer for Rust-Python communication, File discovery and registry patterns, Report generation and formatting architecture. |
| cclab-probe/20-data-flows.md | testing-framework | low | Test discovery and execution flow patterns, Error handling in orchestration, Performance optimization techniques. |
| cclab-probe/40-state-machines.md | testing-framework | low | State machine patterns for test execution lifecycle, Error recovery and state transitions, Test execution orchestration. |
| cclab-sdd/generate/generator-fastapi.md | codegen-generators | high | FastAPI code generation algorithm and patterns. Q1 specifies Python/FastAPI as target framework. Defines request/response model generation, route handler synthesis, middleware integration, and validation logic for FastAPI output. |
| cclab-sdd/generate/codegen-system.md | codegen-architecture | high | Overall code generation system architecture and component interaction. Pipeline orchestration, spec-to-code routing logic, and integration points for language-specific generators. |
| cclab-sdd/generate/spec-ir-contract.md | codegen-input | high | SpecIR contract definition and type hierarchy. SpecIR is the foundational intermediate representation (group name: specir-codegen). Defines API spec, schema, sequence, flowchart, and requirement+ types that generators consume. |
| cclab-sdd/generate/template-engine.md | codegen-rendering | high | Tera template engine integration for code rendering. Q2 specifies 'Filesystem. Templates loaded from configurable path, enables hot-reload and customization.' Maps to TemplateEngine::new(template_dir) and render pipeline. |
| cclab-sdd/generate/test-generation.md | codegen-testing | high | Test generation integration with cclab-probe. User input: 'RequirementPlus test generation'. Defines how generated tests are packaged into probe-compatible suites and execution contract. |
| cclab-sdd/generate/code-generator-contract.md | codegen-contract | high | Code generator interface and cross-section composition rules. Q3: 'Phase 2 is in scope' (cross-section composition, e.g., route handler = rest-api × schema). Defines how generators synthesize code from multiple spec sections. |
| cclab-sdd/generate/spec-ir-evaluation.md | codegen-feasibility | medium | SpecIR feasibility and codegen viability assessment. Background for understanding which spec patterns are codegen-feasible and which require manual implementation or special handling. |

# Spec Plan

## FastAPI Pipeline Code Generation
- **Action**: create
- **Main Spec Reference**: cclab-sdd/generate/codegen-system.md
- **Sections**: overview, logic, schema, test-plan, changes

## Filesystem Template Engine Configuration
- **Action**: modify
- **Source**: cclab-sdd/generate/template-engine.md
- **Sections**: overview, config, changes

## Test Generation for Probe Integration
- **Action**: modify
- **Source**: cclab-sdd/generate/test-generation.md
- **Sections**: overview, logic, test-plan, changes

# Reviews

## Review: artifact revision (Iteration 1)

**Change ID**: sdd-codegen-testgen

**Verdict**: RESOLVED

### Issues Addressed

**[HIGH] Coverage gap: entire cclab-sdd/generate/ subsystem missing** ✅
- **Fix Applied**: Added 6 high-relevance specs from cclab-sdd/generate/:
  1. generator-fastapi.md — FastAPI target (Q1 requirement)
  2. codegen-system.md — System architecture
  3. spec-ir-contract.md — SpecIR foundational input (group name: specir-codegen)
  4. template-engine.md — Filesystem template loading (Q2 requirement)
  5. test-generation.md — RequirementPlus test generation
  6. code-generator-contract.md — Cross-section composition (Q3 Phase 2 requirement)
- **Verification**: All 6 directly-relevant specs now included at HIGH relevance. Scope from pre-clarifications fully covered.

**[HIGH] spec_plan array entirely absent** ✅
- **Fix Applied**: Added spec_plan section with 3 entries:
  1. FastAPI Pipeline Code Generation (create) → main_spec_ref: codegen-system.md → sections: [overview, logic, schema, test-plan, changes]
  2. Filesystem Template Engine Configuration (modify) → source: template-engine.md → sections: [overview, config, changes]
  3. Test Generation for Probe Integration (modify) → source: test-generation.md → sections: [overview, logic, test-plan, changes]
- **Verification**: spec_plan now declares which change specs will be created and their section structure. System can prepare spec files for change_spec phase.

**[HIGH] Two false-positive SDD-internal specs at HIGH relevance** ✅
- **Fix Applied**: Removed both specs:
  1. cclab-sdd/logic/reference-context.md — workflow meta (CRR cycle, artifact writing)
  2. cclab-sdd/sdd-cli.md — workflow CLI (SDD workflow subcommands)
- **Verification**: These specs were meta-workflow, not codegen. Removal improves signal-to-noise for codegen task.

**[MEDIUM] Three specs over-rated for codegen task** ✅
- **Fix Applied**: Downrated:
  1. cclab-sdd/README.md — HIGH → LOW (phase flow background only)
  2. cclab-sdd/logic/change-spec.md — HIGH → LOW (SDD structure, not codegen guidance)
  3. cclab-sdd/logic/implement-task.md — HIGH → MEDIUM (prism_generate_from_spec only codegen touch point)
- **Verification**: Relevance ratings now correctly reflect codegen task priorities.

**[MEDIUM] Missing test-generation integration picture** ✅
- **Fix Applied**: Added cclab-sdd/generate/test-generation.md at HIGH relevance. Already included spec-ir-evaluation.md at MEDIUM for feasibility background.
- **Verification**: Integration contract between codegen and cclab-probe now complete. Probe specs remain at appropriate levels (framework understanding).

### Scope Re-Check (Pre-Clarifications Confirmed)

- ✅ **Q1 (FastAPI/Python)**: generator-fastapi.md (HIGH), codegen-system.md (HIGH) cover target language decision
- ✅ **Q2 (Filesystem Templates)**: template-engine.md (HIGH), spec-ir-evaluation.md (MEDIUM) cover template loading and flexibility
- ✅ **Q3 (Phase 2 Scope)**: code-generator-contract.md (HIGH) covers cross-section composition rules (route handler = rest-api × schema)
- ✅ **Q4 (Output Structure)**: spec-ir-contract.md (HIGH), code-generator-contract.md (HIGH) cover section-to-file mapping
- ✅ **Testing**: test-generation.md (HIGH), cclab-probe/* (MEDIUM) cover full test pipeline
