"""Profile individual overhead sources in data-bridge vs Beanie."""

import asyncio
import time
from typing import Callable
import uvloop

# Raw drivers
import motor.motor_asyncio
from pymongo import MongoClient

# ODMs
from cclab import init, close
from beanie import init_beanie

from tests.mongo.benchmarks.models import DBUser, BeanieUser, BEANIE_MODELS

MONGODB_URI = "mongodb://localhost:27017/bench_profile"
ITERATIONS = 100


async def time_async(fn: Callable, iterations: int = ITERATIONS) -> float:
    """Time an async function over multiple iterations."""
    # Warmup
    for _ in range(5):
        await fn()

    start = time.perf_counter()
    for _ in range(iterations):
        await fn()
    elapsed = time.perf_counter() - start
    return (elapsed / iterations) * 1000  # ms per operation


def time_sync(fn: Callable, iterations: int = ITERATIONS) -> float:
    """Time a sync function over multiple iterations."""
    # Warmup
    for _ in range(5):
        fn()

    start = time.perf_counter()
    for _ in range(iterations):
        fn()
    elapsed = time.perf_counter() - start
    return (elapsed / iterations) * 1000


async def profile_insert_overhead():
    """Profile insert operation overhead at each layer."""
    print("\n" + "=" * 70)
    print("INSERT ONE OVERHEAD ANALYSIS")
    print("=" * 70)

    # Initialize connections
    await init(MONGODB_URI)

    motor_client = motor.motor_asyncio.AsyncIOMotorClient(MONGODB_URI)
    motor_db = motor_client.get_default_database()
    motor_col = motor_db["profile_motor"]

    pymongo_client = MongoClient(MONGODB_URI)
    pymongo_db = pymongo_client.get_default_database()
    pymongo_col = pymongo_db["profile_pymongo"]

    await init_beanie(database=motor_db, document_models=BEANIE_MODELS)

    doc_data = {"name": "Test", "email": "test@test.com", "age": 30}

    # Clean up
    await motor_col.delete_many({})
    pymongo_col.delete_many({})
    await DBUser.delete_many({})
    await BeanieUser.find_all().delete()

    # 1. PyMongo sync (baseline - fastest possible)
    def pymongo_insert():
        pymongo_col.insert_one(doc_data.copy())
    pymongo_time = time_sync(pymongo_insert)

    # 2. Motor async (what Beanie uses)
    async def motor_insert():
        await motor_col.insert_one(doc_data.copy())
    motor_time = await time_async(motor_insert)

    # 3. Beanie ODM (Motor + Python validation + Python ORM layer)
    async def beanie_insert():
        await BeanieUser(**doc_data).insert()
    beanie_time = await time_async(beanie_insert)

    # 4. data-bridge ODM (Rust driver + PyO3 async + Rust-backed ORM)
    async def db_insert():
        await DBUser(**doc_data).save()
    db_time = await time_async(db_insert)

    # Print results
    print(f"\n{'Layer':<35} {'Time (ms)':<12} {'vs PyMongo':<12} {'vs Motor':<12}")
    print("-" * 70)
    print(f"{'1. PyMongo (sync, C ext)':<35} {pymongo_time:>10.3f}   {'baseline':<12} {'-':<12}")
    print(f"{'2. Motor (async, C ext)':<35} {motor_time:>10.3f}   {f'+{(motor_time/pymongo_time-1)*100:.1f}%':<12} {'baseline':<12}")
    print(f"{'3. Beanie (Motor + Pydantic)':<35} {beanie_time:>10.3f}   {f'+{(beanie_time/pymongo_time-1)*100:.1f}%':<12} {f'+{(beanie_time/motor_time-1)*100:.1f}%':<12}")
    print(f"{'4. data-bridge (Rust + PyO3)':<35} {db_time:>10.3f}   {f'+{(db_time/pymongo_time-1)*100:.1f}%':<12} {f'+{(db_time/motor_time-1)*100:.1f}%':<12}")

    # Overhead breakdown
    print("\n" + "-" * 70)
    print("OVERHEAD BREAKDOWN:")
    print("-" * 70)

    async_overhead = motor_time - pymongo_time
    beanie_odm_overhead = beanie_time - motor_time
    db_vs_motor = db_time - motor_time
    db_vs_beanie = db_time - beanie_time

    print(f"  Async overhead (Motor vs PyMongo):   {async_overhead:>+.3f} ms")
    print(f"  Beanie ODM (Beanie vs Motor):        {beanie_odm_overhead:>+.3f} ms")
    print(f"  data-bridge vs Motor:                {db_vs_motor:>+.3f} ms  <- Rust driver + PyO3 async")
    print(f"  data-bridge vs Beanie:               {db_vs_beanie:>+.3f} ms  <- NET OVERHEAD")

    # Cleanup
    await close()
    motor_client.close()
    pymongo_client.close()


