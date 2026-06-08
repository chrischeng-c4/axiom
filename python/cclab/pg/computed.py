"""Computed attributes for PostgreSQL tables.

This module provides SQLAlchemy-style computed attributes:
- @hybrid_property: Works as Python property AND generates SQL
- @hybrid_method: Like hybrid_property but for methods with arguments
- column_property(): Read-only computed column from SQL expression
- Computed: PostgreSQL GENERATED ALWAYS AS columns
- default_factory(): Column default from callable

Example:
    >>> from cclab.pg import Table, Column
    >>> from cclab.pg.computed import hybrid_property, Computed
    >>>
    >>> class User(Table):
    ...     first_name: str
    ...     last_name: str
    ...     age: int
    ...
    ...     @hybrid_property
    ...     def full_name(self):
    ...         return f"{self.first_name} {self.last_name}"
    ...
    ...     @full_name.expression
    ...     def full_name(cls):
    ...         # Return SQL expression for queries
    ...         from .columns import SqlExpr
    ...         return SqlExpr("first_name || ' ' || last_name", "=", None)
    >>>
    >>> class Product(Table):
    ...     price: float
    ...     quantity: int
    ...     # Stored computed column
    ...     total = Computed("price * quantity", stored=True)
"""

from __future__ import annotations

from typing import Any, Callable, Optional, TYPE_CHECKING, TypeVar

if TYPE_CHECKING:
    from .table import Table

__all__ = [
    "hybrid_property",
    "hybrid_method",
    "column_property",
    "Computed",
    "default_factory",
    "HybridPropertyDescriptor",
    "HybridMethodDescriptor",
    "ColumnPropertyDescriptor",
    "ComputedColumn",
]


T = TypeVar("T")
F = TypeVar("F", bound=Callable[..., Any])


class HybridPropertyDescriptor:
    """
    Descriptor that handles both instance and class access for hybrid properties.

    On instance access: calls Python getter function
    On class access: returns SQL expression builder

    Example:
        >>> class User(Table):
        ...     first_name: str
        ...     last_name: str
        ...
        ...     @hybrid_property
        ...     def full_name(self):
        ...         return f"{self.first_name} {self.last_name}"
        ...
        ...     @full_name.expression
        ...     def full_name(cls):
        ...         from .columns import SqlExpr
        ...         return SqlExpr("first_name || ' ' || last_name", "RAW", None)
        >>>
        >>> # Instance access (Python)
        >>> user = User(first_name="Alice", last_name="Smith")
        >>> user.full_name  # "Alice Smith"
        >>>
        >>> # Class access (SQL)
        >>> User.full_name  # Returns SQL expression
    """

    def __init__(self, fget: Callable[[Any], Any]) -> None:
        """
        Initialize hybrid property descriptor.

        Args:
            fget: Python getter function
        """
        self.fget = fget
        self.fset: Optional[Callable[[Any, Any], None]] = None
        self.expr: Optional[Callable[[type], Any]] = None
        self._name: Optional[str] = None
        # Copy function metadata
        self.__doc__ = fget.__doc__
        self.__name__ = fget.__name__

    def __set_name__(self, owner: type, name: str) -> None:
        """Called when descriptor is assigned to class attribute."""
        self._name = name

    def __get__(self, obj: Any, objtype: Optional[type] = None) -> Any:
        """
        Get value for instance or SQL expression for class.

        Args:
            obj: Instance or None for class access
            objtype: Owner class

        Returns:
            Value from getter (instance) or SQL expression (class)
        """
        if obj is None:
            # Class access - return SQL expression if available
            if self.expr is not None:
                return self.expr(objtype)
            # No SQL expression - return self for introspection
            return self
        # Instance access - call Python getter
        return self.fget(obj)

    def __set__(self, obj: Any, value: Any) -> None:
        """
        Set value on instance.

        Args:
            obj: Instance
            value: Value to set

        Raises:
            AttributeError: If no setter is defined
        """
        if self.fset is None:
            raise AttributeError(f"can't set attribute '{self._name}'")
        self.fset(obj, value)

    def setter(self, fset: Callable[[Any, Any], None]) -> "HybridPropertyDescriptor":
        """
        Decorator to add setter function.

        Args:
            fset: Setter function

        Returns:
            Self for chaining

        Example:
            >>> @hybrid_property
            ... def full_name(self):
            ...     return f"{self.first_name} {self.last_name}"
            ...
            >>> @full_name.setter
            ... def full_name(self, value):
            ...     parts = value.split(" ", 1)
            ...     self.first_name = parts[0]
            ...     self.last_name = parts[1] if len(parts) > 1 else ""
        """
        self.fset = fset
        return self

    def expression(self, expr: Callable[[type], Any]) -> "HybridPropertyDescriptor":
        """
        Decorator to add SQL expression callable.

        Args:
            expr: Function that takes class and returns SQL expression

        Returns:
            Self for chaining

        Example:
            >>> @hybrid_property
            ... def full_name(self):
            ...     return f"{self.first_name} {self.last_name}"
            ...
            >>> @full_name.expression
            ... def full_name(cls):
            ...     from .columns import SqlExpr
            ...     return SqlExpr("first_name || ' ' || last_name", "RAW", None)
        """
        self.expr = expr
        return self

    def __repr__(self) -> str:
        return f"<hybrid_property {self.__name__}>"


