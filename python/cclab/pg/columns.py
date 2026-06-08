"""Column descriptors and SQL expressions for PostgreSQL tables.

This module provides:
- ColumnProxy: Enables User.email == "x" syntax for type-safe SQL queries
- SqlExpr: Represents a single SQL query condition
- Column: Column descriptor with defaults

Example:
    >>> class User(Table):
    ...     email: str
    ...     age: int
    ...
    >>> # These create SqlExpr objects
    >>> User.email == "alice@example.com"
    >>> User.age > 25
    >>> User.name.in_(["Alice", "Bob"])
"""

from __future__ import annotations

from typing import Any, List, Optional, TYPE_CHECKING, Tuple

if TYPE_CHECKING:
    from .table import Table

__all__ = [
    "SqlExpr",
    "ColumnProxy",
    "Column",
    "ForeignKeyProxy",
    "BackReference",
    "BackReferenceQuery",
    "ManyToMany",
    "ManyToManyQuery",
    "create_m2m_join_table",
]


class SqlExpr:
    """
    Represents a single SQL query condition.

    Converts to SQL WHERE clause syntax when building queries.

    Example:
        >>> expr = SqlExpr("email", "=", "alice@example.com")
        >>> expr.to_sql()
        ("email = $1", ["alice@example.com"])

        >>> expr = SqlExpr("age", ">", 25)
        >>> expr.to_sql()
        ("age > $1", [25])
    """

    def __init__(self, column: str, op: str, value: Any) -> None:
        """
        Initialize SQL expression.

        Args:
            column: Column name
            op: SQL operator (=, !=, >, >=, <, <=, IN, LIKE, etc.)
            value: Value to compare against
        """
        self.column = column
        self.op = op
        self.value = value

    def to_sql(self, param_index: int = 1) -> tuple[str, list[Any]]:
        """
        Convert to SQL WHERE clause.

        Args:
            param_index: Starting parameter index for placeholders

        Returns:
            Tuple of (sql_string, parameters)
        """
        # Special cases
        if self.op == "IN":
            placeholders = ", ".join(f"${i}" for i in range(param_index, param_index + len(self.value)))
            return (f"{self.column} IN ({placeholders})", list(self.value))
        elif self.op == "BETWEEN":
            return (
                f"{self.column} BETWEEN ${param_index} AND ${param_index + 1}",
                [self.value[0], self.value[1]],
            )
        elif self.op == "IS NULL":
            return (f"{self.column} IS NULL", [])
        elif self.op == "IS NOT NULL":
            return (f"{self.column} IS NOT NULL", [])
        else:
            return (f"{self.column} {self.op} ${param_index}", [self.value])

    def __repr__(self) -> str:
        return f"SqlExpr({self.column!r}, {self.op!r}, {self.value!r})"

    def __and__(self, other: "SqlExpr") -> "SqlExpr":
        """Combine two expressions with AND."""
        if isinstance(other, SqlExpr):
            return SqlExpr("AND", "AND", [self, other])
        raise TypeError(f"Cannot combine SqlExpr with {type(other)}")

    def __or__(self, other: "SqlExpr") -> "SqlExpr":
        """Combine two expressions with OR."""
        if isinstance(other, SqlExpr):
            return SqlExpr("OR", "OR", [self, other])
        raise TypeError(f"Cannot combine SqlExpr with {type(other)}")


