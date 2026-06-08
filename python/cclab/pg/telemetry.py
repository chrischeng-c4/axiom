"""
OpenTelemetry integration for data-bridge PostgreSQL ORM.

Provides tracing and metrics instrumentation for database operations:
- Tracer configuration with service name "data-bridge"
- Instrumentation decorators for spans (@instrument_span, @instrument_query, @instrument_session)
- Span helper functions with semantic conventions
- Metrics configuration for connection pool monitoring
- Graceful degradation when OpenTelemetry SDK is not installed

Example:
    >>> from cclab.pg.telemetry import instrument_span, create_query_span
    >>>
    >>> @instrument_span("db.query.execute")
    >>> async def execute_query(query):
    ...     with create_query_span("find", "users", filters_count=2):
    ...         result = await _execute(query)
    ...     return result
"""

from __future__ import annotations

import functools
import os
from contextlib import contextmanager
from typing import Any, Callable, Dict, Optional, TypeVar, TYPE_CHECKING

# Try to import OpenTelemetry SDK
try:
    from opentelemetry import trace, metrics
    from opentelemetry.trace import Status, StatusCode, Span, Tracer
    from opentelemetry.metrics import Meter
    OTEL_AVAILABLE = True
except ImportError:
    OTEL_AVAILABLE = False
    # Define stub types for type hints when OpenTelemetry is not installed
    if TYPE_CHECKING:
        from opentelemetry.trace import Span, Tracer
        from opentelemetry.metrics import Meter


T = TypeVar('T')
F = TypeVar('F', bound=Callable[..., Any])


# ============================================================================
# Configuration
# ============================================================================

# Service name for tracer
SERVICE_NAME = "data-bridge"

# Environment variable to enable/disable tracing
TRACING_ENABLED_ENV = "DATA_BRIDGE_TRACING_ENABLED"

# Database system constant (PostgreSQL)
DB_SYSTEM = "postgresql"


def is_tracing_enabled() -> bool:
    """
    Check if tracing is enabled.

    Returns True if:
    1. OpenTelemetry SDK is installed
    2. Environment variable DATA_BRIDGE_TRACING_ENABLED is not set to "false" or "0"

    Returns:
        bool: True if tracing is enabled, False otherwise.
    """
    if not OTEL_AVAILABLE:
        return False

    env_value = os.environ.get(TRACING_ENABLED_ENV, "true").lower()
    return env_value not in ("false", "0", "no")


def get_tracer() -> Optional['Tracer']:
    """
    Get the global tracer for data-bridge.

    Returns:
        Optional[Tracer]: Tracer instance if available, None otherwise.
    """
    if not OTEL_AVAILABLE or not is_tracing_enabled():
        return None

    return trace.get_tracer(__name__, SERVICE_NAME)


def get_meter() -> Optional['Meter']:
    """
    Get the global meter for data-bridge metrics.

    Returns:
        Optional[Meter]: Meter instance if available, None otherwise.
    """
    if not OTEL_AVAILABLE or not is_tracing_enabled():
        return None

    return metrics.get_meter(__name__, SERVICE_NAME)


# ============================================================================
# Semantic Conventions
# ============================================================================

class SpanAttributes:
    """OpenTelemetry semantic conventions for database operations."""

    # Database attributes
    DB_SYSTEM = "db.system"
    DB_OPERATION_NAME = "db.operation.name"
    DB_COLLECTION_NAME = "db.collection.name"  # Table name
    DB_STATEMENT = "db.statement"

    # Query attributes
    DB_QUERY_FILTERS_COUNT = "db.query.filters_count"
    DB_QUERY_LIMIT = "db.query.limit"
    DB_QUERY_OFFSET = "db.query.offset"
    DB_QUERY_ORDER_BY = "db.query.order_by"

    # Result attributes
    DB_RESULT_COUNT = "db.result.count"
    DB_RESULT_AFFECTED_ROWS = "db.result.affected_rows"

    # Session attributes
    DB_SESSION_OPERATION = "db.session.operation"
    DB_SESSION_PENDING_COUNT = "db.session.pending_count"
    DB_SESSION_DIRTY_COUNT = "db.session.dirty_count"
    DB_SESSION_DELETED_COUNT = "db.session.deleted_count"

    # Relationship attributes
    DB_RELATIONSHIP_NAME = "db.relationship.name"
    DB_RELATIONSHIP_TARGET_MODEL = "db.relationship.target_model"
    DB_RELATIONSHIP_STRATEGY = "db.relationship.strategy"
    DB_RELATIONSHIP_FK_COLUMN = "db.relationship.fk_column"
    DB_RELATIONSHIP_CACHE_HIT = "db.relationship.cache_hit"
    DB_RELATIONSHIP_BATCH_COUNT = "db.relationship.batch_count"
    DB_RELATIONSHIP_DEPTH = "db.relationship.depth"


