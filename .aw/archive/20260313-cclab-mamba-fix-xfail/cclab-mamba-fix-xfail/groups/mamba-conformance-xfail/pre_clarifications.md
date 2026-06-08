---
change: cclab-mamba-fix-xfail
group: mamba-conformance-xfail
date: 2026-03-13
status: answered
---

# Pre-Clarifications

### Q1: scope
- **Answer**: Close #752, #753, #759 as-is. All their conformance tests pass. No additional test coverage needed for this change — focus effort on the remaining xfail tests.

### Q2: priority
- **Answer**: Implement minimal class system sufficient to unblock custom exceptions and iterator protocol. Scope: class definition with single inheritance, __init__, instance creation, method dispatch, super().__init__(). Skip MRO (C3 linearization), descriptors, metaclasses, __slots__, properties, multiple inheritance for now. This is enough for `class MyError(ValueError)` and `class MyIter` with __iter__/__next__.

### Q3: generators
- **Answer**: Implement full generator support with proper state machine transformation in Cranelift. The user demands high quality — no shortcuts. Scope: yield suspension/resumption, send(value), throw(exc), close(), StopIteration.value, yield from delegation. Use a heap-allocated generator frame with state enum (Created/Suspended/Running/Completed) and stack save/restore.

