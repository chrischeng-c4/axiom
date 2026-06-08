---
change_id: taipan-295-297
type: codebase_context
created_at: 2026-02-13T07:23:27.473908+00:00
updated_at: 2026-02-13T07:23:27.473908+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - grep_search
  - read_file
---

# Codebase Context

## Analyzed Files

- **crates/cclab-taipan/src/codegen/cranelift/mod.rs** — Implementation of the Cranelift backend. Currently uses ObjectModule (AOT). Needs to support JITModule and replace placeholders for object operations with FFI calls.
  - symbols: `CraneliftBackend`, `compile_function`, `emit_inst`, `emit_extern_call`, `taipan_to_cl_type`
- **crates/cclab-taipan/src/driver/mod.rs** — Compiler driver that coordinates parsing, type checking, and codegen. Needs to handle JIT output and execution.
  - symbols: `CompilerSession`, `build`
- **crates/cclab-taipan/src/driver/config.rs** — Compiler configuration definitions. Needs new Backend variant for JIT.
  - symbols: `CompilerConfig`, `Backend`
- **crates/cclab-cli/src/taipan.rs** — CLI integration for Taipan commands. Needs to implement the 'run' command using the JIT backend.
  - symbols: `TaipanCli`, `execute`
- **crates/cclab-taipan/src/runtime/value.rs** — Defines TpValue (NaN-boxed 64-bit value). Critical for FFI boundary mapping.
  - symbols: `TpValue`
- **crates/cclab-taipan/src/runtime/mod.rs** — Entry point for runtime functions (tp_*). These need to be wired into the JIT symbol table.

## Prism Results

- **grep_search** (query: `grep tp_ in runtime/ffi`)
  - Identified over 100 tp_* runtime functions across multiple modules (list_ops, dict_ops, class, etc.).
- **grep_search** (query: `grep placeholder in codegen/cranelift/mod.rs`)
  - Found 10 placeholders in Cranelift backend for GetAttr, SetAttr, GetItem, SetItem, MakeList, MakeDict, MakeTuple, and Raise.

## Dependency Graph

- cclab-cli -> cclab-taipan::driver
- cclab-taipan::driver -> cclab-taipan::codegen
- cclab-taipan::codegen -> cclab-taipan::mir
- cclab-taipan::codegen -> cclab-taipan::types
- cclab-taipan::codegen::cranelift -> cranelift-codegen, cranelift-jit, cranelift-module