class MetricNames:
    """Metric names for connection pool monitoring."""

    # Connection pool metrics
    CONNECTION_POOL_IN_USE = "db.connection.pool.in_use"
    CONNECTION_POOL_IDLE = "db.connection.pool.idle"
    CONNECTION_POOL_MAX = "db.connection.pool.max"

    # Query metrics
    QUERY_DURATION = "db.query.duration"
    QUERY_COUNT = "db.query.count"


# ============================================================================
# Span Helper Functions
# ============================================================================

@contextmanager
def create_query_span(
    operation: str,
    table: Optional[str] = None,
    filters_count: Optional[int] = None,
    limit: Optional[int] = None,
    offset: Optional[int] = None,
    order_by: Optional[str] = None,
    statement: Optional[str] = None,
    **attributes: Any
):
    """
    Create a query span with standard database attributes.

    Args:
        operation: Database operation name (e.g., "find", "insert", "update", "delete")
        table: Table name being queried
        filters_count: Number of filters in the query
        limit: Query limit value
        offset: Query offset value
        order_by: Order by clause
        statement: SQL statement (optional, use with caution for cardinality)
        **attributes: Additional custom attributes

    Yields:
        Span: The created span, or a no-op context if tracing is disabled.

    Example:
        >>> with create_query_span("find", "users", filters_count=2, limit=10):
        ...     result = await db.query(...)
    """
    tracer = get_tracer()
    if tracer is None:
        yield None
        return

    # Use low-cardinality span name
    span_name = f"db.query.{operation}"

    # Build attributes
    span_attributes = {
        SpanAttributes.DB_SYSTEM: DB_SYSTEM,
        SpanAttributes.DB_OPERATION_NAME: operation,
    }

    if table is not None:
        span_attributes[SpanAttributes.DB_COLLECTION_NAME] = table

    if filters_count is not None:
        span_attributes[SpanAttributes.DB_QUERY_FILTERS_COUNT] = filters_count

    if limit is not None:
        span_attributes[SpanAttributes.DB_QUERY_LIMIT] = limit

    if offset is not None:
        span_attributes[SpanAttributes.DB_QUERY_OFFSET] = offset

    if order_by is not None:
        span_attributes[SpanAttributes.DB_QUERY_ORDER_BY] = order_by

    if statement is not None:
        span_attributes[SpanAttributes.DB_STATEMENT] = statement

    # Add custom attributes
    span_attributes.update(attributes)

    with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
        yield span


@contextmanager
def create_session_span(
    operation: str,
    pending_count: Optional[int] = None,
    dirty_count: Optional[int] = None,
    deleted_count: Optional[int] = None,
    **attributes: Any
):
    """
    Create a session span with standard session attributes.

    Args:
        operation: Session operation name (e.g., "flush", "commit", "rollback")
        pending_count: Number of pending objects
        dirty_count: Number of dirty objects
        deleted_count: Number of deleted objects
        **attributes: Additional custom attributes

    Yields:
        Span: The created span, or a no-op context if tracing is disabled.

    Example:
        >>> with create_session_span("flush", pending_count=5, dirty_count=3):
        ...     await session.flush()
    """
    tracer = get_tracer()
    if tracer is None:
        yield None
        return

    # Use low-cardinality span name
    span_name = f"db.session.{operation}"

    # Build attributes
    span_attributes = {
        SpanAttributes.DB_SYSTEM: DB_SYSTEM,
        SpanAttributes.DB_SESSION_OPERATION: operation,
    }

    if pending_count is not None:
        span_attributes[SpanAttributes.DB_SESSION_PENDING_COUNT] = pending_count

    if dirty_count is not None:
        span_attributes[SpanAttributes.DB_SESSION_DIRTY_COUNT] = dirty_count

    if deleted_count is not None:
        span_attributes[SpanAttributes.DB_SESSION_DELETED_COUNT] = deleted_count

    # Add custom attributes
    span_attributes.update(attributes)

    with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
        yield span


