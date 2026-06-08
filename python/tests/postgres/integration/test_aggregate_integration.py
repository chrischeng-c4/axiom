"""
Integration tests for aggregate query functionality.

Tests query_aggregate with a real PostgreSQL database.
"""
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
from cclab.titan import execute, insert_one, query_aggregate

@fixture
async def orders_table():
    """Create and populate an orders table for aggregate testing."""
    await execute('\n        CREATE TABLE IF NOT EXISTS orders (\n            id SERIAL PRIMARY KEY,\n            user_id INTEGER NOT NULL,\n            amount DECIMAL(10, 2) NOT NULL,\n            status VARCHAR(50) NOT NULL,\n            created_at TIMESTAMP DEFAULT NOW()\n        )\n        ')
    test_data = [{'user_id': 1, 'amount': 100.5, 'status': 'completed'}, {'user_id': 1, 'amount': 200.0, 'status': 'completed'}, {'user_id': 1, 'amount': 50.25, 'status': 'pending'}, {'user_id': 2, 'amount': 150.0, 'status': 'completed'}, {'user_id': 2, 'amount': 300.75, 'status': 'completed'}, {'user_id': 3, 'amount': 75.0, 'status': 'cancelled'}]
    for data in test_data:
        await insert_one('orders', data)
    yield
    await execute('DROP TABLE IF EXISTS orders')

