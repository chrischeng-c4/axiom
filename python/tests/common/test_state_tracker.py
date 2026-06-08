"""
Unit tests for StateTracker (Copy-on-Write state management).

Tests the core StateTracker functionality independent of Document class.
Migrated from pytest to cclab.probe framework.
"""
import time

from cclab.nebula.state import StateTracker
from cclab.probe import test, expect
from tests.base import CommonTestSuite


class TestStateTrackerBasics(CommonTestSuite):
    """Basic StateTracker functionality tests."""

    @test(tags=["unit", "state-tracker"])
    async def test_init(self):
        """Test StateTracker initialization."""
        data = {"name": "Alice", "age": 30}
        tracker = StateTracker(data)

        expect(tracker._data).to_equal(data)  # Reference, not copy
        expect(len(tracker._original)).to_equal(0)
        expect(len(tracker._changed_fields)).to_equal(0)
        expect(tracker.is_modified()).to_be_false()

    @test(tags=["unit", "state-tracker"])
    async def test_track_change_single_field(self):
        """Test tracking a single field change."""
        data = {"name": "Alice", "age": 30}
        tracker = StateTracker(data)

        # Track change
        old_name = data["name"]
        tracker.track_change("name", old_name)
        data["name"] = "Bob"

        expect(tracker.is_modified()).to_be_true()
        expect(tracker.has_changed("name")).to_be_true()
        expect(tracker.has_changed("age")).to_be_false()
        expect(tracker.get_original_value("name")).to_equal("Alice")

    @test(tags=["unit", "state-tracker"])
    async def test_track_change_multiple_fields(self):
        """Test tracking multiple field changes."""
        data = {"name": "Alice", "age": 30, "email": "alice@example.com"}
        tracker = StateTracker(data)

        # Track multiple changes
        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        tracker.track_change("age", data["age"])
        data["age"] = 31

        expect(tracker.is_modified()).to_be_true()
        expect(tracker.has_changed("name")).to_be_true()
        expect(tracker.has_changed("age")).to_be_true()
        expect(tracker.has_changed("email")).to_be_false()
        expect(len(tracker._changed_fields)).to_equal(2)

    @test(tags=["unit", "state-tracker"])
    async def test_track_change_cow_behavior(self):
        """Test Copy-on-Write: only first change is stored."""
        data = {"name": "Alice"}
        tracker = StateTracker(data)

        # First change
        tracker.track_change("name", data["name"])
        data["name"] = "Bob"
        expect(tracker.get_original_value("name")).to_equal("Alice")

        # Second change to same field (should not update original)
        tracker.track_change("name", data["name"])  # This should be a no-op
        data["name"] = "Charlie"
        expect(tracker.get_original_value("name")).to_equal("Alice")  # Still Alice, not Bob


class TestStateTrackerQueries(CommonTestSuite):
    """Test StateTracker query methods."""

    @test(tags=["unit", "state-tracker"])
    async def test_is_modified_false_initially(self):
        """Test is_modified returns False initially."""
        data = {"name": "Alice"}
        tracker = StateTracker(data)
        expect(tracker.is_modified()).to_be_false()

    @test(tags=["unit", "state-tracker"])
    async def test_is_modified_true_after_change(self):
        """Test is_modified returns True after change."""
        data = {"name": "Alice"}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        expect(tracker.is_modified()).to_be_true()

    @test(tags=["unit", "state-tracker"])
    async def test_has_changed_specific_field(self):
        """Test has_changed for specific fields."""
        data = {"name": "Alice", "age": 30}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        expect(tracker.has_changed("name")).to_be_true()
        expect(tracker.has_changed("age")).to_be_false()
        expect(tracker.has_changed("nonexistent")).to_be_false()

    @test(tags=["unit", "state-tracker"])
    async def test_get_changes(self):
        """Test get_changes returns all changed fields."""
        data = {"name": "Alice", "age": 30, "email": "alice@example.com"}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        tracker.track_change("age", data["age"])
        data["age"] = 35

        changes = tracker.get_changes()
        expect(changes).to_equal({"name": "Bob", "age": 35})
        expect("email" in changes).to_be_false()

    @test(tags=["unit", "state-tracker"])
    async def test_get_changes_empty(self):
        """Test get_changes returns empty dict when no changes."""
        data = {"name": "Alice", "age": 30}
        tracker = StateTracker(data)

        changes = tracker.get_changes()
        expect(changes).to_equal({})

    @test(tags=["unit", "state-tracker"])
    async def test_compare_field(self):
        """Test compare_field detects value differences."""
        data = {"name": "Alice", "age": 30}
        tracker = StateTracker(data)

        # No change
        expect(tracker.compare_field("name")).to_be_false()

        # After change
        tracker.track_change("name", data["name"])
        data["name"] = "Bob"
        expect(tracker.compare_field("name")).to_be_true()
        expect(tracker.compare_field("age")).to_be_false()