@contextmanager
def create_relationship_span(
    name: str,
    target_model: Optional[str] = None,
    strategy: Optional[str] = None,
    fk_column: Optional[str] = None,
    batch_count: Optional[int] = None,
    depth: int = 0,
    **attributes: Any
):
    """
    Create a relationship loading span.

    Args:
        name: Relationship name
        target_model: Target model class name
        strategy: Loading strategy (e.g., "select", "selectinload", "joined")
        fk_column: Foreign key column name
        batch_count: Number of instances being batch loaded (for eager loading)
        depth: Nesting depth of relationship loading
        **attributes: Additional custom attributes

    Yields:
        Span: The created span, or a no-op context if tracing is disabled.

    Example:
        >>> with create_relationship_span("author", target_model="User", strategy="select"):
        ...     author = await load_relationship(...)
        >>> # Eager loading
        >>> with create_relationship_span("author", target_model="User",
        ...                               strategy="selectinload", batch_count=100):
        ...     authors = await batch_load_relationships(...)
    """
    tracer = get_tracer()
    if tracer is None:
        yield None
        return

    # Use low-cardinality span name (just the strategy, not the relationship name)
    span_name = f"db.relationship.{strategy}" if strategy else "db.relationship.load"

    # Build attributes
    span_attributes = {
        SpanAttributes.DB_SYSTEM: DB_SYSTEM,
        SpanAttributes.DB_RELATIONSHIP_NAME: name,
        SpanAttributes.DB_RELATIONSHIP_DEPTH: depth,
    }

    if target_model is not None:
        span_attributes[SpanAttributes.DB_RELATIONSHIP_TARGET_MODEL] = target_model

    if strategy is not None:
        span_attributes[SpanAttributes.DB_RELATIONSHIP_STRATEGY] = strategy

    if fk_column is not None:
        span_attributes[SpanAttributes.DB_RELATIONSHIP_FK_COLUMN] = fk_column

    if batch_count is not None:
        span_attributes[SpanAttributes.DB_RELATIONSHIP_BATCH_COUNT] = batch_count

    # Add custom attributes
    span_attributes.update(attributes)

    with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
        yield span


def add_exception(span: Optional['Span'], exception: Exception) -> None:
    """
    Add exception information to a span.

    Args:
        span: The span to add exception to
        exception: The exception to record

    Example:
        >>> with create_query_span("find", "users") as span:
        ...     try:
        ...         result = await db.query(...)
        ...     except Exception as e:
        ...         add_exception(span, e)
        ...         raise
    """
    if span is None or not OTEL_AVAILABLE:
        return

    span.record_exception(exception)
    span.set_status(Status(StatusCode.ERROR, str(exception)))


def set_span_result(span: Optional['Span'], count: Optional[int] = None, affected_rows: Optional[int] = None, cache_hit: Optional[bool] = None, **kwargs: Any) -> None:
    """
    Set result attributes on a span.

    Args:
        span: The span to set attributes on
        count: Number of results returned
        affected_rows: Number of rows affected by the operation
        cache_hit: Whether the result was served from cache (for relationships)
        **kwargs: Additional custom attributes to set

    Example:
        >>> with create_query_span("find", "users") as span:
        ...     result = await db.query(...)
        ...     set_span_result(span, count=len(result))
        >>> # Session usage
        >>> with create_session_span("commit") as span:
        ...     await session.commit()
        ...     set_span_result(span, status="committed")
        >>> # Relationship loading
        >>> with create_relationship_span("author", strategy="select") as span:
        ...     author = await load(...)
        ...     set_span_result(span, count=1, cache_hit=True)
    """
    if span is None or not OTEL_AVAILABLE:
        return

    if count is not None:
        span.set_attribute(SpanAttributes.DB_RESULT_COUNT, count)

    if affected_rows is not None:
        span.set_attribute(SpanAttributes.DB_RESULT_AFFECTED_ROWS, affected_rows)

    if cache_hit is not None:
        span.set_attribute(SpanAttributes.DB_RELATIONSHIP_CACHE_HIT, cache_hit)

    # Set additional custom attributes
    for key, value in kwargs.items():
        if value is not None:
            span.set_attribute(key, value)


