---
number: 413
title: "mamba: contextlib module"
state: open
labels: [enhancement, P2, crate:mamba]
dependencies: [385]
---

# #413 — mamba: contextlib module

## Description

Implement `contextlib` — utilities for context managers. Commonly used alongside `with` statements.

## Requirements

- R1: `@contextmanager` decorator — create context managers from generators
- R2: `closing(thing)` — call `thing.close()` on exit
- R3: `suppress(*exceptions)` — suppress specified exceptions
- R4: `redirect_stdout` / `redirect_stderr`
- R5: `nullcontext(enter_result=None)` — no-op context manager
- R6: `ExitStack` — programmatic context manager combination

## Dependencies

Depends on #385 (context manager protocol).

## Priority

P2 — commonly used with context managers.
