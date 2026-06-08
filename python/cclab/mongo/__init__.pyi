from typing import Any

__all__ = ["QueryExpr", "QueryBuilder", "LinkField", "StateTracker", "Document", "fetch_links_batched", "extract_object_id", "init", "is_connected", "close", "reset", "available_features"]

class QueryExpr:
    """Python wrapper for Rust QueryExpr"""
    @staticmethod
    def eq(field: str, value: Any) -> Any:
        """Create an equality expression: field == value"""
        ...
    @staticmethod
    def ne(field: str, value: Any) -> Any:
        """Create a not-equal expression: field != value"""
        ...
    @staticmethod
    def gt(field: str, value: Any) -> Any:
        """Create a greater-than expression: field > value"""
        ...
    @staticmethod
    def gte(field: str, value: Any) -> Any:
        """Create a greater-than-or-equal expression: field >= value"""
        ...
    @staticmethod
    def lt(field: str, value: Any) -> Any:
        """Create a less-than expression: field < value"""
        ...
    @staticmethod
    def lte(field: str, value: Any) -> Any:
        """Create a less-than-or-equal expression: field <= value"""
        ...
    @staticmethod
    def in_(field: str, values: list[Any]) -> Any:
        """Create an $in expression: field in [values]"""
        ...
    @staticmethod
    def nin(field: str, values: list[Any]) -> Any:
        """Create a $nin expression: field not in [values]"""
        ...
    @staticmethod
    def exists(field: str, exists: bool) -> Any:
        """Create an $exists expression"""
        ...
    @staticmethod
    def regex(field: str, pattern: str) -> Any:
        """Create a $regex expression"""
        ...
    @staticmethod
    def and_(exprs: list[Any]) -> Any:
        """Create a logical AND expression"""
        ...
    @staticmethod
    def or_(exprs: list[Any]) -> Any:
        """Create a logical OR expression"""
        ...
    @staticmethod
    def raw(doc: dict[Any, Any]) -> Any:
        """Create a raw BSON expression from a Python dict"""
        ...
    def __repr__(self) -> str:
        ...

class QueryBuilder:
    """Python wrapper for Rust QueryBuilder"""
    def __init__(self, collection_name: str) -> None:
        """Create a new QueryBuilder for the given collection"""
        ...
    @property
    def collection(self) -> str:
        """Get the collection name"""
        ...
    def filter(self, expr: Any) -> Any:
        """Add a filter expression. Returns a new QueryBuilder."""
        ...
    def filter_raw(self, doc: dict[Any, Any]) -> Any:
        """Add a raw dict filter. Returns a new QueryBuilder."""
        ...
    def sort(self, fields: list[Any]) -> Any:
        """Set sort order. Returns a new QueryBuilder."""
        ...
    def skip(self, n: int) -> Any:
        """Set skip count. Returns a new QueryBuilder."""
        ...
    def limit(self, n: int) -> Any:
        """Set limit count. Returns a new QueryBuilder."""
        ...
    def projection(self, fields: list[str]) -> Any:
        """Set projection fields. Returns a new QueryBuilder."""
        ...
    def to_list(self) -> Any:
        """Execute the query and return all matching documents"""
        ...
    def count(self) -> Any:
        """Count matching documents"""
        ...
    def first(self) -> Any:
        """Execute the query and return the first matching document"""
        ...
    def __repr__(self) -> str:
        ...

class LinkField:
    """Python wrapper for Rust LinkField."""
    def __init__(self, field_name: str, link_type: Any, target_collection: str, is_list: bool = False) -> None:
        """Create a new LinkField."""
        ...
    @staticmethod
    def link(field_name: str, target_collection: str) -> Any:
        """Create a single forward link field."""
        ...
    @staticmethod
    def link_list(field_name: str, target_collection: str) -> Any:
        """Create a list forward link field."""
        ...
    @staticmethod
    def back_link(field_name: str, target_collection: str) -> Any:
        """Create a back link field."""
        ...
    @property
    def field_name(self) -> str:
        ...
    @property
    def link_type(self) -> Any:
        ...
    @property
    def target_collection(self) -> str:
        ...
    @property
    def is_list(self) -> bool:
        ...
    def __repr__(self) -> str:
        ...

