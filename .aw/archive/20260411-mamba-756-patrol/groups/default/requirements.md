---
change: mamba-756-patrol
group: default
date: 2026-04-11
source: structured-issue
---

# Requirements

## Problem

Generators "work" in Mamba in the sense that `def f(): yield 1` produces a
value, but the protocol has many moving parts that are not yet verified:
bidirectional communication via `send()`, exception injection via
`throw()`, cleanup via `close()` and `GeneratorExit`, the `StopIteration.value`
propagation from `return` inside a generator, `yield from` delegation, and
the iterator protocol itself (`__iter__`, `__next__`, `StopIteration`
termination). Async generators (`async def` + `yield`) are a second,
orthogonal protocol with its own driving loop.

This issue is **independent of the perf rewrite** (#1187). The coroutine
rewrite must not break these semantics; the conformance test suite built
here is the gate that proves it.

Reference: PEP 255 (simple generators), PEP 342 (coroutines via enhanced
generators), PEP 380 (`yield from`), PEP 525 (async generators),
CPython 3.12 `Lib/test/test_generators.py`, `test_coroutines.py`.

## Requirements

