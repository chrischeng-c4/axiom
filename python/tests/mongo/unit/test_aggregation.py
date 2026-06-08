"""Tests for QueryBuilder aggregation helpers (avg, sum, max, min)."""

from cclab import Document
from cclab.probe import test, expect
from tests.base import MongoTestSuite


class AggProduct(Document):
    """Product document for aggregation tests."""
    name: str
    price: float
    quantity: int
    category: str

    class Settings:
        name = "test_aggregation_products"


class TestAggregationHelpers(MongoTestSuite):
    """Test aggregation helper methods on QueryBuilder."""

    async def setup(self):
        """Set up test data."""
        await AggProduct.delete_many()

        await AggProduct.insert_many([
            AggProduct(name="Apple", price=1.50, quantity=100, category="fruit"),
            AggProduct(name="Banana", price=0.75, quantity=150, category="fruit"),
            AggProduct(name="Orange", price=2.00, quantity=80, category="fruit"),
            AggProduct(name="Milk", price=3.50, quantity=50, category="dairy"),
            AggProduct(name="Cheese", price=5.00, quantity=30, category="dairy"),
        ])

    async def teardown(self):
        """Cleanup."""
        await AggProduct.delete_many()

    @test(tags=["mongo", "aggregation"])
    async def test_avg_all_documents(self):
        """Test avg() on entire collection."""
        # Average price: (1.50 + 0.75 + 2.00 + 3.50 + 5.00) / 5 = 2.55
        avg_price = await AggProduct.find().avg(AggProduct.price)
        expect(abs(avg_price - 2.55) < 0.01).to_be_true()

    @test(tags=["mongo", "aggregation"])
    async def test_avg_with_filter(self):
        """Test avg() with query filter."""
        # Average price for fruits: (1.50 + 0.75 + 2.00) / 3 = 1.4166...
        avg_price = await AggProduct.find(AggProduct.category == "fruit").avg(AggProduct.price)
        expect(abs(avg_price - 1.4166) < 0.01).to_be_true()

    @test(tags=["mongo", "aggregation"])
    async def test_avg_string_field(self):
        """Test avg() with string field name."""
        avg_qty = await AggProduct.find().avg("quantity")
        # Average quantity: (100 + 150 + 80 + 50 + 30) / 5 = 82
        expect(abs(avg_qty - 82.0) < 0.01).to_be_true()

    @test(tags=["mongo", "aggregation"])
    async def test_avg_empty_result(self):
        """Test avg() returns None for empty result set."""
        avg_price = await AggProduct.find(AggProduct.category == "nonexistent").avg(AggProduct.price)
        expect(avg_price).to_be_none()

    @test(tags=["mongo", "aggregation"])
    async def test_sum_all_documents(self):
        """Test sum() on entire collection."""
        # Total price: 1.50 + 0.75 + 2.00 + 3.50 + 5.00 = 12.75
        total = await AggProduct.find().sum(AggProduct.price)
        expect(abs(total - 12.75) < 0.01).to_be_true()

    @test(tags=["mongo", "aggregation"])
    async def test_sum_with_filter(self):
        """Test sum() with query filter."""
        # Total quantity for dairy: 50 + 30 = 80
        total = await AggProduct.find(AggProduct.category == "dairy").sum(AggProduct.quantity)
        expect(abs(total - 80.0) < 0.01).to_be_true()

    @test(tags=["mongo", "aggregation"])
    async def test_sum_empty_result(self):
        """Test sum() returns None for empty result set."""
        total = await AggProduct.find(AggProduct.category == "nonexistent").sum(AggProduct.price)
        expect(total).to_be_none()

    @test(tags=["mongo", "aggregation"])
    async def test_max_all_documents(self):
        """Test max() on entire collection."""
        max_price = await AggProduct.find().max(AggProduct.price)
        expect(max_price).to_equal(5.00)

    @test(tags=["mongo", "aggregation"])
    async def test_max_with_filter(self):
        """Test max() with query filter."""
        # Max price for fruits: 2.00 (Orange)
        max_price = await AggProduct.find(AggProduct.category == "fruit").max(AggProduct.price)
        expect(max_price).to_equal(2.00)

    @test(tags=["mongo", "aggregation"])
    async def test_max_empty_result(self):
        """Test max() returns None for empty result set."""
        max_price = await AggProduct.find(AggProduct.category == "nonexistent").max(AggProduct.price)
        expect(max_price).to_be_none()

    @test(tags=["mongo", "aggregation"])
    async def test_min_all_documents(self):
        """Test min() on entire collection."""
        min_price = await AggProduct.find().min(AggProduct.price)
        expect(min_price).to_equal(0.75)

    @test(tags=["mongo", "aggregation"])
    async def test_min_with_filter(self):
        """Test min() with query filter."""
        # Min quantity for dairy: 30 (Cheese)
        min_qty = await AggProduct.find(AggProduct.category == "dairy").min(AggProduct.quantity)
        expect(min_qty).to_equal(30)

    @test(tags=["mongo", "aggregation"])
    async def test_min_empty_result(self):
        """Test min() returns None for empty result set."""
        min_price = await AggProduct.find(AggProduct.category == "nonexistent").min(AggProduct.price)
        expect(min_price).to_be_none()

    @test(tags=["mongo", "aggregation"])
    async def test_chained_query_with_aggregation(self):
        """Test aggregation after chained query methods."""
        avg_price = await AggProduct.find(AggProduct.price > 1.0).avg(AggProduct.price)
        # Products with price > 1.0: Apple (1.50), Orange (2.00), Milk (3.50), Cheese (5.00)
        # Average: (1.50 + 2.00 + 3.50 + 5.00) / 4 = 3.00
        expect(abs(avg_price - 3.00) < 0.01).to_be_true()


class TestAggregationHelpersUnit(MongoTestSuite):
    """Unit tests for aggregation helpers."""

    @test(tags=["unit", "aggregation"])
    async def test_field_name_extraction_from_proxy(self):
        """Test that field name is correctly extracted from FieldProxy."""
        # Document metaclass creates FieldProxy for each field
        expect(AggProduct.name.name).to_equal("name")
        expect(AggProduct.price.name).to_equal("price")

    @test(tags=["unit", "aggregation"])
    async def test_field_name_from_string(self):
        """Test that string field names work correctly."""
        field_name = "price"
        result = field_name if not hasattr(field_name, "name") else field_name.name
        expect(result).to_equal("price")


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestAggregationHelpers,
        TestAggregationHelpersUnit,
    ], verbose=True)
