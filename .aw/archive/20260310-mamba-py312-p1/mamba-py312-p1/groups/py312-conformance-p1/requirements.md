---
change: mamba-py312-p1
group: py312-conformance-p1
date: 2026-03-10
---

# Requirements

Add conformance test fixtures and runtime fixes for three Py3.12 P1 areas, all using the existing golden-file harness from P0.

### Data Structure Ops (#759)
- Conformance tests for list (append, extend, insert, pop, remove, sort, reverse, slicing, comprehension, comparisons)
- Conformance tests for dict (get, setdefault, update, pop, keys/values/items, dict | merge, comprehension)
- Conformance tests for set (add, discard, remove, union/intersection/difference, set comprehension, frozenset)
- Conformance tests for tuple (unpacking, * unpacking, hashing, comparison)
- Conformance tests for str (split, join, strip, replace, find, format, f-string edge cases)
- Fix any runtime bugs found during conformance testing

### Exception Hierarchy (#755)
- Conformance tests for built-in exception classes (BaseException tree)
- except matching: subclass catching, tuple of exceptions
- raise from — exception chaining (__cause__, __context__)
- Custom exception subclassing
- args attribute on exceptions
- Mark ExceptionGroup/except* as xfail if not implemented

### Generator & Iterator Protocol (#756)
- Conformance tests for yield / yield from
- generator.send(value), generator.throw(exc), generator.close()
- StopIteration.value for return from generator
- Iterator protocol: __iter__, __next__, StopIteration
- Mark async generators as xfail if not fully implemented

### Constraints
- Use existing conformance harness (tests/fixtures/conformance/*.py + .expected)
- Use # mamba-xfail for features not yet implemented
- All 1745+ existing tests must continue to pass
- Golden files generated from CPython 3.12
