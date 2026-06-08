"""
Integration tests for raw SQL execution.

These tests require a running PostgreSQL instance.

Set POSTGRES_URI environment variable to override default:
    export POSTGRES_URI="postgresql://user:pass@localhost:5432/cclab_test"

Run with:
    bash scripts/run_integration_tests.sh
    # or
    POSTGRES_URI="postgresql://rstn:rstn@localhost:5432/cclab_test"         pytest tests/postgres/integration/test_execute_integration.py -v -m integration
"""
from cclab.titan import execute
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite


@fixture
async def test_table():
    """Create a test table for execute integration tests."""
    await execute('''
        CREATE TABLE IF NOT EXISTS test_execute_users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT,
            age INTEGER,
            created_at TIMESTAMP DEFAULT NOW()
        )
    ''')
    await execute('TRUNCATE TABLE test_execute_users RESTART IDENTITY')
    yield


class TestExecuteIntegration(PostgresSuite):
    """Integration tests for execute function with real PostgreSQL."""

    @test
    async def test_execute_select_empty(self, test_table):
        """Test SELECT on empty table returns empty list."""
        results = await execute('SELECT * FROM test_execute_users')
        expect(results).to_equal([])

    @test
    async def test_execute_insert_and_select(self, test_table):
        """Test INSERT followed by SELECT."""
        count = await execute('INSERT INTO test_execute_users (name, email, age) VALUES ($1, $2, $3)', ['Alice', 'alice@example.com', 30])
        expect(count).to_equal(1)
        results = await execute('SELECT * FROM test_execute_users')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Alice')
        expect(results[0]['email']).to_equal('alice@example.com')
        expect(results[0]['age']).to_equal(30)

    @test
    async def test_execute_insert_with_returning(self, test_table):
        """Test INSERT with RETURNING clause."""
        results = await execute('INSERT INTO test_execute_users (name, email, age) VALUES ($1, $2, $3) RETURNING id, name', ['Bob', 'bob@example.com', 25])
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Bob')
        expect('id' in results[0]).to_be_true()

    @test
    async def test_execute_update(self, test_table):
        """Test UPDATE query."""
        await execute('INSERT INTO test_execute_users (name, age) VALUES ($1, $2)', ['Charlie', 20])
        count = await execute('UPDATE test_execute_users SET age = $1 WHERE name = $2', [21, 'Charlie'])
        expect(count).to_equal(1)
        results = await execute('SELECT age FROM test_execute_users WHERE name = $1', ['Charlie'])
        expect(results[0]['age']).to_equal(21)

    @test
    async def test_execute_delete(self, test_table):
        """Test DELETE query."""
        await execute('INSERT INTO test_execute_users (name, age) VALUES ($1, $2), ($3, $4)', ['Dave', 15, 'Eve', 25])
        count = await execute('DELETE FROM test_execute_users WHERE age < $1', [18])
        expect(count).to_equal(1)
        results = await execute('SELECT * FROM test_execute_users')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Eve')

    @test
    async def test_execute_with_null_parameter(self, test_table):
        """Test query with NULL parameter."""
        count = await execute('INSERT INTO test_execute_users (name, email, age) VALUES ($1, $2, $3)', ['Frank', None, 35])
        expect(count).to_equal(1)
        results = await execute('SELECT * FROM test_execute_users WHERE email IS NULL')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Frank')
        expect(results[0]['email']).to_be_none()

    @test
    async def test_execute_with_multiple_types(self, test_table):
        """Test query with various parameter types."""
        await execute('\n            CREATE TABLE IF NOT EXISTS test_types (\n                id SERIAL PRIMARY KEY,\n                text_col TEXT,\n                int_col INTEGER,\n                bigint_col BIGINT,\n                float_col REAL,\n                double_col DOUBLE PRECISION,\n                bool_col BOOLEAN\n            )\n        ')
        try:
            count = await execute('\n                INSERT INTO test_types (text_col, int_col, bigint_col, float_col, double_col, bool_col)\n                VALUES ($1, $2, $3, $4, $5, $6)\n                ', ['test', 42, 9999999999, 3.14, 2.718281828, True])
            expect(count).to_equal(1)
            results = await execute('SELECT * FROM test_types')
            expect(len(results)).to_equal(1)
            row = results[0]
            expect(row['text_col']).to_equal('test')
            expect(row['int_col']).to_equal(42)
            expect(row['bigint_col']).to_equal(9999999999)
            expect(abs(row['float_col'] - 3.14) < 0.01).to_be_true()
            expect(abs(row['double_col'] - 2.718281828) < 1e-06).to_be_true()
            expect(row['bool_col']).to_equal(True)
        finally:
            await execute('DROP TABLE IF EXISTS test_types')

    @test
    async def test_execute_aggregate_query(self, test_table):
        """Test aggregate query."""
        await execute('\n            INSERT INTO test_execute_users (name, age)\n            VALUES ($1, $2), ($3, $4), ($5, $6)\n            ', ['User1', 20, 'User2', 30, 'User3', 40])
        results = await execute('\n            SELECT\n                COUNT(*) as total,\n                AVG(age) as avg_age,\n                MIN(age) as min_age,\n                MAX(age) as max_age\n            FROM test_execute_users\n            WHERE age > $1\n            ', [0])
        expect(len(results)).to_equal(1)
        stats = results[0]
        expect(stats['total']).to_equal(3)
        expect(stats['avg_age']).to_equal(30)
        expect(stats['min_age']).to_equal(20)
        expect(stats['max_age']).to_equal(40)

    @test
    async def test_execute_ddl_create_index(self, test_table):
        """Test DDL operation (CREATE INDEX)."""
        result = await execute('CREATE INDEX IF NOT EXISTS idx_test_execute_users_age ON test_execute_users(age)')
        expect(result).to_be_none()
        results = await execute("\n            SELECT indexname FROM pg_indexes\n            WHERE tablename = 'test_execute_users' AND indexname = $1\n            ", ['idx_test_execute_users_age'])
        expect(len(results)).to_equal(1)

    @test
    async def test_execute_with_query(self, test_table):
        """Test WITH (CTE) query."""
        await execute('\n            INSERT INTO test_execute_users (name, age)\n            VALUES ($1, $2), ($3, $4), ($5, $6)\n            ', ['Young1', 18, 'Young2', 19, 'Old1', 50])
        results = await execute('\n            WITH young_users AS (\n                SELECT * FROM test_execute_users WHERE age < $1\n            )\n            SELECT COUNT(*) as count FROM young_users\n            ', [20])
        expect(len(results)).to_equal(1)
        expect(results[0]['count']).to_equal(2)

    @test
    async def test_execute_parameterized_prevents_injection(self, test_table):
        """Test that parameterized queries prevent SQL injection."""
        await execute('INSERT INTO test_execute_users (name, age) VALUES ($1, $2)', ['Victim', 25])
        malicious_input = "'; DROP TABLE test_execute_users; --"
        results = await execute('SELECT * FROM test_execute_users WHERE name = $1', [malicious_input])
        expect(len(results)).to_equal(0)
        all_results = await execute('SELECT * FROM test_execute_users')
        expect(len(all_results)).to_equal(1)
        expect(all_results[0]['name']).to_equal('Victim')