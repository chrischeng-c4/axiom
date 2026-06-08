---
change_id: mamba-p0-runtime
type: codebase_context
created_at: 2026-02-15T17:13:12.542705+00:00
updated_at: 2026-02-15T17:13:12.542705+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_references
---

# Codebase Context

## Analyzed Files

- **crates/mamba/src/runtime/builtins.rs** — Core builtins (print, len, int, float, str, bool, type, abs, range). Extend with enumerate, zip, min, max, sum, sorted, reversed, isinstance, input.
  - symbols: `mb_print`, `mb_len`, `mb_int`, `mb_float`, `mb_str`, `mb_bool`, `mb_type`, `mb_abs`, `mb_range`
- **crates/mamba/src/runtime/string_ops.rs** — String operations (concat, repeat, index, slice, format). Extend with split, join, strip, replace, find, startswith, endswith, upper, lower, count.
  - symbols: `mb_string_concat`, `mb_string_repeat`, `mb_string_getitem`, `mb_string_slice`, `mb_string_format`, `mb_string_len`
- **crates/mamba/src/runtime/rc.rs** — Core value types: MbObject, ObjData enum (Str, List, Dict, Tuple, Class, Instance, Closure, Iterator, Generator, Exception, Module). Need new variants for File, Set. Method dispatch reads ObjData variant.
  - symbols: `MbObject`, `ObjData`, `mb_value_tag`, `mb_value_int`, `mb_value_float`, `mb_value_ptr`, `mb_obj_alloc`, `mb_obj_dealloc`
- **crates/mamba/src/runtime/class.rs** — Class/instance creation, C3 MRO, attribute access. Magic method dispatch (#380) needs mb_call_method to lookup dunder methods via MRO and invoke them.
  - symbols: `mb_class_new`, `mb_instance_new`, `mb_getattr`, `mb_setattr`, `MbClass`, `compute_c3_mro`
- **crates/mamba/src/runtime/iter.rs** — Iterator types: RangeIterator, ListIterator, DictKeyIterator, StringCharIterator. New builtins (enumerate, zip, reversed) need new iterator variants.
  - symbols: `mb_iter_new`, `mb_iter_next`, `MbIterator`, `RangeIterator`, `ListIterator`
- **crates/mamba/src/runtime/symbols.rs** — Runtime extern function registration. ALL new runtime functions must be registered here as MirExtern entries.
  - symbols: `runtime_externs`, `SYMBOL_TABLE`
- **crates/mamba/src/runtime/mod.rs** — Module declarations for runtime. New modules (file_io, exception hierarchy) need pub mod here.
  - symbols: `pub mod builtins`, `pub mod rc`, `pub mod class`, `pub mod iter`
- **crates/mamba/src/codegen/cranelift/mod.rs** — AOT codegen. emit_inst handles MirInst variants including CallExtern for runtime functions. Magic method dispatch may need new MirInst or enhanced CallExtern.
  - symbols: `emit_inst`, `emit_extern_call`, `emit_internal_call`, `collect_used_externs`
- **crates/mamba/src/codegen/cranelift/jit.rs** — JIT codegen. Parallel to AOT but links symbols at runtime. New runtime functions auto-link via symbol table.
  - symbols: `JitBackend`, `compile_function`, `emit_inst`
- **crates/mamba/src/lower/hir_to_mir.rs** — HIR to MIR lowering. Method calls, attribute access, exception handling lowered here. Dunder method dispatch needs lowering support.
  - symbols: `lower_function`, `lower_expr`, `lower_stmt`, `lower_call`
- **crates/mamba/src/runtime/exception.rs** — Exception types: MbException with type_name, message, cause, traceback. Needs class-based hierarchy (ValueError, TypeError, etc.).
  - symbols: `MbException`, `mb_raise`, `mb_get_exception`, `mb_clear_exception`
- **crates/mamba/tests/pipeline_tests.rs** — Integration tests for full compilation pipeline. New tests for method calls, builtins, exceptions needed.
  - symbols: `test_pipeline_*`
- **crates/mamba/tests/jit_tests.rs** — JIT-specific tests. Test method dispatch and builtins via JIT execution.
  - symbols: `test_jit_*`

## Prism Results

- **prism_symbols** (query: `runtime builtins in cclab-mamba`)
  - Found 9 builtin functions (mb_print, mb_len, mb_int, mb_float, mb_str, mb_bool, mb_type, mb_abs, mb_range) in builtins.rs. All use extern C ABI with i64 args/returns.
- **prism_symbols** (query: `ObjData variants in rc.rs`)
  - ObjData enum has 11 variants: Str, List, Dict, Tuple, Class, Instance, Closure, Iterator, Generator, Exception, Module. No File or Set variant.
- **prism_references** (query: `mb_getattr usage`)
  - mb_getattr called from codegen (emit_inst GetAttr case) and class.rs. Currently does instance dict lookup + class dict MRO walk. Does not dispatch to type-specific methods (str.split, list.append etc).

## Dependency Graph

- builtins.rs depends on rc.rs (MbObject, ObjData, mb_value_* helpers)
- string_ops.rs depends on rc.rs (ObjData::Str extraction)
- class.rs depends on rc.rs (ObjData::Class, ObjData::Instance)
- iter.rs depends on rc.rs (ObjData::Iterator)
- symbols.rs depends on all runtime modules (registers their extern functions)
- codegen/cranelift/mod.rs depends on symbols.rs (links extern functions)
- codegen/cranelift/jit.rs depends on symbols.rs (runtime symbol lookup)
- lower/hir_to_mir.rs depends on types/ (TypeId resolution for method dispatch)
- exception.rs depends on rc.rs (MbException stored in thread-local)
