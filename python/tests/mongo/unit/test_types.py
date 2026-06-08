"""
Tests for type handling and BSON types.

Tests for:
- Basic Python type handling (string, int, float, bool, list, dict)
- DateTime handling in queries
- BSON type round-trips (Decimal128, Binary)
- PydanticObjectId type

Migrated from test_comprehensive.py and split for maintainability.
"""
from datetime import datetime, timezone
from decimal import Decimal
from typing import Annotated, Optional

from pydantic import Field

from cclab import Document, PydanticObjectId
from cclab.probe import test, expect
from tests.base import MongoTestSuite, CommonTestSuite


# =====================
# Test Document Classes
# =====================

class F(Document):
    """Field helper for unit tests - provides clean field access."""
    name: str
    age: int = 0
    score: float = 0.0
    active: bool = True
    tags: Annotated[list, Field(default_factory=list)]
    metadata: Annotated[dict, Field(default_factory=dict)]
    created_at: Optional[datetime] = None
    event_date: Optional[datetime] = None

    class Settings:
        name = "_unit_test_fields"


class TypeTestDoc(Document):
    """Document for type handling tests."""
    name: str
    value: str = ""
    number: int = 0
    score: float = 0.0
    active: bool = True
    tags: Annotated[list, Field(default_factory=list)]
    metadata: Annotated[dict, Field(default_factory=dict)]
    created_at: Optional[datetime] = None

    class Settings:
        name = "test_type_docs"


class BsonTypeDoc(Document):
    """Document for BSON type tests."""
    name: str
    price: Optional[Decimal] = None
    data: Optional[bytes] = None

    class Settings:
        name = "test_bson_type_docs"


class ObjectIdDoc(Document):
    """Document for ObjectId tests."""
    name: str
    ref_id: Optional[PydanticObjectId] = None

    class Settings:
        name = "test_objectid_docs"


# =====================
# Basic Type Tests (Unit)
# =====================

class TestBasicTypeHandling(CommonTestSuite):
    """Test basic Python type handling in filters."""

    @test(tags=["unit", "types"])
    async def test_string_type(self):
        """Test string values in filter."""
        expr = F.name == "Alice"

        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"name": "Alice"})
        expect(isinstance(filter_dict["name"], str)).to_be_true()

    @test(tags=["unit", "types"])
    async def test_int_type(self):
        """Test integer values in filter."""
        expr = F.age == 30

        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"age": 30})
        expect(isinstance(filter_dict["age"], int)).to_be_true()

    @test(tags=["unit", "types"])
    async def test_float_type(self):
        """Test float values in filter."""
        expr = F.score == 95.5

        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"score": 95.5})
        expect(isinstance(filter_dict["score"], float)).to_be_true()

    @test(tags=["unit", "types"])
    async def test_bool_type(self):
        """Test boolean values in filter."""
        expr = F.active == True

        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"active": True})
        expect(isinstance(filter_dict["active"], bool)).to_be_true()

    @test(tags=["unit", "types"])
    async def test_list_type(self):
        """Test list values in filter."""
        tag_list = ["a", "b", "c"]
        expr = F.tags == tag_list

        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"tags": ["a", "b", "c"]})
        expect(isinstance(filter_dict["tags"], list)).to_be_true()

    @test(tags=["unit", "types"])
    async def test_dict_type(self):
        """Test dict values in filter (embedded document)."""
        meta = {"key": "value", "count": 42}
        expr = F.metadata == meta

        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"metadata": {"key": "value", "count": 42}})
        expect(isinstance(filter_dict["metadata"], dict)).to_be_true()


# =====================
# DateTime Tests (Unit)
# =====================

class TestDateTimeHandling(CommonTestSuite):
    """Test datetime handling in filters."""

    @test(tags=["unit", "types", "datetime"])
    async def test_datetime_comparison(self):
        """Test datetime in comparison query."""
        dt = datetime(2024, 1, 15, 12, 30, 0, tzinfo=timezone.utc)
        expr = F.created_at > dt

        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"created_at": {"$gt": dt}})

    @test(tags=["unit", "types", "datetime"])
    async def test_datetime_equality(self):
        """Test datetime equality filter."""
        dt = datetime(2024, 6, 15)
        expr = F.event_date == dt

        filter_dict = expr.to_filter()
        expect(filter_dict).to_equal({"event_date": dt})


# =====================
# PydanticObjectId Tests (Unit)
# =====================