class TestStateTrackerRollback(CommonTestSuite):
    """Test StateTracker rollback functionality."""

    @test(tags=["unit", "state-tracker"])
    async def test_rollback_single_field(self):
        """Test rollback restores original value."""
        data = {"name": "Alice", "age": 30}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        expect(data["name"]).to_equal("Bob")
        tracker.rollback()
        expect(data["name"]).to_equal("Alice")
        expect(tracker.is_modified()).to_be_false()

    @test(tags=["unit", "state-tracker"])
    async def test_rollback_multiple_fields(self):
        """Test rollback restores all original values."""
        data = {"name": "Alice", "age": 30, "email": "alice@example.com"}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        tracker.track_change("age", data["age"])
        data["age"] = 35

        tracker.rollback()

        expect(data["name"]).to_equal("Alice")
        expect(data["age"]).to_equal(30)
        expect(tracker.is_modified()).to_be_false()
        expect(len(tracker._changed_fields)).to_equal(0)

    @test(tags=["unit", "state-tracker"])
    async def test_rollback_clears_tracking(self):
        """Test rollback clears change tracking state."""
        data = {"name": "Alice"}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        tracker.rollback()

        expect(len(tracker._original)).to_equal(0)
        expect(len(tracker._changed_fields)).to_equal(0)


class TestStateTrackerReset(CommonTestSuite):
    """Test StateTracker reset functionality."""

    @test(tags=["unit", "state-tracker"])
    async def test_reset_accepts_current_state(self):
        """Test reset accepts current state as new baseline."""
        data = {"name": "Alice"}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        expect(tracker.is_modified()).to_be_true()
        tracker.reset()

        # State is now clean, but data is not rolled back
        expect(data["name"]).to_equal("Bob")
        expect(tracker.is_modified()).to_be_false()
        expect(len(tracker._changed_fields)).to_equal(0)

    @test(tags=["unit", "state-tracker"])
    async def test_reset_vs_rollback(self):
        """Test difference between reset and rollback."""
        data = {"name": "Alice"}
        tracker = StateTracker(data)

        # Make a change
        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        # Reset accepts the change
        tracker2 = StateTracker(data.copy())
        tracker2.track_change("name", "Alice")
        tracker2.reset()
        expect(data["name"]).to_equal("Bob")
        expect(tracker2.is_modified()).to_be_false()

        # Rollback reverts the change
        tracker3 = StateTracker(data)
        tracker3.track_change("name", data["name"])
        data["name"] = "Charlie"
        tracker3.rollback()
        expect(data["name"]).to_equal("Bob")  # Reverted


class TestStateTrackerOriginalData(CommonTestSuite):
    """Test get_all_original_data reconstruction."""

    @test(tags=["unit", "state-tracker"])
    async def test_get_all_original_data_no_changes(self):
        """Test get_all_original_data with no changes."""
        data = {"name": "Alice", "age": 30}
        tracker = StateTracker(data)

        original = tracker.get_all_original_data()
        expect(original).to_equal({"name": "Alice", "age": 30})

    @test(tags=["unit", "state-tracker"])
    async def test_get_all_original_data_with_changes(self):
        """Test get_all_original_data reconstructs original state."""
        data = {"name": "Alice", "age": 30, "email": "alice@example.com"}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        tracker.track_change("age", data["age"])
        data["age"] = 35

        original = tracker.get_all_original_data()

        # Should have original values for changed fields
        expect(original["name"]).to_equal("Alice")
        expect(original["age"]).to_equal(30)
        # Unchanged field should have current value
        expect(original["email"]).to_equal("alice@example.com")

    @test(tags=["unit", "state-tracker"])
    async def test_get_all_original_data_after_rollback(self):
        """Test get_all_original_data after rollback."""
        data = {"name": "Alice", "age": 30}
        tracker = StateTracker(data)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        tracker.rollback()

        original = tracker.get_all_original_data()
        expect(original).to_equal({"name": "Alice", "age": 30})


class TestStateTrackerEdgeCases(CommonTestSuite):
    """Test edge cases and special scenarios."""

    @test(tags=["unit", "state-tracker"])
    async def test_track_change_field_not_in_data(self):
        """Test tracking change for field that doesn't exist."""
        data = {"name": "Alice"}
        tracker = StateTracker(data)

        # Track change for nonexistent field
        tracker.track_change("age", None)
        data["age"] = 30

        expect(tracker.has_changed("age")).to_be_true()
        expect(tracker.get_original_value("age")).to_equal(None)
        expect(tracker.get_changes()).to_equal({"age": 30})

    @test(tags=["unit", "state-tracker"])
    async def test_nested_dict_not_deep_copied(self):
        """Test that nested dicts are not deep copied (COW only at top level)."""
        data = {"user": {"name": "Alice", "age": 30}}
        tracker = StateTracker(data)

        # Modify nested dict without tracking
        data["user"]["name"] = "Bob"

        # StateTracker only tracks top-level field changes
        expect(tracker.is_modified()).to_be_false()  # Top-level "user" key not changed

    @test(tags=["unit", "state-tracker"])
    async def test_list_field_changes(self):
        """Test tracking changes to list fields."""
        data = {"tags": ["python", "rust"]}
        tracker = StateTracker(data)

        old_tags = data["tags"]
        tracker.track_change("tags", old_tags)
        data["tags"] = ["python", "rust", "mongodb"]

        expect(tracker.has_changed("tags")).to_be_true()
        expect(tracker.get_original_value("tags")).to_equal(["python", "rust"])
        expect(tracker.get_changes()).to_equal({"tags": ["python", "rust", "mongodb"]})

    @test(tags=["unit", "state-tracker"])
    async def test_repr(self):
        """Test __repr__ for debugging."""
        data = {"name": "Alice"}
        tracker = StateTracker(data)

        repr_str = repr(tracker)
        expect("StateTracker" in repr_str).to_be_true()
        expect("modified=False" in repr_str).to_be_true()

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        repr_str = repr(tracker)
        expect("modified=True" in repr_str).to_be_true()
        expect("name" in repr_str).to_be_true()


