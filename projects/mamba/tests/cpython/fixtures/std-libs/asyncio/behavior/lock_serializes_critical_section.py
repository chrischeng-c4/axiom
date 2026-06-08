# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "lock_serializes_critical_section"
# subject = "asyncio.Lock"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Lock not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Lock: an asyncio.Lock used as an async context manager guarantees only one coroutine is inside the critical section at a time"""
import asyncio


async def _main():
    _lock = asyncio.Lock()
    _inside = []

    async def _worker(n):
        async with _lock:
            _inside.append(n)
            await asyncio.sleep(0)
            assert len(_inside) == 1, f"only one inside: {_inside!r}"
            _inside.pop()

    await asyncio.gather(_worker(1), _worker(2), _worker(3))


asyncio.run(_main())

print("lock_serializes_critical_section OK")
