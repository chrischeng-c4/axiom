"""
Integration tests for SQL set operations (UNION/INTERSECT/EXCEPT).

Tests set operation functionality with real PostgreSQL database using raw SQL.
"""
from cclab.titan import execute, insert_one
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
@fixture
async def customers_2023():
    """Create and populate customers_2023 table."""
    await execute('\n        CREATE TABLE IF NOT EXISTS customers_2023 (\n            id INTEGER PRIMARY KEY,\n            name VARCHAR(100) NOT NULL,\n            email VARCHAR(100) NOT NULL\n        )\n        ')
    test_data = [{'id': 1, 'name': 'Alice Johnson', 'email': 'alice@example.com'}, {'id': 2, 'name': 'Bob Smith', 'email': 'bob@example.com'}, {'id': 3, 'name': 'Charlie Brown', 'email': 'charlie@example.com'}, {'id': 4, 'name': 'David Wilson', 'email': 'david@example.com'}, {'id': 5, 'name': 'Eve Martinez', 'email': 'eve@example.com'}]
    for data in test_data:
        await insert_one('customers_2023', data)
    yield
    await execute('DROP TABLE IF EXISTS customers_2023')

@fixture
async def customers_2024():
    """Create and populate customers_2024 table."""
    await execute('\n        CREATE TABLE IF NOT EXISTS customers_2024 (\n            id INTEGER PRIMARY KEY,\n            name VARCHAR(100) NOT NULL,\n            email VARCHAR(100) NOT NULL\n        )\n        ')
    test_data = [{'id': 1, 'name': 'Alice Johnson', 'email': 'alice@example.com'}, {'id': 2, 'name': 'Bob Smith', 'email': 'bob@example.com'}, {'id': 3, 'name': 'Charlie Brown', 'email': 'charlie@example.com'}, {'id': 6, 'name': 'Frank Davis', 'email': 'frank@example.com'}, {'id': 7, 'name': 'Grace Lee', 'email': 'grace@example.com'}]
    for data in test_data:
        await insert_one('customers_2024', data)
    yield
    await execute('DROP TABLE IF EXISTS customers_2024')

