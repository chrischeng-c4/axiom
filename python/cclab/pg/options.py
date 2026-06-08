"""
Query options for eager loading relationships.

This module provides options for controlling how relationships are loaded:
- SelectInLoad: Batch load relationships using IN clause (prevents N+1)
- JoinedLoad: Eager load using SQL JOIN (not yet implemented)
- NoLoad: Never load this relationship

Example:
    >>> from cclab.pg import selectinload
    >>>
    >>> # Prevent N+1 queries by batch loading
    >>> posts = await Post.find().options(selectinload("author")).to_list()
    >>>
    >>> # All authors are already loaded (no additional queries)
    >>> for post in posts:
    ...     author = await post.author  # Already loaded
    ...     print(f"{post.title} by {author.name}")
"""

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, List

from .telemetry import (
    create_relationship_span,
    set_span_result,
    add_exception,
    is_tracing_enabled,
)

if TYPE_CHECKING:
    from .table import Table

__all__ = [
    "QueryOption",
    "SelectInLoad",
    "JoinedLoad",
    "NoLoad",
    "RaiseLoad",
    "selectinload",
    "joinedload",
    "noload",
    "raiseload",
]


class QueryOption(ABC):
    """Base class for query options."""

    @abstractmethod
    async def apply(self, instances: List['Table']) -> None:
        """Apply this option to loaded instances.

        Args:
            instances: List of Table instances that were loaded
        """
        pass


class SelectInLoad(QueryOption):
    """Batch load relationships using IN clause to prevent N+1 queries.

    This strategy:
    1. Collects all FK values from loaded instances
    2. Loads all related objects in one query using WHERE id IN (...)
    3. Populates the relationship on all instances

    This prevents the N+1 query problem where loading relationships
    for N objects would require N additional queries.

    Example:
        >>> # Without selectinload: 1 + N queries
        >>> posts = await Post.find().to_list()  # 1 query
        >>> for post in posts:
        ...     author = await post.author  # N queries (one per post)
        >>>
        >>> # With selectinload: 1 + 1 queries
        >>> posts = await Post.find().options(selectinload("author")).to_list()  # 1 query
        >>> # Authors loaded in batch: 1 query
        >>> for post in posts:
        ...     author = await post.author  # No query (already loaded)
    """

    def __init__(self, relationship_name: str):
        """Initialize SelectInLoad option.

        Args:
            relationship_name: Name of the relationship attribute to load
        """
        self.relationship_name = relationship_name

    async def apply(self, instances: List['Table']) -> None:
        """Load relationships for all instances using IN clause.

        Args:
            instances: List of Table instances to load relationships for
        """
        if not instances:
            return

        # Get relationship descriptor from first instance's class
        model_class = type(instances[0])
        if not hasattr(model_class, '_relationships'):
            return

        if self.relationship_name not in model_class._relationships:
            raise ValueError(
                f"Unknown relationship: {self.relationship_name}. "
                f"Available relationships: {list(model_class._relationships.keys())}"
            )

        descriptor = model_class._relationships[self.relationship_name]
        target_model = descriptor._target_model
        target_model_name = target_model.__name__

        # Fast path: no tracing overhead when disabled
        if not is_tracing_enabled():
            # Original logic without spans
            fk_values = []
            for instance in instances:
                fk_value = instance._data.get(descriptor._foreign_key_column)
                if fk_value is not None:
                    fk_values.append(fk_value)

            if not fk_values:
                return

            seen = set()
            unique_fk_values = []
            for fk in fk_values:
                if fk not in seen:
                    seen.add(fk)
                    unique_fk_values.append(fk)

            related_objects = await target_model.find(
                target_model.id.in_(unique_fk_values)
            ).to_list()

            lookup = {obj.id: obj for obj in related_objects}

            for instance in instances:
                fk_value = instance._data.get(descriptor._foreign_key_column)
                if fk_value is not None:
                    related_obj = lookup.get(fk_value)
                    loader = descriptor.__get__(instance, type(instance))
                    loader._loaded_value = related_obj
                    loader._is_loaded = True
                else:
                    loader = descriptor.__get__(instance, type(instance))
                    loader._loaded_value = None
                    loader._is_loaded = True
            return

        # Instrumented path: create span for batch loading
        with create_relationship_span(
            name=self.relationship_name,
            target_model=target_model_name,
            strategy="selectinload",
            fk_column=descriptor._foreign_key_column,
            batch_count=len(instances),
        ) as span:
            try:
                # Collect all FK values
                fk_values = []
                for instance in instances:
                    fk_value = instance._data.get(descriptor._foreign_key_column)
                    if fk_value is not None:
                        fk_values.append(fk_value)

                if not fk_values:
                    # All FKs are NULL, nothing to load
                    set_span_result(span, count=0)
                    return

                # Remove duplicates while preserving order
                seen = set()
                unique_fk_values = []
                for fk in fk_values:
                    if fk not in seen:
                        seen.add(fk)
                        unique_fk_values.append(fk)

                # Load all related objects in one query
                # Use find with IN clause (this will trigger query spans)
                related_objects = await target_model.find(
                    target_model.id.in_(unique_fk_values)
                ).to_list()

                # Create lookup dict for O(1) access
                lookup = {obj.id: obj for obj in related_objects}

                # Populate relationships on all instances
                for instance in instances:
                    fk_value = instance._data.get(descriptor._foreign_key_column)
                    if fk_value is not None:
                        related_obj = lookup.get(fk_value)
                        # Set the loaded value on the loader
                        loader = descriptor.__get__(instance, type(instance))
                        loader._loaded_value = related_obj
                        loader._is_loaded = True
                    else:
                        # FK is NULL, mark as loaded with None
                        loader = descriptor.__get__(instance, type(instance))
                        loader._loaded_value = None
                        loader._is_loaded = True

                # Set span result
                set_span_result(span, count=len(related_objects))
            except Exception as e:
                add_exception(span, e)
                raise


