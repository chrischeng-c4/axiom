"""
Tests for response_model filtering functionality.

Run: uv run python python/tests/validation/test_response_model.py
"""

from cclab.probe import TestSuite, test, expect
from typing import Optional, List
from cclab.shield import BaseModel, Field

try:
    from typing import Annotated
except ImportError:
    from typing_extensions import Annotated

from cclab.quasar.app import _filter_response


class TestBasicResponseFiltering(TestSuite):
    """Test basic response filtering functionality."""

    @test
    def test_filter_removes_extra_fields(self):
        """Test that fields not in response_model are removed."""

        class UserInternal(BaseModel):
            id: str
            name: str
            email: str
            password_hash: str

        class UserPublic(BaseModel):
            id: str
            name: str
            email: str

        internal_user = UserInternal(
            id="1", name="John", email="john@example.com", password_hash="secret"
        )
        result = _filter_response(internal_user, UserPublic)

        expect(result).to_equal({"id": "1", "name": "John", "email": "john@example.com"})
        expect("password_hash" in result).to_equal(False)

    @test
    def test_filter_from_dict(self):
        """Test filtering from dictionary input."""

        class UserPublic(BaseModel):
            id: str
            name: str

        data = {"id": "1", "name": "John", "email": "john@example.com", "internal_id": "xyz"}
        result = _filter_response(data, UserPublic)

        expect(result).to_equal({"id": "1", "name": "John"})
        expect("email" in result).to_equal(False)

    @test
    def test_filter_with_none_value(self):
        """Test filtering with None input."""

        class Response(BaseModel):
            id: str

        result = _filter_response(None, Response)
        expect(result).to_equal(None)


class TestNestedResponseFiltering(TestSuite):
    """Test response filtering with nested models."""

    @test
    def test_filter_nested_model(self):
        """Test filtering with nested model in response."""

        class AddressPublic(BaseModel):
            city: str
            country: str

        class UserPublic(BaseModel):
            name: str
            address: AddressPublic

        data = {
            "name": "John",
            "internal_id": "xyz",
            "address": {"city": "NYC", "country": "USA", "zip_code": "10001"}
        }
        result = _filter_response(data, UserPublic)

        expect(result).to_equal({
            "name": "John",
            "address": {"city": "NYC", "country": "USA"}
        })
        expect("internal_id" in result).to_equal(False)
        expect("zip_code" in result["address"]).to_equal(False)


class TestListResponseFiltering(TestSuite):
    """Test response filtering with lists."""

    @test
    def test_filter_list_of_dicts(self):
        """Test filtering a list of dictionaries."""

        class UserPublic(BaseModel):
            id: str
            name: str

        data = [
            {"id": "1", "name": "John", "password": "secret1"},
            {"id": "2", "name": "Jane", "password": "secret2"},
        ]
        result = _filter_response(data, UserPublic)

        expect(len(result)).to_equal(2)
        expect(result[0]).to_equal({"id": "1", "name": "John"})
        expect(result[1]).to_equal({"id": "2", "name": "Jane"})


class TestEdgeCases(TestSuite):
    """Test edge cases in response filtering."""

    @test
    def test_filter_empty_dict(self):
        """Test filtering empty dictionary."""

        class Response(BaseModel):
            id: str

        result = _filter_response({}, Response)
        expect(result).to_equal({})

    @test
    def test_filter_non_dict_passthrough(self):
        """Test that non-dict values pass through."""

        class Response(BaseModel):
            id: str

        expect(_filter_response("string", Response)).to_equal("string")
        expect(_filter_response(123, Response)).to_equal(123)

    @test
    def test_filter_empty_list(self):
        """Test filtering empty list."""

        class Response(BaseModel):
            id: str

        result = _filter_response([], Response)
        expect(result).to_equal([])


if __name__ == "__main__":
    import asyncio
    asyncio.run(TestBasicResponseFiltering().run())
    asyncio.run(TestNestedResponseFiltering().run())
    asyncio.run(TestListResponseFiltering().run())
    asyncio.run(TestEdgeCases().run())
