"""
Base classes for PostgreSQL tests using cclab.probe framework.

All tests connect to a real PostgreSQL instance (Homebrew PostgreSQL 17).
"""

import os
from cclab.probe import TestSuite, test, expect, fixture
from cclab.titan import init, close, execute


# PostgreSQL connection URI for tests
# Default: Homebrew PostgreSQL 17 with unix socket authentication
POSTGRES_TEST_URI = os.getenv(
    "POSTGRES_URI",
    "postgresql://chris.cheng@localhost:5432/postgres"
)


class PostgresSuite(TestSuite):
    """
    Base class for PostgreSQL tests with real database connection.

    This suite automatically:
    - Initializes the PostgreSQL connection pool lazily on first use
    - Cleans up all tables after each test (for isolation)

    Example:
        from tests.postgres.base import PostgresSuite
        from cclab.probe import test, expect
        from cclab.titan import execute

        class TestUserCRUD(PostgresSuite):
            @test
            async def test_insert_user(self):
                await self.create_table("users", "id SERIAL PRIMARY KEY, name VARCHAR(255)")
                await execute("INSERT INTO users (name) VALUES ('Alice')")
                rows = await execute("SELECT * FROM users")
                expect(len(rows)).to_equal(1)
    """

    _db_initialized: bool = False

    async def ensure_db(self) -> None:
        """Lazy initialization of database connection. Call at start of each test."""
        if not PostgresSuite._db_initialized:
            await init(POSTGRES_TEST_URI, min_connections=2, max_connections=10)
            PostgresSuite._db_initialized = True

    async def setup_method(self) -> None:
        """Automatically initialize DB before each test."""
        await self.ensure_db()

    async def teardown_method(self) -> None:
        """Clean up all test tables after each test for isolation."""
        try:
            # Get all tables in public schema (exclude system tables)
            tables = await execute(
                "SELECT tablename FROM pg_tables WHERE schemaname = 'public'"
            )

            # Drop each table (CASCADE to handle dependencies)
            for row in tables:
                await execute(f"DROP TABLE IF EXISTS {row['tablename']} CASCADE")
        except Exception:
            # Ignore errors during cleanup
            pass

    async def create_table(self, name: str, columns: str) -> str:
        """Helper to create a test table. Returns the table name."""
        await execute(f"CREATE TABLE IF NOT EXISTS {name} ({columns})")
        return name


# Alias for backwards compatibility
PostgresIntegrationSuite = PostgresSuite


# Fixture functions for ob-test fixture system

@fixture
async def test_table() -> str:
    """
    Create a standard test table.

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
    return "test_users"


@fixture
async def sample_users(test_table: str) -> list:
    """
    Insert sample user data into test table.

    Args:
        test_table: The test table name (from test_table fixture)

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