class ColumnProxy:
    """
    Column proxy that enables attribute-based query expressions.

    When accessing a column on a Table class (e.g., User.email),
    a ColumnProxy is returned. This proxy overloads comparison operators
    to create SqlExpr objects.

    When accessed on an instance, returns the actual value from _data.

    Example:
        >>> class User(Table):
        ...     email: str
        ...     age: int
        ...
        >>> User.email  # Returns ColumnProxy("email", User)
        >>> User.email == "alice@example.com"  # Returns SqlExpr
        >>> User.age > 25  # Returns SqlExpr
        >>> User.name.in_(["Alice", "Bob"])  # Returns SqlExpr
        >>>
        >>> user = User(email="alice@example.com")
        >>> user.email  # Returns "alice@example.com"
    """

    def __init__(self, name: str, model: Optional[type] = None) -> None:
        """
        Initialize column proxy.

        Args:
            name: Column name
            model: Table class this column belongs to
        """
        self.name = name
        self.model = model

    def __hash__(self) -> int:
        """Make ColumnProxy hashable so it can be used as dict key."""
        return hash((self.name, id(self.model)))

    def __get__(self, obj: Any, objtype: Optional[type] = None) -> Any:
        """
        Descriptor protocol: return value on instance access, self on class access.
        """
        if obj is None:
            # Class access: User.email -> ColumnProxy
            return self
        # Instance access: user.email -> value from _data
        if hasattr(obj, "_data") and self.name in obj._data:
            return obj._data[self.name]
        # Fall back to checking __dict__ or returning None
        return obj.__dict__.get(self.name)

    def __set__(self, obj: Any, value: Any) -> None:
        """
        Descriptor protocol: set value on instance.
        """
        if hasattr(obj, "_data"):
            obj._data[self.name] = value
        else:
            obj.__dict__[self.name] = value

    def __repr__(self) -> str:
        model_name = self.model.__name__ if self.model else "None"
        return f"ColumnProxy({self.name!r}, {model_name})"

    # Comparison operators
    def __eq__(self, other: Any) -> SqlExpr:  # type: ignore[override]
        """Create equality query: User.email == "x" """
        return SqlExpr(self.name, "=", other)

    def __ne__(self, other: Any) -> SqlExpr:  # type: ignore[override]
        """Create not-equal query: User.status != "deleted" """
        return SqlExpr(self.name, "!=", other)

    def __gt__(self, other: Any) -> SqlExpr:
        """Create greater-than query: User.age > 25"""
        return SqlExpr(self.name, ">", other)

    def __ge__(self, other: Any) -> SqlExpr:
        """Create greater-than-or-equal query: User.age >= 25"""
        return SqlExpr(self.name, ">=", other)

    def __lt__(self, other: Any) -> SqlExpr:
        """Create less-than query: User.age < 25"""
        return SqlExpr(self.name, "<", other)

    def __le__(self, other: Any) -> SqlExpr:
        """Create less-than-or-equal query: User.age <= 25"""
        return SqlExpr(self.name, "<=", other)

    # Collection operators
    def in_(self, values: List[Any]) -> SqlExpr:
        """Create IN query: User.status.in_(["active", "pending"])"""
        return SqlExpr(self.name, "IN", values)

    def between(self, low: Any, high: Any) -> SqlExpr:
        """Create BETWEEN query: User.age.between(18, 65)"""
        return SqlExpr(self.name, "BETWEEN", [low, high])

    def is_null(self) -> SqlExpr:
        """Check if column is NULL: User.middle_name.is_null()"""
        return SqlExpr(self.name, "IS NULL", None)

    def is_not_null(self) -> SqlExpr:
        """Check if column is NOT NULL: User.email.is_not_null()"""
        return SqlExpr(self.name, "IS NOT NULL", None)

    # String operators
    def like(self, pattern: str) -> SqlExpr:
        """
        Case-sensitive pattern matching: User.email.like("%@example.com")

        Patterns:
            %: Matches any sequence of characters
            _: Matches any single character

        Example:
            >>> User.email.like("%@example.com")  # Ends with @example.com
            >>> User.name.like("A%")  # Starts with A
        """
        return SqlExpr(self.name, "LIKE", pattern)

    def ilike(self, pattern: str) -> SqlExpr:
        """
        Case-insensitive pattern matching: User.email.ilike("%@EXAMPLE.COM")

        Patterns:
            %: Matches any sequence of characters
            _: Matches any single character

        Example:
            >>> User.email.ilike("%@example.com")  # Case-insensitive
        """
        return SqlExpr(self.name, "ILIKE", pattern)

    def startswith(self, prefix: str) -> SqlExpr:
        """
        Check if column starts with prefix: User.name.startswith("A")

        This is a convenience method that uses LIKE with % wildcard.
        """
        return SqlExpr(self.name, "LIKE", f"{prefix}%")

    def contains(self, substring: str) -> SqlExpr:
        """
        Check if column contains substring: User.bio.contains("python")

        This is a convenience method that uses LIKE with % wildcards.
        """
        return SqlExpr(self.name, "LIKE", f"%{substring}%")


