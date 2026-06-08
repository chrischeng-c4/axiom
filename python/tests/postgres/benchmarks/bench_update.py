"""Update benchmarks for PostgreSQL."""

import pytest
from cclab.probe import BenchmarkGroup, register_group
from tests.postgres.benchmarks.models import DBUser, SAUser, SQLALCHEMY_AVAILABLE
from tests.postgres.benchmarks.conftest import generate_user_data
from tests.postgres.benchmarks import benchmark_setup


# =====================
# Update One
# =====================

update_one = BenchmarkGroup("Update One")


@update_one.add("data-bridge")
async def db_update_one():
    """Update one record with data-bridge."""
    # Note: Assumes data exists from previous inserts
    user = await DBUser.find_one(DBUser.id == 1)
    if user:
        user.age = 35
        await user.save()
    return user


@update_one.add("asyncpg")
async def asyncpg_update_one():
    """Update one record with asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        await conn.execute(
            "UPDATE bench_asyncpg_users SET age = $1 WHERE id = $2", 35, 1
        )


@update_one.add("psycopg2")
def psycopg2_update_one():
    """Update one record with psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "UPDATE bench_psycopg2_users SET age = %s WHERE id = %s", (35, 1)
            )
            conn.commit()
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @update_one.add("SQLAlchemy")
    async def sqlalchemy_update_one():
        """Update one record with SQLAlchemy."""
        from sqlalchemy import select
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession

        async_session = sessionmaker(
            benchmark_setup._sqlalchemy_engine,
            class_=AsyncSession,
            expire_on_commit=False
        )

        async with async_session() as session:
            result = await session.execute(select(SAUser).where(SAUser.id == 1))
            user = result.scalar_one_or_none()
            if user:
                user.age = 35
                await session.commit()
            return user


register_group(update_one)


# =====================
# Update Many (1000 records)
# =====================

DATA_1000 = generate_user_data(1000)

update_many = BenchmarkGroup("Update Many (1000)")


@update_many.add("data-bridge")
async def db_update_many():
    """Update 1000 records with data-bridge."""
    result = await DBUser.update_many(
        {"age": 40}, DBUser.age > 25
    )
    return result


@update_many.add("asyncpg")
async def asyncpg_update_many():
    """Update 1000 records with asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        result = await conn.execute(
            "UPDATE bench_asyncpg_users SET age = $1 WHERE age > $2", 40, 25
        )
        return result


@update_many.add("psycopg2")
def psycopg2_update_many():
    """Update 1000 records with psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "UPDATE bench_psycopg2_users SET age = %s WHERE age > %s", (40, 25)
            )
            conn.commit()
            return cur.rowcount
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @update_many.add("SQLAlchemy")
    async def sqlalchemy_update_many():
        """Update 1000 records with SQLAlchemy."""
        from sqlalchemy import update
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession

        async_session = sessionmaker(
            benchmark_setup._sqlalchemy_engine,
            class_=AsyncSession,
            expire_on_commit=False
        )

        async with async_session() as session:
            result = await session.execute(
                update(SAUser).where(SAUser.age > 25).values(age=40)
            )
            await session.commit()
            return result.rowcount


register_group(update_many)


# =====================
# Delete Many (500 records)
# =====================

delete_many = BenchmarkGroup("Delete Many (500)")


@delete_many.add("data-bridge")
async def db_delete_many():
    """Delete 500 records with data-bridge."""
    # Note: This is a placeholder - adjust based on actual data-bridge API
    result = await DBUser.delete_many(DBUser.age > 45)
    return result


@delete_many.add("asyncpg")
async def asyncpg_delete_many():
    """Delete 500 records with asyncpg."""
    async with benchmark_setup._asyncpg_pool.acquire() as conn:
        result = await conn.execute(
            "DELETE FROM bench_asyncpg_users WHERE age > $1", 45
        )
        return result


@delete_many.add("psycopg2")
def psycopg2_delete_many():
    """Delete 500 records with psycopg2."""
    conn = benchmark_setup._psycopg2_pool.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute("DELETE FROM bench_psycopg2_users WHERE age > %s", (45,))
            conn.commit()
            return cur.rowcount
    finally:
        benchmark_setup._psycopg2_pool.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @delete_many.add("SQLAlchemy")
    async def sqlalchemy_delete_many():
        """Delete 500 records with SQLAlchemy."""
        from sqlalchemy import delete
        from sqlalchemy.orm import sessionmaker
        from sqlalchemy.ext.asyncio import AsyncSession

        async_session = sessionmaker(
            benchmark_setup._sqlalchemy_engine,
            class_=AsyncSession,
            expire_on_commit=False
        )

        async with async_session() as session:
            result = await session.execute(
                delete(SAUser).where(SAUser.age > 45)
            )
            await session.commit()
            return result.rowcount


register_group(delete_many)