class TestBasicAggregates(PostgresSuite):
    """Test basic aggregate functions."""

    @test
    async def test_count_all(self, orders_table):
        """Test COUNT(*) - count all rows."""
        results = await query_aggregate('orders', [('count', None, 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(6)

    @test
    async def test_count_column(self, orders_table):
        """Test COUNT(column) - count non-null values."""
        results = await query_aggregate('orders', [('count_column', 'user_id', 'user_count')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(results[0]['user_count']).to_equal(6)

    @test
    async def test_count_distinct(self, orders_table):
        """Test COUNT(DISTINCT column) - count unique values."""
        results = await query_aggregate('orders', [('count_distinct', 'user_id', 'unique_users')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(results[0]['unique_users']).to_equal(3)

    @test
    async def test_sum(self, orders_table):
        """Test SUM(column) - sum of all values."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total_amount')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(float(results[0]['total_amount'])).to_be_close_to(876.5, rel=0.01)

    @test
    async def test_avg(self, orders_table):
        """Test AVG(column) - average of values."""
        results = await query_aggregate('orders', [('avg', 'amount', 'avg_amount')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(float(results[0]['avg_amount'])).to_be_close_to(146.08, rel=0.01)

    @test
    async def test_min(self, orders_table):
        """Test MIN(column) - minimum value."""
        results = await query_aggregate('orders', [('min', 'amount', 'min_amount')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(float(results[0]['min_amount'])).to_be_close_to(50.25, rel=0.01)

    @test
    async def test_max(self, orders_table):
        """Test MAX(column) - maximum value."""
        results = await query_aggregate('orders', [('max', 'amount', 'max_amount')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(float(results[0]['max_amount'])).to_be_close_to(300.75, rel=0.01)

class TestGroupBy(PostgresSuite):
    """Test GROUP BY functionality."""

    @test
    async def test_group_by_single_column(self, orders_table):
        """Test GROUP BY with single column."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total'), ('count', None, 'count')], group_by=['user_id'], having=None, where_conditions=None, order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_equal(3)
        expect(results[0]['user_id']).to_equal(1)
        expect(float(results[0]['total'])).to_be_close_to(350.75, rel=0.01)
        expect(results[0]['count']).to_equal(3)
        expect(results[1]['user_id']).to_equal(2)
        expect(float(results[1]['total'])).to_be_close_to(450.75, rel=0.01)
        expect(results[1]['count']).to_equal(2)
        expect(results[2]['user_id']).to_equal(3)
        expect(float(results[2]['total'])).to_be_close_to(75.0, rel=0.01)
        expect(results[2]['count']).to_equal(1)

    @test
    async def test_group_by_multiple_columns(self, orders_table):
        """Test GROUP BY with multiple columns."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total')], group_by=['user_id', 'status'], having=None, where_conditions=None, order_by=[('user_id', 'asc'), ('status', 'asc')], limit=None)
        expect(len(results)).to_be_greater_than_or_equal(3)

class TestWhereConditions(PostgresSuite):
    """Test WHERE clause filtering with aggregates."""

    @test
    async def test_where_single_condition(self, orders_table):
        """Test aggregate with single WHERE condition."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total')], group_by=None, having=None, where_conditions=[('status', 'eq', 'completed')], order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(float(results[0]['total'])).to_be_close_to(751.25, rel=0.01)

    @test
    async def test_where_multiple_conditions(self, orders_table):
        """Test aggregate with multiple WHERE conditions (AND)."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total')], group_by=None, having=None, where_conditions=[('status', 'eq', 'completed'), ('amount', 'gt', 150)], order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(float(results[0]['total'])).to_be_close_to(500.75, rel=0.01)

    @test
    async def test_where_with_group_by(self, orders_table):
        """Test WHERE clause combined with GROUP BY."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total'), ('count', None, 'count')], group_by=['user_id'], having=None, where_conditions=[('status', 'eq', 'completed')], order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_equal(2)
        expect(results[0]['user_id']).to_equal(1)
        expect(float(results[0]['total'])).to_be_close_to(300.5, rel=0.01)
        expect(results[0]['count']).to_equal(2)
        expect(results[1]['user_id']).to_equal(2)
        expect(float(results[1]['total'])).to_be_close_to(450.75, rel=0.01)
        expect(results[1]['count']).to_equal(2)

class TestOrderByAndLimit(PostgresSuite):
    """Test ORDER BY and LIMIT with aggregates."""

    @test
    async def test_order_by_aggregate(self, orders_table):
        """Test ordering by aggregate result."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total')], group_by=['user_id'], having=None, where_conditions=None, order_by=[('total', 'desc')], limit=None)
        expect(len(results)).to_equal(3)
        expect(results[0]['user_id']).to_equal(2)
        expect(results[1]['user_id']).to_equal(1)
        expect(results[2]['user_id']).to_equal(3)

    @test
    async def test_limit_results(self, orders_table):
        """Test LIMIT on aggregate results."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total')], group_by=['user_id'], having=None, where_conditions=None, order_by=[('total', 'desc')], limit=2)
        expect(len(results)).to_equal(2)
        expect(results[0]['user_id']).to_equal(2)
        expect(results[1]['user_id']).to_equal(1)

class TestMultipleAggregates(PostgresSuite):
    """Test multiple aggregate functions in single query."""

    @test
    async def test_multiple_aggregates_no_group(self, orders_table):
        """Test multiple aggregates without GROUP BY."""
        results = await query_aggregate('orders', [('count', None, 'count'), ('sum', 'amount', 'total'), ('avg', 'amount', 'avg'), ('min', 'amount', 'min'), ('max', 'amount', 'max')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(results[0]['count']).to_equal(6)
        expect(float(results[0]['total'])).to_be_close_to(876.5, rel=0.01)
        expect(float(results[0]['avg'])).to_be_close_to(146.08, rel=0.01)
        expect(float(results[0]['min'])).to_be_close_to(50.25, rel=0.01)
        expect(float(results[0]['max'])).to_be_close_to(300.75, rel=0.01)

    @test
    async def test_multiple_aggregates_with_group(self, orders_table):
        """Test multiple aggregates with GROUP BY."""
        results = await query_aggregate('orders', [('count', None, 'order_count'), ('sum', 'amount', 'total_amount'), ('avg', 'amount', 'avg_amount')], group_by=['user_id'], having=None, where_conditions=None, order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_equal(3)
        for result in results:
            expect('user_id').to_be_in(result)
            expect('order_count').to_be_in(result)
            expect('total_amount').to_be_in(result)
            expect('avg_amount').to_be_in(result)

class TestOperatorVariations(PostgresSuite):
    """Test different operator string variations."""

    @test
    async def test_eq_operator_variations(self, orders_table):
        """Test 'eq' and '=' both work."""
        results1 = await query_aggregate('orders', [('count', None, 'count')], group_by=None, having=None, where_conditions=[('status', 'eq', 'completed')], order_by=None, limit=None)
        results2 = await query_aggregate('orders', [('count', None, 'count')], group_by=None, having=None, where_conditions=[('status', '=', 'completed')], order_by=None, limit=None)
        expect(results1[0]['count'] == results2[0]['count'] == 4).to_be_true()

    @test
    async def test_comparison_operators(self, orders_table):
        """Test gt, gte, lt, lte operators."""
        # Test data: 100.5, 200.0, 50.25, 150.0, 300.75, 75.0
        # gt 100: 100.5, 200.0, 150.0, 300.75 = 4 orders
        results = await query_aggregate('orders', [('count', None, 'count')], group_by=None, having=None, where_conditions=[('amount', 'gt', 100)], order_by=None, limit=None)
        expect(results[0]['count']).to_equal(4)
        # gte 100: same as gt since no amount equals exactly 100
        results = await query_aggregate('orders', [('count', None, 'count')], group_by=None, having=None, where_conditions=[('amount', 'gte', 100)], order_by=None, limit=None)
        expect(results[0]['count']).to_equal(4)
        # lt 100: 50.25, 75.0 = 2 orders
        results = await query_aggregate('orders', [('count', None, 'count')], group_by=None, having=None, where_conditions=[('amount', 'lt', 100)], order_by=None, limit=None)
        expect(results[0]['count']).to_equal(2)

class TestHavingClause(PostgresSuite):
    """Test HAVING clause functionality."""

    @test
    async def test_having_simple_condition(self, orders_table):
        """Test HAVING with simple aggregate condition."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total'), ('count', None, 'count')], group_by=['user_id'], having=[('sum', 'amount', 'gt', 400)], where_conditions=None, order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_equal(1)
        expect(results[0]['user_id']).to_equal(2)
        expect(float(results[0]['total'])).to_be_close_to(450.75, rel=0.01)

    @test
    async def test_having_count_condition(self, orders_table):
        """Test HAVING with COUNT condition."""
        results = await query_aggregate('orders', [('count', None, 'order_count'), ('sum', 'amount', 'total')], group_by=['user_id'], having=[('count', None, 'gte', 2)], where_conditions=None, order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_equal(2)
        expect(results[0]['user_id']).to_equal(1)
        expect(results[0]['order_count']).to_equal(3)
        expect(results[1]['user_id']).to_equal(2)
        expect(results[1]['order_count']).to_equal(2)

    @test
    async def test_having_avg_condition(self, orders_table):
        """Test HAVING with AVG condition."""
        results = await query_aggregate('orders', [('avg', 'amount', 'avg_amount'), ('count', None, 'count')], group_by=['user_id'], having=[('avg', 'amount', 'gt', 150)], where_conditions=None, order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_be_greater_than_or_equal(1)
        expect(results[0]['user_id']).to_equal(2)

    @test
    async def test_having_with_where(self, orders_table):
        """Test HAVING combined with WHERE clause."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total'), ('count', None, 'count')], group_by=['user_id'], having=[('sum', 'amount', 'gt', 200)], where_conditions=[('status', 'eq', 'completed')], order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_equal(2)
        expect(results[0]['user_id']).to_equal(1)
        expect(results[1]['user_id']).to_equal(2)

    @test
    async def test_having_multiple_conditions(self, orders_table):
        """Test HAVING with multiple conditions."""
        results = await query_aggregate('orders', [('sum', 'amount', 'total'), ('count', None, 'count')], group_by=['user_id'], having=[('sum', 'amount', 'gt', 300), ('count', None, 'gte', 2)], where_conditions=None, order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_equal(2)
        expect(results[0]['user_id']).to_equal(1)
        expect(results[1]['user_id']).to_equal(2)

    @test
    async def test_having_min_max(self, orders_table):
        """Test HAVING with MIN/MAX aggregate functions."""
        results = await query_aggregate('orders', [('max', 'amount', 'max_amount'), ('count', None, 'count')], group_by=['user_id'], having=[('max', 'amount', 'gt', 250)], where_conditions=None, order_by=[('user_id', 'asc')], limit=None)
        expect(len(results)).to_equal(1)
        expect(results[0]['user_id']).to_equal(2)
        expect(float(results[0]['max_amount'])).to_be_close_to(300.75, rel=0.01)

class TestErrorHandling(PostgresSuite):
    """Test error handling and validation."""

    @test
    async def test_invalid_table_name(self):
        """Test error on invalid table name."""
        raised = False
        try:
            await query_aggregate('nonexistent_table', [('count', None, 'count')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        except Exception:
            raised = True
        expect(raised).to_be_true()

    @test
    async def test_invalid_column_name(self, orders_table):
        """Test error on invalid column name."""
        raised = False
        try:
            await query_aggregate('orders', [('sum', 'nonexistent_column', 'total')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        except Exception:
            raised = True
        expect(raised).to_be_true()

    @test
    async def test_invalid_aggregate_function(self):
        """Test error on unknown aggregate function."""
        try:
            await query_aggregate('orders', [('invalid_func', 'amount', 'result')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
            raise AssertionError('Expected ValueError')
        except ValueError:
            pass

    @test
    async def test_invalid_operator(self, orders_table):
        """Test error on unknown operator."""
        try:
            await query_aggregate('orders', [('count', None, 'count')], group_by=None, having=None, where_conditions=[('status', 'invalid_op', 'completed')], order_by=None, limit=None)
            raise AssertionError('Expected ValueError')
        except ValueError:
            pass