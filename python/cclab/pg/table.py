"""Table base class for PostgreSQL ORM.

This module provides:
- TableMeta: Metaclass that creates ColumnProxy attributes for query syntax
- Table: Base class for all PostgreSQL tables with CRUD operations

Example:
    >>> from cclab.pg import Table, Column
    >>>
    >>> class User(Table):
    ...     email: str = Column(unique=True)
    ...     name: str
    ...     age: int = 0
    ...
    ...     class Settings:
    ...         table_name = "users"
    ...         schema = "public"
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

from typing import Any, ClassVar, Dict, List, Optional, Type, TypeVar, Union

from .columns import ColumnProxy, SqlExpr
from .query import QueryBuilder
from .relationships import RelationshipDescriptor

# Import from Rust engine when available
try:
    from ..cclab import titan as _titan
except ImportError:
    _engine = None


T = TypeVar("T", bound="Table")


class Settings:
    """
    Default settings for Table classes.

    Override in your Table subclass to configure table name, schema, indexes, etc.

    Example:
        >>> class User(Table):
        ...     email: str
        ...
        ...     class Settings:
        ...         table_name = "users"
        ...         schema = "public"
        ...         indexes = [
        ...             {"columns": ["email"], "unique": True},
        ...         ]
    """

    table_name: str = ""  # Table name (defaults to class name lowercase)
    schema: str = "public"  # PostgreSQL schema
    indexes: List[dict] = []  # Index definitions
    primary_key: str = "id"  # Primary key column name


class TableMeta(type):
    """
    Metaclass for Table classes.

    This metaclass:
    1. Collects field annotations from the class
    2. Creates ColumnProxy attributes for each column (enabling User.email syntax)
    3. Processes the Settings inner class
    4. Sets up the table name and schema

    Example:
        >>> class User(Table):
        ...     email: str  # Creates User.email as ColumnProxy
        ...     name: str   # Creates User.name as ColumnProxy
        ...
        >>> User.email  # ColumnProxy("email", User)
        >>> User.email == "test"  # SqlExpr("email", "=", "test")
    """

    def __new__(
        mcs,
        name: str,
        bases: tuple,
        namespace: dict,
        **kwargs: Any,
    ) -> "TableMeta":
        # Create the class first
        cls = super().__new__(mcs, name, bases, namespace)

        # Skip processing for the base Table class itself
        if name == "Table" and not bases:
            return cls

        # Get all annotations including from parent classes
        annotations = {}
        for base in reversed(cls.__mro__):
            if hasattr(base, "__annotations__"):
                annotations.update(base.__annotations__)

        # Store column names for later use
        cls._columns: Dict[str, type] = {}

        # Store default values before replacing with ColumnProxy
        cls._column_defaults: Dict[str, Any] = {}

        # Store relationships for lazy loading
        cls._relationships: Dict[str, RelationshipDescriptor] = {}

        # Create ColumnProxy for each column annotation
        for column_name, column_type in annotations.items():
            # Skip private attributes and ClassVar
            if column_name.startswith("_"):
                continue
            if hasattr(column_type, "__origin__") and column_type.__origin__ is ClassVar:
                continue

            # Store column info
            cls._columns[column_name] = column_type

            # Capture default value BEFORE replacing with ColumnProxy
            current_value = getattr(cls, column_name, None)

            # Skip if it's a RelationshipDescriptor (lazy loading)
            if isinstance(current_value, RelationshipDescriptor):
                continue

            if current_value is not None and not isinstance(current_value, ColumnProxy):
                cls._column_defaults[column_name] = current_value

            # Create ColumnProxy only if not already set (allows override)
            if not isinstance(current_value, ColumnProxy):
                setattr(cls, column_name, ColumnProxy(column_name, cls))

        # Register RelationshipDescriptor instances
        for attr_name in dir(cls):
            # Skip private attributes and methods
            if attr_name.startswith("_"):
                continue

            attr_value = getattr(cls, attr_name)
            if isinstance(attr_value, RelationshipDescriptor):
                cls._relationships[attr_name] = attr_value

        # Process Settings class
        settings_cls = namespace.get("Settings", Settings)
        cls._settings = settings_cls

        # Set table name (default to lowercase class name)
        if hasattr(settings_cls, "table_name") and settings_cls.table_name:
            cls._table_name = settings_cls.table_name
        else:
            cls._table_name = name.lower()

        # Set schema
        cls._schema = getattr(settings_cls, "schema", "public")

        # Set primary key
        cls._primary_key = getattr(settings_cls, "primary_key", "id")

        return cls


class Table(metaclass=TableMeta):
    """
    Base class for PostgreSQL tables.

    Provides:
    - CRUD operations (save, delete, refresh)
    - Class methods for querying (find, find_one, get)
    - Type-safe query syntax (User.email == "x")
    - Automatic primary key handling

    Example:
        >>> from cclab.pg import Table, Column
        >>>
        >>> class User(Table):
        ...     email: str = Column(unique=True)
        ...     name: str
        ...     age: int = 0
        ...
        ...     class Settings:
        ...         table_name = "users"
        ...
        >>> # Create and save
        >>> user = User(email="alice@example.com", name="Alice")
        >>> await user.save()
        >>> print(user.id)  # 1
        >>>
        >>> # Query
        >>> user = await User.find_one(User.email == "alice@example.com")
        >>> users = await User.find(User.age > 25).to_list()
        >>>
        >>> # Update
        >>> user.age = 31
        >>> await user.save()
        >>>
        >>> # Delete
        >>> await user.delete()
    """

    # Class-level attributes (set by metaclass)
    _columns: ClassVar[Dict[str, type]] = {}
    _column_defaults: ClassVar[Dict[str, Any]] = {}
    _relationships: ClassVar[Dict[str, RelationshipDescriptor]] = {}
    _settings: ClassVar[Type[Settings]] = Settings
    _table_name: ClassVar[str] = ""
    _schema: ClassVar[str] = "public"
    _primary_key: ClassVar[str] = "id"

    # Instance attributes
    id: Optional[int] = None  # Primary key
    _data: Dict[str, Any]

    def __init__(self, **kwargs: Any) -> None:
        """
        Initialize a table row instance.

        Args:
            **kwargs: Column values for the row

        Example:
            >>> user = User(email="alice@example.com", name="Alice", age=30)
        """
        # Extract id if provided
        self.id = kwargs.pop("id", None)
        if self.id is not None:
            self.id = int(self.id)

        # Store all data
        self._data = kwargs.copy()

        # Set default values for columns not provided
        for column_name, column_type in self._columns.items():
            if column_name not in self._data and column_name != "id":
                # Check for stored default value
                if column_name in self._column_defaults:
                    default = self._column_defaults[column_name]
                    # Check if it's a Column with default_factory
                    if hasattr(default, "default_factory") and default.default_factory:
                        self._data[column_name] = default.default_factory()
                    elif hasattr(default, "default") and default.default is not None:
                        self._data[column_name] = default.default
                    else:
                        # It's a plain default value (like age: int = 0)
                        self._data[column_name] = default

    def __getattr__(self, name: str) -> Any:
        """Get attribute from _data if not in __dict__."""
        if name.startswith("_"):
            raise AttributeError(f"'{type(self).__name__}' object has no attribute '{name}'")
        if "_data" in self.__dict__ and name in self._data:
            return self._data[name]
        raise AttributeError(f"'{type(self).__name__}' object has no attribute '{name}'")

    def __setattr__(self, name: str, value: Any) -> None:
        """Set attribute in _data for non-private attributes."""
        if name.startswith("_") or name == "id":
            super().__setattr__(name, value)
        else:
            # Check if this is a descriptor with __set__ method
            # This allows hybrid_property setters, computed columns to raise errors, etc.
            # We need to check __dict__ directly to avoid calling __get__ which may
            # return a SQL expression for hybrid properties
            for cls in type(self).__mro__:
                if name in cls.__dict__:
                    descriptor = cls.__dict__[name]
                    if hasattr(descriptor, "__set__"):
                        # Call the descriptor's __set__ method
                        # If it raises an error, let it propagate
                        descriptor.__set__(self, value)
                        return
                    break

            # Normal attribute - store in _data
            if "_data" not in self.__dict__:
                self.__dict__["_data"] = {}
            self._data[name] = value

    def to_dict(self) -> Dict[str, Any]:
        """
        Convert the table row to a dictionary.

        Returns:
            Dictionary of column values

        Example:
            >>> user = User(email="test@example.com", name="Test")
            >>> user.to_dict()
            {"email": "test@example.com", "name": "Test"}
        """
        result = self._data.copy()
        if self.id is not None:
            result["id"] = self.id
        return result

    @classmethod
    def __table_name__(cls) -> str:
        """Get the full table name with schema."""
        return f"{cls._schema}.{cls._table_name}"

    async def save(self) -> int:
        """
        Save the row to PostgreSQL.

        If the row has an id, updates the existing row.
        Otherwise, inserts a new row.

        Returns:
            The row's primary key value

        Example:
            >>> user = User(email="alice@example.com", name="Alice")
            >>> user_id = await user.save()
            >>> print(user_id)  # 1
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        table_name = self.__table_name__()
        is_insert = self.id is None

        data = self.to_dict()

        if self.id:
            # Update existing
            pk_value = data.pop("id")
            result_id = await _titan.update_one(
                table_name,
                self._primary_key,
                pk_value,
                data,
            )
        else:
            # Insert new
            result_id = await _titan.insert_one(table_name, data)
            self.id = result_id

        return result_id

    async def delete(self) -> bool:
        """
        Delete this row from the database.

        Returns:
            True if deleted, False if not found

        Example:
            >>> user = await User.get(1)
            >>> await user.delete()
            True
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        if self.id is None:
            return False

        table_name = self.__table_name__()
        result = await _titan.delete_one(
            table_name,
            self._primary_key,
            self.id,
        )
        return result > 0

    async def refresh(self) -> None:
        """
        Reload this row from the database.

        Example:
            >>> user = await User.get(1)
            >>> # ... row is updated by another process ...
            >>> await user.refresh()  # Reload from database
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        if self.id is None:
            raise ValueError("Cannot refresh a row without an id")

        table_name = self.__table_name__()
        data = await _titan.find_one(
            table_name,
            self._primary_key,
            self.id,
        )

        if data is None:
            raise ValueError(f"Row with id {self.id} not found")

        # Update instance data
        self.id = data.pop("id", self.id)
        self._data = data

    @classmethod
    def find(
        cls: Type[T],
        *filters: SqlExpr | dict,
        fetch_links: bool = False,
    ) -> QueryBuilder[T]:
        """
        Create a query to find rows matching filters.

        Args:
            *filters: Query expressions or filter dictionaries
            fetch_links: If True, automatically eager-load all relationships
                        using selectinload to prevent N+1 queries

        Returns:
            QueryBuilder for chaining operations

        Example:
            >>> # Find all users
            >>> users = await User.find().to_list()
            >>>
            >>> # Find with filter
            >>> users = await User.find(User.age > 25).to_list()
            >>>
            >>> # Multiple filters (AND)
            >>> users = await User.find(
            ...     User.age > 25,
            ...     User.email.like("%@example.com")
            ... ).to_list()
            >>>
            >>> # Eager-load all relationships (prevents N+1 queries)
            >>> users = await User.find(fetch_links=True).to_list()
            >>> # Equivalent to:
            >>> # User.find().options(selectinload("posts"), selectinload("profile"))
        """
        query = QueryBuilder(cls, filters)

        # If fetch_links=True, automatically apply selectinload for all relationships
        if fetch_links and cls._relationships:
            from .options import selectinload
            options = [selectinload(rel_name) for rel_name in cls._relationships.keys()]
            query = query.options(*options)

        return query

    @classmethod
    async def find_one(
        cls: Type[T],
        *filters: SqlExpr | dict,
        fetch_links: bool = False,
    ) -> Optional[T]:
        """
        Find one row matching filters.

        Args:
            *filters: Query expressions or filter dictionaries
            fetch_links: If True, automatically eager-load all relationships
                        using selectinload to prevent N+1 queries

        Returns:
            Table instance or None if not found

        Example:
            >>> user = await User.find_one(User.email == "alice@example.com")
            >>> if user:
            ...     print(user.name)
            >>>
            >>> # With eager-loaded relationships
            >>> user = await User.find_one(User.id == 1, fetch_links=True)
            >>> posts = await user.posts  # Already loaded, no extra query
        """
        return await cls.find(*filters, fetch_links=fetch_links).first()

    @classmethod
    async def get(cls: Type[T], row_id: int) -> Optional[T]:
        """
        Get a row by primary key.

        Args:
            row_id: Primary key value

        Returns:
            Table instance or None if not found

        Example:
            >>> user = await User.get(1)
            >>> if user:
            ...     print(user.email)
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        table_name = cls.__table_name__()
        data = await _titan.find_one(
            table_name,
            cls._primary_key,
            row_id,
        )

        if data is None:
            return None

        return cls(**data)

    @classmethod
    async def count(cls: Type[T], *filters: SqlExpr | dict) -> int:
        """
        Count rows matching filters.

        Args:
            *filters: Query expressions or filter dictionaries

        Returns:
            Number of matching rows

        Example:
            >>> total = await User.count()
            >>> adults = await User.count(User.age >= 18)
        """
        return await cls.find(*filters).count()

    @classmethod
    async def insert_many(cls: Type[T], rows: List[Dict[str, Any] | T]) -> List[int]:
        """
        Insert multiple rows in a single operation.

        Args:
            rows: List of dictionaries with column values or Table instances

        Returns:
            List of primary key values for inserted rows

        Example:
            >>> # Using dictionaries
            >>> ids = await User.insert_many([
            ...     {"email": "alice@example.com", "name": "Alice"},
            ...     {"email": "bob@example.com", "name": "Bob"},
            ... ])
            >>> print(ids)  # [1, 2]
            >>>
            >>> # Using Table instances
            >>> users = [User(email="alice@example.com", name="Alice"),
            ...          User(email="bob@example.com", name="Bob")]
            >>> ids = await User.insert_many(users)
            >>> print(ids)  # [1, 2]
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        table_name = cls.__table_name__()

        # Convert Table instances to dictionaries
        converted_rows = []
        for row in rows:
            if isinstance(row, cls):
                converted_rows.append(row.to_dict())
            elif isinstance(row, dict):
                converted_rows.append(row)
            else:
                raise TypeError(
                    f"Expected dict or {cls.__name__} instance, got {type(row).__name__}"
                )

        return await _titan.insert_many(table_name, converted_rows)

    @classmethod
    def _build_where_clause(cls, filters: tuple) -> tuple[str, list[Any]]:
        """
        Build WHERE clause from filters.

        Args:
            filters: Tuple of SqlExpr or dict filters

        Returns:
            Tuple of (where_clause, parameters)
        """
        if not filters:
            return ("", [])

        # Convert filters to SQL
        conditions = []
        params = []
        param_index = 1

        for filter_item in filters:
            if isinstance(filter_item, SqlExpr):
                sql, filter_params = filter_item.to_sql(param_index)
                conditions.append(sql)
                params.extend(filter_params)
                param_index += len(filter_params)
            elif isinstance(filter_item, dict):
                # Convert dict to SQL conditions
                for key, value in filter_item.items():
                    conditions.append(f"{key} = ${param_index}")
                    params.append(value)
                    param_index += 1
            else:
                raise TypeError(f"Invalid filter type: {type(filter_item)}")

        where_clause = " AND ".join(conditions) if conditions else ""
        return (where_clause, params)

    @classmethod
    async def delete_many(
        cls: Type[T],
        *filters: SqlExpr | dict,
        returning: Optional[List[str]] = None,
    ) -> Union[int, List[Dict[str, Any]]]:
        """
        Delete multiple rows matching filters.

        Args:
            *filters: Query expressions or filter dictionaries
            returning: Optional list of column names to return (default: None)

        Returns:
            If returning is None: Number of deleted rows (int)
            If returning is Some: List of dicts with returned column values

        Example:
            >>> deleted = await User.delete_many(User.age < 18)
            >>> print(f"Deleted {deleted} users")
            >>>
            >>> # With RETURNING clause
            >>> results = await User.delete_many(
            ...     User.age < 18,
            ...     returning=["id", "name", "age"]
            ... )
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        table_name = cls.__table_name__()

        # Convert filters to SQL using the same logic as QueryBuilder
        where_clause, params = cls._build_where_clause(filters)

        return await _titan.delete_many(table_name, where_clause, params, returning)

    @classmethod
    async def update_many(
        cls: Type[T],
        updates: Dict[str, Any],
        *filters: SqlExpr | dict,
        returning: Optional[List[str]] = None,
    ) -> Union[int, List[Dict[str, Any]]]:
        """
        Update multiple rows matching filters.

        Args:
            updates: Dictionary of column values to set
            *filters: Query expressions or filter dictionaries
            returning: Optional list of column names to return (default: None)

        Returns:
            If returning is None: Number of updated rows (int)
            If returning is Some: List of dicts with returned column values

        Example:
            >>> updated = await User.update_many(
            ...     {"status": "active"},
            ...     User.email.like("%@example.com")
            ... )
            >>> print(f"Updated {updated} users")
            >>>
            >>> # With RETURNING clause
            >>> results = await User.update_many(
            ...     {"status": "active"},
            ...     User.email.like("%@example.com"),
            ...     returning=["id", "name", "status"]
            ... )
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        table_name = cls.__table_name__()

        # Convert filters to SQL using the same logic as QueryBuilder
        where_clause, params = cls._build_where_clause(filters)

        return await _titan.update_many(table_name, updates, where_clause, params, returning)
