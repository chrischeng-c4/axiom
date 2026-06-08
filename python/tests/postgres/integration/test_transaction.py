"""Unit tests for PostgreSQL transaction support."""
from cclab.titan import connection
from cclab.probe import expect, test
from tests.postgres.base import PostgresSuite

class TestTransaction(PostgresSuite):

    @test
    async def test_transaction_basic_flow(self):
        """Test basic transaction flow without database."""
        expect(hasattr(connection, 'begin_transaction')).to_be_true()
        expect(callable(connection.begin_transaction)).to_be_true()

    @test
    async def test_transaction_isolation_levels(self):
        """Test that isolation levels are correctly typed."""
        from cclab.titan.connection import IsolationLevel
        from typing import get_args
        expected_levels = ('read_uncommitted', 'read_committed', 'repeatable_read', 'serializable')
        actual_levels = get_args(IsolationLevel)
        expect(set(actual_levels)).to_equal(set(expected_levels))

    @test
    async def test_transaction_commit(self, test_table):
        """Test transaction commit with live database."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'test_commit', 'email': 'commit@test.com', 'age': 20})
            await tx.commit()
        result = await connection.execute(f'SELECT * FROM {test_table} WHERE name = $1', ['test_commit'])
        expect(len(result)).to_equal(1)
        expect(result[0]['name']).to_equal('test_commit')

    @test
    async def test_transaction_rollback(self, test_table):
        """Test transaction rollback with live database."""
        async with connection.begin_transaction() as tx:
            await connection.insert_one(test_table, {'name': 'test_rollback', 'email': 'rollback@test.com', 'age': 20})
            await tx.rollback()
        result = await connection.execute(f'SELECT * FROM {test_table} WHERE name = $1', ['test_rollback'])
        expect(len(result)).to_equal(0)

    @test
    async def test_transaction_auto_rollback_on_exception(self, test_table):
        """Test automatic rollback on exception."""
        try:
            async with connection.begin_transaction() as tx:
                await connection.insert_one(test_table, {'name': 'test_exception', 'email': 'exception@test.com', 'age': 20})
                raise ValueError('Simulated error')
        except ValueError:
            pass
        result = await connection.execute(f'SELECT * FROM {test_table} WHERE name = $1', ['test_exception'])
        expect(len(result)).to_equal(0)

    @test
    async def test_transaction_isolation_level_serializable(self, test_table):
        """Test transaction with serializable isolation level."""
        async with connection.begin_transaction('serializable') as tx:
            await connection.insert_one(test_table, {'name': 'test_serializable', 'email': 'serial@test.com', 'age': 20})
            await tx.commit()
        result = await connection.execute(f'SELECT * FROM {test_table} WHERE name = $1', ['test_serializable'])
        expect(len(result)).to_equal(1)