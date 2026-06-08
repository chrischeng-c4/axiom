"""
Integration tests for subquery support.

Tests subquery functionality (IN, NOT IN, EXISTS, NOT EXISTS) with real PostgreSQL database.
"""
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
from cclab.titan import execute, insert_one, query_aggregate, query_with_cte

@fixture
async def test_tables():
    """Create and populate test tables for subquery testing."""
    await execute('\n        CREATE TABLE IF NOT EXISTS users (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(100) NOT NULL,\n            email VARCHAR(100) NOT NULL,\n            status VARCHAR(50) NOT NULL\n        )\n        ')
    await execute('\n        CREATE TABLE IF NOT EXISTS orders (\n            id SERIAL PRIMARY KEY,\n            user_id INTEGER NOT NULL,\n            total DECIMAL(10, 2) NOT NULL,\n            status VARCHAR(50) NOT NULL\n        )\n        ')
    users_data = [{'name': 'Alice', 'email': 'alice@example.com', 'status': 'active'}, {'name': 'Bob', 'email': 'bob@example.com', 'status': 'active'}, {'name': 'Charlie', 'email': 'charlie@example.com', 'status': 'inactive'}, {'name': 'David', 'email': 'david@example.com', 'status': 'active'}]
    for data in users_data:
        await insert_one('users', data)
    orders_data = [{'user_id': 1, 'total': 100.0, 'status': 'completed'}, {'user_id': 1, 'total': 200.0, 'status': 'pending'}, {'user_id': 2, 'total': 150.0, 'status': 'completed'}]
    for data in orders_data:
        await insert_one('orders', data)
    yield
    await execute('DROP TABLE IF EXISTS orders')
    await execute('DROP TABLE IF EXISTS users')

class TestSubquery(PostgresSuite):
    pass

class TestSubqueryIn(PostgresSuite):
    """Test IN subquery functionality."""

    @test
    async def test_where_in_subquery(self, test_tables):
        """Test WHERE column IN (subquery)."""
        results = await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('in', 'id', 'SELECT user_id FROM orders', [])])
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(2)

    @test
    async def test_where_in_subquery_with_params(self, test_tables):
        """Test WHERE column IN (subquery with parameters)."""
        results = await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('in', 'id', 'SELECT user_id FROM orders WHERE status = $1', ['completed'])])
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(2)

    @test
    async def test_where_not_in_subquery(self, test_tables):
        """Test WHERE column NOT IN (subquery)."""
        results = await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('not_in', 'id', 'SELECT user_id FROM orders', [])])
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(2)

class TestSubqueryExists(PostgresSuite):
    """Test EXISTS subquery functionality."""

    @test
    async def test_where_exists(self, test_tables):
        """Test WHERE EXISTS (subquery)."""
        results = await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('exists', None, 'SELECT 1 FROM orders WHERE orders.user_id = users.id', [])])
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(2)

    @test
    async def test_where_not_exists(self, test_tables):
        """Test WHERE NOT EXISTS (subquery)."""
        results = await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('not_exists', None, 'SELECT 1 FROM orders WHERE orders.user_id = users.id', [])])
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(2)

    @test
    async def test_where_exists_with_condition(self, test_tables):
        """Test WHERE EXISTS with additional conditions."""
        results = await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('exists', None, 'SELECT 1 FROM orders WHERE orders.user_id = users.id AND orders.total > $1', [150.0])])
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(1)

class TestSubqueryCombinations(PostgresSuite):
    """Test combining subqueries with other WHERE conditions."""

    @test
    async def test_subquery_with_where_conditions(self, test_tables):
        """Test subquery combined with regular WHERE conditions."""
        results = await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=[('status', 'eq', 'active')], order_by=None, limit=None, subqueries=[('in', 'id', 'SELECT user_id FROM orders', [])])
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(2)

    @test
    async def test_multiple_subqueries(self, test_tables):
        """Test multiple subquery conditions."""
        results = await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('in', 'id', 'SELECT user_id FROM orders', []), ('exists', None, 'SELECT 1 FROM orders WHERE orders.user_id = users.id AND orders.status = $1', ['completed'])])
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(2)

class TestSubqueryWithCTE(PostgresSuite):
    """Test subqueries with CTEs."""

    @test
    async def test_cte_with_subquery(self, test_tables):
        """Test query_with_cte with subquery support."""
        results = await query_with_cte('users', [('high_value_orders', 'SELECT user_id FROM orders WHERE total > $1', [100.0])], select_columns=['name', 'email'], where_conditions=None, order_by=None, limit=None, subqueries=[('in', 'id', 'SELECT user_id FROM high_value_orders', [])])
        expect(len(results)).to_equal(2)
        names = [r['name'] for r in results]
        expect('Alice').to_be_in(names)
        expect('Bob').to_be_in(names)

class TestSubqueryErrors(PostgresSuite):
    """Test error handling for subqueries."""

    @test
    async def test_invalid_subquery_type(self, test_tables):
        """Test that invalid subquery type raises error."""
        try:
            await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('invalid_type', 'id', 'SELECT user_id FROM orders', [])])
            raise AssertionError('Expected exception')
        except Exception as e:
            expect('Unknown subquery type').to_be_in(str(e))

    @test
    async def test_in_subquery_without_field(self, test_tables):
        """Test that IN subquery without field raises error."""
        try:
            await query_aggregate('users', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None, subqueries=[('in', None, 'SELECT user_id FROM orders', [])])
            raise AssertionError('Expected exception')
        except Exception as e:
            expect('IN subquery requires a field name').to_be_in(str(e))