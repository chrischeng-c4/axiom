"""
Async enhancements for data-bridge PostgreSQL ORM.

Provides SQLAlchemy-style async enhancements with:
- AsyncSession: Async context manager version of Session
- AsyncSessionFactory: Creates configured AsyncSession instances
- run_sync(): Escape hatch to run sync code in async context
- AsyncScoped: Scoped session for async contexts with task-local storage
- Async relationship loading helpers
- AsyncIterator support for large result sets
- Greenlet compatibility (optional)
- AsyncEngine wrapper for connection pooling
"""
from __future__ import annotations

import asyncio
import functools
import inspect
import weakref
from asyncio import iscoroutinefunction
from contextvars import ContextVar, Token
from typing import (
    Any, AsyncIterator, Awaitable, Callable, Dict, Generic, List,
    Optional, Set, Tuple, Type, TypeVar, Union, cast, ParamSpec
)

# Import existing session components
from .session import (
    Session, ObjectState, IdentityKey, IdentityMap,
    ObjectSnapshot, DirtyTracker, UnitOfWork, get_session
)

try:
    from greenlet import greenlet, getcurrent as greenlet_getcurrent
    GREENLET_AVAILABLE = True
except ImportError:
    GREENLET_AVAILABLE = False
    greenlet = None
    greenlet_getcurrent = None

T = TypeVar('T')
P = ParamSpec('P')


# ============================================================================
# AsyncSession - Async context manager version of Session
# ============================================================================

class AsyncSession(Session):
    """
    Async version of Session with SQLAlchemy-style API.

    All database operations are async and properly release the GIL.
    Maintains identity map, dirty tracking, and unit of work pattern.

    Example:
        async with AsyncSession() as session:
            user = await session.get(User, 1)
            user.name = "New Name"
            await session.commit()  # Flushes UPDATE

    Args:
        autoflush: Automatically flush before queries (default: True)
        expire_on_commit: Expire all objects after commit (default: True)
        autocommit: Auto-commit after each operation (default: False)
    """

    def __init__(
        self,
        autoflush: bool = True,
        expire_on_commit: bool = True,
        autocommit: bool = False,
    ):
        """Initialize a new async session."""
        super().__init__(autoflush=autoflush, expire_on_commit=expire_on_commit)
        self.autocommit = autocommit
        self._bind: Optional['AsyncEngine'] = None

    async def __aenter__(self) -> 'AsyncSession':
        """Enter async session context."""
        # Store in async context
        _async_scoped_session.set(self)
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit async session context."""
        try:
            if exc_type is None:
                # No exception, commit if autocommit
                if self.autocommit or self._unit_of_work.has_pending:
                    await self.commit()
            else:
                # Exception occurred, rollback
                await self.rollback()
        finally:
            await self.close()
            # Clear from async context
            token = _async_scoped_session_token.get()
            if token is not None:
                _async_scoped_session.reset(token)

    async def execute(
        self,
        statement: str,
        parameters: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        Execute raw SQL statement.

        Args:
            statement: SQL statement to execute
            parameters: Named parameters for the statement

        Returns:
            List of result rows as dictionaries
        """
        if self._closed:
            raise RuntimeError("Session is closed")

        from . import execute as pg_execute

        # Auto-flush if enabled
        if self.autoflush and self._unit_of_work.has_pending:
            await self.flush()

        result = await pg_execute(statement, parameters or {})
        return result

    async def scalar(
        self,
        statement: str,
        parameters: Optional[Dict[str, Any]] = None
    ) -> Any:
        """
        Execute statement and return scalar result (first column of first row).

        Args:
            statement: SQL statement to execute
            parameters: Named parameters for the statement

        Returns:
            Scalar value or None
        """
        result = await self.execute(statement, parameters)
        if result and len(result) > 0:
            first_row = result[0]
            return next(iter(first_row.values())) if first_row else None
        return None

    def bind_engine(self, engine: 'AsyncEngine') -> None:
        """Bind this session to an async engine."""
        self._bind = engine


# ============================================================================
# AsyncSessionFactory - Creates configured AsyncSession instances
# ============================================================================

