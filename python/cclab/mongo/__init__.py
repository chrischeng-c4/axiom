"""
MongoDB ORM module for data-bridge.
"""

from .connection import (
    init,
    is_connected,
    close,
    reset,
)

# Core classes
from .document import Document, Settings
from .embedded import EmbeddedDocument
from .fields import Field, FieldProxy, QueryExpr, merge_filters, text_search, TextSearch, escape_regex
from .query import QueryBuilder, AggregationBuilder

# Lifecycle actions/hooks
from .actions import (
    before_event,
    after_event,
    Insert,
    Replace,
    Save,
    Delete,
    ValidateOnSave,
    EventType,
)

# Bulk operations
from .bulk import (
    BulkOperation,
    UpdateOne,
    UpdateMany,
    InsertOne,
    DeleteOne,
    DeleteMany,
    ReplaceOne,
    BulkWriteResult,
)

# Type support
from .types import (
    PydanticObjectId,
    Indexed,
    IndexModelField,
    get_index_fields,
)

# Document relations/links
from .links import (
    Link,
    BackLink,
    WriteRules,
    DeleteRules,
    get_link_fields,
)

# Transactions
from .transactions import (
    Session,
    Transaction,
    start_session,
    TransactionNotSupportedError,
)

# Time-series collections
from .timeseries import (
    TimeSeriesConfig,
    Granularity,
)

# Migrations
from .migrations import (
    Migration,
    MigrationHistory,
    IterativeMigration,
    FreeFallMigration,
    iterative_migration,
    free_fall_migration,
    run_migrations,
    get_pending_migrations,
    get_applied_migrations,
    get_migration_status,
)

# Constraint validators
from .constraints import (
    Constraint,
    MinLen,
    MaxLen,
    Min,
    Max,
    Email,
    Url,
)

__all__ = [
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
    # Bulk
    "BulkOperation",
    "UpdateOne",
    "UpdateMany",
    "InsertOne",
    "DeleteOne",
    "DeleteMany",
    "ReplaceOne",
    "BulkWriteResult",
    # Types
    "PydanticObjectId",
    "Indexed",
    "IndexModelField",
    "get_index_fields",
    # Links
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
    # Time-series
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