# ============================================================================
# Instrumentation Decorators
# ============================================================================

def instrument_span(
    name: Optional[str] = None,
    attributes: Optional[Dict[str, Any]] = None
) -> Callable[[F], F]:
    """
    Decorator to instrument a function with a span.

    Supports both sync and async functions with minimal overhead (<1ms).

    Args:
        name: Span name (defaults to function name)
        attributes: Additional attributes to add to the span

    Returns:
        Callable: Decorated function

    Example:
        >>> @instrument_span("custom.operation", attributes={"key": "value"})
        >>> async def my_function():
        ...     pass
    """
    def decorator(func: F) -> F:
        # Fast path: skip decoration if tracing is disabled
        if not is_tracing_enabled():
            return func

        span_name = name or f"{func.__module__}.{func.__name__}"
        span_attributes = attributes or {}

        if functools.iscoroutinefunction(func):
            @functools.wraps(func)
            async def async_wrapper(*args: Any, **kwargs: Any) -> Any:
                tracer = get_tracer()
                if tracer is None:
                    return await func(*args, **kwargs)

                with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
                    try:
                        result = await func(*args, **kwargs)
                        return result
                    except Exception as e:
                        add_exception(span, e)
                        raise

            return async_wrapper  # type: ignore
        else:
            @functools.wraps(func)
            def sync_wrapper(*args: Any, **kwargs: Any) -> Any:
                tracer = get_tracer()
                if tracer is None:
                    return func(*args, **kwargs)

                with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
                    try:
                        result = func(*args, **kwargs)
                        return result
                    except Exception as e:
                        add_exception(span, e)
                        raise

            return sync_wrapper  # type: ignore

    return decorator


def instrument_query(operation: str) -> Callable[[F], F]:
    """
    Decorator to instrument a query function with a query span.

    Args:
        operation: Database operation name (e.g., "find", "insert", "update", "delete")

    Returns:
        Callable: Decorated function

    Example:
        >>> @instrument_query("find")
        >>> async def find_users(filters):
        ...     pass
    """
    def decorator(func: F) -> F:
        # Fast path: skip decoration if tracing is disabled
        if not is_tracing_enabled():
            return func

        if functools.iscoroutinefunction(func):
            @functools.wraps(func)
            async def async_wrapper(*args: Any, **kwargs: Any) -> Any:
                tracer = get_tracer()
                if tracer is None:
                    return await func(*args, **kwargs)

                span_name = f"db.query.{operation}"
                span_attributes = {
                    SpanAttributes.DB_SYSTEM: DB_SYSTEM,
                    SpanAttributes.DB_OPERATION_NAME: operation,
                }

                with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
                    try:
                        result = await func(*args, **kwargs)
                        return result
                    except Exception as e:
                        add_exception(span, e)
                        raise

            return async_wrapper  # type: ignore
        else:
            @functools.wraps(func)
            def sync_wrapper(*args: Any, **kwargs: Any) -> Any:
                tracer = get_tracer()
                if tracer is None:
                    return func(*args, **kwargs)

                span_name = f"db.query.{operation}"
                span_attributes = {
                    SpanAttributes.DB_SYSTEM: DB_SYSTEM,
                    SpanAttributes.DB_OPERATION_NAME: operation,
                }

                with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
                    try:
                        result = func(*args, **kwargs)
                        return result
                    except Exception as e:
                        add_exception(span, e)
                        raise

            return sync_wrapper  # type: ignore

    return decorator