class JoinedLoad(QueryOption):
    """Eager load using SQL JOIN (not yet implemented).

    This strategy would use SQL JOINs to load relationships in a single query.
    This is more efficient for small result sets but requires query builder changes.

    Note:
        This is a placeholder for future implementation.
        Currently raises NotImplementedError.

    Example:
        >>> # Future usage:
        >>> posts = await Post.find().options(joinedload("author")).to_list()
    """

    def __init__(self, relationship_name: str):
        """Initialize JoinedLoad option.

        Args:
            relationship_name: Name of the relationship attribute to load
        """
        self.relationship_name = relationship_name

    async def apply(self, instances: List['Table']) -> None:
        """Not yet implemented - requires query builder changes.

        Raises:
            NotImplementedError: This strategy is not yet implemented
        """
        raise NotImplementedError(
            "JoinedLoad requires query builder modifications and is not yet implemented. "
            "Use selectinload() instead for now."
        )


class NoLoad(QueryOption):
    """Never load this relationship.

    This strategy marks the relationship as loaded with None,
    preventing any queries from being executed when accessing it.

    This is useful when you know you don't need a relationship
    and want to avoid accidental N+1 queries.

    Example:
        >>> # Prevent loading author relationship
        >>> posts = await Post.find().options(noload("author")).to_list()
        >>> for post in posts:
        ...     author = await post.author  # Returns None (no query)
    """

    def __init__(self, relationship_name: str):
        """Initialize NoLoad option.

        Args:
            relationship_name: Name of the relationship attribute to never load
        """
        self.relationship_name = relationship_name

    async def apply(self, instances: List['Table']) -> None:
        """Mark relationship as loaded with None on all instances.

        Args:
            instances: List of Table instances to mark as not loaded
        """
        if not instances:
            return

        model_class = type(instances[0])
        if not hasattr(model_class, '_relationships'):
            return

        if self.relationship_name not in model_class._relationships:
            raise ValueError(
                f"Unknown relationship: {self.relationship_name}. "
                f"Available relationships: {list(model_class._relationships.keys())}"
            )

        descriptor = model_class._relationships[self.relationship_name]

        for instance in instances:
            loader = descriptor.__get__(instance, type(instance))
            loader._loaded_value = None
            loader._is_loaded = True


