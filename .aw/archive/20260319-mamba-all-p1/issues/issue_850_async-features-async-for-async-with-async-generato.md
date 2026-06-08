---
number: 850
title: "Async features — async for, async with, async generators"
state: open
labels: [enhancement, P1, crate:mamba]
group: "async-generators"
---

# #850 — Async features — async for, async with, async generators

## Summary

Complete async/await feature set beyond basic coroutines:

```python
async def fetch_all(urls):
    async with aiohttp.ClientSession() as session:  # async context manager
        async for chunk in response.content:          # async iteration
            yield chunk                                # async generator
```

## Scope

- **async for**: `__aiter__` / `__anext__` protocol
- **async with**: `__aenter__` / `__aexit__` protocol
- **async generators**: `yield` inside `async def` → AsyncGenerator type
- **async comprehensions**: `[x async for x in aiter]`
- **Runtime**: Integrate with existing tokio_exec.rs async runtime
