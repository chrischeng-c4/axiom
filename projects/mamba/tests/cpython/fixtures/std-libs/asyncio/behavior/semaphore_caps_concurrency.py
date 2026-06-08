# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "semaphore_caps_concurrency"
# subject = "asyncio.Semaphore"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Semaphore not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Semaphore: an asyncio.Semaphore(2) limits the number of coroutines in its critical section to at most 2 at once"""
import asyncio


async def _main():
    _sem = asyncio.Semaphore(2)
    _active = [0]
    _max_active = [0]

    async def _worker():
        async with _sem:
            _active[0] += 1
            if _active[0] > _max_active[0]:
                _max_active[0] = _active[0]
            await asyncio.sleep(0)
            _active[0] -= 1

    await asyncio.gather(*[_worker() for _ in range(5)])
    assert _max_active[0] <= 2, f"max_concurrent={_max_active[0]}"


asyncio.run(_main())

print("semaphore_caps_concurrency OK")
