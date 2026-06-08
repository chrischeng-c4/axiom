"""Find One benchmark."""

import time
from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Find One")


@group.add("Beanie")
async def beanie_find_one():
    return await BeanieUser.find_one({"age": 35})


@group.add("data-bridge")
async def db_find_one():
    return await DBUser.find_one(DBUser.age == 35)


register_group(group)


# T035: Latency target test
async def test_find_one_latency_target():
    """
    Verify find_one completes within performance target.

    Success criteria (FR-005): find_one ≤3.5ms
    Target: 2.5x faster than baseline (8.9ms → 3.5ms)
    """
    # Warm up
    for _ in range(5):
        await DBUser.find_one(DBUser.age == 35)

    # Measure latency over 100 iterations
    latencies = []
    for _ in range(100):
        start = time.perf_counter()
        await DBUser.find_one(DBUser.age == 35)
        latency = (time.perf_counter() - start) * 1000  # Convert to ms
        latencies.append(latency)

    avg_latency = sum(latencies) / len(latencies)
    p50 = sorted(latencies)[len(latencies) // 2]
    p95 = sorted(latencies)[int(len(latencies) * 0.95)]
    p99 = sorted(latencies)[int(len(latencies) * 0.99)]

    print(f"\n=== Find One Latency ===")
    print(f"Average: {avg_latency:.2f}ms")
    print(f"P50: {p50:.2f}ms")
    print(f"P95: {p95:.2f}ms")
    print(f"P99: {p99:.2f}ms")

    # Success criteria: Average ≤3.5ms
    assert avg_latency <= 3.5, (
        f"find_one latency too high: {avg_latency:.2f}ms "
        f"(target: ≤3.5ms)"
    )

    print(f"✅ find_one meets performance target ({avg_latency:.2f}ms ≤ 3.5ms)!")
