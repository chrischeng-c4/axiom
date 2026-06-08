"""
Type support for data-bridge.

This module provides Beanie-compatible types:
- PydanticObjectId: A Pydantic-compatible MongoDB ObjectId
- Indexed: Type annotation wrapper for declaring indexes

Example:
    >>> from cclab.mongo import Document, PydanticObjectId, Indexed
    >>>
    >>> class User(Document):
    ...     id: PydanticObjectId
    ...     email: Indexed(str, unique=True)
    ...     username: Indexed(str)
    ...     location: Indexed(str, index_type="2dsphere")
"""

from __future__ import annotations

from typing import Any, Annotated, Optional, Type, TypeVar, get_args, get_origin
from ..cclab import ObjectId

# For Pydantic v2 compatibility
try:
    from pydantic import GetCoreSchemaHandler
    from pydantic_core import CoreSchema, core_schema
    PYDANTIC_V2 = True
except ImportError:
    PYDANTIC_V2 = False


T = TypeVar("T")


class PydanticObjectId(str):
    """
    A Pydantic-compatible MongoDB ObjectId.

    This class can be used as a type annotation in Pydantic models and
    Document classes. It serializes to a hex string for JSON compatibility
    while maintaining ObjectId semantics.

    Example:
        >>> from cclab.mongo import Document, PydanticObjectId
        >>>
        >>> class User(Document):
        ...     id: PydanticObjectId
        ...     referred_by: Optional[PydanticObjectId] = None
        >>>
        >>> user = User(email="alice@example.com")
        >>> print(user.id)  # Auto-generated hex string
    """

    @classmethod
    def __get_pydantic_core_schema__(
        cls, source_type: Any, handler: GetCoreSchemaHandler
    ) -> CoreSchema:
        """Define Pydantic v2 schema for validation and serialization."""
        return core_schema.union_schema(
            [
                # Accept ObjectId directly
                core_schema.is_instance_schema(ObjectId, cls=ObjectId),
                # Accept PydanticObjectId directly
                core_schema.is_instance_schema(cls),
                # Accept string (hex format)
                core_schema.str_schema(),
            ],
            serialization=core_schema.plain_serializer_function_ser_schema(
                lambda x: str(x) if x else None,
                info_arg=False,
                return_schema=core_schema.str_schema(),
            ),
        )

    def __new__(cls, value: Any = None) -> "PydanticObjectId":
        """
        Create a new PydanticObjectId.

        Args:
            value: Can be:
                - None: Generate a new ObjectId
                - str: Parse as hex string
                - ObjectId: Use directly
                - PydanticObjectId: Use directly

        Returns:
            PydanticObjectId instance (a string subclass)
        """
        if value is None:
            # Generate new ObjectId
            oid = ObjectId.new()
            return str.__new__(cls, str(oid))
        elif isinstance(value, ObjectId):
            return str.__new__(cls, str(value))
        elif isinstance(value, PydanticObjectId):
            return str.__new__(cls, str(value))
        elif isinstance(value, str):
            # Validate it's a valid ObjectId hex string
            if ObjectId.is_valid(value):
                return str.__new__(cls, value)
            else:
                raise ValueError(f"Invalid ObjectId: {value}")
        else:
            raise TypeError(f"Cannot convert {type(value).__name__} to PydanticObjectId")

    def __repr__(self) -> str:
        return f"PydanticObjectId('{self}')"

    def __eq__(self, other: object) -> bool:
        if isinstance(other, (str, ObjectId, PydanticObjectId)):
            return str(self) == str(other)
        return False

    def __hash__(self) -> int:
        return hash(str(self))

    def to_object_id(self) -> ObjectId:
        """Convert to cclab.ObjectId."""
        return ObjectId(str(self))

    @classmethod
    def validate(cls, value: Any) -> "PydanticObjectId":
        """Validate and convert a value to PydanticObjectId."""
        return cls(value)

    @classmethod
    def is_valid(cls, value: Any) -> bool:
        """Check if a value can be converted to PydanticObjectId."""
        if value is None:
            return True
        if isinstance(value, (ObjectId, PydanticObjectId)):
            return True
        if isinstance(value, str):
            return ObjectId.is_valid(value)
        return False


