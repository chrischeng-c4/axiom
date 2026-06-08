---
number: 1037
title: "test(mamba): Py3.12 behavioral conformance — every function must match CPython"
state: open
labels: [priority:p1, crate:mamba, type:test]
group: "mamba-conformance-xfails"
---

# #1037 — test(mamba): Py3.12 behavioral conformance — every function must match CPython

## Context

Layer 1 (line/branch coverage) is tracked in #1035 — currently 74% line, 86% branch.

This issue is **Layer 2: behavioral conformance**. Even with 100% coverage, tests might assert wrong behavior. Every stdlib function, every builtin, every language feature must produce the same result as CPython 3.12.

## Scope

### Builtins (108 functions)
For each builtin (`abs`, `all`, `any`, `bin`, `bool`, `chr`, `dict`, `dir`, `enumerate`, `filter`, `float`, `format`, `getattr`, `hasattr`, `hash`, `hex`, `id`, `input`, `int`, `isinstance`, `issubclass`, `iter`, `len`, `list`, `map`, `max`, `min`, `next`, `oct`, `open`, `ord`, `pow`, `print`, `range`, `repr`, `reversed`, `round`, `set`, `setattr`, `sorted`, `str`, `sum`, `tuple`, `type`, `zip`, ...):
- Test with same inputs as CPython
- Assert same outputs, same exceptions, same edge cases

### Data Structures
- `list`: all 33 methods, slicing, unpacking, comparison
- `dict`: all 17 methods, view objects, merge operators
- `set`: all 17 methods, operators
- `str`: all 47 methods, encoding, f-strings
- `tuple`: immutability, unpacking, comparison
- `bytes`/`bytearray`: all methods

### Stdlib (82 modules)
For each module's public API:
- Port relevant CPython test cases
- Focus on: json, os, re, datetime, collections, pathlib, math, sys, io, csv, hashlib, itertools, functools, struct, random, asyncio

### Language Features
- Class system: MRO, descriptors, metaclass, `super()`, `__init_subclass__`
- Generators: `send()`, `throw()`, `close()`, async generators
- Exception hierarchy: chaining, groups, `except*`
- Pattern matching: all pattern types
- Comprehensions: scope rules (PEP 709)
- Decorators: stacked, parameterized, class decorators

## Method

1. Port CPython's `Lib/test/` cases as Mamba fixtures
2. Run each fixture with both CPython 3.12 and Mamba
3. Diff outputs — any difference is a conformance bug
4. Fix Mamba until outputs match

## Acceptance Criteria

- [ ] All builtins produce same results as CPython 3.12
- [ ] All data structure methods produce same results
- [ ] Top-16 stdlib modules fully conformant
- [ ] All language features match CPython behavior
- [ ] Automated conformance test runner: `cclab mamba test --conformance`
