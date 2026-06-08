"""
Integration tests for PostgreSQL eager loading (JOIN-based fetching).

Tests verify that:
- Single relations can be eagerly loaded
- Multiple relations can be loaded in one query
- NULL foreign keys are handled correctly
- Different join types work correctly
- Batch eager loading works efficiently
"""
from cclab.titan import init, close, execute, fetch_one_with_relations, fetch_one_eager, fetch_many_with_relations
from cclab.probe import expect, test
from tests.postgres.base import PostgresSuite

class TestFetchOneEager(PostgresSuite):
    """Test single row eager loading with simplified tuple-based API."""

    @test
    async def test_fetch_one_eager_basic(self):
        """Test basic eager loading with one relation."""
        await execute('\n            CREATE TABLE test_eager_authors (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_eager_posts (\n                id SERIAL PRIMARY KEY,\n                title TEXT NOT NULL,\n                author_id INTEGER REFERENCES test_eager_authors(id)\n            )\n        ')
        await execute('INSERT INTO test_eager_authors (name) VALUES ($1)', ['Alice'])
        await execute('INSERT INTO test_eager_posts (title, author_id) VALUES ($1, $2)', ['My Post', 1])
        result = await fetch_one_eager('test_eager_posts', 1, [('author', 'author_id', 'test_eager_authors')])
        expect(result).not_to_be_none()
        expect(result['title']).to_equal('My Post')
        expect(result['author_id']).to_equal(1)

    @test
    async def test_fetch_one_eager_not_found(self):
        """Test eager loading when row doesn't exist."""
        await execute('\n            CREATE TABLE test_eager_items (\n                id SERIAL PRIMARY KEY,\n                name TEXT\n            )\n        ')
        result = await fetch_one_eager('test_eager_items', 999, [])
        expect(result).to_be_none()

    @test
    async def test_fetch_one_eager_null_foreign_key(self):
        """Test eager loading with NULL foreign key (LEFT JOIN)."""
        await execute('\n            CREATE TABLE test_eager_categories (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_eager_products (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                category_id INTEGER REFERENCES test_eager_categories(id)\n            )\n        ')
        await execute('INSERT INTO test_eager_products (name, category_id) VALUES ($1, $2)', ['Widget', None])
        result = await fetch_one_eager('test_eager_products', 1, [('category', 'category_id', 'test_eager_categories')])
        expect(result).not_to_be_none()
        expect(result['name']).to_equal('Widget')
        expect(result['category_id']).to_be_none()

    @test
    async def test_fetch_one_eager_multiple_relations(self):
        """Test eager loading with multiple relations."""
        await execute('\n            CREATE TABLE test_eager_users (\n                id SERIAL PRIMARY KEY,\n                username TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_eager_profiles (\n                id SERIAL PRIMARY KEY,\n                bio TEXT\n            )\n        ')
        await execute('\n            CREATE TABLE test_eager_accounts (\n                id SERIAL PRIMARY KEY,\n                user_id INTEGER REFERENCES test_eager_users(id),\n                profile_id INTEGER REFERENCES test_eager_profiles(id),\n                status TEXT\n            )\n        ')
        await execute('INSERT INTO test_eager_users (username) VALUES ($1)', ['alice'])
        await execute('INSERT INTO test_eager_profiles (bio) VALUES ($1)', ['Developer'])
        await execute('INSERT INTO test_eager_accounts (user_id, profile_id, status) VALUES ($1, $2, $3)', [1, 1, 'active'])
        result = await fetch_one_eager('test_eager_accounts', 1, [('account_user', 'user_id', 'test_eager_users'), ('profile', 'profile_id', 'test_eager_profiles')])
        expect(result).not_to_be_none()
        expect(result['status']).to_equal('active')
        expect(result['user_id']).to_equal(1)
        expect(result['profile_id']).to_equal(1)

