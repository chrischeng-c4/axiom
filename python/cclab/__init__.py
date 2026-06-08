"""
cclab: High-performance Rust-powered Python platform.

Features:
    - MongoDB ORM (Beanie-compatible) - cclab.mongo
    - PostgreSQL ORM - cclab.pg
    - HTTP Client - cclab.http
    - Task Queue - cclab.queue
    - KV Store - cclab.kv
    - Agent Framework (LLM-powered agents with tool calling) - cclab.agent
    - All with Rust backend for maximum performance

Quick Start:
    >>> import cclab
    >>> from cclab.mongo import Document, init
    >>>
    >>> await init("mongodb://localhost:27017/mydb")
    >>>
    >>> class User(Document):
    ...     email: str
    ...     name: str
    ...
    >>> user = await User.find_one(User.email == "alice@example.com")
"""

__version__ = "0.3.43"

# Import ObjectId from Rust extension
# Gate the import so tests can be collected even if the extension isn't built yet
try:
    from ._nebula import ObjectId  # type: ignore
except ImportError:
    from uuid import UUID
    ObjectId = UUID  # Fallback for tests

# Import sub-packages (Python wrappers around Rust modules)
# Gate imports so tests can be collected even if modules aren't fully available
try:
    from . import mongo  # MongoDB
except ImportError:
    mongo = None

try:
    from . import http  # HTTP
except ImportError:
    http = None

try:
    from . import pg  # PostgreSQL
except ImportError:
    pg = None

try:
    from . import qc  # QC/Testing
except (ImportError, AttributeError):
    qc = None

# Import Agent module if available (feature-gated)
try:
    from . import agent  # Agent framework
except (ImportError, AttributeError):
    agent = None  # Agent feature not enabled

# Import KV module if available (feature-gated)
try:
    from . import kv  # KV store
except ImportError:
    kv = None  # KV feature not enabled

# Import Crypto module if available (feature-gated)
try:
    from . import crypto  # Crypto & auth
except (ImportError, AttributeError):
    crypto = None  # Crypto feature not enabled

# Import Runtime module if available (feature-gated)
try:
    from . import _runtime  # type: ignore
except (ImportError, AttributeError):
    _runtime = None  # Runtime feature not enabled

