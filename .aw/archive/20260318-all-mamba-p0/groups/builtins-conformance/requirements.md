---
change: all-mamba-p0
group: builtins-conformance
date: 2026-03-18
---

# Requirements

Verify all Python builtins match CPython 3.12 behavior. 108 tests exist but need conformance verification — currently they assert Mamba behavior without comparing to CPython. Cover numeric (int, float, complex, round, abs, pow, divmod), sequence (len, range, sorted, reversed, enumerate, zip, map, filter), string (str, repr, format, chr, ord, ascii), type (type, isinstance, issubclass, callable), and remaining builtins.
