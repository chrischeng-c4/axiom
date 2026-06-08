"""
Tests for programmatic migrations.

Tests for:
- Migration base class
- @iterative_migration decorator
- @free_fall_migration decorator
- run_migrations() function
- MigrationHistory tracking

Migrated from pytest to cclab.probe framework.
"""
from cclab import Document
from cclab.nebula.migrations import (
    Migration,
    iterative_migration,
    free_fall_migration,
    run_migrations,
    get_pending_migrations,
    get_applied_migrations,
    get_migration_status,
)
from cclab.probe import test, expect
from tests.base import MongoTestSuite, CommonTestSuite


# =====================
# Test Document Classes
# =====================

class MigrationTestUser(Document):
    """Test document for migration tests."""
    name: str
    email: str = ""
    status: str = "active"
    email_verified: bool = False

    class Settings:
        name = "test_migration_users"


# =====================
# Migration Classes for Testing
# =====================

class AddStatusField(Migration):
    """Test migration: add status field."""
    version = "001"
    description = "Add status field to users"

    async def forward(self):
        await MigrationTestUser.find().update({"$set": {"status": "pending"}})

    async def backward(self):
        await MigrationTestUser.find().update({"$unset": {"status": ""}})


class AddEmailVerified(Migration):
    """Test migration: add email_verified field."""
    version = "002"
    description = "Add email_verified field"

    async def forward(self):
        await MigrationTestUser.find().update({"$set": {"email_verified": False}})

    async def backward(self):
        await MigrationTestUser.find().update({"$unset": {"email_verified": ""}})


class NormalizeEmails(Migration):
    """Test migration: normalize emails to lowercase."""
    version = "003"
    description = "Normalize email addresses"

    async def forward(self):
        users = await MigrationTestUser.find().to_list()
        for user in users:
            if user.email:
                user.email = user.email.lower()
                await user.save()


class NoRollbackMigration(Migration):
    """Test migration without rollback support."""
    version = "004"
    description = "Migration without backward support"

    async def forward(self):
        pass
    # No backward() method - should raise NotImplementedError


# =====================
# Tests
# =====================

class TestMigrationBase(CommonTestSuite):
    """Tests for Migration base class (unit tests)."""

    @test(tags=["unit", "migrations"])
    async def test_migration_has_version(self):
        """Test that migration has version attribute."""
        expect(AddStatusField.version).to_equal("001")
        expect(AddEmailVerified.version).to_equal("002")

    @test(tags=["unit", "migrations"])
    async def test_migration_has_description(self):
        """Test that migration has description attribute."""
        expect(AddStatusField.description).to_equal("Add status field to users")


class TestMigrationForward(MongoTestSuite):
    """Tests for migration forward/backward operations."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    @test(tags=["mongo", "migrations"])
    async def test_migration_forward(self):
        """Test running migration forward."""
        await MigrationTestUser(name="Alice", status="active").save()
        await MigrationTestUser(name="Bob", status="active").save()

        migration = AddStatusField()
        await migration.forward()

        users = await MigrationTestUser.find().to_list()
        expect(all(u.status == "pending" for u in users)).to_be_true()

    @test(tags=["mongo", "migrations"])
    async def test_migration_backward(self):
        """Test running migration backward."""
        user = MigrationTestUser(name="Alice", status="pending")
        await user.save()

        migration = AddStatusField()
        await migration.backward()

        # Verify status is unset (default should apply on next load)

    @test(tags=["mongo", "migrations"])
    async def test_no_rollback_raises_error(self):
        """Test that migration without backward() raises NotImplementedError."""
        migration = NoRollbackMigration()

        error_caught = False
        try:
            await migration.backward()
        except NotImplementedError:
            error_caught = True

        expect(error_caught).to_be_true()


class TestIterativeMigration(MongoTestSuite):
    """Tests for @iterative_migration decorator."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    @test(tags=["mongo", "migrations", "iterative"])
    async def test_iterative_migration_transforms_all(self):
        """Test that iterative migration transforms all documents."""
        await MigrationTestUser(name="Alice", email="ALICE@EXAMPLE.COM").save()
        await MigrationTestUser(name="Bob", email="BOB@EXAMPLE.COM").save()
        await MigrationTestUser(name="Carol", email="Carol@Example.com").save()

        @iterative_migration(MigrationTestUser, batch_size=2)
        class LowercaseEmails:
            version = "010"
            description = "Lowercase all emails"

            async def transform(self, user: MigrationTestUser) -> MigrationTestUser:
                user.email = user.email.lower()
                return user

        migration = LowercaseEmails()
        await migration.forward()

        users = await MigrationTestUser.find().to_list()
        emails = [u.email for u in users]
        expect("alice@example.com" in emails).to_be_true()
        expect("bob@example.com" in emails).to_be_true()
        expect("carol@example.com" in emails).to_be_true()

    @test(tags=["mongo", "migrations", "iterative"])
    async def test_iterative_migration_preserves_ids(self):
        """Test that iterative migration preserves document IDs."""
        user = MigrationTestUser(name="Test", email="TEST@EXAMPLE.COM")
        await user.save()
        original_id = user._id

        @iterative_migration(MigrationTestUser)
        class Transform:
            version = "011"
            description = "Test transform"

            async def transform(self, user: MigrationTestUser) -> MigrationTestUser:
                user.email = user.email.lower()
                return user

        migration = Transform()
        await migration.forward()

        found = await MigrationTestUser.find_one(MigrationTestUser.email == "test@example.com")
        expect(found._id).to_equal(original_id)