def instrument_session(operation: str) -> Callable[[F], F]:
    """
    Decorator to instrument a session function with a session span.

    Args:
        operation: Session operation name (e.g., "flush", "commit", "rollback")

    Returns:
        Callable: Decorated function

    Example:
        >>> @instrument_session("flush")
        >>> async def flush_session(session):
        ...     pass
    """
    def decorator(func: F) -> F:
        # Fast path: skip decoration if tracing is disabled
        if not is_tracing_enabled():
            return func

        if functools.iscoroutinefunction(func):
            @functools.wraps(func)
            async def async_wrapper(*args: Any, **kwargs: Any) -> Any:
                tracer = get_tracer()
                if tracer is None:
                    return await func(*args, **kwargs)

                span_name = f"db.session.{operation}"
                span_attributes = {
                    SpanAttributes.DB_SYSTEM: DB_SYSTEM,
                    SpanAttributes.DB_SESSION_OPERATION: operation,
                }

                with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
                    try:
                        result = await func(*args, **kwargs)
                        return result
                    except Exception as e:
                        add_exception(span, e)
                        raise

            return async_wrapper  # type: ignore
        else:
            @functools.wraps(func)
            def sync_wrapper(*args: Any, **kwargs: Any) -> Any:
                tracer = get_tracer()
                if tracer is None:
                    return func(*args, **kwargs)

                span_name = f"db.session.{operation}"
                span_attributes = {
                    SpanAttributes.DB_SYSTEM: DB_SYSTEM,
                    SpanAttributes.DB_SESSION_OPERATION: operation,
                }

                with tracer.start_as_current_span(span_name, attributes=span_attributes) as span:
                    try:
                        result = func(*args, **kwargs)
                        return result
                    except Exception as e:
                        add_exception(span, e)
                        raise

            return sync_wrapper  # type: ignore

    return decorator


# ============================================================================
# Metrics Configuration
# ============================================================================

class ConnectionPoolMetrics:
    """Connection pool metrics collector."""

    def __init__(self):
        """Initialize connection pool metrics."""
        self._meter = get_meter()

        if self._meter is None:
            self._in_use_gauge = None
            self._idle_gauge = None
            self._max_gauge = None
        else:
            # Create gauge instruments
            self._in_use_gauge = self._meter.create_gauge(
                name=MetricNames.CONNECTION_POOL_IN_USE,
                description="Number of connections currently in use",
                unit="connections"
            )

            self._idle_gauge = self._meter.create_gauge(
                name=MetricNames.CONNECTION_POOL_IDLE,
                description="Number of idle connections in the pool",
                unit="connections"
            )

            self._max_gauge = self._meter.create_gauge(
                name=MetricNames.CONNECTION_POOL_MAX,
                description="Maximum number of connections in the pool",
                unit="connections"
            )

    def record_pool_stats(self, in_use: int, idle: int, max_size: int) -> None:
        """
        Record connection pool statistics.

        Args:
            in_use: Number of connections in use
            idle: Number of idle connections
            max_size: Maximum pool size

        Example:
            >>> metrics = ConnectionPoolMetrics()
            >>> metrics.record_pool_stats(in_use=5, idle=3, max_size=10)
        """
        if self._in_use_gauge is not None:
            self._in_use_gauge.set(in_use)

        if self._idle_gauge is not None:
            self._idle_gauge.set(idle)

        if self._max_gauge is not None:
            self._max_gauge.set(max_size)


# Global connection pool metrics instance
_connection_pool_metrics: Optional[ConnectionPoolMetrics] = None


def get_connection_pool_metrics() -> ConnectionPoolMetrics:
    """
    Get the global connection pool metrics instance.

    Returns:
        ConnectionPoolMetrics: The metrics instance.
    """
    global _connection_pool_metrics

    if _connection_pool_metrics is None:
        _connection_pool_metrics = ConnectionPoolMetrics()

    return _connection_pool_metrics


# ============================================================================
# Exports
# ============================================================================

__all__ = [
    # Configuration
    "is_tracing_enabled",
    "get_tracer",
    "get_meter",

    # Semantic conventions
    "SpanAttributes",
    "MetricNames",

    # Span helpers
    "create_query_span",
    "create_session_span",
    "create_relationship_span",
    "add_exception",
    "set_span_result",

    # Decorators
    "instrument_span",
    "instrument_query",
    "instrument_session",

    # Metrics
    "ConnectionPoolMetrics",
    "get_connection_pool_metrics",
]
