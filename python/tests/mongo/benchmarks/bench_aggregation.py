"""Aggregation benchmark."""

import time
from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Aggregation")


@group.add("data-bridge")
async def db_aggregate():
    # Group by age and count
    pipeline = [
        {"$group": {"_id": "$age", "count": {"$sum": 1}}},
        {"$sort": {"count": -1}},
        {"$limit": 10}
    ]
    return await DBUser.aggregate(pipeline).to_list()


# NOTE: Beanie's aggregate has issues with Motor cursor - skipping for now
# @group.add("Beanie")
# async def beanie_aggregate():
#     pipeline = [
#         {"$group": {"_id": "$age", "count": {"$sum": 1}}},
#         {"$sort": {"count": -1}},
#         {"$limit": 10}
#     ]
#     cursor = BeanieUser.aggregate(pipeline)
#     return await cursor.to_list()


register_group(group)


# T038: Latency target test
async def test_aggregation_latency_target():
    """
    Verify aggregation completes within performance target.

    Success criteria: aggregation ≤10ms
    Target: Fast aggregation with group and sort operations
    """
    # Ensure we have data
    count = await DBUser.count({})
    if count == 0:
        from tests.mongo.benchmarks.conftest import generate_user_data
        users_data = generate_user_data(1000)
        for user_data in users_data:
            user = DBUser(**user_data)
            await user.save()

    pipeline = [
        {"$group": {"_id": "$age", "count": {"$sum": 1}}},
        {"$sort": {"count": -1}},
        {"$limit": 10}
    ]

    # Warm up
    for _ in range(5):
        await DBUser.aggregate(pipeline).to_list()

    # Measure latency over 50 iterations
    latencies = []
    for _ in range(50):
        start = time.perf_counter()
        await DBUser.aggregate(pipeline).to_list()
        latency = (time.perf_counter() - start) * 1000  # Convert to ms
        latencies.append(latency)

    avg_latency = sum(latencies) / len(latencies)
    p50 = sorted(latencies)[len(latencies) // 2]
    p95 = sorted(latencies)[int(len(latencies) * 0.95)]
    p99 = sorted(latencies)[int(len(latencies) * 0.99)]

    print(f"\n=== Aggregation Latency ===")
    print(f"Average: {avg_latency:.2f}ms")
    print(f"P50: {p50:.2f}ms")
    print(f"P95: {p95:.2f}ms")
    print(f"P99: {p99:.2f}ms")

    # Success criteria: Average ≤10ms
    assert avg_latency <= 10.0, (
        f"aggregation latency too high: {avg_latency:.2f}ms "
        f"(target: ≤10ms)"
    )

    print(f"✅ aggregation meets performance target ({avg_latency:.2f}ms ≤ 10ms)!")