class TestPydanticObjectIdBasic(CommonTestSuite):
    """Unit tests for PydanticObjectId type."""

    @test(tags=["unit", "types", "objectid"])
    async def test_create_new_objectid(self):
        """Test creating a new PydanticObjectId."""
        oid = PydanticObjectId()
        expect(len(oid)).to_equal(24)  # ObjectId hex string is 24 chars
        expect(isinstance(oid, str)).to_be_true()

    @test(tags=["unit", "types", "objectid"])
    async def test_create_from_string(self):
        """Test creating from hex string."""
        hex_str = "507f1f77bcf86cd799439011"
        oid = PydanticObjectId(hex_str)
        expect(str(oid)).to_equal(hex_str)

    @test(tags=["unit", "types", "objectid"])
    async def test_create_from_objectid(self):
        """Test creating from cclab.ObjectId."""
        from cclab import ObjectId

        rust_oid = ObjectId.new()
        oid = PydanticObjectId(rust_oid)
        expect(str(oid)).to_equal(str(rust_oid))

    @test(tags=["unit", "types", "objectid"])
    async def test_create_from_pydantic_objectid(self):
        """Test creating from another PydanticObjectId."""
        oid1 = PydanticObjectId()
        oid2 = PydanticObjectId(oid1)
        expect(str(oid1)).to_equal(str(oid2))

    @test(tags=["unit", "types", "objectid"])
    async def test_invalid_string_raises_error(self):
        """Test that invalid strings raise ValueError."""
        error_caught = False
        try:
            PydanticObjectId("invalid")
        except ValueError:
            error_caught = True

        expect(error_caught).to_be_true()

    @test(tags=["unit", "types", "objectid"])
    async def test_equality(self):
        """Test equality comparisons."""
        from cclab import ObjectId

        hex_str = "507f1f77bcf86cd799439011"
        oid1 = PydanticObjectId(hex_str)
        oid2 = PydanticObjectId(hex_str)
        rust_oid = ObjectId(hex_str)

        expect(oid1 == oid2).to_be_true()
        expect(oid1 == hex_str).to_be_true()
        expect(oid1 == rust_oid).to_be_true()

    @test(tags=["unit", "types", "objectid"])
    async def test_hash(self):
        """Test that PydanticObjectId is hashable."""
        oid = PydanticObjectId()
        hash_val = hash(oid)
        expect(isinstance(hash_val, int)).to_be_true()

        # Can be used in sets
        oid_set = {oid}
        expect(oid in oid_set).to_be_true()

    @test(tags=["unit", "types", "objectid"])
    async def test_to_object_id(self):
        """Test conversion to cclab.ObjectId."""
        from cclab import ObjectId

        oid = PydanticObjectId()
        rust_oid = oid.to_object_id()
        expect(isinstance(rust_oid, ObjectId)).to_be_true()
        expect(str(rust_oid)).to_equal(str(oid))

    @test(tags=["unit", "types", "objectid"])
    async def test_is_valid(self):
        """Test is_valid class method."""
        from cclab import ObjectId

        expect(PydanticObjectId.is_valid("507f1f77bcf86cd799439011")).to_be_true()
        expect(PydanticObjectId.is_valid(ObjectId.new())).to_be_true()
        expect(PydanticObjectId.is_valid(PydanticObjectId())).to_be_true()
        expect(PydanticObjectId.is_valid(None)).to_be_true()
        expect(PydanticObjectId.is_valid("invalid")).to_be_false()
        expect(PydanticObjectId.is_valid(12345)).to_be_false()


# =====================
# Type Integration Tests (MongoDB)
# =====================

