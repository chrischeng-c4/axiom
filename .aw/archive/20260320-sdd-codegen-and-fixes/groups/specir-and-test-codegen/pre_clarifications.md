---
change: sdd-codegen-and-fixes
group: specir-and-test-codegen
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: First codegen target for #932
- **Answer**: Both Axum (Rust) and FastAPI (Python) as first targets.

### Q2: General
- **Question**: Template file location
- **Answer**: Templates live in crates/cclab-sdd/templates/ directory (existing template location). Loaded via include_str! for embedding into binary. No runtime file dependency.

### Q3: General
- **Question**: SpecBundle cross-section composition scope
- **Answer**: Phase 2 (SpecBundle cross-section composition) deferred. Only Phase 1 (per-section-type, single-spec codegen) in scope.

### Q4: General
- **Question**: Test coverage validation mode
- **Answer**: Warning mode — missing coverage prints to stderr and continues. No --strict flag needed for initial implementation.

### Q5: General
- **Question**: cclab-probe integration scope
- **Answer**: Deferred. Deliverable is file scaffold generation only, no probe integration.

