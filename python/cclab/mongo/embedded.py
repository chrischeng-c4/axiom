"""
Embedded document support for data-bridge.

This module provides the EmbeddedDocument base class for creating embedded
documents that integrate with data-bridge's Rust backend.
"""

from typing import Any, Dict, ClassVar, get_type_hints


class EmbeddedDocumentMeta(type):
    """
    Metaclass for EmbeddedDocument classes.

    Collects field annotations and default values.
    """

    def __new__(mcs, name: str, bases: tuple, namespace: dict):
        cls = super().__new__(mcs, name, bases, namespace)

        # Collect field annotations from this class and all bases
        fields = {}
        field_defaults = {}

        # Get annotations from base classes first
        for base in reversed(bases):
            if hasattr(base, '__annotations__'):
                for field_name, field_type in base.__annotations__.items():
                    # Skip private fields (like _fields, _field_defaults)
                    if not field_name.startswith('_'):
                        fields[field_name] = field_type
            if hasattr(base, '_field_defaults'):
                field_defaults.update(base._field_defaults)

        # Add annotations from this class
        if '__annotations__' in namespace:
            for field_name, field_type in namespace['__annotations__'].items():
                # Skip private fields
                if not field_name.startswith('_'):
                    fields[field_name] = field_type

        # Collect default values from class attributes
        for field_name in fields:
            if field_name in namespace:
                field_defaults[field_name] = namespace[field_name]

        # Store on class
        cls._fields = fields
        cls._field_defaults = field_defaults

        return cls


class EmbeddedDocument(metaclass=EmbeddedDocumentMeta):
    """
    Base class for embedded documents in data-bridge.

    Pure Python class - all validation and BSON serialization happens in Rust.

    Example:
        ```python
        from cclab.mongo import Document, EmbeddedDocument

        class Address(EmbeddedDocument):
            city: str
            zip_code: str
            street: str | None = None

        class User(Document):
            name: str
            email: str
            address: Address

            class Settings:
                name = "users"

        # Usage
        user = User(
            name="Alice",
            email="alice@example.com",
            address=Address(city="NYC", zip_code="10001")
        )
        await user.save()
        ```
    """

    # Class-level attributes (set by metaclass)
    _fields: ClassVar[Dict[str, type]] = {}
    _field_defaults: ClassVar[Dict[str, Any]] = {}

    def __init__(self, **kwargs: Any) -> None:
        """
        Initialize an embedded document instance.

        Args:
            **kwargs: Field values for the embedded document
        """
        # Store all provided values
        for key, value in kwargs.items():
            setattr(self, key, value)

        # Set default values for fields not provided
        for field_name, field_type in self._fields.items():
            if not hasattr(self, field_name):
                # Check for stored default value
                if field_name in self._field_defaults:
                    default = self._field_defaults[field_name]
                    # Check if it's a callable (factory function)
                    if callable(default):
                        setattr(self, field_name, default())
                    else:
                        setattr(self, field_name, default)
                # No default - leave unset (will fail validation in Rust if required)

    def to_dict(self) -> Dict[str, Any]:
        """
        Convert to dict for MongoDB storage.

        Returns:
            Dict representation suitable for MongoDB storage
        """
        result = {}
        for field_name in self._fields:
            if hasattr(self, field_name):
                value = getattr(self, field_name)
                # Handle nested EmbeddedDocument
                if isinstance(value, EmbeddedDocument):
                    result[field_name] = value.to_dict()
                # Handle list of EmbeddedDocument
                elif isinstance(value, list):
                    result[field_name] = [
                        item.to_dict() if isinstance(item, EmbeddedDocument) else item
                        for item in value
                    ]
                else:
                    result[field_name] = value
        return result

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "EmbeddedDocument":
        """
        Create instance from dict (from MongoDB).

        Args:
            data: Dictionary data from MongoDB

        Returns:
            EmbeddedDocument instance
        """
        # Get type hints to know which fields are EmbeddedDocument
        try:
            hints = get_type_hints(cls)
        except Exception:
            hints = getattr(cls, '__annotations__', {})

        # Process nested embedded documents
        processed_data = {}
        for key, value in data.items():
            if key in hints:
                field_type = hints[key]
                # Check if it's an EmbeddedDocument type
                if isinstance(value, dict) and isinstance(field_type, type) and issubclass(field_type, EmbeddedDocument):
                    processed_data[key] = field_type.from_dict(value)
                # Check if it's a list of EmbeddedDocument
                elif isinstance(value, list) and hasattr(field_type, '__origin__'):
                    # Handle List[EmbeddedDocument]
                    from typing import get_origin, get_args
                    if get_origin(field_type) is list:
                        args = get_args(field_type)
                        if args and isinstance(args[0], type) and issubclass(args[0], EmbeddedDocument):
                            processed_data[key] = [
                                args[0].from_dict(item) if isinstance(item, dict) else item
                                for item in value
                            ]
                        else:
                            processed_data[key] = value
                    else:
                        processed_data[key] = value
                else:
                    processed_data[key] = value
            else:
                processed_data[key] = value

        return cls(**processed_data)

    def __repr__(self) -> str:
        """String representation."""
        fields = ", ".join(f"{k}={getattr(self, k)!r}" for k in self._fields if hasattr(self, k))
        return f"{self.__class__.__name__}({fields})"

    def __eq__(self, other: Any) -> bool:
        """Equality comparison."""
        if not isinstance(other, self.__class__):
            return False
        return all(
            getattr(self, field, None) == getattr(other, field, None)
            for field in self._fields
        )
