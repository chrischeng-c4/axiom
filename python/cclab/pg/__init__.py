"""PostgreSQL ORM for data-bridge."""

from .table import Table
from .columns import Column, ColumnProxy, ForeignKeyProxy, BackReference, BackReferenceQuery, ManyToMany, ManyToManyQuery, create_m2m_join_table
from .query import QueryBuilder
from .relationships import relationship, LoadingStrategy, RelationshipDescriptor
from .options import QueryOption, selectinload, joinedload, noload, raiseload
from .fulltext import FullTextSearch, fts
from .postgis import Point, GeoQuery
from .arrays import ArrayOps
from .query_ext import (
    filter_by, and_, or_, not_, any_, has, aliased,
    QueryFragment, BooleanClause, AliasedClass,
    active_filter, date_range_filter, in_list_filter, null_check_filter
)
from .connection import (
    init, close, is_connected, execute, query_aggregate, query_with_cte,
    insert_one, insert_many,
    upsert_one, upsert_many,
    list_tables, table_exists, get_columns, get_indexes, get_foreign_keys, inspect_table,
    get_backreferences,
    find_by_foreign_key,
    fetch_one_with_relations, fetch_one_eager, fetch_many_with_relations,
    delete_with_cascade, delete_checked,
    migration_init, migration_status, migration_apply,
    migration_rollback, migration_create
)
from .transactions import pg_transaction, Transaction
from .migrations import Migration, run_migrations, get_migration_status, autogenerate_migration
from .session import Session, IdentityMap, DirtyTracker, UnitOfWork, get_session
from .events import (
    EventType, EventDispatcher, listens_for,
    before_insert, after_insert,
    before_update, after_update,
    before_delete, after_delete,
    before_flush, after_commit,
    AttributeEvents
)
from .telemetry import (
    is_tracing_enabled, get_tracer, get_meter,
    SpanAttributes, MetricNames,
    create_query_span, create_session_span, create_relationship_span,
    add_exception, set_span_result,
    instrument_span, instrument_query, instrument_session,
    ConnectionPoolMetrics, get_connection_pool_metrics
)
from .loading import (
    LoadingStrategy, LoadingConfig,
    lazy, joined, subquery, selectinload, noload, raiseload, defer, undefer,
    LazyLoadingProxy, DeferredColumn, RelationshipLoader,
    LazyLoadError, SQLGenerationError
)
from .inheritance import (
    InheritanceType, InheritanceConfig, inheritance,
    SingleTableInheritance, JoinedTableInheritance, ConcreteTableInheritance,
    PolymorphicQueryMixin,
    get_inheritance_type, get_discriminator_column, get_discriminator_value,
    register_polymorphic_class, get_polymorphic_map
)
from .computed import (
    hybrid_property, hybrid_method, column_property, Computed,
    default_factory,
    HybridPropertyDescriptor, HybridMethodDescriptor,
    ColumnPropertyDescriptor, ComputedColumn
)
from .validation import (
    validates, validates_many,
    TypeDecorator,
    coerce_int, coerce_float, coerce_str, coerce_bool, coerce_datetime, coerce_date, coerce_decimal,
    ValidationError,
    ValidatorRegistry,
    AutoCoerceMixin,
    validate_not_empty, validate_email, validate_url,
    validate_min_length, validate_max_length, validate_regex,
    validate_range, validate_min_value, validate_max_value,
    validate_in_list, validate_positive, validate_non_negative
)
from .async_utils import (
    AsyncSession, AsyncSessionFactory,
    run_sync, async_wrap,
    AsyncScoped, get_async_session,
    async_load, async_refresh, async_expire,
    async_stream, AsyncResultIterator,
    greenlet_spawn, AsyncGreenlet, GREENLET_AVAILABLE,
    AsyncEngine
)

