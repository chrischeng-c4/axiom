---
change: mamba-p1-lang-features
group: decorator-full
date: 2026-03-20
---

# Requirements

Ensure full decorator support in the mamba compiler: decorator expressions, argument passing, and stacking.

- Attribute access decorators: `@app.route("/api")` — decorator position allows arbitrary attribute chains.
- Argument decorators: `@cache(maxsize=128)` — decorator expression is a call expression returning the actual decorator.
- Stacked decorators: multiple `@` lines above a function are applied bottom-up (innermost first), matching CPython semantics.
- Parser: decorator expression can be any expression per PEP 614 (not limited to simple names); handle multiple decorators per function/class definition.
- Type checker: the decorated function's type is replaced by the return type of the decorator callable; for unrecognized or complex decorators, fall back gracefully (e.g., preserve `Any` or original type).
- Codegen: collect decorators, emit function definition, then apply decorators in reverse order.
