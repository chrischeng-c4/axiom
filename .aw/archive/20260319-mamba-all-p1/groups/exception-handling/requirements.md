---
change: mamba-all-p1
group: exception-handling
date: 2026-03-19
---

# Requirements

Implement complete Python exception handling semantics matching CPython 3.12:
- #834 Exception chaining (PEP 3134): `raise X from Y` sets `__cause__`; raising during `except` sets `__context__`; `suppress_context` flag; chained traceback rendering ("The above exception was the direct cause..." / "During handling of the above exception...")
- #755 Exception hierarchy conformance: all built-in BaseException subclasses present; `except` tuple matching and subclass catching; `__traceback__` attribute; `ExceptionGroup` and `except*` (PEP 654); `args` attribute; custom exception subclassing
Acceptance: all CPython 3.12 exception conformance tests pass; chained exceptions display correctly in tracebacks.
