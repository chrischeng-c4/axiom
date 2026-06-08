---
change_id: phase1
type: gap_codebase_spec
created_at: 2026-02-12T17:56:58.534967+00:00
updated_at: 2026-02-12T17:56:58.534967+00:00
---

# Gap Analysis: Codebase vs Specs

## Code without matching spec (high severity)

1. **hir/ module (empty)** — `src/hir/mod.rs` declared but empty. No spec defines HIR structure. Severity: HIGH
2. **lower/ module (missing)** — No `src/lower/` directory exists. No spec for AST→HIR or HIR→MIR lowering. Severity: HIGH
3. **runtime/ module (missing)** — No `src/runtime/` directory exists. No spec for object model, refcounting, or built-in implementations. Severity: HIGH
4. **resolve pass (missing)** — `src/resolve/scope.rs` defines data structures but no AST traversal pass. No spec. Severity: HIGH
5. **driver pipeline (incomplete)** — `src/driver/mod.rs` exists but has no end-to-end compilation pipeline. No spec. Severity: HIGH

## Code with partial spec coverage (medium severity)

6. **codegen/cranelift** — Backend works for hand-constructed MIR but untested with lowering output. Severity: MEDIUM
7. **types/builtins.rs** — Type stubs for 40+ builtins exist but no runtime implementations. Severity: MEDIUM

## Specs without matching code (low severity)

8. **cclab-core/02-architecture-principles** — General principles, not taipan-specific. Severity: LOW

## Summary

5 HIGH gaps represent the core missing pipeline: HIR, lowering, runtime, resolve pass, and driver. These are exactly what Phase 1 (#275-#282) addresses. No specs exist for any of these — they are defined by GitHub issues only."
