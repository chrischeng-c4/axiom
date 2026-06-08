"""
Session and Unit of Work implementation for data-bridge PostgreSQL ORM.

Provides SQLAlchemy-style session management with:
- Identity Map: Single instance per primary key
- Dirty Tracking: Track field changes
- Unit of Work: Accumulate INSERT/UPDATE/DELETE
- Auto-flush: Flush pending changes before queries
"""
from __future__ import annotations

import weakref
from enum import Enum, auto
from typing import (
    Any, Dict, Generic, List, Optional, Set, Tuple, Type, TypeVar,
    Union, TYPE_CHECKING, Callable, Awaitable
)
from dataclasses import dataclass, field

from .telemetry import (
    create_session_span,
    add_exception,
    is_tracing_enabled,
    set_span_result,
)

if TYPE_CHECKING:
    from .table import Table
    from opentelemetry.trace import Span

T = TypeVar('T', bound='Table')


class ObjectState(Enum):
    """State of an object in the session."""
    TRANSIENT = auto()    # Not attached to session, no identity
    PENDING = auto()      # Attached, will be INSERTed on flush
    PERSISTENT = auto()   # Attached, exists in database
    DIRTY = auto()        # Attached, modified, will be UPDATEd
    DELETED = auto()      # Attached, will be DELETEd on flush
    DETACHED = auto()     # Was attached, now removed from session


@dataclass
class IdentityKey:
    """Key for identity map lookup."""
    table_name: str
    primary_key: Any

    def __hash__(self) -> int:
        return hash((self.table_name, self.primary_key))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, IdentityKey):
            return False
        return self.table_name == other.table_name and self.primary_key == other.primary_key


class IdentityMap:
    """
    Maps (table_name, primary_key) to object instances.

    Ensures only one instance exists per database row within a session.
    Uses weak references to allow garbage collection of unused objects.
    """

    def __init__(self, use_weak_refs: bool = True):
        self._use_weak_refs = use_weak_refs
        self._map: Dict[IdentityKey, Any] = {}
        self._weak_map: Dict[IdentityKey, weakref.ref] = {}

    def get(self, table_name: str, pk: Any) -> Optional[Any]:
        """Get object by identity key, or None if not found."""
        key = IdentityKey(table_name, pk)

        if self._use_weak_refs:
            ref = self._weak_map.get(key)
            if ref is not None:
                obj = ref()
                if obj is not None:
                    return obj
                # Reference is dead, clean up
                del self._weak_map[key]
            return None
        else:
            return self._map.get(key)

    def add(self, table_name: str, pk: Any, obj: Any) -> None:
        """Add or update object in identity map."""
        key = IdentityKey(table_name, pk)

        if self._use_weak_refs:
            self._weak_map[key] = weakref.ref(obj)
        else:
            self._map[key] = obj

    def remove(self, table_name: str, pk: Any) -> None:
        """Remove object from identity map."""
        key = IdentityKey(table_name, pk)

        if self._use_weak_refs:
            self._weak_map.pop(key, None)
        else:
            self._map.pop(key, None)

    def contains(self, table_name: str, pk: Any) -> bool:
        """Check if object exists in identity map."""
        return self.get(table_name, pk) is not None

    def clear(self) -> None:
        """Clear all entries from identity map."""
        self._map.clear()
        self._weak_map.clear()

    def __len__(self) -> int:
        if self._use_weak_refs:
            # Count only live references
            return sum(1 for ref in self._weak_map.values() if ref() is not None)
        return len(self._map)


@dataclass
class ObjectSnapshot:
    """Snapshot of object state for dirty tracking."""
    data: Dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_object(cls, obj: Any) -> 'ObjectSnapshot':
        """Create snapshot from object's current state."""
        if hasattr(obj, '_get_column_values'):
            data = obj._get_column_values()
        elif hasattr(obj, '__dict__'):
            data = {k: v for k, v in obj.__dict__.items() if not k.startswith('_')}
        else:
            data = {}
        return cls(data=data.copy())


