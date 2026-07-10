# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "asyncio_threads"
# dimension = "perf"
# case = "asyncio_to_thread_scaling"
# subject = "asyncio.to_thread serial-vs-parallel scaling"
# kind = "bench"
# xfail = "timing output is measurement-only; use issue #1186 evidence for scaling"
# mem_carveout = ""
# source = "issue #1186"
# status = "filled"
# ///
# mamba-xfail: timing output is measurement-only; use issue #1186 evidence for scaling
import os as _os
import sys as _sys

_fixture_dir = _os.path.abspath(_os.path.dirname(__file__))
_sys.path = [
    p for p in _sys.path if _os.path.abspath(p or _os.getcwd()) != _fixture_dir
]

import asyncio
import time


TASKS = 4
WORK = 400_000


def cpu_work(seed: int, work: int = WORK) -> int:
    total = 0
    for i in range(work):
        total = (total + ((i ^ seed) * 2654435761)) & 0xFFFFFFFF
    return total


def run_serial() -> tuple[float, list[int]]:
    start = time.perf_counter()
    results = [cpu_work(i) for i in range(TASKS)]
    return time.perf_counter() - start, results


async def run_parallel() -> tuple[float, list[int]]:
    start = time.perf_counter()
    results = await asyncio.gather(
        asyncio.to_thread(cpu_work, 0),
        asyncio.to_thread(cpu_work, 1),
        asyncio.to_thread(cpu_work, 2),
        asyncio.to_thread(cpu_work, 3),
    )
    return time.perf_counter() - start, list(results)


async def main() -> None:
    serial, serial_results = run_serial()
    parallel, parallel_results = await run_parallel()
    assert parallel_results == serial_results
    speedup = serial / parallel if parallel else float("inf")
    print(f"serial={serial:.6f}")
    print(f"parallel={parallel:.6f}")
    print(f"speedup={speedup:.3f}")


if __name__ == "__main__":
    asyncio.run(main())
