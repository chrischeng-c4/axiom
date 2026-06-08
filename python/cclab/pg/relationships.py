"""Relationship descriptors for lazy loading in PostgreSQL ORM.

This module provides:
- LoadingStrategy: Enum defining different relationship loading strategies
- RelationshipDescriptor: Descriptor that enables lazy loading via await syntax
- RelationshipLoader: Handles actual loading logic per instance
- relationship(): Factory function to create relationship descriptors

Example:
    >>> from cclab.pg import Table, Column, relationship
    >>>
    >>> class User(Table):
    ...     id: int = Column(primary_key=True)
    ...     name: str
    ...
    ...     class Settings:
    ...         table_name = "users"
    ...
    >>> class Post(Table):
    ...     id: int = Column(primary_key=True)
    ...     title: str
    ...     author_id: int = Column(foreign_key="users.id")
    ...
    ...     author: User = relationship(User, foreign_key_column="author_id")
    ...
    ...     class Settings:
    ...         table_name = "posts"
    ...
    >>> # Usage
    >>> post = await Post.get(1)
    >>> author = await post.author  # Automatically loads from database
    >>> author_id = post.author.ref  # Get FK value without loading
"""

from __future__ import annotations

from enum import Enum
from typing import Any, Optional, Type, TYPE_CHECKING

from .telemetry import (
    create_relationship_span,
    set_span_result,
    add_exception,
    is_tracing_enabled,
)

if TYPE_CHECKING:
    from .table import Table
    from .session import Session


__all__ = [
    "LoadingStrategy",
    "RelationshipDescriptor",
    "RelationshipLoader",
    "relationship",
]


class LoadingStrategy(Enum):
    """
    Defines different strategies for loading related objects.

    Strategies:
        SELECT: Lazy load with separate SELECT query (default)
        JOINED: Eager load with SQL JOIN
        SUBQUERY: Eager load with subquery
        SELECTIN: Batch load with IN clause (prevents N+1)
        NOLOAD: Never load the relationship
        RAISE: Raise error if relationship is accessed

    Example:
        >>> author: User = relationship(User, foreign_key_column="author_id", lazy="select")
        >>> author: User = relationship(User, foreign_key_column="author_id", lazy="joined")
    """

    SELECT = "select"
    JOINED = "joined"
    SUBQUERY = "subquery"
    SELECTIN = "selectinload"
    NOLOAD = "noload"
    RAISE = "raise"


class RelationshipDescriptor:
    """
    Descriptor that creates lazy-loading relationships between tables.

    This descriptor:
    1. Returns itself when accessed on the class (Post.author)
    2. Returns a RelationshipLoader when accessed on an instance (post.author)
    3. Stores relationship configuration (target model, FK column, strategy)

    The descriptor protocol enables the syntax:
        author = await post.author  # Load related object
        author_id = post.author.ref  # Get FK value without loading

    Args:
        target_model: The target Table class to load
        foreign_key_column: The foreign key column name on the source table
        lazy: Loading strategy ("select", "joined", etc.)
        back_populates: Name of reverse relationship on target model
        uselist: If True, returns list (for one-to-many relationships)

    Example:
        >>> class Post(Table):
        ...     author_id: int
        ...     author: User = relationship(User, foreign_key_column="author_id")
    """

    def __init__(
        self,
        target_model: Type[Table],
        foreign_key_column: str,
        lazy: str = "select",
        back_populates: Optional[str] = None,
        uselist: bool = False,
    ):
        self._target_model = target_model
        self._foreign_key_column = foreign_key_column
        self._lazy = lazy
        self._back_populates = back_populates
        self._uselist = uselist
        self._name: Optional[str] = None

    def __set_name__(self, owner: Type[Table], name: str) -> None:
        """
        Called when descriptor is assigned to a class attribute.

        This is called automatically by Python when the class is created:
            class Post(Table):
                author = RelationshipDescriptor(...)  # __set_name__ called here

        Args:
            owner: The Table class that owns this descriptor
            name: The attribute name ("author" in the example)
        """
        self._name = name

    def __get__(self, obj: Optional[Table], objtype: Optional[Type[Table]] = None) -> Any:
        """
        Descriptor protocol - get the relationship.

        Returns:
            - self if accessed on class (Post.author)
            - RelationshipLoader if accessed on instance (post.author)

        Example:
            >>> Post.author  # Returns RelationshipDescriptor (self)
            >>> post = Post(...)
            >>> post.author  # Returns RelationshipLoader(post, self)
        """
        if obj is None:
            # Class access: Post.author
            return self

        # Instance access: post.author
        return RelationshipLoader(obj, self)