class StateTracker:
    """Python wrapper for Rust StateTracker.

Tracks changes to document fields using Copy-On-Write semantics."""
    def __init__(self) -> None:
        """Create a new empty StateTracker."""
        ...
    def track_change(self, field: str, original_value: Any) -> None:
        """Track a field change by storing the original value (COW)."""
        ...
    def is_modified(self) -> bool:
        """Check if any field has been modified."""
        ...
    def has_changed(self, field: str) -> bool:
        """Check if a specific field has been modified."""
        ...
    def changed_field_names(self) -> list[str]:
        """Get the names of all changed fields."""
        ...
    def get_changes(self, current_data: dict[Any, Any]) -> dict[Any, Any]:
        """Get changes as a dict containing only the modified fields."""
        ...
    def rollback(self, document: dict[Any, Any]) -> dict[Any, Any]:
        """Rollback all changes by restoring original values."""
        ...
    def reset(self) -> None:
        """Reset the tracker, clearing all change tracking state."""
        ...
    def get_original(self, field: str) -> Any | None:
        """Get the original value for a specific field (if tracked)."""
        ...
    def get_all_original_data(self, current_data: dict[Any, Any]) -> dict[Any, Any]:
        """Reconstruct the full original document state."""
        ...
    def change_count(self) -> int:
        """Get the number of changed fields."""
        ...
    def __repr__(self) -> str:
        ...

class Document:
    """MongoDB Document class for Python"""
    def __init__(self, collection_name: str, data: dict[Any, Any] | None = None) -> None:
        """Create a new document"""
        ...
    @property
    def id(self) -> str | None:
        """Get the document's ObjectId as a hex string"""
        ...
    @property
    def collection(self) -> str:
        """Get the collection name"""
        ...
    def to_dict(self) -> Any:
        """Get document data as a Python dict"""
        ...
    def set(self, key: str, value: Any) -> None:
        """Set a field value"""
        ...
    def get(self, key: str) -> Any | None:
        """Get a field value"""
        ...
    def save(self) -> Any:
        """Save the document to MongoDB (insert or update)"""
        ...
    def delete(self) -> Any:
        """Delete this document from MongoDB"""
        ...
    @staticmethod
    def find_one(collection_name: str, filter: dict[Any, Any] | None = None) -> Any:
        """Find a single document matching the filter"""
        ...
    @staticmethod
    def find(collection_name: str, filter: dict[Any, Any] | None = None) -> Any:
        """Find all documents matching the filter"""
        ...
    @staticmethod
    def find_by_id(collection_name: str, id: str) -> Any:
        """Find a document by its ObjectId"""
        ...
    @staticmethod
    def update_one(collection_name: str, filter: dict[Any, Any], update: dict[Any, Any]) -> Any:
        """Update documents matching the filter"""
        ...
    @staticmethod
    def delete_many(collection_name: str, filter: dict[Any, Any]) -> Any:
        """Delete documents matching the filter"""
        ...
    @staticmethod
    def count(collection_name: str, filter: dict[Any, Any] | None = None) -> Any:
        """Count documents matching the filter"""
        ...
    @staticmethod
    def insert_many(collection_name: str, documents: list[Any]) -> Any:
        """Insert multiple documents"""
        ...
    @staticmethod
    def aggregate(collection_name: str, pipeline: list[Any]) -> Any:
        """Run an aggregation pipeline"""
        ...
    def __repr__(self) -> str:
        ...

def fetch_links_batched(docs: list[Any], link_fields: list[Any], max_depth: int = 1) -> Any:
    """Fetch linked documents in batches."""
    ...

def extract_object_id(value: Any) -> str | None:
    """Extract object IDs from a document field."""
    ...

def init(connection_string: str) -> Any:
    """Initialize MongoDB connection"""
    ...

def is_connected() -> bool:
    """Get connection status"""
    ...

def close() -> Any:
    """Close the MongoDB connection"""
    ...

def reset() -> None:
    """Reset the connection (synchronous, for testing)"""
    ...

def available_features() -> list[str]:
    """Available features in this build"""
    ...

