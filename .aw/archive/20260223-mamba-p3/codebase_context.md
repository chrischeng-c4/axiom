---
change_id: mamba-p3
type: codebase_context
created_at: 2026-02-23T01:10:07.002965+00:00
updated_at: 2026-02-23T01:10:07.002965+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - "prism_symbols(file=crates/mamba/src/runtime/stdlib/mod.rs): Found register_stdlib, register_module, 5 base modules (sys/os/math/json/time) plus 25 P2 modules. P3 adds 16+ new modules."
---

# Codebase Context

## Analyzed Files

- **crates/mamba/src/runtime/stdlib/mod.rs** — stdlib_registry
  - symbols: `register_stdlib`, `register_module`
- **crates/mamba/src/runtime/symbols.rs** — jit_symbols
  - symbols: `rt_sym!`, `RUNTIME_SYMBOLS`
- **crates/mamba/src/runtime/rc.rs** — object_model
  - symbols: `ObjData`, `MbObject`
- **crates/mamba/src/runtime/builtins.rs** — builtins
  - symbols: `mb_print`, `mb_len`
- **crates/mamba/src/runtime/class.rs** — class_dispatch
  - symbols: `mb_getattr`, `mb_setattr`
- **crates/mamba/src/runtime/string_ops.rs** — string_formatting
  - symbols: `value_to_string`
- **crates/mamba/src/runtime/gc.rs** — gc
  - symbols: `gc_track`
- **crates/mamba/src/runtime/exception.rs** — exceptions
  - symbols: `mb_raise`
- **crates/mamba/Cargo.toml** — dependencies
