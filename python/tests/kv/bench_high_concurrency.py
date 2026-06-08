#!/usr/bin/env python3
"""
High-concurrency benchmark for data-bridge-kv

Tests:
- 100 concurrent clients
- 10,000 ops per client = 1M total ops
- Mixed workload (50% GET, 50% SET)
"""

import asyncio
import time
import sys
import statistics

# Add project to path
sys.path.insert(0, "/Users/chris.cheng/chris-project/data-bridge/python")

async def run_benchmark():
    from cclab.ion import KvClient, KvPool, PoolConfig

    SERVER = "127.0.0.1:16380"

    print("=" * 60)
    print("High-Concurrency Benchmark: data-bridge-kv")
    print("=" * 60)

    # Test 1: Single client baseline
    print("\n[Test 1] Single Client Baseline")
    print("-" * 40)
    client = await KvClient.connect(f"{SERVER}/bench1")

    ops = 10000
    start = time.perf_counter()
    for i in range(ops):
        await client.set(f"key:{i}", f"value{i}")
    elapsed = time.perf_counter() - start
    single_set_ops = ops / elapsed
    print(f"  SET: {single_set_ops:,.0f} ops/sec")

    start = time.perf_counter()
    for i in range(ops):
        await client.get(f"key:{i % 1000}")
    elapsed = time.perf_counter() - start
    single_get_ops = ops / elapsed
    print(f"  GET: {single_get_ops:,.0f} ops/sec")

    # Test 2: High concurrency with pool
    print("\n[Test 2] Pool - 100 Concurrent Workers")
    print("-" * 40)

    pool = await KvPool.connect(PoolConfig(
        f"{SERVER}/bench2",
        min_size=20,
        max_size=100,
    ))

    workers = 100
    ops_per_worker = 10000
    total_ops = workers * ops_per_worker

    async def worker_set(worker_id: int, ops: int):
        for i in range(ops):
            await pool.set(f"w{worker_id}:k{i}", f"value{i}")

    async def worker_get(worker_id: int, ops: int):
        for i in range(ops):
            await pool.get(f"w{worker_id}:k{i % 100}")

    async def worker_mixed(worker_id: int, ops: int):
        for i in range(ops):
            if i % 2 == 0:
                await pool.set(f"w{worker_id}:m{i}", f"value{i}")
            else:
                await pool.get(f"w{worker_id}:m{i % 100}")

    # SET benchmark
    start = time.perf_counter()
    await asyncio.gather(*[worker_set(w, ops_per_worker) for w in range(workers)])
    elapsed = time.perf_counter() - start
    set_ops_sec = total_ops / elapsed
    print(f"  SET: {set_ops_sec:,.0f} ops/sec ({total_ops:,} ops in {elapsed:.2f}s)")

    # GET benchmark
    start = time.perf_counter()
    await asyncio.gather(*[worker_get(w, ops_per_worker) for w in range(workers)])
    elapsed = time.perf_counter() - start
    get_ops_sec = total_ops / elapsed
    print(f"  GET: {get_ops_sec:,.0f} ops/sec ({total_ops:,} ops in {elapsed:.2f}s)")

    # Mixed benchmark
    start = time.perf_counter()
    await asyncio.gather(*[worker_mixed(w, ops_per_worker) for w in range(workers)])
    elapsed = time.perf_counter() - start
    mixed_ops_sec = total_ops / elapsed
    print(f"  Mixed: {mixed_ops_sec:,.0f} ops/sec ({total_ops:,} ops in {elapsed:.2f}s)")

    # Test 3: Scalability test
    print("\n[Test 3] Scalability (varying workers)")
    print("-" * 40)

    worker_counts = [1, 10, 50, 100, 200]
    ops_per_worker_scale = 5000

    results = []
    for w in worker_counts:
        pool2 = await KvPool.connect(PoolConfig(
            f"{SERVER}/bench3_{w}",
            min_size=min(w, 20),
            max_size=max(w, 50),
        ))

        total = w * ops_per_worker_scale
        start = time.perf_counter()
        await asyncio.gather(*[
            worker_mixed_simple(pool2, wid, ops_per_worker_scale)
            for wid in range(w)
        ])
        elapsed = time.perf_counter() - start
        ops_sec = total / elapsed
        results.append((w, ops_sec))
        print(f"  {w:3d} workers: {ops_sec:>12,.0f} ops/sec")

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Single client SET:     {single_set_ops:>12,.0f} ops/sec")
    print(f"Single client GET:     {single_get_ops:>12,.0f} ops/sec")
    print(f"100 workers SET:       {set_ops_sec:>12,.0f} ops/sec")
    print(f"100 workers GET:       {get_ops_sec:>12,.0f} ops/sec")
    print(f"100 workers Mixed:     {mixed_ops_sec:>12,.0f} ops/sec")
    print(f"\nScalability factor: {results[-1][1] / results[0][1]:.1f}x (1 → {worker_counts[-1]} workers)")
    print("=" * 60)

async def worker_mixed_simple(pool, worker_id: int, ops: int):
    for i in range(ops):
        if i % 2 == 0:
            await pool.set(f"s{worker_id}:k{i}", f"v{i}")
        else:
            await pool.get(f"s{worker_id}:k{i % 100}")

if __name__ == "__main__":
    asyncio.run(run_benchmark())