class Column:
    """
    Column descriptor with defaults and constraints.

    This provides Pydantic-style field declaration for Table classes.

    Example:
        >>> from cclab.pg import Table, Column
        >>>
        >>> class User(Table):
        ...     email: str = Column(unique=True)
        ...     age: int = Column(default=0)
        ...     created_at: datetime = Column(default_factory=datetime.utcnow)
        ...
        ...     class Settings:
        ...         table_name = "users"
    """

    def __init__(
        self,
        default: Any = None,
        *,
        default_factory: Optional[callable] = None,
        unique: bool = False,
        index: bool = False,
        nullable: bool = True,
        primary_key: bool = False,
        description: Optional[str] = None,
        foreign_key: Optional[str] = None,
        on_delete: Optional[str] = None,
        on_update: Optional[str] = None,
    ) -> None:
        """
        Initialize column with constraints.

        Args:
            default: Default value for the column
            default_factory: Callable that returns default value
            unique: Whether column should have UNIQUE constraint
            index: Whether to create an index on this column
            nullable: Whether column allows NULL values
            primary_key: Whether this is a primary key column
            description: Documentation for this column
            foreign_key: Foreign key reference ("table" or "table.column")
            on_delete: Foreign key ON DELETE action. Valid values:
                "CASCADE", "RESTRICT", "SET NULL", "SET DEFAULT", "NO ACTION"
            on_update: Foreign key ON UPDATE action. Valid values:
                "CASCADE", "RESTRICT", "SET NULL", "SET DEFAULT", "NO ACTION"
        """
        self.default = default
        self.default_factory = default_factory
        self.unique = unique
        self.index = index
        self.nullable = nullable
        self.primary_key = primary_key
        self.description = description
        self.foreign_key = foreign_key
        self.on_delete = on_delete
        self.on_update = on_update

    def __repr__(self) -> str:
        attrs = []
        if self.default is not None:
            attrs.append(f"default={self.default!r}")
        if self.default_factory is not None:
            attrs.append(f"default_factory={self.default_factory!r}")
        if self.unique:
            attrs.append("unique=True")
        if self.index:
            attrs.append("index=True")
        if not self.nullable:
            attrs.append("nullable=False")
        if self.primary_key:
            attrs.append("primary_key=True")
        if self.foreign_key:
            attrs.append(f"foreign_key={self.foreign_key!r}")
        if self.on_delete:
            attrs.append(f"on_delete={self.on_delete!r}")
        if self.on_update:
            attrs.append(f"on_update={self.on_update!r}")
        return f"Column({', '.join(attrs)})"