class TestFreeFallMigration(MongoTestSuite):
    """Tests for @free_fall_migration decorator."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    @test(tags=["mongo", "migrations", "freefall"])
    async def test_freefall_custom_logic(self):
        """Test free-fall migration with custom logic."""
        await MigrationTestUser(name="Alice", email="alice@example.com").save()
        await MigrationTestUser(name="Bob", email="bob@example.com").save()
        await MigrationTestUser(name="Carol", email="carol@example.com").save()

        @free_fall_migration([MigrationTestUser])
        class CountUsers:
            version = "020"
            description = "Count users"

            async def forward(self):
                count = await MigrationTestUser.count()
                expect(count).to_equal(3)

        migration = CountUsers()
        await migration.forward()


class TestRunMigrations(MongoTestSuite):
    """Tests for run_migrations() function."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    @test(tags=["mongo", "migrations", "run"])
    async def test_run_forward_migrations(self):
        """Test running migrations forward."""
        await MigrationTestUser(name="Alice").save()

        applied = await run_migrations([AddStatusField, AddEmailVerified])

        expect(len(applied)).to_equal(2)
        expect("001" in applied).to_be_true()
        expect("002" in applied).to_be_true()

    @test(tags=["mongo", "migrations", "run"])
    async def test_skip_already_applied(self):
        """Test that already applied migrations are skipped."""
        await MigrationTestUser(name="Alice").save()

        applied1 = await run_migrations([AddStatusField])
        expect(len(applied1)).to_equal(1)

        applied2 = await run_migrations([AddStatusField])
        expect(len(applied2)).to_equal(0)

    @test(tags=["mongo", "migrations", "run"])
    async def test_run_to_target_version(self):
        """Test running migrations up to a target version."""
        await MigrationTestUser(name="Alice").save()

        applied = await run_migrations(
            [AddStatusField, AddEmailVerified, NormalizeEmails],
            target_version="002"
        )

        expect(len(applied)).to_equal(2)
        expect("001" in applied).to_be_true()
        expect("002" in applied).to_be_true()
        expect("003" in applied).to_be_false()

    @test(tags=["mongo", "migrations", "run"])
    async def test_run_backward_migrations(self):
        """Test running migrations backward (rollback)."""
        await MigrationTestUser(name="Alice").save()

        await run_migrations([AddStatusField, AddEmailVerified])

        rolled_back = await run_migrations(
            [AddStatusField, AddEmailVerified],
            direction="backward"
        )

        expect(len(rolled_back)).to_equal(2)

    @test(tags=["mongo", "migrations", "run"])
    async def test_rollback_to_target_version(self):
        """Test rolling back to a specific version."""
        await MigrationTestUser(name="Alice").save()

        await run_migrations([AddStatusField, AddEmailVerified])

        rolled_back = await run_migrations(
            [AddStatusField, AddEmailVerified],
            direction="backward",
            target_version="001"
        )

        expect(len(rolled_back)).to_equal(1)
        expect("002" in rolled_back).to_be_true()


class TestMigrationHistory(MongoTestSuite):
    """Tests for MigrationHistory tracking."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    @test(tags=["mongo", "migrations", "history"])
    async def test_history_recorded(self):
        """Test that migration history is recorded."""
        await MigrationTestUser(name="Alice").save()

        await run_migrations([AddStatusField])

        history = await get_applied_migrations()
        expect(len(history)).to_equal(1)
        expect(history[0].version).to_equal("001")
        expect(history[0].name).to_equal("AddStatusField")
        expect(history[0].direction).to_equal("forward")

    @test(tags=["mongo", "migrations", "history"])
    async def test_get_pending_migrations(self):
        """Test getting list of pending migrations."""
        pending = await get_pending_migrations([AddStatusField, AddEmailVerified])
        expect(len(pending)).to_equal(2)

        await run_migrations([AddStatusField])

        pending = await get_pending_migrations([AddStatusField, AddEmailVerified])
        expect(len(pending)).to_equal(1)
        expect(pending[0].version).to_equal("002")

    @test(tags=["mongo", "migrations", "history"])
    async def test_get_migration_status(self):
        """Test getting migration status."""
        await MigrationTestUser(name="Alice").save()
        await run_migrations([AddStatusField])

        status = await get_migration_status([AddStatusField, AddEmailVerified])

        expect(len(status)).to_equal(2)
        expect(status[0]["version"]).to_equal("001")
        expect(status[0]["applied"]).to_be_true()
        expect(status[1]["version"]).to_equal("002")
        expect(status[1]["applied"]).to_be_false()


class TestMigrationErrors(MongoTestSuite):
    """Tests for migration error handling."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_migration_users", {})
        await _engine.delete_many("_migrations", {})

    @test(tags=["mongo", "migrations", "errors"])
    async def test_rollback_without_backward_fails(self):
        """Test that rollback fails if backward() not implemented."""
        await MigrationTestUser(name="Alice").save()

        await run_migrations([NoRollbackMigration])

        error_caught = False
        try:
            await run_migrations([NoRollbackMigration], direction="backward")
        except RuntimeError as e:
            error_caught = True
            expect("does not support rollback" in str(e)).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["mongo", "migrations", "errors"])
    async def test_invalid_direction_raises_error(self):
        """Test that invalid direction raises ValueError."""
        error_caught = False
        try:
            await run_migrations([AddStatusField], direction="sideways")
        except ValueError as e:
            error_caught = True
            expect("Invalid direction" in str(e)).to_be_true()

        expect(error_caught).to_be_true()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestMigrationBase,
        TestMigrationForward,
        TestIterativeMigration,
        TestFreeFallMigration,
        TestRunMigrations,
        TestMigrationHistory,
        TestMigrationErrors,
    ], verbose=True)
