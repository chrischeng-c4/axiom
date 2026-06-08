---
change_id: mamba-p0-runtime
type: gap_spec_knowledge
created_at: 2026-02-15T17:22:54.198424+00:00
updated_at: 2026-02-15T17:22:54.198424+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec Responsibilities Contradicting Knowledge Architecture

### HIGH severity

1. **mamba-oop-model R3 (Magic Method Dispatch) vs NaN-boxing pattern** — Spec assumes dunder methods (__add__, __str__) are dispatched via MRO class attribute lookup. Knowledge pattern documents NaN-boxed i64 values where TAG_INT/TAG_BOOL/TAG_NONE values have no ObjData backing. Integer __add__ cannot go through MRO — it must be handled inline or via type-tag dispatch before falling through to ObjData-based resolution.
   - spec ref: mamba-oop-model R3
   - knowledge ref: Patterns — NaN-boxed MbValue

2. **mamba-stdlib-core (file I/O) vs ObjData Variants** — Spec R1-R4 covers sys/os/math/json but the knowledge pattern documents ObjData as the only heap object mechanism. File I/O (#379) needs an ObjData::File variant, but no spec defines file object lifecycle (open/read/write/close/flush) or how file handles integrate with the GC.
   - spec ref: mamba-stdlib-core
   - knowledge ref: Patterns — ObjData Variants, Pitfalls — GC tracking

### MEDIUM severity

3. **mamba-iteration-protocol R3 vs extern C ABI** — Spec defines iterator protocol via __iter__/__next__ dunder methods. Knowledge pattern requires all runtime functions to use extern C fn(...) -> i64. Iterator protocol needs a sentinel value for StopIteration — spec doesn't address how StopIteration maps to the NaN-boxed i64 return value convention.
   - spec ref: mamba-iteration-protocol R3
   - knowledge ref: Patterns — extern C ABI, NaN-boxed MbValue

4. **mamba-gc-runtime R1 vs Symbol Registration** — Spec requires GC tracking of container objects. Knowledge pattern requires symbol registration for runtime functions. gc::track() is not registered as a symbol — it's called internally. The boundary between codegen-visible symbols and internal runtime functions is not defined in either spec or knowledge.
   - spec ref: mamba-gc-runtime R1
   - knowledge ref: Patterns — Symbol Registration

## Knowledge Patterns Not Reflected in Any Spec

### MEDIUM severity

5. **VarAlloc first-call semantics** — Knowledge pitfall documents that VarAlloc::get first call declares Variable type and subsequent calls return existing. No spec covers codegen variable allocation strategy or its implications for method dispatch lowering.
   - knowledge ref: Pitfalls — VarAlloc

6. **File size limit convention** — Knowledge pitfall notes 1000-line max per file. No spec acknowledges this constraint or plans module splitting for large files like hir_to_mir.rs.
   - knowledge ref: Pitfalls — File size limit

## Responsibility Boundary Misalignments

### HIGH severity

7. **Method dispatch ownership** — mamba-oop-model owns class-based dispatch (MRO), mamba-string-runtime owns string operations, mamba-iteration-protocol owns iterator protocol. But built-in type method dispatch (str.split → string_ops::mb_string_split) crosses all three specs. No single spec owns the dispatch table that maps (type_tag, method_name) → runtime function.
   - spec ref: mamba-oop-model, mamba-string-runtime, mamba-iteration-protocol
   - knowledge ref: Patterns — ObjData Variants

## Summary

| Category | HIGH | MEDIUM | LOW |
|----------|------|--------|-----|
| Spec vs knowledge contradictions | 2 | 2 | 0 |
| Knowledge patterns without spec | 0 | 2 | 0 |
| Boundary misalignments | 1 | 0 | 0 |
| **Total** | **3** | **4** | **0** |