"""
Tests for QueryBuilder functionality.

Tests for:
- Filter expressions (AND, OR, mixed)
- Sorting (ascending, descending, multiple fields)
- Pagination (skip, limit)
- Query execution with MongoDB

Migrated from test_comprehensive.py and split for maintainability.
"""
from typing import Optional

from cclab import Document
from cclab.nebula.query import QueryBuilder
from cclab.probe import test, expect
from tests.base import MongoTestSuite, CommonTestSuite


# =====================
# Test Document Classes
# =====================

class F(Document):
    """Field helper for unit tests - provides clean field access."""
    name: str
    age: int = 0
    status: str = "active"
    role: str = "user"
    active: bool = True
    email: Optional[str] = None

    class Settings:
        name = "_unit_test_fields"


class QueryTestUser(Document):
    """Test user for query tests."""
    name: str
    age: int = 0
    status: str = "active"
    role: str = "user"
    score: float = 0.0

    class Settings:
        name = "test_query_users"


# =====================
# Unit Tests (no MongoDB)
# =====================

class TestQueryBuilderFilters(CommonTestSuite):
    """Test QueryBuilder filter expressions."""

    @test(tags=["unit", "queries"])
    async def test_simple_equality(self):
        """Test simple equality filter."""
        expr = F.name == "Alice"
        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"name": "Alice"})

    @test(tags=["unit", "queries"])
    async def test_comparison_operators(self):
        """Test comparison operators."""
        expect("$gt" in str((F.age > 25).to_filter())).to_be_true()
        expect("$gte" in str((F.age >= 25).to_filter())).to_be_true()
        expect("$lt" in str((F.age < 25).to_filter())).to_be_true()
        expect("$lte" in str((F.age <= 25).to_filter())).to_be_true()
        expect("$ne" in str((F.age != 25).to_filter())).to_be_true()

    @test(tags=["unit", "queries"])
    async def test_complex_filter_chain(self):
        """Test complex filter with multiple AND conditions."""
        expr = (F.name == "Alice") & (F.age > 25)
        filter_dict = expr.to_filter()
        expect("$and" in filter_dict).to_be_true()

    @test(tags=["unit", "queries"])
    async def test_or_chain(self):
        """Test OR conditions."""
        expr = (F.role == "admin") | (F.role == "superuser") | (F.role == "owner")
        filter_dict = expr.to_filter()
        expect("$or" in filter_dict).to_be_true()

    @test(tags=["unit", "queries"])
    async def test_mixed_and_or(self):
        """Test mixed AND/OR conditions."""
        expr = (F.active == True) & ((F.role == "admin") | (F.role == "superuser"))
        filter_dict = expr.to_filter()
        expect("$and" in filter_dict).to_be_true()

    @test(tags=["unit", "queries"])
    async def test_in_operator(self):
        """Test IN operator."""
        expr = F.status.in_(["active", "pending", "review"])
        filter_dict = expr.to_filter()
        expect("$in" in str(filter_dict)).to_be_true()

    @test(tags=["unit", "queries"])
    async def test_not_in_operator(self):
        """Test NOT IN operator."""
        expr = F.status.not_in(["deleted", "archived"])
        filter_dict = expr.to_filter()
        expect("$nin" in str(filter_dict)).to_be_true()

    @test(tags=["unit", "queries"])
    async def test_exists_operator(self):
        """Test EXISTS operator."""
        expr = F.email.exists()
        filter_dict = expr.to_filter()
        expect("$exists" in str(filter_dict)).to_be_true()


class TestQueryBuilderSorting(CommonTestSuite):
    """Test QueryBuilder sorting."""

    @test(tags=["unit", "queries"])
    async def test_sort_ascending(self):
        """Test ascending sort."""
        class MockDoc:
            _collection_name = "test"

        builder = QueryBuilder(MockDoc, ()).sort("created_at")
        expect(builder._sort_spec).to_equal([("created_at", 1)])

    @test(tags=["unit", "queries"])
    async def test_sort_descending_string(self):
        """Test descending sort with - prefix."""
        class MockDoc:
            _collection_name = "test"

        builder = QueryBuilder(MockDoc, ()).sort("-created_at")
        expect(len(builder._sort_spec)).to_equal(1)

    @test(tags=["unit", "queries"])
    async def test_sort_multiple_fields(self):
        """Test sorting by multiple fields."""
        class MockDoc:
            _collection_name = "test"

        builder = QueryBuilder(MockDoc, ()).sort(("status", 1), ("created_at", -1))
        expect(len(builder._sort_spec)).to_equal(2)

    @test(tags=["unit", "queries"])
    async def test_sort_chain(self):
        """Test chained sort calls."""
        class MockDoc:
            _collection_name = "test"

        builder = QueryBuilder(MockDoc, ()).sort("status").sort("name")
        expect(len(builder._sort_spec)).to_equal(2)


class TestQueryBuilderPagination(CommonTestSuite):
    """Test QueryBuilder pagination."""

    @test(tags=["unit", "queries"])
    async def test_skip_limit(self):
        """Test skip and limit."""
        class MockDoc:
            _collection_name = "test"

        builder = QueryBuilder(MockDoc, ()).skip(20).limit(10)
        expect(builder._skip_val).to_equal(20)
        expect(builder._limit_val).to_equal(10)

    @test(tags=["unit", "queries"])
    async def test_skip_zero(self):
        """Test skip with zero."""
        class MockDoc:
            _collection_name = "test"

        builder = QueryBuilder(MockDoc, ()).skip(0)
        expect(builder._skip_val).to_equal(0)

    @test(tags=["unit", "queries"])
    async def test_limit_zero(self):
        """Test limit with zero (no limit)."""
        class MockDoc:
            _collection_name = "test"

        builder = QueryBuilder(MockDoc, ()).limit(0)
        expect(builder._limit_val).to_equal(0)


