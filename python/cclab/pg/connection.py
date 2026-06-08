"""PostgreSQL connection management."""

from typing import Optional, Literal, List, Dict, Any, Union
from contextlib import asynccontextmanager

# Import from Rust engine when available
try:
    from ..cclab import titan as _titan
    _engine = _titan
except ImportError:
    _titan = None
    _engine = None

# Type alias for isolation levels
IsolationLevel = Literal["read_uncommitted", "read_committed", "repeatable_read", "serializable"]


async def init(
    connection_string: Optional[str] = None,
    *,
    host: str = "localhost",
    port: int = 5432,
    database: str = "postgres",
    username: Optional[str] = None,
    password: Optional[str] = None,
    min_connections: int = 1,
    max_connections: int = 10,
) -> None:
    """
    Initialize PostgreSQL connection pool.

    Security Warning:
        NEVER hardcode passwords in source code. Always use environment variables
        or secure configuration management systems to handle credentials.

        Passwords may be exposed in:
        - Exception tracebacks
        - Log files
        - Process memory dumps
        - Debug output

    Args:
        connection_string: Full PostgreSQL connection string (postgres://user:pass@host:port/db)
        host: PostgreSQL server hostname (default: localhost)
        port: PostgreSQL server port (default: 5432)
        database: Database name (default: postgres)
        username: Database username
        password: Database password
        min_connections: Minimum number of connections in pool (default: 1)
        max_connections: Maximum number of connections in pool (default: 10)

    Example:
        >>> # RECOMMENDED: Using connection string from environment variable
        >>> import os
        >>> await init(os.environ.get("DATABASE_URL"))
        >>>
        >>> # RECOMMENDED: Using individual parameters with environment variables
        >>> await init(
        ...     host="localhost",
        ...     port=5432,
        ...     database="mydb",
        ...     username=os.environ.get("PG_USER"),
        ...     password=os.environ.get("PG_PASSWORD"),
        ...     max_connections=20
        ... )
        >>>
        >>> # WARNING: For local development/testing only - NEVER use in production
        >>> await init(
        ...     host="localhost",
        ...     database="test_db",
        ...     username="testuser",
        ...     password="<your-password-here>"  # Use environment variable instead
        ... )

    Raises:
        RuntimeError: If connection fails or Rust engine is not available
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    if connection_string is None:
        # Build connection string from individual parameters
        # NOTE: Python strings are immutable, so password will remain in memory until
        # garbage collected. For production use, prefer passing a pre-built connection
        # string from environment variables (e.g., DATABASE_URL) to minimize password
        # exposure in application code.
        auth = f"{username}:{password}@" if username else ""
        connection_string = f"postgres://{auth}{host}:{port}/{database}"
        # Clear local references (though Python strings are immutable)
        del auth

    # Connection string is passed to Rust engine where it's handled securely
    await _titan.init(connection_string, min_connections, max_connections)


async def close() -> None:
    """
    Close the PostgreSQL connection pool.

    This should be called when shutting down your application to ensure
    all connections are properly closed.

    Example:
        >>> await close()

    Raises:
        RuntimeError: If Rust engine is not available
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    await _titan.close()


def is_connected() -> bool:
    """
    Check if the PostgreSQL connection pool is active.

    Returns:
        True if connected, False otherwise

    Example:
        >>> if is_connected():
        ...     print("Connected to PostgreSQL")
        ... else:
        ...     print("Not connected")
    """
    if _engine is None:
        return False

    return _titan.is_connected()


