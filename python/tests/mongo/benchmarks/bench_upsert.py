"""Upsert benchmark."""

import time
from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Upsert")


@group.add("Beanie")
async def beanie_upsert():
    await BeanieUser.find_one({"email": "bench_upsert@example.com"}).upsert(
        {"$set": {"age": 25, "name": "BenchUser"}},
        on_insert=BeanieUser(email="bench_upsert@example.com", age=25, name="BenchUser")
    )


@group.add("data-bridge")
async def db_upsert():
    # Upsert: update if exists, insert if not
    await DBUser.find(DBUser.email == "bench_upsert@example.com").upsert(
        {"$set": {"age": 25, "name": "BenchUser"}}
    )


register_group(group)


# T039: Latency target test
async def test_upsert_latency_target():
    """
    Verify upsert completes within performance target.

    Success criteria: upsert ≤5ms
    Target: Fast atomic update-or-insert operation
    """
    # Clean up any existing test data
    await DBUser.delete_many(DBUser.email == "upsert_test@example.com")

    # Warm up
    for _ in range(5):
        await DBUser.find(DBUser.email == "upsert_test@example.com").upsert(
            {"$set": {"age": 25, "name": "Test"}}
        )

    # Measure latency over 50 iterations (alternating insert and update)
    latencies = []
    for i in range(50):
        # Delete every other iteration to alternate insert/update
        if i % 2 == 0:
            await DBUser.delete_many(DBUser.email == "upsert_test@example.com")

        start = time.perf_counter()
        await DBUser.find(DBUser.email == "upsert_test@example.com").upsert(
            {"$set": {"age": 25 + i, "name": "Test"}}
        )
        latency = (time.perf_counter() - start) * 1000  # Convert to ms
        latencies.append(latency)

    avg_latency = sum(latencies) / len(latencies)
    p50 = sorted(latencies)[len(latencies) // 2]
    p95 = sorted(latencies)[int(len(latencies) * 0.95)]
    p99 = sorted(latencies)[int(len(latencies) * 0.99)]

    print(f"\n=== Upsert Latency ===")
    print(f"Average: {avg_latency:.2f}ms")
    print(f"P50: {p50:.2f}ms")
    print(f"P95: {p95:.2f}ms")
    print(f"P99: {p99:.2f}ms")

    # Success criteria: Average ≤5ms
    assert avg_latency <= 5.0, (
        f"upsert latency too high: {avg_latency:.2f}ms "
        f"(target: ≤5ms)"
    )

    print(f"✅ upsert meets performance target ({avg_latency:.2f}ms ≤ 5ms)!")

    # Clean up
    await DBUser.delete_many(DBUser.email == "upsert_test@example.com")