class TestStateTrackerAPIEnhancements(CommonTestSuite):
    """Test API enhancements for parity with Rust."""

    @test(tags=["unit", "state-tracker"])
    async def test_change_count(self):
        """Test change_count returns number of changed fields."""
        data = {"name": "Alice", "age": 30, "email": "alice@example.com"}
        tracker = StateTracker(data)

        expect(tracker.change_count()).to_equal(0)

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"
        expect(tracker.change_count()).to_equal(1)

        tracker.track_change("age", data["age"])
        data["age"] = 31
        expect(tracker.change_count()).to_equal(2)

    @test(tags=["unit", "state-tracker"])
    async def test_changed_field_names(self):
        """Test changed_field_names returns list of changed fields."""
        data = {"name": "Alice", "age": 30, "email": "alice@example.com"}
        tracker = StateTracker(data)

        expect(tracker.changed_field_names()).to_equal([])

        tracker.track_change("name", data["name"])
        data["name"] = "Bob"

        tracker.track_change("age", data["age"])
        data["age"] = 31

        names = tracker.changed_field_names()
        expect(len(names)).to_equal(2)
        expect("name" in names).to_be_true()
        expect("age" in names).to_be_true()

    @test(tags=["unit", "state-tracker"])
    async def test_nested_field_tracking(self):
        """Test tracking nested field paths uses top-level field."""
        data = {"user": {"name": "Alice", "address": {"city": "Taipei"}}}
        tracker = StateTracker(data)

        # Track with dotted path - should track top-level 'user'
        tracker.track_change("user.address.city", data["user"])
        data["user"]["address"]["city"] = "Tokyo"

        expect(tracker.has_changed("user")).to_be_true()
        expect(tracker.has_changed("user.address")).to_be_true()
        expect(tracker.has_changed("user.address.city")).to_be_true()
        expect(tracker.change_count()).to_equal(1)  # Only top-level counted
        expect(tracker.changed_field_names()).to_equal(["user"])


class TestStateTrackerMemoryEfficiency(CommonTestSuite):
    """Test memory efficiency of StateTracker."""

    @test(tags=["unit", "state-tracker", "performance"])
    async def test_memory_efficiency(self):
        """Test that StateTracker only stores changed fields."""
        data = {f"field{i}": i for i in range(100)}  # 100 fields
        tracker = StateTracker(data)

        # Change only 2 fields
        tracker.track_change("field0", data["field0"])
        data["field0"] = 999

        tracker.track_change("field50", data["field50"])
        data["field50"] = 888

        # Only 2 fields should be in _original
        expect(len(tracker._original)).to_equal(2)
        expect(len(tracker._changed_fields)).to_equal(2)

        # Verify correctness
        expect(tracker.get_original_value("field0")).to_equal(0)
        expect(tracker.get_original_value("field50")).to_equal(50)
        changes = tracker.get_changes()
        expect(len(changes)).to_equal(2)
        expect(changes["field0"]).to_equal(999)
        expect(changes["field50"]).to_equal(888)


class TestStateTrackerPerformance(CommonTestSuite):
    """Basic performance sanity checks."""

    @test(tags=["unit", "state-tracker", "performance"])
    async def test_track_many_changes_fast(self):
        """Test tracking many changes is fast."""
        data = {f"field{i}": i for i in range(1000)}
        tracker = StateTracker(data)

        start = time.perf_counter()
        for key in data:
            tracker.track_change(key, data[key])
            data[key] = data[key] + 1
        elapsed = time.perf_counter() - start

        # Should be very fast (< 10ms for 1000 fields)
        expect(elapsed < 0.01).to_be_true()

        expect(len(tracker._changed_fields)).to_equal(1000)
        expect(tracker.is_modified()).to_be_true()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestStateTrackerBasics,
        TestStateTrackerQueries,
        TestStateTrackerRollback,
        TestStateTrackerReset,
        TestStateTrackerOriginalData,
        TestStateTrackerEdgeCases,
        TestStateTrackerAPIEnhancements,
        TestStateTrackerMemoryEfficiency,
        TestStateTrackerPerformance,
    ], verbose=True)
