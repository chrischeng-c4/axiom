"""Update benchmark."""

import time
from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Update Many")


@group.add("Beanie")
async def beanie_update_many():
    await BeanieUser.find({"age": {"$gte": 30}}).update_many({"$inc": {"age": 1}})


@group.add("data-bridge")
async def db_update_many():
    await DBUser.update_many(DBUser.age >= 30, {"$inc": {"age": 1}})


register_group(group)


# Performance target test for update_many
async def test_update_many_latency_target():
    """
    Verify update_many completes within performance target.

    Success criteria: update_many ≤150ms
    Target: 3.4x faster than baseline (514ms → 150ms)
    Comparison: 1.7x faster than Beanie (265ms)
    """
    # Ensure we have data with age >= 30
    count = await DBUser.count({"age": {"$gte": 30}})
    if count == 0:
        # Insert test data if missing
        from tests.mongo.benchmarks.conftest import generate_user_data
        users_data = generate_user_data(1000)
        for user_data in users_data:
            user = DBUser(**user_data)
            await user.save()

    # Warm up (reset ages first)
    for _ in range(3):
        await DBUser.update_many({"age": {"$gte": 30}}, {"$set": {"age": 35}})

    # Measure latency over 20 iterations (fewer than find_one since update is slower)
    latencies = []
    for i in range(20):
        # Alternate between setting age to 35 and 36 to ensure actual updates
        new_age = 35 if i % 2 == 0 else 36
        start = time.perf_counter()
        await DBUser.update_many({"age": {"$gte": 30}}, {"$set": {"age": new_age}})
        latency = (time.perf_counter() - start) * 1000  # Convert to ms
        latencies.append(latency)

    avg_latency = sum(latencies) / len(latencies)
    p50 = sorted(latencies)[len(latencies) // 2]
    p95 = sorted(latencies)[int(len(latencies) * 0.95)]
    p99 = sorted(latencies)[int(len(latencies) * 0.99)]

    print(f"\n=== Update Many Latency ===")
    print(f"Average: {avg_latency:.2f}ms")
    print(f"P50: {p50:.2f}ms")
    print(f"P95: {p95:.2f}ms")
    print(f"P99: {p99:.2f}ms")

    # Success criteria: Average ≤150ms
    assert avg_latency <= 150, (
        f"update_many latency too high: {avg_latency:.2f}ms "
        f"(target: ≤150ms)"
    )

    print(f"✅ update_many meets performance target ({avg_latency:.2f}ms ≤ 150ms)!")
