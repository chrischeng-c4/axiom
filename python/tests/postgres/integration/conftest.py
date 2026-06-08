"""
Pytest fixtures for PostgreSQL integration tests.

These fixtures provide:
- Session-scoped database connection
- Function-scoped table cleanup
- Test data fixtures
"""

import pytest
import asyncio
import os
from cclab.titan import init, close, execute


# PostgreSQL connection URI for tests
POSTGRES_TEST_URI = os.getenv(
    "POSTGRES_URI",
    "postgresql://rstn:rstn@localhost:5432/cclab_test"
)


@pytest.fixture(scope="session")
def event_loop():
    """Create event loop for async tests."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@pytest.fixture(scope="session", autouse=True)
async def setup_test_database():
    """
    Set up test database connection pool.

    This fixture runs once per test session and initializes the
    PostgreSQL connection pool.
    """
    # Initialize connection with test database
    await init(POSTGRES_TEST_URI, min_connections=2, max_connections=10)

    yield

    # Cleanup - close connection pool
    await close()


@pytest.fixture(scope="function", autouse=True)
async def cleanup_tables():
    """
    Clean up all test tables after each test.

    This ensures test isolation by dropping all tables in the public schema
    after each test function completes.
    """
    yield

    # Drop all test tables after each test
    try:
        # Get all tables in public schema
        tables = await execute(
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public'"
        )

        # Drop each table (CASCADE to handle dependencies)
        for row in tables:
            await execute(f"DROP TABLE IF EXISTS {row['tablename']} CASCADE")
    except Exception:
        # Ignore errors during cleanup (e.g., if no tables exist)
        pass


@pytest.fixture
async def test_table():
    """
    Create a standard test table for testing.

    Returns:
        str: The table name ('test_users')
    """
    await execute("""
        CREATE TABLE IF NOT EXISTS test_users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE,
            age INTEGER,
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)

    yield "test_users"

    # Table will be cleaned up by cleanup_tables fixture


@pytest.fixture
async def sample_data(test_table):
    """
    Insert sample user data into test table.

    Returns:
        list: List of inserted user dictionaries
    """
    users = [
        {"name": "Alice", "email": "alice@example.com", "age": 30},
        {"name": "Bob", "email": "bob@example.com", "age": 25},
        {"name": "Charlie", "email": "charlie@example.com", "age": 35},
    ]

    for user in users:
        await execute(
            f"INSERT INTO {test_table} (name, email, age) VALUES ($1, $2, $3)",
            [user["name"], user["email"], user["age"]]
        )

    return users
