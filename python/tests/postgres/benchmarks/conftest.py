"""
Benchmark fixtures for PostgreSQL framework comparison.

Provides database setup and test data fixtures for performance testing across
4 different PostgreSQL frameworks: data-bridge, asyncpg, psycopg2, and SQLAlchemy.
"""

import pytest
import os
from typing import List, Dict, Any


# =====================
# Constants
# =====================

POSTGRES_URI = os.environ.get(
    "POSTGRES_URI",
    "postgresql://postgres:postgres@localhost:5432/cclab_benchmark"
)

# Parse connection parameters
def parse_postgres_uri(uri: str) -> Dict[str, str]:
    """Parse PostgreSQL URI into components."""
    # Format: postgresql://user:password@host:port/database
    import urllib.parse
    parsed = urllib.parse.urlparse(uri)
    return {
        "host": parsed.hostname or "localhost",
        "port": parsed.port or 5432,
        "user": parsed.username or "postgres",
        "password": parsed.password or "postgres",
        "database": parsed.path.lstrip("/") or "cclab_benchmark",
    }

CONN_PARAMS = parse_postgres_uri(POSTGRES_URI)

# Batch sizes for parametrized tests
BATCH_SIZES = [10, 100, 1000, 10000]

# Framework identifiers
FRAMEWORKS = [
    "cclab",
    "asyncpg",
    "psycopg2",
    "sqlalchemy",
]


# =====================
# data-bridge setup
# =====================

@pytest.fixture(scope="session")
async def cclab_db():
    """Initialize data-bridge connection for benchmarks (session-scoped)."""
    from cclab.titan import init, close, is_connected

    # Close existing connection if any
    if is_connected():
        await close()

    # Initialize with benchmark database
    await init(POSTGRES_URI)
    yield

    # Clean up connection
    if is_connected():
        await close()


# =====================
# asyncpg setup
# =====================

@pytest.fixture(scope="session")
async def asyncpg_pool():
    """Initialize asyncpg connection pool (session-scoped)."""
    try:
        import asyncpg
    except ImportError:
        pytest.skip("asyncpg not installed")

    pool = await asyncpg.create_pool(
        host=CONN_PARAMS["host"],
        port=CONN_PARAMS["port"],
        user=CONN_PARAMS["user"],
        password=CONN_PARAMS["password"],
        database=CONN_PARAMS["database"],
        min_size=2,
        max_size=10,
    )
    yield pool
    await pool.close()


# =====================
# psycopg2 setup
# =====================

@pytest.fixture(scope="session")
def psycopg2_conn():
    """Initialize psycopg2 connection (session-scoped)."""
    try:
        import psycopg2
        import psycopg2.pool
    except ImportError:
        pytest.skip("psycopg2 not installed")

    # Create connection pool
    pool = psycopg2.pool.SimpleConnectionPool(
        minconn=2,
        maxconn=10,
        host=CONN_PARAMS["host"],
        port=CONN_PARAMS["port"],
        user=CONN_PARAMS["user"],
        password=CONN_PARAMS["password"],
        database=CONN_PARAMS["database"],
    )
    yield pool
    pool.closeall()


# =====================
# SQLAlchemy setup
# =====================

@pytest.fixture(scope="session")
async def sqlalchemy_engine():
    """Initialize SQLAlchemy async engine (session-scoped)."""
    try:
        from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession
        from sqlalchemy.orm import sessionmaker
        from .models import Base, SQLALCHEMY_AVAILABLE
    except ImportError:
        pytest.skip("SQLAlchemy not installed")

    if not SQLALCHEMY_AVAILABLE:
        pytest.skip("SQLAlchemy models not available")

    # Convert postgresql:// to postgresql+asyncpg://
    async_uri = POSTGRES_URI.replace("postgresql://", "postgresql+asyncpg://")

    engine = create_async_engine(
        async_uri,
        echo=False,
        pool_size=10,
        max_overflow=20,
    )

    # Create tables
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    yield engine

    # Cleanup
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)

    await engine.dispose()


@pytest.fixture(scope="function")
async def sqlalchemy_session(sqlalchemy_engine):
    """Create SQLAlchemy session (function-scoped)."""
    from sqlalchemy.ext.asyncio import AsyncSession
    from sqlalchemy.orm import sessionmaker

    async_session = sessionmaker(
        sqlalchemy_engine,
        class_=AsyncSession,
        expire_on_commit=False,
    )

    async with async_session() as session:
        yield session


# =====================
# Test Data Generation
# =====================

def generate_user_data(count: int) -> List[Dict[str, Any]]:
    """
    Generate test user data.

    Args:
        count: Number of records to generate

    Returns:
        List of dictionaries representing user records
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


@pytest.fixture
def benchmark_data_100():
    """Generate 100 test records."""
    return generate_user_data(100)


@pytest.fixture
def benchmark_data_1000():
    """Generate 1000 test records."""
    return generate_user_data(1000)


@pytest.fixture
def benchmark_data_10000():
    """Generate 10000 test records for stress testing."""
    return generate_user_data(10000)


# =====================
# Table Management
# =====================

@pytest.fixture(autouse=True)
async def setup_tables(request):
    """Setup tables before tests and cleanup after."""
    # Skip this fixture for tests marked with no_db_required
    if request.node.get_closest_marker('no_db_required'):
        yield
        return

    # Get fixtures only when needed
    cclab_db = request.getfixturevalue('cclab_db')
    asyncpg_pool = request.getfixturevalue('asyncpg_pool')
    psycopg2_conn = request.getfixturevalue('psycopg2_conn')

    from .models import ASYNCPG_TABLE_SCHEMA, PSYCOPG2_TABLE_SCHEMA

    # Create asyncpg table
    async with asyncpg_pool.acquire() as conn:
        await conn.execute("DROP TABLE IF EXISTS bench_asyncpg_users CASCADE")
        await conn.execute(ASYNCPG_TABLE_SCHEMA)

    # Create psycopg2 table
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute("DROP TABLE IF EXISTS bench_psycopg2_users CASCADE")
            cur.execute(PSYCOPG2_TABLE_SCHEMA)
            conn.commit()
    finally:
        psycopg2_conn.putconn(conn)

    # Create data-bridge table
    from .models import DBUser
    # Note: Assuming data-bridge has a create_table method or similar
    # This is a placeholder - adjust based on actual API
    try:
        # await DBUser.create_table(if_not_exists=True)
        pass
    except Exception:
        pass

    yield

    # Cleanup after test
    async with asyncpg_pool.acquire() as conn:
        await conn.execute("DELETE FROM bench_asyncpg_users")

    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute("DELETE FROM bench_psycopg2_users")
            conn.commit()
    finally:
        psycopg2_conn.putconn(conn)


# =====================
# Benchmark Configuration
# =====================

def get_benchmark_params(batch_size: int) -> Dict[str, int]:
    """
    Calculate adaptive benchmark parameters based on batch size.

    Args:
        batch_size: Number of records in the batch

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
    else:
        iterations = 3
        rounds = 3
        warmup_rounds = 1

    return {
        "iterations": iterations,
        "rounds": rounds,
        "warmup_rounds": warmup_rounds,
    }
