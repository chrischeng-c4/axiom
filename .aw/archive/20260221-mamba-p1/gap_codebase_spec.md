---
change_id: mamba-p1
type: gap_codebase_spec
created_at: 2026-02-20T17:00:07.436248+00:00
updated_at: 2026-02-20T17:00:07.436248+00:00
---

# Gap Analysis: Codebase vs Spec (mamba-p1)

## Code without Matching Spec

- **crates/mamba/src/runtime/dict_ops.rs** [high]
  - Description: Implementation of dictionary runtime operations (creation, get/set, methods).
- **crates/mamba/src/runtime/list_ops.rs** [high]
  - Description: Implementation of list runtime operations (creation, append, len, slicing).
- **crates/mamba/src/runtime/tuple_ops.rs** [high]
  - Description: Implementation of tuple runtime operations (creation, hashing, comparison).
- **crates/mamba/src/runtime/exception.rs** [high]
  - Description: Implementation of the exception handling runtime system (raise, try/except mechanics).
- **crates/mamba/src/runtime/closure.rs** [high]
  - Description: Implementation of function closures and environment capturing (nonlocal support).
- **crates/mamba/src/lower/hir_to_mir.rs** (With-statement) [high]
  - Description: With-statement codegen and context manager protocol implementation (__enter__/__exit__).
- **crates/mamba/src/runtime/file_io.rs** [medium]
  - Description: Implementation of file I/O operations and built-ins.
- **crates/mamba/src/runtime/class.rs** (super()) [medium]
  - Description: Runtime support for super() and Method Resolution Order (MRO) logic.
- **crates/cclab-cli/src/mamba.rs** [low]
  - Description: CLI subcommand and argument definitions for the mamba tool.

## Specs without Matching Implementation

- **mamba-repl-tool** [medium]
  - Description: The REPL and interactive mode spec exists, but the 'mamba' CLI lacks a 'repl' command, and no REPL implementation exists in the source.
- **mamba-py312-syntax#R4** [high]
  - Description: PEP 701 f-strings (nested, comments, multi-line) are specified, but the current implementation in parser/expr.rs (parse_fstring_parts) is a simple placeholder that only handles basic interpolation.