class HybridMethodDescriptor:
    """
    Descriptor for hybrid methods (methods with arguments).

    Like hybrid_property but supports method arguments.

    Example:
        >>> class User(Table):
        ...     age: int
        ...
        ...     @hybrid_method
        ...     def is_older_than(self, min_age):
        ...         return self.age > min_age
        ...
        ...     @is_older_than.expression
        ...     def is_older_than(cls, min_age):
        ...         from .columns import SqlExpr
        ...         return SqlExpr("age", ">", min_age)
        >>>
        >>> # Instance access
        >>> user = User(age=30)
        >>> user.is_older_than(25)  # True
        >>>
        >>> # Class access for queries
        >>> expr = User.is_older_than(25)  # SqlExpr("age", ">", 25)
    """

    def __init__(self, fget: Callable[..., Any]) -> None:
        """
        Initialize hybrid method descriptor.

        Args:
            fget: Python method function
        """
        self.fget = fget
        self.expr: Optional[Callable[..., Any]] = None
        self._name: Optional[str] = None
        # Copy function metadata
        self.__doc__ = fget.__doc__
        self.__name__ = fget.__name__

    def __set_name__(self, owner: type, name: str) -> None:
        """Called when descriptor is assigned to class attribute."""
        self._name = name

    def __get__(self, obj: Any, objtype: Optional[type] = None) -> Any:
        """
        Return callable bound to instance or class.

        Args:
            obj: Instance or None for class access
            objtype: Owner class

        Returns:
            Bound method for instance or SQL expression callable for class
        """
        if obj is None:
            # Class access - return SQL expression callable if available
            if self.expr is not None:
                return lambda *args, **kwargs: self.expr(objtype, *args, **kwargs)
            # No SQL expression - return instance method wrapper
            return lambda *args, **kwargs: self.fget(objtype, *args, **kwargs)
        # Instance access - return bound method
        return lambda *args, **kwargs: self.fget(obj, *args, **kwargs)

    def expression(self, expr: Callable[..., Any]) -> "HybridMethodDescriptor":
        """
        Decorator to add SQL expression callable.

        Args:
            expr: Function that takes class and arguments, returns SQL expression

        Returns:
            Self for chaining

        Example:
            >>> @hybrid_method
            ... def is_older_than(self, min_age):
            ...     return self.age > min_age
            ...
            >>> @is_older_than.expression
            ... def is_older_than(cls, min_age):
            ...     from .columns import SqlExpr
            ...     return SqlExpr("age", ">", min_age)
        """
        self.expr = expr
        return self

    def __repr__(self) -> str:
        return f"<hybrid_method {self.__name__}>"