# ---------------------------------------------------------------------------
# SQLAlchemy compat re-exports
#
# These re-export raw SQLAlchemy symbols so that application code can import
# them from cclab.pg rather than importing sqlalchemy directly.  This keeps
# the "no direct sqlalchemy imports" rule satisfied while allowing incremental
# migration away from SQLAlchemy ORM patterns.
# ---------------------------------------------------------------------------
try:
    from sqlalchemy import select as sa_select, func as sa_func  # noqa: F401
    from sqlalchemy import update as sa_update, delete as sa_delete  # noqa: F401
    from sqlalchemy.dialects.postgresql import insert as sa_pg_insert  # noqa: F401
    from sqlalchemy.orm import selectinload as sa_selectinload  # noqa: F401
    from sqlalchemy.ext.asyncio import AsyncSession as SAAsyncSession  # noqa: F401
    from sqlalchemy.ext.asyncio import create_async_engine as sa_create_async_engine  # noqa: F401
    from sqlalchemy.orm import sessionmaker as sa_sessionmaker  # noqa: F401
    from sqlalchemy import text as sa_text  # noqa: F401

    # ORM types used by Conductor models
    from sqlalchemy.orm import DeclarativeBase  # noqa: F401
    from sqlalchemy.orm import Mapped, mapped_column, relationship  # noqa: F401
    from sqlalchemy import String, Text, Integer, JSON, DateTime  # noqa: F401
    from sqlalchemy import ForeignKey, UniqueConstraint, Index  # noqa: F401
    from sqlalchemy.dialects.postgresql import UUID as Uuid  # noqa: F401

    # Connection pool compat (wraps SQLAlchemy async engine)
    from sqlalchemy.ext.asyncio import create_async_engine as _sa_engine_factory

    class Connection:
        """Compat wrapper — wraps SQLAlchemy AsyncSession."""
        def __init__(self, session):
            self._session = session
        async def execute(self, query, *args, **kwargs):
            if isinstance(query, str):
                from sqlalchemy import text
                return await self._session.execute(text(query), *args, **kwargs)
            return await self._session.execute(query, *args, **kwargs)
        async def fetch(self, query, *args):
            return (await self.execute(query, *args)).fetchall()
        def transaction(self):
            return self._session.begin()

    class Pool:
        """Compat wrapper — wraps SQLAlchemy async engine + session factory."""
        def __init__(self, engine, factory):
            self._engine = engine
            self._factory = factory
        class _Ctx:
            def __init__(self, f): self._f = f; self._s = None
            async def __aenter__(self):
                self._s = self._f()
                return Connection(self._s)
            async def __aexit__(self, *a):
                if self._s: await self._s.close()
        def acquire(self): return self._Ctx(self._factory)
        def session_factory(self): return self._factory
        async def run_migrations(self, pkg): pass
        async def close(self): await self._engine.dispose()

    async def create_pool(url, *, min_size=5, max_size=15, echo=False):
        """Create async connection pool (compat — wraps SQLAlchemy)."""
        engine = _sa_engine_factory(url, echo=echo, pool_size=min_size, max_overflow=max_size - min_size)
        factory = sa_sessionmaker(engine, class_=SAAsyncSession, expire_on_commit=False)
        return Pool(engine, factory)

    class Migration:
        """SQL migration with up/down callables."""
        def __init__(self, *, id: str, depends_on=None):
            self.id = id
            self.depends_on = depends_on
            self._up = None
            self._down = None
        def up(self, fn):
            self._up = fn; return fn
        def down(self, fn):
            self._down = fn; return fn
except ImportError:
    # SQLAlchemy not installed -- stubs so the module still loads
    sa_select = None  # type: ignore[assignment]
    sa_func = None  # type: ignore[assignment]
    sa_update = None  # type: ignore[assignment]
    sa_delete = None  # type: ignore[assignment]
    sa_pg_insert = None  # type: ignore[assignment]
    sa_selectinload = None  # type: ignore[assignment]
    SAAsyncSession = None  # type: ignore[assignment]
    sa_create_async_engine = None  # type: ignore[assignment]
    sa_sessionmaker = None  # type: ignore[assignment]
    sa_text = None  # type: ignore[assignment]