class ForeignKeyProxy:
    """
    Proxy for foreign key relationships with lazy loading.

    This class enables lazy loading of related objects via foreign keys.
    The related object is only fetched from the database when explicitly
    requested via the fetch() method.

    Example:
        >>> # Assuming Post has foreign key to User
        >>> post = await Post.fetch_one(Post.id == 1)
        >>> print(post.author.ref)  # Get foreign key value without fetching
        123
        >>> author = await post.author.fetch()  # Fetch the related user
        >>> print(author["name"])
        "Alice"
    """

    def __init__(self, target_table: str, foreign_key_column: str, foreign_key_value: Any):
        """
        Initialize foreign key proxy.

        Args:
            target_table: Name of the referenced table
            foreign_key_column: Column name in the referenced table (usually "id")
            foreign_key_value: The foreign key value to query by
        """
        self.target_table = target_table
        self.foreign_key_column = foreign_key_column
        self._foreign_key_value = foreign_key_value
        self._fetched_value = None
        self._is_fetched = False

    async def fetch(self) -> Optional[dict]:
        """
        Fetch the related object from the database.

        If already fetched, returns the cached value without re-querying.

        Returns:
            Dictionary with the related row data, or None if not found

        Example:
            >>> author = await post.author.fetch()
            >>> print(author["name"])
        """
        if self._is_fetched:
            return self._fetched_value

        # Import here to avoid circular dependency
        from cclab.pg import find_by_foreign_key

        result = await find_by_foreign_key(
            self.target_table,
            self.foreign_key_column,
            self._foreign_key_value
        )
        self._fetched_value = result
        self._is_fetched = True
        return result

    @property
    def is_fetched(self) -> bool:
        """
        Check if the related object has been fetched.

        Returns:
            True if fetch() has been called, False otherwise
        """
        return self._is_fetched

    @property
    def ref(self) -> Any:
        """
        Get the foreign key value (ID) without fetching the related object.

        This is useful when you only need the foreign key value itself,
        not the full related object.

        Returns:
            The foreign key value

        Example:
            >>> print(post.author.ref)  # Just the ID, no database query
            123
        """
        return self._foreign_key_value

    @property
    def id(self) -> Any:
        """
        Alias for ref property.

        Returns:
            The foreign key value
        """
        return self._foreign_key_value

    @property
    def column_value(self) -> Any:
        """
        Get the raw column value (foreign key ID).

        Returns:
            The foreign key value
        """
        return self._foreign_key_value

    def __repr__(self) -> str:
        if self._is_fetched:
            return f"ForeignKeyProxy({self.target_table}.{self.foreign_key_column}={self._foreign_key_value}, fetched)"
        return f"ForeignKeyProxy({self.target_table}.{self.foreign_key_column}={self._foreign_key_value}, not fetched)"


class BackReferenceQuery:
    """
    Helper class for back-reference queries.

    This class provides methods to query related rows via reverse foreign key relationships.
    It's returned when accessing a BackReference descriptor on an instance.

    Example:
        >>> # Get all posts for a user
        >>> posts = await user.posts.fetch_all()
        >>> # Get first post
        >>> first_post = await user.posts.fetch_one()
        >>> # Count posts
        >>> post_count = await user.posts.count()
    """

    def __init__(
        self,
        source_table: str,
        source_column: str,
        target_column: str,
        ref_value: Any
    ):
        """
        Initialize back-reference query helper.

        Args:
            source_table: The table that has the FK pointing to this table
            source_column: The FK column name in the source table
            target_column: The column in this table being referenced
            ref_value: The value to match (e.g., user.id)
        """
        self.source_table = source_table
        self.source_column = source_column
        self.target_column = target_column
        self._ref_value = ref_value

    @property
    def ref_value(self) -> Any:
        """
        Get the reference value (e.g., user.id).

        Returns:
            The value being referenced
        """
        return self._ref_value

    async def fetch_all(self) -> List[dict]:
        """
        Fetch all related rows.

        Returns:
            List of dictionaries with row data

        Example:
            >>> # Get all posts for a user
            >>> posts = await user.posts.fetch_all()
            >>> for post in posts:
            ...     print(post["title"])
        """
        # Import here to avoid circular dependency
        try:
            from ..cclab import titan as _titan
        except ImportError:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        if self._ref_value is None:
            return []

        # Build WHERE clause for the back-reference
        where_clause = f"{self.source_column} = $1"
        params = [self._ref_value]

        # Use find_many from the Rust engine (use keyword args for clarity)
        rows = await _titan.find_many(
            table=self.source_table,
            where_clause=where_clause,
            params=params
        )

        return rows

    async def fetch_one(self) -> Optional[dict]:
        """
        Fetch the first related row.

        Returns:
            Dictionary with row data, or None if not found

        Example:
            >>> # Get first post for a user
            >>> first_post = await user.posts.fetch_one()
            >>> if first_post:
            ...     print(first_post["title"])
        """
        # Import here to avoid circular dependency
        try:
            from ..cclab import titan as _titan
        except ImportError:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        if self._ref_value is None:
            return None

        # Build WHERE clause for the back-reference
        where_clause = f"{self.source_column} = $1"
        params = [self._ref_value]

        # Use find_many with limit 1 (use keyword args for clarity)
        rows = await _titan.find_many(
            table=self.source_table,
            where_clause=where_clause,
            params=params,
            limit=1
        )

        return rows[0] if rows else None

    async def count(self) -> int:
        """
        Count related rows.

        Returns:
            Number of related rows

        Example:
            >>> # Count posts for a user
            >>> post_count = await user.posts.count()
            >>> print(f"User has {post_count} posts")
        """
        # Import here to avoid circular dependency
        try:
            from ..cclab import titan as _titan
        except ImportError:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        if self._ref_value is None:
            return 0

        # Build WHERE clause for the back-reference
        where_clause = f"{self.source_column} = $1"
        params = [self._ref_value]

        # Use count from the Rust engine
        return await _titan.count(self.source_table, where_clause, params)

    def __repr__(self) -> str:
        return f"BackReferenceQuery({self.source_table}.{self.source_column} -> {self.target_column}={self._ref_value})"