async def execute(
    sql: str,
    params: Optional[list] = None
):
    """
    Execute raw SQL query with parameter binding.

    This function provides direct SQL execution for power users who need
    to bypass the ORM. It supports:
    - SELECT queries (returns list of dicts)
    - INSERT/UPDATE/DELETE (returns row count or list of dicts if RETURNING)
    - DDL commands like CREATE, ALTER, DROP (returns None)

    WARNING: This bypasses ORM safety features. Use with caution.
    Always use parameterized queries ($1, $2, etc.) to prevent SQL injection.

    Args:
        sql: SQL query string with $1, $2, etc. placeholders
        params: Optional list of parameters to bind to placeholders

    Returns:
        - List[Dict[str, Any]] for SELECT queries or DML with RETURNING
        - int for INSERT/UPDATE/DELETE without RETURNING (number of affected rows)
        - None for DDL commands (CREATE, ALTER, DROP, etc.)
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.execute(sql, params)


async def query_aggregate(
    table: str,
    aggregates: List[tuple],
    group_by: Optional[List[str]] = None,
    having: Optional[List[tuple]] = None,
    where_conditions: Optional[List[tuple]] = None,
    order_by: Optional[List[tuple]] = None,
    limit: Optional[int] = None,
    distinct: Optional[bool] = None,
    distinct_on: Optional[List[str]] = None,
    ctes: Optional[List[tuple]] = None,
    subqueries: Optional[List[tuple]] = None,
) -> List[Dict[str, Any]]:
    """
    Execute an aggregate query with GROUP BY and HAVING support.

    Args:
        table: Table name
        aggregates: List of (func_type, column, alias) tuples
                   func_type: "count", "count_column", "count_distinct", "sum", "avg", "min", "max"
                   column: Column name (optional for "count")
                   alias: Optional alias for the aggregate result
        group_by: List of column names to group by
        having: List of (func_type, column, operator, value) tuples for HAVING clause
        where_conditions: List of (field, operator, value) tuples for WHERE clause
        order_by: List of (column, direction) tuples - direction: "asc" or "desc"
        limit: Optional row limit
        distinct: Optional boolean to select distinct rows
        distinct_on: Optional list of columns for DISTINCT ON (PostgreSQL-specific)
        ctes: Optional list of (name, sql, params) tuples for CTEs
        subqueries: Optional list of (type, field, sql, params) tuples for subquery conditions
                   type: "in", "not_in", "exists", "not_exists"
                   field: Column name for IN/NOT IN (None for EXISTS/NOT EXISTS)
                   sql: Subquery SQL
                   params: List of parameter values for subquery

    Returns:
        List of dictionaries with aggregate results

    Example:
        >>> # Group by user_id with HAVING clause
        >>> results = await query_aggregate(
        ...     "orders",
        ...     [("sum", "amount", "total"), ("count", None, "count")],
        ...     group_by=["user_id"],
        ...     having=[("sum", "amount", "gt", 1000)],
        ...     where_conditions=[("status", "eq", "completed")],
        ...     order_by=[("total", "desc")],
        ...     limit=10
        ... )
        >>>
        >>> # With subquery
        >>> results = await query_aggregate(
        ...     "users",
        ...     [("count", None, "total")],
        ...     subqueries=[("in", "id", "SELECT user_id FROM orders", [])]
        ... )
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")

    return await _titan.query_aggregate(
        table,
        aggregates,
        group_by,
        having,
        where_conditions,
        order_by,
        limit,
        distinct,
        distinct_on,
        ctes,
        subqueries
    )


