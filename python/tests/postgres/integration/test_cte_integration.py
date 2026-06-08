"""
Integration tests for Common Table Expressions (CTEs / WITH clauses).

Tests query_with_cte with real PostgreSQL database.
"""
from datetime import datetime, timezone
from cclab.titan import execute, insert_one, query_with_cte
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite

@fixture
async def test_tables():
    """Create and populate test tables for CTE testing."""
    await execute('\n        CREATE TABLE IF NOT EXISTS users (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(100) NOT NULL,\n            email VARCHAR(100) NOT NULL,\n            status VARCHAR(50) NOT NULL\n        )\n        ')
    await execute('\n        CREATE TABLE IF NOT EXISTS orders (\n            id SERIAL PRIMARY KEY,\n            user_id INTEGER NOT NULL,\n            amount DECIMAL(10, 2) NOT NULL,\n            status VARCHAR(50) NOT NULL,\n            created_at TIMESTAMP DEFAULT NOW()\n        )\n        ')
    users_data = [{'name': 'Alice', 'email': 'alice@example.com', 'status': 'active'}, {'name': 'Bob', 'email': 'bob@example.com', 'status': 'active'}, {'name': 'Charlie', 'email': 'charlie@example.com', 'status': 'inactive'}, {'name': 'David', 'email': 'david@example.com', 'status': 'active'}, {'name': 'Eve', 'email': 'eve@example.com', 'status': 'active'}]
    for data in users_data:
        await insert_one('users', data)
    orders_data = [{'user_id': 1, 'amount': 100.0, 'status': 'completed'}, {'user_id': 1, 'amount': 200.0, 'status': 'completed'}, {'user_id': 1, 'amount': 50.0, 'status': 'pending'}, {'user_id': 2, 'amount': 150.0, 'status': 'completed'}, {'user_id': 2, 'amount': 75.0, 'status': 'pending'}, {'user_id': 3, 'amount': 25.0, 'status': 'cancelled'}, {'user_id': 4, 'amount': 300.0, 'status': 'completed'}, {'user_id': 4, 'amount': 250.0, 'status': 'completed'}]
    for data in orders_data:
        await insert_one('orders', data)
    yield
    await execute('DROP TABLE IF EXISTS orders')
    await execute('DROP TABLE IF EXISTS users')

class TestCteIntegration(PostgresSuite):
    pass

class TestSimpleCTE(PostgresSuite):
    """Test simple CTE functionality."""

    @test
    async def test_simple_cte_no_params(self, test_tables):
        """Test simple CTE without parameters."""
        results = await query_with_cte('completed_orders', [('completed_orders', "SELECT * FROM orders WHERE status = 'completed'", [])], select_columns=['user_id', 'amount'], order_by=[('amount', 'desc')])
        expect(len(results)).to_equal(5)
        expect(results[0]['amount']).to_equal(300.0)

    @test
    async def test_simple_cte_with_params(self, test_tables):
        """Test simple CTE with parameters."""
        results = await query_with_cte('high_value_orders', [('high_value_orders', 'SELECT * FROM orders WHERE amount > $1', [100.0])], select_columns=['user_id', 'amount', 'status'], order_by=[('amount', 'asc')])
        expect(len(results)).to_equal(4)
        expect(results[0]['amount']).to_equal(150.0)
        expect(results[-1]['amount']).to_equal(300.0)

    @test
    async def test_cte_with_aggregation(self, test_tables):
        """Test CTE that performs aggregation."""
        results = await query_with_cte('user_totals', [('user_totals', 'SELECT user_id, SUM(amount) as total, COUNT(*) as order_count FROM orders GROUP BY user_id', [])], select_columns=['user_id', 'total', 'order_count'], order_by=[('total', 'desc')])
        expect(len(results)).to_equal(4)
        expect(results[0]['user_id']).to_equal(4)
        expect(float(results[0]['total'])).to_equal(550.0)
        expect(results[0]['order_count']).to_equal(2)

