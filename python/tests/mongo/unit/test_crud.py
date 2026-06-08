"""
Tests for CRUD operations.

Tests for:
- Upsert operations
- Replace operations
- Distinct queries
- Find-one-and-modify operations
- Basic save/delete/update

Migrated from test_comprehensive.py and split for maintainability.
"""
from cclab import Document
from cclab.probe import test, expect
from tests.base import MongoTestSuite


# =====================
# Test Document Classes
# =====================

class CrudTestUser(Document):
    """Test user for CRUD tests."""
    name: str
    email: str = ""
    age: int = 0
    status: str = "active"

    class Settings:
        name = "test_crud_users"


class Counter(Document):
    """Counter document for upsert tests."""
    name: str
    value: int = 0

    class Settings:
        name = "test_counters"


class Profile(Document):
    """Profile document for replace tests."""
    username: str
    bio: str = ""
    followers: int = 0

    class Settings:
        name = "test_profiles_crud"


class Employee(Document):
    """Employee document for distinct tests."""
    name: str
    department: str
    salary: float = 0.0

    class Settings:
        name = "test_employees_crud"


# =====================
# Basic CRUD Tests
# =====================

class TestBasicCRUD(MongoTestSuite):
    """Basic Create, Read, Update, Delete tests."""

    async def setup(self):
        """Clean up test data."""
        await CrudTestUser.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await CrudTestUser.find().delete()

    @test(tags=["mongo", "crud"])
    async def test_save_new_document(self):
        """Test saving a new document."""
        user = CrudTestUser(name="Alice", email="alice@example.com")
        await user.save()

        expect(user.id).not_.to_be_none()
        expect(user._id).not_.to_be_none()

    @test(tags=["mongo", "crud"])
    async def test_save_updates_existing(self):
        """Test that save updates existing document."""
        user = CrudTestUser(name="Bob", email="bob@example.com")
        await user.save()
        original_id = user.id

        user.email = "bob.updated@example.com"
        await user.save()

        expect(user.id).to_equal(original_id)

        found = await CrudTestUser.find_one(CrudTestUser.id == original_id)
        expect(found.email).to_equal("bob.updated@example.com")

    @test(tags=["mongo", "crud"])
    async def test_delete_document(self):
        """Test deleting a document."""
        user = CrudTestUser(name="ToDelete")
        await user.save()
        user_id = user.id

        await user.delete()

        found = await CrudTestUser.find_one(CrudTestUser.id == user_id)
        expect(found).to_be_none()

    @test(tags=["mongo", "crud"])
    async def test_get_by_id(self):
        """Test getting document by ID."""
        user = CrudTestUser(name="GetById")
        await user.save()

        found = await CrudTestUser.get(user.id)
        expect(found).not_.to_be_none()
        expect(found.name).to_equal("GetById")

    @test(tags=["mongo", "crud"])
    async def test_update_many(self):
        """Test updating multiple documents."""
        await CrudTestUser(name="U1", status="pending").save()
        await CrudTestUser(name="U2", status="pending").save()
        await CrudTestUser(name="U3", status="active").save()

        # QueryBuilder.update() returns count of modified documents
        modified_count = await CrudTestUser.find(
            CrudTestUser.status == "pending"
        ).update({"$set": {"status": "processed"}})

        expect(modified_count).to_equal(2)

        pending_count = await CrudTestUser.find(
            CrudTestUser.status == "pending"
        ).count()
        expect(pending_count).to_equal(0)

    @test(tags=["mongo", "crud"])
    async def test_delete_many(self):
        """Test deleting multiple documents."""
        await CrudTestUser(name="D1", status="deleted").save()
        await CrudTestUser(name="D2", status="deleted").save()
        await CrudTestUser(name="D3", status="active").save()

        deleted = await CrudTestUser.find(
            CrudTestUser.status == "deleted"
        ).delete()

        expect(deleted).to_equal(2)

        total = await CrudTestUser.find().count()
        expect(total).to_equal(1)


# =====================
# Upsert Tests
# =====================

