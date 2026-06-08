"""
Bulk write operations with fluent chainable API.

This module provides type-safe bulk operations that go through the Rust backend
for maximum performance. All BSON serialization happens in Rust.

Example:
    >>> from cclab.mongo import UpdateOne, UpdateMany, InsertOne, DeleteOne, DeleteMany
    >>>
    >>> # Fluent chainable API
    >>> await User.bulk_write([
    ...     UpdateOne(User.status == "pending")
    ...         .set(User.status, "active")
    ...         .set(User.updated_at, datetime.now()),
    ...
    ...     UpdateOne(User.id == user_id)
    ...         .inc(User.login_count, 1)
    ...         .push(User.tags, "vip"),
    ...
    ...     UpdateMany(User.expired == True)
    ...         .set(User.archived, True)
    ...         .unset(User.temp_data),
    ...
    ...     InsertOne(User(name="Alice", email="alice@example.com")),
    ...
    ...     DeleteOne(User.id == old_id),
    ...     DeleteMany(User.status == "deleted"),
    ... ])
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional, TYPE_CHECKING, Union

if TYPE_CHECKING:
    from .document import Document
    from .fields import FieldProxy, QueryExpr


class BulkOperation:
    """Base class for bulk operations."""

    def to_dict(self) -> Dict[str, Any]:
        """Convert operation to dict for Rust backend."""
        raise NotImplementedError


class UpdateOne(BulkOperation):
    """
    Update a single document matching the filter.

    Supports fluent chainable API for building update operations.

    Example:
        >>> UpdateOne(User.status == "pending")
        ...     .set(User.status, "active")
        ...     .inc(User.score, 10)
        ...     .push(User.tags, "processed")
    """

    def __init__(
        self,
        filter: Union["QueryExpr", Dict[str, Any]],
        upsert: bool = False,
    ) -> None:
        """
        Initialize UpdateOne operation.

        Args:
            filter: Query filter (QueryExpr or dict)
            upsert: If True, insert if no match found
        """
        self._filter = filter
        self._upsert = upsert
        self._updates: Dict[str, Dict[str, Any]] = {}
        self._array_filters: Optional[List[Dict]] = None

    def set(self, field: Union["FieldProxy", str], value: Any) -> "UpdateOne":
        """
        Set a field to a value.

        Args:
            field: Field to set (FieldProxy or string)
            value: Value to set

        Example:
            >>> UpdateOne(User.id == id).set(User.name, "Alice")
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$set", {})[field_name] = value
        return self

    def unset(self, field: Union["FieldProxy", str]) -> "UpdateOne":
        """
        Remove a field from the document.

        Args:
            field: Field to remove

        Example:
            >>> UpdateOne(User.id == id).unset(User.temp_data)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$unset", {})[field_name] = ""
        return self

    def inc(self, field: Union["FieldProxy", str], value: Union[int, float]) -> "UpdateOne":
        """
        Increment a field by a value.

        Args:
            field: Field to increment
            value: Amount to increment (can be negative)

        Example:
            >>> UpdateOne(User.id == id).inc(User.login_count, 1)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$inc", {})[field_name] = value
        return self

    def mul(self, field: Union["FieldProxy", str], value: Union[int, float]) -> "UpdateOne":
        """
        Multiply a field by a value.

        Args:
            field: Field to multiply
            value: Multiplier

        Example:
            >>> UpdateOne(User.id == id).mul(User.score, 1.5)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$mul", {})[field_name] = value
        return self

    def min(self, field: Union["FieldProxy", str], value: Any) -> "UpdateOne":
        """
        Set field to value if value is less than current.

        Args:
            field: Field to update
            value: Minimum value

        Example:
            >>> UpdateOne(User.id == id).min(User.low_score, 50)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$min", {})[field_name] = value
        return self

    def max(self, field: Union["FieldProxy", str], value: Any) -> "UpdateOne":
        """
        Set field to value if value is greater than current.

        Args:
            field: Field to update
            value: Maximum value

        Example:
            >>> UpdateOne(User.id == id).max(User.high_score, 100)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$max", {})[field_name] = value
        return self

    def push(self, field: Union["FieldProxy", str], value: Any) -> "UpdateOne":
        """
        Push a value to an array field.

        Args:
            field: Array field
            value: Value to push

        Example:
            >>> UpdateOne(User.id == id).push(User.tags, "vip")
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$push", {})[field_name] = value
        return self

    def push_all(self, field: Union["FieldProxy", str], values: List[Any]) -> "UpdateOne":
        """
        Push multiple values to an array field.

        Args:
            field: Array field
            values: Values to push

        Example:
            >>> UpdateOne(User.id == id).push_all(User.tags, ["vip", "premium"])
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$push", {})[field_name] = {"$each": values}
        return self

    def pull(self, field: Union["FieldProxy", str], value: Any) -> "UpdateOne":
        """
        Remove a value from an array field.

        Args:
            field: Array field
            value: Value to remove

        Example:
            >>> UpdateOne(User.id == id).pull(User.tags, "inactive")
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$pull", {})[field_name] = value
        return self

    def pull_all(self, field: Union["FieldProxy", str], values: List[Any]) -> "UpdateOne":
        """
        Remove multiple values from an array field.

        Args:
            field: Array field
            values: Values to remove

        Example:
            >>> UpdateOne(User.id == id).pull_all(User.tags, ["old", "expired"])
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$pullAll", {})[field_name] = values
        return self

    def add_to_set(self, field: Union["FieldProxy", str], value: Any) -> "UpdateOne":
        """
        Add a value to an array only if it doesn't exist.

        Args:
            field: Array field
            value: Value to add

        Example:
            >>> UpdateOne(User.id == id).add_to_set(User.tags, "unique")
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$addToSet", {})[field_name] = value
        return self

    def pop_first(self, field: Union["FieldProxy", str]) -> "UpdateOne":
        """
        Remove the first element from an array.

        Args:
            field: Array field

        Example:
            >>> UpdateOne(User.id == id).pop_first(User.queue)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$pop", {})[field_name] = -1
        return self

    def pop_last(self, field: Union["FieldProxy", str]) -> "UpdateOne":
        """
        Remove the last element from an array.

        Args:
            field: Array field

        Example:
            >>> UpdateOne(User.id == id).pop_last(User.queue)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$pop", {})[field_name] = 1
        return self

    def rename(self, field: Union["FieldProxy", str], new_name: str) -> "UpdateOne":
        """
        Rename a field.

        Args:
            field: Field to rename
            new_name: New field name

        Example:
            >>> UpdateOne(User.id == id).rename(User.old_field, "new_field")
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        self._updates.setdefault("$rename", {})[field_name] = new_name
        return self

    def current_date(self, field: Union["FieldProxy", str], as_timestamp: bool = False) -> "UpdateOne":
        """
        Set field to current date/timestamp.

        Args:
            field: Field to set
            as_timestamp: If True, use timestamp type instead of date

        Example:
            >>> UpdateOne(User.id == id).current_date(User.updated_at)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        if as_timestamp:
            self._updates.setdefault("$currentDate", {})[field_name] = {"$type": "timestamp"}
        else:
            self._updates.setdefault("$currentDate", {})[field_name] = True
        return self

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dict for Rust backend."""
        from .fields import QueryExpr

        # Build filter
        if isinstance(self._filter, QueryExpr):
            filter_doc = self._filter.to_filter()
        else:
            filter_doc = self._filter

        result = {
            "op": "update_one",
            "filter": filter_doc,
            "update": self._updates,
            "upsert": self._upsert,
        }

        if self._array_filters:
            result["array_filters"] = self._array_filters

        return result


class UpdateMany(UpdateOne):
    """
    Update multiple documents matching the filter.

    Same fluent API as UpdateOne, but updates all matching documents.

    Example:
        >>> UpdateMany(User.status == "pending")
        ...     .set(User.status, "processed")
        ...     .current_date(User.processed_at)
    """

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dict for Rust backend."""
        result = super().to_dict()
        result["op"] = "update_many"
        return result


