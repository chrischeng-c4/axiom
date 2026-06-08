---
change_id: mamba-p3
type: knowledge_context
created_at: 2026-02-23T01:08:33.940195+00:00
updated_at: 2026-02-23T01:08:33.940195+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - cclab-mamba
  - spec-to-code
  - changelogs
---

# Knowledge Context

## Relevant Documents

- **spec-to-code/code-generator-contract.md**
  - summary: Code generator contract patterns. Not directly relevant but informs how JIT symbols should be wired.
- **spec-to-code/spec-model.md**
  - summary: Spec model structure. Relevant for how P3 specs should be structured.

## Patterns

- **stdlib-module-pattern** (source: crates/mamba/src/runtime/stdlib/)
  - Each stdlib module: {name}_mod.rs with register() fn returning HashMap<String, MbValue>. Wired via pub mod + register() call in stdlib/mod.rs.
- **nan-boxed-value** (source: crates/mamba/src/runtime/value.rs)
  - MbValue(u64) NaN-boxing. None=0, True=2, False=4. Heap objects via pointer. Methods: as_i64(), as_f64(), as_bool(), as_ptr(), from_ptr().
- **rt-sym-registration** (source: crates/mamba/src/runtime/symbols.rs)
  - rt_sym!(name, fn_ptr, [param_types], ret_type) macro registers runtime functions for JIT symbol resolution.
- **objdata-variant-addition** (source: crates/mamba/src/runtime/rc.rs)
  - Adding new ObjData variant requires updating match arms in ~7 files: string_ops.rs, class.rs, json_mod.rs, gc.rs, iter.rs, builtins.rs, rc.rs.
- **thread-local-registry** (source: crates/mamba/src/runtime/class.rs)
  - Class/slots registries use thread_local! with RefCell<HashMap>. New registries for threading must account for cross-thread access.

## Pitfalls

- Adding ObjData variants causes match exhaustiveness errors in ~7 files - must update all simultaneously
- Thread-local state is not shared between threads - threading module needs careful design
- External crate dependencies (rusqlite, flate2) increase binary size and compile time
- eval/exec requires re-entering the parser pipeline at runtime - potential circular dependency
