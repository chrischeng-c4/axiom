---
number: 755
title: "Py3.12 conformance: Exception hierarchy"
state: open
labels: [enhancement, P1, crate:mamba]
group: "py312-conformance-p1"
---

# #755 — Py3.12 conformance: Exception hierarchy

## Parent

Part of #750

## Goal

Verify mamba exception hierarchy and semantics match CPython 3.12.

## Scope

- [ ] All built-in exception classes present (BaseException tree)
- [ ] `except` matching: subclass catching, tuple of exceptions
- [ ] `raise from` — exception chaining (`__cause__`, `__context__`)
- [ ] `__traceback__` attribute
- [ ] `ExceptionGroup` (PEP 654, Py3.11+)
- [ ] `except*` syntax (PEP 654)
- [ ] `args` attribute on exceptions
- [ ] Custom exception subclassing

## Current State

Basic try/except works. Exception hierarchy completeness and chaining untested.