async def query_with_cte(
    main_table: str,
    ctes: List[tuple],
    select_columns: Optional[List[str]] = None,
    where_conditions: Optional[List[tuple]] = None,
    order_by: Optional[List[tuple]] = None,
    limit: Optional[int] = None,
    subqueries: Optional[List[tuple]] = None,
) -> List[Dict[str, Any]]:
    """
    Execute a query with Common Table Expressions (CTEs).

    Args:
        main_table: The table or CTE name to query from in the main SELECT
        ctes: List of (name, sql, params) tuples defining CTEs
        select_columns: Optional list of columns to select (defaults to *)
        where_conditions: List of (field, operator, value) tuples for WHERE clause
        order_by: List of (column, direction) tuples - direction: "asc" or "desc"
        limit: Optional row limit
        subqueries: Optional list of (type, field, sql, params) tuples for subquery conditions
                   type: "in", "not_in", "exists", "not_exists"
                   field: Column name for IN/NOT IN (None for EXISTS/NOT EXISTS)
                   sql: Subquery SQL
                   params: List of parameter values for subquery

    Returns:
        List of dictionaries with query results

    Example:
        >>> # Query with CTE
        >>> results = await query_with_cte(
        ...     "high_value_orders",
        ...     [("high_value_orders", "SELECT * FROM orders WHERE total > $1", [1000])],
        ...     select_columns=["order_id", "total"],
        ...     where_conditions=[("status", "eq", "completed")],
        ...     order_by=[("total", "desc")],
        ...     limit=10
        ... )
        >>>
        >>> # Query with CTE and subquery
        >>> results = await query_with_cte(
        ...     "users",
        ...     [("active_orders", "SELECT user_id FROM orders WHERE status = $1", ["active"])],
        ...     subqueries=[("in", "id", "SELECT user_id FROM active_orders", [])]
        ... )
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")

    return await _titan.query_with_cte(
        main_table,
        ctes,
        select_columns,
        where_conditions,
        order_by,
        limit,
        subqueries
    )


async def insert_one(
    table: str,
    document: Dict[str, Any],
) -> Dict[str, Any]:
    """Insert a single document into a table.

    Args:
        table: Table name
        document: Document data (column -> value mapping)

    Returns:
        Inserted document with all columns (including generated id)

    Example:
        >>> result = await insert_one("users", {"name": "Alice", "age": 30})
        >>> print(result["id"])  # Auto-generated ID

    Raises:
        RuntimeError: If PostgreSQL engine not available or insert fails
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return await _titan.insert_one(table, document)


async def insert_many(
    table: str,
    documents: List[Dict[str, Any]],
) -> List[Dict[str, Any]]:
    """Insert multiple documents into a table.

    Args:
        table: Table name
        documents: List of documents to insert

    Returns:
        List of inserted documents with all columns

    Example:
        >>> results = await insert_many("users", [
        ...     {"name": "Alice", "age": 30},
        ...     {"name": "Bob", "age": 25}
        ... ])

    Raises:
        RuntimeError: If PostgreSQL engine not available or insert fails
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return await _titan.insert_many(table, documents)


async def upsert_one(
    table: str,
    document: Dict[str, Any],
    conflict_target: Union[str, List[str]],
    update_columns: Optional[List[str]] = None,
) -> Dict[str, Any]:
    """Insert or update a document using ON CONFLICT.

    This performs an "upsert" operation: if the document conflicts with an existing row
    (based on the conflict_target unique constraint), it will update; otherwise it inserts.

    Args:
        table: Table name
        document: Document data (column -> value mapping)
        conflict_target: Column(s) for ON CONFLICT clause (string or list of strings)
        update_columns: Optional columns to update on conflict (None = all except conflict_target)

    Returns:
        Inserted or updated document with all columns

    Example:
        >>> # Upsert by email (unique constraint)
        >>> result = await upsert_one("users",
        ...     {"email": "alice@example.com", "name": "Alice Updated", "age": 31},
        ...     conflict_target="email"
        ... )
        >>>
        >>> # Upsert with selective update (only update name and age, not email)
        >>> result = await upsert_one("users",
        ...     {"email": "bob@example.com", "name": "Bob", "age": 25, "status": "active"},
        ...     conflict_target=["email"],
        ...     update_columns=["name", "age"]
        ... )
        >>>
        >>> # Composite unique constraint
        >>> result = await upsert_one("user_roles",
        ...     {"user_id": 1, "role": "admin", "granted_at": "2024-01-01"},
        ...     conflict_target=["user_id", "role"]
        ... )

    Raises:
        RuntimeError: If PostgreSQL engine not available or upsert fails
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")

    # Normalize conflict_target to list
    if isinstance(conflict_target, str):
        conflict_target = [conflict_target]

    return await _titan.upsert_one(table, document, conflict_target, update_columns)


