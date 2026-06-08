---
number: 382
title: "feat(mamba): isinstance/issubclass and type narrowing"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #382 — feat(mamba): isinstance/issubclass and type narrowing

## Summary
Implement `isinstance()` and `issubclass()` builtins with type checker narrowing support.

## Runtime
- `isinstance(obj, classinfo)` — check object type against class or tuple of classes
- `issubclass(cls, classinfo)` — check class inheritance
- Must walk C3 MRO for inheritance checks
- Support tuple of types: `isinstance(x, (int, str))`

## Type Checker Integration
- After `if isinstance(x, Foo):` the type of `x` narrows to `Foo` in the if-body
- After `if not isinstance(x, Foo):` narrows in the else-body
- Flow-sensitive type narrowing in `check_expr.rs` / `check_stmt.rs`
- Support `assert isinstance(x, Foo)` as narrowing assertion
