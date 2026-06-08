# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "queue_preserves_fifo_order"
# subject = "asyncio.Queue"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Queue not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Queue: asyncio.Queue dequeues items in the same order they were enqueued (FIFO: [0,1,2,3,4])"""
import asyncio


async def _main():
    _q = asyncio.Queue()
    for _i in range(5):
        await _q.put(_i)
    _results = []
    while not _q.empty():
        _results.append(await _q.get())
    assert _results == [0, 1, 2, 3, 4], f"FIFO queue: {_results!r}"


asyncio.run(_main())

print("queue_preserves_fifo_order OK")
