---
number: 561
title: "Add targeted fixture: complex string literals & escape sequences"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #561 — Add targeted fixture: complex string literals & escape sequences

## Context
Python string syntax is surprisingly complex with multiple prefix combinations and escape sequences.

## Test cases to cover
- All prefixes: `""`, `b""`, `r""`, `f""`, `rb""`, `br""`, `rf""`, `fr""`
- Triple-quoted variants of all above
- Escape sequences: `\n`, `\t`, `\\`, `\'`, `\"`, `\0`, `\x41`, `\u0041`, `\U00000041`, `\N{SNOWMAN}`
- Raw string escapes: `r"\n"` (backslash preserved)
- Implicit concatenation: `"a" "b"`, `"a" f"{x}"`, `b"a" b"b"`
- Multi-line implicit concatenation
- String with continuation: `"long \`
- Bytes with hex: `b"\x00\xff"`
- Docstrings in various positions

## Task
Create `tests/fixtures/parse/edge_cases/string_literals_complex.py` with `# RUN: parse`.
