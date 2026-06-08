---
number: 756
title: "Py3.12 conformance: Generator & iterator protocol"
state: closed
labels: [type:enhancement, priority:p0, crate:mamba]
---

# #756 — Py3.12 conformance: Generator & iterator protocol

## Parent

Part of #750

## Goal

Verify generator and iterator protocol matches CPython 3.12.

## Scope

- [ ] `yield` / `yield from` semantics
- [ ] `generator.send(value)` — resume with value
- [ ] `generator.throw(exc)` — inject exception
- [ ] `generator.close()` — GeneratorExit handling
- [ ] `StopIteration.value` for return from generator
- [ ] Async generators: `async for`, `async yield`
- [ ] Iterator protocol: `__iter__`, `__next__`, StopIteration
- [ ] `itertools`-compatible iteration behavior

## Current State

Basic yield works. send/throw/close and async generators need conformance testing.
