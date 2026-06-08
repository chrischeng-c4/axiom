---
change: phase1
date: 2026-02-13
---

# Clarifications

## Q1: Git Workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place — work on current feat/taipan branch
- **Rationale**: Continuing work on the same branch where all taipan-all changes were made.

## Q2: Scope
- **Question**: Which issues to implement?
- **Answer**: #275-#282 (Phase 1 P0 core pipeline)
- **Rationale**: These 8 issues complete the end-to-end compiler pipeline from .py source to executable.

## Q3: Object Model
- **Question**: NaN-boxing vs tagged pointer for TpValue?
- **Answer**: NaN-boxing for compact 64-bit representation
- **Rationale**: NaN-boxing fits in a single i64, aligns with Cranelift's I64 representation, and is proven by LuaJIT/SpiderMonkey.

## Q4: GC Strategy
- **Question**: Reference counting strategy?
- **Answer**: Reference counting with deferred cycle collection
- **Rationale**: RC is deterministic and simple. Cycle collector runs periodically for reference cycles. Matches CPython's approach.

## Q5: Exception Mechanism
- **Question**: setjmp/longjmp vs DWARF unwinding for exceptions?
- **Answer**: setjmp/longjmp for Phase 1 (simpler), DWARF in future
- **Rationale**: setjmp/longjmp is portable and easier to emit from Cranelift. Can migrate to DWARF unwinding later for performance.