class BackReference:
    """
    Descriptor for reverse relationship access.

    This enables accessing related rows that have a foreign key pointing to this table.
    It's the inverse of a ForeignKeyProxy - instead of following a FK from this table
    to another, it finds all rows in another table that point back to this one.

    Example:
        >>> from cclab.pg import Table, Column, BackReference
        >>>
        >>> class User(Table):
        ...     id = Column(primary_key=True)
        ...     name = Column()
        ...     # Reverse relationship - posts that reference this user
        ...     posts = BackReference("posts", "user_id")
        ...
        >>> # Usage
        >>> user = await User.fetch_one(User.id == 1)
        >>> user_posts = await user.posts.fetch_all()  # Get all posts where user_id = user.id
        >>> post_count = await user.posts.count()      # Count posts for this user
        >>> first_post = await user.posts.fetch_one()  # Get first post
    """

    def __init__(
        self,
        source_table: str,
        source_column: str,
        target_column: str = "id"
    ):
        """
        Initialize back-reference descriptor.

        Args:
            source_table: The table that has the FK pointing to this table
            source_column: The FK column name in the source table
            target_column: The column in this table being referenced (default: "id")

        Example:
            >>> # Posts table has user_id FK to users.id
            >>> class User(Table):
            ...     id = Column(primary_key=True)
            ...     posts = BackReference("posts", "user_id", "id")
        """
        self.source_table = source_table
        self.source_column = source_column
        self.target_column = target_column
        self._name = None

    def __set_name__(self, owner: type, name: str) -> None:
        """
        Capture the attribute name when descriptor is assigned to class.

        Args:
            owner: The class this descriptor is assigned to
            name: The attribute name
        """
        self._name = name

    def __get__(self, obj: Any, objtype: Optional[type] = None) -> Any:
        """
        Descriptor protocol: return BackReferenceQuery on instance access, self on class access.

        Args:
            obj: Instance or None for class access
            objtype: Owner class

        Returns:
            self for class access, BackReferenceQuery for instance access
        """
        if obj is None:
            # Class access: User.posts -> BackReference
            return self

        # Instance access: user.posts -> BackReferenceQuery
        # Get the reference value from the instance
        if hasattr(obj, "_data") and self.target_column in obj._data:
            ref_value = obj._data[self.target_column]
        else:
            ref_value = getattr(obj, self.target_column, None)

        return BackReferenceQuery(
            self.source_table,
            self.source_column,
            self.target_column,
            ref_value
        )

    def __repr__(self) -> str:
        return f"BackReference({self.source_table}.{self.source_column} -> {self.target_column})"