class TestUpsertOperations(MongoTestSuite):
    """Tests for upsert operations."""

    async def setup(self):
        """Clean up test data."""
        await Counter.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await Counter.find().delete()

    @test(tags=["mongo", "crud", "upsert"])
    async def test_upsert_insert(self):
        """Test upsert creating a new document."""
        result = await Counter.find(Counter.name == "visits").upsert(
            {"$set": {"name": "visits", "value": 0}}
        )

        expect(result.get("upserted_id")).not_.to_be_none()
        expect(result["matched_count"]).to_equal(0)

        counter = await Counter.find_one(Counter.name == "visits")
        expect(counter).not_.to_be_none()
        expect(counter.value).to_equal(0)

    @test(tags=["mongo", "crud", "upsert"])
    async def test_upsert_update(self):
        """Test upsert updating existing document."""
        counter = Counter(name="pageviews", value=100)
        await counter.save()

        result = await Counter.find(Counter.name == "pageviews").upsert(
            {"$inc": {"value": 10}}
        )

        expect(result.get("upserted_id")).to_be_none()
        expect(result["matched_count"]).to_equal(1)

        found = await Counter.find_one(Counter.name == "pageviews")
        expect(found.value).to_equal(110)

    @test(tags=["mongo", "crud", "upsert"])
    async def test_upsert_with_setOnInsert(self):
        """Test upsert with $setOnInsert."""
        result = await Counter.find(Counter.name == "new_counter").upsert(
            {
                "$setOnInsert": {"value": 1000},
                "$set": {"name": "new_counter"}
            }
        )

        expect(result.get("upserted_id")).not_.to_be_none()

        counter = await Counter.find_one(Counter.name == "new_counter")
        expect(counter.value).to_equal(1000)


# =====================
# Replace Tests
# =====================

class TestReplaceOperations(MongoTestSuite):
    """Tests for replace operations."""

    async def setup(self):
        """Clean up test data."""
        await Profile.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await Profile.find().delete()

    @test(tags=["mongo", "crud", "replace"])
    async def test_replace_instance(self):
        """Test replace() instance method."""
        profile = Profile(username="alice", bio="Original bio", followers=100)
        await profile.save()
        original_id = profile.id

        profile.bio = "New bio"
        profile.followers = 200
        profile.username = "alice_updated"
        await profile.replace()

        found = await Profile.get(original_id)
        expect(found.username).to_equal("alice_updated")
        expect(found.bio).to_equal("New bio")
        expect(found.followers).to_equal(200)

    @test(tags=["mongo", "crud", "replace"])
    async def test_replace_one_class_method(self):
        """Test replace_one() class method."""
        profile = Profile(username="bob", bio="Old bio", followers=50)
        await profile.save()

        result = await Profile.replace_one(
            Profile.username == "bob",
            {"username": "bob", "bio": "Replaced bio", "followers": 100}
        )

        expect(result["matched_count"]).to_equal(1)
        expect(result["modified_count"]).to_equal(1)

        found = await Profile.find_one(Profile.username == "bob")
        expect(found.bio).to_equal("Replaced bio")
        expect(found.followers).to_equal(100)

    @test(tags=["mongo", "crud", "replace"])
    async def test_replace_one_with_upsert(self):
        """Test replace_one with upsert=True."""
        result = await Profile.replace_one(
            Profile.username == "new_user",
            {"username": "new_user", "bio": "created_via_upsert", "followers": 0},
            upsert=True
        )

        expect(result.get("upserted_id")).not_.to_be_none()
        expect(result["matched_count"]).to_equal(0)

        found = await Profile.find_one(Profile.username == "new_user")
        expect(found).not_.to_be_none()
        expect(found.bio).to_equal("created_via_upsert")


# =====================
# Distinct Tests
# =====================