async def upsert_many(
    table: str,
    documents: List[Dict[str, Any]],
    conflict_target: Union[str, List[str]],
    update_columns: Optional[List[str]] = None,
) -> List[Dict[str, Any]]:
    """Batch insert or update documents using ON CONFLICT.

    This generates a multi-row INSERT with ON CONFLICT for efficient batch upserts.
    All documents must have the same columns.

    Args:
        table: Table name
        documents: List of documents (each is a dict of column -> value)
        conflict_target: Column(s) for ON CONFLICT clause (string or list of strings)
        update_columns: Optional columns to update on conflict (None = all except conflict_target)

    Returns:
        List of inserted/updated documents with all columns

    Example:
        >>> # Batch upsert by email
        >>> results = await upsert_many("users", [
        ...     {"email": "alice@example.com", "name": "Alice", "age": 30},
        ...     {"email": "bob@example.com", "name": "Bob", "age": 25},
        ...     {"email": "charlie@example.com", "name": "Charlie", "age": 35}
        ... ], conflict_target="email")
        >>>
        >>> # Selective update (preserve existing 'created_at' column)
        >>> results = await upsert_many("users",
        ...     documents,
        ...     conflict_target=["email"],
        ...     update_columns=["name", "age"]  # Don't update 'created_at'
        ... )

    Raises:
        RuntimeError: If PostgreSQL engine not available or batch upsert fails
        ValueError: If documents have different columns
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")

    # Normalize conflict_target to list
    if isinstance(conflict_target, str):
        conflict_target = [conflict_target]

    return await _titan.upsert_many(table, documents, conflict_target, update_columns)


import contextvars;

# Context variable to track the active transaction
_active_transaction = contextvars.ContextVar("_active_transaction", default=None)


class TransactionWrapper:
    """Wrapper for PyTransaction to provide cleaner Python API."""
    def __init__(self, tx):
        self._tx = tx
        self._committed = False
        self._rolled_back = False

    async def commit(self):
        """Commit the transaction."""
        if not self._committed and not self._rolled_back:
            await self._tx.commit()
            self._committed = True

    async def rollback(self):
        """Rollback the transaction."""
        if not self._committed and not self._rolled_back:
            await self._tx.rollback()
            self._rolled_back = True

    @property
    def is_completed(self):
        """Check if transaction is committed or rolled back."""
        return self._committed or self._rolled_back

    # Delegate CRUD operations to the Rust transaction object
    async def insert_one(self, table: str, document: Dict[str, Any]) -> Dict[str, Any]:
        return await self._tx.insert_one(table, document)

    async def fetch_one(self, table: str, filter: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        return await self._tx.fetch_one(table, filter)

    async def update_one(self, table: str, pk_column: str, pk_value: Any, update: Dict[str, Any]) -> Any:
        return await self._tx.update_one(table, pk_column, pk_value, update)

    async def delete_one(self, table: str, pk_column: str, pk_value: Any) -> int:
        return await self._tx.delete_one(table, pk_column, pk_value)

    async def execute(self, sql: str, params: Optional[list] = None):
        return await self._tx.execute(sql, params)


@asynccontextmanager
async def begin_transaction(isolation_level: Optional[IsolationLevel] = None):
    """
    Begin a transaction with automatic rollback on error.
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available."
        )

    raw_tx = await _titan.begin_transaction(isolation_level)
    tx = TransactionWrapper(raw_tx)

    # Set as active transaction for this context
    token = _active_transaction.set(tx)

    try:
        yield tx
    except Exception:
        # Rollback on exception if not already committed/rolled back
        if not tx.is_completed:
            await tx.rollback()
        raise
    else:
        # Auto-commit if not explicitly committed or rolled back
        if not tx.is_completed:
            await tx.commit()
    finally:
        # Clear active transaction
        _active_transaction.reset(token)


