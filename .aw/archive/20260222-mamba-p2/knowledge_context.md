---
change_id: mamba-p2
type: knowledge_context
created_at: 2026-02-22T10:59:33.506008+00:00
updated_at: 2026-02-22T10:59:33.506008+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - cclab-mamba
  - spec-to-code
---

# Knowledge Context

## Relevant Documents

- **spec-to-code/code-generator-contract.md**
  - summary: General code generation contract. Not directly relevant to mamba P2 but establishes conventions.

## Patterns

- **stdlib-module-pattern** (source: crates/mamba/src/runtime/stdlib/)
  - Each stdlib module is a separate _mod.rs file with pub functions taking/returning MbValue. Registered via register_stdlib() in mod.rs.
- **symbol-registration-pattern** (source: crates/mamba/src/runtime/symbols.rs)
  - rt_sym! macro registers runtime functions for JIT with name, fn pointer, param types, return type.
- **nan-boxed-value-convention** (source: crates/mamba/src/runtime/value.rs)
  - MbValue(u64) NaN-boxed: ints/floats/bools/none inline, heap objects as pointers.
- **objdata-variant-pattern** (source: crates/mamba/src/runtime/rc.rs)
  - New heap types require ObjData variant, ObjKind enum, MbObject constructor, match arms in ~7 files.

## Pitfalls

- Adding ObjData variants requires updating non-exhaustive match statements in builtins, iter, gc, string_ops, json_mod, class dispatch.
- CLASS_REGISTRY is thread_local RefCell not Mutex — use .with() pattern.
- File limit: no file > 1000 lines, consider split at 500+. class.rs already ~1179 lines.
