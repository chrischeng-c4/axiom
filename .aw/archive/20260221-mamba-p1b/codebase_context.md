---
change_id: mamba-p1b
type: codebase_context
created_at: 2026-02-21T14:51:33.417055+00:00
updated_at: 2026-02-21T14:51:33.417055+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_references
---

# Codebase Context

## Analyzed Files

- **crates/mamba/src/runtime/rc.rs**
  - symbols: `MbObjectHeader`, `ObjKind`, `MbObject`, `ObjData`, `mb_retain`, `mb_release`
- **crates/mamba/src/runtime/value.rs**
  - symbols: `MbValue`, `NAN_PREFIX`, `TAG_PTR`, `TAG_INT`, `TAG_BOOL`, `TAG_NONE`
- **crates/mamba/src/runtime/class.rs**
  - symbols: `MbClass`, `mb_class_register`, `mb_getattr`, `mb_setattr`, `mb_call_method`, `mb_super`, `mb_super_getattr`, `mb_dispatch_binop`, `mb_dispatch_unaryop`
- **crates/mamba/src/runtime/module.rs**
  - symbols: `MbModule`, `mb_import`, `mb_import_from`
- **crates/mamba/src/codegen/cranelift/mod.rs**
  - symbols: `CraneliftBackend`, `VarAlloc`, `emit_inst`, `emit_binop`, `emit_terminator`
- **crates/mamba/src/codegen/cranelift/jit.rs**
  - symbols: `CraneliftJitBackend`, `emit_inst`
- **crates/mamba/src/mir/mod.rs**
  - symbols: `MirInst`, `MirBinOp`, `MirUnaryOp`, `MirConst`
- **crates/mamba/src/runtime/symbols.rs**
  - symbols: `runtime_symbols`, `runtime_externs`
