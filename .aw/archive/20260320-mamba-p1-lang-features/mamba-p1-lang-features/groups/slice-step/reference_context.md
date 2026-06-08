---
change: mamba-p1-lang-features
group: slice-step
date: 2026-03-20
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| runtime/list-ops.md | slice-step | high | R4: mb_list_slice_full with step support; raise ValueError when step==0 (instead of empty list silently) |
| runtime/string-ops.md | slice-step | high | New: mb_str_slice_full(s, start, stop, step) supporting Unicode codepoint stepping (Q3 requirement) |
| runtime/tuple-ops.md | slice-step | high | New: mb_tuple_slice_full(t, start, stop, step) for tuple slicing with step |
| runtime/class.md | slice-step | high | R1, R5: Type-aware mb_obj_getitem dispatch for slice 3-tuple: Str→mb_str_slice_full, Tuple→mb_tuple_slice_full, Bytes→mb_bytes_slice_full, List→mb_list_slice_full (Q1 critical bug) |
| runtime/exception.md | slice-step | high | R1, R2, R4: ValueError exception class in hierarchy, instantiation, thread-local raise mechanism for step==0 error (Q4 requirement) |
| runtime/symbols.md | slice-step | medium | R1, R2: Register mb_str_slice_full, mb_tuple_slice_full, mb_bytes_slice_full in symbol table with MirExtern declarations |
| runtime/bytes-ops.md | slice-step | medium | R1: New mb_bytes_slice_full(b, start, stop, step) for immutable byte slicing with step |
| lower/hir-to-mir.md | slice-step | medium | Background context: HirExpr::Slice already lowered to 3-element MakeTuple(start, stop, step); no code changes needed, spec gap to document |
| testing/test-harness.md | slice-step | medium | R1, R3: JIT fixtures for list[::2], 'hello'[::2] (Unicode), tuple[::2], a[::0] raises ValueError, a[::-1] reverse step |
| parser/expressions.md | slice-step | low | Background: Parser already handles 3-arg Slice { start, stop, step }; no parser changes in scope |
| parser/ast.md | slice-step | low | Background: AST Expr::Slice { start, stop, step } already exists; no spec updates needed |

