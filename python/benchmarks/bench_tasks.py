#!/usr/bin/env python3
"""
Benchmark: data-bridge-tasks vs Celery

Usage:
    # Run data-bridge-tasks benchmark
    python benchmarks/bench_tasks.py --backend data-bridge

    # Run Celery benchmark
    python benchmarks/bench_tasks.py --backend celery

    # Run comparison
    python benchmarks/bench_tasks.py --compare
"""

import asyncio
import time
import argparse
import statistics
from typing import List, Tuple

# Benchmark configuration
NUM_TASKS = 1000
BATCH_SIZES = [1, 10, 100, 1000]
WARMUP_TASKS = 100


async def benchmark_cclab():
    """Benchmark data-bridge-tasks"""
    from cclab.swarm import task, init, AsyncResult

    # Initialize
    await init("nats://localhost:4222", "redis://localhost:6379")

    @task(name="bench_add")
    async def add(x: int, y: int) -> int:
        return x + y

    results = {}

    # Warmup
    print("  Warming up...")
    for _ in range(WARMUP_TASKS):
        await add.delay(1, 2)

    # Benchmark task submission
    for batch_size in BATCH_SIZES:
        print(f"  Benchmarking batch size: {batch_size}")
        times = []
        for _ in range(10):  # 10 iterations per batch size
            start = time.perf_counter()
            tasks = [add.delay(i, i) for i in range(batch_size)]
            await asyncio.gather(*tasks)
            elapsed = time.perf_counter() - start
            times.append(elapsed)

        avg_time = statistics.mean(times)
        ops_per_sec = batch_size / avg_time
        results[f"submit_{batch_size}"] = {
            "avg_time_ms": avg_time * 1000,
            "ops_per_sec": ops_per_sec,
        }

    return results


def benchmark_celery():
    """Benchmark Celery (synchronous)"""
    from celery import Celery

    app = Celery('benchmark', broker='redis://localhost:6379')

    @app.task
    def add(x: int, y: int) -> int:
        return x + y

    results = {}

    # Warmup
    print("  Warming up...")
    for _ in range(WARMUP_TASKS):
        add.delay(1, 2)

    # Benchmark
    for batch_size in BATCH_SIZES:
        print(f"  Benchmarking batch size: {batch_size}")
        times = []
        for _ in range(10):
            start = time.perf_counter()
            tasks = [add.delay(i, i) for i in range(batch_size)]
            elapsed = time.perf_counter() - start
            times.append(elapsed)

        avg_time = statistics.mean(times)
        ops_per_sec = batch_size / avg_time
        results[f"submit_{batch_size}"] = {
            "avg_time_ms": avg_time * 1000,
            "ops_per_sec": ops_per_sec,
        }

    return results


def print_results(name: str, results: dict):
    """Print benchmark results"""
    print(f"\n{'=' * 60}")
    print(f"  {name} Benchmark Results")
    print(f"{'=' * 60}")

    for key, value in results.items():
        print(f"\n  {key}:")
        print(f"    Average Time: {value['avg_time_ms']:.2f} ms")
        print(f"    Throughput:   {value['ops_per_sec']:.0f} ops/sec")


def compare_results(db_results: dict, celery_results: dict):
    """Compare and print comparison"""
    print(f"\n{'=' * 60}")
    print(f"  Comparison: data-bridge-tasks vs Celery")
    print(f"{'=' * 60}")

    print(f"\n  {'Metric':<20} {'data-bridge':<15} {'Celery':<15} {'Speedup':<10}")
    print(f"  {'-' * 60}")

    for key in db_results:
        db_ops = db_results[key]["ops_per_sec"]
        celery_ops = celery_results.get(key, {}).get("ops_per_sec", 1)
        speedup = db_ops / celery_ops

        print(f"  {key:<20} {db_ops:>12.0f}/s {celery_ops:>12.0f}/s {speedup:>8.1f}x")


async def main():
    parser = argparse.ArgumentParser(description="Task queue benchmark")
    parser.add_argument("--backend", choices=["data-bridge", "celery", "both"], default="both")
    parser.add_argument("--compare", action="store_true")
    args = parser.parse_args()

    db_results = None
    celery_results = None

    if args.backend in ["data-bridge", "both"]:
        print("\nRunning data-bridge-tasks benchmark...")
        db_results = await benchmark_cclab()
        print_results("data-bridge-tasks", db_results)

    if args.backend in ["celery", "both"]:
        print("\nRunning Celery benchmark...")
        celery_results = benchmark_celery()
        print_results("Celery", celery_results)

    if args.compare or args.backend == "both":
        if db_results and celery_results:
            compare_results(db_results, celery_results)


if __name__ == "__main__":
    asyncio.run(main())