class InsertOne(BulkOperation):
    """
    Insert a single document.

    Example:
        >>> InsertOne(User(name="Alice", email="alice@example.com"))
        >>> InsertOne({"name": "Bob", "email": "bob@example.com"})
    """

    def __init__(self, document: Union["Document", Dict[str, Any]]) -> None:
        """
        Initialize InsertOne operation.

        Args:
            document: Document instance or dict to insert
        """
        self._document = document

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dict for Rust backend."""
        if hasattr(self._document, "to_dict"):
            doc = self._document.to_dict()
            doc.pop("_id", None)  # Remove _id for insert
        else:
            doc = dict(self._document)
            doc.pop("_id", None)

        return {
            "op": "insert_one",
            "document": doc,
        }


class DeleteOne(BulkOperation):
    """
    Delete a single document matching the filter.

    Example:
        >>> DeleteOne(User.id == user_id)
        >>> DeleteOne(User.status == "deleted")
    """

    def __init__(self, filter: Union["QueryExpr", Dict[str, Any]]) -> None:
        """
        Initialize DeleteOne operation.

        Args:
            filter: Query filter
        """
        self._filter = filter

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dict for Rust backend."""
        from .fields import QueryExpr

        if isinstance(self._filter, QueryExpr):
            filter_doc = self._filter.to_filter()
        else:
            filter_doc = self._filter

        return {
            "op": "delete_one",
            "filter": filter_doc,
        }