class TestCTEWithWhereConditions(PostgresSuite):
    """Test CTE combined with WHERE conditions."""

    @test
    async def test_cte_with_simple_where(self, test_tables):
        """Test CTE with WHERE clause on main query."""
        results = await query_with_cte('completed_orders', [('completed_orders', 'SELECT * FROM orders WHERE status = $1', ['completed'])], select_columns=['user_id', 'amount'], where_conditions=[('amount', 'gt', 150)], order_by=[('amount', 'asc')])
        expect(len(results)).to_equal(3)
        expect(results[0]['amount']).to_equal(200.0)
        expect(results[-1]['amount']).to_equal(300.0)

    @test
    async def test_cte_with_multiple_where(self, test_tables):
        """Test CTE with multiple WHERE conditions."""
        results = await query_with_cte('all_orders', [('all_orders', 'SELECT * FROM orders', [])], select_columns=['user_id', 'amount', 'status'], where_conditions=[('status', 'eq', 'completed'), ('amount', 'gte', 100)], order_by=[('user_id', 'asc')])
        expect(len(results)).to_equal(5)

class TestCTEWithOrderAndLimit(PostgresSuite):
    """Test CTE with ORDER BY and LIMIT."""

    @test
    async def test_cte_with_order_by(self, test_tables):
        """Test CTE with ORDER BY."""
        results = await query_with_cte('all_orders', [('all_orders', 'SELECT * FROM orders', [])], select_columns=['user_id', 'amount'], order_by=[('amount', 'desc'), ('user_id', 'asc')])
        expect(len(results)).to_equal(8)
        expect(results[0]['amount']).to_equal(300.0)
        expect(results[-1]['amount']).to_equal(25.0)

    @test
    async def test_cte_with_limit(self, test_tables):
        """Test CTE with LIMIT."""
        results = await query_with_cte('all_orders', [('all_orders', 'SELECT * FROM orders WHERE status = $1', ['completed'])], select_columns=['amount'], order_by=[('amount', 'desc')], limit=3)
        expect(len(results)).to_equal(3)
        expect(results[0]['amount']).to_equal(300.0)
        expect(results[1]['amount']).to_equal(250.0)
        expect(results[2]['amount']).to_equal(200.0)

class TestMultipleCTEs(PostgresSuite):
    """Test multiple CTEs in a single query."""

    @test
    async def test_two_ctes_independent(self, test_tables):
        """Test two independent CTEs."""
        results = await query_with_cte('high_value', [('completed', 'SELECT * FROM orders WHERE status = $1', ['completed']), ('high_value', 'SELECT * FROM completed WHERE amount > $1', [150])], select_columns=['user_id', 'amount'], order_by=[('amount', 'asc')])
        expect(len(results)).to_equal(3)
        expect(results[0]['amount']).to_equal(200.0)
        expect(results[-1]['amount']).to_equal(300.0)

    @test
    async def test_cte_referencing_another_cte(self, test_tables):
        """Test CTE that references another CTE."""
        results = await query_with_cte('active_customers', [('user_orders', 'SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id', []), ('active_customers', 'SELECT user_id FROM user_orders WHERE order_count > $1', [1])], select_columns=['user_id'], order_by=[('user_id', 'asc')])
        expect(len(results)).to_equal(3)
        expect(results[0]['user_id']).to_equal(1)
        expect(results[1]['user_id']).to_equal(2)
        expect(results[2]['user_id']).to_equal(4)

    @test
    async def test_multiple_ctes_with_aggregation(self, test_tables):
        """Test multiple CTEs with different aggregations."""
        results = await query_with_cte('summary', [('completed_orders', 'SELECT user_id, SUM(amount) as total FROM orders WHERE status = $1 GROUP BY user_id', ['completed']), ('summary', 'SELECT user_id, total FROM completed_orders WHERE total > $1', [200])], select_columns=['user_id', 'total'], order_by=[('total', 'desc')])
        expect(len(results)).to_equal(2)
        expect(results[0]['user_id']).to_equal(4)
        expect(float(results[0]['total'])).to_equal(550.0)
        expect(results[1]['user_id']).to_equal(1)
        expect(float(results[1]['total'])).to_equal(300.0)

