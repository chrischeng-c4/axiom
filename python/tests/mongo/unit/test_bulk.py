"""
Tests for fast path bulk operations (insert_many with dicts).

Tests that:
1. insert_many() accepts raw dicts
2. validate parameter controls validation
3. return_type parameter controls return format
4. Mixed lists (dicts + Documents) work correctly

Migrated from pytest to cclab.probe framework.
"""
from cclab import Document
from cclab.probe import test, expect
from tests.base import MongoTestSuite


class BulkTestUser(Document):
    """Test user model for bulk operations."""
    name: str
    email: str
    age: int

    class Settings:
        name = "bulk_test_users"


class BulkTestUserWithValidation(Document):
    """Test user model with custom validation."""
    name: str
    age: int

    class Settings:
        name = "bulk_test_users_validation"

    def __init__(self, **kwargs):
        if "age" in kwargs and kwargs["age"] < 0:
            raise ValueError("Age cannot be negative")
        super().__init__(**kwargs)


class TestBulkFastPath(MongoTestSuite):
    """Fast path tests for bulk operations."""

    async def setup(self):
        """Clean up test data."""
        await BulkTestUser.find().delete()
        await BulkTestUserWithValidation.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await BulkTestUser.find().delete()
        await BulkTestUserWithValidation.find().delete()

    @test(tags=["mongo", "bulk", "fast-path"])
    async def test_insert_many_with_dicts(self):
        """FAST PATH: insert_many() should accept raw dicts."""
        dicts = [
            {"name": "Alice", "email": "alice@example.com", "age": 30},
            {"name": "Bob", "email": "bob@example.com", "age": 25},
            {"name": "Charlie", "email": "charlie@example.com", "age": 35},
        ]

        ids = await BulkTestUser.insert_many(dicts)

        expect(len(ids)).to_equal(3)
        expect(all(isinstance(id_, str) for id_ in ids)).to_be_true()

        users = await BulkTestUser.find().to_list()
        expect(len(users)).to_equal(3)
        names = {u.name for u in users}
        expect(names).to_equal({"Alice", "Bob", "Charlie"})

    @test(tags=["mongo", "bulk"])
    async def test_insert_many_with_documents(self):
        """STANDARD PATH: insert_many() should work with Document instances."""
        users = [
            BulkTestUser(name="Alice", email="alice@example.com", age=30),
            BulkTestUser(name="Bob", email="bob@example.com", age=25),
        ]

        ids = await BulkTestUser.insert_many(users)

        expect(len(ids)).to_equal(2)
        expect(users[0].id).to_equal(ids[0])
        expect(users[1].id).to_equal(ids[1])

    @test(tags=["mongo", "bulk", "fast-path"])
    async def test_insert_many_mixed_list(self):
        """MIXED: insert_many() should handle mixed lists (dicts + Documents)."""
        documents = [
            BulkTestUser(name="Alice", email="alice@example.com", age=30),
            {"name": "Bob", "email": "bob@example.com", "age": 25},
            BulkTestUser(name="Charlie", email="charlie@example.com", age=35),
        ]

        ids = await BulkTestUser.insert_many(documents)

        expect(len(ids)).to_equal(3)
        expect(documents[0].id).to_equal(ids[0])
        expect(documents[2].id).to_equal(ids[2])

        users = await BulkTestUser.find().to_list()
        expect(len(users)).to_equal(3)


class TestBulkReturnType(MongoTestSuite):
    """Return type tests for bulk operations."""

    async def setup(self):
        """Clean up test data."""
        await BulkTestUser.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await BulkTestUser.find().delete()

    @test(tags=["mongo", "bulk", "return-type"])
    async def test_insert_many_return_ids(self):
        """RETURN TYPE: return_type="ids" should return list of ObjectId strings."""
        dicts = [
            {"name": "Alice", "email": "alice@example.com", "age": 30},
            {"name": "Bob", "email": "bob@example.com", "age": 25},
        ]

        result = await BulkTestUser.insert_many(dicts, return_type="ids")

        expect(isinstance(result, list)).to_be_true()
        expect(len(result)).to_equal(2)
        expect(all(isinstance(id_, str) for id_ in result)).to_be_true()

    @test(tags=["mongo", "bulk", "return-type"])
    async def test_insert_many_return_documents(self):
        """RETURN TYPE: return_type="documents" should return list of Document instances."""
        dicts = [
            {"name": "Alice", "email": "alice@example.com", "age": 30},
            {"name": "Bob", "email": "bob@example.com", "age": 25},
        ]

        result = await BulkTestUser.insert_many(dicts, return_type="documents")

        expect(isinstance(result, list)).to_be_true()
        expect(len(result)).to_equal(2)
        expect(all(isinstance(doc, BulkTestUser) for doc in result)).to_be_true()
        expect(result[0].name).to_equal("Alice")
        expect(result[1].name).to_equal("Bob")
        expect(result[0].id).not_.to_be_none()
        expect(result[1].id).not_.to_be_none()

    @test(tags=["mongo", "bulk", "return-type"])
    async def test_insert_many_return_documents_mixed(self):
        """RETURN TYPE: return_type="documents" with mixed input."""
        doc = BulkTestUser(name="Alice", email="alice@example.com", age=30)
        documents = [
            doc,
            {"name": "Bob", "email": "bob@example.com", "age": 25},
        ]

        result = await BulkTestUser.insert_many(documents, return_type="documents")

        expect(len(result)).to_equal(2)
        expect(result[0] is doc).to_be_true()
        expect(isinstance(result[1], BulkTestUser)).to_be_true()
        expect(result[1].name).to_equal("Bob")