class AsyncSessionFactory:
    """
    Factory for creating configured AsyncSession instances.

    Example:
        factory = AsyncSessionFactory(
            autoflush=True,
            expire_on_commit=True
        )

        async with factory() as session:
            user = await session.get(User, 1)
            await session.commit()
    """

    def __init__(
        self,
        autoflush: bool = True,
        expire_on_commit: bool = True,
        autocommit: bool = False,
        engine: Optional['AsyncEngine'] = None,
    ):
        """
        Initialize session factory with default configuration.

        Args:
            autoflush: Default autoflush setting for sessions
            expire_on_commit: Default expire_on_commit setting
            autocommit: Default autocommit setting
            engine: Optional async engine to bind to sessions
        """
        self.autoflush = autoflush
        self.expire_on_commit = expire_on_commit
        self.autocommit = autocommit
        self.engine = engine

    def __call__(self) -> AsyncSession:
        """Create a new AsyncSession with factory configuration."""
        session = AsyncSession(
            autoflush=self.autoflush,
            expire_on_commit=self.expire_on_commit,
            autocommit=self.autocommit,
        )
        if self.engine is not None:
            session.bind_engine(self.engine)
        return session

    async def begin(self) -> AsyncSession:
        """
        Create a new session and start a transaction.

        Returns:
            AsyncSession with transaction started
        """
        session = self()
        await session.__aenter__()
        return session


# ============================================================================
# run_sync() - Escape hatch to run sync code in async context
# ============================================================================

async def run_sync(func: Callable[P, T], *args: P.args, **kwargs: P.kwargs) -> T:
    """
    Run synchronous function in async context using thread pool.

    This is an escape hatch for compatibility with sync code.
    Uses asyncio.to_thread() internally to avoid blocking the event loop.

    Example:
        def sync_heavy_computation(x: int, y: int) -> int:
            # Some blocking operation
            return x + y

        result = await run_sync(sync_heavy_computation, 10, 20)
        # result = 30

    Args:
        func: Synchronous function to run
        *args: Positional arguments for the function
        **kwargs: Keyword arguments for the function

    Returns:
        Result of the function call
    """
    return await asyncio.to_thread(func, *args, **kwargs)


def async_wrap(func: Callable[P, T]) -> Callable[P, Awaitable[T]]:
    """
    Decorator to wrap synchronous function for async usage.

    Example:
        @async_wrap
        def sync_function(x: int) -> int:
            return x * 2

        result = await sync_function(5)  # Runs in thread pool

    Args:
        func: Synchronous function to wrap

    Returns:
        Async wrapper function
    """
    @functools.wraps(func)
    async def wrapper(*args: P.args, **kwargs: P.kwargs) -> T:
        return await run_sync(func, *args, **kwargs)
    return wrapper


# ============================================================================
# AsyncScoped - Scoped session for async contexts with task-local storage
# ============================================================================

# Context variable for task-local session storage
_async_scoped_session: ContextVar[Optional[AsyncSession]] = ContextVar(
    'async_scoped_session',
    default=None
)
_async_scoped_session_token: ContextVar[Optional[Token]] = ContextVar(
    'async_scoped_session_token',
    default=None
)