class TestCTEWithSubquery(PostgresSuite):
    """Test CTE combined with subqueries."""

    @test
    async def test_cte_with_in_subquery(self, test_tables):
        """Test CTE with IN subquery."""
        results = await query_with_cte('users', [('high_value_orders', 'SELECT user_id FROM orders WHERE amount > $1', [100.0])], select_columns=['name', 'email'], subqueries=[('in', 'id', 'SELECT user_id FROM high_value_orders', [])], order_by=[('name', 'asc')])
        expect(len(results)).to_equal(3)
        names = [r['name'] for r in results]
        expect('Alice').to_be_in(names)
        expect('Bob').to_be_in(names)
        expect('David').to_be_in(names)

    @test
    async def test_cte_with_exists_subquery(self, test_tables):
        """Test CTE with EXISTS subquery."""
        results = await query_with_cte('users', [('completed_orders', 'SELECT user_id FROM orders WHERE status = $1', ['completed'])], select_columns=['name', 'status'], subqueries=[('exists', None, 'SELECT 1 FROM completed_orders WHERE completed_orders.user_id = users.id', [])], where_conditions=[('status', 'eq', 'active')], order_by=[('name', 'asc')])
        expect(len(results)).to_equal(3)
        names = [r['name'] for r in results]
        expect('Alice').to_be_in(names)
        expect('Bob').to_be_in(names)
        expect('David').to_be_in(names)

    @test
    async def test_cte_with_not_in_subquery(self, test_tables):
        """Test CTE with NOT IN subquery."""
        results = await query_with_cte('users', [('order_users', 'SELECT DISTINCT user_id FROM orders', [])], select_columns=['name', 'email'], subqueries=[('not_in', 'id', 'SELECT user_id FROM order_users', [])], order_by=[('name', 'asc')])
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Eve')

class TestCTEJoiningTables(PostgresSuite):
    """Test CTE that joins multiple tables."""

    @test
    async def test_cte_with_join(self, test_tables):
        """Test CTE that performs a JOIN."""
        results = await query_with_cte('user_orders', [('user_orders', '\n                SELECT u.name, u.email, o.amount, o.status\n                FROM users u\n                JOIN orders o ON u.id = o.user_id\n                WHERE u.status = $1\n                ', ['active'])], select_columns=['name', 'amount', 'status'], order_by=[('name', 'asc'), ('amount', 'desc')])
        expect(len(results)).to_be_greater_than_or_equal(5)
        names = [r['name'] for r in results]
        expect('Charlie').to_not_be_in(names)
        expect('Alice').to_be_in(names)
        expect('Bob').to_be_in(names)

    @test
    async def test_cte_with_complex_join(self, test_tables):
        """Test CTE with complex join and aggregation."""
        results = await query_with_cte('user_stats', [('user_stats', '\n                SELECT\n                    u.id,\n                    u.name,\n                    COUNT(o.id) as order_count,\n                    COALESCE(SUM(o.amount), 0) as total_spent\n                FROM users u\n                LEFT JOIN orders o ON u.id = o.user_id AND o.status = $1\n                GROUP BY u.id, u.name\n                ', ['completed'])], select_columns=['name', 'order_count', 'total_spent'], where_conditions=[('order_count', 'gt', 0)], order_by=[('total_spent', 'desc')])
        expect(len(results)).to_be_greater_than_or_equal(3)
        expect(results[0]['name']).to_equal('David')
        expect(float(results[0]['total_spent'])).to_equal(550.0)

class TestRecursiveCTE(PostgresSuite):
    """Test recursive CTEs."""

    @test
    async def test_simple_recursive_cte(self, test_tables):
        """Test simple recursive CTE (number sequence)."""
        results = await query_with_cte('numbers', [('numbers', '\n                WITH RECURSIVE numbers AS (\n                    SELECT 1 as n\n                    UNION ALL\n                    SELECT n + 1 FROM numbers WHERE n < 10\n                )\n                SELECT * FROM numbers\n                ', [])], select_columns=['n'], order_by=[('n', 'asc')])
        expect(len(results)).to_equal(10)
        expect(results[0]['n']).to_equal(1)
        expect(results[-1]['n']).to_equal(10)

    @test
    async def test_recursive_cte_with_data(self):
        """Test recursive CTE with real data."""
        await execute('DROP TABLE IF EXISTS categories')
        await execute('\n            CREATE TABLE categories (\n                id SERIAL PRIMARY KEY,\n                name VARCHAR(100) NOT NULL,\n                parent_id INTEGER\n            )\n            ')
        try:
            await execute("\n                INSERT INTO categories (name, parent_id) VALUES\n                    ('Electronics', NULL),\n                    ('Computers', 1),\n                    ('Laptops', 2),\n                    ('Desktops', 2),\n                    ('Phones', 1)\n                ")
            results = await query_with_cte('category_tree', [('category_tree', '\n                    WITH RECURSIVE category_tree AS (\n                        SELECT id, name, parent_id, 1 as level\n                        FROM categories\n                        WHERE id = $1\n                        UNION ALL\n                        SELECT c.id, c.name, c.parent_id, ct.level + 1\n                        FROM categories c\n                        JOIN category_tree ct ON c.parent_id = ct.id\n                    )\n                    SELECT * FROM category_tree\n                    ', [1])], select_columns=['id', 'name', 'level'], order_by=[('level', 'asc'), ('id', 'asc')])
            expect(len(results)).to_equal(5)
            electronics = next((r for r in results if r['name'] == 'Electronics'))
            expect(electronics['level']).to_equal(1)
            computers = next((r for r in results if r['name'] == 'Computers'))
            expect(computers['level']).to_equal(2)
            laptops = next((r for r in results if r['name'] == 'Laptops'))
            expect(laptops['level']).to_equal(3)
        finally:
            await execute('DROP TABLE IF EXISTS categories')