class ColumnPropertyDescriptor:
    """
    Read-only descriptor for computed columns.

    The value is computed from SQL expression when the row is loaded.
    This is different from hybrid_property which computes in Python on instance access.

    Example:
        >>> from cclab.pg.computed import column_property
        >>>
        >>> class Order(Table):
        ...     amount: float
        ...     tax_rate: float
        ...     # Computed at SQL level
        ...     total = column_property("amount * (1 + tax_rate)")
        >>>
        >>> order = Order(amount=100.0, tax_rate=0.2)
        >>> await order.save()
        >>> # When loaded from DB, total is already computed
        >>> order = await Order.get(order.id)
        >>> order.total  # 120.0
    """

    def __init__(self, expression: str) -> None:
        """
        Initialize column property descriptor.

        Args:
            expression: SQL expression string
        """
        self.expression = expression
        self._name: Optional[str] = None
        self._cache: dict[int, Any] = {}

    def __set_name__(self, owner: type, name: str) -> None:
        """Called when descriptor is assigned to class attribute."""
        self._name = name

    def __get__(self, obj: Any, objtype: Optional[type] = None) -> Any:
        """
        Get computed value from row data.

        Args:
            obj: Instance or None for class access
            objtype: Owner class

        Returns:
            Computed value from _data or self for class access
        """
        if obj is None:
            # Class access - return self for introspection
            return self

        # Instance access - get from _data
        if hasattr(obj, "_data") and self._name in obj._data:
            return obj._data[self._name]

        # Try to get from cache by instance id
        obj_id = id(obj)
        if obj_id in self._cache:
            return self._cache[obj_id]

        return None

    def __set__(self, obj: Any, value: Any) -> None:
        """
        Setting is not allowed - computed columns are read-only.

        Args:
            obj: Instance
            value: Value (ignored)

        Raises:
            AttributeError: Always, computed columns are read-only
        """
        raise AttributeError(
            f"can't set attribute '{self._name}': column_property is read-only"
        )

    def __repr__(self) -> str:
        return f"<column_property {self.expression!r}>"


class ComputedColumn:
    """
    Represents a PostgreSQL GENERATED ALWAYS AS column.

    These are columns whose values are computed from other columns
    and stored in the database (or computed on read if not stored).

    Example:
        >>> from cclab.pg import Table, Column
        >>> from cclab.pg.computed import Computed
        >>>
        >>> class Product(Table):
        ...     price: float
        ...     quantity: int
        ...     # Stored computed column
        ...     total = Computed("price * quantity", stored=True)
        ...     # Virtual computed column
        ...     tax = Computed("price * 0.1", stored=False)
        >>>
        >>> # The SQL DDL will include:
        >>> # total FLOAT GENERATED ALWAYS AS (price * quantity) STORED
        >>> # tax FLOAT GENERATED ALWAYS AS (price * 0.1)
    """

    def __init__(self, expression: str, stored: bool = True) -> None:
        """
        Initialize computed column.

        Args:
            expression: SQL expression string
            stored: If True, column is STORED on disk. If False, computed on read (VIRTUAL)
        """
        self.expression = expression
        self.stored = stored
        self._name: Optional[str] = None

    def __set_name__(self, owner: type, name: str) -> None:
        """Called when descriptor is assigned to class attribute."""
        self._name = name

    def __get__(self, obj: Any, objtype: Optional[type] = None) -> Any:
        """
        Get computed value from row data.

        Args:
            obj: Instance or None for class access
            objtype: Owner class

        Returns:
            Computed value from _data or self for class access
        """
        if obj is None:
            # Class access - return self for introspection
            return self

        # Instance access - get from _data
        if hasattr(obj, "_data") and self._name in obj._data:
            return obj._data[self._name]

        return None

    def __set__(self, obj: Any, value: Any) -> None:
        """
        Setting is not allowed - computed columns are read-only.

        Args:
            obj: Instance
            value: Value (ignored)

        Raises:
            AttributeError: Always, computed columns are read-only
        """
        raise AttributeError(
            f"can't set attribute '{self._name}': computed columns are read-only"
        )

    def to_sql(self, column_type: str = "FLOAT") -> str:
        """
        Generate SQL DDL for this computed column.

        Args:
            column_type: SQL data type for the column

        Returns:
            SQL column definition string

        Example:
            >>> computed = Computed("price * quantity", stored=True)
            >>> computed.to_sql("DECIMAL(10,2)")
            'DECIMAL(10,2) GENERATED ALWAYS AS (price * quantity) STORED'
        """
        sql = f"{column_type} GENERATED ALWAYS AS ({self.expression})"
        if self.stored:
            sql += " STORED"
        return sql

    def __repr__(self) -> str:
        stored_str = "stored" if self.stored else "virtual"
        return f"<Computed {self.expression!r} ({stored_str})>"


