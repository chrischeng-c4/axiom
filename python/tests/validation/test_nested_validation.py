"""
Tests for nested model validation with Rust backend.

Run: uv run python python/tests/validation/test_nested_validation.py
"""

from cclab.probe import TestSuite, test, expect
from typing import Optional, List
from cclab.shield import BaseModel, Field

try:
    from typing import Annotated
except ImportError:
    from typing_extensions import Annotated


class TestNestedValidation(TestSuite):
    """Test nested model validation through Rust."""

    @test
    def test_simple_nested_model(self):
        """Test validation of simple nested model."""

        class Address(BaseModel):
            street: str
            city: str
            country: str = "USA"

        class User(BaseModel):
            name: str
            address: Address

        user = User(
            name="John",
            address=Address(street="123 Main St", city="New York")
        )
        expect(user.name).to_equal("John")
        expect(user.address.street).to_equal("123 Main St")
        expect(user.address.country).to_equal("USA")

    @test
    def test_nested_model_from_dict(self):
        """Test nested model created from dictionary."""

        class Address(BaseModel):
            street: str
            city: str

        class User(BaseModel):
            name: str
            address: Address

        user = User(
            name="John",
            address={"street": "123 Main St", "city": "New York"}
        )
        expect(user.name).to_equal("John")
        expect(isinstance(user.address, Address)).to_equal(True)
        expect(user.address.street).to_equal("123 Main St")

    @test
    def test_deeply_nested_models(self):
        """Test validation of deeply nested models."""

        class Country(BaseModel):
            name: str
            code: str

        class Address(BaseModel):
            street: str
            city: str
            country: Country

        class User(BaseModel):
            name: str
            address: Address

        user = User(
            name="John",
            address=Address(
                street="123 Main St",
                city="New York",
                country=Country(name="United States", code="US")
            )
        )
        expect(user.address.country.name).to_equal("United States")
        expect(user.address.country.code).to_equal("US")


class TestListOfNestedModels(TestSuite):
    """Test validation of lists containing nested models."""

    @test
    def test_list_of_nested_models(self):
        """Test list of nested models validation."""

        class Tag(BaseModel):
            name: str
            priority: int = 0

        class Article(BaseModel):
            title: str
            tags: List[Tag]

        article = Article(
            title="Python Tips",
            tags=[Tag(name="python", priority=5), Tag(name="tips", priority=3)]
        )
        expect(len(article.tags)).to_equal(2)
        expect(article.tags[0].name).to_equal("python")

    @test
    def test_list_of_nested_from_dicts(self):
        """Test list of nested models from list of dicts."""

        class Tag(BaseModel):
            name: str
            priority: int = 0

        class Article(BaseModel):
            title: str
            tags: List[Tag]

        article = Article(
            title="Python Tips",
            tags=[{"name": "python", "priority": 5}, {"name": "tips"}]
        )
        expect(len(article.tags)).to_equal(2)
        expect(article.tags[0].priority).to_equal(5)
        expect(article.tags[1].priority).to_equal(0)


class TestOptionalNestedModels(TestSuite):
    """Test validation of optional nested models."""

    @test
    def test_optional_nested_none(self):
        """Test optional nested model with None value."""

        class Address(BaseModel):
            city: str

        class User(BaseModel):
            name: str
            address: Optional[Address] = None

        user = User(name="John")
        expect(user.address).to_equal(None)

    @test
    def test_optional_nested_present(self):
        """Test optional nested model with value."""

        class Address(BaseModel):
            city: str

        class User(BaseModel):
            name: str
            address: Optional[Address] = None

        user = User(name="Jane", address=Address(city="Boston"))
        expect(user.address.city).to_equal("Boston")


class TestNestedModelDump(TestSuite):
    """Test model_dump for nested models."""

    @test
    def test_nested_dump(self):
        """Test dumping nested models to dict."""

        class Address(BaseModel):
            street: str
            city: str

        class User(BaseModel):
            name: str
            address: Address

        user = User(
            name="John",
            address=Address(street="123 Main St", city="NYC")
        )
        data = user.model_dump()

        expect(data).to_equal({
            "name": "John",
            "address": {"street": "123 Main St", "city": "NYC"}
        })

    @test
    def test_list_nested_dump(self):
        """Test dumping list of nested models."""

        class Tag(BaseModel):
            name: str
            priority: int = 0

        class Article(BaseModel):
            title: str
            tags: List[Tag]

        article = Article(
            title="Test",
            tags=[Tag(name="a", priority=1), Tag(name="b")]
        )
        data = article.model_dump()

        expect(data).to_equal({
            "title": "Test",
            "tags": [{"name": "a", "priority": 1}, {"name": "b", "priority": 0}]
        })


if __name__ == "__main__":
    import asyncio
    asyncio.run(TestNestedValidation().run())
    asyncio.run(TestListOfNestedModels().run())
    asyncio.run(TestOptionalNestedModels().run())
    asyncio.run(TestNestedModelDump().run())
