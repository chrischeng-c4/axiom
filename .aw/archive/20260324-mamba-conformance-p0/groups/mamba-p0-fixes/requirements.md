---
change: mamba-conformance-p0
group: mamba-p0-fixes
date: 2026-03-24
---

# Requirements

Fix 6 P0 conformance gaps that block real Python programs:

1. **Lambda SIGBUS** — lambda expressions crash JIT codegen. Root cause: closure codegen in cranelift/mod.rs doesn't handle anonymous functions.

2. **With-statement SIGBUS** — `with` context managers crash. Root cause: __enter__/__exit__ calling convention wrong in codegen.

3. **@decorator SIGBUS** — stacked decorators crash JIT. Root cause: decorator application codegen emits invalid IR.

4. **Stdlib functions return None** — 11 stdlib modules (collections, datetime, hashlib, etc.) have functions that return None instead of results. Root cause: module Dict callable dispatch in class.rs incomplete.

5. **Comprehension scope leaks** — list/dict/set comprehension iteration variables leak into enclosing scope. Root cause: resolve/pass.rs doesn't create isolated scope for comprehensions.

6. **Walrus := wrong scope** — walrus operator in comprehensions assigns to comprehension scope instead of enclosing function scope. Root cause: resolve/pass.rs NamedExpr handling.
