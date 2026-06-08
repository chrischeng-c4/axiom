"""
Query builder for chainable PostgreSQL queries.

This module provides a thin Python wrapper around the Rust QueryBuilder,
enabling fluent query building with full type safety and performance.

Example:
    >>> users = await User.find(User.age > 25) \\
    ...     .order_by(-User.created_at) \\
    ...     .limit(20) \\
    ...     .to_list()
"""

from __future__ import annotations

from typing import Any, Generic, List, Optional, Type, TypeVar, TYPE_CHECKING, Union

from .columns import SqlExpr
from .telemetry import create_query_span, set_span_result, add_exception, is_tracing_enabled

if TYPE_CHECKING:
    from .table import Table
    from .columns import ColumnProxy
    from .options import QueryOption

# Import Rust bindings
try:
    from ..cclab import titan as _titan
    RustQueryBuilder = _titan.RustQueryBuilder
    RustWindowSpec = _titan.WindowSpec
except ImportError:
    _titan = None
    RustQueryBuilder = None
    RustWindowSpec = None


T = TypeVar("T", bound="Table")


class WindowSpec:
    """Window specification for PARTITION BY and ORDER BY."""

    def __init__(self):
        self._partition_by: list[str] = []
        self._order_by: list[tuple[str, str]] = []

    def partition_by(self, *columns: Union[str, "ColumnProxy"]) -> "WindowSpec":
        """Add PARTITION BY columns."""
        spec = WindowSpec()
        spec._partition_by = self._partition_by.copy()
        spec._order_by = self._order_by.copy()
        for col in columns:
            col_name = col.name if hasattr(col, 'name') else col
            spec._partition_by.append(col_name)
        return spec

    def order_by(self, column: Union[str, "ColumnProxy"], direction: str = "asc") -> "WindowSpec":
        """Add ORDER BY column."""
        spec = WindowSpec()
        spec._partition_by = self._partition_by.copy()
        spec._order_by = self._order_by.copy()
        col_name = column.name if hasattr(column, 'name') else column
        spec._order_by.append((col_name, direction))
        return spec

    def _to_rust(self) -> "RustWindowSpec":
        """Convert to Rust WindowSpec."""
        if RustWindowSpec is None:
            raise RuntimeError("Rust bindings not available")
        rust_spec = RustWindowSpec()
        if self._partition_by:
            rust_spec = rust_spec.partition_by(self._partition_by)
        for col, direction in self._order_by:
            rust_spec = rust_spec.order_by(col, direction)
        return rust_spec