class TestCTEEdgeCases(PostgresSuite):
    """Test edge cases and special scenarios."""

    @fixture
    async def edge_test_tables(self):
        """Create fresh test tables for edge case testing."""
        await execute('\n            CREATE TABLE IF NOT EXISTS users (\n                id SERIAL PRIMARY KEY,\n                name VARCHAR(100) NOT NULL,\n                email VARCHAR(100) NOT NULL,\n                status VARCHAR(50) NOT NULL\n            )\n            ')
        await execute('\n            CREATE TABLE IF NOT EXISTS orders (\n                id SERIAL PRIMARY KEY,\n                user_id INTEGER NOT NULL,\n                amount DECIMAL(10, 2) NOT NULL,\n                status VARCHAR(50) NOT NULL,\n                created_at TIMESTAMP DEFAULT NOW()\n            )\n            ')
        users_data = [{'name': 'Alice', 'email': 'alice@example.com', 'status': 'active'}, {'name': 'Bob', 'email': 'bob@example.com', 'status': 'active'}, {'name': 'David', 'email': 'david@example.com', 'status': 'active'}]
        for data in users_data:
            await insert_one('users', data)
        orders_data = [{'user_id': 1, 'amount': 100.0, 'status': 'completed'}, {'user_id': 1, 'amount': 200.0, 'status': 'completed'}, {'user_id': 2, 'amount': 150.0, 'status': 'completed'}, {'user_id': 3, 'amount': 300.0, 'status': 'completed'}, {'user_id': 3, 'amount': 250.0, 'status': 'completed'}]
        for data in orders_data:
            await insert_one('orders', data)
        yield

    @test
    async def test_cte_with_no_results(self, edge_test_tables):
        """Test CTE that returns no results."""
        results = await query_with_cte('empty_cte', [('empty_cte', 'SELECT * FROM orders WHERE amount > $1', [10000])], select_columns=['user_id', 'amount'])
        expect(len(results)).to_equal(0)

    @test
    async def test_cte_with_all_features(self, edge_test_tables):
        """Test CTE with all features combined."""
        results = await query_with_cte('high_value_orders', [('completed', 'SELECT * FROM orders WHERE status = $1', ['completed']), ('high_value_orders', 'SELECT * FROM completed WHERE amount > $1', [100])], select_columns=['user_id', 'amount'], where_conditions=[('amount', 'lt', 300)], subqueries=[('in', 'user_id', 'SELECT id FROM users WHERE status = $1', ['active'])], order_by=[('amount', 'desc')], limit=3)
        expect(len(results)).to_be_less_than_or_equal(3)
        for result in results:
            expect(100 < result['amount'] < 300).to_be_true()

    @test
    async def test_cte_select_all_columns(self, test_tables):
        """Test CTE with select_columns=None (SELECT *)."""
        results = await query_with_cte('all_orders', [('all_orders', 'SELECT * FROM orders WHERE status = $1', ['completed'])], select_columns=None, limit=1)
        expect(len(results)).to_equal(1)
        result = results[0]
        expect('id').to_be_in(result)
        expect('user_id').to_be_in(result)
        expect('amount').to_be_in(result)
        expect('status').to_be_in(result)
        expect('created_at').to_be_in(result)

    @test
    async def test_cte_with_null_values(self, test_tables):
        """Test CTE handling of NULL values."""
        await execute('DROP TABLE IF EXISTS products')
        await execute('\n            CREATE TABLE products (\n                id SERIAL PRIMARY KEY,\n                name VARCHAR(100) NOT NULL,\n                description TEXT,\n                price DECIMAL(10, 2)\n            )\n            ')
        await insert_one('products', {'name': 'Product A', 'description': None, 'price': None})
        await insert_one('products', {'name': 'Product B', 'description': 'Description', 'price': 99.99})
        results = await query_with_cte('all_products', [('all_products', 'SELECT * FROM products', [])], select_columns=['name', 'description', 'price'], order_by=[('name', 'asc')])
        expect(len(results)).to_equal(2)
        expect(results[0]['name']).to_equal('Product A')
        expect(results[0]['description']).to_be_none()
        expect(results[0]['price']).to_be_none()
        expect(results[1]['name']).to_equal('Product B')
        expect(results[1]['description']).to_equal('Description')
        expect(float(results[1]['price'])).to_equal(99.99)
        await execute('DROP TABLE IF EXISTS products')

