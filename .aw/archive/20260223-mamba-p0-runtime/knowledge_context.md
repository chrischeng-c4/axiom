---
change_id: mamba-p0-runtime
type: knowledge_context
created_at: 2026-02-15T17:11:44.158681+00:00
updated_at: 2026-02-15T17:11:44.158681+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - 05-titan
  - 30-claude
  - 40-mcp
  - changelogs
  - genesis-325-329
  - grid
  - orbit
  - spec-to-code
  - index
---

# Knowledge Context

## Relevant Documents

- **orbit/bridge-internals.md**
  - summary: Orbit bridge internals — relevant for async runtime GIL release patterns that Mamba's async_rt.rs reuses. Documents waker-driven scheduling and GIL batching conventions.
  - relevant sections: GIL release protocol, Waker-driven scheduling
- **spec-to-code/code-generator-contract.md**
  - summary: Code generator contract — defines how specs map to runtime code. Relevant pattern for how new runtime functions should be registered and tested.
  - relevant sections: Runtime function registration

## Patterns

- **NaN-boxed MbValue** (source: crates/mamba/src/runtime/rc.rs)
  - All Mamba values are i64 at codegen level using NaN-boxing: TAG_INT=1, TAG_BOOL=2, TAG_NONE=3, TAG_PTR=0. Runtime functions accept and return i64 (MbValue). New methods must follow this convention.
- **Symbol Registration** (source: crates/mamba/src/runtime/symbols.rs)
  - Runtime functions are registered as MirExtern entries via runtime_externs(). New builtins/methods must be added here with correct MirType signatures for JIT/AOT linking.
- **ObjData Variants** (source: crates/mamba/src/runtime/rc.rs)
  - Heap objects use MbObject with ObjData enum variants: Str, List, Dict, Tuple, Class, Instance, Closure, Iterator, Generator, Exception, Module. New types (File, Set) need new variants.
- **extern C ABI** (source: crates/mamba/src/runtime/)
  - All runtime functions exposed to codegen use extern "C" fn(...) -> i64. Arguments are i64 MbValues. Functions must pack/unpack via mb_value_* helpers.

## Pitfalls

- VarAlloc::get first call declares Cranelift Variable type — subsequent calls return existing variable regardless of ty parameter. Using wrong type on first declaration causes verifier errors.
- mb_coroutine_get_local always returns i64 (NaN-boxed MbValue) regardless of declared parameter type. Using Ty::Float would cause Cranelift F64 VReg mismatch.
- File size limit: CLAUDE.md requires splitting files >= 1000 lines. hir_to_mir.rs is already 1515 lines and needs splitting.
- Thread-local storage (COROUTINES, TASKS, etc.) is not Send — runtime functions using thread_local! cannot be called from other threads.
- GC tracking: new container objects (lists, dicts created by methods) must be registered with the GC via gc::track() to avoid memory leaks from circular references.
