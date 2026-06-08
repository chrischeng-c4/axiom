"""
Connection management for data-bridge.

This module provides async initialization and connection management
that bridges to the Rust backend.

Example:
    >>> from cclab.mongo import init
    >>> import os
    >>>
    >>> # Initialize with connection string (recommended)
    >>> await init("mongodb://localhost:27017/mydb")
    >>>
    >>> # Or with separate parameters using environment variables
    >>> await init(
    ...     host="localhost",
    ...     port=27017,
    ...     database="mydb",
    ...     username=os.environ.get("MONGODB_USER"),
    ...     password=os.environ.get("MONGODB_PASSWORD"),
    ... )
"""

from __future__ import annotations

from typing import Optional


async def init(
    connection_string: Optional[str] = None,
    *,
    host: Optional[str] = None,
    port: Optional[int] = None,
    database: Optional[str] = None,
    username: Optional[str] = None,
    password: Optional[str] = None,
    auth_source: Optional[str] = None,
    replica_set: Optional[str] = None,
    **options: str,
) -> None:
    """
    Initialize MongoDB connection.

    Can be called with either a connection string or individual parameters.
    The connection is established to the Rust backend which handles all
    MongoDB operations.

    Args:
        connection_string: Full MongoDB URI
            (e.g., "mongodb://user:pass@localhost:27017/mydb?authSource=admin")
        host: MongoDB host (default: "localhost")
        port: MongoDB port (default: 27017)
        database: Database name (required if not using connection_string)
        username: Authentication username
        password: Authentication password
        auth_source: Authentication database (default: database or "admin")
        replica_set: Replica set name
        **options: Additional connection options

    Raises:
        ValueError: If neither connection_string nor database is provided
        RuntimeError: If connection fails

    Example:
        >>> # Using connection string (recommended)
        >>> await init("mongodb://localhost:27017/mydb")
        >>>
        >>> # Using individual parameters with environment variables (recommended)
        >>> import os
        >>> await init(
        ...     host="localhost",
        ...     port=27017,
        ...     database="mydb",
        ...     username=os.environ.get("MONGODB_USER"),
        ...     password=os.environ.get("MONGODB_PASSWORD"),
        ... )
        >>>
        >>> # With replica set
        >>> await init(
        ...     "mongodb://host1:27017,host2:27017/mydb?replicaSet=rs0"
        ... )
    """
    from . import _engine

    if connection_string:
        # Use connection string directly
        await _engine.init(connection_string)
    else:
        # Build connection string from parameters
        if not database:
            raise ValueError(
                "Either connection_string or database must be provided"
            )

        conn_str = _build_connection_string(
            host=host or "localhost",
            port=port or 27017,
            database=database,
            username=username,
            password=password,
            auth_source=auth_source,
            replica_set=replica_set,
            **options,
        )
        await _engine.init(conn_str)


def _build_connection_string(
    host: str,
    port: int,
    database: str,
    username: Optional[str] = None,
    password: Optional[str] = None,
    auth_source: Optional[str] = None,
    replica_set: Optional[str] = None,
    **options: str,
) -> str:
    """Build MongoDB connection string from parameters."""
    from urllib.parse import quote_plus

    # Start with scheme
    conn_str = "mongodb://"

    # Add credentials if provided
    if username and password:
        conn_str += f"{quote_plus(username)}:{quote_plus(password)}@"
    elif username:
        conn_str += f"{quote_plus(username)}@"

    # Add host and port
    conn_str += f"{host}:{port}"

    # Add database
    conn_str += f"/{database}"

    # Build query parameters
    params = {}
    if auth_source:
        params["authSource"] = auth_source
    if replica_set:
        params["replicaSet"] = replica_set
    params.update(options)

    # Add query string
    if params:
        query = "&".join(f"{k}={v}" for k, v in params.items())
        conn_str += f"?{query}"

    return conn_str


def is_connected() -> bool:
    """
    Check if MongoDB is connected.

    Returns:
        True if connected, False otherwise

    Example:
        >>> if not is_connected():
        ...     await init("mongodb://localhost:27017/mydb")
    """
    from . import _engine

    return _engine.is_connected()


async def close() -> None:
    """
    Close the MongoDB connection (Week 10: Connection Lifecycle).

    Closes and releases the current connection. After calling this,
    init() can be called again to establish a new connection.

    This is useful for:
    - Clean shutdown
    - Testing (reset between tests)
    - Connection refresh/reconnection

    Example:
        >>> await init("mongodb://localhost:27017/db1")
        >>> # ... use database ...
        >>> await close()
        >>> await init("mongodb://localhost:27017/db2")  # Different database
    """
    from . import _engine

    if hasattr(_engine._rust, 'close'):
        await _engine._rust.close()
    else:
        # Fallback: no-op if Rust backend doesn't support close
        pass


def reset() -> None:
    """
    Reset the connection without async operation (Week 10: Connection Lifecycle).

    This is a synchronous helper for testing. For production use, prefer close().

    Example:
        >>> reset()  # Synchronous, for testing
        >>> await init("mongodb://localhost:27017/test")
    """
    from . import _engine

    if hasattr(_engine._rust, 'reset'):
        _engine._rust.reset()
    else:
        # Fallback: no-op if Rust backend doesn't support reset
        pass
