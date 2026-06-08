from typing import Any

__all__ = ["Transaction", "RustQueryBuilder", "WindowSpec", "PyOrderDirection", "PyJoinType", "PyIsolationLevel", "PyAccessMode", "PoolConfig", "begin_transaction", "list_tables", "table_exists", "get_columns", "get_indexes", "get_foreign_keys", "inspect_table", "init", "close", "is_connected", "ping", "execute", "query", "query_one"]

class Transaction:
    """Python wrapper for PostgreSQL transactions.

Supports ACID transactions with savepoint support."""
    def commit(self) -> Any:
        """Commit the transaction."""
        ...
    def rollback(self) -> Any:
        """Rollback the transaction."""
        ...
    def savepoint(self, name: str) -> Any:
        """Create a savepoint."""
        ...
    def rollback_to(self, name: str) -> Any:
        """Rollback to a savepoint."""
        ...
    def release_savepoint(self, name: str) -> Any:
        """Release a savepoint."""
        ...
    def execute(self, sql: str, params: list[Any] = []) -> Any:
        """Execute a SQL statement within the transaction."""
        ...
    def query(self, sql: str, params: list[Any] = []) -> Any:
        """Query within the transaction."""
        ...
    def __repr__(self) -> str:
        ...

class RustQueryBuilder:
    """Fluent SQL query builder.

Supports SELECT, WHERE, JOIN, ORDER BY, GROUP BY, window functions, CTEs, and more."""
    def __init__(self, table: str) -> None:
        ...
    @property
    def table(self) -> str:
        """Get the table name."""
        ...
    def select(self, columns: list[str]) -> Any:
        """Select specific columns."""
        ...
    def distinct(self) -> Any:
        """Add DISTINCT to the query."""
        ...
    def distinct_on(self, columns: list[str]) -> Any:
        """Add DISTINCT ON (PostgreSQL-specific)."""
        ...
    def where_clause(self, field: str, operator: str, value: Any) -> Any:
        """Add a WHERE condition."""
        ...
    def where_null(self, field: str) -> Any:
        """Add WHERE field IS NULL."""
        ...
    def where_not_null(self, field: str) -> Any:
        """Add WHERE field IS NOT NULL."""
        ...
    def join(self, join_type: Any, table: str, left_col: str, right_col: str, alias: str | None = None) -> Any:
        """Add a JOIN clause."""
        ...
    def inner_join(self, table: str, left_col: str, right_col: str, alias: str | None = None) -> Any:
        """Add an INNER JOIN."""
        ...
    def left_join(self, table: str, left_col: str, right_col: str, alias: str | None = None) -> Any:
        """Add a LEFT JOIN."""
        ...
    def right_join(self, table: str, left_col: str, right_col: str, alias: str | None = None) -> Any:
        """Add a RIGHT JOIN."""
        ...
    def order_by(self, field: str, direction: Any | None = None) -> Any:
        """Add ORDER BY clause."""
        ...
    def order_by_asc(self, field: str) -> Any:
        """Add ORDER BY field ASC."""
        ...
    def order_by_desc(self, field: str) -> Any:
        """Add ORDER BY field DESC."""
        ...
    def limit(self, limit: int) -> Any:
        """Set LIMIT clause."""
        ...
    def offset(self, offset: int) -> Any:
        """Set OFFSET clause."""
        ...
    def count_agg(self, alias: str | None = None) -> Any:
        """Add COUNT(*) aggregate."""
        ...
    def count_distinct(self, column: str, alias: str | None = None) -> Any:
        """Add COUNT(DISTINCT column) aggregate."""
        ...
    def sum(self, column: str, alias: str | None = None) -> Any:
        """Add SUM(column) aggregate."""
        ...
    def avg(self, column: str, alias: str | None = None) -> Any:
        """Add AVG(column) aggregate."""
        ...
    def min(self, column: str, alias: str | None = None) -> Any:
        """Add MIN(column) aggregate."""
        ...
    def max(self, column: str, alias: str | None = None) -> Any:
        """Add MAX(column) aggregate."""
        ...
    def group_by(self, columns: list[str]) -> Any:
        """Add GROUP BY columns."""
        ...
    def having_count(self, operator: str, value: Any) -> Any:
        """Add HAVING COUNT(*) condition."""
        ...
    def having_sum(self, column: str, operator: str, value: Any) -> Any:
        """Add HAVING SUM(column) condition."""
        ...
    def row_number(self, spec: Any, alias: str) -> Any:
        """Add ROW_NUMBER() window function."""
        ...
    def rank(self, spec: Any, alias: str) -> Any:
        """Add RANK() window function."""
        ...
    def dense_rank(self, spec: Any, alias: str) -> Any:
        """Add DENSE_RANK() window function."""
        ...
    def lag(self, column: str, spec: Any, alias: str, offset: int | None = None, default: Any | None = None) -> Any:
        """Add LAG() window function."""
        ...
    def lead(self, column: str, spec: Any, alias: str, offset: int | None = None, default: Any | None = None) -> Any:
        """Add LEAD() window function."""
        ...
    def with_cte_raw(self, name: str, sql: str, params: list[Any] = []) -> Any:
        """Add a raw SQL CTE (Common Table Expression)."""
        ...
    @staticmethod
    def from_cte(name: str, cte_builder: Any) -> Any:
        """Create a QueryBuilder that queries from a CTE."""
        ...
    def union(self, other: Any) -> Any:
        """Combine with UNION (removes duplicates)."""
        ...
    def union_all(self, other: Any) -> Any:
        """Combine with UNION ALL (keeps duplicates)."""
        ...
    def build_select(self) -> tuple[str, Any]:
        """Build the SELECT query, returning (sql, params)."""
        ...
    def build(self) -> tuple[str, Any]:
        """Build the query (alias for build_select)."""
        ...
    def __repr__(self) -> str:
        ...