class DirtyTracker:
    """
    Tracks which objects have been modified.

    Compares current object state against saved snapshots.
    """

    def __init__(self):
        self._snapshots: Dict[int, ObjectSnapshot] = {}  # id(obj) -> snapshot

    def take_snapshot(self, obj: Any) -> None:
        """Save current state of object for later comparison."""
        self._snapshots[id(obj)] = ObjectSnapshot.from_object(obj)

    def get_dirty_fields(self, obj: Any) -> Dict[str, Tuple[Any, Any]]:
        """
        Get fields that have changed since snapshot.

        Returns dict of {field_name: (old_value, new_value)}
        """
        snapshot = self._snapshots.get(id(obj))
        if snapshot is None:
            return {}

        if hasattr(obj, '_get_column_values'):
            current = obj._get_column_values()
        elif hasattr(obj, '__dict__'):
            current = {k: v for k, v in obj.__dict__.items() if not k.startswith('_')}
        else:
            current = {}

        dirty = {}
        for key, old_val in snapshot.data.items():
            new_val = current.get(key)
            if old_val != new_val:
                dirty[key] = (old_val, new_val)

        # Check for new fields
        for key, new_val in current.items():
            if key not in snapshot.data:
                dirty[key] = (None, new_val)

        return dirty

    def is_dirty(self, obj: Any) -> bool:
        """Check if object has any dirty fields."""
        return len(self.get_dirty_fields(obj)) > 0

    def clear_snapshot(self, obj: Any) -> None:
        """Remove snapshot for object."""
        self._snapshots.pop(id(obj), None)

    def refresh_snapshot(self, obj: Any) -> None:
        """Update snapshot with current object state."""
        self.take_snapshot(obj)

    def clear(self) -> None:
        """Clear all snapshots."""
        self._snapshots.clear()


class UnitOfWork:
    """
    Tracks pending database operations.

    Accumulates new, dirty, and deleted objects for batch execution.
    """

    def __init__(self):
        self._new: List[Any] = []           # Objects to INSERT
        self._dirty: Set[int] = set()        # Object IDs that are dirty
        self._deleted: List[Any] = []        # Objects to DELETE
        self._dirty_tracker = DirtyTracker()

    def register_new(self, obj: Any) -> None:
        """Register a new object for INSERT."""
        if obj not in self._new:
            self._new.append(obj)

    def register_dirty(self, obj: Any) -> None:
        """Mark object as dirty (needs UPDATE)."""
        self._dirty.add(id(obj))

    def register_deleted(self, obj: Any) -> None:
        """Register object for DELETE."""
        if obj not in self._deleted:
            self._deleted.append(obj)
            # Remove from new if it was pending insert
            if obj in self._new:
                self._new.remove(obj)

    def register_clean(self, obj: Any) -> None:
        """Register a clean (loaded) object with snapshot."""
        self._dirty_tracker.take_snapshot(obj)

    def is_dirty(self, obj: Any) -> bool:
        """Check if object is dirty."""
        return id(obj) in self._dirty or self._dirty_tracker.is_dirty(obj)

    def get_dirty_fields(self, obj: Any) -> Dict[str, Tuple[Any, Any]]:
        """Get changed fields for an object."""
        return self._dirty_tracker.get_dirty_fields(obj)

    @property
    def has_pending(self) -> bool:
        """Check if there are any pending operations."""
        return bool(self._new or self._dirty or self._deleted)

    @property
    def new_objects(self) -> List[Any]:
        """Get list of new objects pending INSERT."""
        return list(self._new)

    @property
    def dirty_objects(self) -> List[Any]:
        """Get list of dirty objects pending UPDATE."""
        # Return objects that have actual changes
        return [obj for obj in self._new if self._dirty_tracker.is_dirty(obj)]

    @property
    def deleted_objects(self) -> List[Any]:
        """Get list of deleted objects pending DELETE."""
        return list(self._deleted)

    def clear(self) -> None:
        """Clear all pending operations."""
        self._new.clear()
        self._dirty.clear()
        self._deleted.clear()
        self._dirty_tracker.clear()


