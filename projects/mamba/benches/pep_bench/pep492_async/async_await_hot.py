"""PEP 492 async/await — coroutine dispatch perf bench.

End-user scenario: `async def` + `await` chain driven through
asyncio.run, the canonical event-loop send/drive primitive
that backs every fastapi route, every aiohttp client. CPython
compiles to GET_AWAITABLE + SEND + RETURN_GENERATOR; mamba
lowers the coroutine frame to a state-machine struct the JIT
can sometimes inline.

Bounded context (DDD): pep_bench/pep492_async.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

The inner coroutine performs no I/O — pure CPU await chain —
so the measurement isolates coroutine machinery cost.

Workaround: mamba's parser rejects `await` as part of a larger
expression (e.g. `acc + await inner(i)`); bind to a temp local
first so `await` is the whole RHS.
"""

import asyncio
import sys
import time


async def inner(x):
    return x + 1


async def outer(n):
    acc = 0
    for i in range(n):
        v = await inner(i)
        acc = acc + v
    return acc


N = 1000
ITERS = 100

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = acc + asyncio.run(outer(N))
_t1 = time.perf_counter()

print("async_await_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per outer iter: sum(i+1 for i in range(N)) = N*(N+1)//2.
expected = ITERS * (N * (N + 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
