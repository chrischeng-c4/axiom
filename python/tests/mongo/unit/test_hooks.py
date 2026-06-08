"""
Tests for lifecycle hooks, revision tracking, and state management.

Tests for:
- Lifecycle hooks (before_event, after_event)
- Insert, Delete, Replace hooks
- Async hooks
- Revision tracking (optimistic locking)
- State management (change tracking, rollback)

Migrated from test_comprehensive.py and split for maintainability.
"""
from datetime import datetime, timezone
from typing import Optional

from cclab import Document, before_event, after_event, Insert, Delete, Replace
from cclab.probe import test, expect
from tests.base import MongoTestSuite


# =====================
# Lifecycle Hooks Tests
# =====================

class TestBeforeInsertHook(MongoTestSuite):
    """Tests for before_event(Insert) hook."""

    async def setup(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_articles_hooks", {})

    async def teardown(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_articles_hooks", {})

    @test(tags=["mongo", "hooks", "lifecycle"])
    async def test_before_insert_sets_field(self):
        """Test before_event(Insert) hook sets field."""
        class Article(Document):
            title: str
            created_at: Optional[datetime] = None

            class Settings:
                name = "test_articles_hooks"

            @before_event(Insert)
            def set_created_at(self):
                self.created_at = datetime.now(timezone.utc)

        article = Article(title="Test Article")
        expect(article.created_at).to_be_none()

        await article.save()

        expect(article.created_at).not_.to_be_none()
        await article.delete()


class TestAfterInsertHook(MongoTestSuite):
    """Tests for after_event(Insert) hook."""

    async def setup(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_notifications_hooks", {})

    async def teardown(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_notifications_hooks", {})

    @test(tags=["mongo", "hooks", "lifecycle"])
    async def test_after_insert_hook_called(self):
        """Test after_event(Insert) hook is called."""
        hook_called = {"count": 0}

        class Notification(Document):
            message: str

            class Settings:
                name = "test_notifications_hooks"

            @after_event(Insert)
            def on_insert(self):
                hook_called["count"] += 1

        notification = Notification(message="Test")
        expect(hook_called["count"]).to_equal(0)

        await notification.save()

        expect(hook_called["count"]).to_equal(1)
        await notification.delete()


class TestBeforeDeleteHook(MongoTestSuite):
    """Tests for before_event(Delete) hook."""

    async def setup(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_logs_delete_hook", {})

    async def teardown(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_logs_delete_hook", {})

    @test(tags=["mongo", "hooks", "lifecycle"])
    async def test_before_delete_hook_called(self):
        """Test before_event(Delete) hook is called."""
        deleted_messages = []

        class Log(Document):
            message: str

            class Settings:
                name = "test_logs_delete_hook"

            @before_event(Delete)
            def log_deletion(self):
                deleted_messages.append(self.message)

        log = Log(message="Important log")
        await log.save()

        expect(len(deleted_messages)).to_equal(0)

        await log.delete()

        expect(len(deleted_messages)).to_equal(1)
        expect(deleted_messages[0]).to_equal("Important log")


class TestMultipleHooks(MongoTestSuite):
    """Tests for multiple hooks on same document."""

    async def setup(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_tracked_items", {})

    async def teardown(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_tracked_items", {})

    @test(tags=["mongo", "hooks", "lifecycle"])
    async def test_multiple_hooks_fire_in_order(self):
        """Test multiple hooks on same document fire correctly."""
        events_log = []

        class TrackedItem(Document):
            name: str
            created_at: Optional[datetime] = None

            class Settings:
                name = "test_tracked_items"

            @before_event(Insert)
            def before_insert(self):
                self.created_at = datetime.now(timezone.utc)
                events_log.append(f"before_insert:{self.name}")

            @after_event(Insert)
            def after_insert(self):
                events_log.append(f"after_insert:{self.name}")

            @before_event(Delete)
            def before_delete(self):
                events_log.append(f"before_delete:{self.name}")

            @after_event(Delete)
            def after_delete(self):
                events_log.append(f"after_delete:{self.name}")

        events_log.clear()
        item = TrackedItem(name="Widget")
        await item.save()

        expect("before_insert:Widget" in events_log).to_be_true()
        expect("after_insert:Widget" in events_log).to_be_true()
        expect(item.created_at).not_.to_be_none()

        await item.delete()

        expect("before_delete:Widget" in events_log).to_be_true()
        expect("after_delete:Widget" in events_log).to_be_true()


class TestAsyncHook(MongoTestSuite):
    """Tests for async hook functions."""

    async def setup(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_async_hooks", {})

    async def teardown(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_async_hooks", {})

    @test(tags=["mongo", "hooks", "lifecycle", "async"])
    async def test_async_hook_called(self):
        """Test async hook function is awaited."""
        import asyncio
        async_results = []

        class AsyncDoc(Document):
            value: str

            class Settings:
                name = "test_async_hooks"

            @after_event(Insert)
            async def async_after_insert(self):
                await asyncio.sleep(0.01)  # Simulate async work
                async_results.append(self.value)

        async_results.clear()
        doc = AsyncDoc(value="async_test")
        await doc.save()

        expect("async_test" in async_results).to_be_true()
        await doc.delete()


class TestBeforeReplaceHook(MongoTestSuite):
    """Tests for before_event(Replace) hook."""

    async def setup(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_versioned_hooks", {})

    async def teardown(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_versioned_hooks", {})

    @test(tags=["mongo", "hooks", "lifecycle"])
    async def test_before_replace_bumps_version(self):
        """Test before_event(Replace) hook modifies document."""
        class Versioned(Document):
            name: str
            version: int = 1
            updated_at: Optional[datetime] = None

            class Settings:
                name = "test_versioned_hooks"

            @before_event(Replace)
            def bump_version(self):
                self.version += 1
                self.updated_at = datetime.now(timezone.utc)

        item = Versioned(name="Original")
        await item.save()
        expect(item.version).to_equal(1)
        expect(item.updated_at).to_be_none()

        item.name = "Updated"
        await item.replace()

        expect(item.version).to_equal(2)
        expect(item.updated_at).not_.to_be_none()
        await item.delete()


# =====================
# Revision Tracking Tests
# =====================

class TestRevisionTracking(MongoTestSuite):
    """Tests for revision tracking (optimistic locking)."""

    async def setup(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_revisioned", {})

    async def teardown(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_revisioned", {})

    @test(tags=["mongo", "hooks", "revision"])
    async def test_revision_on_insert(self):
        """Test revision_id is set on insert."""
        class RevisionedDoc(Document):
            name: str

            class Settings:
                name = "test_revisioned"
                use_revision = True

        doc = RevisionedDoc(name="Test")
        expect(doc.revision_id).to_be_none()

        await doc.save()
        expect(doc.revision_id).to_equal(1)
        await doc.delete()

    @test(tags=["mongo", "hooks", "revision"])
    async def test_revision_increments_on_update(self):
        """Test revision_id increments on updates."""
        class RevisionedDoc(Document):
            name: str

            class Settings:
                name = "test_revisioned"
                use_revision = True

        doc = RevisionedDoc(name="Test")
        await doc.save()
        expect(doc.revision_id).to_equal(1)

        doc.name = "Updated"
        await doc.save()
        expect(doc.revision_id).to_equal(2)

        doc.name = "Updated Again"
        await doc.save()
        expect(doc.revision_id).to_equal(3)
        await doc.delete()

    @test(tags=["mongo", "hooks", "revision"])
    async def test_revision_persists_to_db(self):
        """Test revision_id is stored in database."""
        class RevisionedDoc(Document):
            name: str

            class Settings:
                name = "test_revisioned"
                use_revision = True

        doc = RevisionedDoc(name="Test")
        await doc.save()
        doc_id = doc.id

        doc.name = "Updated"
        await doc.save()
        expect(doc.revision_id).to_equal(2)

        # Reload from database
        loaded = await RevisionedDoc.get(doc_id)
        expect(loaded.revision_id).to_equal(2)
        await doc.delete()

    @test(tags=["mongo", "hooks", "revision"])
    async def test_revision_conflict_detection(self):
        """Test optimistic locking detects conflicts."""
        class RevisionedDoc(Document):
            name: str

            class Settings:
                name = "test_revisioned"
                use_revision = True

        # Create document
        doc1 = RevisionedDoc(name="Original")
        await doc1.save()
        doc_id = doc1.id

        # Load same document in another "session"
        doc2 = await RevisionedDoc.get(doc_id)

        # First update succeeds
        doc1.name = "Updated by doc1"
        await doc1.save()
        expect(doc1.revision_id).to_equal(2)

        # Second update should fail (revision conflict)
        doc2.name = "Updated by doc2"
        error_caught = False
        try:
            await doc2.save()
        except ValueError as e:
            error_caught = True
            expect("Revision conflict" in str(e)).to_be_true()

        expect(error_caught).to_be_true()
        await doc1.delete()


# =====================
# State Management Tests
# =====================

class TestStateManagement(MongoTestSuite):
    """Tests for state management (change tracking)."""

    async def setup(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_tracked", {})

    async def teardown(self):
        """Clean up test data."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_tracked", {})

    @test(tags=["mongo", "hooks", "state"])
    async def test_is_changed_property(self):
        """Test is_changed property."""
        class TrackedDoc(Document):
            name: str
            value: int = 0

            class Settings:
                name = "test_tracked"
                use_state_management = True

        doc = TrackedDoc(name="Test", value=10)
        await doc.save()

        # No changes after save
        expect(doc.is_changed).to_be_false()

        # Modify a field
        doc.name = "Modified"
        expect(doc.is_changed).to_be_true()

        # Save clears changes
        await doc.save()
        expect(doc.is_changed).to_be_false()
        await doc.delete()

    @test(tags=["mongo", "hooks", "state"])
    async def test_has_changed_method(self):
        """Test has_changed() for specific fields."""
        class TrackedDoc(Document):
            name: str
            value: int = 0

            class Settings:
                name = "test_tracked"
                use_state_management = True

        doc = TrackedDoc(name="Test", value=10)
        await doc.save()

        # Modify only name
        doc.name = "Modified"

        expect(doc.has_changed("name")).to_be_true()
        expect(doc.has_changed("value")).to_be_false()
        await doc.delete()

    @test(tags=["mongo", "hooks", "state"])
    async def test_get_changes(self):
        """Test get_changes() returns modified fields."""
        class TrackedDoc(Document):
            name: str
            value: int = 0
            status: str = "active"

            class Settings:
                name = "test_tracked"
                use_state_management = True

        doc = TrackedDoc(name="Test", value=10, status="active")
        await doc.save()

        # Modify multiple fields
        doc.name = "Modified"
        doc.value = 20

        changes = doc.get_changes()
        expect(changes).to_equal({"name": "Modified", "value": 20})
        expect("status" in changes).to_be_false()
        await doc.delete()

    @test(tags=["mongo", "hooks", "state"])
    async def test_get_previous_changes(self):
        """Test get_previous_changes() after save."""
        class TrackedDoc(Document):
            name: str
            value: int = 0

            class Settings:
                name = "test_tracked"
                use_state_management = True

        doc = TrackedDoc(name="Test", value=10)
        await doc.save()

        # First update
        doc.name = "First Update"
        await doc.save()

        previous = doc.get_previous_changes()
        expect(previous).to_equal({"name": "First Update"})

        # Second update
        doc.value = 20
        await doc.save()

        previous = doc.get_previous_changes()
        expect(previous).to_equal({"value": 20})
        await doc.delete()

    @test(tags=["mongo", "hooks", "state"])
    async def test_rollback(self):
        """Test rollback() reverts changes."""
        class TrackedDoc(Document):
            name: str
            value: int = 0

            class Settings:
                name = "test_tracked"
                use_state_management = True

        doc = TrackedDoc(name="Original", value=10)
        await doc.save()

        # Modify fields
        doc.name = "Modified"
        doc.value = 99
        expect(doc.is_changed).to_be_true()

        # Rollback
        doc.rollback()

        expect(doc.name).to_equal("Original")
        expect(doc.value).to_equal(10)
        expect(doc.is_changed).to_be_false()
        await doc.delete()

    @test(tags=["mongo", "hooks", "state"])
    async def test_state_after_load(self):
        """Test state is properly initialized after loading from DB."""
        class TrackedDoc(Document):
            name: str
            value: int = 0

            class Settings:
                name = "test_tracked"
                use_state_management = True

        doc = TrackedDoc(name="Test", value=10)
        await doc.save()
        doc_id = doc.id

        # Load from database
        loaded = await TrackedDoc.get(doc_id)
        expect(loaded.is_changed).to_be_false()

        # Modify and check
        loaded.name = "Modified"
        expect(loaded.is_changed).to_be_true()
        expect(loaded.get_changes()).to_equal({"name": "Modified"})
        await doc.delete()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestBeforeInsertHook,
        TestAfterInsertHook,
        TestBeforeDeleteHook,
        TestMultipleHooks,
        TestAsyncHook,
        TestBeforeReplaceHook,
        TestRevisionTracking,
        TestStateManagement,
    ], verbose=True)