class RelationshipLoader:
    """
    Handles actual loading logic for a relationship on a specific instance.

    This class:
    1. Can be awaited: `author = await post.author`
    2. Caches loaded values to avoid repeated queries
    3. Provides `.ref` property to get FK value without loading
    4. Tracks loading state with `.is_loaded`

    Args:
        instance: The Table instance that owns this relationship
        descriptor: The RelationshipDescriptor with configuration

    Example:
        >>> post = await Post.get(1)
        >>> loader = post.author  # Returns RelationshipLoader instance
        >>> author = await loader  # Load the related User
        >>> author_id = loader.ref  # Get FK value without loading
        >>> is_loaded = loader.is_loaded  # Check if already loaded
    """

    def __init__(self, instance: Table, descriptor: RelationshipDescriptor):
        self._instance = instance
        self._descriptor = descriptor
        self._loaded_value: Optional[Any] = None
        self._is_loaded: bool = False
        self._should_raise: bool = False  # Flag for raiseload option

    def __await__(self):
        """
        Make this object awaitable.

        Enables the syntax:
            author = await post.author

        Returns:
            Iterator that yields the loaded value
        """
        return self._load().__await__()

    async def _load(self) -> Optional[Any]:
        """
        Load the related object from the database.

        This method:
        1. Checks if should raise error (for testing N+1 detection)
        2. Checks if already loaded (cache)
        3. Handles NULL foreign keys
        4. Uses Session identity map if available
        5. Falls back to standalone SELECT query
        6. Caches the result

        Returns:
            The loaded related object (or list if uselist=True), or None if FK is NULL

        Raises:
            RuntimeError: If raiseload option was used and relationship is accessed

        Example:
            >>> post = await Post.get(1)
            >>> author = await post.author  # Triggers _load()
        """
        # 1. Check if should raise (for N+1 detection in tests)
        if self._should_raise:
            raise RuntimeError(
                f"Attempted to access unloaded relationship '{self._descriptor._name}'. "
                f"Use selectinload() to eagerly load this relationship."
            )

        # 2. Check cache first
        if self._is_loaded:
            return self._loaded_value

        # Fast path: no tracing overhead when disabled
        if not is_tracing_enabled():
            # Original logic without spans
            fk_value = self.ref
            if fk_value is None:
                self._loaded_value = None
                self._is_loaded = True
                return None

            from .session import Session
            session = Session.get_current()

            if session is not None:
                value = await self._load_via_session(session, fk_value)
            else:
                value = await self._load_standalone(fk_value)

            self._loaded_value = value
            self._is_loaded = True
            return value

        # Instrumented path: create span for relationship loading
        relationship_name = self._descriptor._name or "unknown"
        target_model_name = self._descriptor._target_model.__name__

        with create_relationship_span(
            name=relationship_name,
            target_model=target_model_name,
            strategy=self._descriptor._lazy,
            fk_column=self._descriptor._foreign_key_column,
        ) as span:
            try:
                # 2. Handle NULL FK early
                fk_value = self.ref  # Uses the .ref property
                if fk_value is None:
                    self._loaded_value = None
                    self._is_loaded = True
                    set_span_result(span, count=0, cache_hit=False)
                    return None

                # 3. Try to use session if available
                from .session import Session
                session = Session.get_current()

                if session is not None:
                    value = await self._load_via_session(session, fk_value)
                    cache_hit = True
                else:
                    value = await self._load_standalone(fk_value)
                    cache_hit = False

                # 4. Cache result
                self._loaded_value = value
                self._is_loaded = True

                # Set span result
                result_count = 1 if value is not None else 0
                set_span_result(span, count=result_count, cache_hit=cache_hit)

                return value
            except Exception as e:
                add_exception(span, e)
                raise

    async def _load_standalone(self, fk_value: Any) -> Optional[Any]:
        """
        Load without session (direct query).

        Uses find_by_foreign_key() to query the database directly.

        Args:
            fk_value: The foreign key value to look up

        Returns:
            The loaded Table instance, or None if not found

        Example:
            >>> # Load author with ID 123
            >>> author = await loader._load_standalone(123)
        """
        from .connection import find_by_foreign_key

        # Get target table name
        target_model = self._descriptor._target_model
        if hasattr(target_model, '_get_table_name'):
            target_table = target_model._get_table_name()
        elif hasattr(target_model.Settings, 'table_name'):
            target_table = target_model.Settings.table_name
        else:
            target_table = target_model.__name__.lower()

        # Get primary key column name (what the FK references)
        if hasattr(target_model, '_get_pk_column'):
            pk_column = target_model._get_pk_column()
        else:
            pk_column = 'id'  # Assume 'id' as default

        # Load data as dict
        data = await find_by_foreign_key(
            table=target_table,
            foreign_key_column=pk_column,
            foreign_key_value=fk_value
        )

        if data is None:
            return None

        # Convert dict to Table instance
        return target_model(**data)

    async def _load_via_session(self, session: 'Session', fk_value: Any) -> Optional[Any]:
        """
        Load through session with identity map.

        Uses session.get() which automatically checks the identity map first,
        ensuring that the same instance is returned if already loaded.

        Args:
            session: The active Session instance
            fk_value: The foreign key value (primary key of related object)

        Returns:
            The loaded Table instance from session, or None if not found

        Example:
            >>> async with Session() as session:
            ...     author = await loader._load_via_session(session, 123)
        """
        target_model = self._descriptor._target_model

        # Use session.get() which checks identity map first
        obj = await session.get(target_model, fk_value)
        return obj

    @property
    def ref(self) -> Any:
        """
        Get the foreign key value without loading the relationship.

        This is useful when you only need the ID and don't want to
        trigger a database query.

        Returns:
            The foreign key value (e.g., author_id)

        Example:
            >>> post = await Post.get(1)
            >>> author_id = post.author.ref  # Get ID without loading
            >>> print(f"Author ID: {author_id}")  # No database query
        """
        # Get FK value from instance data
        fk_column = self._descriptor._foreign_key_column
        return getattr(self._instance, fk_column, None)

    @property
    def is_loaded(self) -> bool:
        """
        Check if the relationship has been loaded.

        Returns:
            True if the relationship has been loaded, False otherwise

        Example:
            >>> post = await Post.get(1)
            >>> post.author.is_loaded  # False
            >>> author = await post.author
            >>> post.author.is_loaded  # True
        """
        return self._is_loaded


