---
change: sdd-codegen-completion
group: core-codegen
date: 2026-03-20
---

# Requirements

Implement the SpecIR → code generation pipeline by wiring the existing Tera template engine to actual .tera template files for backend targets (Axum required, FastAPI optional). At least one single-section generator must produce compilable Rust from a SpecIR input. A template test suite is required: known SpecIR input → known correct output. Additionally, add a RequirementPlus → test file scaffold generator: BDD fields (given/when/then, test_type) map to #[test] function skeletons with assertion stubs and traceability comments. A coverage validation report must flag requirements that have no Verifies relationship to any Element. SpecBundle cross-section composition (Phase 2) and cclab-probe integration are deferred unless confirmed in-scope. Hybrid cutover (Phase 3) — templates for verified section types, agent fallback for unverified — is in scope as part of the implementation phase integration.