class DeleteMany(BulkOperation):
    """
    Delete multiple documents matching the filter.

    Example:
        >>> DeleteMany(User.status == "deleted")
        >>> DeleteMany(User.expired == True)
    """

    def __init__(self, filter: Union["QueryExpr", Dict[str, Any]]) -> None:
        """
        Initialize DeleteMany operation.

        Args:
            filter: Query filter
        """
        self._filter = filter

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dict for Rust backend."""
        from .fields import QueryExpr

        if isinstance(self._filter, QueryExpr):
            filter_doc = self._filter.to_filter()
        else:
            filter_doc = self._filter

        return {
            "op": "delete_many",
            "filter": filter_doc,
        }


class ReplaceOne(BulkOperation):
    """
    Replace a single document matching the filter.

    Example:
        >>> ReplaceOne(
        ...     User.id == user_id,
        ...     User(name="New Name", email="new@example.com")
        ... )
    """

    def __init__(
        self,
        filter: Union["QueryExpr", Dict[str, Any]],
        replacement: Union["Document", Dict[str, Any]],
        upsert: bool = False,
    ) -> None:
        """
        Initialize ReplaceOne operation.

        Args:
            filter: Query filter
            replacement: Replacement document
            upsert: If True, insert if no match found
        """
        self._filter = filter
        self._replacement = replacement
        self._upsert = upsert

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dict for Rust backend."""
        from .fields import QueryExpr

        if isinstance(self._filter, QueryExpr):
            filter_doc = self._filter.to_filter()
        else:
            filter_doc = self._filter

        if hasattr(self._replacement, "to_dict"):
            replacement_doc = self._replacement.to_dict()
            replacement_doc.pop("_id", None)
        else:
            replacement_doc = dict(self._replacement)
            replacement_doc.pop("_id", None)

        return {
            "op": "replace_one",
            "filter": filter_doc,
            "replacement": replacement_doc,
            "upsert": self._upsert,
        }


class BulkWriteResult:
    """
    Result of a bulk write operation.

    Attributes:
        inserted_count: Number of inserted documents
        matched_count: Number of matched documents
        modified_count: Number of modified documents
        deleted_count: Number of deleted documents
        upserted_count: Number of upserted documents
        upserted_ids: Dict mapping index to upserted _id
    """

    def __init__(
        self,
        inserted_count: int = 0,
        matched_count: int = 0,
        modified_count: int = 0,
        deleted_count: int = 0,
        upserted_count: int = 0,
        upserted_ids: Optional[Dict[int, str]] = None,
    ) -> None:
        self.inserted_count = inserted_count
        self.matched_count = matched_count
        self.modified_count = modified_count
        self.deleted_count = deleted_count
        self.upserted_count = upserted_count
        self.upserted_ids = upserted_ids or {}

    def __repr__(self) -> str:
        return (
            f"BulkWriteResult("
            f"inserted={self.inserted_count}, "
            f"matched={self.matched_count}, "
            f"modified={self.modified_count}, "
            f"deleted={self.deleted_count}, "
            f"upserted={self.upserted_count})"
        )


__all__ = [
    "BulkOperation",
    "UpdateOne",
    "UpdateMany",
    "InsertOne",
    "DeleteOne",
    "DeleteMany",
    "ReplaceOne",
    "BulkWriteResult",
]
