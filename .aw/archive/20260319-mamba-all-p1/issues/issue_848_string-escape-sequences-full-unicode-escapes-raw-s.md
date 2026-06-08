---
number: 848
title: "String escape sequences — full Unicode escapes, raw strings, byte strings"
state: open
labels: [enhancement, P1, crate:mamba]
group: "syntax-features"
---

# #848 — String escape sequences — full Unicode escapes, raw strings, byte strings

## Summary

Ensure complete string literal support:
- Unicode escapes: `\uXXXX`, `\UXXXXXXXX`, `\N{name}`
- Raw strings: `r"no\escape"`, `r'\n is literal'`
- Byte strings: `b"bytes"`, `b'\xff'`
- Raw byte strings: `rb"raw\bytes"`
- Hex/octal escapes: `\xNN`, `\0NNN`

## Scope

- **Lexer**: Handle all string prefix combinations (r, b, rb, f, fr)
- **Parser**: Distinguish str vs bytes literal types
- **Type checker**: `b"..."` is `bytes`, not `str`
- **Runtime**: Proper bytes type operations