class ManyToManyQuery:
    """
    Query helper for many-to-many relationships.

    Provides methods to query, add, remove, and manage related objects
    through a join table.

    Example:
        # Get all tags for a post
        tags = await post.tags.fetch_all()

        # Add a tag to a post
        await post.tags.add(tag_id)

        # Remove a tag from a post
        await post.tags.remove(tag_id)

        # Check if post has a specific tag
        has_tag = await post.tags.has(tag_id)

        # Count tags
        count = await post.tags.count()

        # Replace all tags
        await post.tags.set([tag_id1, tag_id2])
    """

    def __init__(
        self,
        join_table: str,
        source_key: str,
        target_key: str,
        target_table: str,
        source_id: Any,
        source_reference: str = "id",
        target_reference: str = "id",
    ):
        self.join_table = join_table
        self.source_key = source_key
        self.target_key = target_key
        self.target_table = target_table
        self.source_id = source_id
        self.source_reference = source_reference
        self.target_reference = target_reference

    async def fetch_all(
        self,
        select_columns: Optional[List[str]] = None,
        order_by: Optional[List[Tuple[str, str]]] = None,
        limit: Optional[int] = None,
    ) -> List[dict]:
        """
        Fetch all related objects.

        Args:
            select_columns: Columns to select (None for all)
            order_by: List of (column, direction) tuples
            limit: Maximum number of results

        Returns:
            List of related objects as dictionaries
        """
        from . import _engine
        return await _titan.m2m_fetch_related(
            self.join_table,
            self.source_key,
            self.target_key,
            self.target_table,
            self.source_id,
            select_columns,
            order_by,
            limit,
            self.source_reference,
            self.target_reference,
        )

    async def fetch_one(self) -> Optional[dict]:
        """Fetch the first related object."""
        results = await self.fetch_all(limit=1)
        return results[0] if results else None

    async def add(self, target_id: int) -> None:
        """
        Add a relation to the target.

        Args:
            target_id: ID of the target to relate
        """
        from . import _engine
        await _titan.m2m_add_relation(
            self.join_table,
            self.source_key,
            self.target_key,
            self.target_table,
            self.source_id,
            target_id,
            self.source_reference,
            self.target_reference,
        )

    async def remove(self, target_id: int) -> int:
        """
        Remove a relation to the target.

        Args:
            target_id: ID of the target to unrelate

        Returns:
            Number of relations removed (0 or 1)
        """
        from . import _engine
        return await _titan.m2m_remove_relation(
            self.join_table,
            self.source_key,
            self.target_key,
            self.target_table,
            self.source_id,
            target_id,
            self.source_reference,
            self.target_reference,
        )

    async def clear(self) -> int:
        """
        Remove all relations for this source.

        Returns:
            Number of relations removed
        """
        from . import _engine
        return await _titan.m2m_clear_relations(
            self.join_table,
            self.source_key,
            self.target_key,
            self.target_table,
            self.source_id,
            self.source_reference,
            self.target_reference,
        )

    async def count(self) -> int:
        """
        Count the number of related objects.

        Returns:
            Number of related objects
        """
        from . import _engine
        return await _titan.m2m_count_related(
            self.join_table,
            self.source_key,
            self.target_key,
            self.target_table,
            self.source_id,
            self.source_reference,
            self.target_reference,
        )

    async def has(self, target_id: int) -> bool:
        """
        Check if a relation to the target exists.

        Args:
            target_id: ID of the target to check

        Returns:
            True if the relation exists
        """
        from . import _engine
        return await _titan.m2m_has_relation(
            self.join_table,
            self.source_key,
            self.target_key,
            self.target_table,
            self.source_id,
            target_id,
            self.source_reference,
            self.target_reference,
        )

    async def set(self, target_ids: List[int]) -> None:
        """
        Replace all relations with the given targets.

        This will remove all existing relations and add the new ones.

        Args:
            target_ids: List of target IDs to relate
        """
        from . import _engine
        await _titan.m2m_set_relations(
            self.join_table,
            self.source_key,
            self.target_key,
            self.target_table,
            self.source_id,
            target_ids,
            self.source_reference,
            self.target_reference,
        )

    async def ids(self) -> List[int]:
        """
        Get just the IDs of related objects.

        Returns:
            List of related object IDs
        """
        results = await self.fetch_all(select_columns=[self.target_reference])
        return [r[self.target_reference] for r in results]