class TestFetchOneWithRelations(PostgresSuite):
    """Test fetch_one_with_relations with full configuration."""

    @test
    async def test_fetch_with_left_join_default(self):
        """Test LEFT JOIN (default) - includes rows even without matching relations."""
        await execute('\n            CREATE TABLE test_rel_departments (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_rel_employees (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                dept_id INTEGER REFERENCES test_rel_departments(id)\n            )\n        ')
        await execute('INSERT INTO test_rel_employees (name, dept_id) VALUES ($1, $2)', ['Bob', None])
        result = await fetch_one_with_relations('test_rel_employees', 1, [{'name': 'department', 'table': 'test_rel_departments', 'foreign_key': 'dept_id', 'reference_column': 'id', 'join_type': 'left'}])
        expect(result).not_to_be_none()
        expect(result['name']).to_equal('Bob')
        expect(result['dept_id']).to_be_none()

    @test
    async def test_fetch_with_inner_join(self):
        """Test INNER JOIN - should exclude rows without matching relations."""
        await execute('\n            CREATE TABLE test_rel_companies (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_rel_managers (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                company_id INTEGER REFERENCES test_rel_companies(id)\n            )\n        ')
        await execute('INSERT INTO test_rel_companies (name) VALUES ($1)', ['Acme Corp'])
        await execute('INSERT INTO test_rel_managers (name, company_id) VALUES ($1, $2)', ['Alice', 1])
        await execute('INSERT INTO test_rel_managers (name, company_id) VALUES ($1, $2)', ['Bob', None])
        result = await fetch_one_with_relations('test_rel_managers', 1, [{'name': 'company', 'table': 'test_rel_companies', 'foreign_key': 'company_id', 'reference_column': 'id', 'join_type': 'inner'}])
        expect(result).not_to_be_none()
        expect(result['name']).to_equal('Alice')
        expect(result['company_id']).to_equal(1)
        result_null = await fetch_one_with_relations('test_rel_managers', 2, [{'name': 'company', 'table': 'test_rel_companies', 'foreign_key': 'company_id', 'reference_column': 'id', 'join_type': 'inner'}])

    @test
    async def test_fetch_with_custom_reference_column(self):
        """Test custom reference column (not just 'id')."""
        await execute('\n            CREATE TABLE test_rel_countries (\n                id SERIAL PRIMARY KEY,\n                code TEXT UNIQUE NOT NULL,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_rel_cities (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                country_code TEXT REFERENCES test_rel_countries(code)\n            )\n        ')
        await execute('INSERT INTO test_rel_countries (code, name) VALUES ($1, $2)', ['US', 'United States'])
        await execute('INSERT INTO test_rel_cities (name, country_code) VALUES ($1, $2)', ['New York', 'US'])
        result = await fetch_one_with_relations('test_rel_cities', 1, [{'name': 'country', 'table': 'test_rel_countries', 'foreign_key': 'country_code', 'reference_column': 'code', 'join_type': 'left'}])
        expect(result).not_to_be_none()
        expect(result['name']).to_equal('New York')
        expect(result['country_code']).to_equal('US')

