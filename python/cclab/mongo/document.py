"""
Document base class with metaclass for Beanie-compatible API.

This module provides:
- DocumentMeta: Metaclass that creates FieldProxy attributes for query syntax
- Document: Base class for all MongoDB documents with CRUD operations

Example:
    >>> from cclab.mongo import Document, Field
    >>>
    >>> class User(Document):
    ...     email: str
    ...     name: str
    ...     age: int = 0
    ...
    ...     class Settings:
    ...         name = "users"
    ...
    >>> # Create and save
    >>> user = User(email="alice@example.com", name="Alice", age=30)
    >>> await user.save()
    >>>
    >>> # Query with type-safe expressions
    >>> user = await User.find_one(User.email == "alice@example.com")
    >>> users = await User.find(User.age > 25).to_list()
"""

from __future__ import annotations

import inspect
from typing import Any, ClassVar, Dict, List, Optional, Type, TypeVar, Union, get_type_hints, get_origin, get_args

from .fields import FieldProxy, QueryExpr, merge_filters
from .query import QueryBuilder, AggregationBuilder
from .actions import (
    run_before_event, run_after_event, run_validate_on_save,
    EventType, Insert, Replace, Save, Delete,
)
from .links import Link, BackLink, WriteRules, DeleteRules, get_link_fields

# Import EmbeddedDocument for embedded document support
# Note: We don't use Pydantic - EmbeddedDocument is pure Python
# Validation happens in Rust
try:
    from .embedded import EmbeddedDocument
    EMBEDDED_DOCUMENT_AVAILABLE = True
except ImportError:
    EmbeddedDocument = None  # type: ignore
    EMBEDDED_DOCUMENT_AVAILABLE = False

T = TypeVar("T", bound="Document")


# ===================
# Embedded Document Helpers
# ===================

def is_embedded_document_type(field_type: Type) -> bool:
    """
    Check if a type is an EmbeddedDocument subclass.
    Handles Optional[EmbeddedDocument] by checking the inner type.

    Args:
        field_type: The type to check

    Returns:
        True if the type is an EmbeddedDocument subclass, False otherwise
    """
    if not EMBEDDED_DOCUMENT_AVAILABLE or EmbeddedDocument is None:
        return False

    # Handle Optional[EmbeddedDocument] which is Union[EmbeddedDocument, None]
    origin = get_origin(field_type)
    if origin is Union:
        args = get_args(field_type)
        # Optional[T] is Union[T, None]
        if len(args) == 2 and type(None) in args:
            # Get the non-None type
            inner_type = args[0] if args[1] is type(None) else args[1]
            return is_embedded_document_type(inner_type)

    try:
        # Check if it's a class and subclass of EmbeddedDocument
        return (
            inspect.isclass(field_type)
            and issubclass(field_type, EmbeddedDocument)
            and field_type is not EmbeddedDocument
        )
    except TypeError:
        # Not a class or can't check subclass
        return False


def is_list_of_embedded_document(field_type: Type) -> bool:
    """
    Check if a type is List[EmbeddedDocument] or list[EmbeddedDocument].

    Args:
        field_type: The type to check

    Returns:
        True if the type is List[SomeEmbeddedDocument], False otherwise
    """
    if not EMBEDDED_DOCUMENT_AVAILABLE:
        return False

    origin = get_origin(field_type)
    if origin is list or origin is List:
        args = get_args(field_type)
        if args and len(args) == 1:
            return is_embedded_document_type(args[0])

    return False


def get_embedded_document_inner_type(field_type: Type) -> Optional[Type]:
    """
    Get the inner EmbeddedDocument type from List[EmbeddedDocument].

    Args:
        field_type: The List type

    Returns:
        The inner EmbeddedDocument type, or None if not applicable
    """
    if is_list_of_embedded_document(field_type):
        args = get_args(field_type)
        return args[0] if args else None
    return None


def get_optional_embedded_document_type(field_type: Type) -> Optional[Type]:
    """
    Get the EmbeddedDocument type from Optional[EmbeddedDocument].

    Args:
        field_type: The Optional type

    Returns:
        The inner EmbeddedDocument type, or None if not Optional[EmbeddedDocument]
    """
    origin = get_origin(field_type)
    if origin is Union:
        args = get_args(field_type)
        # Optional[T] is Union[T, None]
        if len(args) == 2 and type(None) in args:
            inner_type = args[0] if args[1] is type(None) else args[1]
            if is_embedded_document_type(inner_type):  # Check the inner type, not the Union
                return inner_type
    return None


class Settings:
    """
    Default settings for Document classes.

    Override in your Document subclass to configure collection name, indexes, etc.

    Example:
        >>> class User(Document):
        ...     email: str
        ...
        ...     class Settings:
        ...         name = "users"
        ...         indexes = [
        ...             {"keys": [("email", 1)], "unique": True},
        ...         ]
        ...         use_revision = True  # Enable optimistic locking
        ...         use_state_management = True  # Track field changes
        ...
        >>> # Time-series collection example:
        >>> from cclab.timeseries import TimeSeriesConfig, Granularity
        >>> class SensorReading(Document):
        ...     sensor_id: str
        ...     timestamp: datetime
        ...     value: float
        ...
        ...     class Settings:
        ...         name = "sensor_readings"
        ...         timeseries = TimeSeriesConfig(
        ...             time_field="timestamp",
        ...             meta_field="sensor_id",
        ...             granularity=Granularity.minutes,
        ...         )
    """

    name: str = ""  # Collection name (defaults to class name lowercase)
    indexes: List[dict] = []  # Index definitions
    use_revision: bool = False  # Enable revision tracking (optimistic locking)
    use_state_management: bool = False  # Enable state management (track changes)
    use_validation: bool = False  # Enable validation on save
    timeseries: Optional[Any] = None  # TimeSeriesConfig for time-series collections
    is_root: bool = False  # Mark as root class for document inheritance


