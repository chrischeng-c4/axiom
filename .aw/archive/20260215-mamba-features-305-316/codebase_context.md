---
change_id: mamba-features-305-316
type: codebase_context
created_at: 2026-02-14T09:27:29.969054+00:00
updated_at: 2026-02-14T09:27:29.969054+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_references
  - prism_impact
---

# Codebase Context

## Analyzed Files

- **crates/mamba/src/codegen/mod.rs** — Defines the interface for codegen backends (CodegenBackend trait).
  - symbols: `CodegenBackend`, `CodegenOutput`
- **crates/mamba/src/runtime/rc.rs** — Implements reference counting and object headers. Needs extension for cycle-detecting GC (#315).
  - symbols: `MbObjectHeader`, `mb_retain`, `mb_release`
- **crates/mamba/src/runtime/async_rt.rs** — Basic async runtime. Needs integration with cclab-orbit and a real scheduler (#313).
  - symbols: `MbCoroutine`, `mb_await`, `mb_create_task`
- **crates/mamba/src/runtime/class.rs** — Implements the OOP system, including MRO and operator overloading (#307).
  - symbols: `MbClass`, `mb_class_register`, `mb_getattr`, `mb_dispatch_binop`
- **crates/mamba/src/parser/pattern.rs** — Parses patterns for match/case (#309).
  - symbols: `parse_pattern`
- **crates/mamba/src/parser/stmt_compound.rs** — Parses compound statements like match, class, def, and type alias (#307, #309, #314).
  - symbols: `parse_match`, `parse_class_def`, `parse_type_alias`
- **crates/mamba/src/parser/expr_compound.rs** — Parses comprehensions and generator expressions (#308).
  - symbols: `parse_list_or_comp`, `parse_comprehension_clauses`

## Prism Results

- **list_directory** (query: `list crates/mamba/src/codegen/`)
  - Confirmed only cranelift backend exists currently.
- **read_file** (query: `read crates/mamba/src/runtime/rc.rs`)
  - Current memory management is simple reference counting.
- **read_file** (query: `read crates/mamba/src/parser/stmt_compound.rs`)
  - Syntax for match, class, and type alias is already present in the parser.

## Dependency Graph

- crates/mamba/src/codegen/mod.rs: Defines CodegenBackend trait for pluggable backends.
- crates/mamba/src/runtime/mod.rs: Entry point for the Mamba runtime.
- crates/mamba/src/parser/ast.rs: Defines AST nodes for all language features.