# Convenience functions

def selectinload(relationship_name: str) -> SelectInLoad:
    """Create a selectinload option for batch loading relationships.

    This prevents N+1 queries by loading all related objects in a single
    query using WHERE id IN (...).

    Args:
        relationship_name: Name of the relationship attribute to load

    Returns:
        SelectInLoad option instance

    Example:
        >>> # Load posts with their authors in 2 queries (instead of N+1)
        >>> posts = await Post.find().options(selectinload("author")).to_list()
        >>>
        >>> # All authors already loaded
        >>> for post in posts:
        ...     author = await post.author  # No additional query
        ...     print(f"{post.title} by {author.name}")
    """
    return SelectInLoad(relationship_name)


def joinedload(relationship_name: str) -> JoinedLoad:
    """Create a joinedload option (not yet implemented).

    This would load relationships using SQL JOINs.
    Currently raises NotImplementedError.

    Args:
        relationship_name: Name of the relationship attribute to load

    Returns:
        JoinedLoad option instance

    Raises:
        NotImplementedError: When apply() is called

    Note:
        Use selectinload() instead until this is implemented.
    """
    return JoinedLoad(relationship_name)


def noload(relationship_name: str) -> NoLoad:
    """Create a noload option to prevent loading a relationship.

    This marks the relationship as loaded with None,
    preventing any queries when accessing it.

    Args:
        relationship_name: Name of the relationship attribute to never load

    Returns:
        NoLoad option instance

    Example:
        >>> # Don't load author relationship
        >>> posts = await Post.find().options(noload("author")).to_list()
        >>> for post in posts:
        ...     author = await post.author  # Returns None (no query)
    """
    return NoLoad(relationship_name)


class RaiseLoad(QueryOption):
    """Raise error if relationship is accessed.

    This strategy sets a flag that causes a RuntimeError to be raised
    if the relationship is accessed without being loaded.

    This is useful for detecting N+1 query problems in testing,
    as it will immediately fail if code attempts to access an
    unloaded relationship.

    Example:
        >>> # Detect N+1 problems in tests
        >>> posts = await Post.find().options(raiseload("author")).to_list()
        >>> for post in posts:
        ...     author = await post.author  # Raises RuntimeError
        >>>
        >>> # Instead, use selectinload:
        >>> posts = await Post.find().options(selectinload("author")).to_list()
        >>> for post in posts:
        ...     author = await post.author  # Works (already loaded)
    """

    def __init__(self, relationship_name: str):
        """Initialize RaiseLoad option.

        Args:
            relationship_name: Name of the relationship attribute to raise on access
        """
        self.relationship_name = relationship_name

    async def apply(self, instances: List['Table']) -> None:
        """Mark relationship to raise on access.

        Args:
            instances: List of Table instances to mark
        """
        if not instances:
            return

        model_class = type(instances[0])
        if not hasattr(model_class, '_relationships'):
            return

        if self.relationship_name not in model_class._relationships:
            raise ValueError(
                f"Unknown relationship: {self.relationship_name}. "
                f"Available relationships: {list(model_class._relationships.keys())}"
            )

        descriptor = model_class._relationships[self.relationship_name]

        # Set the raise flag on all loaders
        for instance in instances:
            loader = descriptor.__get__(instance, type(instance))
            loader._should_raise = True


def raiseload(relationship_name: str) -> RaiseLoad:
    """Create a raiseload option.

    Raises a RuntimeError if the relationship is accessed.
    Useful for detecting N+1 problems in testing.

    Args:
        relationship_name: Name of the relationship attribute

    Returns:
        RaiseLoad option instance

    Example:
        >>> # In tests - detect N+1 queries
        >>> posts = await Post.find().options(raiseload("author")).to_list()
        >>> try:
        ...     author = await posts[0].author  # Raises RuntimeError
        ... except RuntimeError as e:
        ...     print(f"N+1 detected: {e}")
        >>>
        >>> # Fix: use selectinload
        >>> posts = await Post.find().options(selectinload("author")).to_list()
        >>> author = await posts[0].author  # Works
    """
    return RaiseLoad(relationship_name)
