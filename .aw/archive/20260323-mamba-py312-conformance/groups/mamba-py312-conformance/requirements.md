---
change: mamba-py312-conformance
group: mamba-py312-conformance
date: 2026-03-23
---

# Requirements

Build a behavioral conformance test suite for `crates/cclab-mamba` that verifies every Python builtin, data structure method, top-16 stdlib module, and core language feature produces identical output to CPython 3.12.

Two distinct deliverables:

**1. Conformance test fixtures**
Port CPython `Lib/test/` cases as Mamba fixtures. Each fixture tests a function or language feature with the same inputs as CPython and asserts identical outputs, exceptions, and edge-case behavior.

Coverage areas:
- Builtins (108 functions): abs, all, any, bin, bool, chr, dict, dir, enumerate, filter, float, format, getattr, hasattr, hash, hex, id, int, isinstance, issubclass, iter, len, list, map, max, min, next, oct, open, ord, pow, print, range, repr, reversed, round, set, setattr, sorted, str, sum, tuple, type, zip, ...
- Data structures: list (33 methods, slicing, unpacking, comparison), dict (17 methods, view objects, merge operators), set (17 methods, operators), str (47 methods, encoding, f-strings), tuple (immutability, unpacking, comparison), bytes/bytearray (all methods)
- Stdlib top-16: json, os, re, datetime, collections, pathlib, math, sys, io, csv, hashlib, itertools, functools, struct, random, asyncio
- Language features: class system (MRO, descriptors, metaclass, super(), __init_subclass__), generators (send/throw/close, async generators), exception hierarchy (chaining, groups, except*), pattern matching (all PEP 634 pattern types), comprehensions (PEP 709 scope rules), decorators (stacked, parameterized, class decorators)

**2. Automated conformance runner**
`cclab mamba test --conformance` CLI subcommand that runs all conformance fixtures against both CPython 3.12 and Mamba, diffs the outputs, and reports divergences.

Acceptance criteria:
- All builtins produce same results as CPython 3.12
- All data structure methods produce same results
- Top-16 stdlib modules fully conformant
- All language features match CPython behavior
- `cclab mamba test --conformance` runs the full suite and reports pass/fail per fixture
