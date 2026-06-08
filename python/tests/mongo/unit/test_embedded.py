"""
Tests for embedded document support (EmbeddedDocument fields).

Beanie-compatible embedded documents allow type-safe nested structures.
Migrated from pytest to cclab.probe framework.
"""
from typing import Optional, List

from cclab import Document, EmbeddedDocument
from cclab.probe import test, expect
from tests.base import MongoTestSuite


# ===========================
# Embedded Document Models
# ===========================

class Address(EmbeddedDocument):
    """Embedded document for address."""
    city: str
    zip: str
    street: Optional[str] = None


class Coordinates(EmbeddedDocument):
    """Nested embedded document for coordinates."""
    lat: float
    lng: float


class Location(EmbeddedDocument):
    """Embedded document with nested EmbeddedDocument."""
    name: str
    coords: Coordinates


class Tag(EmbeddedDocument):
    """Simple embedded document for tags."""
    label: str
    color: Optional[str] = None


class EmbedUser(Document):
    """Document with embedded EmbeddedDocument fields."""
    name: str
    email: str
    address: Address
    tags: List[Tag]
    location: Optional[Location] = None

    class Settings:
        name = "test_embedded_users"


class EmbedCompany(Document):
    """Document with list of EmbeddedDocument."""
    name: str
    offices: List[Address]

    class Settings:
        name = "test_embedded_companies"


class EmbedProfile(Document):
    """Document with optional EmbeddedDocument."""
    username: str
    bio: Optional[Address] = None

    class Settings:
        name = "test_embedded_profiles"


# ===========================
# Basic Embedded Document Tests
# ===========================

class TestEmbeddedDocumentBasics(MongoTestSuite):
    """Basic embedded document tests."""

    async def setup(self):
        """Cleanup before each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})
        await _engine.delete_many("test_embedded_companies", {})
        await _engine.delete_many("test_embedded_profiles", {})

    async def teardown(self):
        """Cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})
        await _engine.delete_many("test_embedded_companies", {})
        await _engine.delete_many("test_embedded_profiles", {})

    @test(tags=["mongo", "embedded"])
    async def test_basic_embedded_document(self):
        """Test basic embedded document creation and retrieval."""
        user = EmbedUser(
            name="Alice",
            email="alice@example.com",
            address=Address(city="NYC", zip="10001", street="5th Ave"),
            tags=[Tag(label="developer", color="blue")]
        )

        await user.save()
        expect(user.id).not_.to_be_none()

        found = await EmbedUser.find_one(EmbedUser.email == "alice@example.com")
        expect(found).not_.to_be_none()
        expect(found.name).to_equal("Alice")

        expect(isinstance(found.address, Address)).to_be_true()
        expect(found.address.city).to_equal("NYC")
        expect(found.address.zip).to_equal("10001")
        expect(found.address.street).to_equal("5th Ave")

    @test(tags=["mongo", "embedded"])
    async def test_list_of_embedded_documents(self):
        """Test List[EmbeddedDocument] fields."""
        user = EmbedUser(
            name="Bob",
            email="bob@example.com",
            address=Address(city="LA", zip="90001"),
            tags=[
                Tag(label="python", color="yellow"),
                Tag(label="rust", color="orange"),
                Tag(label="mongodb", color="green"),
            ]
        )

        await user.save()

        found = await EmbedUser.find_one(EmbedUser.name == "Bob")
        expect(found).not_.to_be_none()
        expect(len(found.tags)).to_equal(3)

        expect(all(isinstance(tag, Tag) for tag in found.tags)).to_be_true()
        expect(found.tags[0].label).to_equal("python")
        expect(found.tags[0].color).to_equal("yellow")
        expect(found.tags[1].label).to_equal("rust")
        expect(found.tags[2].label).to_equal("mongodb")

    @test(tags=["mongo", "embedded"])
    async def test_optional_embedded_document(self):
        """Test Optional[EmbeddedDocument] fields."""
        profile1 = EmbedProfile(username="user1", bio=None)
        await profile1.save()

        profile2 = EmbedProfile(
            username="user2",
            bio=Address(city="SF", zip="94102")
        )
        await profile2.save()

        found1 = await EmbedProfile.find_one(EmbedProfile.username == "user1")
        expect(found1.bio).to_be_none()

        found2 = await EmbedProfile.find_one(EmbedProfile.username == "user2")
        expect(found2.bio).not_.to_be_none()
        expect(isinstance(found2.bio, Address)).to_be_true()
        expect(found2.bio.city).to_equal("SF")

    @test(tags=["mongo", "embedded"])
    async def test_nested_embedded_documents(self):
        """Test nested EmbeddedDocument fields."""
        user = EmbedUser(
            name="Charlie",
            email="charlie@example.com",
            address=Address(city="Boston", zip="02101"),
            tags=[],
            location=Location(
                name="Office",
                coords=Coordinates(lat=42.3601, lng=-71.0589)
            )
        )

        await user.save()

        found = await EmbedUser.find_one(EmbedUser.name == "Charlie")
        expect(found.location).not_.to_be_none()
        expect(isinstance(found.location, Location)).to_be_true()
        expect(found.location.name).to_equal("Office")

        expect(isinstance(found.location.coords, Coordinates)).to_be_true()
        expect(found.location.coords.lat).to_equal(42.3601)
        expect(found.location.coords.lng).to_equal(-71.0589)


