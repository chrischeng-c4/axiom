"""Query builder enhancements for SQLAlchemy-style syntax.

This module provides:
- filter_by(): Simple equality filters using kwargs
- and_(), or_(), not_(): Boolean logic combinators
- any_(), has(): Relationship filters with EXISTS subqueries
- aliased(): Table aliases for self-joins
- QueryFragment: Reusable query conditions
- BooleanClause: Composable AND/OR/NOT expressions
- AliasedClass: Wrapper for aliased tables

Example:
    >>> from cclab.pg import Table
    >>> from cclab.pg.query_ext import filter_by, and_, or_, not_, any_, has, aliased
    >>>
    >>> # Simple equality filter
    >>> users = await User.find(*filter_by(name="Alice", status="active")).to_list()
    >>>
    >>> # Boolean combinators
    >>> condition = and_(User.age > 18, User.status == "active")
    >>> users = await User.find(condition).to_list()
    >>>
    >>> # OR condition
    >>> condition = or_(User.role == "admin", User.role == "moderator")
    >>> users = await User.find(condition).to_list()
    >>>
    >>> # NOT condition
    >>> condition = not_(User.status == "deleted")
    >>> users = await User.find(condition).to_list()
    >>>
    >>> # Relationship filters (ANY)
    >>> users = await User.find(any_(User.posts, Post.views > 1000)).to_list()
    >>>
    >>> # Relationship filters (HAS)
    >>> posts = await Post.find(has(Post.author, User.verified == True)).to_list()
    >>>
    >>> # Self-joins with aliases
    >>> Manager = aliased(Employee)
    >>> results = await Employee.find().join(Manager, Employee.manager_id == Manager.id).to_list()
    >>>
    >>> # Reusable query fragments
    >>> active = QueryFragment(status="active")
    >>> recent = QueryFragment(created_at__gte=datetime.now() - timedelta(days=7))
    >>> users = await User.find(active & recent).to_list()
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional, Type, TYPE_CHECKING, Union

if TYPE_CHECKING:
    from .columns import ColumnProxy, BackReference
    from .table import Table

from .columns import SqlExpr

__all__ = [
    "filter_by",
    "and_",
    "or_",
    "not_",
    "any_",
    "has",
    "aliased",
    "QueryFragment",
    "BooleanClause",
    "AliasedClass",
]


def filter_by(**kwargs: Any) -> List[SqlExpr]:
    """
    Create simple equality filters from keyword arguments.

    This is a convenience function for creating multiple equality conditions
    without using the column proxy syntax.

    Args:
        **kwargs: Column names and values for equality comparison

    Returns:
        List of SqlExpr objects representing equality conditions

    Example:
        >>> # Simple equality filters
        >>> users = await User.find(*filter_by(name="Alice", status="active")).to_list()
        >>>
        >>> # Equivalent to:
        >>> users = await User.find(User.name == "Alice", User.status == "active").to_list()
        >>>
        >>> # Can combine with other filters
        >>> users = await User.find(*filter_by(status="active"), User.age > 25).to_list()
    """
    filters = []
    for key, value in kwargs.items():
        filters.append(SqlExpr(key, "=", value))
    return filters


class BooleanClause:
    """
    Represents a boolean combination of SQL expressions.

    This class allows composing complex boolean logic with AND, OR, and NOT operators.
    It can be nested to create arbitrarily complex conditions.

    Example:
        >>> # AND clause
        >>> clause = BooleanClause("AND", [User.age > 18, User.status == "active"])
        >>>
        >>> # OR clause
        >>> clause = BooleanClause("OR", [User.role == "admin", User.role == "moderator"])
        >>>
        >>> # NOT clause
        >>> clause = BooleanClause("NOT", [User.status == "deleted"])
        >>>
        >>> # Nested clauses
        >>> clause = BooleanClause("AND", [
        ...     User.age > 18,
        ...     BooleanClause("OR", [User.role == "admin", User.role == "moderator"])
        ... ])
    """

    def __init__(self, operator: str, conditions: List[Union[SqlExpr, "BooleanClause"]]) -> None:
        """
        Initialize a boolean clause.

        Args:
            operator: The boolean operator ("AND", "OR", "NOT")
            conditions: List of SqlExpr or BooleanClause objects
        """
        self.operator = operator.upper()
        self.conditions = conditions

        if self.operator not in ("AND", "OR", "NOT"):
            raise ValueError(f"Invalid boolean operator: {operator}. Must be AND, OR, or NOT.")

        if self.operator == "NOT" and len(conditions) != 1:
            raise ValueError("NOT operator requires exactly one condition")

        if self.operator in ("AND", "OR") and len(conditions) < 2:
            raise ValueError(f"{self.operator} operator requires at least two conditions")

    def to_sql(self, param_index: int = 1) -> tuple[str, list[Any]]:
        """
        Convert to SQL WHERE clause.

        Args:
            param_index: Starting parameter index for placeholders

        Returns:
            Tuple of (sql_string, parameters)
        """
        if self.operator == "NOT":
            # NOT has exactly one condition
            condition = self.conditions[0]
            if isinstance(condition, BooleanClause):
                sql, params = condition.to_sql(param_index)
                return (f"NOT ({sql})", params)
            elif isinstance(condition, SqlExpr):
                sql, params = condition.to_sql(param_index)
                return (f"NOT ({sql})", params)
            else:
                raise TypeError(f"Invalid condition type: {type(condition)}")

        # AND or OR
        parts = []
        all_params = []
        current_index = param_index

        for condition in self.conditions:
            if isinstance(condition, BooleanClause):
                sql, params = condition.to_sql(current_index)
                parts.append(f"({sql})")
                all_params.extend(params)
                current_index += len(params)
            elif isinstance(condition, SqlExpr):
                sql, params = condition.to_sql(current_index)
                parts.append(sql)
                all_params.extend(params)
                current_index += len(params)
            else:
                raise TypeError(f"Invalid condition type: {type(condition)}")

        separator = f" {self.operator} "
        return (separator.join(parts), all_params)

    def __and__(self, other: Union[SqlExpr, "BooleanClause"]) -> "BooleanClause":
        """Combine with another condition using AND."""
        return and_(self, other)

    def __or__(self, other: Union[SqlExpr, "BooleanClause"]) -> "BooleanClause":
        """Combine with another condition using OR."""
        return or_(self, other)

    def __invert__(self) -> "BooleanClause":
        """Negate this condition using NOT."""
        return not_(self)

    def __repr__(self) -> str:
        return f"BooleanClause({self.operator!r}, {self.conditions!r})"


def and_(*conditions: Union[SqlExpr, BooleanClause]) -> BooleanClause:
    """
    Combine multiple conditions with AND.

    Args:
        *conditions: SqlExpr or BooleanClause objects to combine

    Returns:
        BooleanClause representing the AND combination

    Example:
        >>> # Basic AND
        >>> condition = and_(User.age > 18, User.status == "active")
        >>> users = await User.find(condition).to_list()
        >>>
        >>> # Multiple conditions
        >>> condition = and_(
        ...     User.age > 18,
        ...     User.status == "active",
        ...     User.verified == True
        ... )
        >>>
        >>> # Nested with OR
        >>> condition = and_(
        ...     User.age > 18,
        ...     or_(User.role == "admin", User.role == "moderator")
        ... )
    """
    if len(conditions) < 2:
        raise ValueError("and_() requires at least two conditions")
    return BooleanClause("AND", list(conditions))


def or_(*conditions: Union[SqlExpr, BooleanClause]) -> BooleanClause:
    """
    Combine multiple conditions with OR.

    Args:
        *conditions: SqlExpr or BooleanClause objects to combine

    Returns:
        BooleanClause representing the OR combination

    Example:
        >>> # Basic OR
        >>> condition = or_(User.role == "admin", User.role == "moderator")
        >>> users = await User.find(condition).to_list()
        >>>
        >>> # Multiple conditions
        >>> condition = or_(
        ...     User.status == "active",
        ...     User.status == "pending",
        ...     User.status == "trial"
        ... )
        >>>
        >>> # Nested with AND
        >>> condition = or_(
        ...     and_(User.role == "admin", User.verified == True),
        ...     User.role == "superuser"
        ... )
    """
    if len(conditions) < 2:
        raise ValueError("or_() requires at least two conditions")
    return BooleanClause("OR", list(conditions))


def not_(condition: Union[SqlExpr, BooleanClause]) -> BooleanClause:
    """
    Negate a condition.

    Args:
        condition: SqlExpr or BooleanClause to negate

    Returns:
        BooleanClause representing the NOT condition

    Example:
        >>> # Simple NOT
        >>> condition = not_(User.status == "deleted")
        >>> users = await User.find(condition).to_list()
        >>>
        >>> # NOT with AND
        >>> condition = not_(and_(User.role == "guest", User.verified == False))
        >>>
        >>> # Double negative (NOT NOT)
        >>> condition = not_(not_(User.status == "active"))
    """
    return BooleanClause("NOT", [condition])


def any_(relationship: "BackReference", condition: SqlExpr) -> BooleanClause:
    """
    Test if ANY related item matches the condition.

    This translates to an EXISTS subquery that checks if at least one
    related row satisfies the given condition.

    Args:
        relationship: BackReference or relationship attribute
        condition: SqlExpr condition to test against related items

    Returns:
        BooleanClause representing the EXISTS subquery

    Example:
        >>> # Find users who have posts with views > 1000
        >>> users = await User.find(any_(User.posts, Post.views > 1000)).to_list()
        >>>
        >>> # Equivalent SQL:
        >>> # SELECT * FROM users
        >>> # WHERE EXISTS (
        >>> #     SELECT 1 FROM posts
        >>> #     WHERE posts.user_id = users.id
        >>> #     AND posts.views > 1000
        >>> # )
        >>>
        >>> # Multiple conditions on related items
        >>> users = await User.find(
        ...     any_(User.posts, and_(Post.views > 1000, Post.status == "published"))
        ... ).to_list()

    Note:
        This is a placeholder implementation. The actual EXISTS subquery
        construction requires integration with the query builder and
        knowledge of the foreign key relationships.
    """
    # TODO: Implement actual EXISTS subquery logic
    # This requires:
    # 1. Access to the BackReference metadata (source_table, source_column)
    # 2. Building a correlated subquery
    # 3. Integration with the query builder
    raise NotImplementedError(
        "any_() is not yet implemented. This requires EXISTS subquery support in the query builder."
    )


def has(relationship: "ColumnProxy", condition: SqlExpr) -> BooleanClause:
    """
    Test if related item (singular) exists with condition.

    This translates to an EXISTS subquery for one-to-one or many-to-one
    relationships (i.e., following a foreign key).

    Args:
        relationship: ColumnProxy or relationship attribute
        condition: SqlExpr condition to test against the related item

    Returns:
        BooleanClause representing the EXISTS subquery

    Example:
        >>> # Find posts whose author is verified
        >>> posts = await Post.find(has(Post.author, User.verified == True)).to_list()
        >>>
        >>> # Equivalent SQL:
        >>> # SELECT * FROM posts
        >>> # WHERE EXISTS (
        >>> #     SELECT 1 FROM users
        >>> #     WHERE users.id = posts.user_id
        >>> #     AND users.verified = TRUE
        >>> # )
        >>>
        >>> # Multiple conditions on related item
        >>> posts = await Post.find(
        ...     has(Post.author, and_(User.verified == True, User.role == "admin"))
        ... ).to_list()

    Note:
        This is a placeholder implementation. The actual EXISTS subquery
        construction requires integration with the query builder and
        knowledge of the foreign key relationships.
    """
    # TODO: Implement actual EXISTS subquery logic
    # This requires:
    # 1. Access to the ForeignKey metadata
    # 2. Building a correlated subquery
    # 3. Integration with the query builder
    raise NotImplementedError(
        "has() is not yet implemented. This requires EXISTS subquery support in the query builder."
    )


class AliasedClass:
    """
    Wrapper around a table with an alias.

    This is used for self-joins and multi-join scenarios where you need
    to reference the same table multiple times in a query.

    Example:
        >>> # Find employees and their managers
        >>> Manager = aliased(Employee)
        >>> results = await Employee.find() \\
        ...     .join(Manager, Employee.manager_id == Manager.id) \\
        ...     .to_list()
        >>>
        >>> # Access aliased columns
        >>> Manager.name  # Generates manager.name reference
        >>> Manager.email  # Generates manager.email reference

    Note:
        This is a placeholder implementation. Full support requires:
        - Integration with join() method in QueryBuilder
        - SQL generation with table aliases
        - Column reference aliasing
    """

    def __init__(self, table_class: Type["Table"], alias: Optional[str] = None) -> None:
        """
        Initialize an aliased table.

        Args:
            table_class: The Table class to alias
            alias: Optional explicit alias name (defaults to lowercase class name + "_1")
        """
        self._table_class = table_class
        self._alias = alias or f"{table_class.__name__.lower()}_1"
        self._column_proxies: Dict[str, "ColumnProxy"] = {}

    def __getattr__(self, name: str) -> "ColumnProxy":
        """
        Get an aliased column proxy.

        Args:
            name: Column name

        Returns:
            ColumnProxy for the aliased column
        """
        # Check if the column exists in the original table
        if hasattr(self._table_class, name):
            attr = getattr(self._table_class, name)
            from .columns import ColumnProxy

            if isinstance(attr, ColumnProxy):
                # Create a new ColumnProxy with the aliased table
                if name not in self._column_proxies:
                    # Store the alias information for SQL generation
                    proxy = ColumnProxy(name, self._table_class)
                    proxy._alias = self._alias  # type: ignore
                    self._column_proxies[name] = proxy
                return self._column_proxies[name]

        raise AttributeError(f"Aliased table {self._alias} has no column {name}")

    def __repr__(self) -> str:
        return f"AliasedClass({self._table_class.__name__}, alias={self._alias!r})"


def aliased(table_class: Type["Table"], name: Optional[str] = None) -> AliasedClass:
    """
    Create an alias for a table (for self-joins and multi-joins).

    Args:
        table_class: The Table class to alias
        name: Optional explicit alias name

    Returns:
        AliasedClass wrapper with aliased column access

    Example:
        >>> # Self-join to find employees and their managers
        >>> Manager = aliased(Employee)
        >>> results = await Employee.find() \\
        ...     .join(Manager, Employee.manager_id == Manager.id) \\
        ...     .select(Employee.name, Manager.name.label("manager_name")) \\
        ...     .to_list()
        >>>
        >>> # Multiple aliases for complex queries
        >>> Manager = aliased(Employee, "mgr")
        >>> Director = aliased(Employee, "dir")
        >>> results = await Employee.find() \\
        ...     .join(Manager, Employee.manager_id == Manager.id) \\
        ...     .join(Director, Manager.manager_id == Director.id) \\
        ...     .to_list()

    Note:
        This is a placeholder implementation. Full support requires:
        - join() method in QueryBuilder
        - SQL generation with proper table aliases
        - Column reference aliasing in SELECT and WHERE clauses
    """
    return AliasedClass(table_class, name)


class QueryFragment:
    """
    Reusable query condition that can be composed and reused.

    Query fragments allow you to define common query conditions once
    and reuse them across multiple queries.

    Example:
        >>> # Define reusable fragments
        >>> active = QueryFragment(status="active")
        >>> recent = QueryFragment(created_at__gte=datetime.now() - timedelta(days=7))
        >>> verified = QueryFragment(verified=True)
        >>>
        >>> # Combine fragments
        >>> users = await User.find(active & recent & verified).to_list()
        >>>
        >>> # Or use individually
        >>> users = await User.find(active).to_list()
        >>> posts = await Post.find(active, recent).to_list()
        >>>
        >>> # Complex fragments with operators
        >>> premium = QueryFragment(and_(
        ...     User.subscription_tier.in_(["pro", "enterprise"]),
        ...     User.subscription_expires > datetime.now()
        ... ))
    """

    def __init__(
        self,
        *conditions: Union[SqlExpr, BooleanClause],
        **kwargs: Any,
    ) -> None:
        """
        Initialize a query fragment.

        Args:
            *conditions: SqlExpr or BooleanClause objects
            **kwargs: Column names and values for equality filters
        """
        self._conditions: List[Union[SqlExpr, BooleanClause]] = list(conditions)

        # Add keyword arguments as equality filters
        for key, value in kwargs.items():
            # Support Django-style lookups (e.g., created_at__gte)
            if "__" in key:
                column, lookup = key.rsplit("__", 1)
                op_map = {
                    "gt": ">",
                    "gte": ">=",
                    "lt": "<",
                    "lte": "<=",
                    "ne": "!=",
                    "in": "IN",
                    "like": "LIKE",
                    "ilike": "ILIKE",
                }
                if lookup in op_map:
                    self._conditions.append(SqlExpr(column, op_map[lookup], value))
                elif lookup == "isnull":
                    if value:
                        self._conditions.append(SqlExpr(column, "IS NULL", None))
                    else:
                        self._conditions.append(SqlExpr(column, "IS NOT NULL", None))
                else:
                    raise ValueError(f"Unknown lookup: {lookup}")
            else:
                self._conditions.append(SqlExpr(key, "=", value))

    def __and__(self, other: Union["QueryFragment", SqlExpr, BooleanClause]) -> "QueryFragment":
        """Combine with another fragment or condition using AND."""
        if isinstance(other, QueryFragment):
            return QueryFragment(*self._conditions, *other._conditions)
        else:
            return QueryFragment(*self._conditions, other)

    def __or__(self, other: Union["QueryFragment", SqlExpr, BooleanClause]) -> BooleanClause:
        """Combine with another fragment or condition using OR."""
        if isinstance(other, QueryFragment):
            # Create an OR clause combining all conditions from both fragments
            return or_(
                and_(*self._conditions) if len(self._conditions) > 1 else self._conditions[0],
                and_(*other._conditions) if len(other._conditions) > 1 else other._conditions[0],
            )
        else:
            return or_(
                and_(*self._conditions) if len(self._conditions) > 1 else self._conditions[0],
                other,
            )

    def __invert__(self) -> BooleanClause:
        """Negate this fragment using NOT."""
        if len(self._conditions) == 1:
            return not_(self._conditions[0])
        else:
            return not_(and_(*self._conditions))

    def to_conditions(self) -> List[Union[SqlExpr, BooleanClause]]:
        """
        Get the list of conditions in this fragment.

        Returns:
            List of SqlExpr or BooleanClause objects
        """
        return self._conditions

    def __iter__(self):
        """Allow unpacking fragment as arguments."""
        return iter(self._conditions)

    def __repr__(self) -> str:
        return f"QueryFragment({self._conditions!r})"


# Convenience functions for common fragment patterns

def active_filter(column: str = "status", value: str = "active") -> QueryFragment:
    """
    Create a fragment for active status filtering.

    Args:
        column: Column name (default: "status")
        value: Active value (default: "active")

    Returns:
        QueryFragment for active filtering

    Example:
        >>> users = await User.find(*active_filter()).to_list()
        >>> # Equivalent to: User.find(User.status == "active")
    """
    return QueryFragment(**{column: value})


def date_range_filter(
    column: str,
    start: Optional[datetime] = None,
    end: Optional[datetime] = None,
) -> QueryFragment:
    """
    Create a fragment for date range filtering.

    Args:
        column: Column name to filter
        start: Start datetime (inclusive)
        end: End datetime (inclusive)

    Returns:
        QueryFragment for date range filtering

    Example:
        >>> from datetime import datetime, timedelta
        >>> now = datetime.now()
        >>> week_ago = now - timedelta(days=7)
        >>> posts = await Post.find(*date_range_filter("created_at", week_ago, now)).to_list()
    """
    conditions = []
    if start is not None:
        conditions.append(SqlExpr(column, ">=", start))
    if end is not None:
        conditions.append(SqlExpr(column, "<=", end))
    return QueryFragment(*conditions)


def in_list_filter(column: str, values: List[Any]) -> QueryFragment:
    """
    Create a fragment for IN list filtering.

    Args:
        column: Column name to filter
        values: List of values

    Returns:
        QueryFragment for IN filtering

    Example:
        >>> users = await User.find(*in_list_filter("role", ["admin", "moderator"])).to_list()
        >>> # Equivalent to: User.find(User.role.in_(["admin", "moderator"]))
    """
    return QueryFragment(SqlExpr(column, "IN", values))


def null_check_filter(column: str, is_null: bool = True) -> QueryFragment:
    """
    Create a fragment for NULL checking.

    Args:
        column: Column name to check
        is_null: True for IS NULL, False for IS NOT NULL

    Returns:
        QueryFragment for NULL checking

    Example:
        >>> users = await User.find(*null_check_filter("deleted_at", True)).to_list()
        >>> # Equivalent to: User.find(User.deleted_at.is_null())
    """
    if is_null:
        return QueryFragment(SqlExpr(column, "IS NULL", None))
    else:
        return QueryFragment(SqlExpr(column, "IS NOT NULL", None))
