---
change: mamba-all-p1
group: syntax-features
date: 2026-03-19
---

# Requirements

Implement 7 missing Python syntax features across the parser/lexer/type-checker/codegen pipeline:
- #830 PEP 695 type parameter syntax: type bounds, ParamSpec, TypeVarTuple, generic class keywords, `type` alias statement
- #831 Dict literal unpacking: `{**d1, **d2, key: val}` (PEP 448) — parser + type inference + codegen dict-merge
- #832 Parenthesized with statements (PEP 617): `with (cm1 as a, cm2 as b,):` — parser-only change, rest of pipeline reuses existing with-handling
- #845 Star expressions / extended unpacking (PEP 3132): `first, *rest = [...]`, `[*a, 3]` — parser + type inference + codegen iterator-split
- #846 Global and nonlocal statements: `global x` / `nonlocal x` — parser + resolver scope-chain marking + codegen outer-scope load/store
- #847 Decorator expressions: `@app.route("/api")`, `@cache(maxsize=128)`, stacked decorators — parser (arbitrary expr) + type checker (replace type) + codegen (reverse-order application)
- #848 String literal completeness: all prefix combos (r, b, rb, f, fr), `\uXXXX`/`\UXXXXXXXX`/`\N{name}`, hex/octal escapes — lexer + parser + type checker (bytes vs str) + runtime bytes ops
Acceptance: each feature passes its corresponding CPython conformance test cases; no regressions in existing parser tests.