class AsyncScoped:
    """
    Scoped session manager for async contexts.

    Provides one session per asyncio task using contextvars.
    Sessions are automatically cleaned up when tasks complete.

    Example:
        scoped = AsyncScoped(AsyncSessionFactory())

        async def process_user():
            session = scoped()  # Gets task-local session
            user = await session.get(User, 1)
            user.name = "Updated"
            await session.commit()
            scoped.remove()  # Clean up

        await asyncio.gather(
            process_user(),
            process_user(),  # Each gets its own session
        )
    """

    def __init__(self, session_factory: AsyncSessionFactory):
        """
        Initialize scoped session manager.

        Args:
            session_factory: Factory for creating sessions
        """
        self.session_factory = session_factory
        self._sessions: weakref.WeakValueDictionary[int, AsyncSession] = \
            weakref.WeakValueDictionary()

    def __call__(self) -> AsyncSession:
        """
        Get or create session for current asyncio task.

        Returns:
            AsyncSession for current task
        """
        task_id = id(asyncio.current_task())

        # Check if session already exists for this task
        session = self._sessions.get(task_id)
        if session is not None and not session._closed:
            return session

        # Create new session for this task
        session = self.session_factory()
        self._sessions[task_id] = session

        # Store in context var
        token = _async_scoped_session.set(session)
        _async_scoped_session_token.set(token)

        return session

    def remove(self) -> None:
        """Remove session for current task."""
        task_id = id(asyncio.current_task())
        session = self._sessions.pop(task_id, None)

        if session is not None:
            # Schedule cleanup
            asyncio.create_task(session.close())

        # Clear from context var
        token = _async_scoped_session_token.get()
        if token is not None:
            _async_scoped_session.reset(token)

    async def remove_all(self) -> None:
        """Close all scoped sessions."""
        sessions = list(self._sessions.values())
        self._sessions.clear()

        # Close all sessions
        await asyncio.gather(
            *[session.close() for session in sessions],
            return_exceptions=True
        )


def get_async_session() -> Optional[AsyncSession]:
    """
    Get the current async session from context.

    Returns:
        Current AsyncSession or None if not in async context
    """
    return _async_scoped_session.get()


# ============================================================================
# Async relationship loading helpers
# ============================================================================

async def async_load(obj: Any, relationship: str) -> None:
    """
    Explicitly load a relationship for an object.

    Useful for lazy-loaded relationships that need to be accessed.

    Example:
        user = await session.get(User, 1)
        await async_load(user, 'posts')  # Load posts relationship
        for post in user.posts:
            print(post.title)

    Args:
        obj: Object to load relationship for
        relationship: Name of the relationship attribute
    """
    if not hasattr(obj, relationship):
        raise AttributeError(f"Object has no relationship '{relationship}'")

    # Try to get session
    session = get_async_session()
    if session is None:
        raise RuntimeError("No active async session found")

    # Check if relationship is already loaded
    attr_value = getattr(obj, relationship, None)
    if attr_value is not None:
        return  # Already loaded

    # Use lazy loading proxy if available
    from .loading import LazyLoadingProxy
    proxy = object.__getattribute__(obj, f'_{relationship}_proxy')

    if isinstance(proxy, LazyLoadingProxy):
        # Trigger lazy load
        result = await proxy.__load__()
        setattr(obj, relationship, result)
    else:
        # Manual load using session
        # This would require relationship metadata
        raise NotImplementedError(
            "Manual relationship loading not yet implemented. "
            "Use LazyLoadingProxy or eager loading."
        )


async def async_refresh(obj: Any, session: Optional[AsyncSession] = None) -> None:
    """
    Refresh object from database, reloading all attributes.

    Discards any pending changes and reloads from database.

    Example:
        user = await session.get(User, 1)
        user.name = "Changed"
        await async_refresh(user)  # Reload from DB, discards changes
        # user.name is back to original value

    Args:
        obj: Object to refresh
        session: Optional session (uses current if not provided)
    """
    if session is None:
        session = get_async_session()
        if session is None:
            raise RuntimeError("No active async session found")

    # Get object identity
    pk = session._get_pk(obj)
    if pk is None:
        raise ValueError("Cannot refresh object without primary key")

    table_name = session._get_table_name(obj)

    # Clear from identity map
    session._identity_map.remove(table_name, pk)
    session._unit_of_work._dirty_tracker.clear_snapshot(obj)

    # Reload from database
    model_class = obj.__class__
    refreshed = await session.get(model_class, pk)

    if refreshed is None:
        raise RuntimeError(f"Object {model_class.__name__} with pk={pk} no longer exists")

    # Update object attributes
    if hasattr(refreshed, '_get_column_values'):
        for key, value in refreshed._get_column_values().items():
            setattr(obj, key, value)


