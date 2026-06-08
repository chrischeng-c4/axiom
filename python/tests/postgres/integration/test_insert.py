"""
Integration tests for PostgreSQL insert operations.

Tests cover:
- Single document insert (insert_one)
- Bulk insert (insert_many)
- Auto-generated IDs
- Error handling for duplicate keys
"""
from cclab.titan import init, close, execute, insert_one, insert_many
from cclab.probe import expect, test
from tests.postgres.base import PostgresSuite

class TestInsertOne(PostgresSuite):
    """Test single document insert operations."""

    @test
    async def test_insert_one_basic(self):
        """
        Test insert_one inserting a new row with auto-generated ID.

        Verifies that insert_one correctly inserts a document and returns
        the inserted row with the auto-generated ID.
        """
        await execute('\n            CREATE TABLE test_insert_users (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                age INTEGER\n            )\n        ')
        result = await insert_one('test_insert_users', {'name': 'Alice', 'age': 30})
        expect(result['name']).to_equal('Alice')
        expect(result['age']).to_equal(30)
        expect(result['id']).not_to_be_none()
        expect(isinstance(result['id'], int)).to_be_true()
        rows = await execute('SELECT * FROM test_insert_users')
        expect(len(rows)).to_equal(1)
        expect(rows[0]['name']).to_equal('Alice')
        expect(rows[0]['age']).to_equal(30)
        expect(rows[0]['id']).to_equal(result['id'])

    @test
    async def test_insert_one_with_unique_constraint(self):
        """
        Test insert_one with unique constraint violation.

        Verifies that insert_one raises appropriate error when trying to
        insert a duplicate value for a unique column.
        """
        await execute('\n            CREATE TABLE test_insert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT\n            )\n        ')
        result1 = await insert_one('test_insert_users', {'email': 'alice@example.com', 'name': 'Alice'})
        expect(result1['email']).to_equal('alice@example.com')
        exception_raised = False
        try:
            await insert_one('test_insert_users', {'email': 'alice@example.com', 'name': 'Alice Duplicate'})
        except AssertionError:
            raise
        except Exception as e:
            exception_raised = True
            # Error message should indicate insert failure
            error_msg = str(e).lower()
            expect('insert' in error_msg or 'failed' in error_msg or 'error' in error_msg).to_be_true()
        expect(exception_raised).to_be_true()

    @test
    async def test_insert_one_nullable_columns(self):
        """
        Test insert_one with nullable columns.

        Verifies that insert_one correctly handles NULL values for nullable columns.
        """
        await execute('\n            CREATE TABLE test_insert_nullable (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                email TEXT,\n                age INTEGER\n            )\n        ')
        result = await insert_one('test_insert_nullable', {'name': 'Bob'})
        expect(result['name']).to_equal('Bob')
        expect(result['email']).to_be_none()
        expect(result['age']).to_be_none()
        expect(result['id']).not_to_be_none()
        rows = await execute('SELECT * FROM test_insert_nullable')
        expect(len(rows)).to_equal(1)
        expect(rows[0]['name']).to_equal('Bob')
        expect(rows[0]['email']).to_be_none()
        expect(rows[0]['age']).to_be_none()