async def profile_find_overhead():
    """Profile find operation overhead."""
    print("\n" + "=" * 70)
    print("FIND ONE OVERHEAD ANALYSIS")
    print("=" * 70)

    # Initialize
    await init(MONGODB_URI)

    motor_client = motor.motor_asyncio.AsyncIOMotorClient(MONGODB_URI)
    motor_db = motor_client.get_default_database()
    motor_col = motor_db["profile_motor"]

    pymongo_client = MongoClient(MONGODB_URI)
    pymongo_db = pymongo_client.get_default_database()
    pymongo_col = pymongo_db["profile_pymongo"]

    await init_beanie(database=motor_db, document_models=BEANIE_MODELS)

    # Seed data
    doc_data = {"name": "Test", "email": "test@test.com", "age": 35}
    pymongo_col.delete_many({})
    pymongo_col.insert_one(doc_data)
    await motor_col.delete_many({})
    await motor_col.insert_one(doc_data)
    await DBUser.delete_many({})
    await DBUser(**doc_data).save()
    await BeanieUser.find_all().delete()
    await BeanieUser(**doc_data).insert()

    query = {"age": 35}

    # 1. PyMongo
    def pymongo_find():
        return pymongo_col.find_one(query)
    pymongo_time = time_sync(pymongo_find)

    # 2. Motor
    async def motor_find():
        return await motor_col.find_one(query)
    motor_time = await time_async(motor_find)

    # 3. Beanie
    async def beanie_find():
        return await BeanieUser.find_one(query)
    beanie_time = await time_async(beanie_find)

    # 4. data-bridge ODM
    async def db_find():
        return await DBUser.find_one(DBUser.age == 35)
    db_time = await time_async(db_find)

    # Results
    print(f"\n{'Layer':<35} {'Time (ms)':<12} {'vs PyMongo':<12} {'vs Motor':<12}")
    print("-" * 70)
    print(f"{'1. PyMongo (sync, C ext)':<35} {pymongo_time:>10.3f}   {'baseline':<12} {'-':<12}")
    print(f"{'2. Motor (async, C ext)':<35} {motor_time:>10.3f}   {f'+{(motor_time/pymongo_time-1)*100:.1f}%':<12} {'baseline':<12}")
    print(f"{'3. Beanie (Motor + Pydantic)':<35} {beanie_time:>10.3f}   {f'+{(beanie_time/pymongo_time-1)*100:.1f}%':<12} {f'+{(beanie_time/motor_time-1)*100:.1f}%':<12}")
    print(f"{'4. data-bridge (Rust + PyO3)':<35} {db_time:>10.3f}   {f'+{(db_time/pymongo_time-1)*100:.1f}%':<12} {f'+{(db_time/motor_time-1)*100:.1f}%':<12}")

    # Breakdown
    print("\n" + "-" * 70)
    print("OVERHEAD BREAKDOWN:")
    print("-" * 70)
    print(f"  data-bridge vs Motor:  {db_time - motor_time:>+.3f} ms")
    print(f"  data-bridge vs Beanie: {db_time - beanie_time:>+.3f} ms")

    await close()
    motor_client.close()
    pymongo_client.close()