class TestFetchManyWithRelations(PostgresSuite):
    """Test fetch_many_with_relations for batch eager loading."""

    @test
    async def test_fetch_many_basic(self):
        """Test fetching multiple rows with relations."""
        await execute('\n            CREATE TABLE test_many_brands (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_many_cars (\n                id SERIAL PRIMARY KEY,\n                model TEXT NOT NULL,\n                brand_id INTEGER REFERENCES test_many_brands(id)\n            )\n        ')
        await execute('INSERT INTO test_many_brands (name) VALUES ($1)', ['Toyota'])
        await execute('INSERT INTO test_many_brands (name) VALUES ($1)', ['Honda'])
        await execute('INSERT INTO test_many_cars (model, brand_id) VALUES ($1, $2)', ['Camry', 1])
        await execute('INSERT INTO test_many_cars (model, brand_id) VALUES ($1, $2)', ['Accord', 2])
        await execute('INSERT INTO test_many_cars (model, brand_id) VALUES ($1, $2)', ['Corolla', 1])
        results = await fetch_many_with_relations('test_many_cars', [{'name': 'brand', 'table': 'test_many_brands', 'foreign_key': 'brand_id', 'reference_column': 'id', 'join_type': 'left'}], limit=10)
        expect(len(results)).to_equal(3)
        expect(all((r['brand_id'] is not None for r in results))).to_be_true()
        brand_ids = {r['brand_id'] for r in results}
        expect(len(brand_ids)).to_equal(2)

    @test
    async def test_fetch_many_with_filter(self):
        """Test fetching with filter and relations."""
        await execute('\n            CREATE TABLE test_many_stores (\n                id SERIAL PRIMARY KEY,\n                city TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_many_orders (\n                id SERIAL PRIMARY KEY,\n                total INTEGER NOT NULL,\n                store_id INTEGER REFERENCES test_many_stores(id)\n            )\n        ')
        await execute('INSERT INTO test_many_stores (city) VALUES ($1)', ['NYC'])
        await execute('INSERT INTO test_many_stores (city) VALUES ($1)', ['LA'])
        await execute('INSERT INTO test_many_orders (total, store_id) VALUES ($1, $2)', [100, 1])
        await execute('INSERT INTO test_many_orders (total, store_id) VALUES ($1, $2)', [200, 1])
        await execute('INSERT INTO test_many_orders (total, store_id) VALUES ($1, $2)', [150, 2])
        results = await fetch_many_with_relations('test_many_orders', [{'name': 'store', 'table': 'test_many_stores', 'foreign_key': 'store_id', 'reference_column': 'id', 'join_type': 'left'}], filter={'total': 100}, limit=10)
        expect(len(results)).to_equal(1)
        expect(results[0]['total']).to_equal(100)
        expect(results[0]['store_id']).to_equal(1)

    @test
    async def test_fetch_many_with_limit_offset(self):
        """Test pagination with limit and offset."""
        await execute('\n            CREATE TABLE test_many_categories (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_many_items (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                category_id INTEGER REFERENCES test_many_categories(id)\n            )\n        ')
        await execute('INSERT INTO test_many_categories (name) VALUES ($1)', ['Electronics'])
        for i in range(10):
            await execute('INSERT INTO test_many_items (name, category_id) VALUES ($1, $2)', [f'Item {i}', 1])
        page1 = await fetch_many_with_relations('test_many_items', [{'name': 'category', 'table': 'test_many_categories', 'foreign_key': 'category_id', 'reference_column': 'id', 'join_type': 'left'}], limit=5, offset=0)
        expect(len(page1)).to_equal(5)
        page2 = await fetch_many_with_relations('test_many_items', [{'name': 'category', 'table': 'test_many_categories', 'foreign_key': 'category_id', 'reference_column': 'id', 'join_type': 'left'}], limit=5, offset=5)
        expect(len(page2)).to_equal(5)
        page1_ids = {item['id'] for item in page1}
        page2_ids = {item['id'] for item in page2}
        expect(page1_ids.isdisjoint(page2_ids)).to_be_true()

    @test
    async def test_fetch_many_with_order_by(self):
        """Test ordering results."""
        await execute('\n            CREATE TABLE test_many_publishers (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_many_books (\n                id SERIAL PRIMARY KEY,\n                title TEXT NOT NULL,\n                year INTEGER,\n                publisher_id INTEGER REFERENCES test_many_publishers(id)\n            )\n        ')
        await execute('INSERT INTO test_many_publishers (name) VALUES ($1)', ['Acme Publishing'])
        await execute('INSERT INTO test_many_books (title, year, publisher_id) VALUES ($1, $2, $3)', ['Book A', 2020, 1])
        await execute('INSERT INTO test_many_books (title, year, publisher_id) VALUES ($1, $2, $3)', ['Book B', 2022, 1])
        await execute('INSERT INTO test_many_books (title, year, publisher_id) VALUES ($1, $2, $3)', ['Book C', 2019, 1])
        results = await fetch_many_with_relations('test_many_books', [{'name': 'publisher', 'table': 'test_many_publishers', 'foreign_key': 'publisher_id', 'reference_column': 'id', 'join_type': 'left'}], order_by=('year', 'asc'), limit=10)
        expect(len(results)).to_equal(3)
        expect(results[0]['year']).to_equal(2019)
        expect(results[1]['year']).to_equal(2020)
        expect(results[2]['year']).to_equal(2022)
        results_desc = await fetch_many_with_relations('test_many_books', [{'name': 'publisher', 'table': 'test_many_publishers', 'foreign_key': 'publisher_id', 'reference_column': 'id', 'join_type': 'left'}], order_by=('year', 'desc'), limit=10)
        expect(len(results_desc)).to_equal(3)
        expect(results_desc[0]['year']).to_equal(2022)
        expect(results_desc[1]['year']).to_equal(2020)
        expect(results_desc[2]['year']).to_equal(2019)

    @test
    async def test_fetch_many_no_relations(self):
        """Test fetch_many with empty relations list (just fetch without joins)."""
        await execute('\n            CREATE TABLE test_many_simple (\n                id SERIAL PRIMARY KEY,\n                value TEXT NOT NULL\n            )\n        ')
        await execute('INSERT INTO test_many_simple (value) VALUES ($1)', ['A'])
        await execute('INSERT INTO test_many_simple (value) VALUES ($1)', ['B'])
        await execute('INSERT INTO test_many_simple (value) VALUES ($1)', ['C'])
        results = await fetch_many_with_relations('test_many_simple', [], limit=10)
        expect(len(results)).to_equal(3)
        values = {r['value'] for r in results}
        expect(values).to_equal({'A', 'B', 'C'})

    @test
    async def test_fetch_many_with_null_foreign_keys(self):
        """Test that LEFT JOIN includes rows with NULL foreign keys."""
        await execute('\n            CREATE TABLE test_many_teams (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_many_players (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                team_id INTEGER REFERENCES test_many_teams(id)\n            )\n        ')
        await execute('INSERT INTO test_many_teams (name) VALUES ($1)', ['Team A'])
        await execute('INSERT INTO test_many_players (name, team_id) VALUES ($1, $2)', ['Player 1', 1])
        await execute('INSERT INTO test_many_players (name, team_id) VALUES ($1, $2)', ['Player 2', None])
        await execute('INSERT INTO test_many_players (name, team_id) VALUES ($1, $2)', ['Player 3', 1])
        results = await fetch_many_with_relations('test_many_players', [{'name': 'team', 'table': 'test_many_teams', 'foreign_key': 'team_id', 'reference_column': 'id', 'join_type': 'left'}], limit=10)
        expect(len(results)).to_equal(3)
        player_without_team = next((p for p in results if p['name'] == 'Player 2'))
        expect(player_without_team['team_id']).to_be_none()

