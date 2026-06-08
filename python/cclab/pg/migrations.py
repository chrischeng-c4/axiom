"""Migration support for PostgreSQL.

This module provides migration management for PostgreSQL schema and data:
- Migration base class with up() and down() methods
- run_migrations() function for executing pending migrations
- get_migration_status() for checking applied migrations
- MigrationHistory table for tracking applied migrations

Example:
    >>> from cclab.pg import Migration, run_migrations
    >>>
    >>> class CreateUsersTable(Migration):
    ...     version = "001"
    ...     description = "Create users table"
    ...
    ...     async def up(self):
    ...         await self.execute('''
    ...             CREATE TABLE users (
    ...                 id SERIAL PRIMARY KEY,
    ...                 email VARCHAR(255) UNIQUE NOT NULL,
    ...                 name VARCHAR(255) NOT NULL,
    ...                 created_at TIMESTAMP DEFAULT NOW()
    ...             )
    ...         ''')
    ...
    ...     async def down(self):
    ...         await self.execute('DROP TABLE users')
    >>>
    >>> # Run all pending migrations
    >>> applied = await run_migrations([CreateUsersTable])
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from datetime import datetime
from typing import Any, Dict, List, Optional, Type

# Import from Rust engine when available
try:
    from ..cclab import titan as _titan
except ImportError:
    _engine = None


class MigrationHistory:
    """
    Tracks applied migrations in the database.

    This is stored in the _migrations table to track which migrations
    have been applied and when.

    Attributes:
        id: Auto-incremented primary key
        version: The migration version string
        name: The migration class name
        description: Migration description
        applied_at: When the migration was applied
        direction: "up" or "down"
    """

    @classmethod
    async def create_table(cls) -> None:
        """Create the migrations tracking table if it doesn't exist."""
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        await _titan.execute(
            """
            CREATE TABLE IF NOT EXISTS _migrations (
                id SERIAL PRIMARY KEY,
                version VARCHAR(255) NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                applied_at TIMESTAMP NOT NULL DEFAULT NOW(),
                direction VARCHAR(10) NOT NULL,
                UNIQUE(version, direction)
            )
            """
        )

    @classmethod
    async def record_migration(
        cls,
        version: str,
        name: str,
        description: str,
        direction: str,
    ) -> None:
        """Record that a migration was applied."""
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        await _titan.execute(
            """
            INSERT INTO _migrations (version, name, description, direction)
            VALUES ($1, $2, $3, $4)
            """,
            [version, name, description, direction],
        )

    @classmethod
    async def get_applied_versions(cls) -> List[str]:
        """Get list of applied migration versions."""
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        rows = await _titan.query(
            "SELECT DISTINCT version FROM _migrations WHERE direction = 'up' ORDER BY version"
        )
        return [row["version"] for row in rows]

    @classmethod
    async def is_applied(cls, version: str) -> bool:
        """Check if a migration version has been applied."""
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        rows = await _titan.query(
            "SELECT COUNT(*) as count FROM _migrations WHERE version = $1 AND direction = 'up'",
            [version],
        )
        return rows[0]["count"] > 0 if rows else False


class Migration(ABC):
    """
    Base class for PostgreSQL migrations.

    Subclass this to create migrations. Each migration must have a unique
    version string and implement the up() method. Optionally implement
    down() for rollback support.

    Example:
        >>> class CreateUsersTable(Migration):
        ...     version = "001"
        ...     description = "Create users table"
        ...
        ...     async def up(self):
        ...         await self.execute('''
        ...             CREATE TABLE users (
        ...                 id SERIAL PRIMARY KEY,
        ...                 email VARCHAR(255) UNIQUE NOT NULL,
        ...                 name VARCHAR(255) NOT NULL
        ...             )
        ...         ''')
        ...
        ...     async def down(self):
        ...         await self.execute('DROP TABLE users')
    """

    version: str = ""
    description: str = ""

    async def execute(self, sql: str, params: Optional[List[Any]] = None) -> None:
        """
        Execute a raw SQL statement.

        Args:
            sql: SQL statement to execute
            params: Optional parameters for parameterized queries

        Example:
            >>> await self.execute(
            ...     "INSERT INTO users (email, name) VALUES ($1, $2)",
            ...     ["test@example.com", "Test User"]
            ... )
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        await _titan.execute(sql, params or [])

    async def query(self, sql: str, params: Optional[List[Any]] = None) -> List[Dict[str, Any]]:
        """
        Execute a SQL query and return results.

        Args:
            sql: SQL query to execute
            params: Optional parameters for parameterized queries

        Returns:
            List of dictionaries representing rows

        Example:
            >>> rows = await self.query("SELECT * FROM users WHERE age > $1", [18])
            >>> for row in rows:
            ...     print(row["email"])
        """
        if _engine is None:
            raise RuntimeError(
                "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
            )

        return await _titan.query(sql, params or [])

    @abstractmethod
    async def up(self) -> None:
        """
        Apply the migration.

        This method is called when running migrations forward.
        It should modify the database schema or data as needed.
        """
        pass

    async def down(self) -> None:
        """
        Reverse the migration.

        This method is called when rolling back migrations.
        Override this to provide rollback functionality.

        Raises:
            NotImplementedError: If rollback is not supported
        """
        raise NotImplementedError(
            f"Migration {self.__class__.__name__} does not support rollback"
        )


async def run_migrations(
    migrations: List[Type[Migration]],
    *,
    direction: str = "up",
    target_version: Optional[str] = None,
) -> List[str]:
    """
    Run pending migrations.

    This function:
    1. Ensures the _migrations table exists
    2. Checks which migrations have been applied
    3. Runs pending migrations in order
    4. Records each migration in the _migrations table

    Args:
        migrations: List of Migration classes to run
        direction: "up" to apply migrations, "down" to rollback
        target_version: Optional target version to migrate to

    Returns:
        List of version strings that were applied/rolled back

    Raises:
        RuntimeError: If PostgreSQL engine is not available
        ValueError: If migrations have duplicate versions or invalid direction

    Example:
        >>> # Apply all pending migrations
        >>> applied = await run_migrations([
        ...     CreateUsersTable,
        ...     AddEmailVerified,
        ...     CreateOrdersTable,
        ... ])
        >>> print(f"Applied {len(applied)} migrations")
        >>>
        >>> # Rollback to a specific version
        >>> rolled_back = await run_migrations(
        ...     all_migrations,
        ...     direction="down",
        ...     target_version="001"
        ... )
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    if direction not in ("up", "down"):
        raise ValueError(f"Invalid direction: {direction}. Must be 'up' or 'down'")

    # Ensure migrations table exists
    await MigrationHistory.create_table()

    # Get applied versions
    applied_versions = set(await MigrationHistory.get_applied_versions())

    # Validate no duplicate versions
    versions = [m.version for m in migrations]
    if len(versions) != len(set(versions)):
        raise ValueError("Migrations have duplicate version numbers")

    # Sort migrations by version
    sorted_migrations = sorted(migrations, key=lambda m: m.version)

    # Determine which migrations to run
    to_run = []
    if direction == "up":
        # Apply migrations that haven't been applied yet
        for migration_cls in sorted_migrations:
            if migration_cls.version not in applied_versions:
                to_run.append(migration_cls)
                if target_version and migration_cls.version == target_version:
                    break
    else:  # direction == "down"
        # Rollback migrations in reverse order
        for migration_cls in reversed(sorted_migrations):
            if migration_cls.version in applied_versions:
                to_run.append(migration_cls)
                if target_version and migration_cls.version == target_version:
                    break

    # Run migrations
    applied = []
    for migration_cls in to_run:
        migration = migration_cls()

        try:
            if direction == "up":
                print(f"Applying migration {migration.version}: {migration.description}")
                await migration.up()
            else:
                print(f"Rolling back migration {migration.version}: {migration.description}")
                await migration.down()

            # Record the migration
            await MigrationHistory.record_migration(
                version=migration.version,
                name=migration_cls.__name__,
                description=migration.description,
                direction=direction,
            )

            applied.append(migration.version)

        except Exception as e:
            print(f"Migration {migration.version} failed: {e}")
            raise

    return applied