class TestInsertMany(PostgresSuite):
    """Test bulk insert operations."""

    @test
    async def test_insert_many_basic(self):
        """
        Test insert_many inserting multiple new rows.

        Verifies that insert_many can efficiently insert multiple documents
        in a single operation.
        """
        await execute('\n            CREATE TABLE test_insert_users (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                age INTEGER\n            )\n        ')
        users = [{'name': 'Alice', 'age': 30}, {'name': 'Bob', 'age': 25}, {'name': 'Charlie', 'age': 35}]
        results = await insert_many('test_insert_users', users)
        expect(len(results)).to_equal(3)
        expect(all((r['id'] is not None for r in results))).to_be_true()
        expect({r['name'] for r in results}).to_equal({'Alice', 'Bob', 'Charlie'})
        ids = [r['id'] for r in results]
        expect(len(set(ids))).to_equal(3)
        rows = await execute('SELECT * FROM test_insert_users ORDER BY name')
        expect(len(rows)).to_equal(3)
        expect(rows[0]['name']).to_equal('Alice')
        expect(rows[1]['name']).to_equal('Bob')
        expect(rows[2]['name']).to_equal('Charlie')

    @test
    async def test_insert_many_large_batch(self):
        """
        Test insert_many with large batch (100+ rows).

        Verifies that bulk inserts work correctly with large batches.
        """
        await execute('\n            CREATE TABLE test_insert_users (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                age INTEGER\n            )\n        ')
        large_batch = [{'name': f'User {i}', 'age': 20 + i % 50} for i in range(100)]
        results = await insert_many('test_insert_users', large_batch)
        expect(len(results)).to_equal(100)
        expect(all((r['id'] is not None for r in results))).to_be_true()
        ids = [r['id'] for r in results]
        expect(len(set(ids))).to_equal(100)
        rows = await execute('SELECT COUNT(*) as count FROM test_insert_users')
        expect(rows[0]['count']).to_equal(100)

    @test
    async def test_insert_many_empty_list(self):
        """
        Test insert_many with empty document list.

        Verifies that insert_many handles empty input gracefully.
        """
        await execute('\n            CREATE TABLE test_insert_users (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                age INTEGER\n            )\n        ')
        results = await insert_many('test_insert_users', [])
        expect(results).to_equal([])
        rows = await execute('SELECT * FROM test_insert_users')
        expect(len(rows)).to_equal(0)

    @test
    async def test_insert_many_with_unique_constraint_violation(self):
        """
        Test insert_many with unique constraint violation.

        Verifies that insert_many raises appropriate error when batch
        contains duplicate values for a unique column.
        """
        await execute('\n            CREATE TABLE test_insert_users (\n                id SERIAL PRIMARY KEY,\n                email TEXT UNIQUE NOT NULL,\n                name TEXT\n            )\n        ')
        users = [{'email': 'alice@example.com', 'name': 'Alice'}, {'email': 'bob@example.com', 'name': 'Bob'}, {'email': 'alice@example.com', 'name': 'Alice Duplicate'}]
        exception_raised = False
        try:
            await insert_many('test_insert_users', users)
        except AssertionError:
            raise
        except Exception as e:
            exception_raised = True
            # Error message should indicate insert failure
            error_msg = str(e).lower()
            expect('insert' in error_msg or 'failed' in error_msg or 'error' in error_msg).to_be_true()
        expect(exception_raised).to_be_true()

class TestInsertErrors(PostgresSuite):
    """Test error handling for insert operations."""

    @test
    async def test_insert_table_not_exists(self):
        """
        Test insert operations on non-existent table.

        Verifies that appropriate error is raised when table doesn't exist.
        """
        exception_raised = False
        try:
            await insert_one('nonexistent_table', {'name': 'Alice'})
        except AssertionError:
            raise
        except Exception as e:
            exception_raised = True
            # Error message should indicate insert failure
            error_msg = str(e).lower()
            expect('insert' in error_msg or 'failed' in error_msg or 'error' in error_msg).to_be_true()
        expect(exception_raised).to_be_true()

    @test
    async def test_insert_missing_required_column(self):
        """
        Test insert_one with missing required (NOT NULL) column.

        Verifies that appropriate error is raised when a required column
        is not provided in the document.
        """
        await execute('\n            CREATE TABLE test_insert_users (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                email TEXT NOT NULL\n            )\n        ')
        exception_raised = False
        try:
            await insert_one('test_insert_users', {'name': 'Alice'})
        except AssertionError:
            raise
        except Exception as e:
            exception_raised = True
            # Error message should indicate insert failure
            error_msg = str(e).lower()
            expect('insert' in error_msg or 'failed' in error_msg or 'error' in error_msg).to_be_true()
        expect(exception_raised).to_be_true()