# Re-export commonly used classes from mongo for convenience/backward compatibility
_mongo_imports = {}
if mongo is not None:
    try:
        from .mongo import (
            Document, Settings, EmbeddedDocument,
            Field, FieldProxy, QueryExpr, merge_filters, text_search, TextSearch, escape_regex,
            QueryBuilder, AggregationBuilder,
            init, is_connected, close, reset,
            # Actions
            before_event, after_event, Insert, Replace, Save, Delete, ValidateOnSave, EventType,
            # Bulk
            BulkOperation, UpdateOne, UpdateMany, InsertOne, DeleteOne, DeleteMany, ReplaceOne, BulkWriteResult,
            # Types
            PydanticObjectId, Indexed, IndexModelField, get_index_fields,
            # Links
            Link, BackLink, WriteRules, DeleteRules, get_link_fields,
            # Transactions
            Session, Transaction, start_session, TransactionNotSupportedError,
            # Time-series
            TimeSeriesConfig, Granularity,
            # Migrations
            Migration, MigrationHistory, IterativeMigration, FreeFallMigration, iterative_migration, free_fall_migration, run_migrations, get_pending_migrations, get_applied_migrations, get_migration_status,
            # Constraints
            Constraint, MinLen, MaxLen, Min, Max, Email, Url,
        )
        _mongo_imports = {
            'Document': Document, 'Settings': Settings, 'EmbeddedDocument': EmbeddedDocument,
            'Field': Field, 'FieldProxy': FieldProxy, 'QueryExpr': QueryExpr, 'merge_filters': merge_filters,
            'text_search': text_search, 'TextSearch': TextSearch, 'escape_regex': escape_regex,
            'QueryBuilder': QueryBuilder, 'AggregationBuilder': AggregationBuilder,
            'init': init, 'is_connected': is_connected, 'close': close, 'reset': reset,
            'before_event': before_event, 'after_event': after_event, 'Insert': Insert, 'Replace': Replace,
            'Save': Save, 'Delete': Delete, 'ValidateOnSave': ValidateOnSave, 'EventType': EventType,
            'BulkOperation': BulkOperation, 'UpdateOne': UpdateOne, 'UpdateMany': UpdateMany,
            'InsertOne': InsertOne, 'DeleteOne': DeleteOne, 'DeleteMany': DeleteMany, 'ReplaceOne': ReplaceOne,
            'BulkWriteResult': BulkWriteResult, 'PydanticObjectId': PydanticObjectId, 'Indexed': Indexed,
            'IndexModelField': IndexModelField, 'get_index_fields': get_index_fields, 'Link': Link,
            'BackLink': BackLink, 'WriteRules': WriteRules, 'DeleteRules': DeleteRules, 'get_link_fields': get_link_fields,
            'Session': Session, 'Transaction': Transaction, 'start_session': start_session,
            'TransactionNotSupportedError': TransactionNotSupportedError, 'TimeSeriesConfig': TimeSeriesConfig,
            'Granularity': Granularity, 'Migration': Migration, 'MigrationHistory': MigrationHistory,
            'IterativeMigration': IterativeMigration, 'FreeFallMigration': FreeFallMigration,
            'iterative_migration': iterative_migration, 'free_fall_migration': free_fall_migration,
            'run_migrations': run_migrations, 'get_pending_migrations': get_pending_migrations,
            'get_applied_migrations': get_applied_migrations, 'get_migration_status': get_migration_status,
            'Constraint': Constraint, 'MinLen': MinLen, 'MaxLen': MaxLen, 'Min': Min, 'Max': Max,
            'Email': Email, 'Url': Url,
        }
    except ImportError:
        pass

# Make imports available at module level
for name, obj in _mongo_imports.items():
    globals()[name] = obj

__all__ = [
    # Version
    "__version__",
    # Modules
    "mongo",    # MongoDB
    "pg",       # PostgreSQL
    "http",     # HTTP
    "qc",       # QC/Testing
    "kv",       # KV
    "agent",    # Agent
    # Core Types
    "ObjectId",
    # Connection
    "init",
    "is_connected",
    "close",
    "reset",
    # Core
    "Document",
    "Settings",
    "EmbeddedDocument",
    # Fields
    "Field",
    "FieldProxy",
    "QueryExpr",
    "merge_filters",
    "text_search",
    "TextSearch",
    "escape_regex",
    # Query
    "QueryBuilder",
    "AggregationBuilder",
    # Actions
    "before_event",
    "after_event",
    "Insert",
    "Replace",
    "Save",
    "Delete",
    "ValidateOnSave",
    "EventType",
    # Bulk Operations
    "BulkOperation",
    "UpdateOne",
    "UpdateMany",
    "InsertOne",
    "DeleteOne",
    "DeleteMany",
    "ReplaceOne",
    "BulkWriteResult",
    # Type Support
    "PydanticObjectId",
    "Indexed",
    "IndexModelField",
    "get_index_fields",
    # Document Relations
    "Link",
    "BackLink",
    "WriteRules",
    "DeleteRules",
    "get_link_fields",
    # Transactions
    "Session",
    "Transaction",
    "start_session",
    "TransactionNotSupportedError",
    # Time-series Collections
    "TimeSeriesConfig",
    "Granularity",
    # Migrations
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
    # Constraints
    "Constraint",
    "MinLen",
    "MaxLen",
    "Min",
    "Max",
    "Email",
    "Url",
]

# Re-export KvClient if available
try:
    from .kv import KvClient  # type: ignore
    __all__.append("KvClient")
except (ImportError, AttributeError):
    pass  # KV feature not enabled
