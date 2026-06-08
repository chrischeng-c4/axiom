---
number: 517
title: "Add CPython 3.12 async fixtures (test_coroutines, test_asyncgen, test_contextlib_async)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #517 — Add CPython 3.12 async fixtures (test_coroutines, test_asyncgen, test_contextlib_async)

## Context
Async syntax (`async def`, `await`, `async for`, `async with`, async generators) is complex and has many edge cases. Current mamba coverage: only `async_await.py` (5 lines).

## Files
- `test_coroutines.py` — async/await syntax, coroutine protocol
- `test_asyncgen.py` — async generator functions
- `test_contextlib_async.py` — async context managers

## Task
For each file:
1. Download from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/`

## Acceptance
- All 3 fixtures parse without errors
- `cargo test --test fixture_tests` passes
