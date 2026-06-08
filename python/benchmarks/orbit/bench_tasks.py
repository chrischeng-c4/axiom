"""Task creation and concurrent scaling benchmark - 3-way comparison.

Two benchmark groups:
1. Task Creation & Execution - gather N coroutines (100 tasks)
2. Concurrent Task Scaling - 500 concurrent tasks with simulated work

Note: Orbit's PyLoop only supports low-level callback APIs (call_soon,
call_later, run_forever, stop). It does not implement run_until_complete
or integrate with asyncio.gather. These benchmarks compare uvloop vs
asyncio only. Orbit is tested in bench_call_soon.py and bench_timers.py.
"""

from __future__ import annotations

import asyncio

from cclab.probe import BenchmarkGroup, register_group

from ._helpers import has_uvloop

# Backends that support full asyncio protocol (run_until_complete + gather)
TASK_BACKENDS = ["uvloop", "asyncio"] if has_uvloop() else ["asyncio"]

# ---------------------------------------------------------------------------
# Group 1: Task Creation & Execution
# ---------------------------------------------------------------------------

TASK_COUNT = 100

creation_group = BenchmarkGroup("Task Creation & Execution")


def _make_creation_bench(backend: str):
    """Return a benchmark function for *backend*."""

    def bench():
        if backend == "uvloop":
            import uvloop
            loop = uvloop.new_event_loop()
        else:
            loop = asyncio.new_event_loop()

        async def noop():
            pass

        async def run():
            await asyncio.gather(*[noop() for _ in range(TASK_COUNT)])

        loop.run_until_complete(run())
        loop.close()

    return bench


for _backend in TASK_BACKENDS:
    creation_group.add(_backend)(_make_creation_bench(_backend))

register_group(creation_group)

# ---------------------------------------------------------------------------
# Group 2: Concurrent Task Scaling
# ---------------------------------------------------------------------------

CONCURRENT_TASKS = 500

scaling_group = BenchmarkGroup("Concurrent Task Scaling")


def _make_scaling_bench(backend: str):
    """Return a benchmark function for *backend*."""

    def bench():
        if backend == "uvloop":
            import uvloop
            loop = uvloop.new_event_loop()
        else:
            loop = asyncio.new_event_loop()

        async def work():
            total = 0
            for i in range(100):
                total += i
            return total

        async def run():
            await asyncio.gather(*[work() for _ in range(CONCURRENT_TASKS)])

        loop.run_until_complete(run())
        loop.close()

    return bench


for _backend in TASK_BACKENDS:
    scaling_group.add(_backend)(_make_scaling_bench(_backend))

register_group(scaling_group)