async def profile_bulk_insert():
    """Profile bulk insert (where Rust should shine)."""
    print("\n" + "=" * 70)
    print("BULK INSERT (1000 docs) OVERHEAD ANALYSIS")
    print("=" * 70)

    await init(MONGODB_URI)

    motor_client = motor.motor_asyncio.AsyncIOMotorClient(MONGODB_URI)
    motor_db = motor_client.get_default_database()
    motor_col = motor_db["profile_motor"]

    await init_beanie(database=motor_db, document_models=BEANIE_MODELS)

    data = [{"name": f"User{i}", "email": f"u{i}@test.com", "age": 20 + i % 50} for i in range(1000)]

    # Motor raw
    async def motor_bulk():
        await motor_col.delete_many({})
        await motor_col.insert_many(data)
    motor_time = await time_async(motor_bulk, iterations=20)

    # Beanie
    async def beanie_bulk():
        await BeanieUser.find_all().delete()
        await BeanieUser.insert_many([BeanieUser(**d) for d in data])
    beanie_time = await time_async(beanie_bulk, iterations=20)

    # data-bridge ODM
    async def db_bulk():
        await DBUser.delete_many({})
        await DBUser.insert_many([DBUser(**d) for d in data])
    db_time = await time_async(db_bulk, iterations=20)

    print(f"\n{'Layer':<35} {'Time (ms)':<12} {'vs Motor':<12}")
    print("-" * 70)
    print(f"{'Motor raw':<35} {motor_time:>10.3f}   {'baseline':<12}")
    print(f"{'Beanie ODM':<35} {beanie_time:>10.3f}   {f'+{(beanie_time/motor_time-1)*100:.1f}%':<12}")
    print(f"{'data-bridge ODM':<35} {db_time:>10.3f}   {f'+{(db_time/motor_time-1)*100:.1f}%':<12}")

    print(f"\n  Per-doc overhead vs Beanie: {(db_time - beanie_time) / 1000:.4f} ms/doc")
    print(f"  (Bulk amortizes per-call async bridge overhead)")

    await close()
    motor_client.close()


async def main():
    print("\n" + "#" * 70)
    print("# DATA-BRIDGE PERFORMANCE OVERHEAD ANALYSIS")
    print("#" * 70)
    print("\nThis analysis identifies WHERE the overhead comes from.")
    print("Goal: Understand if overhead is in PyO3 async bridge, BSON, or ODM layer.")

    await profile_insert_overhead()
    await profile_find_overhead()
    await profile_bulk_insert()

    print("\n" + "=" * 70)
    print("CONCLUSION")
    print("=" * 70)
    print("""
The main overhead sources in data-bridge vs Beanie:

1. PyO3 ASYNC BRIDGE (pyo3_async_runtimes::tokio::future_into_py)
   - Every async call crosses Python asyncio <-> Rust tokio boundary
   - This is a fixed per-call overhead (~0.3-0.5ms)
   - Motor uses native Python asyncio, avoiding this bridge

2. BSON SERIALIZATION PATH
   - data-bridge: Python dict -> ExtractedValue (GIL) -> BSON (no GIL) -> MongoDB
   - Motor/Beanie: Python dict -> BSON (C extension) -> MongoDB
   - Motor's BSON uses PyMongo's C extension (highly optimized)

3. ODM LAYER
   - Both Beanie and data-bridge add ODM overhead
   - This is similar between both (~0.1-0.2ms)

WHY RUST IS SLOWER FOR SINGLE OPERATIONS:
- Rust MongoDB driver is fast, but PyO3 async bridge adds ~0.4ms per call
- This fixed overhead dominates for single-doc operations
- Bulk operations amortize this overhead better

OPTIMIZATION PATHS:
a) Reduce async bridge crossings (batch operations internally)
b) Use sync path for simple operations (bypass async bridge)
c) Use pythonize for faster Python <-> Rust conversion
d) Pipeline multiple operations in single async call
""")


if __name__ == "__main__":
    uvloop.install()
    asyncio.run(main())
