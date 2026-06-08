"""Cue database connection management.

This is the community-standard reference implementation for Cue persistence:
SQLAlchemy async engine + asyncpg against PostgreSQL locally and AlloyDB in
managed environments. The cclab ecosystem can later match this behavior.
"""

import os
from collections.abc import AsyncGenerator

from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncEngine, AsyncSession, async_sessionmaker, create_async_engine

from .schema import CONTROL_PLANE_UP_SQL

CUE_DATABASE_URL_ENV = "CUE_DATABASE_URL"
DATABASE_URL_ENV = "DATABASE_URL"


def database_url_from_env() -> str:
    """Return the configured Cue database URL."""
    database_url = os.getenv(CUE_DATABASE_URL_ENV) or os.getenv(DATABASE_URL_ENV)
    if not database_url:
        raise RuntimeError("Set CUE_DATABASE_URL or DATABASE_URL before using Cue PG storage")
    return normalize_database_url(database_url)


def normalize_database_url(database_url: str) -> str:
    """Normalize common Postgres URLs for SQLAlchemy asyncpg."""
    if database_url.startswith("postgres://"):
        database_url = "postgresql://" + database_url.removeprefix("postgres://")
    if database_url.startswith("postgresql://"):
        database_url = "postgresql+asyncpg://" + database_url.removeprefix("postgresql://")
    return database_url


class Database:
    """SQLAlchemy async database manager for Cue."""

    def __init__(self, database_url: str | None = None, echo: bool = False) -> None:
        self.url = normalize_database_url(database_url) if database_url else database_url_from_env()
        self.echo = echo
        self.engine: AsyncEngine = create_async_engine(
            self.url,
            echo=echo,
            pool_pre_ping=True,
        )
        self.sessionmaker = async_sessionmaker(
            self.engine,
            class_=AsyncSession,
            expire_on_commit=False,
        )

    async def connect(self) -> None:
        """Open a test connection and apply the lightweight SQL schema."""
        async with self.engine.begin() as conn:
            for statement in CONTROL_PLANE_UP_SQL:
                await conn.execute(text(statement))

    async def close(self) -> None:
        """Close pooled connections."""
        await self.engine.dispose()

    async def get_session(self) -> AsyncGenerator[AsyncSession, None]:
        """Yield a transactional SQLAlchemy session."""
        async with self.sessionmaker() as session:
            async with session.begin():
                yield session
