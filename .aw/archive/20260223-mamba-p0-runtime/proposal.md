---
id: mamba-p0-runtime
type: proposal
version: 2
created_at: 2026-02-15T17:24:17.116783+00:00
updated_at: 2026-02-15T17:24:17.116783+00:00
iteration: 1
scope: minor
spec_plan:
  - id: method-dispatch
    title: "Type-Tagged Method Dispatch for Built-in Types"
    depends: []
    context_refs:
      codebase: ["class.rs: mb_getattr", "rc.rs: ObjData enum"]
      spec: ["mamba-oop-model R3: Magic Method Dispatch"]
      knowledge: ["NaN-boxed MbValue", "ObjData Variants"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
      - { source: gap_spec_knowledge, gap_index: 1 }
      - { source: gap_spec_knowledge, gap_index: 7 }
    affected_code: ["crates/mamba/src/runtime/class.rs", "crates/mamba/src/runtime/rc.rs", "crates/mamba/src/lower/hir_to_mir.rs", "crates/mamba/src/runtime/symbols.rs"]
  - id: string-methods
    title: "String Method Implementations (split, join, strip, replace, etc.)"
    depends: [method-dispatch]
    context_refs:
      codebase: ["string_ops.rs: mb_string_concat, mb_string_repeat"]
      spec: ["mamba-string-runtime R3: String Methods"]
      knowledge: ["extern C ABI", "Symbol Registration"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 2 }
      - { source: gap_codebase_spec, gap_index: 7 }
    affected_code: ["crates/mamba/src/runtime/string_ops.rs", "crates/mamba/src/runtime/symbols.rs"]
  - id: list-methods
    title: "List Method Implementations (append, pop, sort, extend, etc.)"
    depends: [method-dispatch]
    context_refs:
      codebase: ["rc.rs: ObjData::List"]
      spec: ["mamba-iteration-protocol R3: Built-in Iterators"]
      knowledge: ["ObjData Variants", "GC tracking"]
    gap_repairs:
      - { source: gap_codebase_knowledge, gap_index: 2 }
    affected_code: ["crates/mamba/src/runtime/list_ops.rs", "crates/mamba/src/runtime/symbols.rs", "crates/mamba/src/runtime/mod.rs"]
  - id: dict-methods
    title: "Dict Method Implementations (get, keys, values, items, update, etc.)"
    depends: [method-dispatch]
    context_refs:
      codebase: ["rc.rs: ObjData::Dict"]
      spec: ["mamba-iteration-protocol R3: Built-in Iterators"]
      knowledge: ["ObjData Variants", "GC tracking"]
    gap_repairs:
      - { source: gap_codebase_knowledge, gap_index: 2 }
    affected_code: ["crates/mamba/src/runtime/dict_ops.rs", "crates/mamba/src/runtime/symbols.rs", "crates/mamba/src/runtime/mod.rs"]
  - id: core-builtins
    title: "Core Built-in Functions (enumerate, zip, min, max, sum, sorted, isinstance, input)"
    depends: [method-dispatch]
    context_refs:
      codebase: ["builtins.rs: mb_print, mb_len, mb_abs, mb_range", "iter.rs: RangeIterator"]
      spec: ["mamba-stdlib-core R1-R4", "mamba-iteration-protocol R3"]
      knowledge: ["Symbol Registration", "extern C ABI"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 5 }
      - { source: gap_codebase_spec, gap_index: 8 }
    affected_code: ["crates/mamba/src/runtime/builtins.rs", "crates/mamba/src/runtime/iter.rs", "crates/mamba/src/runtime/symbols.rs"]
  - id: exception-hierarchy
    title: "Class-Based Exception Hierarchy (BaseException → Exception → ValueError, TypeError, etc.)"
    depends: [method-dispatch]
    context_refs:
      codebase: ["exception.rs: MbException", "class.rs: MbClass, compute_c3_mro"]
      spec: ["mamba-oop-model R3: Magic Method Dispatch"]
      knowledge: ["ObjData Variants", "Thread-local storage"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 4 }
      - { source: gap_codebase_knowledge, gap_index: 4 }
      - { source: gap_codebase_knowledge, gap_index: 6 }
    affected_code: ["crates/mamba/src/runtime/exception.rs", "crates/mamba/src/runtime/rc.rs", "crates/mamba/src/runtime/class.rs", "crates/mamba/src/runtime/symbols.rs"]
  - id: magic-methods
    title: "Magic Method Dispatch (__add__, __str__, __eq__, __len__, __iter__, __next__)"
    depends: [method-dispatch, exception-hierarchy]
    context_refs:
      codebase: ["class.rs: mb_getattr", "codegen/cranelift/mod.rs: emit_inst"]
      spec: ["mamba-oop-model R3: Magic Method Dispatch"]
      knowledge: ["NaN-boxed MbValue", "VarAlloc first-call semantics"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 5 }
      - { source: gap_spec_knowledge, gap_index: 1 }
    affected_code: ["crates/mamba/src/runtime/class.rs", "crates/mamba/src/lower/hir_to_mir.rs", "crates/mamba/src/codegen/cranelift/mod.rs", "crates/mamba/src/codegen/cranelift/jit.rs", "crates/mamba/src/runtime/symbols.rs"]
  - id: file-io
    title: "File I/O Runtime (open, read, write, close with ObjData::File)"
    depends: [method-dispatch, exception-hierarchy]
    context_refs:
      codebase: ["rc.rs: ObjData enum (needs File variant)"]
      spec: ["mamba-stdlib-core R1-R4"]
      knowledge: ["ObjData Variants", "GC tracking"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 3 }
      - { source: gap_spec_knowledge, gap_index: 2 }
    affected_code: ["crates/mamba/src/runtime/file_io.rs", "crates/mamba/src/runtime/rc.rs", "crates/mamba/src/runtime/mod.rs", "crates/mamba/src/runtime/symbols.rs"]
history:
  - timestamp: 2026-02-15T17:24:17.116783+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: mamba-p0-runtime

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((mamba-p0-runtime))  
    Method Dispatch
      type-tag dispatch table
      ObjData variant routing
      built-in type methods
      MRO fallback for user classes
    String Methods
      split/join/strip/replace
      find/startswith/endswith
      upper/lower/count
      isdigit/isalpha
    Collection Methods
      list: append/pop/sort/extend/insert/remove
      dict: get/keys/values/items/update/pop
      GC tracking for new containers
    Core Builtins
      enumerate/zip/reversed iterators
      min/max/sum/sorted
      isinstance/type checks
      input/hash/id
    Exception Hierarchy
      BaseException → Exception tree
      ValueError/TypeError/KeyError/IndexError
      class-based raise/except matching
      thread-local current exception
    Magic Methods
      __add__/__sub__/__mul__ operators
      __str__/__repr__ conversion
      __eq__/__lt__/__gt__ comparison
      __len__/__iter__/__next__ protocol
    File I/O
      ObjData::File variant
      open/read/write/close
      context manager (__enter__/__exit__)
      error handling (FileNotFoundError)
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  method_dispatch["method-dispatch\n codebase: class.rs: mb_getattr, rc.rs: ObjData enum\n gaps: codebase_spec#1, spec_knowledge#1, spec_knowledge#7"]
  string_methods["string-methods\n codebase: string_ops.rs: mb_string_concat, mb_string_repeat\n gaps: codebase_spec#2, codebase_spec#7"]
  list_methods["list-methods\n codebase: rc.rs: ObjData::List\n gaps: codebase_knowledge#2"]
  dict_methods["dict-methods\n codebase: rc.rs: ObjData::Dict\n gaps: codebase_knowledge#2"]
  core_builtins["core-builtins\n codebase: builtins.rs: mb_print, mb_len, mb_abs, mb_range, iter.rs: RangeIterator\n gaps: codebase_spec#5, codebase_spec#8"]
  exception_hierarchy["exception-hierarchy\n codebase: exception.rs: MbException, class.rs: MbClass, compute_c3_mro\n gaps: codebase_spec#4, codebase_knowledge#4, codebase_knowledge#6"]
  magic_methods["magic-methods\n codebase: class.rs: mb_getattr, codegen/cranelift/mod.rs: emit_inst\n gaps: codebase_spec#5, spec_knowledge#1"]
  file_io["file-io\n codebase: rc.rs: ObjData enum (needs File variant)\n gaps: codebase_spec#3, spec_knowledge#2"]

  method_dispatch --> string_methods
  method_dispatch --> list_methods
  method_dispatch --> dict_methods
  method_dispatch --> core_builtins
  method_dispatch --> exception_hierarchy
  method_dispatch --> magic_methods
  exception_hierarchy --> magic_methods
  method_dispatch --> file_io
  exception_hierarchy --> file_io
```

## Spec Execution Order

1. **method-dispatch** — Type-Tagged Method Dispatch for Built-in Types
   - code: crates/mamba/src/runtime/class.rs, crates/mamba/src/runtime/rc.rs, crates/mamba/src/lower/hir_to_mir.rs, crates/mamba/src/runtime/symbols.rs
2. **core-builtins** — Core Built-in Functions (enumerate, zip, min, max, sum, sorted, isinstance, input)
   - depends: method-dispatch
   - code: crates/mamba/src/runtime/builtins.rs, crates/mamba/src/runtime/iter.rs, crates/mamba/src/runtime/symbols.rs
3. **dict-methods** — Dict Method Implementations (get, keys, values, items, update, etc.)
   - depends: method-dispatch
   - code: crates/mamba/src/runtime/dict_ops.rs, crates/mamba/src/runtime/symbols.rs, crates/mamba/src/runtime/mod.rs
4. **exception-hierarchy** — Class-Based Exception Hierarchy (BaseException → Exception → ValueError, TypeError, etc.)
   - depends: method-dispatch
   - code: crates/mamba/src/runtime/exception.rs, crates/mamba/src/runtime/rc.rs, crates/mamba/src/runtime/class.rs, crates/mamba/src/runtime/symbols.rs
5. **file-io** — File I/O Runtime (open, read, write, close with ObjData::File)
   - depends: method-dispatch, exception-hierarchy
   - code: crates/mamba/src/runtime/file_io.rs, crates/mamba/src/runtime/rc.rs, crates/mamba/src/runtime/mod.rs, crates/mamba/src/runtime/symbols.rs
6. **list-methods** — List Method Implementations (append, pop, sort, extend, etc.)
   - depends: method-dispatch
   - code: crates/mamba/src/runtime/list_ops.rs, crates/mamba/src/runtime/symbols.rs, crates/mamba/src/runtime/mod.rs
7. **magic-methods** — Magic Method Dispatch (__add__, __str__, __eq__, __len__, __iter__, __next__)
   - depends: method-dispatch, exception-hierarchy
   - code: crates/mamba/src/runtime/class.rs, crates/mamba/src/lower/hir_to_mir.rs, crates/mamba/src/codegen/cranelift/mod.rs, crates/mamba/src/codegen/cranelift/jit.rs, crates/mamba/src/runtime/symbols.rs
8. **string-methods** — String Method Implementations (split, join, strip, replace, etc.)
   - depends: method-dispatch
   - code: crates/mamba/src/runtime/string_ops.rs, crates/mamba/src/runtime/symbols.rs

</proposal>