class TestDistinctOperations(MongoTestSuite):
    """Tests for distinct operations."""

    async def setup(self):
        """Clean up test data."""
        await Employee.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await Employee.find().delete()

    @test(tags=["mongo", "crud", "distinct"])
    async def test_distinct_basic(self):
        """Test basic distinct query."""
        employees = [
            Employee(name="Alice", department="Engineering"),
            Employee(name="Bob", department="Engineering"),
            Employee(name="Charlie", department="Sales"),
            Employee(name="Diana", department="Marketing"),
            Employee(name="Eve", department="Sales"),
        ]
        await Employee.insert_many(employees)

        departments = await Employee.distinct("department")
        expect(sorted(departments)).to_equal(["Engineering", "Marketing", "Sales"])

    @test(tags=["mongo", "crud", "distinct"])
    async def test_distinct_with_filter(self):
        """Test distinct with filter."""
        employees = [
            Employee(name="A", department="Eng", salary=100000),
            Employee(name="B", department="Eng", salary=50000),
            Employee(name="C", department="Sales", salary=80000),
            Employee(name="D", department="Sales", salary=40000),
        ]
        await Employee.insert_many(employees)

        # Distinct departments for high earners
        high_earner_depts = await Employee.distinct(
            "department",
            {"salary": {"$gte": 80000}}
        )
        expect(sorted(high_earner_depts)).to_equal(["Eng", "Sales"])

    @test(tags=["mongo", "crud", "distinct"])
    async def test_distinct_with_field_proxy(self):
        """Test distinct using FieldProxy."""
        employees = [
            Employee(name="A", department="HR"),
            Employee(name="B", department="IT"),
            Employee(name="C", department="HR"),
        ]
        await Employee.insert_many(employees)

        departments = await Employee.distinct(Employee.department)
        expect(sorted(departments)).to_equal(["HR", "IT"])


# =====================
# Find and Modify Tests
# =====================

class TestFindAndModify(MongoTestSuite):
    """Tests for find_one_and_* operations."""

    async def setup(self):
        """Clean up test data."""
        await Counter.find().delete()

    async def teardown(self):
        """Clean up test data."""
        await Counter.find().delete()

    @test(tags=["mongo", "crud", "find-modify"])
    async def test_find_one_and_update(self):
        """Test find_one_and_update."""
        counter = Counter(name="atomic", value=10)
        await counter.save()

        # Update and return new value
        result = await Counter.find_one_and_update(
            Counter.name == "atomic",
            {"$inc": {"value": 5}},
            return_document="after"
        )

        expect(result).not_.to_be_none()
        expect(result.value).to_equal(15)

    @test(tags=["mongo", "crud", "find-modify"])
    async def test_find_one_and_update_return_before(self):
        """Test find_one_and_update returning document before update."""
        counter = Counter(name="before_test", value=20)
        await counter.save()

        result = await Counter.find_one_and_update(
            Counter.name == "before_test",
            {"$inc": {"value": 10}},
            return_document="before"
        )

        expect(result).not_.to_be_none()
        expect(result.value).to_equal(20)

        # Verify actual update happened
        found = await Counter.find_one(Counter.name == "before_test")
        expect(found.value).to_equal(30)

    @test(tags=["mongo", "crud", "find-modify"])
    async def test_find_one_and_delete(self):
        """Test find_one_and_delete."""
        counter = Counter(name="to_delete", value=999)
        await counter.save()

        result = await Counter.find_one_and_delete(Counter.name == "to_delete")

        expect(result).not_.to_be_none()
        expect(result.value).to_equal(999)

        # Verify deleted
        found = await Counter.find_one(Counter.name == "to_delete")
        expect(found).to_be_none()

    @test(tags=["mongo", "crud", "find-modify"])
    async def test_find_one_and_replace(self):
        """Test find_one_and_replace."""
        counter = Counter(name="to_replace", value=1)
        await counter.save()

        result = await Counter.find_one_and_replace(
            Counter.name == "to_replace",
            {"name": "replaced", "value": 100},
            return_document="after"
        )

        expect(result).not_.to_be_none()
        expect(result.name).to_equal("replaced")
        expect(result.value).to_equal(100)


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestBasicCRUD,
        TestUpsertOperations,
        TestReplaceOperations,
        TestDistinctOperations,
        TestFindAndModify,
    ], verbose=True)