class TestEmbeddedSerialization(MongoTestSuite):
    """Serialization tests for embedded documents."""

    async def setup(self):
        """Cleanup before each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})

    async def teardown(self):
        """Cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})

    @test(tags=["mongo", "embedded", "serialization"])
    async def test_to_dict_serializes_embedded_documents(self):
        """Test that to_dict() properly serializes EmbeddedDocument fields."""
        user = EmbedUser(
            name="Dave",
            email="dave@example.com",
            address=Address(city="Seattle", zip="98101"),
            tags=[Tag(label="devops", color="red")]
        )

        data = user.to_dict()

        expect(isinstance(data["address"], dict)).to_be_true()
        expect(data["address"]["city"]).to_equal("Seattle")
        expect(data["address"]["zip"]).to_equal("98101")

        expect(isinstance(data["tags"], list)).to_be_true()
        expect(isinstance(data["tags"][0], dict)).to_be_true()
        expect(data["tags"][0]["label"]).to_equal("devops")

    @test(tags=["mongo", "embedded", "serialization"])
    async def test_from_db_deserializes_embedded_documents(self):
        """Test that _from_db() properly deserializes EmbeddedDocument fields."""
        raw_data = {
            "name": "Eve",
            "email": "eve@example.com",
            "address": {"city": "Austin", "zip": "78701", "street": None},
            "tags": [
                {"label": "backend", "color": "purple"},
                {"label": "api", "color": None}
            ],
            "location": None
        }

        user = EmbedUser._from_db(raw_data)

        expect(isinstance(user.address, Address)).to_be_true()
        expect(user.address.city).to_equal("Austin")

        expect(isinstance(user.tags[0], Tag)).to_be_true()
        expect(user.tags[0].label).to_equal("backend")


class TestEmbeddedRoundTrip(MongoTestSuite):
    """Round-trip tests for embedded documents."""

    async def setup(self):
        """Cleanup before each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})

    async def teardown(self):
        """Cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})

    @test(tags=["mongo", "embedded", "roundtrip"])
    async def test_save_load_roundtrip(self):
        """Test that embedded documents survive save/load cycle."""
        original = EmbedUser(
            name="Frank",
            email="frank@example.com",
            address=Address(city="Denver", zip="80201", street="Main St"),
            tags=[
                Tag(label="cloud", color="blue"),
                Tag(label="kubernetes")
            ],
            location=Location(
                name="HQ",
                coords=Coordinates(lat=39.7392, lng=-104.9903)
            )
        )

        await original.save()
        user_id = original.id

        loaded = await EmbedUser.find_one(EmbedUser.id == user_id)

        expect(loaded.name).to_equal(original.name)
        expect(loaded.email).to_equal(original.email)

        expect(loaded.address.city).to_equal(original.address.city)
        expect(loaded.address.zip).to_equal(original.address.zip)
        expect(loaded.address.street).to_equal(original.address.street)

        expect(len(loaded.tags)).to_equal(len(original.tags))
        expect(loaded.tags[0].label).to_equal(original.tags[0].label)
        expect(loaded.tags[0].color).to_equal(original.tags[0].color)

        expect(loaded.location.name).to_equal(original.location.name)
        expect(loaded.location.coords.lat).to_equal(original.location.coords.lat)
        expect(loaded.location.coords.lng).to_equal(original.location.coords.lng)