async def get_migration_status(
    migrations: List[Type[Migration]],
) -> Dict[str, Dict[str, Any]]:
    """
    Get the status of all migrations.

    Args:
        migrations: List of Migration classes to check

    Returns:
        Dictionary mapping version to status information:
        {
            "001": {
                "name": "CreateUsersTable",
                "description": "Create users table",
                "applied": True,
                "applied_at": datetime(...),
            },
            ...
        }

    Raises:
        RuntimeError: If PostgreSQL engine is not available

    Example:
        >>> status = await get_migration_status(all_migrations)
        >>> for version, info in status.items():
        ...     status_str = "✓" if info["applied"] else "✗"
        ...     print(f"{status_str} {version}: {info['description']}")
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    # Ensure migrations table exists
    await MigrationHistory.create_table()

    # Get all migration records
    rows = await _titan.query(
        """
        SELECT version, name, description, applied_at, direction
        FROM _migrations
        WHERE direction = 'up'
        ORDER BY version
        """
    )

    applied_map = {row["version"]: row for row in rows}

    # Build status for all migrations
    status = {}
    for migration_cls in migrations:
        version = migration_cls.version
        migration = migration_cls()

        if version in applied_map:
            record = applied_map[version]
            status[version] = {
                "name": migration_cls.__name__,
                "description": migration.description,
                "applied": True,
                "applied_at": record["applied_at"],
            }
        else:
            status[version] = {
                "name": migration_cls.__name__,
                "description": migration.description,
                "applied": False,
                "applied_at": None,
            }

    return status


def autogenerate_migration(
    current_tables: List[Dict[str, Any]],
    desired_tables: List[Dict[str, Any]],
) -> Dict[str, Any]:
    """
    Auto-generate migration SQL from schema diff.

    Compares current database schema with desired schema and generates
    UP and DOWN SQL statements to migrate between them.

    Args:
        current_tables: List of current table dictionaries (from introspection)
        desired_tables: List of desired table dictionaries (from Table classes)

    Returns:
        Dictionary with:
        - 'up': SQL for forward migration
        - 'down': SQL for rollback migration
        - 'has_changes': Boolean indicating if there are changes

    Example:
        >>> # Get current schema from introspection
        >>> current = await inspect_table("users")
        >>>
        >>> # Define desired schema
        >>> desired = [{
        ...     "name": "users",
        ...     "schema": "public",
        ...     "columns": [
        ...         {
        ...             "name": "id",
        ...             "data_type": "SERIAL",
        ...             "nullable": False,
        ...             "default": None,
        ...             "is_primary_key": True,
        ...             "is_unique": False,
        ...         },
        ...         {
        ...             "name": "email",
        ...             "data_type": "VARCHAR",
        ...             "nullable": False,
        ...             "default": None,
        ...             "is_primary_key": False,
        ...             "is_unique": True,
        ...         }
        ...     ],
        ...     "indexes": [],
        ...     "foreign_keys": [],
        ... }]
        >>>
        >>> # Generate migration
        >>> migration = autogenerate_migration([current], desired)
        >>> if migration['has_changes']:
        ...     print(migration['up'])
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return _titan.autogenerate_migration(current_tables, desired_tables)


__all__ = [
    "Migration",
    "MigrationHistory",
    "run_migrations",
    "get_migration_status",
    "autogenerate_migration",
]