def hybrid_property(fget: F) -> HybridPropertyDescriptor:
    """
    Decorator for hybrid properties that work in Python and SQL.

    Args:
        fget: Python getter function

    Returns:
        HybridPropertyDescriptor

    Example:
        >>> from cclab.pg import Table
        >>> from cclab.pg.computed import hybrid_property
        >>>
        >>> class User(Table):
        ...     first_name: str
        ...     last_name: str
        ...
        ...     @hybrid_property
        ...     def full_name(self):
        ...         return f"{self.first_name} {self.last_name}"
        ...
        ...     @full_name.expression
        ...     def full_name(cls):
        ...         from .columns import SqlExpr
        ...         return SqlExpr("first_name || ' ' || last_name", "RAW", None)
        >>>
        >>> # Instance access
        >>> user = User(first_name="Alice", last_name="Smith")
        >>> user.full_name  # "Alice Smith"
        >>>
        >>> # Class access for queries
        >>> expr = User.full_name  # SQL expression
    """
    return HybridPropertyDescriptor(fget)


def hybrid_method(fget: F) -> HybridMethodDescriptor:
    """
    Decorator for hybrid methods that work in Python and SQL.

    Args:
        fget: Python method function

    Returns:
        HybridMethodDescriptor

    Example:
        >>> from cclab.pg import Table
        >>> from cclab.pg.computed import hybrid_method
        >>>
        >>> class User(Table):
        ...     age: int
        ...
        ...     @hybrid_method
        ...     def is_older_than(self, min_age):
        ...         return self.age > min_age
        ...
        ...     @is_older_than.expression
        ...     def is_older_than(cls, min_age):
        ...         from .columns import SqlExpr
        ...         return SqlExpr("age", ">", min_age)
        >>>
        >>> # Instance access
        >>> user = User(age=30)
        >>> user.is_older_than(25)  # True
        >>>
        >>> # Class access for queries
        >>> expr = User.is_older_than(25)  # SqlExpr("age", ">", 25)
    """
    return HybridMethodDescriptor(fget)


def column_property(expression: str) -> ColumnPropertyDescriptor:
    """
    Create a read-only computed column from SQL expression.

    The value is computed at SQL level when the row is loaded,
    not computed in Python on property access.

    Args:
        expression: SQL expression string

    Returns:
        ColumnPropertyDescriptor

    Example:
        >>> from cclab.pg import Table
        >>> from cclab.pg.computed import column_property
        >>>
        >>> class Order(Table):
        ...     amount: float
        ...     tax_rate: float
        ...     # Loaded from database, not computed in Python
        ...     total = column_property("amount * (1 + tax_rate)")
        >>>
        >>> order = Order(amount=100.0, tax_rate=0.2)
        >>> await order.save()
        >>> # When loaded from DB, total is already computed
        >>> order = await Order.get(order.id)
        >>> order.total  # 120.0
    """
    return ColumnPropertyDescriptor(expression)


def default_factory(factory: Callable[[], T]) -> Callable[[], T]:
    """
    Create a column default from callable.

    The callable is invoked at INSERT time to generate the default value.

    Args:
        factory: Callable that returns default value

    Returns:
        The factory callable (for type checking)

    Example:
        >>> from datetime import datetime
        >>> import uuid
        >>> from cclab.pg import Table, Column
        >>> from cclab.pg.computed import default_factory
        >>>
        >>> class Post(Table):
        ...     title: str
        ...     created_at: datetime = Column(default_factory=default_factory(datetime.utcnow))
        ...     uuid: str = Column(default_factory=default_factory(lambda: str(uuid.uuid4())))
        >>>
        >>> post = Post(title="Hello World")
        >>> await post.save()
        >>> # created_at and uuid are set automatically
        >>> print(post.created_at)  # 2024-01-01 12:00:00
        >>> print(post.uuid)  # "550e8400-e29b-41d4-a716-446655440000"
    """
    return factory


# Alias for Computed (for consistency with SQLAlchemy naming)
Computed = ComputedColumn