class TestEmbeddedQueries(MongoTestSuite):
    """Query tests for embedded documents using dot notation."""

    async def setup(self):
        """Cleanup before each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})

    async def teardown(self):
        """Cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})

    @test(tags=["mongo", "embedded", "query"])
    async def test_query_by_embedded_field(self):
        """Test querying by embedded document field using dot notation."""
        user1 = EmbedUser(
            name="User1",
            email="user1@example.com",
            address=Address(city="NYC", zip="10001"),
            tags=[]
        )
        user2 = EmbedUser(
            name="User2",
            email="user2@example.com",
            address=Address(city="LA", zip="90001"),
            tags=[]
        )
        await user1.save()
        await user2.save()

        # Nested field access: EmbedUser.address.city → FieldProxy("address.city")
        found = await EmbedUser.find_one(EmbedUser.address.city == "NYC")

        expect(found).not_.to_be_none()
        expect(found.name).to_equal("User1")
        expect(found.address.city).to_equal("NYC")

    @test(tags=["mongo", "embedded", "query"])
    async def test_query_by_list_embedded_field(self):
        """Test querying array of embedded documents."""
        user = EmbedUser(
            name="TagUser",
            email="taguser@example.com",
            address=Address(city="Boston", zip="02101"),
            tags=[
                Tag(label="python", color="blue"),
                Tag(label="rust", color="orange")
            ]
        )
        await user.save()

        # Nested field access on list: EmbedUser.tags.label → FieldProxy("tags.label")
        found = await EmbedUser.find_one(EmbedUser.tags.label == "python")

        expect(found).not_.to_be_none()
        expect(found.name).to_equal("TagUser")


class TestEmbeddedEdgeCases(MongoTestSuite):
    """Edge case tests for embedded documents."""

    async def setup(self):
        """Cleanup before each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})
        await _engine.delete_many("test_embedded_companies", {})

    async def teardown(self):
        """Cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_embedded_users", {})
        await _engine.delete_many("test_embedded_companies", {})

    @test(tags=["mongo", "embedded", "edge-case"])
    async def test_empty_list_of_embedded_documents(self):
        """Test empty List[EmbeddedDocument] field."""
        user = EmbedUser(
            name="EmptyTags",
            email="emptytags@example.com",
            address=Address(city="Portland", zip="97201"),
            tags=[]
        )

        await user.save()
        found = await EmbedUser.find_one(EmbedUser.name == "EmptyTags")

        expect(found.tags).to_equal([])

    @test(tags=["mongo", "embedded", "edge-case"])
    async def test_update_embedded_document(self):
        """Test updating embedded document field."""
        user = EmbedUser(
            name="UpdateTest",
            email="update@example.com",
            address=Address(city="Original", zip="00000"),
            tags=[]
        )
        await user.save()

        user.address = Address(city="Updated", zip="11111", street="New Street")
        await user.save()

        found = await EmbedUser.find_one(EmbedUser.name == "UpdateTest")
        expect(found.address.city).to_equal("Updated")
        expect(found.address.zip).to_equal("11111")
        expect(found.address.street).to_equal("New Street")

    @test(tags=["mongo", "embedded", "edge-case"])
    async def test_modify_embedded_document_in_place(self):
        """Test modifying embedded document fields in place."""
        user = EmbedUser(
            name="ModifyTest",
            email="modify@example.com",
            address=Address(city="Before", zip="00000"),
            tags=[Tag(label="old")]
        )
        await user.save()

        user.address.city = "After"
        user.address.zip = "99999"
        user.tags[0].label = "new"

        await user.save()

        found = await EmbedUser.find_one(EmbedUser.name == "ModifyTest")
        expect(found.address.city).to_equal("After")
        expect(found.address.zip).to_equal("99999")
        expect(found.tags[0].label).to_equal("new")

    @test(tags=["mongo", "embedded", "edge-case"])
    async def test_multiple_documents_with_same_embedded_type(self):
        """Test multiple documents sharing same embedded document type."""
        company = EmbedCompany(
            name="MultiOffice Corp",
            offices=[
                Address(city="NYC", zip="10001"),
                Address(city="LA", zip="90001"),
                Address(city="SF", zip="94102")
            ]
        )

        await company.save()
        found = await EmbedCompany.find_one(EmbedCompany.name == "MultiOffice Corp")

        expect(len(found.offices)).to_equal(3)
        expect(all(isinstance(office, Address) for office in found.offices)).to_be_true()
        expect(found.offices[0].city).to_equal("NYC")
        expect(found.offices[1].city).to_equal("LA")
        expect(found.offices[2].city).to_equal("SF")


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestEmbeddedDocumentBasics,
        TestEmbeddedSerialization,
        TestEmbeddedRoundTrip,
        TestEmbeddedQueries,
        TestEmbeddedEdgeCases,
    ], verbose=True)
