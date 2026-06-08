#!/usr/bin/env python
"""
Quick verification script for Feature 004: Fast-path Insert

Compares insert_one performance with and without fast-path optimizations.
"""

import asyncio
import time
from statistics import mean, stdev

from cclab import init, close
from cclab.nebula.document import Document


class User(Document):
    name: str
    email: str
    age: int

    class Settings:
        collection_name = "users_test"


async def benchmark_insert(validate: bool, hooks: bool, iterations: int = 100) -> list[float]:
    """Benchmark insert with given parameters."""
    times = []

    for i in range(iterations):
        user = User(
            name=f"User{i}",
            email=f"user{i}@example.com",
            age=25 + (i % 50)
        )

        start = time.perf_counter()
        await user.save(validate=validate, hooks=hooks)
        end = time.perf_counter()

        times.append((end - start) * 1000)  # Convert to milliseconds

    return times


async def main():
    print("=" * 70)
    print("Feature 004: Fast-path Insert Performance Verification")
    print("=" * 70)
    print()

    # Connect to MongoDB
    await init("mongodb://localhost:27017/feature-004-test")

    # Clean up
    await User.find().delete()

    print("Running benchmarks (100 iterations each)...")
    print()

    # Benchmark 1: Standard path (with validation and hooks)
    print("1. Standard save() - validate=True, hooks=True")
    standard_times = await benchmark_insert(validate=True, hooks=True, iterations=100)
    standard_mean = mean(standard_times)
    standard_std = stdev(standard_times) if len(standard_times) > 1 else 0
    print(f"   Mean: {standard_mean:.2f}ms ± {standard_std:.2f}ms")
    print(f"   Min:  {min(standard_times):.2f}ms")
    print(f"   Max:  {max(standard_times):.2f}ms")
    print()

    # Clean up between benchmarks
    await User.find().delete()

    # Benchmark 2: Fast-path (skip validation and hooks)
    print("2. Fast-path save() - validate=False, hooks=False")
    fastpath_times = await benchmark_insert(validate=False, hooks=False, iterations=100)
    fastpath_mean = mean(fastpath_times)
    fastpath_std = stdev(fastpath_times) if len(fastpath_times) > 1 else 0
    print(f"   Mean: {fastpath_mean:.2f}ms ± {fastpath_std:.2f}ms")
    print(f"   Min:  {min(fastpath_times):.2f}ms")
    print(f"   Max:  {max(fastpath_times):.2f}ms")
    print()

    # Calculate speedup
    speedup = standard_mean / fastpath_mean
    print("=" * 70)
    print("RESULTS")
    print("=" * 70)
    print(f"Standard path: {standard_mean:.2f}ms")
    print(f"Fast-path:     {fastpath_mean:.2f}ms")
    print(f"Speedup:       {speedup:.2f}x faster")
    print()

    # Check if target met
    target_speedup = 2.0
    if speedup >= target_speedup:
        print(f"✅ TARGET MET: {speedup:.2f}x >= {target_speedup}x")
    else:
        print(f"⚠️  BELOW TARGET: {speedup:.2f}x < {target_speedup}x")

    print()
    print("Expected targets (from spec.md):")
    print("  - Fast-path should be 2-3x faster than standard")
    print("  - Standard: ~2.4ms → Fast-path: ~0.8ms (3x improvement)")
    print("=" * 70)

    # Clean up
    await User.find().delete()
    await close()


if __name__ == "__main__":
    asyncio.run(main())
