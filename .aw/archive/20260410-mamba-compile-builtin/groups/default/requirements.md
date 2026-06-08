---
change: mamba-compile-builtin
group: default
date: 2026-04-10
---

# Requirements

Implement the `compile(source, filename, mode)` builtin for cclab-mamba (#976). The function currently returns the source string as a placeholder. The implementation must:

- Add a `CodeObject` variant to `ObjKind` and `ObjData` in `crates/mamba/src/runtime/rc.rs` that stores the parsed AST (`Module`), filename, mode, and source string.
- Implement `mb_compile` in `crates/mamba/src/runtime/builtins.rs` to parse the source string using the mamba parser and return a heap-allocated `CodeObject` value.
- Support `mode = "exec"` (parse as module/statement sequence), `mode = "eval"` (parse as single expression), `mode = "single"` (parse as single interactive statement).
- Raise `SyntaxError` with mode-appropriate error messages for invalid modes and parse failures (R2, R4).
- Include line/column information in SyntaxError messages (R4).
- Accept `flags` and `dont_inherit` parameters without error (R5).
- Accept `source` as `bytes` object (R7, nice-to-have).
- Write tests covering: successful code object creation for all three modes, SyntaxError on invalid mode, SyntaxError on malformed source, eval mode rejection of statements, single mode rejection of multi-statement input.
