---
change_id: mamba-p0-runtime
type: gap_codebase_knowledge
created_at: 2026-02-15T17:22:05.935528+00:00
updated_at: 2026-02-15T17:22:05.935528+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention Violations

### HIGH severity

1. **hir_to_mir.rs exceeds file size limit** — Knowledge pitfall notes CLAUDE.md requires splitting files >= 1000 lines. `hir_to_mir.rs` is 1515 lines and has not been split. Adding method dispatch lowering (#380) will grow it further.
   - file: `crates/mamba/src/runtime/../lower/hir_to_mir.rs`
   - knowledge ref: Pitfalls — File size limit

2. **No GC tracking on list/dict creation** — Knowledge pitfall states new container objects must be registered with `gc::track()`. Current `builtins.rs` creates list objects (mb_range) without GC registration. New list/dict methods (#376, #377) creating containers will compound this gap.
   - file: `crates/mamba/src/runtime/builtins.rs`
   - knowledge ref: Pitfalls — GC tracking

### MEDIUM severity

3. **Missing symbol registration for existing string_ops** — Knowledge pattern "Symbol Registration" requires all runtime functions to be registered as MirExtern entries. `string_ops.rs` has 6 functions but `symbols.rs` registration may be incomplete — some operations are inlined by codegen rather than registered.
   - file: `crates/mamba/src/runtime/symbols.rs`
   - knowledge ref: Patterns — Symbol Registration

4. **exception.rs does not follow ObjData pattern** — Knowledge pattern "ObjData Variants" states heap objects use MbObject with ObjData enum. Current `MbException` is a standalone struct stored in thread-local, not an ObjData variant. Exception hierarchy (#381) needs exceptions to be proper ObjData::Exception variants for isinstance() support.
   - file: `crates/mamba/src/runtime/exception.rs`
   - knowledge ref: Patterns — ObjData Variants

## Pattern Mismatches

### HIGH severity

5. **No type-tagged method dispatch** — Knowledge pattern "NaN-boxed MbValue" + "extern C ABI" establishes that all runtime functions take/return i64. However, method calls (str.split, list.append) need a dispatch layer that: (a) extracts the ObjData tag from the receiver, (b) looks up the method name, (c) calls the corresponding Rust function. No such dispatch exists.
   - file: `crates/mamba/src/runtime/class.rs` (mb_getattr only does dict lookup)
   - knowledge ref: Patterns — NaN-boxed MbValue, ObjData Variants

### MEDIUM severity

6. **Thread-local exception state vs class-based exceptions** — Knowledge pitfall notes thread_local! storage is not Send. Current MbException uses thread-local storage for the "current exception". Exception hierarchy (#381) introduces class-based exceptions (ValueError inherits Exception inherits BaseException) which need both the class hierarchy AND thread-local storage to coexist.
   - file: `crates/mamba/src/runtime/exception.rs`
   - knowledge ref: Pitfalls — Thread-local storage

## Summary

| Category | HIGH | MEDIUM | LOW |
|----------|------|--------|-----|
| Convention violations | 2 | 2 | 0 |
| Pattern mismatches | 1 | 1 | 0 |
| **Total** | **3** | **3** | **0** |