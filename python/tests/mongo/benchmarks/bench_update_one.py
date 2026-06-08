"""Update One benchmark."""

import time
from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Update One")


@group.add("Beanie")
async def beanie_update_one():
    user = await BeanieUser.find_one({"age": 30})
    if user:
        user.age += 1
        await user.save()


@group.add("data-bridge")
async def db_update_one():
    # Find and update single document
    user = await DBUser.find_one(DBUser.age == 30)
    if user:
        user.age += 1
        await user.save()


register_group(group)


# T036: Latency target test
async def test_update_one_latency_target():
    """
    Verify update_one completes within performance target.

    Success criteria: update_one ≤5ms
    Target: 2x faster than baseline (10ms → 5ms)
    """
    # Ensure we have data with age=30
    count = await DBUser.count({"age": 30})
    if count == 0:
        from tests.mongo.benchmarks.conftest import generate_user_data
        users_data = generate_user_data(1000)
        for user_data in users_data:
            user = DBUser(**user_data)
            await user.save()

    # Warm up
    for _ in range(5):
        user = await DBUser.find_one(DBUser.age == 30)
        if user:
            user.age = 30  # Reset to 30 each time
            await user.save()

    # Measure latency over 50 iterations
    latencies = []
    for i in range(50):
        start = time.perf_counter()
        user = await DBUser.find_one(DBUser.age == 30)
        if user:
            user.age = 30 if i % 2 == 0 else 31
            await user.save()
        latency = (time.perf_counter() - start) * 1000  # Convert to ms
        latencies.append(latency)

    avg_latency = sum(latencies) / len(latencies)
    p50 = sorted(latencies)[len(latencies) // 2]
    p95 = sorted(latencies)[int(len(latencies) * 0.95)]
    p99 = sorted(latencies)[int(len(latencies) * 0.99)]

    print(f"\n=== Update One Latency ===")
    print(f"Average: {avg_latency:.2f}ms")
    print(f"P50: {p50:.2f}ms")
    print(f"P95: {p95:.2f}ms")
    print(f"P99: {p99:.2f}ms")

    # Success criteria: Average ≤5ms
    assert avg_latency <= 5.0, (
        f"update_one latency too high: {avg_latency:.2f}ms "
        f"(target: ≤5ms)"
    )

    print(f"✅ update_one meets performance target ({avg_latency:.2f}ms ≤ 5ms)!")