class QueryBuilder(Generic[T]):
    """
    Chainable query builder wrapping Rust QueryBuilder.

    Provides a fluent API for building and executing queries.
    All terminal operations (to_list, first, count, exists, aggregate) are async.
    """

    def __init__(
        self,
        model: Type[T],
        filters: tuple,
        _rust_builder: Optional["RustQueryBuilder"] = None,
        _options: Optional[List['QueryOption']] = None,
    ) -> None:
        self._model = model
        self._filters = filters
        self._options: list['QueryOption'] = _options or []

        # Initialize Rust builder
        if _rust_builder is not None:
            self._rust = _rust_builder
        else:
            table_name = model.__table_name__()
            self._rust = RustQueryBuilder(table_name)
            # Apply filters
            self._apply_filters(filters)

    def _apply_filters(self, filters: tuple) -> None:
        """Apply filter expressions to Rust builder."""
        for filter_item in filters:
            if isinstance(filter_item, SqlExpr):
                op_map = {
                    "=": "eq", "!=": "ne", ">": "gt", ">=": "gte",
                    "<": "lt", "<=": "lte", "LIKE": "like", "ILIKE": "ilike",
                    "IN": "in", "IS NULL": "is_null", "IS NOT NULL": "is_not_null",
                }
                operator = op_map.get(filter_item.op, filter_item.op.lower())
                self._rust = self._rust.where_clause(
                    filter_item.column, operator, filter_item.value
                )
            elif isinstance(filter_item, dict):
                for key, value in filter_item.items():
                    self._rust = self._rust.where_clause(key, "eq", value)

    def _clone(self, rust_builder: "RustQueryBuilder") -> "QueryBuilder[T]":
        """Create a new QueryBuilder with updated Rust builder."""
        return QueryBuilder(
            model=self._model,
            filters=self._filters,
            _rust_builder=rust_builder,
            _options=self._options.copy(),
        )

    # =========================================================================
    # Chainable Methods - delegate to Rust
    # =========================================================================

    def order_by(self, *fields: Union["ColumnProxy", str]) -> "QueryBuilder[T]":
        """Add ordering to the query."""
        rust = self._rust
        for field in fields:
            if isinstance(field, str):
                if field.startswith("-"):
                    rust = rust.order_by(field[1:], "desc")
                else:
                    rust = rust.order_by(field, "asc")
            elif hasattr(field, "name"):
                rust = rust.order_by(field.name, "asc")
        return self._clone(rust)

    def offset(self, count: int) -> "QueryBuilder[T]":
        """Skip the first N rows."""
        return self._clone(self._rust.offset(count))

    def limit(self, count: int) -> "QueryBuilder[T]":
        """Limit the number of rows returned."""
        return self._clone(self._rust.limit(count))

    def select(self, *columns: Union["ColumnProxy", str]) -> "QueryBuilder[T]":
        """Select specific columns to return."""
        cols = [c.name if hasattr(c, 'name') else c for c in columns]
        return self._clone(self._rust.select(cols))

    def distinct(self) -> "QueryBuilder[T]":
        """Return only distinct (unique) rows."""
        return self._clone(self._rust.distinct())

    def distinct_on(self, *columns: Union[str, "ColumnProxy"]) -> "QueryBuilder[T]":
        """Return first row for each unique combination of columns."""
        cols = [c.name if hasattr(c, 'name') else c for c in columns]
        return self._clone(self._rust.distinct_on(cols))

    # =========================================================================
    # Aggregate Methods
    # =========================================================================

    def count_agg(self, alias: Optional[str] = None) -> "QueryBuilder[T]":
        """Add COUNT(*) aggregate."""
        return self._clone(self._rust.count_agg(alias))

    def count_distinct(self, column: Union["ColumnProxy", str], alias: Optional[str] = None) -> "QueryBuilder[T]":
        """Add COUNT(DISTINCT column) aggregate."""
        col = column.name if hasattr(column, 'name') else column
        return self._clone(self._rust.count_distinct(col, alias))

    def sum(self, column: Union["ColumnProxy", str], alias: Optional[str] = None) -> "QueryBuilder[T]":
        """Add SUM(column) aggregate."""
        col = column.name if hasattr(column, 'name') else column
        return self._clone(self._rust.sum(col, alias))

    def avg(self, column: Union["ColumnProxy", str], alias: Optional[str] = None) -> "QueryBuilder[T]":
        """Add AVG(column) aggregate."""
        col = column.name if hasattr(column, 'name') else column
        return self._clone(self._rust.avg(col, alias))

    def min(self, column: Union["ColumnProxy", str], alias: Optional[str] = None) -> "QueryBuilder[T]":
        """Add MIN(column) aggregate."""
        col = column.name if hasattr(column, 'name') else column
        return self._clone(self._rust.min(col, alias))

    def max(self, column: Union["ColumnProxy", str], alias: Optional[str] = None) -> "QueryBuilder[T]":
        """Add MAX(column) aggregate."""
        col = column.name if hasattr(column, 'name') else column
        return self._clone(self._rust.max(col, alias))

    def group_by(self, *columns: Union["ColumnProxy", str]) -> "QueryBuilder[T]":
        """Add GROUP BY columns."""
        cols = [c.name if hasattr(c, 'name') else c for c in columns]
        return self._clone(self._rust.group_by(cols))

    # =========================================================================
    # HAVING Methods
    # =========================================================================

    def having_count(self, operator: str, value: int) -> "QueryBuilder[T]":
        """Add HAVING COUNT(*) condition."""
        return self._clone(self._rust.having_count(operator, value))

    def having_sum(self, column: Union[str, "ColumnProxy"], operator: str, value: Union[float, int]) -> "QueryBuilder[T]":
        """Add HAVING SUM(column) condition."""
        col = column.name if hasattr(column, 'name') else column
        return self._clone(self._rust.having_sum(col, operator, value))

    # =========================================================================
    # Window Functions
    # =========================================================================

    def row_number(self, alias: str, spec: WindowSpec | None = None) -> "QueryBuilder[T]":
        """Add ROW_NUMBER() window function."""
        rust_spec = spec._to_rust() if spec else RustWindowSpec()
        return self._clone(self._rust.row_number(rust_spec, alias))

    def rank(self, alias: str, spec: WindowSpec | None = None) -> "QueryBuilder[T]":
        """Add RANK() window function."""
        rust_spec = spec._to_rust() if spec else RustWindowSpec()
        return self._clone(self._rust.rank(rust_spec, alias))

    def lag(
        self,
        column: Union[str, "ColumnProxy"],
        alias: str,
        offset: int = 1,
        default: Any = None,
        spec: WindowSpec | None = None,
    ) -> "QueryBuilder[T]":
        """Add LAG() window function."""
        col = column.name if hasattr(column, 'name') else column
        rust_spec = spec._to_rust() if spec else RustWindowSpec()
        return self._clone(self._rust.lag(col, rust_spec, alias, offset, default))

    def lead(
        self,
        column: Union[str, "ColumnProxy"],
        alias: str,
        offset: int = 1,
        default: Any = None,
        spec: WindowSpec | None = None,
    ) -> "QueryBuilder[T]":
        """Add LEAD() window function."""
        col = column.name if hasattr(column, 'name') else column
        rust_spec = spec._to_rust() if spec else RustWindowSpec()
        return self._clone(self._rust.lead(col, rust_spec, alias, offset, default))

    def window_sum(self, column: Union[str, "ColumnProxy"], alias: str, spec: WindowSpec | None = None) -> "QueryBuilder[T]":
        """Add SUM() as window function."""
        col = column.name if hasattr(column, 'name') else column
        rust_spec = spec._to_rust() if spec else RustWindowSpec()
        return self._clone(self._rust.window_sum(col, rust_spec, alias))

    # =========================================================================
    # CTE Methods
    # =========================================================================

    def with_cte_raw(self, name: str, sql: str, params: Optional[List[Any]] = None) -> "QueryBuilder[T]":
        """Add a raw SQL CTE."""
        return self._clone(self._rust.with_cte_raw(name, sql, params or []))

    @classmethod
    def from_cte(cls, cte_name: str, cte_query: "QueryBuilder[Any]", model: Optional[Type[T]] = None) -> "QueryBuilder[T]":
        """Create a QueryBuilder that queries from a CTE."""
        result_model = model if model is not None else cte_query._model
        rust = RustQueryBuilder.from_cte(cte_name, cte_query._rust)
        return cls(model=result_model, filters=(), _rust_builder=rust)

    # =========================================================================
    # Set Operations
    # =========================================================================

    def union(self, other: "QueryBuilder[Any]") -> "QueryBuilder[T]":
        """Combine with UNION."""
        return self._clone(self._rust.union(other._rust))

    def union_all(self, other: "QueryBuilder[Any]") -> "QueryBuilder[T]":
        """Combine with UNION ALL."""
        return self._clone(self._rust.union_all(other._rust))

    # =========================================================================
    # Query Options
    # =========================================================================

    def options(self, *options: 'QueryOption') -> "QueryBuilder[T]":
        """Specify eager loading options."""
        new_qb = self._clone(self._rust)
        new_qb._options.extend(options)
        return new_qb

    # =========================================================================
    # Terminal Methods (async)
    # =========================================================================

    async def aggregate(self) -> List[dict]:
        """Execute aggregate query and return results."""
        if not is_tracing_enabled():
            rows = await self._rust.fetch_all()
            return list(rows)

        table_name = self._model.__table_name__()
        with create_query_span(operation="aggregate", table=table_name) as span:
            try:
                rows = await self._rust.fetch_all()
                result = list(rows)
                set_span_result(span, count=len(result))
                return result
            except Exception as e:
                add_exception(span, e)
                raise

    async def to_list(self) -> List[T]:
        """Execute query and return all matching rows."""
        if not is_tracing_enabled():
            rows = await self._rust.fetch_all()
            instances = [self._model(**row) for row in rows]
            for option in self._options:
                await option.apply(instances)
            return instances

        table_name = self._model.__table_name__()
        with create_query_span(operation="find", table=table_name) as span:
            try:
                rows = await self._rust.fetch_all()
                instances = [self._model(**row) for row in rows]
                for option in self._options:
                    await option.apply(instances)
                set_span_result(span, count=len(instances))
                return instances
            except Exception as e:
                add_exception(span, e)
                raise

    async def first(self) -> Optional[T]:
        """Execute query and return the first matching row."""
        if not is_tracing_enabled():
            row = await self._rust.fetch_one()
            return self._model(**row) if row else None

        table_name = self._model.__table_name__()
        with create_query_span(operation="find_one", table=table_name) as span:
            try:
                row = await self._rust.fetch_one()
                result = self._model(**row) if row else None
                set_span_result(span, count=1 if result else 0)
                return result
            except Exception as e:
                add_exception(span, e)
                raise

    async def count(self) -> int:
        """Count the number of matching rows."""
        # Use COUNT(*) aggregate
        rust = self._rust.count_agg("_count")
        rows = await rust.fetch_all()
        if rows:
            return rows[0].get("_count", 0)
        return 0

    async def exists(self) -> bool:
        """Check if any rows match the query."""
        count = await self.count()
        return count > 0