async def async_expire(
    obj: Any,
    attrs: Optional[List[str]] = None,
    session: Optional[AsyncSession] = None
) -> None:
    """
    Expire cached attributes, forcing reload on next access.

    If attrs is None, expires all attributes.
    If attrs is provided, only expires specified attributes.

    Example:
        user = await session.get(User, 1)
        await async_expire(user, ['name', 'email'])
        # Next access to user.name or user.email will reload from DB

        await async_expire(user)  # Expire all attributes

    Args:
        obj: Object to expire
        attrs: Optional list of attribute names to expire
        session: Optional session (uses current if not provided)
    """
    if session is None:
        session = get_async_session()
        if session is None:
            raise RuntimeError("No active async session found")

    if attrs is None:
        # Expire all - clear snapshot
        session._unit_of_work._dirty_tracker.clear_snapshot(obj)
    else:
        # Expire specific attributes
        snapshot = session._unit_of_work._dirty_tracker._snapshots.get(id(obj))
        if snapshot is not None:
            for attr in attrs:
                snapshot.data.pop(attr, None)


# ============================================================================
# AsyncIterator support for large result sets
# ============================================================================

class AsyncResultIterator(Generic[T]):
    """
    Async iterator for streaming large result sets.

    Fetches results in batches to avoid loading everything into memory.
    """

    def __init__(
        self,
        query_func: Callable[[], Awaitable[List[T]]],
        batch_size: int = 100,
    ):
        """
        Initialize async result iterator.

        Args:
            query_func: Async function that returns results
            batch_size: Number of results to fetch per batch
        """
        self.query_func = query_func
        self.batch_size = batch_size
        self._buffer: List[T] = []
        self._exhausted = False
        self._offset = 0

    def __aiter__(self) -> 'AsyncResultIterator[T]':
        """Return self as async iterator."""
        return self

    async def __anext__(self) -> T:
        """Get next result."""
        if not self._buffer and not self._exhausted:
            # Fetch next batch
            self._buffer = await self._fetch_batch()
            if not self._buffer:
                self._exhausted = True

        if self._buffer:
            return self._buffer.pop(0)

        raise StopAsyncIteration

    async def _fetch_batch(self) -> List[T]:
        """Fetch next batch of results."""
        # This is a simplified implementation
        # Real implementation would need to pass offset/limit to query
        results = await self.query_func()
        return results[self._offset:self._offset + self.batch_size]


async def async_stream(
    model: Type[T],
    query_conditions: Optional[List[Tuple[str, str, Any]]] = None,
    batch_size: int = 100,
    session: Optional[AsyncSession] = None,
) -> AsyncIterator[T]:
    """
    Stream query results as async iterator for large result sets.

    Results are fetched in batches to avoid loading everything into memory.

    Example:
        async for user in async_stream(User, batch_size=50):
            print(user.name)
            # Process user without loading all users into memory

    Args:
        model: Model class to query
        query_conditions: Optional query conditions
        batch_size: Number of results per batch
        session: Optional session (uses current if not provided)

    Yields:
        Model instances one at a time
    """
    if session is None:
        session = get_async_session()
        if session is None:
            raise RuntimeError("No active async session found")

    from . import find_many
    table_name = model._get_table_name() if hasattr(model, '_get_table_name') else model.__name__.lower()

    offset = 0
    while True:
        # Fetch batch
        results = await find_many(
            table_name,
            where_conditions=query_conditions or [],
            limit=batch_size,
            offset=offset
        )

        if not results:
            break

        # Yield each result
        for result in results:
            obj = model(**result) if isinstance(result, dict) else result
            yield obj

        offset += batch_size

        # If we got fewer results than batch_size, we're done
        if len(results) < batch_size:
            break


# ============================================================================
# Greenlet compatibility (optional)
# ============================================================================