class DocumentMeta(type):
    """
    Metaclass for Document classes.

    This metaclass:
    1. Collects field annotations from the class
    2. Creates FieldProxy attributes for each field (enabling User.email syntax)
    3. Processes the Settings inner class
    4. Sets up the collection name
    5. Handles document inheritance (is_root, _class_id, child classes)

    Example:
        >>> class User(Document):
        ...     email: str  # Creates User.email as FieldProxy
        ...     name: str   # Creates User.name as FieldProxy
        ...
        >>> User.email  # FieldProxy("email", User)
        >>> User.email == "test"  # QueryExpr("email", "$eq", "test")

    Inheritance Example:
        >>> class Vehicle(Document):
        ...     name: str
        ...
        ...     class Settings:
        ...         name = "vehicles"
        ...         is_root = True  # Mark as root class
        ...
        >>> class Car(Vehicle):
        ...     num_wheels: int = 4
        ...
        >>> # Car documents stored in "vehicles" collection with _class_id="Car"
    """

    # Global registry of document classes by name (for polymorphic loading)
    _document_registry: Dict[str, type] = {}

    # Cache for get_type_hints() results per class (performance optimization)
    # Avoids expensive compile() and eval() calls on every _from_db()
    _type_hints_cache: Dict[type, Dict[str, Any]] = {}

    def __new__(
        mcs,
        name: str,
        bases: tuple,
        namespace: dict,
        **kwargs: Any,
    ) -> "DocumentMeta":
        # Create the class first
        cls = super().__new__(mcs, name, bases, namespace)

        # Skip processing for the base Document class itself
        if name == "Document" and not bases:
            return cls

        # Get all annotations including from parent classes
        annotations = {}
        for base in reversed(cls.__mro__):
            if hasattr(base, "__annotations__"):
                annotations.update(base.__annotations__)

        # Store field names for later use
        cls._fields: Dict[str, type] = {}

        # Store default values before replacing with FieldProxy
        cls._field_defaults: Dict[str, Any] = {}

        # Create FieldProxy for each field annotation
        for field_name, field_type in annotations.items():
            # Skip private attributes and ClassVar
            if field_name.startswith("_"):
                continue
            if hasattr(field_type, "__origin__") and field_type.__origin__ is ClassVar:
                continue

            # Store field info
            cls._fields[field_name] = field_type

            # Capture default value BEFORE replacing with FieldProxy
            current_value = getattr(cls, field_name, None)
            if current_value is not None and not isinstance(current_value, FieldProxy):
                cls._field_defaults[field_name] = current_value

            # Create FieldProxy only if not already set (allows override)
            if not isinstance(current_value, FieldProxy):
                setattr(cls, field_name, FieldProxy(field_name, cls))

        # Process Settings class
        settings_cls = namespace.get("Settings", Settings)
        cls._settings = settings_cls

        # Set collection name (default to lowercase class name)
        if hasattr(settings_cls, "name") and settings_cls.name:
            cls._collection_name = settings_cls.name
        else:
            cls._collection_name = name.lower()

        # Store time-series configuration if present
        cls._timeseries_config = getattr(settings_cls, "timeseries", None)

        # =====================
        # Document Inheritance
        # =====================

        # Check if this is a root class
        is_root = getattr(settings_cls, "is_root", False)

        if is_root:
            # This is a root class - initialize inheritance tracking
            cls._is_root = True
            cls._class_id = name
            cls._child_classes: Dict[str, type] = {name: cls}
            cls._root_class = cls
        else:
            # Check if any base class is a root class
            root_found = False
            for base in bases:
                if hasattr(base, "_is_root") and base._is_root:
                    # Found a root parent
                    cls._is_root = False
                    cls._class_id = name
                    cls._root_class = base._root_class
                    # Register this child class with the root
                    base._root_class._child_classes[name] = cls
                    # Inherit collection name from root
                    cls._collection_name = base._root_class._collection_name
                    root_found = True
                    break
                elif hasattr(base, "_root_class") and base._root_class:
                    # Found a child of a root (grandchild inheritance)
                    cls._is_root = False
                    cls._class_id = name
                    cls._root_class = base._root_class
                    # Register this child class with the root
                    base._root_class._child_classes[name] = cls
                    # Inherit collection name from root
                    cls._collection_name = base._root_class._collection_name
                    root_found = True
                    break

            if not root_found:
                # Not part of an inheritance hierarchy
                cls._is_root = False
                cls._class_id = None
                cls._root_class = None
                cls._child_classes = {}

        # Register in global registry for polymorphic loading
        mcs._document_registry[name] = cls

        return cls


