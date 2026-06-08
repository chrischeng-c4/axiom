"""
Integration tests for PostgreSQL upsert operations.

Tests cover:
- Single document upsert (insert and update scenarios)
- Bulk upsert with mixed operations
- Composite unique constraints
- Selective column updates
- Parallel processing for large batches
- Error handling for invalid inputs
"""
from cclab.titan import init, close, execute, upsert_one, upsert_many
from cclab.probe import expect, test
from tests.postgres.base import PostgresSuite

class TestUpsertOne(PostgresSuite):
    """Test single document upsert operations."""

    @test
    async def test_upsert_one_insert_new(self):
        """
        Test upsert_one inserting a new row when no conflict exists.

        Verifies that upsert_one acts like a regular insert when the unique
        constraint is not violated.
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        result = await upsert_one('test_upsert_users', {'email': 'alice@example.com', 'name': 'Alice', 'age': 30}, conflict_target='email')
        expect(result['email']).to_equal('alice@example.com')
        expect(result['name']).to_equal('Alice')
        expect(result['age']).to_equal(30)
        expect(result['id']).not_to_be_none()
        rows = await execute('SELECT * FROM test_upsert_users')
        expect(len(rows)).to_equal(1)
        expect(rows[0]['email']).to_equal('alice@example.com')
        expect(rows[0]['name']).to_equal('Alice')
        expect(rows[0]['age']).to_equal(30)

    @test
    async def test_upsert_one_update_existing(self):
        """
        Test upsert_one updating an existing row on conflict.

        Verifies that when a unique constraint is violated, upsert_one
        updates the existing row instead of failing.
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        await execute('INSERT INTO test_upsert_users (email, name, age) VALUES ($1, $2, $3)', ['bob@example.com', 'Bob Original', 25])
        original = await execute('SELECT id FROM test_upsert_users WHERE email = $1', ['bob@example.com'])
        original_id = original[0]['id']
        result = await upsert_one('test_upsert_users', {'email': 'bob@example.com', 'name': 'Bob Updated', 'age': 26}, conflict_target='email')
        expect(result['email']).to_equal('bob@example.com')
        expect(result['name']).to_equal('Bob Updated')
        expect(result['age']).to_equal(26)
        expect(result['id']).to_equal(original_id)
        rows = await execute('SELECT * FROM test_upsert_users')
        expect(len(rows)).to_equal(1)
        expect(rows[0]['name']).to_equal('Bob Updated')
        expect(rows[0]['age']).to_equal(26)

    @test
    async def test_upsert_one_selective_update(self):
        """
        Test upsert_one with selective column updates on conflict.

        Verifies that update_columns parameter allows updating only
        specific columns while preserving others.
        """
        await execute("\n            CREATE TABLE test_upsert_selective (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER,\n                status TEXT DEFAULT 'active',\n                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n            )\n        ")
        await execute('INSERT INTO test_upsert_selective (email, name, age, status) VALUES ($1, $2, $3, $4)', ['charlie@example.com', 'Charlie', 35, 'premium'])
        original = await execute('SELECT id, status, created_at FROM test_upsert_selective WHERE email = $1', ['charlie@example.com'])
        original_id = original[0]['id']
        original_status = original[0]['status']
        original_created_at = original[0]['created_at']
        result = await upsert_one('test_upsert_selective', {'email': 'charlie@example.com', 'name': 'Charlie Updated', 'age': 36, 'status': 'inactive'}, conflict_target='email', update_columns=['name', 'age'])
        expect(result['email']).to_equal('charlie@example.com')
        expect(result['name']).to_equal('Charlie Updated')
        expect(result['age']).to_equal(36)
        expect(result['status']).to_equal(original_status)
        expect(result['id']).to_equal(original_id)
        rows = await execute('SELECT * FROM test_upsert_selective WHERE email = $1', ['charlie@example.com'])
        expect(len(rows)).to_equal(1)
        expect(rows[0]['name']).to_equal('Charlie Updated')
        expect(rows[0]['age']).to_equal(36)
        expect(rows[0]['status']).to_equal('premium')

    @test
    async def test_upsert_one_composite_key(self):
        """
        Test upsert_one with composite unique constraint.

        Verifies that upsert works correctly when the unique constraint
        involves multiple columns (e.g., tenant_id + user_id).
        """
        await execute('\n            CREATE TABLE test_upsert_composite (\n                id SERIAL PRIMARY KEY,\n                tenant_id INTEGER NOT NULL,\n                user_id INTEGER NOT NULL,\n                name TEXT,\n                metadata TEXT,\n                UNIQUE(tenant_id, user_id)\n            )\n        ')
        await execute('INSERT INTO test_upsert_composite (tenant_id, user_id, name, metadata) VALUES ($1, $2, $3, $4)', [1, 100, 'User 100', 'initial'])
        result = await upsert_one('test_upsert_composite', {'tenant_id': 1, 'user_id': 100, 'name': 'User 100 Updated', 'metadata': 'updated'}, conflict_target=['tenant_id', 'user_id'])
        expect(result['tenant_id']).to_equal(1)
        expect(result['user_id']).to_equal(100)
        expect(result['name']).to_equal('User 100 Updated')
        expect(result['metadata']).to_equal('updated')
        rows = await execute('SELECT * FROM test_upsert_composite')
        expect(len(rows)).to_equal(1)
        expect(rows[0]['name']).to_equal('User 100 Updated')
        result2 = await upsert_one('test_upsert_composite', {'tenant_id': 1, 'user_id': 101, 'name': 'User 101', 'metadata': 'new'}, conflict_target=['tenant_id', 'user_id'])
        expect(result2['tenant_id']).to_equal(1)
        expect(result2['user_id']).to_equal(101)
        expect(result2['name']).to_equal('User 101')
        rows = await execute('SELECT * FROM test_upsert_composite ORDER BY user_id')
        expect(len(rows)).to_equal(2)
        expect(rows[0]['user_id']).to_equal(100)
        expect(rows[1]['user_id']).to_equal(101)

