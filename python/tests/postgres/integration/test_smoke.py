"""
Smoke tests to verify basic PostgreSQL connectivity and functionality.

These tests ensure that the test infrastructure is working correctly
before running more complex integration tests.
"""

from tests.postgres.base import PostgresSuite
from cclab.titan import execute
from cclab.probe import test, expect


class TestSmoke(PostgresSuite):
    """Basic smoke tests for PostgreSQL connectivity."""

    async def create_test_table(self) -> str:
        """Create standard test table and return its name."""
        await self.ensure_db()
        await execute("DROP TABLE IF EXISTS test_users CASCADE")
        await execute("""
            CREATE TABLE test_users (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) UNIQUE,
                age INTEGER,
                active BOOLEAN DEFAULT true,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        """)
        return "test_users"

    async def insert_sample_data(self, table_name: str) -> list:
        """Insert sample data and return the list of users."""
        users = [
            {"name": "Alice", "email": "alice@example.com", "age": 30},
            {"name": "Bob", "email": "bob@example.com", "age": 25},
            {"name": "Charlie", "email": "charlie@example.com", "age": 35},
        ]
        for user in users:
            await execute(
                f"INSERT INTO {table_name} (name, email, age) VALUES ($1, $2, $3)",
                [user["name"], user["email"], user["age"]]
            )
        return users

    @test
    async def test_database_connection(self):
        """Test basic database connectivity with simple query."""
        await self.ensure_db()
        result = await execute("SELECT 1 as num")
        expect(len(result)).to_equal(1)
        expect(result[0]["num"]).to_equal(1)

    @test
    async def test_database_version(self):
        """Test database version query."""
        await self.ensure_db()
        result = await execute("SELECT version()")
        expect(len(result)).to_equal(1)
        expect("PostgreSQL" in result[0]["version"]).to_be_true()

    @test
    async def test_create_and_query_table(self):
        """Test table creation and basic CRUD operations."""
        test_table = await self.create_test_table()

        # Insert data
        await execute(
            f"INSERT INTO {test_table} (name, email, age) VALUES ($1, $2, $3)",
            ["Test User", "test@example.com", 25]
        )

        # Query data
        result = await execute(
            f"SELECT * FROM {test_table} WHERE name = $1",
            ["Test User"]
        )

        expect(len(result)).to_equal(1)
        expect(result[0]["name"]).to_equal("Test User")
        expect(result[0]["email"]).to_equal("test@example.com")
        expect(result[0]["age"]).to_equal(25)

    @test
    async def test_sample_data_fixture(self):
        """Test that sample_data fixture works correctly."""
        test_table = await self.create_test_table()
        sample_data = await self.insert_sample_data(test_table)

        # Query all data
        result = await execute(f"SELECT * FROM {test_table} ORDER BY name")

        # Should have 3 users from sample_data
        expect(len(result)).to_equal(3)
        expect(result[0]["name"]).to_equal("Alice")
        expect(result[1]["name"]).to_equal("Bob")
        expect(result[2]["name"]).to_equal("Charlie")

        # Verify sample_data matches
        expect(len(sample_data)).to_equal(3)

    @test
    async def test_parameterized_query(self):
        """Test parameterized queries work correctly."""
        test_table = await self.create_test_table()

        # Insert multiple rows
        await execute(
            f"INSERT INTO {test_table} (name, age) VALUES ($1, $2), ($3, $4)",
            ["User1", 20, "User2", 30]
        )

        # Query with parameter
        result = await execute(
            f"SELECT * FROM {test_table} WHERE age > $1",
            [25]
        )

        expect(len(result)).to_equal(1)
        expect(result[0]["name"]).to_equal("User2")

    @test
    async def test_transaction_support(self):
        """Test basic transaction support (implicitly via execute)."""
        test_table = await self.create_test_table()

        # Each execute should be in its own transaction
        await execute(
            f"INSERT INTO {test_table} (name, age) VALUES ($1, $2)",
            ["TransactionTest", 40]
        )

        # Should be immediately visible in next query
        result = await execute(
            f"SELECT * FROM {test_table} WHERE name = $1",
            ["TransactionTest"]
        )

        expect(len(result)).to_equal(1)
        expect(result[0]["age"]).to_equal(40)


if __name__ == "__main__":
    import asyncio
    asyncio.run(TestSmoke().run())