class ManyToMany:
    """
    Descriptor for many-to-many relationships via join table.

    When accessed on a Table instance, returns a ManyToManyQuery
    that provides async methods to manage the relationship.

    Example:
        class Post(Table):
            id: int
            title: str
            tags: ManyToMany["Tag"] = ManyToMany(
                join_table="post_tags",
                source_key="post_id",
                target_key="tag_id",
                target_table="tags",
            )

        class Tag(Table):
            id: int
            name: str
            posts: ManyToMany["Post"] = ManyToMany(
                join_table="post_tags",
                source_key="tag_id",
                target_key="post_id",
                target_table="posts",
            )

        # Usage
        post = await Post.find_one(id=1)
        tags = await post.tags.fetch_all()
        await post.tags.add(new_tag.id)
    """

    def __init__(
        self,
        join_table: str,
        source_key: str,
        target_key: str,
        target_table: str,
        source_reference: str = "id",
        target_reference: str = "id",
    ):
        """
        Initialize a ManyToMany descriptor.

        Args:
            join_table: Name of the join table (e.g., "post_tags")
            source_key: Column in join table for source FK (e.g., "post_id")
            target_key: Column in join table for target FK (e.g., "tag_id")
            target_table: Name of the target table (e.g., "tags")
            source_reference: Column in source table being referenced (default: "id")
            target_reference: Column in target table being referenced (default: "id")
        """
        self.join_table = join_table
        self.source_key = source_key
        self.target_key = target_key
        self.target_table = target_table
        self.source_reference = source_reference
        self.target_reference = target_reference
        self._name: Optional[str] = None

    def __set_name__(self, owner: type, name: str) -> None:
        """Called when the descriptor is assigned to a class attribute."""
        self._name = name

    def __get__(self, obj: Any, objtype: Optional[type] = None) -> "ManyToManyQuery":
        """
        Get the ManyToManyQuery for this relationship.

        When accessed on a class, raises AttributeError.
        When accessed on an instance, returns a ManyToManyQuery bound to the instance's ID.
        """
        if obj is None:
            # Class-level access - return self for introspection
            return self  # type: ignore

        # Instance-level access - get the source ID
        source_id = getattr(obj, self.source_reference, None)
        if source_id is None:
            raise ValueError(
                f"Cannot access ManyToMany relationship: "
                f"'{self.source_reference}' is not set on the instance"
            )

        return ManyToManyQuery(
            join_table=self.join_table,
            source_key=self.source_key,
            target_key=self.target_key,
            target_table=self.target_table,
            source_id=source_id,
            source_reference=self.source_reference,
            target_reference=self.target_reference,
        )

    def __repr__(self) -> str:
        return (
            f"ManyToMany(join_table={self.join_table!r}, "
            f"source_key={self.source_key!r}, target_key={self.target_key!r}, "
            f"target_table={self.target_table!r})"
        )


async def create_m2m_join_table(
    join_table: str,
    source_key: str,
    target_key: str,
    source_table: str,
    target_table: str,
    source_reference: str = "id",
    target_reference: str = "id",
) -> None:
    """
    Create a join table for a many-to-many relationship.

    This is a helper function to create the join table with proper
    foreign key constraints.

    Args:
        join_table: Name of the join table to create
        source_key: Column name for source FK
        target_key: Column name for target FK
        source_table: Name of the source table
        target_table: Name of the target table
        source_reference: Column in source table (default: "id")
        target_reference: Column in target table (default: "id")

    Example:
        await create_m2m_join_table(
            "post_tags",
            "post_id", "tag_id",
            "posts", "tags"
        )
    """
    from . import _engine
    await _titan.m2m_create_join_table(
        join_table,
        source_key,
        target_key,
        source_table,
        target_table,
        source_reference,
        target_reference,
    )
