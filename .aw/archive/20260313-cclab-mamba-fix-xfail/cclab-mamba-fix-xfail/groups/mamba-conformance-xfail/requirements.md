---
change: cclab-mamba-fix-xfail
group: mamba-conformance-xfail
date: 2026-03-13
---

# Requirements

Fix all 7 remaining xfail conformance tests and verify/close the 7 open conformance issues.

## Already Done (verify & close)
- #752 Test harness: conformance framework with golden-file + xfail working
- #753 Arithmetic/comparison: all 6 tests pass (int, float, mixed, unary, compare, truthiness)
- #759 Data structures: all 9 tests pass (list/dict/set/tuple/string ops, comprehensions, slicing)

## Partially Done (complete remaining)
- #755 Exceptions: 3/5 pass. Remaining: custom.py (needs custom exception classes from #754), exception_group.py (needs PEP 654 ExceptionGroup/except*)
- #758 Builtins: 1 test passes (type_conversions). Need broader builtin verification

## Not Started (implement)
- #754 Object model: class definitions, inheritance, MRO, instance attributes, custom exception subclassing. Needed by #755 custom exceptions and #756 iterator protocol
- #756 Generators: yield state machine in Cranelift JIT, send/throw, StopIteration.value, yield from, iterator protocol (__iter__/__next__)

## Key Implementation Areas
1. Class definitions in JIT: `class Foo(Bar):`, __init__, instance creation, method dispatch
2. Generator state machine: yield suspension/resumption via Cranelift, send/throw protocol
3. Iterator protocol: __iter__/__next__ with StopIteration
4. ExceptionGroup / except* (PEP 654)
5. Custom exception subclassing with inheritance