class Session:
    """
    Database session with Unit of Work pattern.

    Manages object lifecycle, tracks changes, and batches database operations.

    Example:
        async with Session() as session:
            user = await session.get(User, 1)
            user.name = "New Name"
            await session.commit()  # Flushes UPDATE
    """

    _current: Optional['Session'] = None  # Thread-local would be better

    def __init__(
        self,
        autoflush: bool = True,
        expire_on_commit: bool = True,
    ):
        """
        Initialize a new session.

        Args:
            autoflush: Automatically flush before queries
            expire_on_commit: Expire all objects after commit
        """
        self.autoflush = autoflush
        self.expire_on_commit = expire_on_commit
        self._identity_map = IdentityMap()
        self._unit_of_work = UnitOfWork()
        self._transaction_started = False
        self._closed = False
        self._loaded_relationships: Dict[Tuple[Any, str], Any] = {}
        self._session_span_ctx = None  # Track session span context manager
        self._session_span: Optional['Span'] = None  # Track session lifecycle span

    @classmethod
    def get_current(cls) -> Optional['Session']:
        """Get the current active session."""
        return cls._current

    async def __aenter__(self) -> 'Session':
        """Enter session context."""
        if is_tracing_enabled():
            # Create session lifecycle span
            self._session_span_ctx = create_session_span(
                operation="open",
                autoflush=self.autoflush,
                expire_on_commit=self.expire_on_commit,
            )
            # Enter the context manager and store the span
            self._session_span = self._session_span_ctx.__enter__()

        Session._current = self
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit session context."""
        try:
            if exc_type is None:
                # No exception, commit
                await self.commit()
            else:
                # Exception occurred, rollback
                await self.rollback()
        finally:
            # End session span if it exists
            if self._session_span_ctx is not None:
                if exc_type and self._session_span is not None:
                    add_exception(self._session_span, exc_val)
                elif self._session_span is not None:
                    # Set success status
                    if hasattr(self._session_span, 'set_attribute'):
                        self._session_span.set_attribute("db.session.status", "closed")
                # Exit the span context
                self._session_span_ctx.__exit__(exc_type, exc_val, exc_tb)
                self._session_span_ctx = None
                self._session_span = None

            await self.close()
            if Session._current is self:
                Session._current = None

    def add(self, obj: T) -> T:
        """
        Add an object to the session.

        New objects will be INSERTed on flush.
        """
        if self._closed:
            raise RuntimeError("Session is closed")

        pk = self._get_pk(obj)
        table_name = self._get_table_name(obj)

        if pk is not None:
            # Object has PK, might already exist
            existing = self._identity_map.get(table_name, pk)
            if existing is not None:
                return existing  # Return existing instance
            self._identity_map.add(table_name, pk, obj)

        self._unit_of_work.register_new(obj)
        return obj

    def add_all(self, objects: List[T]) -> List[T]:
        """Add multiple objects to the session."""
        return [self.add(obj) for obj in objects]

    async def get(
        self,
        model: Type[T],
        pk: Any,
        *,
        with_for_update: bool = False,
    ) -> Optional[T]:
        """
        Get object by primary key.

        First checks identity map, then queries database.
        """
        if self._closed:
            raise RuntimeError("Session is closed")

        table_name = model._get_table_name() if hasattr(model, '_get_table_name') else model.__name__.lower()

        # Check identity map first
        cached = self._identity_map.get(table_name, pk)
        if cached is not None:
            return cached

        # Auto-flush if enabled
        if self.autoflush and self._unit_of_work.has_pending:
            await self.flush()

        # Query database
        from . import find_one
        pk_column = model._get_pk_column() if hasattr(model, '_get_pk_column') else 'id'

        result = await find_one(table_name, where_conditions=[(pk_column, "eq", pk)])
        if result is None:
            return None

        # Create instance and add to session
        obj = model(**result) if isinstance(result, dict) else result
        self._identity_map.add(table_name, pk, obj)
        self._unit_of_work.register_clean(obj)
        return obj

    def delete(self, obj: T) -> None:
        """Mark object for deletion."""
        if self._closed:
            raise RuntimeError("Session is closed")

        self._unit_of_work.register_deleted(obj)

        # Remove from identity map
        pk = self._get_pk(obj)
        if pk is not None:
            table_name = self._get_table_name(obj)
            self._identity_map.remove(table_name, pk)

    async def flush(self) -> None:
        """
        Flush pending changes to database.

        Executes all pending INSERT, UPDATE, DELETE operations.
        """
        if self._closed:
            raise RuntimeError("Session is closed")

        # Fast path: no tracing
        if not is_tracing_enabled():
            await self._execute_flush()
            return

        # With tracing
        new_count = len(self._unit_of_work._new)
        dirty_count = len(self._unit_of_work._dirty)
        deleted_count = len(self._unit_of_work._deleted)

        with create_session_span(
            operation="flush",
            new_count=new_count,
            dirty_count=dirty_count,
            deleted_count=deleted_count,
        ) as span:
            try:
                await self._execute_flush()
                # Record result
                if span is not None and hasattr(span, 'set_attribute'):
                    span.set_attribute("db.session.status", "flushed")
            except Exception as e:
                add_exception(span, e)
                raise

    async def _execute_flush(self) -> None:
        """Internal method to execute flush operations."""
        from . import insert_one, update_many, delete_many

        # Process INSERTs
        for obj in self._unit_of_work.new_objects:
            table_name = self._get_table_name(obj)
            data = self._get_data(obj)
            result = await insert_one(table_name, data)

            # Update object with returned values (including generated PK)
            if isinstance(result, dict):
                for key, value in result.items():
                    if hasattr(obj, key):
                        setattr(obj, key, value)

            # Add to identity map with new PK
            pk = self._get_pk(obj)
            if pk is not None:
                self._identity_map.add(table_name, pk, obj)

            # Take snapshot for future dirty tracking
            self._unit_of_work.register_clean(obj)

        # Process UPDATEs (dirty objects)
        for obj in list(self._unit_of_work._dirty):
            if isinstance(obj, int):
                continue  # Skip IDs
            dirty_fields = self._unit_of_work.get_dirty_fields(obj)
            if dirty_fields:
                table_name = self._get_table_name(obj)
                pk = self._get_pk(obj)
                pk_column = self._get_pk_column(obj)

                updates = {k: v[1] for k, v in dirty_fields.items()}
                await update_many(
                    table_name,
                    [(pk_column, "eq", pk)],
                    updates
                )

                # Refresh snapshot
                self._unit_of_work._dirty_tracker.refresh_snapshot(obj)

        # Process DELETEs
        for obj in self._unit_of_work.deleted_objects:
            table_name = self._get_table_name(obj)
            pk = self._get_pk(obj)
            pk_column = self._get_pk_column(obj)

            await delete_many(table_name, [(pk_column, "eq", pk)])

        # Clear pending operations (but keep snapshots)
        self._unit_of_work._new.clear()
        self._unit_of_work._dirty.clear()
        self._unit_of_work._deleted.clear()

    async def commit(self) -> None:
        """Flush and commit the current transaction."""
        # Fast path: no tracing
        if not is_tracing_enabled():
            await self.flush()
            if self.expire_on_commit:
                # Clear snapshots to force refresh on next access
                self._unit_of_work._dirty_tracker.clear()
            return

        # With tracing
        with create_session_span(operation="commit") as span:
            try:
                await self.flush()
                if self.expire_on_commit:
                    # Clear snapshots to force refresh on next access
                    self._unit_of_work._dirty_tracker.clear()

                # Record success
                if span is not None and hasattr(span, 'set_attribute'):
                    span.set_attribute("db.session.status", "committed")
            except Exception as e:
                add_exception(span, e)
                raise

    async def rollback(self) -> None:
        """Rollback the current transaction and clear pending changes."""
        # Fast path: no tracing
        if not is_tracing_enabled():
            self._unit_of_work.clear()
            self._identity_map.clear()
            return

        # With tracing
        with create_session_span(operation="rollback") as span:
            try:
                self._unit_of_work.clear()
                self._identity_map.clear()

                # Record success
                if span is not None and hasattr(span, 'set_attribute'):
                    span.set_attribute("db.session.status", "rolled_back")
            except Exception as e:
                add_exception(span, e)
                raise

    async def close(self) -> None:
        """Close the session."""
        self._closed = True
        self._unit_of_work.clear()
        self._identity_map.clear()
        self._loaded_relationships.clear()

    def expunge(self, obj: T) -> None:
        """Remove object from session without deleting from database."""
        pk = self._get_pk(obj)
        if pk is not None:
            table_name = self._get_table_name(obj)
            self._identity_map.remove(table_name, pk)

        self._unit_of_work._dirty_tracker.clear_snapshot(obj)
        if obj in self._unit_of_work._new:
            self._unit_of_work._new.remove(obj)
        if obj in self._unit_of_work._deleted:
            self._unit_of_work._deleted.remove(obj)

    def expunge_all(self) -> None:
        """Remove all objects from session."""
        self._identity_map.clear()
        self._unit_of_work.clear()

    def is_modified(self, obj: T) -> bool:
        """Check if object has uncommitted changes."""
        return self._unit_of_work.is_dirty(obj) or obj in self._unit_of_work._new

    def _track_relationship(self, instance: 'Table', relationship_name: str, loaded_value: Any) -> None:
        """
        Track a loaded relationship.

        This allows the session to keep track of which relationships have been
        loaded for which instances, enabling future optimizations.

        Args:
            instance: The Table instance that owns the relationship
            relationship_name: Name of the relationship attribute
            loaded_value: The loaded related object(s)

        Note:
            This is preparation for Phase 3 optimizations.
        """
        instance_id = id(instance)
        key = (instance_id, relationship_name)
        self._loaded_relationships[key] = loaded_value

    def _get_tracked_relationship(self, instance: 'Table', relationship_name: str) -> Optional[Any]:
        """
        Get a tracked relationship if it exists.

        Args:
            instance: The Table instance
            relationship_name: Name of the relationship attribute

        Returns:
            The tracked relationship value, or None if not tracked

        Note:
            This is preparation for Phase 3 optimizations.
        """
        instance_id = id(instance)
        key = (instance_id, relationship_name)
        return self._loaded_relationships.get(key)

    # Helper methods
    def _get_pk(self, obj: Any) -> Any:
        """Get primary key value from object."""
        if hasattr(obj, '_get_pk'):
            return obj._get_pk()
        return getattr(obj, 'id', None)

    def _get_pk_column(self, obj: Any) -> str:
        """Get primary key column name."""
        if hasattr(obj, '_get_pk_column'):
            return obj._get_pk_column()
        return 'id'

    def _get_table_name(self, obj: Any) -> str:
        """Get table name for object."""
        if hasattr(obj, '_get_table_name'):
            return obj._get_table_name()
        if hasattr(obj.__class__, '_get_table_name'):
            return obj.__class__._get_table_name()
        return obj.__class__.__name__.lower()

    def _get_data(self, obj: Any) -> Dict[str, Any]:
        """Get data dictionary from object for INSERT/UPDATE."""
        if hasattr(obj, '_get_column_values'):
            return obj._get_column_values()
        if hasattr(obj, 'to_dict'):
            return obj.to_dict()
        return {k: v for k, v in obj.__dict__.items() if not k.startswith('_')}


# Convenience function
def get_session() -> Optional[Session]:
    """Get the current active session."""
    return Session.get_current()


__all__ = [
    'ObjectState',
    'IdentityKey',
    'IdentityMap',
    'ObjectSnapshot',
    'DirtyTracker',
    'UnitOfWork',
    'Session',
    'get_session',
]
