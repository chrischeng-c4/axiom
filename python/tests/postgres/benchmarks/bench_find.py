"""Find/Select benchmarks for PostgreSQL."""

import pytest
from cclab.probe import BenchmarkGroup, register_group
from tests.postgres.benchmarks.models import DBUser, SAUser, SQLALCHEMY_AVAILABLE
from tests.postgres.benchmarks.conftest import generate_user_data
from tests.postgres.benchmarks import benchmark_setup


# =====================
# Find One (by ID)
# =====================

find_one = BenchmarkGroup("Find One (by ID)")


@find_one.add("data-bridge")
async def db_find_one():
    """Find one record by ID with data-bridge."""
    # Note: Assumes data exists from previous inserts
    # In real benchmarks, setup would happen in benchmark_setup
    user = await DBUser.find_one(DBUser.id == 1)
    return user


@find_one.add("asyncpg")
async def asyncpg_find_one():
    """Find one record by ID with asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        row = await conn.fetchrow(
            "SELECT * FROM bench_asyncpg_users WHERE id = $1", 1
        )
        return dict(row) if row else None


@find_one.add("psycopg2")
def psycopg2_find_one():
    """Find one record by ID with psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute("SELECT * FROM bench_psycopg2_users WHERE id = %s", (1,))
            row = cur.fetchone()
            return row
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @find_one.add("SQLAlchemy")
    async def sqlalchemy_find_one():
        """Find one record by ID with SQLAlchemy."""
        from sqlalchemy import select
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession

        async_session = sessionmaker(
            benchmark_setup._sqlalchemy_engine,
            class_=AsyncSession,
            expire_on_commit=False
        )

        async with async_session() as session:
            result = await session.execute(
                select(SAUser).where(SAUser.id == 1)
            )
            return result.scalar_one_or_none()


register_group(find_one)


# =====================
# Find Many (1000 records)
# =====================

DATA_1000 = generate_user_data(1000)

find_many = BenchmarkGroup("Find Many (1000)")


@find_many.add("data-bridge")
async def db_find_many():
    """Find 1000 records with data-bridge."""
    users = await DBUser.find(DBUser.age > 25).limit(1000).to_list()
    return users


@find_many.add("asyncpg")
async def asyncpg_find_many():
    """Find 1000 records with asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        rows = await conn.fetch(
            "SELECT * FROM bench_asyncpg_users WHERE age > $1 LIMIT 1000", 25
        )
        return [dict(row) for row in rows]


@find_many.add("psycopg2")
def psycopg2_find_many():
    """Find 1000 records with psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "SELECT * FROM bench_psycopg2_users WHERE age > %s LIMIT 1000", (25,)
            )
            rows = cur.fetchall()
            return rows
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @find_many.add("SQLAlchemy")
    async def sqlalchemy_find_many():
        """Find 1000 records with SQLAlchemy."""
        from sqlalchemy import select
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession

        async_session = sessionmaker(
            benchmark_setup._sqlalchemy_engine,
            class_=AsyncSession,
            expire_on_commit=False
        )

        async with async_session() as session:
            result = await session.execute(
                select(SAUser).where(SAUser.age > 25).limit(1000)
            )
            return result.scalars().all()


register_group(find_many)


# =====================
# Count with Filter
# =====================

count_filtered = BenchmarkGroup("Count (with filter)")


@count_filtered.add("data-bridge")
async def db_count():
    """Count records with filter using data-bridge."""
    count = await DBUser.count(DBUser.age > 30)
    return count


@count_filtered.add("asyncpg")
async def asyncpg_count():
    """Count records with filter using asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        count = await conn.fetchval(
            "SELECT COUNT(*) FROM bench_asyncpg_users WHERE age > $1", 30
        )
        return count


@count_filtered.add("psycopg2")
def psycopg2_count():
    """Count records with filter using psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "SELECT COUNT(*) FROM bench_psycopg2_users WHERE age > %s", (30,)
            )
            count = cur.fetchone()[0]
            return count
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @count_filtered.add("SQLAlchemy")
    async def sqlalchemy_count():
        """Count records with filter using SQLAlchemy."""
        from sqlalchemy import select, func
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession

        async_session = sessionmaker(
            benchmark_setup._sqlalchemy_engine,
            class_=AsyncSession,
            expire_on_commit=False
        )

        async with async_session() as session:
            result = await session.execute(
                select(func.count()).select_from(SAUser).where(SAUser.age > 30)
            )
            return result.scalar()


register_group(count_filtered)
