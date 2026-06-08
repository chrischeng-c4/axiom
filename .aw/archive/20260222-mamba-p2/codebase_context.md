---
change_id: mamba-p2
type: codebase_context
created_at: 2026-02-22T11:00:26.496782+00:00
updated_at: 2026-02-22T11:00:26.496782+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
---

# Codebase Context

## Analyzed Files

- **crates/mamba/src/runtime/stdlib/mod.rs** — stdlib registry
  - symbols: `register_stdlib`
- **crates/mamba/src/runtime/symbols.rs** — JIT symbol registry
  - symbols: `register_symbols`, `rt_sym!`
- **crates/mamba/src/runtime/rc.rs** — object representation
  - symbols: `ObjData`, `ObjKind`, `MbObject`
- **crates/mamba/src/runtime/class.rs** — OOP runtime
  - symbols: `MbClass`, `CLASS_REGISTRY`, `mb_getattr`
- **crates/mamba/src/runtime/gc.rs** — GC
  - symbols: `mark_object`
- **crates/mamba/src/runtime/iter.rs** — iterators
  - symbols: `mb_iter`, `mb_next`
- **crates/mamba/src/runtime/module.rs** — import system
  - symbols: `mb_module_register`, `mb_import`
- **crates/mamba/src/runtime/set_ops.rs** — set operations
  - symbols: `mb_set_*`
- **crates/mamba/src/runtime/bytes_ops.rs** — bytes ops
  - symbols: `mb_bytes_*`
- **crates/mamba/src/lower/hir_to_mir.rs** — HIR to MIR
  - symbols: `lower_stmt`, `lower_expr`
- **crates/mamba/src/runtime/exception.rs** — exceptions
  - symbols: `mb_raise`
