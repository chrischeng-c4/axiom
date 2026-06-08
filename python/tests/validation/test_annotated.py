"""
Tests for Annotated syntax support in BaseModel.

Run: uv run python python/tests/validation/test_annotated.py
"""

from cclab.probe import TestSuite, test, expect
from typing import Optional, List
from cclab.shield import BaseModel, Field

try:
    from typing import Annotated
except ImportError:
    from typing_extensions import Annotated


class TestAnnotatedSyntax(TestSuite):
    """Test Annotated syntax support."""

    @test
    def test_basic_annotated_field(self):
        """Test basic Annotated syntax for string field."""

        class User(BaseModel):
            name: Annotated[str, Field(min_length=1, max_length=100)]

        user = User(name="John")
        expect(user.name).to_equal("John")

        schema = User.model_json_schema()
        expect(schema["properties"]["name"]["minLength"]).to_equal(1)
        expect(schema["properties"]["name"]["maxLength"]).to_equal(100)

    @test
    def test_annotated_numeric_constraints(self):
        """Test Annotated syntax with numeric constraints."""

        class Product(BaseModel):
            price: Annotated[float, Field(ge=0, le=10000)]
            quantity: Annotated[int, Field(gt=0, lt=1000)]

        product = Product(price=99.99, quantity=10)
        expect(product.price).to_equal(99.99)
        expect(product.quantity).to_equal(10)

        schema = Product.model_json_schema()
        expect(schema["properties"]["price"]["minimum"]).to_equal(0)
        expect(schema["properties"]["price"]["maximum"]).to_equal(10000)

    @test
    def test_annotated_with_default(self):
        """Test Annotated syntax with default value via class attribute."""

        class Config(BaseModel):
            timeout: Annotated[int, Field(ge=1, le=3600)] = 30
            retries: Annotated[int, Field(ge=0, le=10)] = 3

        config = Config()
        expect(config.timeout).to_equal(30)
        expect(config.retries).to_equal(3)

    @test
    def test_annotated_optional_field(self):
        """Test Annotated syntax with Optional type."""

        class User(BaseModel):
            name: Annotated[str, Field(min_length=1)]
            bio: Annotated[Optional[str], Field(max_length=500)] = None

        user = User(name="John")
        expect(user.name).to_equal("John")
        expect(user.bio).to_equal(None)


class TestAnnotatedNestedModels(TestSuite):
    """Test Annotated syntax with nested models."""

    @test
    def test_annotated_nested_model(self):
        """Test Annotated syntax with nested BaseModel."""

        class Address(BaseModel):
            street: Annotated[str, Field(min_length=1)]
            city: Annotated[str, Field(min_length=1)]

        class User(BaseModel):
            name: Annotated[str, Field(min_length=1)]
            address: Address

        user = User(
            name="John",
            address=Address(street="123 Main St", city="NYC")
        )
        expect(user.name).to_equal("John")
        expect(user.address.street).to_equal("123 Main St")

    @test
    def test_annotated_list_of_models(self):
        """Test Annotated syntax with List of nested models."""

        class Tag(BaseModel):
            name: Annotated[str, Field(min_length=1, max_length=50)]
            priority: Annotated[int, Field(ge=0, le=10)] = 0

        class Article(BaseModel):
            title: Annotated[str, Field(min_length=1, max_length=200)]
            tags: List[Tag]

        article = Article(
            title="Python Tips",
            tags=[Tag(name="python", priority=5), Tag(name="tips", priority=3)]
        )
        expect(article.title).to_equal("Python Tips")
        expect(len(article.tags)).to_equal(2)
        expect(article.tags[0].name).to_equal("python")


class TestAnnotatedModelDump(TestSuite):
    """Test model_dump with Annotated syntax models."""

    @test
    def test_annotated_model_dump(self):
        """Test model_dump with Annotated fields."""

        class User(BaseModel):
            name: Annotated[str, Field(min_length=1)]
            age: Annotated[int, Field(ge=0)]
            active: Annotated[bool, Field()] = True

        user = User(name="John", age=30)
        data = user.model_dump()

        expect(data).to_equal({"name": "John", "age": 30, "active": True})


if __name__ == "__main__":
    import asyncio
    asyncio.run(TestAnnotatedSyntax().run())
    asyncio.run(TestAnnotatedNestedModels().run())
    asyncio.run(TestAnnotatedModelDump().run())