def relationship(
    target_model: Type[Table],
    foreign_key_column: str,
    lazy: str = "select",
    back_populates: Optional[str] = None,
    uselist: bool = False,
) -> RelationshipDescriptor:
    """
    Create a relationship descriptor for lazy loading related objects.

    This is the main API for defining relationships between tables.
    It creates a RelationshipDescriptor that enables:
    - Automatic lazy loading with await syntax
    - FK value access without loading
    - Multiple loading strategies
    - Bidirectional relationships

    Args:
        target_model: The target Table class to load
        foreign_key_column: The foreign key column name on the source table
        lazy: Loading strategy (default: "select")
            - "select": Lazy load with separate SELECT query
            - "joined": Eager load with SQL JOIN
            - "subquery": Eager load with subquery
            - "selectinload": Batch load with IN clause (prevents N+1)
            - "noload": Never load the relationship
            - "raise": Raise error if relationship is accessed
        back_populates: Name of reverse relationship on target model
        uselist: If True, returns list (for one-to-many relationships)

    Returns:
        RelationshipDescriptor instance

    Example:
        >>> # One-to-many: Post -> User (author)
        >>> class Post(Table):
        ...     author_id: int = Column(foreign_key="users.id")
        ...     author: User = relationship(User, foreign_key_column="author_id")
        ...
        >>> # Many-to-one: User -> Posts (reverse)
        >>> class User(Table):
        ...     id: int = Column(primary_key=True)
        ...     posts: list[Post] = relationship(
        ...         Post,
        ...         foreign_key_column="author_id",
        ...         back_populates="author",
        ...         uselist=True
        ...     )
        ...
        >>> # Usage
        >>> post = await Post.get(1)
        >>> author = await post.author  # Automatic loading
        >>> author_id = post.author.ref  # FK value without loading
    """
    return RelationshipDescriptor(
        target_model=target_model,
        foreign_key_column=foreign_key_column,
        lazy=lazy,
        back_populates=back_populates,
        uselist=uselist,
    )