# =====================
# Integration Tests (MongoDB)
# =====================

class TestQueryExecution(MongoTestSuite):
    """Integration tests for query execution."""

    async def setup(self):
        """Clean up test data."""
        await QueryTestUser.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await QueryTestUser.find().delete()

    @test(tags=["mongo", "queries"])
    async def test_find_with_equality(self):
        """Test find with equality filter."""
        await QueryTestUser(name="Alice", age=30).save()
        await QueryTestUser(name="Bob", age=25).save()

        found = await QueryTestUser.find_one(QueryTestUser.name == "Alice")
        expect(found).not_.to_be_none()
        expect(found.name).to_equal("Alice")

    @test(tags=["mongo", "queries"])
    async def test_find_with_comparison(self):
        """Test find with comparison operators."""
        await QueryTestUser(name="Young", age=20).save()
        await QueryTestUser(name="Middle", age=35).save()
        await QueryTestUser(name="Senior", age=50).save()

        results = await QueryTestUser.find(QueryTestUser.age > 30).to_list()
        expect(len(results)).to_equal(2)
        names = {u.name for u in results}
        expect("Middle" in names).to_be_true()
        expect("Senior" in names).to_be_true()

    @test(tags=["mongo", "queries"])
    async def test_find_with_multiple_conditions(self):
        """Test find with multiple conditions (implicit AND via dict)."""
        await QueryTestUser(name="Alice", age=30, status="active").save()
        await QueryTestUser(name="Bob", age=30, status="inactive").save()
        await QueryTestUser(name="Carol", age=25, status="active").save()

        # Use dict-style filter for multiple conditions (implicit AND)
        results = await QueryTestUser.find(
            {"age": 30, "status": "active"}
        ).to_list()

        expect(len(results)).to_equal(1)
        expect(results[0].name).to_equal("Alice")

    @test(tags=["mongo", "queries"])
    async def test_find_with_or_dict(self):
        """Test find with OR conditions using dict syntax."""
        await QueryTestUser(name="Admin", role="admin").save()
        await QueryTestUser(name="Super", role="superuser").save()
        await QueryTestUser(name="Regular", role="user").save()

        # Use MongoDB $or syntax directly
        results = await QueryTestUser.find(
            {"$or": [{"role": "admin"}, {"role": "superuser"}]}
        ).to_list()

        expect(len(results)).to_equal(2)
        names = {u.name for u in results}
        expect("Admin" in names).to_be_true()
        expect("Super" in names).to_be_true()

    @test(tags=["mongo", "queries"])
    async def test_find_with_in(self):
        """Test find with IN operator."""
        await QueryTestUser(name="U1", status="active").save()
        await QueryTestUser(name="U2", status="pending").save()
        await QueryTestUser(name="U3", status="deleted").save()

        results = await QueryTestUser.find(
            QueryTestUser.status.in_(["active", "pending"])
        ).to_list()

        expect(len(results)).to_equal(2)

    @test(tags=["mongo", "queries"])
    async def test_find_with_sort(self):
        """Test find with sorting."""
        await QueryTestUser(name="Charlie", score=75.0).save()
        await QueryTestUser(name="Alice", score=95.0).save()
        await QueryTestUser(name="Bob", score=85.0).save()

        # Sort by score descending
        results = await QueryTestUser.find().sort(("score", -1)).to_list()

        expect(len(results)).to_equal(3)
        expect(results[0].name).to_equal("Alice")
        expect(results[1].name).to_equal("Bob")
        expect(results[2].name).to_equal("Charlie")

    @test(tags=["mongo", "queries"])
    async def test_find_with_pagination(self):
        """Test find with skip and limit."""
        for i in range(10):
            await QueryTestUser(name=f"User{i:02d}", age=20 + i).save()

        # Get page 2 (items 3-5)
        results = await QueryTestUser.find().sort("name").skip(3).limit(3).to_list()

        expect(len(results)).to_equal(3)
        expect(results[0].name).to_equal("User03")
        expect(results[2].name).to_equal("User05")

    @test(tags=["mongo", "queries"])
    async def test_count_with_filter(self):
        """Test count with filter."""
        await QueryTestUser(name="A1", status="active").save()
        await QueryTestUser(name="A2", status="active").save()
        await QueryTestUser(name="I1", status="inactive").save()

        total = await QueryTestUser.find().count()
        expect(total).to_equal(3)

        active = await QueryTestUser.find(QueryTestUser.status == "active").count()
        expect(active).to_equal(2)

    @test(tags=["mongo", "queries"])
    async def test_first_and_first_or_none(self):
        """Test first() and first_or_none() methods."""
        await QueryTestUser(name="First", age=1).save()
        await QueryTestUser(name="Second", age=2).save()

        first = await QueryTestUser.find().sort("age").first_or_none()
        expect(first).not_.to_be_none()
        expect(first.name).to_equal("First")

        none_result = await QueryTestUser.find(QueryTestUser.age > 100).first_or_none()
        expect(none_result).to_be_none()

    @test(tags=["mongo", "queries"])
    async def test_exists_query(self):
        """Test exists() on query."""
        await QueryTestUser(name="Exists").save()

        exists = await QueryTestUser.find(QueryTestUser.name == "Exists").exists()
        expect(exists).to_be_true()

        not_exists = await QueryTestUser.find(QueryTestUser.name == "NotExists").exists()
        expect(not_exists).to_be_false()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestQueryBuilderFilters,
        TestQueryBuilderSorting,
        TestQueryBuilderPagination,
        TestQueryExecution,
    ], verbose=True)