class TestBulkValidation(MongoTestSuite):
    """Validation tests for bulk operations."""

    async def setup(self):
        """Clean up test data."""
        await BulkTestUserWithValidation.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await BulkTestUserWithValidation.find().delete()

    @test(tags=["mongo", "bulk", "validation"])
    async def test_insert_many_validate_false_skips_validation(self):
        """VALIDATION: validate=False should skip validation (default)."""
        dicts = [
            {"name": "Alice", "age": -5},  # Invalid age for custom validation
        ]

        ids = await BulkTestUserWithValidation.insert_many(dicts, validate=False)
        expect(len(ids)).to_equal(1)

    @test(tags=["mongo", "bulk", "validation"])
    async def test_insert_many_validate_true_validates_dicts(self):
        """VALIDATION: validate=True should validate dicts against model."""
        dicts = [
            {"name": "Alice", "age": -5},  # Invalid age
        ]

        error_caught = False
        try:
            await BulkTestUserWithValidation.insert_many(dicts, validate=True)
        except ValueError as e:
            error_caught = True
            expect("Age cannot be negative" in str(e)).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["mongo", "bulk", "validation"])
    async def test_insert_many_validate_true_valid_dicts(self):
        """VALIDATION: validate=True with valid dicts should succeed."""
        dicts = [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25},
        ]

        ids = await BulkTestUserWithValidation.insert_many(dicts, validate=True)
        expect(len(ids)).to_equal(2)


class TestBulkCorrectness(MongoTestSuite):
    """Correctness tests for bulk operations."""

    async def setup(self):
        """Clean up test data."""
        await BulkTestUser.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await BulkTestUser.find().delete()

    @test(tags=["mongo", "bulk", "correctness"])
    async def test_fast_path_data_integrity(self):
        """CORRECTNESS: Fast path should insert data correctly."""
        dicts = [
            {"name": "Alice", "email": "alice@example.com", "age": 30},
            {"name": "Bob", "email": "bob@example.com", "age": 25},
        ]

        ids = await BulkTestUser.insert_many(dicts)
        expect(len(ids)).to_equal(2)

        users = await BulkTestUser.find().to_list()
        expect(len(users)).to_equal(2)

        user_by_name = {u.name: u for u in users}

        for dict_data in dicts:
            user = user_by_name.get(dict_data["name"])
            expect(user).not_.to_be_none()
            expect(user.email).to_equal(dict_data["email"])
            expect(user.age).to_equal(dict_data["age"])
            expect(user.id in ids).to_be_true()

    @test(tags=["mongo", "bulk", "edge-case"])
    async def test_fast_path_empty_list(self):
        """EDGE CASE: Empty list should return empty list."""
        ids = await BulkTestUser.insert_many([])
        expect(ids).to_equal([])

    @test(tags=["mongo", "bulk", "performance"])
    async def test_fast_path_large_batch(self):
        """PERFORMANCE: Fast path should handle large batches."""
        dicts = [
            {"name": f"User{i}", "email": f"user{i}@example.com", "age": 20 + (i % 50)}
            for i in range(100)
        ]

        ids = await BulkTestUser.insert_many(dicts)

        expect(len(ids)).to_equal(100)

        count = await BulkTestUser.find().count()
        expect(count).to_equal(100)


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestBulkFastPath,
        TestBulkReturnType,
        TestBulkValidation,
        TestBulkCorrectness,
    ], verbose=True)
