---
change: mamba-p1-lang-features
group: string-literals
date: 2026-03-20
---

# Requirements

Complete string literal type coverage across the lexer, parser, type checker, and runtime.

- Unicode escapes: `\uXXXX`, `\UXXXXXXXX` must be handled in the lexer; `\N{name}` support is optional (see questions).
- Raw strings: `r"..."`, `r'...'` — backslash is literal, no escape processing; combine with b/f prefixes as `rb`, `br`, `rf`, `fr`.
- Byte strings: `b"..."`, `b'...'` with hex escapes (`\xNN`) and octal escapes (`\0NNN`); raw byte strings `rb"..."`.
- Lexer: handle all valid prefix combinations (r, b, rb, br, f, fr, rf) and distinguish them in the token stream.
- Parser: produce distinct AST nodes / type tags for `str` vs `bytes` literals.
- Type checker: `b"..."` literals have type `bytes`, not `str`.
- Runtime: implement `bytes` type with at minimum: construction, `len`, indexing (returns `int`), slicing, concatenation, comparison, and `repr`.
