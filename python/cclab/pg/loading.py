"""
SQLAlchemy-style loading strategies for PostgreSQL relationships.

This module provides loading strategy patterns to control how relationships
are loaded from the database, enabling performance optimization and prevention
of N+1 query problems.

Supported strategies:
- lazy: Load on first access (default)
- joined: Eager load via SQL JOIN
- subquery: Eager load via separate subquery
- selectin: Batch load with IN clause
- noload: Never load (return None)
- raise: Raise error if accessed (prevent N+1)
- raise_on_sql: Raise only if SQL would be generated

Example:
    >>> from cclab.pg import Table, Column, BackReference
    >>> from cclab.pg.loading import joined, selectinload, raiseload
    >>>
    >>> class User(Table):
    ...     id = Column(primary_key=True)
    ...     name = Column()
    ...     posts = BackReference("posts", "user_id")
    >>>
    >>> # Eager load posts with JOIN
    >>> user = await User.fetch_one(User.id == 1, load_strategy=joined())
    >>>
    >>> # Batch load with selectin
    >>> users = await User.fetch_many(load_strategy=selectinload())
    >>>
    >>> # Prevent N+1 queries
    >>> user = await User.fetch_one(User.id == 1, load_strategy=raiseload())
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional, TYPE_CHECKING

if TYPE_CHECKING:
    from .query import QueryBuilder

__all__ = [
    "LoadingStrategy",
    "LoadingConfig",
    "lazy",
    "joined",
    "subquery",
    "selectinload",
    "noload",
    "raiseload",
    "defer",
    "undefer",
    "LazyLoadingProxy",
    "DeferredColumn",
    "RelationshipLoader",
    "LazyLoadError",
    "SQLGenerationError",
]


class LoadingStrategy(Enum):
    """
    Enumeration of available loading strategies for relationships.

    Attributes:
        LAZY: Load relationship on first access (default behavior)
        JOINED: Eager load using SQL JOIN in the same query
        SUBQUERY: Eager load using a separate subquery
        SELECTIN: Batch load multiple objects with SQL IN clause
        NOLOAD: Never load the relationship (returns None)
        RAISE: Raise error if relationship is accessed
        RAISE_ON_SQL: Raise error only if accessing would generate SQL
    """
    LAZY = "lazy"
    JOINED = "joined"
    SUBQUERY = "subquery"
    SELECTIN = "selectin"
    NOLOAD = "noload"
    RAISE = "raise"
    RAISE_ON_SQL = "raise_on_sql"


@dataclass
class LoadingConfig:
    """
    Configuration for relationship loading strategy.

    Attributes:
        strategy: The loading strategy to use
        sql_only: For RAISE_ON_SQL - only raise if SQL would be generated
        deferred_columns: List of column names to defer loading
        innerjoin: Use INNER JOIN instead of LEFT JOIN for JOINED strategy
        columns: Specific columns to load/unload for defer/undefer

    Example:
        >>> config = LoadingConfig(
        ...     strategy=LoadingStrategy.JOINED,
        ...     innerjoin=True
        ... )
    """
    strategy: LoadingStrategy
    sql_only: bool = False
    deferred_columns: List[str] = field(default_factory=list)
    innerjoin: bool = False
    columns: Optional[List[str]] = None


# Strategy factory functions

def lazy() -> LoadingConfig:
    """
    Create lazy loading configuration (default strategy).

    Relationships are loaded on first access, generating a separate SQL query.

    Returns:
        LoadingConfig configured for lazy loading

    Example:
        >>> from cclab.pg.loading import lazy
        >>> user = await User.fetch_one(User.id == 1, load_strategy=lazy())
        >>> # Posts will be loaded when accessed
        >>> posts = await user.posts.fetch_all()
    """
    return LoadingConfig(strategy=LoadingStrategy.LAZY)


def joined(innerjoin: bool = False) -> LoadingConfig:
    """
    Create joined loading configuration.

    Relationships are eagerly loaded using SQL JOIN, fetching related data
    in the same query as the parent object.

    Args:
        innerjoin: Use INNER JOIN instead of LEFT JOIN (default: False)

    Returns:
        LoadingConfig configured for joined loading

    Example:
        >>> from cclab.pg.loading import joined
        >>> # Use LEFT JOIN (default)
        >>> user = await User.fetch_one(User.id == 1, load_strategy=joined())
        >>>
        >>> # Use INNER JOIN (only users with posts)
        >>> user = await User.fetch_one(User.id == 1, load_strategy=joined(innerjoin=True))
    """
    return LoadingConfig(strategy=LoadingStrategy.JOINED, innerjoin=innerjoin)


def subquery() -> LoadingConfig:
    """
    Create subquery loading configuration.

    Relationships are eagerly loaded using a separate subquery, executed
    immediately after the main query.

    Returns:
        LoadingConfig configured for subquery loading

    Example:
        >>> from cclab.pg.loading import subquery
        >>> user = await User.fetch_one(User.id == 1, load_strategy=subquery())
        >>> # Posts are already loaded via subquery
        >>> print(user.posts)
    """
    return LoadingConfig(strategy=LoadingStrategy.SUBQUERY)


def selectinload() -> LoadingConfig:
    """
    Create selectin loading configuration.

    Relationships are batch-loaded using SQL IN clause. Efficient for loading
    relationships for multiple parent objects.

    Returns:
        LoadingConfig configured for selectin loading

    Example:
        >>> from cclab.pg.loading import selectinload
        >>> users = await User.fetch_many(load_strategy=selectinload())
        >>> # All posts for all users loaded with single IN query:
        >>> # SELECT * FROM posts WHERE user_id IN (1, 2, 3, ...)
        >>> for user in users:
        ...     print(user.posts)
    """
    return LoadingConfig(strategy=LoadingStrategy.SELECTIN)


def noload() -> LoadingConfig:
    """
    Create noload configuration.

    Relationships are never loaded and always return None. Useful when you
    know you won't need certain relationships.

    Returns:
        LoadingConfig configured for noload

    Example:
        >>> from cclab.pg.loading import noload
        >>> user = await User.fetch_one(User.id == 1, load_strategy=noload())
        >>> print(user.posts)  # None
    """
    return LoadingConfig(strategy=LoadingStrategy.NOLOAD)


def raiseload(sql_only: bool = False) -> LoadingConfig:
    """
    Create raiseload configuration.

    Accessing relationships raises an error, preventing N+1 query problems.

    Args:
        sql_only: Only raise if SQL would be generated (not if already loaded)

    Returns:
        LoadingConfig configured for raiseload

    Example:
        >>> from cclab.pg.loading import raiseload
        >>>
        >>> # Prevent any lazy loading
        >>> user = await User.fetch_one(User.id == 1, load_strategy=raiseload())
        >>> posts = await user.posts.fetch_all()  # Raises LazyLoadError
        >>>
        >>> # Only prevent SQL generation (ok if already loaded)
        >>> user = await User.fetch_one(User.id == 1, load_strategy=raiseload(sql_only=True))

    Raises:
        LazyLoadError: When relationship is accessed (if sql_only=False)
        SQLGenerationError: When SQL would be generated (if sql_only=True)
    """
    strategy = LoadingStrategy.RAISE_ON_SQL if sql_only else LoadingStrategy.RAISE
    return LoadingConfig(strategy=strategy, sql_only=sql_only)


def defer(*columns: str) -> LoadingConfig:
    """
    Defer loading of specific columns until accessed.

    Deferred columns are not included in the initial SELECT, reducing data
    transfer. They're loaded separately when first accessed.

    Args:
        *columns: Column names to defer loading

    Returns:
        LoadingConfig configured to defer specified columns

    Example:
        >>> from cclab.pg.loading import defer
        >>> # Don't load large text column initially
        >>> posts = await Post.fetch_many(load_strategy=defer("content", "metadata"))
        >>> # These columns will be loaded on access
        >>> content = await posts[0].content.load()
    """
    return LoadingConfig(
        strategy=LoadingStrategy.LAZY,
        deferred_columns=list(columns),
        columns=list(columns)
    )


def undefer(*columns: str) -> LoadingConfig:
    """
    Force loading of previously deferred columns.

    Override defer() to load specific columns that were previously deferred.

    Args:
        *columns: Column names to force loading

    Returns:
        LoadingConfig configured to undefer specified columns

    Example:
        >>> from cclab.pg.loading import undefer
        >>> # Load deferred columns immediately
        >>> posts = await Post.fetch_many(load_strategy=undefer("content"))
    """
    return LoadingConfig(
        strategy=LoadingStrategy.LAZY,
        deferred_columns=[],
        columns=list(columns)
    )


# Custom exceptions

class LazyLoadError(RuntimeError):
    """
    Raised when lazy loading is attempted but not allowed.

    This occurs when using raiseload() strategy to prevent N+1 queries.

    Example:
        >>> user = await User.fetch_one(User.id == 1, load_strategy=raiseload())
        >>> posts = await user.posts.fetch_all()  # Raises LazyLoadError
    """
    pass


class SQLGenerationError(RuntimeError):
    """
    Raised when SQL generation is attempted but not allowed.

    This occurs when using raiseload(sql_only=True) strategy and the
    relationship would require a database query.

    Example:
        >>> user = await User.fetch_one(User.id == 1, load_strategy=raiseload(sql_only=True))
        >>> posts = await user.posts.fetch_all()  # Raises SQLGenerationError
    """
    pass


# Loading infrastructure classes

class LazyLoadingProxy:
    """
    Proxy for lazy-loaded relationships.

    Wraps an unloaded relationship and handles loading on first access.
    Supports async loading with await syntax.

    Attributes:
        _loader: RelationshipLoader instance
        _instance: Parent object instance
        _relationship: Relationship attribute name
        _config: Loading configuration
        _is_loaded: Whether relationship has been loaded
        _value: Cached loaded value

    Example:
        >>> # Automatically created by ORM
        >>> user = await User.fetch_one(User.id == 1)
        >>> proxy = user.posts  # LazyLoadingProxy instance
        >>>
        >>> # Load with await syntax
        >>> posts = await proxy
        >>>
        >>> # Or use fetch method
        >>> posts = await proxy.fetch()
        >>>
        >>> # Check if loaded
        >>> if proxy.is_loaded:
        ...     print("Already loaded!")
    """

    def __init__(
        self,
        loader: RelationshipLoader,
        instance: Any,
        relationship: str,
        config: LoadingConfig
    ):
        """
        Initialize lazy loading proxy.

        Args:
            loader: RelationshipLoader instance to use for loading
            instance: Parent object instance
            relationship: Name of relationship attribute
            config: Loading configuration
        """
        self._loader = loader
        self._instance = instance
        self._relationship = relationship
        self._config = config
        self._is_loaded = False
        self._value: Any = None

    async def fetch(self) -> Any:
        """
        Load the relationship data.

        Returns:
            Loaded relationship value (list, dict, or None)

        Raises:
            LazyLoadError: If strategy is RAISE
            SQLGenerationError: If strategy is RAISE_ON_SQL and not loaded
        """
        # Check if loading is allowed
        if self._config.strategy == LoadingStrategy.RAISE:
            raise LazyLoadError(
                f"Lazy loading is disabled for relationship '{self._relationship}'. "
                f"Use eager loading (joined/selectinload) or load explicitly."
            )

        if self._config.strategy == LoadingStrategy.RAISE_ON_SQL and not self._is_loaded:
            raise SQLGenerationError(
                f"SQL generation is disabled for relationship '{self._relationship}'. "
                f"Relationship must be eager loaded or already in memory."
            )

        # Return cached value if already loaded
        if self._is_loaded:
            return self._value

        # Handle NOLOAD strategy
        if self._config.strategy == LoadingStrategy.NOLOAD:
            self._is_loaded = True
            self._value = None
            return None

        # Load based on strategy
        if self._config.strategy == LoadingStrategy.LAZY:
            self._value = await self._loader.load_lazy(
                self._instance,
                self._relationship,
                self._config
            )
        elif self._config.strategy == LoadingStrategy.SUBQUERY:
            self._value = await self._loader.load_subquery(
                self._instance,
                self._relationship,
                self._config
            )
        else:
            # JOINED and SELECTIN should have been loaded already
            # This is a fallback to lazy loading
            self._value = await self._loader.load_lazy(
                self._instance,
                self._relationship,
                self._config
            )

        self._is_loaded = True
        return self._value

    def __await__(self):
        """
        Enable await syntax for proxy.

        Allows using `await proxy` instead of `await proxy.fetch()`.

        Example:
            >>> posts = await user.posts  # Instead of await user.posts.fetch()
        """
        return self.fetch().__await__()

    @property
    def is_loaded(self) -> bool:
        """
        Check if relationship has been loaded.

        Returns:
            True if loaded, False otherwise
        """
        return self._is_loaded


class DeferredColumn:
    """
    Wrapper for deferred column access.

    Deferred columns are not loaded in the initial query, reducing data
    transfer. They're loaded separately when first accessed.

    Attributes:
        _instance: Parent object instance
        _column_name: Name of deferred column
        _config: Loading configuration
        _is_loaded: Whether column has been loaded
        _value: Cached column value

    Example:
        >>> # Column deferred at query time
        >>> post = await Post.fetch_one(Post.id == 1, load_strategy=defer("content"))
        >>> deferred = post.content  # DeferredColumn instance
        >>>
        >>> # Load the column
        >>> content = await deferred.load()
        >>>
        >>> # Or use await syntax
        >>> content = await deferred
    """

    def __init__(
        self,
        instance: Any,
        column_name: str,
        config: LoadingConfig
    ):
        """
        Initialize deferred column wrapper.

        Args:
            instance: Parent object instance
            column_name: Name of the deferred column
            config: Loading configuration
        """
        self._instance = instance
        self._column_name = column_name
        self._config = config
        self._is_loaded = False
        self._value: Any = None

    async def load(self) -> Any:
        """
        Load the deferred column value.

        Returns:
            Column value from database

        Raises:
            LazyLoadError: If raiseload strategy is active
        """
        # Check if loading is allowed
        if self._config.strategy == LoadingStrategy.RAISE:
            raise LazyLoadError(
                f"Column loading is disabled for '{self._column_name}'. "
                f"Use undefer() to load this column."
            )

        # Return cached value if already loaded
        if self._is_loaded:
            return self._value

        # Load column from database
        # This would integrate with the ORM's column loading mechanism
        # For now, we'll return None as a placeholder
        # TODO: Implement actual column loading via SQL SELECT
        from .connection import execute

        # Get table name from instance
        table_name = getattr(self._instance, "_table_name", None)
        if not table_name:
            raise RuntimeError("Cannot determine table name for deferred column loading")

        # Get primary key
        pk_column = "id"  # Default assumption
        pk_value = getattr(self._instance, "_data", {}).get(pk_column)
        if not pk_value:
            pk_value = getattr(self._instance, pk_column, None)

        if not pk_value:
            raise RuntimeError("Cannot load deferred column without primary key")

        # Execute query to load column
        query = f"SELECT {self._column_name} FROM {table_name} WHERE {pk_column} = $1"
        result = await execute(query, [pk_value])

        if result and len(result) > 0:
            self._value = result[0].get(self._column_name)
        else:
            self._value = None

        self._is_loaded = True
        return self._value

    def __await__(self):
        """
        Enable await syntax for deferred column.

        Example:
            >>> content = await post.content
        """
        return self.load().__await__()

    @property
    def is_loaded(self) -> bool:
        """
        Check if column has been loaded.

        Returns:
            True if loaded, False otherwise
        """
        return self._is_loaded


class RelationshipLoader:
    """
    Executes relationship loading based on strategy.

    This class handles the actual loading of relationships using different
    strategies (lazy, joined, selectin, subquery).

    Example:
        >>> loader = RelationshipLoader()
        >>> # Lazy load
        >>> posts = await loader.load_lazy(user, "posts", lazy())
        >>>
        >>> # Batch load with selectin
        >>> post_map = await loader.load_selectin(users, "posts", selectinload())
    """

    async def load_lazy(
        self,
        instance: Any,
        relationship: str,
        config: LoadingConfig
    ) -> Any:
        """
        Lazy load relationship on first access.

        Generates a separate SQL query to fetch the relationship data.

        Args:
            instance: Parent object instance
            relationship: Name of relationship attribute
            config: Loading configuration

        Returns:
            Loaded relationship data

        Example:
            >>> loader = RelationshipLoader()
            >>> posts = await loader.load_lazy(user, "posts", lazy())
        """
        # Get relationship descriptor from class
        relationship_attr = getattr(type(instance), relationship, None)
        if relationship_attr is None:
            return None

        # Check if it's a ForeignKeyProxy
        from .columns import ForeignKeyProxy, BackReferenceQuery

        if isinstance(relationship_attr, ForeignKeyProxy):
            # Get the proxy instance
            proxy = getattr(instance, relationship, None)
            if proxy and hasattr(proxy, 'fetch'):
                return await proxy.fetch()
            return None

        # Check if it's a BackReference
        from .columns import BackReference

        if isinstance(relationship_attr, BackReference):
            # Get the query object
            query_obj = getattr(instance, relationship, None)
            if query_obj and hasattr(query_obj, 'fetch_all'):
                return await query_obj.fetch_all()
            return None

        # Unknown relationship type
        return None

    def load_joined(
        self,
        query: QueryBuilder,
        relationships: List[str],
        config: LoadingConfig
    ) -> QueryBuilder:
        """
        Add JOINs to query for eager loading.

        Modifies the query to include SQL JOINs for specified relationships.

        Args:
            query: QueryBuilder instance to modify
            relationships: List of relationship names to join
            config: Loading configuration

        Returns:
            Modified QueryBuilder with JOINs added

        Example:
            >>> loader = RelationshipLoader()
            >>> query = User.find(User.active == True)
            >>> query = loader.load_joined(query, ["posts", "profile"], joined())
        """
        # This would integrate with QueryBuilder to add JOIN clauses
        # For now, return the query unchanged as a placeholder
        # TODO: Implement JOIN building via fetch_one_with_relations pattern
        return query

    async def load_selectin(
        self,
        instances: List[Any],
        relationship: str,
        config: LoadingConfig
    ) -> Dict[Any, Any]:
        """
        Batch load relationships with IN clause.

        Loads relationships for multiple instances using a single SQL query
        with IN clause. More efficient than N lazy loads.

        Args:
            instances: List of parent object instances
            relationship: Name of relationship to load
            config: Loading configuration

        Returns:
            Dictionary mapping instance IDs to relationship data

        Example:
            >>> loader = RelationshipLoader()
            >>> users = await User.fetch_many()
            >>> posts_map = await loader.load_selectin(users, "posts", selectinload())
            >>> for user in users:
            ...     user._loaded_posts = posts_map.get(user.id, [])
        """
        if not instances:
            return {}

        # Get relationship descriptor from first instance's class
        relationship_attr = getattr(type(instances[0]), relationship, None)
        if relationship_attr is None:
            return {}

        from .columns import BackReference

        # Handle BackReference
        if isinstance(relationship_attr, BackReference):
            # Collect all IDs
            ids = []
            for instance in instances:
                if hasattr(instance, "_data") and relationship_attr.target_column in instance._data:
                    pk = instance._data[relationship_attr.target_column]
                else:
                    pk = getattr(instance, relationship_attr.target_column, None)
                if pk is not None:
                    ids.append(pk)

            if not ids:
                return {}

            # Load all related objects with IN query
            from .connection import execute

            # Build IN query
            placeholders = ", ".join(f"${i+1}" for i in range(len(ids)))
            query = f"""
                SELECT * FROM {relationship_attr.source_table}
                WHERE {relationship_attr.source_column} IN ({placeholders})
            """

            results = await execute(query, ids)

            # Group by foreign key
            grouped: Dict[Any, List[Dict]] = {}
            for row in results:
                fk_value = row.get(relationship_attr.source_column)
                if fk_value not in grouped:
                    grouped[fk_value] = []
                grouped[fk_value].append(row)

            return grouped

        # Unknown relationship type
        return {}

    async def load_subquery(
        self,
        instance: Any,
        relationship: str,
        config: LoadingConfig
    ) -> Any:
        """
        Load relationship via separate subquery.

        Executes a subquery immediately after the main query to load
        relationship data.

        Args:
            instance: Parent object instance
            relationship: Name of relationship to load
            config: Loading configuration

        Returns:
            Loaded relationship data

        Example:
            >>> loader = RelationshipLoader()
            >>> posts = await loader.load_subquery(user, "posts", subquery())
        """
        # For now, delegate to lazy loading
        # TODO: Implement optimized subquery strategy
        return await self.load_lazy(instance, relationship, config)
