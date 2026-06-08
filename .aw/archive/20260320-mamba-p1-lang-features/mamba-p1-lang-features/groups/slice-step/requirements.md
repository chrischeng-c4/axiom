---
change: mamba-p1-lang-features
group: slice-step
date: 2026-03-20
---

# Requirements

Implement step parameter in slice operations across list, tuple, and string types in the mamba runtime.

- Location: `runtime/list_ops.rs`, `runtime/tuple_ops.rs`, `runtime/string_ops.rs` — basic start:stop slicing works; step is currently TODO.
- Algorithm: iterate from start to stop collecting every step-th element.
- Negative step: must support reverse iteration (e.g., `a[::-1]` reverses the sequence); default start/stop must flip when step is negative (Python semantics: start defaults to len-1, stop defaults to before index 0).
- Zero step: raise `ValueError` (CPython behavior).
- Edge cases: negative start/stop indices with negative step must follow Python semantics (e.g., `a[-1::-1]`).
- Codegen: ensure 3-argument slice expressions are lowered to step-aware runtime calls; confirm whether the AST/HIR already represents the third slice argument or if codegen changes are also needed.