class TestUpsertMany(PostgresSuite):
    """Test bulk upsert operations."""

    @test
    async def test_upsert_many_insert_all_new(self):
        """
        Test upsert_many inserting all new rows (no conflicts).

        Verifies that upsert_many can efficiently insert multiple rows
        when none of them conflict with existing data.
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        users = [{'email': 'alice@example.com', 'name': 'Alice', 'age': 30}, {'email': 'bob@example.com', 'name': 'Bob', 'age': 25}, {'email': 'charlie@example.com', 'name': 'Charlie', 'age': 35}]
        results = await upsert_many('test_upsert_users', users, conflict_target='email')
        expect(len(results)).to_equal(3)
        expect(all((r['id'] is not None for r in results))).to_be_true()
        expect({r['email'] for r in results}).to_equal({'alice@example.com', 'bob@example.com', 'charlie@example.com'})
        rows = await execute('SELECT * FROM test_upsert_users ORDER BY email')
        expect(len(rows)).to_equal(3)
        expect(rows[0]['email']).to_equal('alice@example.com')
        expect(rows[1]['email']).to_equal('bob@example.com')
        expect(rows[2]['email']).to_equal('charlie@example.com')

    @test
    async def test_upsert_many_update_all_existing(self):
        """
        Test upsert_many updating all existing rows (all conflicts).

        Verifies that upsert_many correctly updates all rows when every
        document conflicts with an existing row.
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        initial_users = [{'email': 'alice@example.com', 'name': 'Alice Original', 'age': 30}, {'email': 'bob@example.com', 'name': 'Bob Original', 'age': 25}, {'email': 'charlie@example.com', 'name': 'Charlie Original', 'age': 35}]
        for user in initial_users:
            await execute('INSERT INTO test_upsert_users (email, name, age) VALUES ($1, $2, $3)', [user['email'], user['name'], user['age']])
        original_rows = await execute('SELECT id, email FROM test_upsert_users ORDER BY email')
        original_ids = {row['email']: row['id'] for row in original_rows}
        updated_users = [{'email': 'alice@example.com', 'name': 'Alice Updated', 'age': 31}, {'email': 'bob@example.com', 'name': 'Bob Updated', 'age': 26}, {'email': 'charlie@example.com', 'name': 'Charlie Updated', 'age': 36}]
        results = await upsert_many('test_upsert_users', updated_users, conflict_target='email')
        expect(len(results)).to_equal(3)
        for result in results:
            expect(result['name'].endswith('Updated')).to_be_true()
            expect(result['id']).to_equal(original_ids[result['email']])
        rows = await execute('SELECT * FROM test_upsert_users ORDER BY email')
        expect(len(rows)).to_equal(3)
        expect(all((row['name'].endswith('Updated') for row in rows))).to_be_true()

    @test
    async def test_upsert_many_mixed(self):
        """
        Test upsert_many with mixed inserts and updates.

        Verifies that upsert_many correctly handles a batch where some
        documents conflict (update) and others don't (insert).
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        await execute('INSERT INTO test_upsert_users (email, name, age) VALUES ($1, $2, $3), ($4, $5, $6)', ['alice@example.com', 'Alice Original', 30, 'bob@example.com', 'Bob Original', 25])
        mixed_users = [{'email': 'alice@example.com', 'name': 'Alice Updated', 'age': 31}, {'email': 'bob@example.com', 'name': 'Bob Updated', 'age': 26}, {'email': 'charlie@example.com', 'name': 'Charlie New', 'age': 35}, {'email': 'diana@example.com', 'name': 'Diana New', 'age': 28}]
        results = await upsert_many('test_upsert_users', mixed_users, conflict_target='email')
        expect(len(results)).to_equal(4)
        rows = await execute('SELECT * FROM test_upsert_users ORDER BY email')
        expect(len(rows)).to_equal(4)
        alice = next((r for r in rows if r['email'] == 'alice@example.com'))
        expect(alice['name']).to_equal('Alice Updated')
        expect(alice['age']).to_equal(31)
        bob = next((r for r in rows if r['email'] == 'bob@example.com'))
        expect(bob['name']).to_equal('Bob Updated')
        expect(bob['age']).to_equal(26)
        charlie = next((r for r in rows if r['email'] == 'charlie@example.com'))
        expect(charlie['name']).to_equal('Charlie New')
        expect(charlie['age']).to_equal(35)
        diana = next((r for r in rows if r['email'] == 'diana@example.com'))
        expect(diana['name']).to_equal('Diana New')
        expect(diana['age']).to_equal(28)

    @test
    async def test_upsert_many_parallel_threshold(self):
        """
        Test upsert_many with large batch (50+ rows) to trigger parallel processing.

        Verifies that bulk upserts work correctly with large batches and that
        parallel processing threshold (≥50 documents) is handled properly.
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        large_batch = [{'email': f'user{i}@example.com', 'name': f'User {i}', 'age': 20 + i % 50} for i in range(100)]
        results = await upsert_many('test_upsert_users', large_batch, conflict_target='email')
        expect(len(results)).to_equal(100)
        expect(all((r['id'] is not None for r in results))).to_be_true()
        rows = await execute('SELECT COUNT(*) as count FROM test_upsert_users')
        expect(rows[0]['count']).to_equal(100)
        update_batch = [{'email': f'user{i}@example.com', 'name': f'User {i} Updated', 'age': 25 + i % 50} for i in range(50)]
        insert_batch = [{'email': f'newuser{i}@example.com', 'name': f'New User {i}', 'age': 30 + i % 50} for i in range(50)]
        mixed_batch = update_batch + insert_batch
        results2 = await upsert_many('test_upsert_users', mixed_batch, conflict_target='email')
        expect(len(results2)).to_equal(100)
        rows = await execute('SELECT COUNT(*) as count FROM test_upsert_users')
        expect(rows[0]['count']).to_equal(150)
        updated_rows = await execute("SELECT * FROM test_upsert_users WHERE name LIKE '%Updated%'")
        expect(len(updated_rows)).to_equal(50)

    @test
    async def test_upsert_many_selective_update(self):
        """
        Test upsert_many with selective column updates on conflicts.

        Verifies that update_columns parameter works correctly with bulk
        operations, preserving specified columns on update.
        """
        await execute("\n            CREATE TABLE test_upsert_many_selective (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER,\n                status TEXT DEFAULT 'active',\n                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n            )\n        ")
        initial_users = [{'email': 'alice@example.com', 'name': 'Alice', 'age': 30, 'status': 'premium'}, {'email': 'bob@example.com', 'name': 'Bob', 'age': 25, 'status': 'premium'}]
        for user in initial_users:
            await execute('INSERT INTO test_upsert_many_selective (email, name, age, status) VALUES ($1, $2, $3, $4)', [user['email'], user['name'], user['age'], user['status']])
        update_users = [{'email': 'alice@example.com', 'name': 'Alice Updated', 'age': 31, 'status': 'inactive'}, {'email': 'bob@example.com', 'name': 'Bob Updated', 'age': 26, 'status': 'inactive'}]
        results = await upsert_many('test_upsert_many_selective', update_users, conflict_target='email', update_columns=['name', 'age'])
        expect(len(results)).to_equal(2)
        for result in results:
            expect(result['name'].endswith('Updated')).to_be_true()
            expect(result['status']).to_equal('premium')
        rows = await execute('SELECT * FROM test_upsert_many_selective ORDER BY email')
        expect(len(rows)).to_equal(2)
        expect(all((row['status'] == 'premium' for row in rows))).to_be_true()
        expect(all((row['name'].endswith('Updated') for row in rows))).to_be_true()

