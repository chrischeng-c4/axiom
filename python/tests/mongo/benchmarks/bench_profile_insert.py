"""Profile insert_one to find the actual bottleneck."""

import asyncio
import time
from cclab import Document, init, close
from cclab import mongodb as _rust
from motor.motor_asyncio import AsyncIOMotorClient
from pymongo import MongoClient


class ProfileUser(Document):
    name: str
    email: str
    age: int

    class Settings:
        name = "profile_users"


MONGO_URI = "mongodb://testuser:testpass@localhost:27017/data-bridge-benchmark?authSource=admin"


async def profile_insert():
    """Profile each step of save()."""
    await init(MONGO_URI)

    # Also connect with Motor and PyMongo for comparison
    motor_client = AsyncIOMotorClient(MONGO_URI)
    motor_db = motor_client["data-bridge-benchmark"]
    motor_col = motor_db["profile_motor"]

    pymongo_client = MongoClient(MONGO_URI)
    pymongo_db = pymongo_client["data-bridge-benchmark"]
    pymongo_col = pymongo_db["profile_pymongo"]

    # Cleanup
    await ProfileUser.find().delete()
    await motor_col.delete_many({})
    pymongo_col.delete_many({})

    iterations = 100
    doc = {"name": "Alice", "email": "alice@example.com", "age": 30}

    # Warmup
    for _ in range(10):
        user = ProfileUser(**doc)
        await user.save()
        await motor_col.insert_one(doc.copy())
        pymongo_col.insert_one(doc.copy())

    await ProfileUser.find().delete()
    await motor_col.delete_many({})
    pymongo_col.delete_many({})

    # 1. data-bridge full save() (includes Python overhead)
    times = []
    for _ in range(iterations):
        user = ProfileUser(**doc)
        start = time.perf_counter()
        await user.save()
        times.append(time.perf_counter() - start)
    db_full_time = sum(times) / len(times) * 1000

    # 2. Rust Document.save() directly (bypasses Python Document)
    times = []
    for _ in range(iterations):
        rust_doc = _rust.Document("profile_rust_direct", doc)
        start = time.perf_counter()
        await rust_doc.save()
        times.append(time.perf_counter() - start)
    rust_direct_time = sum(times) / len(times) * 1000

    # 3. Motor (async pymongo)
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        await motor_col.insert_one(doc.copy())
        times.append(time.perf_counter() - start)
    motor_time = sum(times) / len(times) * 1000

    # 4. PyMongo (sync)
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        pymongo_col.insert_one(doc.copy())
        times.append(time.perf_counter() - start)
    pymongo_time = sum(times) / len(times) * 1000

    # Cleanup
    await ProfileUser.find().delete()
    await motor_col.delete_many({})
    pymongo_col.delete_many({})
    await close()
    motor_client.close()
    pymongo_client.close()

    print(f"\n{'='*60}")
    print("Insert Comparison (single insert)")
    print(f"{'='*60}")
    print(f"data-bridge full:    {db_full_time:.3f} ms")
    print(f"Rust direct:         {rust_direct_time:.3f} ms")
    print(f"Motor (async):       {motor_time:.3f} ms")
    print(f"PyMongo (sync):      {pymongo_time:.3f} ms")
    print(f"{'='*60}")
    print(f"\nOverhead breakdown:")
    print(f"  Python Document overhead: {db_full_time - rust_direct_time:.3f} ms")
    print(f"  Rust vs Motor overhead:   {rust_direct_time - motor_time:.3f} ms")
    print(f"  Async vs Sync overhead:   {motor_time - pymongo_time:.3f} ms")
    print(f"{'='*60}\n")


if __name__ == "__main__":
    asyncio.run(profile_insert())
