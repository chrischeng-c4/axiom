"""
Programmatic migrations for data-bridge.

This module provides Beanie-compatible migration support:
- Migration base class with forward() and backward() methods
- @iterative_migration decorator for document-by-document transforms
- @free_fall_migration decorator for arbitrary migration logic
- run_migrations() function for programmatic migration execution
- MigrationHistory document for tracking applied migrations

Example:
    >>> from cclab.migrations import Migration, run_migrations
    >>>
    >>> class AddEmailVerifiedField(Migration):
    ...     version = "001"
    ...     description = "Add email_verified field to users"
    ...
    ...     async def forward(self):
    ...         await User.find().update({"$set": {"email_verified": False}})
    ...
    ...     async def backward(self):
    ...         await User.find().update({"$unset": {"email_verified": ""}})
    >>>
    >>> # Run all pending migrations
    >>> applied = await run_migrations([AddEmailVerifiedField])
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from datetime import datetime, timezone
from functools import wraps
from typing import Any, Callable, List, Optional, Type, TypeVar, Union

from .document import Document


T = TypeVar("T", bound="Document")
U = TypeVar("U", bound="Document")


class MigrationHistory(Document):
    """
    Document that tracks applied migrations.

    Stored in the _migrations collection to track which migrations
    have been applied and when.

    Attributes:
        version: The migration version string
        name: The migration class name
        applied_at: When the migration was applied
        direction: "forward" or "backward"
    """
    version: str
    name: str
    applied_at: datetime
    direction: str  # "forward" or "backward"

    class Settings:
        name = "_migrations"


class Migration(ABC):
    """
    Base class for database migrations.

    Subclass this to create migrations. Each migration must have a unique
    version string and implement the forward() method. Optionally implement
    backward() for rollback support.

    Example:
        >>> class AddStatusField(Migration):
        ...     version = "001"
        ...     description = "Add status field to all documents"
        ...
        ...     async def forward(self):
        ...         await MyDoc.find().update({"$set": {"status": "active"}})
        ...
        ...     async def backward(self):
        ...         await MyDoc.find().update({"$unset": {"status": ""}})
    """

    version: str = ""
    description: str = ""

    @abstractmethod
    async def forward(self) -> None:
        """
        Apply the migration.

        This method is called when running migrations forward.
        It should modify the database schema or data as needed.
        """
        pass

    async def backward(self) -> None:
        """
        Reverse the migration.

        This method is called when running migrations backward (rollback).
        Override this to provide rollback functionality.

        Raises:
            NotImplementedError: If rollback is not supported
        """
        raise NotImplementedError(
            f"Migration {self.__class__.__name__} does not support rollback"
        )


class IterativeMigration(Migration):
    """
    Base class for document-by-document migrations.

    This is used by the @iterative_migration decorator. Each document
    is transformed individually, allowing for progress tracking and
    memory-efficient processing of large collections.

    Attributes:
        input_model: The source document class
        output_model: The destination document class (can be same as input)
        batch_size: Number of documents to process per batch
    """

    input_model: Type[Document]
    output_model: Type[Document]
    batch_size: int = 100

    @abstractmethod
    async def transform(self, document: Document) -> Document:
        """
        Transform a single document.

        Args:
            document: The input document to transform

        Returns:
            The transformed output document
        """
        pass

    async def forward(self) -> None:
        """
        Apply migration by transforming all documents.
        """
        skip = 0
        while True:
            # Fetch a batch of documents
            docs = await self.input_model.find().skip(skip).limit(self.batch_size).to_list()
            if not docs:
                break

            # Transform each document
            for doc in docs:
                transformed = await self.transform(doc)
                await transformed.save()

            skip += len(docs)


class FreeFallMigration(Migration):
    """
    Base class for free-form migrations.

    This is used by the @free_fall_migration decorator. It provides
    access to document models but doesn't enforce any structure
    on the migration logic.

    Attributes:
        document_models: List of document classes this migration operates on
    """

    document_models: List[Type[Document]] = []


def iterative_migration(
    input_model: Type[T],
    output_model: Optional[Type[U]] = None,
    batch_size: int = 100,
) -> Callable:
    """
    Decorator for creating iterative migrations.

    Use this decorator to create migrations that process documents
    one at a time. This is memory-efficient and allows for progress
    tracking.

    Args:
        input_model: The source document class to read from
        output_model: The destination document class (defaults to input_model)
        batch_size: Number of documents to process per batch

    Returns:
        A decorator that creates an IterativeMigration class

    Example:
        >>> @iterative_migration(User, batch_size=50)
        ... class NormalizeEmails:
        ...     version = "002"
        ...     description = "Normalize all email addresses to lowercase"
        ...
        ...     async def transform(self, user: User) -> User:
        ...         user.email = user.email.lower()
        ...         return user
    """
    def decorator(cls: type) -> Type[Migration]:
        # Get the transform method from the decorated class
        transform_method = getattr(cls, "transform", None)
        if transform_method is None:
            raise ValueError(f"Class {cls.__name__} must have a transform() method")

        # Create a concrete migration class (not using abstract base)
        class WrappedMigration(Migration):
            version = getattr(cls, "version", "")
            description = getattr(cls, "description", "")

            async def forward(self) -> None:
                """Apply migration by transforming all documents."""
                skip = 0
                while True:
                    # Fetch a batch of documents
                    docs = await input_model.find().skip(skip).limit(batch_size).to_list()
                    if not docs:
                        break

                    # Transform each document
                    for doc in docs:
                        transformed = await self._transform(doc)
                        await transformed.save()

                    skip += len(docs)

            async def _transform(self, document: Document) -> Document:
                """Transform wrapper that calls the original method."""
                return await transform_method(self, document)

        # Store model info as class attributes
        WrappedMigration.input_model = input_model
        WrappedMigration.output_model = output_model or input_model
        WrappedMigration.batch_size = batch_size

        # Copy class name for better error messages
        WrappedMigration.__name__ = cls.__name__
        WrappedMigration.__qualname__ = cls.__qualname__

        return WrappedMigration

    return decorator


def free_fall_migration(
    document_models: List[Type[Document]],
) -> Callable:
    """
    Decorator for creating free-form migrations.

    Use this decorator when you need full control over the migration
    logic. The migration can do anything - modify multiple collections,
    run aggregations, etc.

    Args:
        document_models: List of document classes this migration operates on

    Returns:
        A decorator that creates a FreeFallMigration class

    Example:
        >>> @free_fall_migration([User, Post])
        ... class MergeUserProfiles:
        ...     version = "003"
        ...     description = "Merge duplicate user profiles"
        ...
        ...     async def forward(self):
        ...         # Custom logic to find and merge duplicates
        ...         duplicates = await User.aggregate([
        ...             {"$group": {"_id": "$email", "count": {"$sum": 1}}},
        ...             {"$match": {"count": {"$gt": 1}}},
        ...         ]).to_list()
        ...         # ... merge logic
    """
    def decorator(cls: type) -> Type[Migration]:
        # Get forward and backward methods
        forward_method = getattr(cls, "forward", None)
        backward_method = getattr(cls, "backward", None)

        if forward_method is None:
            raise ValueError(f"Class {cls.__name__} must have a forward() method")

        # Create a concrete migration class
        class WrappedMigration(Migration):
            version = getattr(cls, "version", "")
            description = getattr(cls, "description", "")

            async def forward(self) -> None:
                """Run the migration forward."""
                await forward_method(self)

        WrappedMigration.document_models = document_models

        # Add backward method if provided
        if backward_method:
            async def backward_wrapper(self) -> None:
                """Run the migration backward."""
                await backward_method(self)
            WrappedMigration.backward = backward_wrapper

        # Copy class name
        WrappedMigration.__name__ = cls.__name__
        WrappedMigration.__qualname__ = cls.__qualname__

        return WrappedMigration

    return decorator


async def get_applied_migrations() -> List[MigrationHistory]:
    """
    Get list of all applied migrations.

    Returns:
        List of MigrationHistory documents, sorted by applied_at
    """
    return await MigrationHistory.find().sort(("applied_at", 1)).to_list()


async def get_pending_migrations(
    migrations: List[Type[Migration]],
) -> List[Type[Migration]]:
    """
    Get list of migrations that haven't been applied yet.

    Args:
        migrations: List of all migration classes

    Returns:
        List of migration classes that haven't been applied
    """
    applied = await get_applied_migrations()
    applied_versions = {m.version for m in applied if m.direction == "forward"}

    pending = []
    for migration_cls in migrations:
        if migration_cls.version not in applied_versions:
            pending.append(migration_cls)

    # Sort by version
    pending.sort(key=lambda m: m.version)
    return pending


async def run_migrations(
    migrations: List[Type[Migration]],
    direction: str = "forward",
    target_version: Optional[str] = None,
) -> List[str]:
    """
    Run migrations programmatically.

    Args:
        migrations: List of migration classes to consider
        direction: "forward" to apply, "backward" to rollback
        target_version: Optional version to migrate to. If None, runs all pending
                       migrations (forward) or rolls back all (backward).

    Returns:
        List of applied migration versions

    Example:
        >>> # Run all pending migrations
        >>> applied = await run_migrations([Migration1, Migration2, Migration3])
        >>> print(f"Applied {len(applied)} migrations")
        >>>
        >>> # Rollback to a specific version
        >>> rolled_back = await run_migrations(
        ...     [Migration1, Migration2, Migration3],
        ...     direction="backward",
        ...     target_version="001"
        ... )
    """
    applied_versions = []

    if direction == "forward":
        # Get pending migrations
        pending = await get_pending_migrations(migrations)

        # Filter by target version if specified
        if target_version:
            pending = [m for m in pending if m.version <= target_version]

        # Run each migration
        for migration_cls in pending:
            migration = migration_cls()
            try:
                await migration.forward()

                # Record in history
                history = MigrationHistory(
                    version=migration.version,
                    name=migration_cls.__name__,
                    applied_at=datetime.now(timezone.utc),
                    direction="forward",
                )
                await history.save()

                applied_versions.append(migration.version)
            except Exception as e:
                raise RuntimeError(
                    f"Migration {migration_cls.__name__} (v{migration.version}) failed: {e}"
                ) from e

    elif direction == "backward":
        # Get applied migrations in reverse order
        applied = await get_applied_migrations()
        applied.reverse()

        # Filter forward migrations only
        applied = [m for m in applied if m.direction == "forward"]

        # Filter by target version if specified
        if target_version:
            applied = [m for m in applied if m.version > target_version]

        # Build a map of version -> migration class
        migration_map = {m.version: m for m in migrations}

        # Rollback each migration
        for history_entry in applied:
            migration_cls = migration_map.get(history_entry.version)
            if migration_cls is None:
                raise ValueError(
                    f"Cannot rollback migration v{history_entry.version}: "
                    "migration class not found"
                )

            migration = migration_cls()
            try:
                await migration.backward()

                # Record in history
                history = MigrationHistory(
                    version=migration.version,
                    name=migration_cls.__name__,
                    applied_at=datetime.now(timezone.utc),
                    direction="backward",
                )
                await history.save()

                applied_versions.append(migration.version)
            except NotImplementedError:
                raise RuntimeError(
                    f"Migration {migration_cls.__name__} (v{migration.version}) "
                    "does not support rollback"
                )
            except Exception as e:
                raise RuntimeError(
                    f"Migration {migration_cls.__name__} (v{migration.version}) "
                    f"rollback failed: {e}"
                ) from e

    else:
        raise ValueError(f"Invalid direction: {direction}. Use 'forward' or 'backward'")

    return applied_versions


async def get_migration_status(
    migrations: List[Type[Migration]],
) -> List[dict]:
    """
    Get status of all migrations.

    Args:
        migrations: List of all migration classes

    Returns:
        List of dicts with version, name, description, and applied status

    Example:
        >>> status = await get_migration_status([Migration1, Migration2])
        >>> for m in status:
        ...     print(f"{m['version']}: {m['name']} - {'Applied' if m['applied'] else 'Pending'}")
    """
    applied = await get_applied_migrations()
    applied_versions = {m.version for m in applied if m.direction == "forward"}

    status = []
    for migration_cls in sorted(migrations, key=lambda m: m.version):
        status.append({
            "version": migration_cls.version,
            "name": migration_cls.__name__,
            "description": migration_cls.description,
            "applied": migration_cls.version in applied_versions,
        })

    return status


__all__ = [
    "Migration",
    "MigrationHistory",
    "IterativeMigration",
    "FreeFallMigration",
    "iterative_migration",
    "free_fall_migration",
    "run_migrations",
    "get_pending_migrations",
    "get_applied_migrations",
    "get_migration_status",
]
