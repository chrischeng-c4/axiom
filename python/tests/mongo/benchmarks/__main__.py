"""
Benchmark runner: data-bridge vs Beanie (Motor async)

Usage: cd data-bridge && uv run python -m tests.mongo.benchmarks
"""

import asyncio
from pathlib import Path

try:
    import uvloop
    uvloop.install()
except ImportError:
    pass

from cclab import init, close, is_connected
from cclab.probe import discover_benchmarks, run_benchmarks

from beanie import init_beanie
from motor.motor_asyncio import AsyncIOMotorClient

from .models import DBUser, BeanieUser, BEANIE_MODELS

MONGODB_URI = "mongodb://testuser:testpass@localhost:27017/data-bridge-benchmark?authSource=admin"
BENCHMARK_DIR = Path(__file__).parent

motor_client = None


async def setup_connections():
    """Initialize ODM connections."""
    global motor_client

    # data-bridge
    if is_connected():
        await close()
    await init(MONGODB_URI)

    # Beanie (Motor async)
    motor_client = AsyncIOMotorClient(MONGODB_URI)
    await init_beanie(
        database=motor_client["data-bridge-benchmark"],
        document_models=BEANIE_MODELS,
    )


async def teardown_connections():
    """Close all connections."""
    global motor_client
    await close()
    if motor_client:
        motor_client.close()


async def seed_data(count: int = 1000):
    """Seed test data."""
    await DBUser.find().delete()
    await BeanieUser.find_all().delete()

    data = [{"name": f"User{i}", "email": f"u{i}@test.com", "age": 20 + i % 50} for i in range(count)]

    await DBUser.insert_many([DBUser(**d) for d in data])
    await BeanieUser.insert_many([BeanieUser(**d) for d in data])

    print(f"Seeded {count} documents")


async def main():
    print("=" * 70)
    print("data-bridge vs Beanie Benchmark")
    print("=" * 70)

    await setup_connections()

    try:
        await seed_data()

        info = discover_benchmarks(BENCHMARK_DIR)
        print(f"\nDiscovered: {info['files']}")
        print(f"Groups: {info['groups']}")

        report = await run_benchmarks(
            baseline_name="Beanie",
            title="data-bridge vs Beanie",
            description="Rust async ODM vs Python async ODM (Motor)",
        )

        print(report.to_console())

        report.save("benchmark_report", "markdown")
        report.save("benchmark_report", "json")

    finally:
        await teardown_connections()


if __name__ == "__main__":
    asyncio.run(main())
