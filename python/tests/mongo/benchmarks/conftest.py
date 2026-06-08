"""
Benchmark fixtures for comprehensive framework comparison.

Provides database setup and test data fixtures for performance testing across
6 different MongoDB frameworks: data-bridge, beanie, motor, pymongo (sync),
pymongo+gevent, and mongoengine.
"""

import pytest
import asyncio
from typing import List, Dict, Any
from cclab import init, close, is_connected


# =====================
# Constants
# =====================

MONGODB_URI = "mongodb://localhost:27017/data-bridge-benchmark"
DATABASE_NAME = "data-bridge-benchmark"

# Batch sizes for parametrized tests
BATCH_SIZES = [10, 100, 1000, 10000, 50000]

# Framework identifiers
FRAMEWORKS = [
    "cclab",
    "beanie",
    "motor",
    "pymongo_sync",
    "pymongo_gevent",
    "mongoengine",
]


# =====================
# data-bridge setup
# =====================

# Note: We don't override event_loop - use the one from tests/conftest.py

@pytest.fixture(scope="session")
async def benchmark_db():
    """
    Initialize data-bridge connection for benchmarks (session-scoped).

    Uses separate database for benchmarks to avoid interference.
    """
    from cclab import init, close, is_connected

    # Close existing connection and switch to benchmark database
    if is_connected():
        await close()

    # Use separate database for benchmarks
    await init(MONGODB_URI)
    yield

    # Clean up connection
    if is_connected():
        await close()


# Alias for backward compatibility
@pytest.fixture(scope="session")
async def cclab_db(benchmark_db):
    """Alias for benchmark_db."""
    yield


# =====================
# Beanie setup
# =====================

@pytest.fixture(scope="function")
async def beanie_db():
    """Initialize Beanie connection (function-scoped for proper event loop binding)."""
    from motor.motor_asyncio import AsyncIOMotorClient
    from beanie import init_beanie

    # Create client without binding to specific loop (uses current loop)
    client = AsyncIOMotorClient(MONGODB_URI)
    # Note: Beanie models are registered dynamically in each test
    # This fixture just provides the client
    yield client
    client.close()


# =====================
# Motor (pure async) setup
# =====================

@pytest.fixture(scope="function")
async def motor_db():
    """Initialize Motor client (function-scoped for proper event loop binding)."""
    from motor.motor_asyncio import AsyncIOMotorClient

    # Create client without binding to specific loop (uses current loop)
    client = AsyncIOMotorClient(MONGODB_URI)
    yield client[DATABASE_NAME]
    client.close()


# =====================
# PyMongo sync setup
# =====================

@pytest.fixture(scope="session")
def pymongo_sync_db():
    """Initialize sync PyMongo client (session-scoped)."""
    from pymongo import MongoClient

    client = MongoClient(MONGODB_URI)
    yield client[DATABASE_NAME]
    client.close()


# =====================
# PyMongo + Gevent setup
# =====================

@pytest.fixture(scope="session")
def pymongo_gevent_db():
    """Initialize PyMongo with gevent (session-scoped)."""
    from pymongo import MongoClient

    # Note: For proper gevent support, you may need to monkey patch
    # at the module level before importing pymongo. For benchmarking,
    # we're comparing raw performance without full monkey patching.
    client = MongoClient(MONGODB_URI)
    yield client[DATABASE_NAME]
    client.close()


# =====================
# MongoEngine setup
# =====================

@pytest.fixture(scope="session")
def mongoengine_db():
    """Initialize MongoEngine connection (session-scoped)."""
    from mongoengine import connect, disconnect

    disconnect(alias='default')
    connect(DATABASE_NAME, host=MONGODB_URI, alias='default')
    yield
    disconnect(alias='default')


# =====================
# Test Data Generation
# =====================

def generate_user_data(count: int) -> List[Dict[str, Any]]:
    """
    Generate test user documents with rich schema.

    Args:
        count: Number of documents to generate

    Returns:
        List of dictionaries representing user documents
    """
    return [
        {
            "name": f"User{i}",
            "email": f"user{i}@example.com",
            "age": 20 + (i % 50),
            "city": ["NYC", "LA", "SF", "Chicago", "Boston"][i % 5],
            "score": float(i * 1.5),
            "active": i % 2 == 0,
        }
        for i in range(count)
    ]


# Legacy fixtures for backward compatibility
@pytest.fixture
def benchmark_data_100():
    """Generate 100 test documents."""
    return generate_user_data(100)


@pytest.fixture
def benchmark_data_1000():
    """Generate 1000 test documents."""
    return generate_user_data(1000)


@pytest.fixture
def benchmark_data_10000():
    """Generate 10000 test documents for stress testing."""
    return generate_user_data(10000)


# =====================
# Collection Naming
# =====================

def get_collection_name(framework: str, operation: str, batch_size: int = None) -> str:
    """
    Generate unique collection name per framework/operation/batch.

    Ensures complete isolation to prevent data interference.

    Args:
        framework: Framework identifier (e.g., "cclab", "beanie")
        operation: Operation name (e.g., "insert_bulk", "find_one")
        batch_size: Optional batch size for further isolation

    Returns:
        Unique collection name
    """
    if batch_size is not None:
        return f"bench_{framework}_{operation}_{batch_size}"
    return f"bench_{framework}_{operation}"


# =====================
# Benchmark Configuration
# =====================

def get_benchmark_params(batch_size: int) -> Dict[str, int]:
    """
    Calculate adaptive benchmark parameters based on batch size.

    Scales down iterations for larger batches to keep total time reasonable.
    Includes warmup rounds to stabilize measurements.

    Args:
        batch_size: Number of documents in the batch

    Returns:
        Dictionary with 'iterations', 'rounds', and 'warmup_rounds' keys
    """
    if batch_size <= 100:
        iterations = 50
        rounds = 5
        warmup_rounds = 3
    elif batch_size <= 1000:
        iterations = 20
        rounds = 5
        warmup_rounds = 2
    elif batch_size <= 10000:
        iterations = 10
        rounds = 3
        warmup_rounds = 1
    else:  # 50000
        iterations = 3
        rounds = 3
        warmup_rounds = 1

    return {
        "iterations": iterations,
        "rounds": rounds,
        "warmup_rounds": warmup_rounds,
    }


# =====================
# Cleanup fixture
# =====================

@pytest.fixture(autouse=True)
async def cleanup_benchmark_collections(benchmark_db):
    """Clean up collections after each benchmark test."""
    yield
    # Cleanup happens after test
