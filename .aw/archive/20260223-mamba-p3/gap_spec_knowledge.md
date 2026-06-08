---
change_id: mamba-p3
type: gap_spec_knowledge
created_at: 2026-02-23T01:13:38.101274+00:00
updated_at: 2026-02-23T01:13:38.101274+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec Responsibilities vs Knowledge Architecture
1. **mamba-stdlib-core spec covers only 4 modules** (medium) — The spec defines sys/os/math/json but the stdlib-module-pattern in knowledge covers 30+ modules. The spec is outdated relative to actual scope.
2. **mamba-jit-backend R2 (symbol wiring) vs actual pattern** (low) — Spec describes symbol wiring abstractly. Knowledge documents concrete rt_sym! macro pattern. No contradiction, but spec lacks specificity.

## Knowledge Patterns Not in Specs
1. **ObjData variant addition pattern** (medium) — Knowledge documents the ~7 file update pattern for new ObjData variants, but no spec covers this. Complex number (#453) needs this pattern.
2. **Thread-local registry pattern** (medium) — Knowledge documents thread_local! with RefCell<HashMap> pattern, but no spec covers threading implications for #417.
3. **External crate wrapping** (low) — Knowledge notes this is a new pattern for P3. No existing spec covers external Rust crate integration.

## Responsibility Boundary Misalignments
None identified. The mamba-stdlib-core and mamba-import-system specs have clear boundaries. New P3 modules fit within existing architectural boundaries.

## Summary
No contradictions between specs and knowledge. Main gaps: stdlib-core spec is narrow (4 modules vs 30+ actual), and several implementation patterns documented in knowledge have no spec coverage. These are documentation gaps, not architectural issues.