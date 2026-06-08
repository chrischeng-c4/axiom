"""Count benchmark."""

import time
from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Count")


@group.add("Beanie")
async def beanie_count():
    return await BeanieUser.find({"age": {"$gte": 30}}).count()


@group.add("data-bridge")
async def db_count():
    return await DBUser.count({"age": {"$gte": 30}})


register_group(group)


# T037: Latency target test
async def test_count_latency_target():
    """
    Verify count completes within performance target.

    Success criteria: count ≤2ms
    Target: Fast count operation with index support
    """
    # Ensure we have data
    count = await DBUser.count({})
    if count == 0:
        from tests.mongo.benchmarks.conftest import generate_user_data
        users_data = generate_user_data(1000)
        for user_data in users_data:
            user = DBUser(**user_data)
            await user.save()

    # Warm up
    for _ in range(5):
        await DBUser.count({"age": {"$gte": 30}})

    # Measure latency over 100 iterations
    latencies = []
    for _ in range(100):
        start = time.perf_counter()
        await DBUser.count({"age": {"$gte": 30}})
        latency = (time.perf_counter() - start) * 1000  # Convert to ms
        latencies.append(latency)

    avg_latency = sum(latencies) / len(latencies)
    p50 = sorted(latencies)[len(latencies) // 2]
    p95 = sorted(latencies)[int(len(latencies) * 0.95)]
    p99 = sorted(latencies)[int(len(latencies) * 0.99)]

    print(f"\n=== Count Latency ===")
    print(f"Average: {avg_latency:.2f}ms")
    print(f"P50: {p50:.2f}ms")
    print(f"P95: {p95:.2f}ms")
    print(f"P99: {p99:.2f}ms")

    # Success criteria: Average ≤2ms
    assert avg_latency <= 2.0, (
        f"count latency too high: {avg_latency:.2f}ms "
        f"(target: ≤2ms)"
    )

    print(f"✅ count meets performance target ({avg_latency:.2f}ms ≤ 2ms)!")
