"""SQLAlchemy-style table inheritance patterns for PostgreSQL.

This module provides three inheritance strategies:

1. **Single Table Inheritance (SINGLE_TABLE)**:
   - All subclasses share one table with a discriminator column
   - Fastest queries (no JOINs)
   - May waste space with NULL columns for unused fields

   Example:
       >>> class Employee(Table, SingleTableInheritance):
       ...     name: str
       ...     employee_type: str  # Discriminator column
       ...
       >>> class Manager(Employee):
       ...     __discriminator_value__ = "manager"
       ...     department: str
       ...
       >>> class Engineer(Employee):
       ...     __discriminator_value__ = "engineer"
       ...     programming_language: str

2. **Joined Table Inheritance (JOINED)**:
   - Each class has its own table with FK to parent table
   - No NULL columns (normalized)
   - Requires JOINs for queries

   Example:
       >>> class Employee(Table, JoinedTableInheritance):
       ...     name: str
       ...
       >>> class Manager(Employee):
       ...     department: str  # Stored in managers table

3. **Concrete Table Inheritance (CONCRETE)**:
   - Each class has a complete standalone table
   - No foreign keys or joins
   - Requires UNION for polymorphic queries

   Example:
       >>> class Employee(Table, ConcreteTableInheritance):
       ...     name: str
       ...
       >>> class Manager(Employee):
       ...     name: str  # Duplicated in managers table
       ...     department: str

Usage:
    from cclab.pg import Table, Column
    from cclab.pg.inheritance import (
        SingleTableInheritance,
        InheritanceType,
        inheritance,
    )

    @inheritance(type=InheritanceType.SINGLE_TABLE, discriminator="type")
    class Employee(Table):
        name: str
        type: str  # Discriminator

    class Manager(Employee):
        __discriminator_value__ = "manager"
        department: str
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any, ClassVar, Dict, List, Optional, Type, TypeVar, TYPE_CHECKING

if TYPE_CHECKING:
    from .table import Table

__all__ = [
    "InheritanceType",
    "InheritanceConfig",
    "inheritance",
    "SingleTableInheritance",
    "JoinedTableInheritance",
    "ConcreteTableInheritance",
    "PolymorphicQueryMixin",
    "get_inheritance_type",
    "get_discriminator_column",
    "get_discriminator_value",
    "register_polymorphic_class",
    "get_polymorphic_map",
]


T = TypeVar("T", bound="Table")


class InheritanceType(Enum):
    """
    Type of table inheritance strategy.

    Attributes:
        SINGLE_TABLE: All subclasses share one table with discriminator column
        JOINED: Each class has own table with FK to parent table
        CONCRETE: Each class has complete standalone table
    """

    SINGLE_TABLE = "single_table"
    JOINED = "joined"
    CONCRETE = "concrete"


@dataclass
class InheritanceConfig:
    """
    Configuration for table inheritance.

    Attributes:
        inheritance_type: Type of inheritance strategy
        discriminator_column: Column name for type discrimination (SINGLE_TABLE)
        discriminator_value: Value in discriminator column for this class
        polymorphic_on: Column name to distinguish types (usually same as discriminator_column)

    Example:
        >>> config = InheritanceConfig(
        ...     inheritance_type=InheritanceType.SINGLE_TABLE,
        ...     discriminator_column="type",
        ...     discriminator_value="manager"
        ... )
    """

    inheritance_type: InheritanceType
    discriminator_column: str = "type"
    discriminator_value: Optional[str] = None
    polymorphic_on: Optional[str] = None

    def __post_init__(self) -> None:
        """Set polymorphic_on to discriminator_column if not specified."""
        if self.polymorphic_on is None:
            self.polymorphic_on = self.discriminator_column


# Global registry for polymorphic classes
_polymorphic_registry: Dict[Type, Dict[str, Type]] = {}


def inheritance(
    type: InheritanceType = InheritanceType.SINGLE_TABLE,
    discriminator: str = "type",
    polymorphic_on: Optional[str] = None,
) -> Any:
    """
    Class decorator to configure inheritance strategy.

    Args:
        type: Inheritance strategy to use
        discriminator: Name of discriminator column (SINGLE_TABLE only)
        polymorphic_on: Column to distinguish types (defaults to discriminator)

    Returns:
        Decorator function that configures the class

    Example:
        >>> @inheritance(type=InheritanceType.SINGLE_TABLE, discriminator="type")
        ... class Employee(Table):
        ...     name: str
        ...     type: str
        ...
        >>> @inheritance(type=InheritanceType.JOINED)
        ... class Vehicle(Table):
        ...     make: str
        ...     model: str
    """
    def decorator(cls: Type[T]) -> Type[T]:
        config = InheritanceConfig(
            inheritance_type=type,
            discriminator_column=discriminator,
            polymorphic_on=polymorphic_on or discriminator,
        )
        cls._inheritance_config = config  # type: ignore

        # Initialize polymorphic registry for this base class
        if cls not in _polymorphic_registry:
            _polymorphic_registry[cls] = {}

        return cls

    return decorator


class SingleTableInheritance:
    """
    Base mixin for single table inheritance.

    All subclasses share one physical table. A discriminator column
    distinguishes between different types.

    Pros:
        - Fastest queries (no JOINs)
        - Simple schema

    Cons:
        - NULL columns for unused fields
        - All columns must be nullable (except discriminator)

    Example:
        >>> class Employee(Table, SingleTableInheritance):
        ...     name: str
        ...     employee_type: str  # Discriminator
        ...
        ...     class Settings:
        ...         table_name = "employees"
        ...
        >>> class Manager(Employee):
        ...     __discriminator_value__ = "manager"
        ...     department: str
        ...
        >>> class Engineer(Employee):
        ...     __discriminator_value__ = "engineer"
        ...     programming_language: str
        ...
        >>> # Queries automatically filter by discriminator
        >>> managers = await Manager.find().to_list()
        >>> # SELECT * FROM employees WHERE employee_type = 'manager'
    """

    # Class attribute for discriminator value
    __discriminator_value__: ClassVar[Optional[str]] = None

    # Inheritance config
    _inheritance_config: ClassVar[InheritanceConfig]

    def __init_subclass__(cls, **kwargs: Any) -> None:
        """Register subclass in polymorphic registry."""
        super().__init_subclass__(**kwargs)

        # Get parent class with inheritance config
        parent = None
        for base in cls.__mro__[1:]:
            if hasattr(base, "_inheritance_config"):
                parent = base
                break

        if parent is not None:
            discriminator_value = getattr(cls, "__discriminator_value__", None)
            if discriminator_value:
                register_polymorphic_class(parent, cls, discriminator_value)

    @classmethod
    def _get_discriminator_filter(cls) -> Optional[tuple[str, Any]]:
        """
        Get discriminator filter for queries.

        Returns:
            Tuple of (column_name, value) or None if base class
        """
        if not hasattr(cls, "__discriminator_value__"):
            return None

        discriminator_value = cls.__discriminator_value__
        if discriminator_value is None:
            return None

        config = getattr(cls, "_inheritance_config", None)
        if config is None:
            # Try to find config in parent classes
            for base in cls.__mro__[1:]:
                config = getattr(base, "_inheritance_config", None)
                if config is not None:
                    break

        if config is None:
            return None

        return (config.discriminator_column, discriminator_value)

    @classmethod
    def polymorphic_identity(cls) -> Optional[str]:
        """
        Get discriminator value for this class.

        Returns:
            Discriminator value or None if base class
        """
        return cls.__discriminator_value__


class JoinedTableInheritance:
    """
    Base mixin for joined table inheritance.

    Each class has its own table with a foreign key to the parent table.
    Queries require JOINs to assemble complete objects.

    Pros:
        - Normalized schema (no NULL columns)
        - Clear separation of concerns

    Cons:
        - Slower queries (requires JOINs)
        - More complex schema

    Example:
        >>> class Employee(Table, JoinedTableInheritance):
        ...     name: str
        ...     email: str
        ...
        ...     class Settings:
        ...         table_name = "employees"
        ...
        >>> class Manager(Employee):
        ...     department: str
        ...
        ...     class Settings:
        ...         table_name = "managers"
        ...
        >>> class Engineer(Employee):
        ...     programming_language: str
        ...
        ...     class Settings:
        ...         table_name = "engineers"
        ...
        >>> # Queries automatically JOIN parent table
        >>> managers = await Manager.find().to_list()
        >>> # SELECT m.*, e.* FROM managers m
        >>> # JOIN employees e ON m.id = e.id
    """

    _inheritance_config: ClassVar[InheritanceConfig]
    _parent_table: ClassVar[Optional[Type["Table"]]] = None

    def __init_subclass__(cls, **kwargs: Any) -> None:
        """Set up joined table relationships."""
        super().__init_subclass__(**kwargs)

        # Find parent table class
        for base in cls.__mro__[1:]:
            if hasattr(base, "_table_name") and base.__name__ != "Table":
                cls._parent_table = base  # type: ignore
                break

    @classmethod
    def _get_join_config(cls) -> Optional[Dict[str, Any]]:
        """
        Get JOIN configuration for queries.

        Returns:
            Dictionary with parent_table, join_column, etc.
        """
        if cls._parent_table is None:
            return None

        return {
            "parent_table": cls._parent_table._table_name,  # type: ignore
            "parent_pk": cls._parent_table._primary_key,  # type: ignore
            "child_table": cls._table_name,  # type: ignore
            "child_fk": cls._primary_key,  # type: ignore
        }


class ConcreteTableInheritance:
    """
    Base mixin for concrete table inheritance.

    Each class has a complete standalone table with all columns
    (including inherited ones). No foreign keys or joins.

    Pros:
        - Fast queries (no JOINs)
        - Complete independence

    Cons:
        - Duplicated columns across tables
        - UNION queries for polymorphic loading
        - Schema changes must be applied to all tables

    Example:
        >>> class Employee(Table, ConcreteTableInheritance):
        ...     name: str
        ...     email: str
        ...
        ...     class Settings:
        ...         table_name = "employees"
        ...
        >>> class Manager(Employee):
        ...     name: str  # Duplicated
        ...     email: str  # Duplicated
        ...     department: str
        ...
        ...     class Settings:
        ...         table_name = "managers"
        ...
        >>> class Engineer(Employee):
        ...     name: str  # Duplicated
        ...     email: str  # Duplicated
        ...     programming_language: str
        ...
        ...     class Settings:
        ...         table_name = "engineers"
        ...
        >>> # Polymorphic queries use UNION
        >>> all_employees = await Employee.find_polymorphic().to_list()
        >>> # SELECT * FROM employees
        >>> # UNION ALL SELECT * FROM managers
        >>> # UNION ALL SELECT * FROM engineers
    """

    _inheritance_config: ClassVar[InheritanceConfig]
    _concrete_subclasses: ClassVar[List[Type["Table"]]] = []

    def __init_subclass__(cls, **kwargs: Any) -> None:
        """Register subclass for UNION queries."""
        super().__init_subclass__(**kwargs)

        # Find parent with concrete inheritance
        for base in cls.__mro__[1:]:
            if isinstance(base, type) and issubclass(base, ConcreteTableInheritance):
                if not hasattr(base, "_concrete_subclasses"):
                    base._concrete_subclasses = []
                if cls not in base._concrete_subclasses:
                    base._concrete_subclasses.append(cls)
                break


class PolymorphicQueryMixin:
    """
    Mixin to add polymorphic query methods to Table classes.

    Provides methods to load objects as their correct subclass type
    based on discriminator values.

    Example:
        >>> class Employee(Table, SingleTableInheritance, PolymorphicQueryMixin):
        ...     name: str
        ...     type: str
        ...
        >>> # Fetch returns correct subclass instances
        >>> employees = await Employee.fetch_polymorphic()
        >>> # Returns mix of Manager, Engineer, etc. instances
    """

    @classmethod
    async def fetch_polymorphic(
        cls: Type[T],
        *conditions: Any,
        limit: Optional[int] = None,
    ) -> List[T]:
        """
        Fetch objects as their correct polymorphic subclass.

        Args:
            *conditions: Query conditions
            limit: Maximum number of results

        Returns:
            List of objects typed as their specific subclass

        Example:
            >>> employees = await Employee.fetch_polymorphic()
            >>> for emp in employees:
            ...     if isinstance(emp, Manager):
            ...         print(emp.department)
            ...     elif isinstance(emp, Engineer):
            ...         print(emp.programming_language)
        """
        # This would need integration with the query system
        # For now, this is a placeholder
        raise NotImplementedError(
            "Polymorphic queries require integration with query builder"
        )

    @classmethod
    def polymorphic_identity(cls) -> Optional[str]:
        """
        Get the discriminator value for this class.

        Returns:
            Discriminator value or None if base class
        """
        return getattr(cls, "__discriminator_value__", None)

    @classmethod
    def get_subclasses(cls: Type[T]) -> List[Type[T]]:
        """
        Get all registered subclasses.

        Returns:
            List of subclass types

        Example:
            >>> subclasses = Employee.get_subclasses()
            >>> # [Manager, Engineer, Contractor, ...]
        """
        polymorphic_map = get_polymorphic_map(cls)
        return list(polymorphic_map.values())  # type: ignore


# Helper functions


def get_inheritance_type(cls: Type) -> Optional[InheritanceType]:
    """
    Get the inheritance type configured for a class.

    Args:
        cls: Table class to inspect

    Returns:
        InheritanceType or None if not configured

    Example:
        >>> inh_type = get_inheritance_type(Employee)
        >>> if inh_type == InheritanceType.SINGLE_TABLE:
        ...     print("Using single table inheritance")
    """
    config = getattr(cls, "_inheritance_config", None)
    if config is None:
        # Try parent classes
        for base in cls.__mro__[1:]:
            config = getattr(base, "_inheritance_config", None)
            if config is not None:
                break

    return config.inheritance_type if config else None


def get_discriminator_column(cls: Type) -> Optional[str]:
    """
    Get the discriminator column name for a class.

    Args:
        cls: Table class to inspect

    Returns:
        Column name or None if not configured

    Example:
        >>> col_name = get_discriminator_column(Employee)
        >>> print(col_name)  # "type"
    """
    config = getattr(cls, "_inheritance_config", None)
    if config is None:
        # Try parent classes
        for base in cls.__mro__[1:]:
            config = getattr(base, "_inheritance_config", None)
            if config is not None:
                break

    return config.discriminator_column if config else None


def get_discriminator_value(cls: Type) -> Optional[str]:
    """
    Get the discriminator value for a class.

    Args:
        cls: Table class to inspect

    Returns:
        Discriminator value or None if base class

    Example:
        >>> value = get_discriminator_value(Manager)
        >>> print(value)  # "manager"
    """
    return getattr(cls, "__discriminator_value__", None)


def register_polymorphic_class(
    parent: Type,
    child: Type,
    discriminator_value: str,
) -> None:
    """
    Register a child class in the polymorphic registry.

    Args:
        parent: Base class with inheritance config
        child: Subclass to register
        discriminator_value: Value in discriminator column

    Example:
        >>> register_polymorphic_class(Employee, Manager, "manager")
        >>> register_polymorphic_class(Employee, Engineer, "engineer")
    """
    if parent not in _polymorphic_registry:
        _polymorphic_registry[parent] = {}

    _polymorphic_registry[parent][discriminator_value] = child


def get_polymorphic_map(cls: Type) -> Dict[str, Type]:
    """
    Get the polymorphic class mapping for a base class.

    Args:
        cls: Base class to get mapping for

    Returns:
        Dictionary mapping discriminator values to class types

    Example:
        >>> poly_map = get_polymorphic_map(Employee)
        >>> print(poly_map)
        >>> # {"manager": Manager, "engineer": Engineer, ...}
        >>>
        >>> # Use to instantiate correct subclass
        >>> employee_type = row["type"]
        >>> cls = poly_map.get(employee_type, Employee)
        >>> employee = cls(**row)
    """
    return _polymorphic_registry.get(cls, {})
