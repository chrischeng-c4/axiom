---
change: fix-conformance-xfails
group: mamba-conformance-xfails
date: 2026-03-23
---

# Requirements

Fix all 31 conformance xfails in the cclab-mamba crate so that each affected fixture produces output identical to CPython 3.12. The xfails span four categories:

1. **Codegen IR bugs** (4 fixtures): classmethod, descriptor `__get__`, getattr/setattr/delattr, and super() each emit invalid or incompatible Cranelift IR — resulting in compilation errors, duplicate function definitions, or SIGBUS crashes. These must be fixed at the IR-generation layer (likely in the class-system and attribute-access codegen passes).

2. **Runtime bugs** (multiple fixtures): (a) Several stdlib module wrappers (itertools, io, pathlib, random, re, struct) return non-iterable objects where CPython returns iterables, causing downstream TypeError at runtime. (b) bytes/bytearray methods — replace, strip, startswith, endswith — are partially unimplemented or return wrong values. (c) Exception chaining: `__cause__` and `__context__` attributes are not populated, and ExceptionGroup is not implemented.

3. **Parser gaps** (2 fixtures): Nested f-strings (arbitrary nesting introduced in Python 3.12) cause parse failures; the metaclass= keyword in class declarations is not recognized by the parser.

4. **Compiler bugs** (3 fixtures): (a) Walrus operator (:=) inside comprehensions assigns to the comprehension's inner scope instead of the enclosing scope (violates PEP 572). (b) Integer literal patterns in match statements produce wrong values. (c) Comprehension scope isolation (PEP 709) not fully enforced.

All fixes must preserve existing passing tests. Acceptance: every xfail fixture transitions to passing in `cclab mamba test --conformance`.
