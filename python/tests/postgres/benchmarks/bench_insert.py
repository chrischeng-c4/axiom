"""Insert benchmarks for PostgreSQL."""

import pytest
from cclab.probe import BenchmarkGroup, register_group
from tests.postgres.benchmarks.models import DBUser, SAUser, SQLALCHEMY_AVAILABLE
from tests.postgres.benchmarks.conftest import generate_user_data
from tests.postgres.benchmarks import benchmark_setup


# =====================
# Insert One
# =====================

insert_one = BenchmarkGroup("Insert One")


@insert_one.add("data-bridge")
async def db_insert_one():
    """Insert one record with data-bridge."""
    await benchmark_setup.async_ensure_setup()
    user = DBUser(name="Test", email="test@test.com", age=30, active=True)
    await user.save()


@insert_one.add("asyncpg")
async def asyncpg_insert_one():
    """Insert one record with asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        await conn.execute(
            """
            INSERT INTO bench_asyncpg_users (name, email, age, active)
            VALUES ($1, $2, $3, $4)
            """,
            "Test",
            "test@test.com",
            30,
            True,
        )


@insert_one.add("psycopg2")
def psycopg2_insert_one():
    """Insert one record with psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                """
                INSERT INTO bench_psycopg2_users (name, email, age, active)
                VALUES (%s, %s, %s, %s)
                """,
                ("Test", "test@test.com", 30, True),
            )
            conn.commit()
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @insert_one.add("SQLAlchemy")
    async def sqlalchemy_insert_one():
        """Insert one record with SQLAlchemy."""
        async with benchmark_setup._sqlalchemy_engine.begin() as conn:
             pass # just for context manager
        
        # We need a new session for each operation
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession
        async_session = sessionmaker(benchmark_setup._sqlalchemy_engine, class_=AsyncSession, expire_on_commit=False)
        
        async with async_session() as session:
            user = SAUser(name="Test", email="test@test.com", age=30, active=True)
            session.add(user)
            await session.commit()


register_group(insert_one)


# =====================
# Bulk Insert (1000)
# =====================

DATA_1000 = generate_user_data(1000)

bulk_insert_1000 = BenchmarkGroup("Bulk Insert (1000)")


@bulk_insert_1000.add("data-bridge")
async def db_bulk_insert_1000():
    """Bulk insert 1000 records with data-bridge."""
    users = [DBUser(**d) for d in DATA_1000]
    await DBUser.insert_many(users)


@bulk_insert_1000.add("asyncpg")
async def asyncpg_bulk_insert_1000():
    """Bulk insert 1000 records with asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        await conn.executemany(
            """
            INSERT INTO bench_asyncpg_users (name, email, age, city, score, active)
            VALUES ($1, $2, $3, $4, $5, $6)
            """,
            [(d["name"], d["email"], d["age"], d["city"], d["score"], d["active"]) for d in DATA_1000],
        )


@bulk_insert_1000.add("psycopg2")
def psycopg2_bulk_insert_1000():
    """Bulk insert 1000 records with psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            from psycopg2.extras import execute_values

            values = [
                (d["name"], d["email"], d["age"], d["city"], d["score"], d["active"])
                for d in DATA_1000
            ]
            execute_values(
                cur,
                """
                INSERT INTO bench_psycopg2_users (name, email, age, city, score, active)
                VALUES %s
                """,
                values,
            )
            conn.commit()
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @bulk_insert_1000.add("SQLAlchemy")
    async def sqlalchemy_bulk_insert_1000():
        """Bulk insert 1000 records with SQLAlchemy."""
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession
        async_session = sessionmaker(benchmark_setup._sqlalchemy_engine, class_=AsyncSession, expire_on_commit=False)

        async with async_session() as session:
            users = [SAUser(**d) for d in DATA_1000]
            session.add_all(users)
            await session.commit()


register_group(bulk_insert_1000)


# =====================
# Bulk Insert (10000)
# =====================

DATA_10000 = generate_user_data(10000)

bulk_insert_10000 = BenchmarkGroup("Bulk Insert (10000)")


@bulk_insert_10000.add("data-bridge")
async def db_bulk_insert_10000():
    """Bulk insert 10000 records with data-bridge."""
    users = [DBUser(**d) for d in DATA_10000]
    await DBUser.insert_many(users)


@bulk_insert_10000.add("asyncpg")
async def asyncpg_bulk_insert_10000():
    """Bulk insert 10000 records with asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        await conn.executemany(
            """
            INSERT INTO bench_asyncpg_users (name, email, age, city, score, active)
            VALUES ($1, $2, $3, $4, $5, $6)
            """,
            [(d["name"], d["email"], d["age"], d["city"], d["score"], d["active"]) for d in DATA_10000],
        )


@bulk_insert_10000.add("psycopg2")
def psycopg2_bulk_insert_10000():
    """Bulk insert 10000 records with psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            from psycopg2.extras import execute_values

            values = [
                (d["name"], d["email"], d["age"], d["city"], d["score"], d["active"])
                for d in DATA_10000
            ]
            execute_values(
                cur,
                """
                INSERT INTO bench_psycopg2_users (name, email, age, city, score, active)
                VALUES %s
                """,
                values,
            )
            conn.commit()
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @bulk_insert_10000.add("SQLAlchemy")
    async def sqlalchemy_bulk_insert_10000():
        """Bulk insert 10000 records with SQLAlchemy."""
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession
        async_session = sessionmaker(benchmark_setup._sqlalchemy_engine, class_=AsyncSession, expire_on_commit=False)

        async with async_session() as session:
            users = [SAUser(**d) for d in DATA_10000]
            session.add_all(users)
            await session.commit()


register_group(bulk_insert_10000)