class Document(metaclass=DocumentMeta):
    """
    Base class for MongoDB documents.

    Provides:
    - CRUD operations (save, delete, refresh)
    - Class methods for querying (find, find_one, get, aggregate)
    - Beanie-compatible query syntax (User.email == "x")
    - Automatic _id handling

    Example:
        >>> from cclab.mongo import Document, Field
        >>>
        >>> class User(Document):
        ...     email: str
        ...     name: str
        ...     age: int = 0
        ...     groups: list[str] = Field(default_factory=list)
        ...
        ...     class Settings:
        ...         name = "users"
        ...
        >>> # Create and save
        >>> user = User(email="alice@example.com", name="Alice")
        >>> await user.save()
        >>> print(user.id)  # "507f1f77bcf86cd799439011"
        >>>
        >>> # Query
        >>> user = await User.find_one(User.email == "alice@example.com")
        >>> users = await User.find(User.age > 25).sort(-User.created_at).to_list()
        >>>
        >>> # Update
        >>> user.age = 31
        >>> await user.save()
        >>>
        >>> # Delete
        >>> await user.delete()
    """

    # Class-level attributes (set by metaclass)
    _fields: ClassVar[Dict[str, type]] = {}
    _field_defaults: ClassVar[Dict[str, Any]] = {}
    _settings: ClassVar[Type[Settings]] = Settings
    _collection_name: ClassVar[str] = ""
    _timeseries_config: ClassVar[Optional[Any]] = None  # TimeSeriesConfig

    # Inheritance attributes (set by metaclass)
    _is_root: ClassVar[bool] = False  # True if this is a root class
    _class_id: ClassVar[Optional[str]] = None  # Type discriminator for inheritance
    _root_class: ClassVar[Optional[Type["Document"]]] = None  # Reference to root class
    _child_classes: ClassVar[Dict[str, Type["Document"]]] = {}  # Child class registry

    # Instance attributes
    _id: Optional[str] = None
    _data: Dict[str, Any]
    _revision_id: Optional[int] = None  # Revision tracking
    _original_data: Optional[Any] = None  # State management (StateTracker or Dict for backward compat)
    _previous_changes: Optional[Dict[str, Any]] = None  # Previous saved changes

    def __init__(self, **kwargs: Any) -> None:
        """
        Initialize a document instance.

        Args:
            **kwargs: Field values for the document

        Example:
            >>> user = User(email="alice@example.com", name="Alice", age=30)
        """
        # Extract _id if provided
        self._id = kwargs.pop("_id", None)
        if self._id is not None:
            self._id = str(self._id)

        # Handle 'id' alias for _id
        if "id" in kwargs and self._id is None:
            self._id = str(kwargs.pop("id"))

        # Store all data
        self._data = kwargs.copy()

        # Set default values for fields not provided
        for field_name, field_type in self._fields.items():
            if field_name not in self._data:
                # Check for stored default value
                if field_name in self._field_defaults:
                    default = self._field_defaults[field_name]
                    # Check if it's a FieldInfo with default_factory
                    if hasattr(default, "default_factory") and default.default_factory:
                        self._data[field_name] = default.default_factory()
                    elif hasattr(default, "default") and default.default is not ...:
                        self._data[field_name] = default.default
                    else:
                        # It's a plain default value (like version: int = 1)
                        self._data[field_name] = default

        # Initialize state management if enabled
        if self._use_state_management:
            self._save_state()

    def _save_state(self) -> None:
        """Save current data as original state for change tracking."""
        from .state import StateTracker
        self._original_data = StateTracker(self._data)

    @property
    def _use_state_management(self) -> bool:
        """Check if state management is enabled."""
        return getattr(self._settings, "use_state_management", False)

    @property
    def _use_revision(self) -> bool:
        """Check if revision tracking is enabled."""
        return getattr(self._settings, "use_revision", False)

    @property
    def revision_id(self) -> Optional[int]:
        """Get the current revision ID."""
        return self._revision_id

    # ===================
    # State Management
    # ===================

    @property
    def is_changed(self) -> bool:
        """
        Check if any field has been modified since last save/load.

        Only works if use_state_management = True in Settings.

        Returns:
            True if any field has changed, False otherwise

        Example:
            >>> user.name = "New Name"
            >>> user.is_changed  # True
        """
        if not self._use_state_management or self._original_data is None:
            return False

        from .state import StateTracker
        if isinstance(self._original_data, StateTracker):
            return self._original_data.is_modified()

        # Backward compatibility with dict-based state management
        return self._data != self._original_data

    def has_changed(self, field: str) -> bool:
        """
        Check if a specific field has been modified.

        Args:
            field: Field name to check

        Returns:
            True if the field has changed

        Example:
            >>> user.name = "New Name"
            >>> user.has_changed("name")  # True
            >>> user.has_changed("email")  # False
        """
        if not self._use_state_management or self._original_data is None:
            return False

        from .state import StateTracker
        if isinstance(self._original_data, StateTracker):
            return self._original_data.has_changed(field)

        # Backward compatibility with dict-based state management
        return self._data.get(field) != self._original_data.get(field)

    def get_changes(self) -> Dict[str, Any]:
        """
        Get all changed fields with their new values.

        Returns:
            Dict mapping field names to new values

        Example:
            >>> user.name = "New Name"
            >>> user.age = 30
            >>> user.get_changes()  # {"name": "New Name", "age": 30}
        """
        if not self._use_state_management or self._original_data is None:
            return {}

        from .state import StateTracker
        if isinstance(self._original_data, StateTracker):
            return self._original_data.get_changes()

        # Backward compatibility with dict-based state management
        changes = {}
        for key, value in self._data.items():
            if key not in self._original_data or self._original_data[key] != value:
                changes[key] = value
        return changes

    def get_previous_changes(self) -> Optional[Dict[str, Any]]:
        """
        Get the changes from the previous save operation.

        Returns:
            Dict of field changes from last save, or None if no previous save

        Example:
            >>> await user.save()
            >>> user.get_previous_changes()  # {"name": "New Name"}
        """
        return self._previous_changes

    def rollback(self) -> None:
        """
        Rollback all changes to the original state.

        Reverts all field values to what they were when loaded or last saved.

        Example:
            >>> user.name = "Wrong Name"
            >>> user.rollback()
            >>> user.name  # Original name
        """
        if self._use_state_management and self._original_data is not None:
            from .state import StateTracker
            if isinstance(self._original_data, StateTracker):
                self._original_data.rollback()
            else:
                # Backward compatibility with dict-based state management
                import copy
                self._data = copy.deepcopy(self._original_data)

    @property
    def id(self) -> Optional[str]:
        """Get the document's ObjectId as a hex string."""
        return self._id

    @classmethod
    def __collection_name__(cls) -> str:
        """Get the collection name for this document type."""
        return cls._collection_name

    def __getattr__(self, name: str) -> Any:
        """Get field value by attribute access for non-FieldProxy fields."""
        if name.startswith("_"):
            raise AttributeError(f"'{type(self).__name__}' has no attribute '{name}'")
        # FieldProxy handles its own fields via descriptor protocol
        # This is for any other dynamic attributes
        if "_data" in self.__dict__ and name in self._data:
            return self._data[name]
        raise AttributeError(f"'{type(self).__name__}' has no attribute '{name}'")

    def __setattr__(self, name: str, value: Any) -> None:
        """Set field value by attribute access."""
        if name.startswith("_"):
            super().__setattr__(name, value)
        elif name in self._fields:
            # Track change for state management (COW)
            if "_original_data" in self.__dict__ and self._original_data is not None:
                from .state import StateTracker
                if isinstance(self._original_data, StateTracker):
                    old_value = self._data.get(name)
                    self._original_data.track_change(name, old_value)
            # Use descriptor for known fields
            self._data[name] = value
        elif "_data" in self.__dict__:
            # Track change for state management (COW)
            if "_original_data" in self.__dict__ and self._original_data is not None:
                from .state import StateTracker
                if isinstance(self._original_data, StateTracker):
                    old_value = self._data.get(name)
                    self._original_data.track_change(name, old_value)
            self._data[name] = value
        else:
            super().__setattr__(name, value)

    def to_dict(self) -> Dict[str, Any]:
        """
        Convert document to dictionary.

        Handles embedded documents (EmbeddedDocument), Links, and regular fields.

        Returns:
            Dictionary with all field values, including _id and _class_id if set
        """
        result = {}
        for key, value in self._data.items():
            # Convert Link to reference for storage
            if isinstance(value, Link):
                result[key] = value.to_ref()
            elif isinstance(value, BackLink):
                # BackLinks are not stored - they're computed on fetch
                continue
            # Serialize embedded documents (EmbeddedDocument)
            elif EMBEDDED_DOCUMENT_AVAILABLE and EmbeddedDocument is not None and isinstance(value, EmbeddedDocument):
                result[key] = value.to_dict()
            # Serialize lists containing embedded documents
            elif isinstance(value, list):
                result[key] = [
                    item.to_dict() if (EMBEDDED_DOCUMENT_AVAILABLE and EmbeddedDocument is not None and isinstance(item, EmbeddedDocument)) else item
                    for item in value
                ]
            else:
                result[key] = value

        if self._id:
            result["_id"] = self._id
        # Add _class_id for polymorphic documents (inheritance hierarchy)
        if self._class_id is not None:
            result["_class_id"] = self._class_id
        return result

    @classmethod
    def _from_db(cls: Type[T], data: Dict[str, Any], validate: bool = False) -> T:
        """
        Create instance from database document.

        Supports polymorphic deserialization: if the document contains a
        _class_id field and the class is part of an inheritance hierarchy,
        the correct child class will be instantiated.

        Args:
            data: Raw document from MongoDB
            validate: If True, run Pydantic validation. If False (default), skip validation
                     for better performance. Database data is assumed to be valid.

        Returns:
            Document instance (may be a subclass if polymorphic)

        Performance:
            - validate=False: 2-3x faster (recommended for database loads)
            - validate=True: Full Pydantic validation (use for untrusted data)
        """
        # Extract _id
        _id = data.pop("_id", None)
        if _id is not None:
            _id = str(_id)

        # Extract _class_id for polymorphic loading
        class_id = data.pop("_class_id", None)

        # Extract revision_id if present
        revision_id = data.pop("revision_id", None)

        # Determine the target class for instantiation
        target_cls = cls
        if class_id:
            # First try the class-level registry (for same-session classes)
            if cls._child_classes:
                child_cls = cls._child_classes.get(class_id)
                if child_cls is not None:
                    target_cls = child_cls
            # If not found in class registry, try global registry
            if target_cls is cls and class_id in DocumentMeta._document_registry:
                registered_cls = DocumentMeta._document_registry[class_id]
                # Only use if it's a compatible type (subclass or same hierarchy)
                if issubclass(registered_cls, cls) or (
                    hasattr(registered_cls, "_root_class")
                    and registered_cls._root_class is cls._root_class
                ):
                    target_cls = registered_cls

        # Convert Link fields from stored references
        link_fields = get_link_fields(target_cls)
        for field_name, (link_type, target_type) in link_fields.items():
            if link_type == "Link" and field_name in data:
                ref = data[field_name]
                if ref is not None and not isinstance(ref, Link):
                    # Create Link from stored reference
                    data[field_name] = Link(ref)

        # Deserialize embedded documents (EmbeddedDocument fields)
        if EMBEDDED_DOCUMENT_AVAILABLE:
            try:
                # Check cache first to avoid expensive get_type_hints() call
                # get_type_hints() uses compile() and eval() which are extremely slow
                if target_cls not in DocumentMeta._type_hints_cache:
                    DocumentMeta._type_hints_cache[target_cls] = get_type_hints(target_cls)
                hints = DocumentMeta._type_hints_cache[target_cls]
            except Exception:
                # get_type_hints can fail in some edge cases
                hints = {}

            for field_name, field_type in hints.items():
                if field_name.startswith("_") or field_name not in data:
                    continue

                field_value = data[field_name]

                # Handle EmbeddedDocument fields (including Optional[EmbeddedDocument])
                if is_embedded_document_type(field_type):
                    if field_value is not None and isinstance(field_value, dict):
                        # For Optional[EmbeddedDocument], extract the inner type
                        # is_embedded_document_type() already handles Optional unwrapping
                        inner_type = get_optional_embedded_document_type(field_type)
                        if inner_type:
                            # It's Optional[EmbeddedDocument], use the inner type
                            data[field_name] = inner_type.from_dict(field_value)
                        else:
                            # It's a direct EmbeddedDocument type
                            data[field_name] = field_type.from_dict(field_value)

                # Handle List[EmbeddedDocument] fields
                elif is_list_of_embedded_document(field_type):
                    inner_type = get_embedded_document_inner_type(field_type)
                    if inner_type and isinstance(field_value, list):
                        # Deserialize list of dicts to list of EmbeddedDocument instances
                        data[field_name] = [
                            inner_type.from_dict(item) if isinstance(item, dict) else item
                            for item in field_value
                        ]

        if validate:
            # Slow path: Full Pydantic validation (backward compatibility)
            instance = target_cls(**data)
            instance._id = _id
        else:
            # Fast path: Skip Pydantic validation (2-3x faster)
            # Database data is already valid, so we bypass __init__
            instance = object.__new__(target_cls)
            instance._id = _id
            instance._data = data
            instance._revision_id = None
            instance._original_data = None
            instance._previous_changes = None

        # Restore revision_id
        if revision_id is not None:
            instance._revision_id = revision_id

        # Initialize state management after loading
        if instance._use_state_management:
            instance._save_state()

        return instance

    # ===================
    # CRUD Operations
    # ===================

    async def save(
        self,
        link_rule: WriteRules = WriteRules.DO_NOTHING,
    ) -> str:
        """
        Save the document to MongoDB.

        If the document has an _id, updates the existing document.
        Otherwise, inserts a new document.

        All validation happens in Rust during BSON conversion - there is no
        overhead to skipping validation in Python since Rust always validates.

        Args:
            link_rule: How to handle linked documents on save:
                - WriteRules.DO_NOTHING (default): Only save this document
                - WriteRules.WRITE: Cascade save to all linked documents first

        Lifecycle hooks:
        - before_event(Insert) or before_event(Save) runs before insert
        - after_event(Insert) or after_event(Save) runs after insert

        Returns:
            The document's ObjectId as a hex string

        Example:
            >>> user = User(email="alice@example.com", name="Alice")
            >>> doc_id = await user.save()
            >>> print(doc_id)  # "507f1f77bcf86cd799439011"
            >>>
            >>> # With cascade save to linked documents:
            >>> post = Post(title="Hello", author=user)
            >>> await post.save(link_rule=WriteRules.WRITE)  # Saves user first
        """
        from . import _engine

        collection_name = self.__collection_name__()
        is_insert = self._id is None

        # Run validation hooks if configured in Settings
        await run_validate_on_save(self)

        # Cascade save linked documents if requested
        if link_rule == WriteRules.WRITE:
            await self._save_linked_documents()

        # Run before hooks
        if is_insert:
            await run_before_event(self, EventType.INSERT)
        else:
            await run_before_event(self, EventType.SAVE)

        # Store current changes for state management
        if self._use_state_management:
            self._previous_changes = self.get_changes()

        # Handle revision tracking
        if self._use_revision:
            if is_insert:
                self._revision_id = 1
            else:
                self._revision_id = (self._revision_id or 0) + 1

        data = self.to_dict()

        # Add revision_id to data if revision tracking is enabled
        if self._use_revision:
            data["revision_id"] = self._revision_id

        if self._id:
            # Update existing
            data.pop("_id", None)

            # Build filter with optimistic locking if revision tracking enabled
            filter_doc = {"_id": self._id}
            if self._use_revision and self._revision_id > 1:
                # Check that revision hasn't changed (optimistic locking)
                filter_doc["revision_id"] = self._revision_id - 1

            result = await _engine.update_one(
                collection_name,
                filter_doc,
                {"$set": data},
            )

            # Check for revision conflict (optimistic locking failure)
            if self._use_revision and result == 0 and self._revision_id > 1:
                raise ValueError(
                    f"Revision conflict: document was modified by another process. "
                    f"Expected revision {self._revision_id - 1}, but document has changed."
                )

            result_id = self._id
        else:
            # Insert new (with schema validation if enabled)
            result_id = await _engine.insert_one(collection_name, data, type(self))
            self._id = result_id

        # Update state management after successful save
        if self._use_state_management:
            self._save_state()

        # Run after hooks
        if is_insert:
            await run_after_event(self, EventType.INSERT)
        else:
            await run_after_event(self, EventType.SAVE)

        return result_id

    async def _save_linked_documents(self) -> None:
        """
        Save all linked documents before saving this document.

        This is called when save(link_rule=WriteRules.WRITE) is used.
        It saves all Link[T] field values that are Document instances.
        """
        link_fields = get_link_fields(type(self))

        for field_name, (link_type, _) in link_fields.items():
            if link_type != "Link":
                continue

            value = self._data.get(field_name)
            if value is None:
                continue

            # If it's a Link instance with a document, save the document
            if isinstance(value, Link) and value._document is not None:
                await value._document.save(link_rule=WriteRules.WRITE)
            # If it's a Document instance directly, save it
            elif isinstance(value, Document):
                await value.save(link_rule=WriteRules.WRITE)
                # Convert to Link after saving
                self._data[field_name] = Link(value)

    async def insert(self) -> str:
        """
        Insert a new document.

        Raises an error if the document already has an _id.

        Returns:
            The document's ObjectId as a hex string

        Example:
            >>> user = User(email="alice@example.com", name="Alice")
            >>> doc_id = await user.insert()
        """
        if self._id:
            raise ValueError("Document already has an _id. Use save() to update.")
        return await self.save()

    async def delete(self, link_rule: DeleteRules = DeleteRules.DO_NOTHING) -> bool:
        """
        Delete this document from MongoDB.

        Args:
            link_rule: How to handle linked documents on delete:
                - DeleteRules.DO_NOTHING (default): Only delete this document
                - DeleteRules.DELETE_LINKS: Delete documents that link to this one via BackLink

        Lifecycle hooks:
        - before_event(Delete) runs before delete
        - after_event(Delete) runs after delete

        Returns:
            True if document was deleted, False if not found

        Example:
            >>> user = await User.find_one(User.email == "alice@example.com")
            >>> await user.delete()
            >>>
            >>> # With cascade delete of linking documents:
            >>> await user.delete(link_rule=DeleteRules.DELETE_LINKS)
        """
        if not self._id:
            raise ValueError("Document has no _id")

        # Run before hooks
        await run_before_event(self, EventType.DELETE)

        # Delete documents that link to this one if requested
        if link_rule == DeleteRules.DELETE_LINKS:
            await self._delete_linked_documents()

        from . import _engine

        collection_name = self.__collection_name__()
        deleted_count = await _engine.delete_one(collection_name, {"_id": self._id})
        deleted = deleted_count > 0

        # Run after hooks
        if deleted:
            await run_after_event(self, EventType.DELETE)

        return deleted

    async def _delete_linked_documents(self) -> None:
        """
        Delete documents that link to this document via BackLink.

        This is called when delete(link_rule=DeleteRules.DELETE_LINKS) is used.
        It finds all BackLink fields and deletes documents that reference this one.
        """
        link_fields = get_link_fields(type(self))

        for field_name, (link_type, target_type) in link_fields.items():
            if link_type != "BackLink":
                continue

            # Get BackLink configuration from _field_defaults
            # (The class attribute is replaced with FieldProxy by metaclass)
            default = self._field_defaults.get(field_name)
            if default is None or not isinstance(default, BackLink):
                continue

            link_field = default._link_field
            if not link_field:
                continue

            # Resolve the target class using the same method as fetch
            target_cls = self._resolve_document_class(target_type)

            if target_cls is not None:
                # Delete all documents that link to this one
                await target_cls.delete_many({link_field: self._id})

    async def refresh(self) -> None:
        """
        Refresh the document from the database.

        Reloads all field values from MongoDB.

        Example:
            >>> await user.refresh()  # Get latest values from DB
        """
        if not self._id:
            raise ValueError("Document has no _id")

        from . import _engine

        collection_name = self.__collection_name__()
        data = await _engine.find_one(collection_name, {"_id": self._id})

        if data is None:
            raise ValueError("Document not found in database")

        # Update data
        data.pop("_id", None)
        self._data = data

    async def fetch_all_links(self, depth: int = 1, batch_mode: bool = True) -> None:
        """
        Fetch all linked documents for this document.

        This method resolves all Link[T] fields by fetching the referenced
        documents from the database. BackLink fields are also populated
        by querying for documents that reference this one.

        Args:
            depth: How many levels of nested links to fetch (default 1).
                   Set to 0 to skip fetching. Higher values fetch links
                   of linked documents recursively.
            batch_mode: If True, uses batched fetching for better performance (default).
                        Set to False for backward-compatible individual queries.

        Performance:
            With batch_mode=True (default):
            - 100 docs with 3 links each = 4 queries (1 fetch + 3 batches)
            - ~75x faster than individual queries for large result sets

        Example:
            >>> post = await Post.find_one(Post.id == id)
            >>> await post.fetch_all_links()
            >>> print(post.author.name)  # Author is now resolved
            >>>
            >>> # Fetch nested links (author's posts)
            >>> await post.fetch_all_links(depth=2)
        """
        if depth <= 0:
            return

        if batch_mode:
            # Use batched fetching for better performance (Week 4-5 optimization)
            await self._fetch_all_links_batched(depth)
        else:
            # Fallback to individual queries (backward compatible)
            link_fields = get_link_fields(type(self))
            for field_name, (link_type, target_type) in link_fields.items():
                if link_type == "Link":
                    await self._fetch_link_field(field_name, target_type, depth)
                elif link_type == "BackLink":
                    await self._fetch_backlink_field(field_name, target_type, depth)

    async def _fetch_all_links_batched(self, depth: int) -> None:
        """
        Fetch all linked documents using batched queries (Week 4-5 optimization).

        This is a high-performance implementation that batches link fetching
        to avoid the N+1 query problem. Instead of making one query per link,
        it groups links by target collection and fetches them in batches.

        Performance:
            - Before: N documents × M links = N×M queries
            - After: N documents × M links = M queries (one per unique target type)
            - Improvement: Up to 75x reduction in latency for large datasets

        Algorithm:
            1. Collection Phase: Gather all link refs grouped by target type
            2. Batch Phase: Single $in query per target type
            3. Mapping Phase: Map fetched docs back to links
        """
        from collections import defaultdict

        link_fields = get_link_fields(type(self))

        # Phase 1: Collect all link references grouped by target type
        # Format: {target_cls: [(field_name, ref_id), ...]}
        link_refs_by_type = defaultdict(list)
        backlink_fields = []

        for field_name, (link_type, target_type) in link_fields.items():
            if link_type == "Link":
                value = self._data.get(field_name)
                if value is None:
                    continue

                # Resolve target class
                target_cls = self._resolve_document_class(target_type)
                if target_cls is None:
                    continue

                # Get the reference ID
                if isinstance(value, Link):
                    if value._document is not None:
                        # Already fetched
                        continue
                    ref_id = value._ref
                elif isinstance(value, str):
                    ref_id = value
                else:
                    continue

                if ref_id is not None:
                    link_refs_by_type[target_cls].append((field_name, ref_id))

            elif link_type == "BackLink":
                # BackLinks need special handling (can't batch easily)
                backlink_fields.append((field_name, target_type))

        # Phase 2: Batch fetch all links per target type
        for target_cls, refs in link_refs_by_type.items():
            if not refs:
                continue

            # Extract unique IDs
            ref_ids = list(set(ref_id for _, ref_id in refs))

            # Batch query: fetch all documents with $in
            linked_docs = await target_cls.find({"_id": {"$in": ref_ids}}).to_list()

            # Create ID -> document mapping for O(1) lookup
            docs_by_id = {str(doc._id): doc for doc in linked_docs}

            # Phase 3: Map fetched documents back to their links
            for field_name, ref_id in refs:
                linked_doc = docs_by_id.get(str(ref_id))
                if linked_doc is not None:
                    # Update the link with the fetched document
                    link = Link(linked_doc, document_class=target_cls)
                    self._data[field_name] = link

            # Recursively fetch nested links (depth - 1)
            if depth > 1:
                for doc in linked_docs:
                    await doc.fetch_all_links(depth=depth - 1, batch_mode=True)

        # Handle BackLinks (can't batch these easily, use original method)
        for field_name, target_type in backlink_fields:
            await self._fetch_backlink_field(field_name, target_type, depth)

    async def _fetch_link_field(
        self, field_name: str, target_type: type, depth: int
    ) -> None:
        """Fetch a single Link field."""
        value = self._data.get(field_name)
        if value is None:
            return

        # Resolve target class
        target_cls = self._resolve_document_class(target_type)
        if target_cls is None:
            return

        # Get the reference ID
        if isinstance(value, Link):
            if value._document is not None:
                # Already fetched, maybe fetch nested links
                if depth > 1:
                    await value._document.fetch_all_links(depth=depth - 1)
                return
            ref_id = value._ref
        elif isinstance(value, str):
            ref_id = value
        else:
            return

        if ref_id is None:
            return

        # Fetch the linked document
        linked_doc = await target_cls.find_one({"_id": ref_id})
        if linked_doc is not None:
            # Fetch nested links if depth > 1
            if depth > 1:
                await linked_doc.fetch_all_links(depth=depth - 1)

            # Update the link with the fetched document
            link = Link(linked_doc, document_class=target_cls)
            self._data[field_name] = link

    async def _fetch_backlink_field(
        self, field_name: str, target_type: type, depth: int
    ) -> None:
        """Fetch a BackLink field (reverse relationship)."""
        if self._id is None:
            return

        # Get the BackLink configuration
        default = self._field_defaults.get(field_name)
        if not hasattr(default, "_link_field") or not default._link_field:
            return

        link_field = default._link_field

        # Resolve target class
        target_cls = self._resolve_document_class(target_type)
        if target_cls is None:
            return

        # Find documents that link to this one
        linked_docs = await target_cls.find({link_field: self._id}).to_list()

        # Fetch nested links if depth > 1
        if depth > 1:
            for doc in linked_docs:
                await doc.fetch_all_links(depth=depth - 1)

        # Create a BackLink instance with the fetched documents
        backlink = BackLink(document_class=target_cls, link_field=link_field)
        backlink._documents = linked_docs
        self._data[field_name] = backlink

    def _resolve_document_class(self, target_type: type) -> Optional[Type["Document"]]:
        """Resolve a type annotation to a Document class."""
        if target_type is None:
            return None

        if isinstance(target_type, type) and issubclass(target_type, Document):
            return target_type

        # Handle forward references (strings)
        if isinstance(target_type, str):
            return DocumentMeta._document_registry.get(target_type)

        # Handle ForwardRef
        from typing import ForwardRef
        if isinstance(target_type, ForwardRef):
            type_name = target_type.__forward_arg__
            return DocumentMeta._document_registry.get(type_name)

        return None

    # ===================
    # Query Methods
    # ===================

    @classmethod
    def find(cls: Type[T], *filters: QueryExpr | dict) -> QueryBuilder[T]:
        """
        Find documents matching the filters.

        Args:
            *filters: QueryExpr objects or dict filters

        Returns:
            QueryBuilder for chaining additional operations

        Example:
            >>> # Find all active users
            >>> users = await User.find(User.active == True).to_list()
            >>>
            >>> # Chained operations
            >>> users = await User.find(User.age > 25) \\
            ...     .sort(-User.created_at) \\
            ...     .skip(10) \\
            ...     .limit(20) \\
            ...     .to_list()
        """
        return QueryBuilder(cls, filters)

    @classmethod
    async def find_one(
        cls: Type[T],
        *filters: QueryExpr | dict,
        fetch_links: bool = False,
    ) -> Optional[T]:
        """
        Find a single document matching the filters.

        Args:
            *filters: QueryExpr objects or dict filters
            fetch_links: If True, automatically fetch all linked documents

        Returns:
            Document instance or None if not found

        Example:
            >>> user = await User.find_one(User.email == "alice@example.com")
            >>> if user:
            ...     print(user.name)
            >>>
            >>> # With fetch_links - linked documents are resolved
            >>> post = await Post.find_one(Post.id == id, fetch_links=True)
            >>> print(post.author.name)  # Author is fetched
        """
        from . import _engine

        collection_name = cls.__collection_name__()
        filter_doc = merge_filters(filters)

        data = await _engine.find_one(collection_name, filter_doc)
        if data is None:
            return None

        doc = cls._from_db(data, validate=False)  # Database data is already valid

        if fetch_links:
            await doc.fetch_all_links()

        return doc

    @classmethod
    async def get(cls: Type[T], doc_id: str) -> Optional[T]:
        """
        Get a document by its ObjectId.

        Args:
            doc_id: ObjectId as a hex string

        Returns:
            Document instance or None if not found

        Example:
            >>> user = await User.get("507f1f77bcf86cd799439011")
        """
        return await cls.find_one({"_id": doc_id})

    @classmethod
    async def all(cls: Type[T]) -> List[T]:
        """
        Get all documents in the collection.

        Returns:
            List of all documents

        Example:
            >>> all_users = await User.all()
        """
        return await cls.find().to_list()

    @classmethod
    async def count(cls: Type[T], *filters: QueryExpr | dict) -> int:
        """
        Count documents matching the filters.

        Args:
            *filters: QueryExpr objects or dict filters

        Returns:
            Number of matching documents

        Example:
            >>> active_count = await User.count(User.active == True)
        """
        return await cls.find(*filters).count()

    @classmethod
    def aggregate(cls: Type[T], pipeline: List[dict]) -> AggregationBuilder[T]:
        """
        Run an aggregation pipeline.

        Args:
            pipeline: MongoDB aggregation pipeline stages

        Returns:
            AggregationBuilder for executing the pipeline

        Example:
            >>> results = await User.aggregate([
            ...     {"$match": {"active": True}},
            ...     {"$group": {"_id": "$department", "count": {"$sum": 1}}},
            ... ]).to_list()
        """
        return AggregationBuilder(cls, pipeline)

    # ===================
    # Replace Operations
    # ===================

    async def replace(self) -> bool:
        """
        Replace the entire document in MongoDB.

        Unlike save(), this replaces the document entirely rather than
        using $set for partial updates.

        Lifecycle hooks:
        - before_event(Replace) runs before replace
        - after_event(Replace) runs after replace

        Returns:
            True if document was replaced, False if not found

        Example:
            >>> user = await User.get(user_id)
            >>> user.name = "New Name"
            >>> user.email = "new@email.com"
            >>> await user.replace()  # Replaces entire document
        """
        if not self._id:
            raise ValueError("Document has no _id. Use save() for new documents.")

        # Run validation if enabled
        await run_validate_on_save(self)

        # Run before hooks
        await run_before_event(self, EventType.REPLACE)

        from . import _engine

        collection_name = self.__collection_name__()
        data = self.to_dict()
        data.pop("_id", None)

        result = await _engine.replace_one(
            collection_name,
            {"_id": self._id},
            data,
        )
        replaced = result["modified_count"] > 0

        # Run after hooks
        if replaced:
            await run_after_event(self, EventType.REPLACE)

        return replaced

    @classmethod
    async def replace_one(
        cls: Type[T],
        filter: QueryExpr | dict,
        replacement: T | dict,
        upsert: bool = False,
    ) -> dict:
        """
        Replace a single document matching the filter.

        Args:
            filter: Query filter
            replacement: Replacement document (Document instance or dict)
            upsert: If True, insert if no match

        Returns:
            Dict with matched_count, modified_count, upserted_id

        Example:
            >>> result = await User.replace_one(
            ...     User.email == "old@email.com",
            ...     User(email="new@email.com", name="New User"),
            ... )
        """
        from . import _engine

        collection_name = cls.__collection_name__()

        # Build filter
        if isinstance(filter, QueryExpr):
            filter_doc = filter.to_filter()
        else:
            filter_doc = filter

        # Get replacement data
        if isinstance(replacement, Document):
            replacement_doc = replacement.to_dict()
            replacement_doc.pop("_id", None)
        else:
            replacement_doc = dict(replacement)
            replacement_doc.pop("_id", None)

        return await _engine.replace_one(
            collection_name, filter_doc, replacement_doc, upsert
        )

    # ===================
    # Distinct Operations
    # ===================

    @classmethod
    async def distinct(
        cls: Type[T],
        field: str | Any,
        *filters: QueryExpr | dict,
    ) -> List[Any]:
        """
        Get distinct values for a field.

        Args:
            field: Field name (string or FieldProxy)
            *filters: Optional query filters

        Returns:
            List of distinct values

        Example:
            >>> # Get all unique departments
            >>> departments = await User.distinct("department")
            >>> departments = await User.distinct(User.department)

            >>> # With filter
            >>> active_depts = await User.distinct(
            ...     User.department,
            ...     User.active == True
            ... )
        """
        from . import _engine

        collection_name = cls.__collection_name__()

        # Handle FieldProxy
        field_name = field.name if hasattr(field, "name") else str(field)

        # Build filter
        filter_doc = merge_filters(filters) if filters else None

        return await _engine.distinct(collection_name, field_name, filter_doc)

    # ===================
    # Find One and Modify
    # ===================

    @classmethod
    async def find_one_and_update(
        cls: Type[T],
        filter: QueryExpr | dict,
        update: dict,
        return_document: str = "before",
        upsert: bool = False,
        sort: Optional[dict] = None,
    ) -> Optional[T]:
        """
        Find a document and update it atomically.

        Args:
            filter: Query filter
            update: Update operations
            return_document: "before" or "after" (which version to return)
            upsert: If True, insert if no match
            sort: Sort specification to determine which doc if multiple match

        Returns:
            The document (before or after update) or None

        Example:
            >>> # Get and increment a counter atomically
            >>> counter = await Counter.find_one_and_update(
            ...     Counter.name == "orders",
            ...     {"$inc": {"value": 1}},
            ...     return_document="after",
            ... )
            >>> print(counter.value)  # Incremented value
        """
        from . import _engine

        collection_name = cls.__collection_name__()

        # Build filter
        if isinstance(filter, QueryExpr):
            filter_doc = filter.to_filter()
        else:
            filter_doc = filter

        result = await _engine.find_one_and_update(
            collection_name, filter_doc, update, return_document, upsert, sort
        )

        if result is None:
            return None
        return cls._from_db(result, validate=False)  # Database data is already valid

    @classmethod
    async def find_one_and_replace(
        cls: Type[T],
        filter: QueryExpr | dict,
        replacement: T | dict,
        return_document: str = "before",
        upsert: bool = False,
    ) -> Optional[T]:
        """
        Find a document and replace it atomically.

        Args:
            filter: Query filter
            replacement: Replacement document
            return_document: "before" or "after" (which version to return)
            upsert: If True, insert if no match

        Returns:
            The document (before or after replace) or None

        Example:
            >>> old_user = await User.find_one_and_replace(
            ...     User.email == "old@email.com",
            ...     User(email="new@email.com", name="Updated User"),
            ...     return_document="before",
            ... )
        """
        from . import _engine

        collection_name = cls.__collection_name__()

        # Build filter
        if isinstance(filter, QueryExpr):
            filter_doc = filter.to_filter()
        else:
            filter_doc = filter

        # Get replacement data
        if isinstance(replacement, Document):
            replacement_doc = replacement.to_dict()
            replacement_doc.pop("_id", None)
        else:
            replacement_doc = dict(replacement)
            replacement_doc.pop("_id", None)

        result = await _engine.find_one_and_replace(
            collection_name, filter_doc, replacement_doc, return_document, upsert
        )

        if result is None:
            return None
        return cls._from_db(result, validate=False)  # Database data is already valid

    @classmethod
    async def find_one_and_delete(
        cls: Type[T],
        filter: QueryExpr | dict,
        sort: Optional[dict] = None,
    ) -> Optional[T]:
        """
        Find a document and delete it atomically.

        Args:
            filter: Query filter
            sort: Sort specification to determine which doc if multiple match

        Returns:
            The deleted document or None

        Example:
            >>> # Pop the oldest item from a queue
            >>> task = await Task.find_one_and_delete(
            ...     Task.status == "pending",
            ...     sort={"created_at": 1},  # Oldest first
            ... )
        """
        from . import _engine

        collection_name = cls.__collection_name__()

        # Build filter
        if isinstance(filter, QueryExpr):
            filter_doc = filter.to_filter()
        else:
            filter_doc = filter

        result = await _engine.find_one_and_delete(collection_name, filter_doc, sort)

        if result is None:
            return None
        return cls._from_db(result, validate=False)  # Database data is already valid

    # ===================
    # Bulk Operations
    # ===================

    @classmethod
    async def insert_many(
        cls: Type[T],
        documents: List[Union[T, dict]],
        validate: bool = False,
        return_type: str = "ids",
    ) -> Union[List[str], List[T]]:
        """
        Insert multiple documents.

        Supports both Document instances and raw dicts. When using raw dicts
        with validate=False (default), documents bypass validation for maximum
        performance (5-10x faster for bulk inserts).

        Args:
            documents: List of Document instances or raw dicts
            validate: If True, validate dicts against model schema before insert.
                     If False (default), skip validation for speed.
                     Document instances are always validated on construction.
            return_type: "ids" returns List[str] of ObjectIds (default, fast).
                        "documents" returns List[T] of Document instances.

        Returns:
            List of ObjectIds (str) or Document instances based on return_type

        Example:
            >>> # Standard usage with Document instances
            >>> users = [
            ...     User(email="alice@example.com", name="Alice"),
            ...     User(email="bob@example.com", name="Bob"),
            ... ]
            >>> ids = await User.insert_many(users)

            >>> # Fast path with raw dicts (5-10x faster)
            >>> dicts = [
            ...     {"email": "alice@example.com", "name": "Alice"},
            ...     {"email": "bob@example.com", "name": "Bob"},
            ... ]
            >>> ids = await User.insert_many(dicts)

            >>> # Get Document instances back
            >>> docs = await User.insert_many(dicts, return_type="documents")
        """
        from . import _engine

        # Handle empty list
        if not documents:
            return [] if return_type == "ids" else []

        collection_name = cls.__collection_name__()

        # Track original dicts for return_type="documents"
        original_dicts: List[Optional[dict]] = []

        # Check if all items are dicts (fast path eligible)
        all_dicts = all(isinstance(d, dict) for d in documents)

        if all_dicts:
            # Fast path: raw dicts
            if validate:
                # Validate each dict against model schema
                for d in documents:
                    cls(**d)  # Raises ValidationError if invalid
            docs = documents  # type: ignore
            original_dicts = list(documents)  # type: ignore
        else:
            # Standard path: mixed or all Documents
            docs = []
            for doc in documents:
                if isinstance(doc, dict):
                    if validate:
                        cls(**doc)  # Validate dict
                    docs.append(doc)
                    original_dicts.append(doc)
                else:
                    docs.append(doc.to_dict())
                    original_dicts.append(None)  # Already a Document

        ids = await _engine.insert_many(collection_name, docs)

        # Update _id on Document instances (not dicts)
        for doc, doc_id in zip(documents, ids):
            if hasattr(doc, '_id'):
                doc._id = doc_id

        # Return based on return_type
        if return_type == "ids":
            return ids
        else:  # return_type == "documents"
            result: List[T] = []
            for doc, doc_id, orig_dict in zip(documents, ids, original_dicts):
                if isinstance(doc, cls):
                    result.append(doc)
                else:
                    # Create Document from dict + id using fast path
                    instance = cls._from_db({**orig_dict, "_id": doc_id}, validate=False)  # type: ignore
                    result.append(instance)
            return result

    @classmethod
    async def delete_many(cls: Type[T], *filters: QueryExpr | dict) -> int:
        """
        Delete multiple documents matching the filters.

        Args:
            *filters: QueryExpr objects or dict filters

        Returns:
            Number of deleted documents

        Example:
            >>> deleted = await User.delete_many(User.status == "deleted")
        """
        return await cls.find(*filters).delete()

    @classmethod
    async def update_many(
        cls: Type[T],
        filter: QueryExpr | dict,
        update: dict,
    ) -> int:
        """
        Update multiple documents.

        Args:
            filter: QueryExpr or dict filter
            update: Update operations

        Returns:
            Number of modified documents

        Example:
            >>> modified = await User.update_many(
            ...     User.status == "pending",
            ...     {"$set": {"status": "active"}}
            ... )
        """
        return await cls.find(filter).update(update)

    @classmethod
    async def bulk_write(
        cls: Type[T],
        operations: List[Any],
        ordered: bool = True,
    ) -> "BulkWriteResult":
        """
        Execute bulk write operations with fluent chainable API.

        All BSON serialization happens in Rust for maximum performance.

        Args:
            operations: List of bulk operation objects (UpdateOne, UpdateMany,
                       InsertOne, DeleteOne, DeleteMany, ReplaceOne)
            ordered: If True, stop on first error. If False, continue with remaining ops.

        Returns:
            BulkWriteResult with counts of inserted, matched, modified, deleted, upserted

        Example:
            >>> from cclab.mongo import UpdateOne, UpdateMany, InsertOne, DeleteOne
            >>>
            >>> result = await User.bulk_write([
            ...     # Fluent update operations
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
            ...     # Insert new document
            ...     InsertOne(User(name="Alice", email="alice@example.com")),
            ...
            ...     # Delete operations
            ...     DeleteOne(User.id == old_id),
            ...     DeleteMany(User.status == "deleted"),
            ... ])
            >>>
            >>> print(f"Modified: {result.modified_count}")
        """
        from . import _engine
        from .bulk import BulkWriteResult

        collection_name = cls.__collection_name__()

        # Convert operation objects to dicts
        op_dicts = [op.to_dict() for op in operations]

        result = await _engine.bulk_write(collection_name, op_dicts, ordered)

        return BulkWriteResult(
            inserted_count=result["inserted_count"],
            matched_count=result["matched_count"],
            modified_count=result["modified_count"],
            deleted_count=result["deleted_count"],
            upserted_count=result["upserted_count"],
            upserted_ids=result.get("upserted_ids", {}),
        )

    # ===================
    # Index Management
    # ===================

    @classmethod
    async def create_index(
        cls,
        keys: List[tuple],
        unique: bool = False,
        sparse: bool = False,
        name: Optional[str] = None,
        background: bool = False,
        expire_after_seconds: Optional[int] = None,
        **kwargs: Any,
    ) -> str:
        """
        Create an index on the collection.

        Args:
            keys: List of (field_name, direction) tuples.
                  Direction can be 1 (ascending), -1 (descending),
                  "text", "2dsphere", "hashed"
            unique: If True, create a unique index
            sparse: If True, only index documents with the field
            name: Optional custom index name
            background: Build index in background (deprecated in MongoDB 4.2+)
            expire_after_seconds: TTL in seconds for TTL indexes
            **kwargs: Additional index options

        Returns:
            The name of the created index

        Example:
            >>> # Simple ascending index
            >>> await User.create_index([("email", 1)], unique=True)
            >>>
            >>> # Compound index
            >>> await User.create_index([("status", 1), ("created_at", -1)])
            >>>
            >>> # Text index
            >>> await User.create_index([("content", "text")])
            >>>
            >>> # TTL index
            >>> await User.create_index([("expires_at", 1)], expire_after_seconds=3600)
        """
        from . import _engine

        collection_name = cls.__collection_name__()

        options = {}
        if unique:
            options["unique"] = True
        if sparse:
            options["sparse"] = True
        if name:
            options["name"] = name
        if background:
            options["background"] = True
        if expire_after_seconds is not None:
            options["expire_after_seconds"] = expire_after_seconds
        options.update(kwargs)

        return await _engine.create_index(collection_name, keys, **options)

    @classmethod
    async def list_indexes(cls) -> List[Dict[str, Any]]:
        """
        List all indexes on the collection.

        Returns:
            List of index specifications, each containing:
            - name: Index name
            - key: Fields and their directions
            - unique: Whether index is unique
            - sparse: Whether index is sparse
            - And other index options

        Example:
            >>> indexes = await User.list_indexes()
            >>> for idx in indexes:
            ...     print(f"{idx['name']}: {idx['key']}")
            _id_: {'_id': 1}
            email_1: {'email': 1}
        """
        from . import _engine

        collection_name = cls.__collection_name__()
        return await _engine.list_indexes(collection_name)

    @classmethod
    async def drop_index(cls, index_name: str) -> None:
        """
        Drop an index from the collection.

        Args:
            index_name: Name of the index to drop.
                       Cannot drop the default _id_ index.

        Raises:
            Exception: If the index doesn't exist or cannot be dropped

        Example:
            >>> await User.drop_index("email_1")
        """
        from . import _engine

        collection_name = cls.__collection_name__()
        return await _engine.drop_index(collection_name, index_name)

    @classmethod
    async def ensure_indexes(cls) -> List[str]:
        """
        Create indexes from Indexed() field annotations.

        This method inspects the class's type annotations for Indexed()
        fields and creates the corresponding indexes in MongoDB.

        Returns:
            List of created index names

        Example:
            >>> class User(Document):
            ...     email: Indexed(str, unique=True)
            ...     username: Indexed(str)
            ...
            >>> created = await User.ensure_indexes()
            >>> print(created)  # ['email_1', 'username_1']
        """
        from . import _engine
        from .types import get_index_fields

        collection_name = cls.__collection_name__()
        indexed_fields = get_index_fields(cls)

        created_indexes = []
        for field_name, index_model in indexed_fields.items():
            keys, options = index_model.to_index_spec(field_name)
            index_name = await _engine.create_index(collection_name, keys, **options)
            created_indexes.append(index_name)

        return created_indexes

    # ===================
    # Time-Series Collections (MongoDB 5.0+)
    # ===================

    @classmethod
    def is_timeseries(cls) -> bool:
        """
        Check if this document type is a time-series collection.

        Returns:
            True if Settings.timeseries is configured

        Example:
            >>> if SensorReading.is_timeseries():
            ...     await SensorReading.ensure_timeseries_collection()
        """
        return cls._timeseries_config is not None

    @classmethod
    async def ensure_timeseries_collection(cls) -> bool:
        """
        Create the time-series collection if it doesn't exist.

        This method should be called during application startup for time-series
        document types. MongoDB time-series collections must be created with
        special options before documents can be inserted.

        Returns:
            True if collection was created, False if it already exists

        Raises:
            ValueError: If this document type doesn't have timeseries configured

        Example:
            >>> class SensorReading(Document):
            ...     sensor_id: str
            ...     timestamp: datetime
            ...     value: float
            ...
            ...     class Settings:
            ...         name = "sensor_readings"
            ...         timeseries = TimeSeriesConfig(
            ...             time_field="timestamp",
            ...             meta_field="sensor_id",
            ...             granularity=Granularity.minutes,
            ...         )
            ...
            >>> # Call during app startup
            >>> await SensorReading.ensure_timeseries_collection()
            >>>
            >>> # Then insert data as normal
            >>> reading = SensorReading(
            ...     sensor_id="sensor-001",
            ...     timestamp=datetime.now(),
            ...     value=23.5,
            ... )
            >>> await reading.save()
        """
        if cls._timeseries_config is None:
            raise ValueError(
                f"{cls.__name__} does not have timeseries configured in Settings. "
                "Add a TimeSeriesConfig to Settings.timeseries to use this method."
            )

        from . import _engine

        collection_name = cls.__collection_name__()
        options = cls._timeseries_config.to_create_options()

        return await _engine.create_collection(collection_name, options)

    @classmethod
    def get_timeseries_config(cls) -> Optional[Any]:
        """
        Get the TimeSeriesConfig for this document type.

        Returns:
            TimeSeriesConfig instance or None if not a time-series collection

        Example:
            >>> config = SensorReading.get_timeseries_config()
            >>> if config:
            ...     print(f"Time field: {config.time_field}")
        """
        return cls._timeseries_config

    # ===================
    # Dunder Methods
    # ===================

    def __repr__(self) -> str:
        cls_name = type(self).__name__
        return f"{cls_name}(id={self._id!r}, data={self._data!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Document):
            return False
        if type(self) != type(other):
            return False
        return self._id == other._id and self._data == other._data

    def __hash__(self) -> int:
        return hash((type(self).__name__, self._id))