class TestEagerLoadingComplexScenarios(PostgresSuite):
    """Test complex eager loading scenarios."""

    @test
    async def test_multiple_relations_different_tables(self):
        """Test loading multiple different relations in one query."""
        await execute('\n            CREATE TABLE test_complex_authors (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_complex_categories (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_complex_articles (\n                id SERIAL PRIMARY KEY,\n                title TEXT NOT NULL,\n                author_id INTEGER REFERENCES test_complex_authors(id),\n                category_id INTEGER REFERENCES test_complex_categories(id)\n            )\n        ')
        await execute('INSERT INTO test_complex_authors (name) VALUES ($1)', ['Alice'])
        await execute('INSERT INTO test_complex_categories (name) VALUES ($1)', ['Tech'])
        await execute('INSERT INTO test_complex_articles (title, author_id, category_id) VALUES ($1, $2, $3)', ['My Article', 1, 1])
        result = await fetch_one_with_relations('test_complex_articles', 1, [{'name': 'author', 'table': 'test_complex_authors', 'foreign_key': 'author_id', 'reference_column': 'id', 'join_type': 'left'}, {'name': 'category', 'table': 'test_complex_categories', 'foreign_key': 'category_id', 'reference_column': 'id', 'join_type': 'left'}])
        expect(result).not_to_be_none()
        expect(result['title']).to_equal('My Article')
        expect(result['author_id']).to_equal(1)
        expect(result['category_id']).to_equal(1)

    @test
    async def test_eager_loading_empty_table(self):
        """Test eager loading on empty table."""
        await execute('\n            CREATE TABLE test_complex_empty1 (\n                id SERIAL PRIMARY KEY,\n                name TEXT\n            )\n        ')
        await execute('\n            CREATE TABLE test_complex_empty2 (\n                id SERIAL PRIMARY KEY,\n                ref_id INTEGER REFERENCES test_complex_empty1(id)\n            )\n        ')
        results = await fetch_many_with_relations('test_complex_empty2', [{'name': 'ref', 'table': 'test_complex_empty1', 'foreign_key': 'ref_id', 'reference_column': 'id', 'join_type': 'left'}], limit=10)
        expect(results).to_equal([])

    @test
    async def test_filter_with_multiple_conditions(self):
        """Test filtering with multiple conditions."""
        await execute('\n            CREATE TABLE test_complex_vendors (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL\n            )\n        ')
        await execute('\n            CREATE TABLE test_complex_products (\n                id SERIAL PRIMARY KEY,\n                name TEXT NOT NULL,\n                price INTEGER,\n                vendor_id INTEGER REFERENCES test_complex_vendors(id)\n            )\n        ')
        await execute('INSERT INTO test_complex_vendors (name) VALUES ($1)', ['Vendor A'])
        await execute('INSERT INTO test_complex_products (name, price, vendor_id) VALUES ($1, $2, $3)', ['Product 1', 100, 1])
        await execute('INSERT INTO test_complex_products (name, price, vendor_id) VALUES ($1, $2, $3)', ['Product 2', 100, 1])
        await execute('INSERT INTO test_complex_products (name, price, vendor_id) VALUES ($1, $2, $3)', ['Product 3', 200, 1])
        results = await fetch_many_with_relations('test_complex_products', [{'name': 'vendor', 'table': 'test_complex_vendors', 'foreign_key': 'vendor_id', 'reference_column': 'id', 'join_type': 'left'}], filter={'price': 100}, limit=10)
        expect(len(results)).to_equal(2)
        expect(all((r['price'] == 100 for r in results))).to_be_true()