class WindowSpec:
    """Window specification for PARTITION BY and ORDER BY."""
    def __init__(self) -> None:
        ...
    def partition_by(self, columns: list[str]) -> Any:
        """Add PARTITION BY columns."""
        ...
    def order_by(self, column: str, direction: Any | None = None) -> Any:
        """Add ORDER BY column."""
        ...
    def order_by_asc(self, column: str) -> Any:
        """Add ORDER BY column ascending."""
        ...
    def order_by_desc(self, column: str) -> Any:
        """Add ORDER BY column descending."""
        ...
    def __repr__(self) -> str:
        ...

class PyOrderDirection:
    @staticmethod
    def asc() -> Any:
        """Create ascending order."""
        ...
    @staticmethod
    def desc() -> Any:
        """Create descending order."""
        ...
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PyJoinType:
    @staticmethod
    def inner() -> Any:
        ...
    @staticmethod
    def left() -> Any:
        ...
    @staticmethod
    def right() -> Any:
        ...
    @staticmethod
    def full() -> Any:
        ...
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PyIsolationLevel:
    @staticmethod
    def read_uncommitted() -> Any:
        ...
    @staticmethod
    def read_committed() -> Any:
        ...
    @staticmethod
    def repeatable_read() -> Any:
        ...
    @staticmethod
    def serializable() -> Any:
        ...
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PyAccessMode:
    @staticmethod
    def read_write() -> Any:
        ...
    @staticmethod
    def read_only() -> Any:
        ...
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PoolConfig:
    """Pool configuration for Python."""
    min_connections: int
    max_connections: int
    connect_timeout: int
    max_lifetime: int | None
    idle_timeout: int | None
    statement_cache_capacity: int
    def __init__(self, min_connections: int = 1, max_connections: int = 10, connect_timeout: int = 30, max_lifetime: int | None = 1800, idle_timeout: int | None = 600, statement_cache_capacity: int = 100) -> None:
        ...
    @staticmethod
    def default_config() -> Any:
        """Create default configuration."""
        ...
    def __repr__(self) -> str:
        ...

def begin_transaction(isolation_level: Any | None = None) -> Any:
    """Begin a new transaction."""
    ...

def list_tables(schema: str | None = None) -> Any:
    """List all tables in the specified schema (defaults to "public")."""
    ...

def table_exists(table: str, schema: str | None = None) -> Any:
    """Check if a table exists in the specified schema."""
    ...

def get_columns(table: str, schema: str | None = None) -> Any:
    """Get column information for a table."""
    ...

def get_indexes(table: str, schema: str | None = None) -> Any:
    """Get index information for a table."""
    ...

def get_foreign_keys(table: str, schema: str | None = None) -> Any:
    """Get foreign key information for a table."""
    ...

def inspect_table(table: str, schema: str | None = None) -> Any:
    """Get detailed information about a table."""
    ...

def init(uri: str, config: Any | None = None) -> Any:
    """Initialize the PostgreSQL connection pool.

# Arguments

* `uri` - PostgreSQL connection string
* `config` - Optional pool configuration

# Example

```python
await init("postgresql://user:pass@localhost/db")
```"""
    ...

def close() -> Any:
    """Close the PostgreSQL connection pool."""
    ...

def is_connected() -> bool:
    """Check if the database connection is initialized."""
    ...

def ping() -> Any:
    """Ping the database to verify connectivity."""
    ...

def execute(sql: str, params: list[Any] = []) -> Any:
    """Execute a SQL statement (INSERT, UPDATE, DELETE).

Returns the number of rows affected."""
    ...

def query(sql: str, params: list[Any] = []) -> Any:
    """Execute a query and return all rows."""
    ...

def query_one(sql: str, params: list[Any] = []) -> Any:
    """Execute a query and return the first row, or None."""
    ...

