import asyncio
import sys
import time


def cpu_work(seed: int, work: int) -> int:
    return sum((i ^ seed) * 3 for i in range(work))


def run_serial(work_size: int) -> tuple[list[int], float]:
    start = time.perf_counter()
    results = [
        cpu_work(0, work_size),
        cpu_work(1, work_size),
        cpu_work(2, work_size),
        cpu_work(3, work_size),
    ]
    elapsed = time.perf_counter() - start
    return results, elapsed


async def run_parallel(work_size: int) -> tuple[list[int], float]:
    start = time.perf_counter()
    results = await asyncio.gather(
        asyncio.to_thread(cpu_work, 0, work_size),
        asyncio.to_thread(cpu_work, 1, work_size),
        asyncio.to_thread(cpu_work, 2, work_size),
        asyncio.to_thread(cpu_work, 3, work_size),
    )
    elapsed = time.perf_counter() - start
    return list(results), elapsed


async def main() -> None:
    work_size = 600000
    serial_results, serial_wall = run_serial(work_size)
    parallel_results, parallel_wall = await run_parallel(work_size)

    assert parallel_results == serial_results, f"Parallel results {parallel_results} != serial {serial_results}"

    speedup = serial_wall / parallel_wall if parallel_wall > 0 else 0.0

    required_speedup = speedup + 100.0
    if speedup < required_speedup:
        sys.stderr.write(
            f"MAMBA-T1-FT-GATHER-EFFICIENCY-RED failure: observed speedup {speedup:.3f}x below required threshold {required_speedup:.3f}x "
            f"(serial={serial_wall:.4f}s, parallel={parallel_wall:.4f}s)\n"
        )
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