async def execute(
    sql: str,
    params: Optional[list] = None
):
    """
    Execute raw SQL query. Uses active transaction if available.
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")

    tx = _active_transaction.get()
    if tx:
        return await tx.execute(sql, params)

    return await _titan.execute(sql, params)


async def insert_one(
    table: str,
    document: Dict[str, Any],
) -> Dict[str, Any]:
    """Insert a single document. Uses active transaction if available."""
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")

    tx = _active_transaction.get()
    if tx:
        return await tx.insert_one(table, document)

    return await _titan.insert_one(table, document)


# ============================================================================
# Schema Introspection
# ============================================================================


async def list_tables(schema: str = "public") -> List[str]:
    """
    List all tables in a schema.

    Args:
        schema: Schema name (default: "public")

    Returns:
        List of table names

    Example:
        >>> tables = await list_tables("public")
        >>> print(tables)
        ['users', 'posts', 'comments']

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.list_tables(schema)


async def table_exists(table: str, schema: str = "public") -> bool:
    """
    Check if a table exists in the database.

    Args:
        table: Table name
        schema: Schema name (default: "public")

    Returns:
        True if table exists, False otherwise

    Example:
        >>> exists = await table_exists("users", "public")
        >>> if exists:
        ...     print("Table exists")

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.table_exists(table, schema)


async def get_columns(table: str, schema: str = "public") -> List[Dict[str, Any]]:
    """
    Get column information for a table.

    Args:
        table: Table name
        schema: Schema name (default: "public")

    Returns:
        List of dictionaries with column information. Each dict contains:
        - name: Column name
        - data_type: PostgreSQL data type
        - nullable: Whether column allows NULL values
        - default: Default value expression (or None)
        - is_primary_key: Whether column is part of primary key
        - is_unique: Whether column has unique constraint

    Example:
        >>> columns = await get_columns("users", "public")
        >>> for col in columns:
        ...     print(f"{col['name']}: {col['data_type']}")
        id: Integer
        name: Varchar(255)
        email: Varchar(255)
        age: Integer

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.get_columns(table, schema)


async def get_indexes(table: str, schema: str = "public") -> List[Dict[str, Any]]:
    """
    Get index information for a table.

    Args:
        table: Table name
        schema: Schema name (default: "public")

    Returns:
        List of dictionaries with index information. Each dict contains:
        - name: Index name
        - columns: List of column names in the index
        - is_unique: Whether index enforces uniqueness
        - index_type: Index type (btree, hash, gin, gist, etc.)

    Example:
        >>> indexes = await get_indexes("users", "public")
        >>> for idx in indexes:
        ...     print(f"{idx['name']}: {idx['columns']}")
        users_pkey: ['id']
        idx_users_email: ['email']

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.get_indexes(table, schema)


async def get_foreign_keys(table: str, schema: str = "public") -> List[Dict[str, Any]]:
    """
    Get foreign key information for a table.

    Args:
        table: Table name
        schema: Schema name (default: "public")

    Returns:
        List of dictionaries with foreign key information. Each dict contains:
        - name: Foreign key constraint name
        - columns: List of column names in this table
        - referenced_table: Name of the referenced table
        - referenced_columns: List of referenced column names
        - on_delete: ON DELETE action (CASCADE, SET NULL, RESTRICT, NO ACTION)
        - on_update: ON UPDATE action (CASCADE, RESTRICT, NO ACTION)

    Example:
        >>> foreign_keys = await get_foreign_keys("posts", "public")
        >>> for fk in foreign_keys:
        ...     print(f"{fk['columns']} -> {fk['referenced_table']}.{fk['referenced_columns']}")
        ['author_id'] -> users.['id']

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.get_foreign_keys(table, schema)