class TestTypeRoundTrip(MongoTestSuite):
    """Integration tests for type round-trips."""

    async def setup(self):
        """Clean up test data."""
        await TypeTestDoc.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await TypeTestDoc.find().delete()

    @test(tags=["mongo", "types"])
    async def test_string_roundtrip(self):
        """Test string values persist correctly."""
        doc = TypeTestDoc(name="test", value="hello world")
        await doc.save()

        found = await TypeTestDoc.find_one(TypeTestDoc.name == "test")
        expect(found).not_.to_be_none()
        expect(found.value).to_equal("hello world")

    @test(tags=["mongo", "types"])
    async def test_int_roundtrip(self):
        """Test integer values persist correctly."""
        doc = TypeTestDoc(name="test_int", number=42)
        await doc.save()

        found = await TypeTestDoc.find_one(TypeTestDoc.name == "test_int")
        expect(found).not_.to_be_none()
        expect(found.number).to_equal(42)

    @test(tags=["mongo", "types"])
    async def test_float_roundtrip(self):
        """Test float values persist correctly."""
        doc = TypeTestDoc(name="test_float", score=3.14159)
        await doc.save()

        found = await TypeTestDoc.find_one(TypeTestDoc.name == "test_float")
        expect(found).not_.to_be_none()
        # Float comparison with tolerance
        expect(abs(found.score - 3.14159) < 0.0001).to_be_true()

    @test(tags=["mongo", "types"])
    async def test_bool_roundtrip(self):
        """Test boolean values persist correctly."""
        doc = TypeTestDoc(name="test_bool", active=False)
        await doc.save()

        found = await TypeTestDoc.find_one(TypeTestDoc.name == "test_bool")
        expect(found).not_.to_be_none()
        expect(found.active).to_be_false()

    @test(tags=["mongo", "types"])
    async def test_list_roundtrip(self):
        """Test list values persist correctly."""
        doc = TypeTestDoc(name="test_list", tags=["python", "rust", "mongodb"])
        await doc.save()

        found = await TypeTestDoc.find_one(TypeTestDoc.name == "test_list")
        expect(found).not_.to_be_none()
        expect(found.tags).to_equal(["python", "rust", "mongodb"])

    @test(tags=["mongo", "types"])
    async def test_dict_roundtrip(self):
        """Test dict values persist correctly."""
        doc = TypeTestDoc(name="test_dict", metadata={"key": "value", "count": 42})
        await doc.save()

        found = await TypeTestDoc.find_one(TypeTestDoc.name == "test_dict")
        expect(found).not_.to_be_none()
        expect(found.metadata["key"]).to_equal("value")
        expect(found.metadata["count"]).to_equal(42)

    @test(tags=["mongo", "types"])
    async def test_datetime_roundtrip(self):
        """Test datetime values persist correctly."""
        now = datetime.now(timezone.utc)
        doc = TypeTestDoc(name="test_datetime", created_at=now)
        await doc.save()

        found = await TypeTestDoc.find_one(TypeTestDoc.name == "test_datetime")
        expect(found).not_.to_be_none()
        expect(found.created_at).not_.to_be_none()
        # Compare timestamps (may have microsecond differences)
        expect(abs((found.created_at - now).total_seconds()) < 1).to_be_true()


# =====================
# BSON Type Tests (MongoDB)
# =====================

class TestBsonTypeRoundTrip(MongoTestSuite):
    """Integration tests for BSON type round-trips."""

    async def setup(self):
        """Clean up test data."""
        await BsonTypeDoc.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await BsonTypeDoc.find().delete()

    @test(tags=["mongo", "types", "bson"])
    async def test_decimal_roundtrip(self):
        """Test Decimal128 round-trip."""
        price = Decimal("123.45")
        doc = BsonTypeDoc(name="decimal_test", price=price)
        await doc.save()

        found = await BsonTypeDoc.find_one(BsonTypeDoc.name == "decimal_test")
        expect(found).not_.to_be_none()
        expect(found.price).not_.to_be_none()
        # Decimal should round-trip (may be converted to float)
        if isinstance(found.price, Decimal):
            expect(found.price).to_equal(price)
        else:
            expect(abs(float(found.price) - float(price)) < 0.001).to_be_true()

    @test(tags=["mongo", "types", "bson"])
    async def test_binary_roundtrip(self):
        """Test Binary data round-trip."""
        binary_data = b"\x00\x01\x02\x03\xff\xfe\xfd"
        doc = BsonTypeDoc(name="binary_test", data=binary_data)
        await doc.save()

        found = await BsonTypeDoc.find_one(BsonTypeDoc.name == "binary_test")
        expect(found).not_.to_be_none()
        expect(found.data).not_.to_be_none()
        expect(isinstance(found.data, bytes)).to_be_true()
        expect(found.data).to_equal(binary_data)


# =====================
# ObjectId Integration Tests (MongoDB)
# =====================

class TestObjectIdRoundTrip(MongoTestSuite):
    """Integration tests for PydanticObjectId with MongoDB."""

    async def setup(self):
        """Clean up test data."""
        await ObjectIdDoc.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await ObjectIdDoc.find().delete()

    @test(tags=["mongo", "types", "objectid"])
    async def test_objectid_in_document(self):
        """Test PydanticObjectId field persists correctly."""
        ref_id = PydanticObjectId()
        doc = ObjectIdDoc(name="oid_test", ref_id=ref_id)
        await doc.save()

        found = await ObjectIdDoc.find_one(ObjectIdDoc.name == "oid_test")
        expect(found).not_.to_be_none()
        expect(found.ref_id).not_.to_be_none()
        expect(str(found.ref_id)).to_equal(str(ref_id))

    @test(tags=["mongo", "types", "objectid"])
    async def test_query_by_objectid(self):
        """Test querying by ObjectId field."""
        ref_id = PydanticObjectId()
        doc = ObjectIdDoc(name="query_test", ref_id=ref_id)
        await doc.save()

        found = await ObjectIdDoc.find_one(ObjectIdDoc.ref_id == ref_id)
        expect(found).not_.to_be_none()
        expect(found.name).to_equal("query_test")


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestBasicTypeHandling,
        TestDateTimeHandling,
        TestPydanticObjectIdBasic,
        TestTypeRoundTrip,
        TestBsonTypeRoundTrip,
        TestObjectIdRoundTrip,
    ], verbose=True)
