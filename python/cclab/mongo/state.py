"""
Copy-on-Write state tracker for efficient change tracking.

Replaces expensive copy.deepcopy() with field-level change tracking.
Expected performance: 10x faster than deepcopy, 50% memory reduction.
"""
from typing import Any, Dict, List, Set


class StateTracker:
    """
    Copy-on-Write state tracker with field-level granularity.

    Instead of deep copying the entire document, only tracks which fields
    have changed and stores their original values on first write.

    Performance characteristics:
    - Memory: O(changed_fields) instead of O(all_fields)
    - Time: O(1) per field write instead of O(document_size)
    - Expected speedup: 10x faster than copy.deepcopy()

    Example:
        >>> data = {"name": "Alice", "age": 30, "email": "alice@example.com"}
        >>> tracker = StateTracker(data)
        >>> data["name"] = "Bob"  # Change field
        >>> tracker.track_change("name", "Alice")  # Track the change
        >>> tracker.is_modified()  # True
        >>> tracker.get_changes()  # {"name": "Bob"}
    """

    __slots__ = ('_data', '_original', '_changed_fields')

    def __init__(self, data: Dict[str, Any]):
        """
        Initialize state tracker.

        Args:
            data: Reference to the document's data dict (not copied)
        """
        self._data = data
        self._original: Dict[str, Any] = {}  # Only stores changed fields' original values
        self._changed_fields: Set[str] = set()

    def track_change(self, key: str, old_value: Any) -> None:
        """
        Track a field change (call before modifying the field).

        Only stores the original value on the first write to a field (COW).
        Subsequent writes to the same field do not update _original.
        For nested fields (e.g., "user.address.city"), tracks the top-level field.

        Args:
            key: Field name that will be changed
            old_value: Original value before change

        Example:
            >>> tracker = StateTracker({"name": "Alice"})
            >>> old_name = tracker._data["name"]
            >>> tracker.track_change("name", old_name)
            >>> tracker._data["name"] = "Bob"
        """
        # Extract top-level field name for nested paths
        top_level_field = key.split('.')[0] if '.' in key else key

        if top_level_field not in self._changed_fields:
            # Copy-on-Write: Only store original value on first write
            self._original[top_level_field] = old_value
            self._changed_fields.add(top_level_field)

    def is_modified(self) -> bool:
        """
        Check if any field has been modified.

        Returns:
            True if any field has changed

        Example:
            >>> tracker = StateTracker({"name": "Alice"})
            >>> tracker.is_modified()  # False
            >>> tracker.track_change("name", "Alice")
            >>> tracker.is_modified()  # True
        """
        return len(self._changed_fields) > 0

    def has_changed(self, field: str) -> bool:
        """
        Check if a specific field has been modified.

        For nested fields (e.g., "user.address.city"), checks the top-level field.

        Args:
            field: Field name to check

        Returns:
            True if the field has changed

        Example:
            >>> tracker = StateTracker({"name": "Alice", "age": 30})
            >>> tracker.track_change("name", "Alice")
            >>> tracker.has_changed("name")  # True
            >>> tracker.has_changed("age")   # False
        """
        # Check top-level field for nested paths
        top_level_field = field.split('.')[0] if '.' in field else field
        return top_level_field in self._changed_fields

    def get_changes(self) -> Dict[str, Any]:
        """
        Get all changed fields with their new values.

        Returns:
            Dict mapping field names to their current (new) values

        Example:
            >>> tracker = StateTracker({"name": "Alice", "age": 30})
            >>> tracker.track_change("name", "Alice")
            >>> tracker._data["name"] = "Bob"
            >>> tracker.get_changes()  # {"name": "Bob"}
        """
        return {key: self._data[key] for key in self._changed_fields if key in self._data}

    def get_original_value(self, field: str) -> Any:
        """
        Get the original value of a changed field.

        Args:
            field: Field name

        Returns:
            Original value, or None if field hasn't changed

        Example:
            >>> tracker = StateTracker({"name": "Alice"})
            >>> tracker.track_change("name", "Alice")
            >>> tracker._data["name"] = "Bob"
            >>> tracker.get_original_value("name")  # "Alice"
        """
        return self._original.get(field)

    def compare_field(self, field: str) -> bool:
        """
        Compare a field's current value to its original value.

        For nested fields (e.g., "user.address.city"), compares the top-level field.

        Args:
            field: Field name

        Returns:
            True if the field is tracked AND its current value differs from original

        Example:
            >>> data = {"name": "Alice"}
            >>> tracker = StateTracker(data)
            >>> tracker.track_change("name", "Alice")
            >>> data["name"] = "Bob"
            >>> tracker.compare_field("name")  # True (different)
        """
        top_level_field = field.split('.')[0] if '.' in field else field
        if top_level_field not in self._changed_fields:
            return False
        return self._data.get(top_level_field) != self._original.get(top_level_field)

    def rollback(self) -> None:
        """
        Rollback all changes to original values.

        Restores all changed fields to their original values.

        Example:
            >>> data = {"name": "Alice"}
            >>> tracker = StateTracker(data)
            >>> tracker.track_change("name", "Alice")
            >>> data["name"] = "Bob"
            >>> tracker.rollback()
            >>> data["name"]  # "Alice" (restored)
        """
        for key in self._changed_fields:
            if key in self._original:
                self._data[key] = self._original[key]
        self._changed_fields.clear()
        self._original.clear()

    def reset(self) -> None:
        """
        Clear all change tracking (mark current state as "clean").

        This does NOT rollback changes - it accepts current state as new baseline.

        Example:
            >>> tracker = StateTracker({"name": "Alice"})
            >>> tracker.track_change("name", "Alice")
            >>> tracker._data["name"] = "Bob"
            >>> tracker.reset()
            >>> tracker.is_modified()  # False (changes accepted)
        """
        self._changed_fields.clear()
        self._original.clear()

    def get_all_original_data(self) -> Dict[str, Any]:
        """
        Reconstruct the full original document state.

        Combines unchanged fields (from _data) with original values of changed fields.
        This is needed for compatibility with rollback() that expects full document.

        Returns:
            Dict representing the original state before any changes

        Example:
            >>> data = {"name": "Alice", "age": 30, "email": "alice@example.com"}
            >>> tracker = StateTracker(data)
            >>> tracker.track_change("name", "Alice")
            >>> data["name"] = "Bob"
            >>> tracker.get_all_original_data()
            # {"name": "Alice", "age": 30, "email": "alice@example.com"}
        """
        # Start with current data
        original = dict(self._data)
        # Replace changed fields with their original values
        for key in self._changed_fields:
            if key in self._original:
                original[key] = self._original[key]
        return original

    def change_count(self) -> int:
        """
        Get the number of changed fields.

        Returns:
            Number of fields that have been tracked as changed

        Example:
            >>> tracker = StateTracker({"name": "Alice", "age": 30})
            >>> tracker.change_count()  # 0
            >>> tracker.track_change("name", "Alice")
            >>> tracker.change_count()  # 1
        """
        return len(self._changed_fields)

    def changed_field_names(self) -> List[str]:
        """
        Get the names of all changed fields (sorted for deterministic ordering).

        Returns:
            Sorted list of field names that have been changed

        Example:
            >>> tracker = StateTracker({"name": "Alice", "age": 30})
            >>> tracker.track_change("name", "Alice")
            >>> tracker.track_change("age", 30)
            >>> tracker.changed_field_names()  # ["age", "name"]
        """
        return sorted(self._changed_fields)

    def __repr__(self) -> str:
        """String representation for debugging."""
        return (f"StateTracker(modified={self.is_modified()}, "
                f"change_count={self.change_count()}, "
                f"fields={self.changed_field_names()})")
