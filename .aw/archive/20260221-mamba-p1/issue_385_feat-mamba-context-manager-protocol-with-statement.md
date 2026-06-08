---
number: 385
title: "feat(mamba): context manager protocol (with statement __enter__/__exit__)"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #385 — feat(mamba): context manager protocol (with statement __enter__/__exit__)

## Summary
Wire up `with` statement to call `__enter__` and `__exit__` magic methods.

## Required
- `with expr as var:` — call `__enter__()`, bind result to `var`, call `__exit__()` on block exit
- Multiple context managers: `with a as x, b as y:`
- Exception propagation: `__exit__(exc_type, exc_val, exc_tb)` — if returns True, suppress exception
- `async with` — `__aenter__` / `__aexit__`
- `contextlib.contextmanager` (stdlib, lower priority)

## Implementation Notes
- Parser already handles `with` statement
- Codegen needs to emit `__enter__` call before body, `__exit__` in finally-equivalent cleanup
- Critical for file I/O (`with open(...) as f:`)