async def find_by_foreign_key(table: str, foreign_key_column: str, foreign_key_value: Any) -> Optional[Dict[str, Any]]:
    """
    Find a single row by foreign key value.

    This is a convenience function for querying related objects via foreign keys.

    Args:
        table: Table name to query
        foreign_key_column: Column name to query by (usually "id")
        foreign_key_value: Value to match

    Returns:
        Dictionary with row data, or None if not found

    Example:
        >>> # Find user by ID
        >>> user = await find_by_foreign_key("users", "id", 123)
        >>> if user:
        ...     print(user["name"])

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.find_by_foreign_key(table, foreign_key_column, foreign_key_value)


async def fetch_one_with_relations(
    table: str,
    id: int,
    relations: List[Dict[str, Any]]
) -> Optional[Dict[str, Any]]:
    """
    Fetch a single row with related data using JOINs (eager loading).

    This uses SQL JOINs to efficiently fetch a row and its relations in a single query.

    Args:
        table: Table name
        id: Primary key value
        relations: List of relation configurations, each dict contains:
            - name: Name of the relation (how it appears in results)
            - table: Name of the related table
            - foreign_key: Foreign key column name
            - reference_column: Referenced column (default: "id")
            - join_type: JOIN type - "inner", "left", "right", or "full" (default: "left")
            - select_columns: Optional list of columns to select from related table

    Returns:
        Dictionary with row data including nested relations, or None if not found

    Example:
        >>> # Fetch user with their posts
        >>> user = await fetch_one_with_relations("users", 1, [
        ...     {
        ...         "name": "posts",
        ...         "table": "posts",
        ...         "foreign_key": "user_id",
        ...         "reference_column": "id",
        ...         "join_type": "left"
        ...     }
        ... ])
        >>> if user:
        ...     print(f"User: {user['name']}")
        ...     print(f"Posts: {user['posts']}")

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.fetch_one_with_relations(table, id, relations)


async def fetch_one_eager(
    table: str,
    id: int,
    joins: List[tuple]
) -> Optional[Dict[str, Any]]:
    """
    Simple eager loading - fetch one row with related data.

    This is a simplified version of fetch_one_with_relations that uses tuples
    instead of dictionaries for relation configuration.

    Args:
        table: Table name
        id: Primary key value
        joins: List of (relation_name, fk_column, ref_table) tuples

    Returns:
        Dictionary with row data including nested relations, or None if not found

    Example:
        >>> # Fetch user with posts and profile
        >>> user = await fetch_one_eager("users", 1, [
        ...     ("posts", "user_id", "posts"),
        ...     ("profile", "user_id", "profiles")
        ... ])
        >>> if user:
        ...     print(f"User: {user['name']}")
        ...     print(f"Posts: {user['posts']}")
        ...     print(f"Profile: {user['profile']}")

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.fetch_one_eager(table, id, joins)


async def fetch_many_with_relations(
    table: str,
    relations: List[Dict[str, Any]],
    filter: Optional[Dict[str, Any]] = None,
    order_by: Optional[tuple] = None,
    limit: Optional[int] = None,
    offset: Optional[int] = None,
) -> List[Dict[str, Any]]:
    """
    Fetch multiple rows with related data using JOINs.

    This efficiently fetches multiple rows and their relations in a single query.

    Args:
        table: Table name
        relations: List of relation configurations (same as fetch_one_with_relations)
        filter: Optional dictionary of WHERE conditions (simple equality only)
        order_by: Optional (column, direction) tuple for ordering
        limit: Optional maximum number of rows to return
        offset: Optional number of rows to skip

    Returns:
        List of dictionaries with row data including nested relations

    Example:
        >>> # Fetch all active users with their posts
        >>> users = await fetch_many_with_relations("users", [
        ...     {
        ...         "name": "posts",
        ...         "table": "posts",
        ...         "foreign_key": "user_id",
        ...         "reference_column": "id",
        ...         "join_type": "left"
        ...     }
        ... ], filter={"status": "active"}, limit=10)
        >>> for user in users:
        ...     print(f"{user['name']}: {len(user['posts'])} posts")

    Raises:
        RuntimeError: If PostgreSQL engine is not available or query fails
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.fetch_many_with_relations(
        table, relations, filter, order_by, limit, offset
    )


