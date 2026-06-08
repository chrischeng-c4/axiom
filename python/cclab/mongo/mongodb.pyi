"""Type stubs for cclab.mongo module (Rust backend)"""

from typing import Any, Awaitable, Optional

def init(connection_string: str) -> Awaitable[None]:
    """
    Initialize MongoDB connection.

    Args:
        connection_string: MongoDB connection string (e.g., "mongodb://localhost:27017/mydb")

    Returns:
        Awaitable that completes when connection is established

    Example:
        >>> await init("mongodb://localhost:27017/mydb")
    """
    ...

def is_connected() -> bool:
    """
    Check if MongoDB connection is active.

    Returns:
        True if connected, False otherwise
    """
    ...

def available_features() -> list[str]:
    """
    Get list of available features in this build.

    Returns:
        List of feature names (e.g., ["mongodb", "redis"])
    """
    ...

class Document:
    """
    Low-level MongoDB Document class (Rust backend).

    For high-level Beanie-compatible API, use cclab.Document instead.
    """

    def __init__(self, collection_name: str, data: Optional[dict[str, Any]] = None) -> None: ...

    @property
    def id(self) -> Optional[str]: ...

    @property
    def collection(self) -> str: ...

    def to_dict(self) -> dict[str, Any]: ...
    def set(self, key: str, value: Any) -> None: ...
    def get(self, key: str) -> Optional[Any]: ...
    def save(self) -> Awaitable[str]: ...
    def delete(self) -> Awaitable[bool]: ...

    @staticmethod
    def find_one(
        collection_name: str,
        filter: Optional[dict[str, Any]] = None
    ) -> Awaitable[Optional["Document"]]: ...

    @staticmethod
    def find(
        collection_name: str,
        filter: Optional[dict[str, Any]] = None
    ) -> Awaitable[list["Document"]]: ...

    @staticmethod
    def find_with_options(
        collection_name: str,
        filter: Optional[dict[str, Any]] = None,
        sort: Optional[dict[str, int]] = None,
        skip: Optional[int] = None,
        limit: Optional[int] = None,
        projection: Optional[dict[str, int]] = None,
    ) -> Awaitable[list["Document"]]: ...

    @staticmethod
    def find_by_id(collection_name: str, id: str) -> Awaitable[Optional["Document"]]: ...

    @staticmethod
    def insert_many(
        collection_name: str,
        documents: list[dict[str, Any]]
    ) -> Awaitable[list[str]]: ...

    @staticmethod
    def update_one(
        collection_name: str,
        filter: dict[str, Any],
        update: dict[str, Any]
    ) -> Awaitable[int]: ...

    @staticmethod
    def update_many(
        collection_name: str,
        filter: dict[str, Any],
        update: dict[str, Any]
    ) -> Awaitable[int]: ...

    @staticmethod
    def delete_one(
        collection_name: str,
        filter: dict[str, Any]
    ) -> Awaitable[int]: ...

    @staticmethod
    def delete_many(
        collection_name: str,
        filter: dict[str, Any]
    ) -> Awaitable[int]: ...

    @staticmethod
    def count(
        collection_name: str,
        filter: Optional[dict[str, Any]] = None
    ) -> Awaitable[int]: ...

    @staticmethod
    def aggregate(
        collection_name: str,
        pipeline: list[dict[str, Any]]
    ) -> Awaitable[list["Document"]]: ...

__doc__: str
