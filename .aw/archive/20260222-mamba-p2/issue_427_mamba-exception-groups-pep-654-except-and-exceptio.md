---
number: 427
title: "mamba: exception groups (PEP 654) — except* and ExceptionGroup"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #427 — mamba: exception groups (PEP 654) — except* and ExceptionGroup

## Description

Implement exception groups from PEP 654 (Python 3.11+). Used by asyncio and modern error handling.

## Requirements

- R1: `ExceptionGroup(message, exceptions)` type
- R2: `except*` syntax for matching exception groups
- R3: `BaseExceptionGroup` base class
- R4: `.exceptions` attribute — tuple of contained exceptions
- R5: `.subgroup(condition)` — filter exceptions
- R6: `.derive(excs)` — create new group with same message
- R7: Nesting support — groups can contain other groups

## Priority

P2 — Python 3.11+ feature, increasingly used with asyncio.