class IndexModelField:
    """
    Metadata for an indexed field.

    This class stores index configuration that can be attached to
    type annotations using typing.Annotated.
    """

    def __init__(
        self,
        *,
        unique: bool = False,
        sparse: bool = False,
        index_type: str = "ascending",
        name: Optional[str] = None,
        background: bool = False,
        expire_after_seconds: Optional[int] = None,
    ) -> None:
        """
        Initialize index metadata.

        Args:
            unique: If True, create a unique index
            sparse: If True, only index documents with the field
            index_type: Type of index:
                - "ascending" (default) or 1
                - "descending" or -1
                - "text" for text search
                - "2dsphere" for geospatial
                - "hashed" for hash-based sharding
            name: Optional custom index name
            background: Build index in background (deprecated in MongoDB 4.2+)
            expire_after_seconds: TTL in seconds for TTL indexes
        """
        self.unique = unique
        self.sparse = sparse
        self.index_type = index_type
        self.name = name
        self.background = background
        self.expire_after_seconds = expire_after_seconds

    def to_index_spec(self, field_name: str) -> tuple:
        """
        Convert to MongoDB index specification.

        Returns:
            Tuple of (keys, options) for create_index
        """
        # Convert index_type to MongoDB format
        type_map = {
            "ascending": 1,
            "descending": -1,
            "text": "text",
            "2dsphere": "2dsphere",
            "hashed": "hashed",
            1: 1,
            -1: -1,
        }
        index_direction = type_map.get(self.index_type, 1)

        keys = [(field_name, index_direction)]

        options = {}
        if self.unique:
            options["unique"] = True
        if self.sparse:
            options["sparse"] = True
        if self.name:
            options["name"] = self.name
        if self.background:
            options["background"] = True
        if self.expire_after_seconds is not None:
            options["expireAfterSeconds"] = self.expire_after_seconds

        return keys, options

    def __repr__(self) -> str:
        attrs = []
        if self.unique:
            attrs.append("unique=True")
        if self.sparse:
            attrs.append("sparse=True")
        if self.index_type != "ascending":
            attrs.append(f"index_type='{self.index_type}'")
        if self.name:
            attrs.append(f"name='{self.name}'")
        if self.expire_after_seconds is not None:
            attrs.append(f"expire_after_seconds={self.expire_after_seconds}")
        return f"IndexModelField({', '.join(attrs)})"


def Indexed(
    type_: Type[T],
    *,
    unique: bool = False,
    sparse: bool = False,
    index_type: str = "ascending",
    name: Optional[str] = None,
    background: bool = False,
    expire_after_seconds: Optional[int] = None,
) -> Type[T]:
    """
    Mark a field as indexed.

    This function returns an Annotated type that includes index metadata.
    When the Document class is initialized, these indexes can be
    automatically created.

    Args:
        type_: The field type (e.g., str, int)
        unique: If True, create a unique index
        sparse: If True, only index documents with the field
        index_type: Type of index:
            - "ascending" (default) or 1
            - "descending" or -1
            - "text" for text search
            - "2dsphere" for geospatial
            - "hashed" for hash-based sharding
        name: Optional custom index name
        background: Build index in background (deprecated in MongoDB 4.2+)
        expire_after_seconds: TTL in seconds (for datetime fields)

    Returns:
        Annotated type with index metadata

    Example:
        >>> class User(Document):
        ...     email: Indexed(str, unique=True)
        ...     username: Indexed(str)
        ...     created_at: Indexed(datetime, expire_after_seconds=86400)
        ...     location: Indexed(str, index_type="2dsphere")
    """
    index_model = IndexModelField(
        unique=unique,
        sparse=sparse,
        index_type=index_type,
        name=name,
        background=background,
        expire_after_seconds=expire_after_seconds,
    )
    return Annotated[type_, index_model]


def get_index_fields(cls: type) -> dict[str, IndexModelField]:
    """
    Extract indexed fields from a class's type annotations.

    Args:
        cls: A class with type annotations

    Returns:
        Dict mapping field name to IndexModelField
    """
    indexed_fields = {}

    # Get annotations from the class and its bases
    annotations = {}
    for base in reversed(cls.__mro__):
        if hasattr(base, "__annotations__"):
            annotations.update(base.__annotations__)

    for field_name, annotation in annotations.items():
        # Check if it's an Annotated type
        if get_origin(annotation) is Annotated:
            args = get_args(annotation)
            for arg in args[1:]:  # First arg is the actual type
                if isinstance(arg, IndexModelField):
                    indexed_fields[field_name] = arg
                    break

    return indexed_fields


__all__ = [
    "PydanticObjectId",
    "Indexed",
    "IndexModelField",
    "get_index_fields",
]