__all__ = [
    # Base class
    "Table",
    # Fields
    "Column",
    "ColumnProxy",
    "ForeignKeyProxy",
    "BackReference",
    "BackReferenceQuery",
    "ManyToMany",
    "ManyToManyQuery",
    "create_m2m_join_table",
    # Relationships (Lazy Loading)
    "relationship",
    "LoadingStrategy",
    "RelationshipDescriptor",
    # Query Options (Eager Loading)
    "QueryOption",
    "selectinload",
    "joinedload",
    "noload",
    "raiseload",
    # Query
    "QueryBuilder",
    # Query Extensions
    "filter_by",
    "and_",
    "or_",
    "not_",
    "any_",
    "has",
    "aliased",
    "QueryFragment",
    "BooleanClause",
    "AliasedClass",
    "active_filter",
    "date_range_filter",
    "in_list_filter",
    "null_check_filter",
    # Connection
    "init",
    "close",
    "is_connected",
    "execute",
    "query_aggregate",
    "query_with_cte",
    # CRUD Operations
    "insert_one",
    "insert_many",
    "upsert_one",
    "upsert_many",
    # Schema Introspection
    "list_tables",
    "table_exists",
    "get_columns",
    "get_indexes",
    "get_foreign_keys",
    "get_backreferences",
    "inspect_table",
    # Relationships
    "find_by_foreign_key",
    "fetch_one_with_relations",
    "fetch_one_eager",
    "fetch_many_with_relations",
    # Cascade Delete
    "delete_with_cascade",
    "delete_checked",
    # Transactions
    "pg_transaction",
    "Transaction",
    # Migrations (Rust-based)
    "migration_init",
    "migration_status",
    "migration_apply",
    "migration_rollback",
    "migration_create",
    # Migrations (Python-based - legacy)
    "Migration",
    "run_migrations",
    "get_migration_status",
    "autogenerate_migration",
    # Session Management
    "Session",
    "IdentityMap",
    "DirtyTracker",
    "UnitOfWork",
    "get_session",
    # Event System
    "EventType",
    "EventDispatcher",
    "listens_for",
    "before_insert",
    "after_insert",
    "before_update",
    "after_update",
    "before_delete",
    "after_delete",
    "before_flush",
    "after_commit",
    "AttributeEvents",
    # PostgreSQL Extensions
    "FullTextSearch",
    "fts",
    "Point",
    "GeoQuery",
    "ArrayOps",
    # OpenTelemetry Integration
    "is_tracing_enabled",
    "get_tracer",
    "get_meter",
    "SpanAttributes",
    "MetricNames",
    "create_query_span",
    "create_session_span",
    "create_relationship_span",
    "add_exception",
    "set_span_result",
    "instrument_span",
    "instrument_query",
    "instrument_session",
    "ConnectionPoolMetrics",
    "get_connection_pool_metrics",
    # Loading Strategies
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
    # Inheritance Patterns
    "InheritanceType",
    "InheritanceConfig",
    "inheritance",
    "SingleTableInheritance",
    "JoinedTableInheritance",
    "ConcreteTableInheritance",
    "PolymorphicQueryMixin",
    "get_inheritance_type",
    "get_discriminator_column",
    "get_discriminator_value",
    "register_polymorphic_class",
    "get_polymorphic_map",
    # Computed Attributes
    "hybrid_property",
    "hybrid_method",
    "column_property",
    "Computed",
    "default_factory",
    "HybridPropertyDescriptor",
    "HybridMethodDescriptor",
    "ColumnPropertyDescriptor",
    "ComputedColumn",
    # Validation
    "validates",
    "validates_many",
    "TypeDecorator",
    "coerce_int",
    "coerce_float",
    "coerce_str",
    "coerce_bool",
    "coerce_datetime",
    "coerce_date",
    "coerce_decimal",
    "ValidationError",
    "ValidatorRegistry",
    "AutoCoerceMixin",
    "validate_not_empty",
    "validate_email",
    "validate_url",
    "validate_min_length",
    "validate_max_length",
    "validate_regex",
    "validate_range",
    "validate_min_value",
    "validate_max_value",
    "validate_in_list",
    "validate_positive",
    "validate_non_negative",
    # Async Utilities
    "AsyncSession",
    "AsyncSessionFactory",
    "run_sync",
    "async_wrap",
    "AsyncScoped",
    "get_async_session",
    "async_load",
    "async_refresh",
    "async_expire",
    "async_stream",
    "AsyncResultIterator",
    "greenlet_spawn",
    "AsyncGreenlet",
    "GREENLET_AVAILABLE",
    "AsyncEngine",
    # SQLAlchemy compat re-exports
    "sa_select",
    "sa_func",
    "sa_update",
    "sa_delete",
    "sa_pg_insert",
    "sa_selectinload",
    "SAAsyncSession",
]