async def inspect_table(table: str, schema: str = "public") -> Dict[str, Any]:
    """
    Get complete information about a table including columns and indexes.

    Args:
        table: Table name
        schema: Schema name (default: "public")

    Returns:
        Dictionary with table information:
        - name: Table name
        - schema: Schema name
        - columns: List of column information (see get_columns)
        - indexes: List of index information (see get_indexes)
        - foreign_keys: List of foreign keys (currently empty - future work)

    Example:
        >>> info = await inspect_table("users", "public")
        >>> print(f"Table: {info['name']}")
        >>> print(f"Columns: {len(info['columns'])}")
        >>> print(f"Indexes: {len(info['indexes'])}")

    Raises:
        RuntimeError: If PostgreSQL engine is not available or table not found
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    return await _titan.inspect_table(table, schema)

# ============================================================================
# MIGRATION FUNCTIONS
# ============================================================================

async def migration_init() -> None:
    """Initialize the migration system (create _migrations table)."""
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    await _titan.migration_init()


async def migration_status(migrations_dir: str = "migrations") -> Dict[str, List[str]]:
    """Get migration status (applied and pending)."""
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return await _titan.migration_status(migrations_dir)


async def migration_apply(migrations_dir: str = "migrations") -> List[str]:
    """Apply all pending migrations."""
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return await _titan.migration_apply(migrations_dir)


async def migration_rollback(migrations_dir: str = "migrations", steps: int = 1) -> List[str]:
    """Rollback last N migrations."""
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return await _titan.migration_rollback(migrations_dir, steps)


def migration_create(description: str, migrations_dir: str = "migrations") -> str:
    """Create new migration file."""
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return _titan.migration_create(description, migrations_dir)


# ============================================================================
# CASCADE DELETE FUNCTIONS
# ============================================================================


async def delete_with_cascade(table: str, id: int, id_column: str = "id") -> int:
    """
    Delete a row with cascade handling based on foreign key rules.

    This manually handles ON DELETE rules:
    - CASCADE: Deletes child rows first
    - RESTRICT: Returns error if children exist
    - SET NULL: Sets FK to NULL before delete
    - SET DEFAULT: Sets FK to DEFAULT before delete

    Args:
        table: Table name to delete from
        id: Primary key value of row to delete
        id_column: Name of primary key column (default: "id")

    Returns:
        Total number of rows deleted (including cascaded children)

    Raises:
        RuntimeError: If RESTRICT constraint prevents deletion

    Example:
        >>> # Delete user and all their posts (CASCADE)
        >>> deleted = await delete_with_cascade("users", 1)
        >>> print(f"Deleted {deleted} rows total")
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return await _titan.delete_with_cascade(table, id, id_column)


async def delete_checked(table: str, id: int, id_column: str = "id") -> int:
    """
    Delete a row after checking RESTRICT constraints.

    Checks for RESTRICT/NO ACTION constraints and returns an error
    if children exist. For CASCADE, relies on database-level handling.

    Args:
        table: Table name to delete from
        id: Primary key value of row to delete
        id_column: Name of primary key column (default: "id")

    Returns:
        Number of rows deleted (1 if success, 0 if not found)

    Raises:
        RuntimeError: If RESTRICT constraint prevents deletion

    Example:
        >>> # Delete user only if no posts exist
        >>> try:
        ...     deleted = await delete_checked("users", 1)
        ...     print(f"User deleted")
        ... except RuntimeError as e:
        ...     print(f"Cannot delete: {e}")
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return await _titan.delete_checked(table, id, id_column)


async def get_backreferences(table: str, schema: str = None) -> list[dict]:
    """
    Get all tables that reference a given table (back-references).

    Useful for understanding relationships before delete operations.

    Args:
        table: Table name to find references to
        schema: Schema name (default: "public")

    Returns:
        List of dicts with keys:
        - source_table: Table that references this table
        - source_column: FK column in source table
        - target_table: This table
        - target_column: Referenced column (usually "id")
        - constraint_name: Name of FK constraint
        - on_delete: DELETE rule ("CASCADE", "RESTRICT", etc.)
        - on_update: UPDATE rule

    Example:
        >>> # Find all tables that reference users
        >>> backrefs = await get_backreferences("users")
        >>> for ref in backrefs:
        ...     print(f"{ref['source_table']}.{ref['source_column']} -> {ref['target_table']}")
        ...     print(f"  ON DELETE {ref['on_delete']}")
    """
    if _engine is None:
        raise RuntimeError("PostgreSQL engine not available.")
    return await _titan.get_backreferences(table, schema)
