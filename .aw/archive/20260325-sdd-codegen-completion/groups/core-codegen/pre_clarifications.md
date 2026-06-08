---
change: sdd-codegen-completion
group: core-codegen
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: First generator target for #932
- **Answer**: Both schema (JSON Schema → Rust struct) and rest-api (OpenAPI → Axum route handler skeleton) for Axum target. FastAPI also in scope.

### Q2: General
- **Question**: Template file embedding vs filesystem
- **Answer**: Embedded via include_str! in crates/cclab-sdd/templates/. No runtime file dependency.

### Q3: General
- **Question**: SpecBundle cross-section composition scope
- **Answer**: Phase 1 only (single-section codegen). Phase 2 SpecBundle cross-section composition deferred.

### Q4: General
- **Question**: cclab-probe integration for #933
- **Answer**: Standalone test files only. No cclab-probe integration. Probe is deferred.

