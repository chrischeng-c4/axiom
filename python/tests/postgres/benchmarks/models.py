"""
Shared models for PostgreSQL benchmarks.

Comparing: data-bridge-postgres (Rust async) vs asyncpg vs psycopg2 vs SQLAlchemy
"""

from typing import Optional
from cclab.titan import Table, Column


# data-bridge model
class DBUser(Table):
    """data-bridge User model."""

    id: int = Column(primary_key=True)
    name: str
    email: str
    age: int
    city: Optional[str] = None
    score: Optional[float] = None
    active: bool = True

    class Settings:
        table_name = "bench_db_users"
        schema = "public"


# SQLAlchemy models (async)
try:
    from sqlalchemy.ext.asyncio import AsyncAttrs
    from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column
    from sqlalchemy import String, Integer, Float, Boolean

    class Base(AsyncAttrs, DeclarativeBase):
        pass

    class SAUser(Base):
        """SQLAlchemy User model."""

        __tablename__ = "bench_sa_users"

        id: Mapped[int] = mapped_column(Integer, primary_key=True)
        name: Mapped[str] = mapped_column(String(100))
        email: Mapped[str] = mapped_column(String(100))
        age: Mapped[int] = mapped_column(Integer)
        city: Mapped[Optional[str]] = mapped_column(String(100), nullable=True)
        score: Mapped[Optional[float]] = mapped_column(Float, nullable=True)
        active: Mapped[bool] = mapped_column(Boolean, default=True)

    SQLALCHEMY_AVAILABLE = True
except ImportError:
    SQLALCHEMY_AVAILABLE = False
    SAUser = None
    Base = None


# Raw SQL schema for asyncpg and psycopg2
ASYNCPG_TABLE_SCHEMA = """
CREATE TABLE IF NOT EXISTS bench_asyncpg_users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) NOT NULL,
    age INTEGER NOT NULL,
    city VARCHAR(100),
    score FLOAT,
    active BOOLEAN DEFAULT TRUE
)
"""

PSYCOPG2_TABLE_SCHEMA = """
CREATE TABLE IF NOT EXISTS bench_psycopg2_users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) NOT NULL,
    age INTEGER NOT NULL,
    city VARCHAR(100),
    score FLOAT,
    active BOOLEAN DEFAULT TRUE
)
"""


def get_table_name(framework: str) -> str:
    """Get table name for a specific framework."""
    return {
        "cclab": "bench_db_users",
        "asyncpg": "bench_asyncpg_users",
        "psycopg2": "bench_psycopg2_users",
        "sqlalchemy": "bench_sa_users",
    }[framework]