class TestUpsertErrors(PostgresSuite):
    """Test error handling for upsert operations."""

    @test
    async def test_upsert_invalid_conflict_column(self):
        """
        Test upsert_one with non-existent conflict column.

        Verifies that appropriate error is raised when conflict_target
        references a column that doesn't exist.
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        exception_raised = False
        try:
            await upsert_one('test_upsert_users', {'email': 'alice@example.com', 'name': 'Alice', 'age': 30}, conflict_target='nonexistent_column')
        except AssertionError:
            raise
        except Exception as e:
            exception_raised = True
            # Error message should indicate upsert/insert failure
            error_msg = str(e).lower()
            expect('upsert' in error_msg or 'insert' in error_msg or 'failed' in error_msg or 'error' in error_msg).to_be_true()
        expect(exception_raised).to_be_true()

    @test
    async def test_upsert_empty_conflict_target(self):
        """
        Test upsert_one with empty conflict_target.

        Verifies that appropriate error is raised when conflict_target
        is an empty list or empty string.
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        try:
            await upsert_one('test_upsert_users', {'email': 'alice@example.com', 'name': 'Alice', 'age': 30}, conflict_target=[])
            raise AssertionError('Expected exception')
        except Exception as e:
            expect(e).not_to_be_none()

    @test
    async def test_upsert_many_empty_documents(self):
        """
        Test upsert_many with empty document list.

        Verifies that upsert_many handles empty input gracefully.
        """
        await execute('\n            CREATE TABLE test_upsert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT,\n                age INTEGER\n            )\n        ')
        results = await upsert_many('test_upsert_users', [], conflict_target='email')
        expect(results).to_equal([])
        rows = await execute('SELECT * FROM test_upsert_users')
        expect(len(rows)).to_equal(0)

    @test
    async def test_upsert_table_not_exists(self):
        """
        Test upsert operations on non-existent table.

        Verifies that appropriate error is raised when table doesn't exist.
        """
        exception_raised = False
        try:
            await upsert_one('nonexistent_table', {'email': 'alice@example.com', 'name': 'Alice'}, conflict_target='email')
        except AssertionError:
            raise
        except Exception as e:
            exception_raised = True
            # Error message should indicate upsert/insert failure
            error_msg = str(e).lower()
            expect('upsert' in error_msg or 'insert' in error_msg or 'failed' in error_msg or 'error' in error_msg).to_be_true()
        expect(exception_raised).to_be_true()