class TestCTEErrorHandling(PostgresSuite):
    """Test error handling for CTEs."""

    @test
    async def test_invalid_cte_sql(self, test_tables):
        """Test error on invalid CTE SQL."""
        raised = False
        try:
            await query_with_cte('bad_cte', [('bad_cte', 'SELECT * FROM nonexistent_table', [])], select_columns=['id'])
        except Exception:
            raised = True
        expect(raised).to_be_true()

    @test
    async def test_invalid_main_table(self, test_tables):
        """Test error when main table doesn't exist and no CTE matches."""
        raised = False
        try:
            await query_with_cte('nonexistent_table', [('some_cte', 'SELECT * FROM orders', [])], select_columns=['id'])
        except Exception:
            raised = True
        expect(raised).to_be_true()

    @test
    async def test_cte_with_invalid_column(self, test_tables):
        """Test error when selecting non-existent column from CTE."""
        raised = False
        try:
            await query_with_cte('orders_cte', [('orders_cte', 'SELECT user_id, amount FROM orders', [])], select_columns=['nonexistent_column'])
        except Exception:
            raised = True
        expect(raised).to_be_true()

    @test
    async def test_cte_parameter_mismatch(self, test_tables):
        """Test error when CTE parameters don't match placeholders."""
        raised = False
        try:
            await query_with_cte('orders_cte', [('orders_cte', 'SELECT * FROM orders WHERE status = $1 AND amount > $2', ['completed'])], select_columns=['id'])
        except Exception:
            raised = True
        expect(raised).to_be_true()

class TestCTEPerformance(PostgresSuite):
    """Test CTE performance characteristics."""

    @test
    async def test_cte_materialization(self, test_tables):
        """Test that CTE is materialized and can be referenced multiple times."""
        results = await query_with_cte('user_stats', [('user_stats', '\n                SELECT\n                    user_id,\n                    COUNT(*) as order_count,\n                    SUM(amount) as total\n                FROM orders\n                GROUP BY user_id\n                ', [])], select_columns=['user_id', 'order_count', 'total'], where_conditions=[('order_count', 'gte', 2), ('total', 'gt', 200)], order_by=[('total', 'desc')])
        expect(len(results)).to_be_greater_than_or_equal(1)
        for result in results:
            expect(result['order_count']).to_be_greater_than_or_equal(2)
            expect(float(result['total'])).to_be_greater_than(200)

    @test
    async def test_cte_vs_subquery_equivalence(self, test_tables):
        """Test that CTE produces same results as equivalent subquery."""
        cte_results = await query_with_cte('high_spenders', [('high_spenders', 'SELECT user_id FROM orders GROUP BY user_id HAVING SUM(amount) > $1', [200])], select_columns=['user_id'], order_by=[('user_id', 'asc')])
        subquery_results = await execute('\n            SELECT user_id\n            FROM (\n                SELECT user_id\n                FROM orders\n                GROUP BY user_id\n                HAVING SUM(amount) > $1\n            ) AS high_spenders\n            ORDER BY user_id ASC\n            ', [200])
        expect(len(cte_results)).to_equal(len(subquery_results))
        for cte_row, sub_row in zip(cte_results, subquery_results):
            expect(cte_row['user_id']).to_equal(sub_row['user_id'])