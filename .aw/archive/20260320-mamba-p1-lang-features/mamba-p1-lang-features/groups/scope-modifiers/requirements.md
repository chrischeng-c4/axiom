---
change: mamba-p1-lang-features
group: scope-modifiers
date: 2026-03-20
---

# Requirements

Support `global` and `nonlocal` statements for explicit scope control in the mamba compiler.

- `global x`: parsed inside function bodies; resolver marks `x` as resolved against the module-level global scope; codegen emits load/store to the global table rather than the local frame.
- `nonlocal x`: parsed inside nested functions; resolver marks `x` as captured from the nearest enclosing non-global scope; codegen emits closure capture/upvalue operations.
- Parser: `global NAME (, NAME)*` and `nonlocal NAME (, NAME)*` statement forms.
- Resolver: propagate global/nonlocal annotations through the scope chain during name resolution; `nonlocal` in outermost function (no enclosing function scope) is a `SyntaxError`.
- Codegen: emit correct load/store opcodes for global vs local vs closure-captured variables based on resolver annotations.
