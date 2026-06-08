---
change: mamba-all-p1
group: async-generators
date: 2026-03-19
---

# Requirements

Implement complete generator/iterator and async-iteration semantics:
- #850 Async features: `async for` (`__aiter__`/`__anext__` protocol), `async with` (`__aenter__`/`__aexit__` protocol), async generators (`yield` inside `async def`), async comprehensions; integrate with existing tokio_exec.rs runtime
- #756 Generator & iterator conformance (CPython 3.12): `yield`/`yield from` semantics, `generator.send(value)`, `generator.throw(exc)`, `generator.close()` + GeneratorExit, `StopIteration.value`, async generators, iterator protocol (`__iter__`/`__next__`/StopIteration), itertools-compatible behavior
Acceptance: CPython 3.12 generator conformance tests pass; async for/with/generators run correctly against the tokio runtime.