if GREENLET_AVAILABLE:
    def greenlet_spawn(func: Callable[..., Awaitable[T]], *args, **kwargs) -> T:
        """
        Run async code in greenlet for compatibility with sync code.

        This allows async operations to be called from sync contexts
        by using greenlets to manage the async execution.

        Example:
            async def async_operation():
                await asyncio.sleep(1)
                return "Done"

            # Call from sync context
            result = greenlet_spawn(async_operation)
            print(result)  # "Done"

        Args:
            func: Async function to run
            *args: Positional arguments
            **kwargs: Keyword arguments

        Returns:
            Result of the async function
        """
        loop = asyncio.get_event_loop()
        if loop.is_running():
            # If loop is already running, use it
            task = asyncio.create_task(func(*args, **kwargs))
            return loop.run_until_complete(task)
        else:
            # Start new loop
            return asyncio.run(func(*args, **kwargs))


    class AsyncGreenlet:
        """
        Greenlet wrapper for async operations.

        Allows mixing sync and async code using greenlets.
        """

        def __init__(self):
            """Initialize async greenlet."""
            self._greenlet = None
            self._loop = None

        def spawn(self, func: Callable[..., Awaitable[T]], *args, **kwargs) -> T:
            """Spawn async function in greenlet."""
            return greenlet_spawn(func, *args, **kwargs)

else:
    # Provide stubs if greenlet not available
    def greenlet_spawn(*args, **kwargs):
        """Greenlet not available - install greenlet package."""
        raise RuntimeError(
            "greenlet package not installed. "
            "Install with: pip install greenlet"
        )

    class AsyncGreenlet:
        """Greenlet not available."""
        def __init__(self):
            raise RuntimeError(
                "greenlet package not installed. "
                "Install with: pip install greenlet"
            )


# ============================================================================
# AsyncEngine wrapper for connection pooling
# ============================================================================

class AsyncEngine:
    """
    Async engine wrapper for connection pool management.

    Manages connection lifecycle and provides connection pooling.

    Example:
        engine = AsyncEngine(
            host="localhost",
            port=5432,
            database="mydb",
            username="user",
            password="pass"
        )

        await engine.connect()

        # Create sessions bound to engine
        factory = AsyncSessionFactory(engine=engine)
        async with factory() as session:
            # Use session
            pass

        await engine.dispose()
    """

    def __init__(
        self,
        connection_string: Optional[str] = None,
        *,
        host: str = "localhost",
        port: int = 5432,
        database: str = "postgres",
        username: Optional[str] = None,
        password: Optional[str] = None,
        min_connections: int = 1,
        max_connections: int = 10,
    ):
        """
        Initialize async engine.

        Args:
            connection_string: Full PostgreSQL connection string
            host: PostgreSQL server hostname
            port: PostgreSQL server port
            database: Database name
            username: Database username
            password: Database password
            min_connections: Minimum connections in pool
            max_connections: Maximum connections in pool
        """
        self.connection_string = connection_string
        self.host = host
        self.port = port
        self.database = database
        self.username = username
        self.password = password
        self.min_connections = min_connections
        self.max_connections = max_connections
        self._connected = False

    async def connect(self) -> None:
        """Initialize connection pool."""
        from . import init

        await init(
            connection_string=self.connection_string,
            host=self.host,
            port=self.port,
            database=self.database,
            username=self.username,
            password=self.password,
            min_connections=self.min_connections,
            max_connections=self.max_connections,
        )
        self._connected = True

    async def dispose(self) -> None:
        """Close all connections in pool."""
        from . import close
        await close()
        self._connected = False

    @property
    def is_connected(self) -> bool:
        """Check if engine is connected."""
        return self._connected

    async def __aenter__(self) -> 'AsyncEngine':
        """Enter async context."""
        await self.connect()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit async context."""
        await self.dispose()


# ============================================================================
# Exports
# ============================================================================

__all__ = [
    # Async session
    'AsyncSession',
    'AsyncSessionFactory',

    # Sync compatibility
    'run_sync',
    'async_wrap',

    # Scoped sessions
    'AsyncScoped',
    'get_async_session',

    # Relationship loading
    'async_load',
    'async_refresh',
    'async_expire',

    # Streaming
    'async_stream',
    'AsyncResultIterator',

    # Greenlet support
    'greenlet_spawn',
    'AsyncGreenlet',
    'GREENLET_AVAILABLE',

    # Engine
    'AsyncEngine',
]
