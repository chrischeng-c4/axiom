"""Integration tests for PostgreSQL savepoint functionality."""
from cclab.titan import connection, execute
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
class TestSavepoints(PostgresSuite):

    @test
    @fixture
    async def test_table(self):
        """
    Create a simple test table for savepoint testing.

    Returns:
        str: The table name ('savepoint_test')
    """
        await execute('\n        CREATE TABLE IF NOT EXISTS savepoint_test (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL,\n            value INTEGER,\n            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n        )\n    ')
        yield 'savepoint_test'

    @test
    async def test_savepoint_create(self, test_table):
        """Test creating a savepoint within a transaction."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'initial', 'value': 1})
            await tx._tx.savepoint('sp1')
            await connection.insert_one(test_table, {'name': 'after_savepoint', 'value': 2})
            await tx._tx.release_savepoint('sp1')
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(2)
        expect(result[0]['name']).to_equal('initial')
        expect(result[1]['name']).to_equal('after_savepoint')

    @test
    async def test_savepoint_rollback(self, test_table):
        """Test rolling back to a savepoint undoes changes after it."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'before', 'value': 1})
            await tx._tx.savepoint('sp1')
            await connection.insert_one(test_table, {'name': 'after', 'value': 2})
            result = await tx.execute(f'SELECT * FROM {test_table} WHERE name = $1', ['after'])
            expect(len(result)).to_equal(1)
            await tx._tx.rollback_to_savepoint('sp1')
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(1)
        expect(result[0]['name']).to_equal('before')
        result = await execute(f'SELECT * FROM {test_table} WHERE name = $1', ['after'])
        expect(len(result)).to_equal(0)

    @test
    async def test_savepoint_release(self, test_table):
        """Test releasing a savepoint keeps changes."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'before', 'value': 1})
            await tx._tx.savepoint('sp1')
            await connection.insert_one(test_table, {'name': 'after', 'value': 2})
            await tx._tx.release_savepoint('sp1')
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(2)
        expect(result[0]['name']).to_equal('before')
        expect(result[1]['name']).to_equal('after')

    @test
    async def test_savepoint_nested(self, test_table):
        """Test multiple nested savepoints."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'initial', 'value': 0})
            await tx._tx.savepoint('sp1')
            await connection.insert_one(test_table, {'name': 'sp1_data', 'value': 1})
            await tx._tx.savepoint('sp2')
            await connection.insert_one(test_table, {'name': 'sp2_data', 'value': 2})
            await tx._tx.rollback_to_savepoint('sp1')
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(1)
        expect(result[0]['name']).to_equal('initial')
        result = await execute(f'SELECT * FROM {test_table} WHERE name = $1', ['sp1_data'])
        expect(len(result)).to_equal(0)
        result = await execute(f'SELECT * FROM {test_table} WHERE name = $1', ['sp2_data'])
        expect(len(result)).to_equal(0)

    @test
    async def test_savepoint_commit_after_rollback(self, test_table):
        """Test transaction commits successfully after savepoint rollback."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'initial', 'value': 1})
            await tx._tx.savepoint('sp1')
            await connection.insert_one(test_table, {'name': 'rollback_me', 'value': 2})
            await tx._tx.rollback_to_savepoint('sp1')
            await connection.insert_one(test_table, {'name': 'after_rollback', 'value': 3})
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(2)
        expect(result[0]['name']).to_equal('initial')
        expect(result[1]['name']).to_equal('after_rollback')
        result = await execute(f'SELECT * FROM {test_table} WHERE name = $1', ['rollback_me'])
        expect(len(result)).to_equal(0)

    @test
    async def test_savepoint_multiple_sequential(self, test_table):
        """Test creating multiple savepoints sequentially."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'initial', 'value': 0})
            await tx._tx.savepoint('sp1')
            await connection.insert_one(test_table, {'name': 'sp1_data', 'value': 1})
            await tx._tx.release_savepoint('sp1')
            await tx._tx.savepoint('sp2')
            await connection.insert_one(test_table, {'name': 'sp2_data', 'value': 2})
            await tx._tx.rollback_to_savepoint('sp2')
            await tx._tx.savepoint('sp3')
            await connection.insert_one(test_table, {'name': 'sp3_data', 'value': 3})
            await tx._tx.release_savepoint('sp3')
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(3)
        expect(result[0]['name']).to_equal('initial')
        expect(result[1]['name']).to_equal('sp1_data')
        expect(result[2]['name']).to_equal('sp3_data')
        result = await execute(f'SELECT * FROM {test_table} WHERE name = $1', ['sp2_data'])
        expect(len(result)).to_equal(0)

    @test
    async def test_savepoint_complex_workflow(self, test_table):
        """Test complex workflow with multiple savepoints and operations."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'user1', 'value': 100})
            await tx._tx.savepoint('risky_operation')
            await connection.insert_one(test_table, {'name': 'risky1', 'value': 200})
            await connection.insert_one(test_table, {'name': 'risky2', 'value': 300})
            await tx._tx.rollback_to_savepoint('risky_operation')
            await tx._tx.savepoint('safe_operation')
            await connection.insert_one(test_table, {'name': 'safe1', 'value': 400})
            await connection.insert_one(test_table, {'name': 'safe2', 'value': 500})
            await tx._tx.release_savepoint('safe_operation')
            await connection.insert_one(test_table, {'name': 'final', 'value': 600})
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(4)
        expect(result[0]['name']).to_equal('user1')
        expect(result[1]['name']).to_equal('safe1')
        expect(result[2]['name']).to_equal('safe2')
        expect(result[3]['name']).to_equal('final')
        result = await execute(f"SELECT * FROM {test_table} WHERE name LIKE 'risky%'")
        expect(len(result)).to_equal(0)

    @test
    async def test_savepoint_error_handling(self, test_table):
        """Test that savepoint errors are properly handled."""
        async with connection.begin_transaction() as tx:
            await tx._tx.savepoint('sp1')
            try:
                await tx._tx.rollback_to_savepoint('non_existent')
                raise AssertionError('Expected exception')
            except Exception:
                pass

    @test
    async def test_savepoint_transaction_isolation(self, test_table):
        """Test that savepoint changes are isolated within transaction."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'data1', 'value': 1})
            await tx._tx.savepoint('sp1')
            await connection.insert_one(test_table, {'name': 'data2', 'value': 2})
            result = await tx.execute(f'SELECT * FROM {test_table} WHERE name = $1', ['data2'])
            expect(len(result)).to_equal(1)
            await tx._tx.rollback_to_savepoint('sp1')
            result = await tx.execute(f'SELECT * FROM {test_table} WHERE name = $1', ['data2'])
            expect(len(result)).to_equal(0)
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table}')
        expect(len(result)).to_equal(1)
        expect(result[0]['name']).to_equal('data1')

    @test
    async def test_savepoint_release_after_release(self, test_table):
        """Test that releasing a savepoint twice causes an error."""
        async with connection.begin_transaction() as tx:
            await tx._tx.savepoint('sp1')
            await tx._tx.release_savepoint('sp1')
            try:
                await tx._tx.release_savepoint('sp1')
                raise AssertionError('Expected exception')
            except Exception:
                pass

    @test
    async def test_savepoint_with_py_transaction_api(self, test_table):
        """Test using savepoint with the higher-level pg_transaction API."""
        from cclab.titan.transactions import pg_transaction
        async with pg_transaction() as tx:
            await tx._tx.insert_one(test_table, {'name': 'initial', 'value': 1})
            sp = await tx.savepoint('sp1')
            await tx._tx.insert_one(test_table, {'name': 'after', 'value': 2})
            await sp.release()
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(2)
        expect(result[0]['name']).to_equal('initial')
        expect(result[1]['name']).to_equal('after')

    @test
    async def test_savepoint_context_manager_with_pg_transaction(self, test_table):
        """Test using savepoint as async context manager with pg_transaction."""
        from cclab.titan.transactions import pg_transaction
        async with pg_transaction() as tx:
            await tx._tx.insert_one(test_table, {'name': 'before', 'value': 1})
            async with await tx.savepoint('sp1'):
                await tx._tx.insert_one(test_table, {'name': 'in_savepoint', 'value': 2})
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(2)
        expect(result[0]['name']).to_equal('before')
        expect(result[1]['name']).to_equal('in_savepoint')

    @test
    async def test_savepoint_context_manager_exception_with_pg_transaction(self, test_table):
        """Test savepoint context manager auto-rollback on exception with pg_transaction."""
        from cclab.titan.transactions import pg_transaction
        async with pg_transaction() as tx:
            await tx._tx.insert_one(test_table, {'name': 'before', 'value': 1})
            try:
                async with await tx.savepoint('sp1'):
                    await tx._tx.insert_one(test_table, {'name': 'exception_data', 'value': 2})
                    raise ValueError('Simulated error')
            except ValueError:
                pass
            await tx.commit()
        result = await execute(f'SELECT * FROM {test_table} ORDER BY id')
        expect(len(result)).to_equal(1)
        expect(result[0]['name']).to_equal('before')
        result = await execute(f'SELECT * FROM {test_table} WHERE name = $1', ['exception_data'])
        expect(len(result)).to_equal(0)