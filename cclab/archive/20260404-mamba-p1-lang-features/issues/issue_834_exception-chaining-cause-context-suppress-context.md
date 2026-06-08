---
number: 834
title: "Exception chaining — __cause__, __context__, suppress_context"
state: open
labels: [type:enhancement, priority:p1, crate:mamba]
group: "exception-chaining"
---

# #834 — Exception chaining — __cause__, __context__, suppress_context

## Summary

Implement Python exception chaining (PEP 3134):
```python
try:
    1/0
except ZeroDivisionError as e:
    raise ValueError("bad") from e  # explicit chaining

try:
    1/0
except:
    raise ValueError("bad")  # implicit chaining
```

## Current State

Marked as TODO in `runtime/exception.rs`:
- `__cause__` (explicit chain via `raise X from Y`)
- `__context__` (implicit chain when raising during `except`)
- `suppress_context` flag
- Traceback rendering of chained exceptions

## Scope

- **Parser**: `raise X from Y` syntax (may already parse)
- **Runtime**: Store `__cause__` / `__context__` on exception objects
- **Diagnostic**: Print chained tracebacks with "The above exception was the direct cause..." / "During handling of the above exception..."
