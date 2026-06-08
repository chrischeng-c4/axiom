"""
Automatic setup for PostgreSQL benchmarks.

This module initializes PostgreSQL connections when imported, allowing benchmarks
to be auto-discovered and run by dbtest without explicit setup.

Import this module at the top of benchmark files to enable auto-initialization.
"""

import asyncio
import os
from typing import Optional

_setup_complete = False
_asyncpg_pool = None
_psycopg2_pool = None
_sqlalchemy_engine = None


def get_postgres_uri() -> str:
    """Get PostgreSQL URI from environment or use default."""
    return os.environ.get(
        "POSTGRES_URI",
        "postgresql://postgres:postgres@localhost:5432/cclab_benchmark"
    )


def parse_postgres_uri(uri: str) -> dict:
    """Parse PostgreSQL URI into components."""
    import urllib.parse
    parsed = urllib.parse.urlparse(uri)
    return {
        "host": parsed.hostname or "localhost",
        "port": parsed.port or 5432,
        "user": parsed.username or "postgres",
        "password": parsed.password or "postgres",
        "database": parsed.path.lstrip("/") or "cclab_benchmark",
    }


async def _async_setup():
    """Initialize PostgreSQL connections asynchronously."""
    global _setup_complete, _asyncpg_pool, _psycopg2_pool, _sqlalchemy_engine

    if _setup_complete:
        return

    from cclab.titan import init, close, is_connected

    postgres_uri = get_postgres_uri()
    conn_params = parse_postgres_uri(postgres_uri)

    # Initialize asyncpg
    try:
        import asyncpg

        _asyncpg_pool = await asyncpg.create_pool(
            host=conn_params["host"],
            port=conn_params["port"],
            user=conn_params["user"],
            password=conn_params["password"],
            database=conn_params["database"],
            min_size=2,
            max_size=10,
        )
    except ImportError:
        pass

    # Initialize data-bridge
    if is_connected():
        await close()
    await init(postgres_uri)

    # Create tables for data-bridge models
    # We need to do this manually since we don't have a migration tool yet
    from cclab.titan import is_connected as db_is_connected
    if db_is_connected():
        # Use asyncpg or psycopg2 to create tables if available, or use data-bridge execute if implemented
        # For now, we'll use asyncpg if available, otherwise fail gracefully
        if _asyncpg_pool:
            async with _asyncpg_pool.acquire() as conn:
                await conn.execute("""
                    CREATE TABLE IF NOT EXISTS bench_db_users (
                        id SERIAL PRIMARY KEY,
                        name VARCHAR(255) NOT NULL,
                        email VARCHAR(255) NOT NULL,
                        age INTEGER NOT NULL,
                        city VARCHAR(100),
                        score DOUBLE PRECISION,
                        active BOOLEAN DEFAULT TRUE
                    );
                    CREATE TABLE IF NOT EXISTS bench_asyncpg_users (
                        id SERIAL PRIMARY KEY,
                        name VARCHAR(255) NOT NULL,
                        email VARCHAR(255) NOT NULL,
                        age INTEGER NOT NULL,
                        city VARCHAR(100),
                        score DOUBLE PRECISION,
                        active BOOLEAN DEFAULT TRUE
                    );
                    CREATE TABLE IF NOT EXISTS bench_psycopg2_users (
                        id SERIAL PRIMARY KEY,
                        name VARCHAR(255) NOT NULL,
                        email VARCHAR(255) NOT NULL,
                        age INTEGER NOT NULL,
                        city VARCHAR(100),
                        score DOUBLE PRECISION,
                        active BOOLEAN DEFAULT TRUE
                    );
                """)

                # Add indexes for filtered columns to prevent full table scans
                await conn.execute("""
                    CREATE INDEX IF NOT EXISTS idx_bench_db_users_age ON bench_db_users(age);
                    CREATE INDEX IF NOT EXISTS idx_bench_asyncpg_users_age ON bench_asyncpg_users(age);
                    CREATE INDEX IF NOT EXISTS idx_bench_psycopg2_users_age ON bench_psycopg2_users(age);
                    CREATE INDEX IF NOT EXISTS idx_bench_sa_users_age ON bench_sa_users(age);
                """)

    # Initialize psycopg2
    try:
        import psycopg2.pool

        _psycopg2_pool = psycopg2.pool.SimpleConnectionPool(
            minconn=2,
            maxconn=10,
            host=conn_params["host"],
            port=conn_params["port"],
            user=conn_params["user"],
            password=conn_params["password"],
            database=conn_params["database"],
        )
    except ImportError:
        pass

    # Initialize SQLAlchemy
    try:
        from sqlalchemy.ext.asyncio import create_async_engine
        from tests.postgres.benchmarks.models import Base

        async_uri = postgres_uri.replace("postgresql://", "postgresql+asyncpg://")
        _sqlalchemy_engine = create_async_engine(
            async_uri,
            echo=False,
            pool_size=10,
            max_overflow=20,
        )

        # Create tables
        async with _sqlalchemy_engine.begin() as conn:
            await conn.run_sync(Base.metadata.create_all)
    except ImportError:
        pass

    _setup_complete = True


def ensure_setup():
    """
    Ensure PostgreSQL is initialized (synchronous wrapper).

    This is called automatically when benchmarks are imported.
    """
    global _setup_complete

    if not _setup_complete:
        try:
            loop = asyncio.get_running_loop()
        except RuntimeError:
            asyncio.run(_async_setup())


async def async_ensure_setup():
    """
    Ensure PostgreSQL is initialized (async version).

    Call this from benchmark functions if needed.
    """
    await _async_setup()


async def cleanup():
    """Clean up PostgreSQL connections."""
    global _asyncpg_pool, _psycopg2_pool, _sqlalchemy_engine, _setup_complete

    if not _setup_complete:
        return

    from cclab.titan import close

    await close()

    if _asyncpg_pool:
        await _asyncpg_pool.close()
        _asyncpg_pool = None

    if _psycopg2_pool:
        _psycopg2_pool.closeall()
        _psycopg2_pool = None

    if _sqlalchemy_engine:
        await _sqlalchemy_engine.dispose()
        _sqlalchemy_engine = None

    _setup_complete = False


# Auto-initialize when this module is imported (for dbtest auto-discovery)
try:
    ensure_setup()
except Exception as e:
    print(f"Warning: Benchmark setup failed: {e}")
    print("PostgreSQL connection will be attempted when benchmarks run")