class TestBasicSetOperations(PostgresSuite):
    """Test basic set operations."""

    @test
    async def test_union_combines_and_removes_duplicates(self, customers_2023, customers_2024):
        """Test UNION - combine two tables with same columns, removing duplicates."""
        results = await execute('\n            SELECT * FROM customers_2023\n            UNION\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(7)
        emails = [r['email'] for r in results]
        expect(len(set(emails))).to_equal(7)
        expect('alice@example.com').to_be_in(emails)
        expect('eve@example.com').to_be_in(emails)
        expect('frank@example.com').to_be_in(emails)

    @test
    async def test_union_all_includes_duplicates(self, customers_2023, customers_2024):
        """Test UNION ALL - include duplicates."""
        results = await execute('\n            SELECT * FROM customers_2023\n            UNION ALL\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(10)
        emails = [r['email'] for r in results]
        alice_count = sum((1 for email in emails if email == 'alice@example.com'))
        expect(alice_count).to_equal(2)

    @test
    async def test_intersect_finds_common_rows(self, customers_2023, customers_2024):
        """Test INTERSECT - find rows present in both tables."""
        results = await execute('\n            SELECT * FROM customers_2023\n            INTERSECT\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(3)
        emails = [r['email'] for r in results]
        expect('alice@example.com').to_be_in(emails)
        expect('bob@example.com').to_be_in(emails)
        expect('charlie@example.com').to_be_in(emails)
        expect('eve@example.com').to_not_be_in(emails)
        expect('frank@example.com').to_not_be_in(emails)

    @test
    async def test_except_finds_rows_in_first_only(self, customers_2023, customers_2024):
        """Test EXCEPT - find rows in first table but not second."""
        results = await execute('\n            SELECT * FROM customers_2023\n            EXCEPT\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(2)
        emails = [r['email'] for r in results]
        expect('david@example.com').to_be_in(emails)
        expect('eve@example.com').to_be_in(emails)
        expect('alice@example.com').to_not_be_in(emails)
        expect('bob@example.com').to_not_be_in(emails)
        expect('frank@example.com').to_not_be_in(emails)

class TestSetOperationsWithFilters(PostgresSuite):
    """Test set operations combined with WHERE conditions."""

    @test
    async def test_union_with_where_conditions(self, customers_2023, customers_2024):
        """Test UNION with WHERE conditions on each part."""
        results = await execute('\n            SELECT * FROM customers_2023 WHERE id < 3\n            UNION\n            SELECT * FROM customers_2024 WHERE id > 5\n        ')
        expect(len(results)).to_equal(4)
        emails = [r['email'] for r in results]
        expect('alice@example.com').to_be_in(emails)
        expect('bob@example.com').to_be_in(emails)
        expect('frank@example.com').to_be_in(emails)
        expect('grace@example.com').to_be_in(emails)

    @test
    async def test_intersect_with_filters(self, customers_2023, customers_2024):
        """Test INTERSECT with WHERE conditions."""
        results = await execute('\n            SELECT * FROM customers_2023 WHERE id <= 3\n            INTERSECT\n            SELECT * FROM customers_2024 WHERE id <= 3\n        ')
        expect(len(results)).to_equal(3)

    @test
    async def test_except_with_filters(self, customers_2023, customers_2024):
        """Test EXCEPT with WHERE conditions."""
        results = await execute('\n            SELECT * FROM customers_2023 WHERE id >= 3\n            EXCEPT\n            SELECT * FROM customers_2024 WHERE id >= 3\n        ')
        expect(len(results)).to_equal(2)
        emails = [r['email'] for r in results]
        expect('david@example.com').to_be_in(emails)
        expect('eve@example.com').to_be_in(emails)

class TestChainedSetOperations(PostgresSuite):
    """Test multiple set operations chained together."""

    @test
    async def test_multiple_unions_chained(self, customers_2023, customers_2024):
        """Test chaining multiple UNION operations."""
        results = await execute('\n            SELECT * FROM customers_2023 WHERE id <= 2\n            UNION\n            SELECT * FROM customers_2023 WHERE id >= 4\n            UNION\n            SELECT * FROM customers_2024 WHERE id >= 6\n        ')
        expect(len(results)).to_equal(6)
        emails = [r['email'] for r in results]
        expect('alice@example.com').to_be_in(emails)
        expect('bob@example.com').to_be_in(emails)
        expect('david@example.com').to_be_in(emails)
        expect('eve@example.com').to_be_in(emails)
        expect('frank@example.com').to_be_in(emails)
        expect('grace@example.com').to_be_in(emails)

    @test
    async def test_union_then_intersect(self, customers_2023, customers_2024):
        """Test combining UNION with INTERSECT using CTE."""
        results = await execute('\n            WITH combined AS (\n                SELECT * FROM customers_2023\n                UNION\n                SELECT * FROM customers_2024 WHERE id <= 3\n            )\n            SELECT * FROM combined\n            INTERSECT\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(3)

class TestSetOperationsWithOrderBy(PostgresSuite):
    """Test set operations with ORDER BY clause."""

    @test
    async def test_union_with_order_by(self, customers_2023, customers_2024):
        """Test set operations with ORDER BY."""
        results = await execute('\n            SELECT * FROM customers_2023\n            UNION\n            SELECT * FROM customers_2024\n            ORDER BY name DESC\n        ')
        expect(len(results)).to_equal(7)
        names = [r['name'] for r in results]
        expect(names).to_equal(sorted(names, reverse=True))
        expect(results[0]['name']).to_be_in(['Grace Lee', 'Frank Davis'])

    @test
    async def test_intersect_with_order_by(self, customers_2023, customers_2024):
        """Test INTERSECT with ORDER BY."""
        results = await execute('\n            SELECT * FROM customers_2023\n            INTERSECT\n            SELECT * FROM customers_2024\n            ORDER BY id ASC\n        ')
        expect(len(results)).to_equal(3)
        ids = [r['id'] for r in results]
        expect(ids).to_equal(sorted(ids))
        expect(ids).to_equal([1, 2, 3])

class TestSetOperationsWithLimit(PostgresSuite):
    """Test set operations with LIMIT clause."""

    @test
    async def test_union_with_limit(self, customers_2023, customers_2024):
        """Test UNION with LIMIT."""
        results = await execute('\n            SELECT * FROM customers_2023\n            UNION\n            SELECT * FROM customers_2024\n            LIMIT 3\n        ')
        expect(len(results)).to_equal(3)

    @test
    async def test_union_with_offset_and_limit(self, customers_2023, customers_2024):
        """Test UNION with OFFSET and LIMIT."""
        results = await execute('\n            SELECT * FROM customers_2023\n            UNION\n            SELECT * FROM customers_2024\n            ORDER BY id\n            OFFSET 2\n            LIMIT 3\n        ')
        expect(len(results)).to_equal(3)
        ids = [r['id'] for r in results]
        expect(min(ids)).to_be_greater_than_or_equal(3)

class TestSetOperationsWithColumns(PostgresSuite):
    """Test set operations must have matching column structure."""

    @test
    async def test_union_compatible_columns(self, customers_2023, customers_2024):
        """Test UNION works when selecting same columns."""
        results = await execute('\n            SELECT id, name FROM customers_2023\n            UNION\n            SELECT id, name FROM customers_2024\n        ')
        expect(len(results)).to_equal(7)
        for result in results:
            expect('id').to_be_in(result)
            expect('name').to_be_in(result)

class TestIntersectAll(PostgresSuite):
    """Test INTERSECT ALL (keeps duplicates)."""

    @test
    async def test_intersect_all(self, customers_2023, customers_2024):
        """Test INTERSECT ALL keeps all matching rows."""
        results = await execute('\n            SELECT * FROM customers_2023\n            INTERSECT ALL\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(3)

class TestExceptAll(PostgresSuite):
    """Test EXCEPT ALL (keeps duplicates)."""

    @test
    async def test_except_all(self, customers_2023, customers_2024):
        """Test EXCEPT ALL keeps all non-matching rows."""
        results = await execute('\n            SELECT * FROM customers_2023\n            EXCEPT ALL\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(2)
        emails = [r['email'] for r in results]
        expect('david@example.com').to_be_in(emails)
        expect('eve@example.com').to_be_in(emails)

class TestSetOperationsEdgeCases(PostgresSuite):
    """Test edge cases and special scenarios."""

    @test
    async def test_union_empty_result(self, customers_2023, customers_2024):
        """Test UNION when one query returns no results."""
        results = await execute('\n            SELECT * FROM customers_2023 WHERE id > 100\n            UNION\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(5)

    @test
    async def test_intersect_no_overlap(self, customers_2023, customers_2024):
        """Test INTERSECT when there's no overlap."""
        results = await execute('\n            SELECT * FROM customers_2023 WHERE id >= 4\n            INTERSECT\n            SELECT * FROM customers_2024 WHERE id >= 6\n        ')
        expect(len(results)).to_equal(0)

    @test
    async def test_except_all_removed(self, customers_2023, customers_2024):
        """Test EXCEPT when all rows would be removed."""
        results = await execute('\n            SELECT * FROM customers_2023 WHERE id <= 3\n            EXCEPT\n            SELECT * FROM customers_2024\n        ')
        expect(len(results)).to_equal(0)