# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "real_world"
# case = "async_pipeline_walkthrough"
# subject = "asyncio"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Queue / concurrent gather coordination not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio: an end-user async pipeline: asyncio.run drives a main that gather-fans-out worker coroutines, feeds results through an asyncio.Queue, and aggregates a deterministic total"""
import asyncio


async def _main():
    # Producers compute work items and hand them off through a shared queue;
    # a single consumer drains the queue and aggregates a deterministic total.
    _q = asyncio.Queue()
    _n_workers = 5
    _per_worker = 10

    async def _producer(worker_id):
        for i in range(_per_worker):
            await _q.put(worker_id * 100 + i)
            await asyncio.sleep(0)  # yield so producers interleave

    async def _consumer():
        _seen = []
        _expected = _n_workers * _per_worker
        while len(_seen) < _expected:
            _seen.append(await _q.get())
        return _seen

    _consumer_task = asyncio.create_task(_consumer())
    await asyncio.gather(*[_producer(w) for w in range(_n_workers)])
    _seen = await _consumer_task

    assert len(_seen) == _n_workers * _per_worker, f"drained {len(_seen)} items"
    _total = sum(_seen)
    # Each item is worker*100 + i; total is order-independent so it is stable
    # regardless of how the producers interleaved.
    _expected_total = sum(
        w * 100 + i for w in range(_n_workers) for i in range(_per_worker)
    )
    assert _total == _expected_total, f"total {_total} != {_expected_total}"
    return _total


_result = asyncio.run(_main())
assert _result == 10225, f"pipeline total = {_result!r}"

print("async_pipeline_walkthrough OK")
