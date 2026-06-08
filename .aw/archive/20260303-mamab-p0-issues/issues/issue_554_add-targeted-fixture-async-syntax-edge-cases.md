---
number: 554
title: "Add targeted fixture: async syntax edge cases"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #554 — Add targeted fixture: async syntax edge cases

## Context
Async syntax is complex. Current `async_await.py` is only 5 lines.

## Test cases to cover
- `async def` with various signatures
- `await` in various expression positions
- `async for` loops
- `async with` statements
- `async with` multiple context managers
- Async comprehensions: `[x async for x in aiter]`
- Async generator: `async def f(): yield x`
- Nested async: async inside sync inside async
- `async def` with decorators
- `async def` with type annotations
- Mixing `yield` and `await` in async generators

## Task
Create `tests/fixtures/parse/edge_cases/async_advanced.py` with `# RUN: parse`.
