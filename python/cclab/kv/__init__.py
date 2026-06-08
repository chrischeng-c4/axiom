"""
KV Store client for data-bridge.

High-performance key-value store client that connects to kv-server via TCP.

Example:
    >>> from cclab.kv import KvClient
    >>>
    >>> async with KvClient.connect("127.0.0.1:6380") as client:
    ...     await client.set("key", "value")
    ...     result = await client.get("key")
    ...     print(result)
    'value'
"""

from __future__ import annotations

from typing import Any, Optional, Union
from decimal import Decimal

# Import from Rust bindings (cclab-kv-pyo3 → _ion module)
try:
    from .._ion import KvClient as _KvClient, _PoolConfig, _PoolStats, _KvPool  # type: ignore
except ImportError:
    # KV feature not enabled
    _KvClient = None
    _PoolConfig = None
    _PoolStats = None
    _KvPool = None

__all__ = ["KvClient", "KvValue", "Lock", "KvPool", "PoolConfig", "PoolStats"]

# Type alias for supported value types
KvValue = Union[None, int, float, Decimal, str, bytes, list, dict]


class KvClient:
    """
    Async KV store client.

    Connects to a kv-server instance via TCP and provides high-performance
    key-value operations.

    Attributes:
        _client: The underlying Rust client instance.
    """

    __slots__ = ("_client",)

    def __init__(self, client: Any) -> None:
        """Initialize with an existing client instance."""
        self._client = client

    @classmethod
    async def connect(cls, addr: str = "127.0.0.1:6380") -> "KvClient":
        """
        Connect to a KV server.

        Args:
            addr: Server address in format "host:port" or "host:port/namespace"
                  Examples:
                  - "127.0.0.1:6380" - no namespace
                  - "127.0.0.1:6380/tasks" - namespace "tasks"
                  - "127.0.0.1:6380/prod/cache" - nested namespace
                  Defaults to "127.0.0.1:6380".

        Returns:
            Connected KvClient instance.

        Raises:
            ConnectionError: If connection fails.

        Example:
            >>> client = await KvClient.connect("localhost:6380")
            >>> client_with_ns = await KvClient.connect("localhost:6380/tasks")
        """
        if _KvClient is None:
            raise ImportError(
                "KV module not available. "
                "Rebuild with: maturin develop --features kv"
            )
        client = await _KvClient.connect(addr)
        return cls(client)

    async def __aenter__(self) -> "KvClient":
        """Async context manager entry."""
        return self

    async def __aexit__(self, *args: Any) -> None:
        """Async context manager exit."""
        # Connection cleanup handled by Rust
        pass

    @property
    def namespace(self) -> Optional[str]:
        """
        Get the namespace for this client, or None if not configured.

        Returns:
            The namespace string if configured, None otherwise.

        Example:
            >>> client = await KvClient.connect("localhost:6380/tasks")
            >>> print(client.namespace)
            'tasks'
            >>>
            >>> client2 = await KvClient.connect("localhost:6380")
            >>> print(client2.namespace)
            None
        """
        return self._client.namespace

    async def ping(self) -> str:
        """
        Ping the server.

        Returns:
            "PONG" if server is responsive.

        Raises:
            ConnectionError: If server is unreachable.
        """
        return await self._client.ping()

    async def get(self, key: str) -> Optional[KvValue]:
        """
        Get a value by key.

        Args:
            key: The key to look up (max 256 characters).

        Returns:
            The value if found, None otherwise.

        Example:
            >>> value = await client.get("mykey")
            >>> if value is not None:
            ...     print(f"Found: {value}")
        """
        return await self._client.get(key)

    async def set(
        self,
        key: str,
        value: KvValue,
        ttl: Optional[float] = None,
    ) -> None:
        """
        Set a value.

        Args:
            key: The key to set (max 256 characters).
            value: The value to store. Supported types:
                   int, float, Decimal, str, bytes, list, dict, None
            ttl: Optional time-to-live in seconds.

        Example:
            >>> await client.set("name", "Alice")
            >>> await client.set("counter", 0)
            >>> await client.set("temp", 123, ttl=60)  # Expires in 60s
        """
        await self._client.set(key, value, ttl)

    async def delete(self, key: str) -> bool:
        """
        Delete a key.

        Args:
            key: The key to delete.

        Returns:
            True if the key existed, False otherwise.

        Example:
            >>> deleted = await client.delete("mykey")
            >>> print(f"Key was {'deleted' if deleted else 'not found'}")
        """
        return await self._client.delete(key)

    async def exists(self, key: str) -> bool:
        """
        Check if a key exists.

        Args:
            key: The key to check.

        Returns:
            True if the key exists (and not expired).

        Example:
            >>> if await client.exists("mykey"):
            ...     print("Key exists!")
        """
        return await self._client.exists(key)

    async def incr(self, key: str, delta: int = 1) -> int:
        """
        Atomically increment an integer value.

        If the key doesn't exist, it's created with the delta as initial value.

        Args:
            key: The key to increment.
            delta: Amount to add (default: 1).

        Returns:
            The new value after incrementing.

        Raises:
            TypeError: If the existing value is not an integer.

        Example:
            >>> await client.set("counter", 10)
            >>> new_value = await client.incr("counter", 5)
            >>> print(new_value)  # 15
        """
        return await self._client.incr(key, delta)

    async def decr(self, key: str, delta: int = 1) -> int:
        """
        Atomically decrement an integer value.

        Args:
            key: The key to decrement.
            delta: Amount to subtract (default: 1).

        Returns:
            The new value after decrementing.

        Raises:
            TypeError: If the existing value is not an integer.

        Example:
            >>> await client.set("counter", 10)
            >>> new_value = await client.decr("counter", 3)
            >>> print(new_value)  # 7
        """
        return await self._client.decr(key, delta)

    async def info(self) -> str:
        """
        Get server information.

        Returns:
            JSON string with server statistics.

        Example:
            >>> info = await client.info()
            >>> print(info)
            {"shards": 256, "entries": 1000}
        """
        return await self._client.info()

    async def setnx(
        self,
        key: str,
        value: KvValue,
        ttl: Optional[float] = None,
    ) -> bool:
        """
        Set if not exists (atomic).

        Args:
            key: The key to set.
            value: The value to store.
            ttl: Optional time-to-live in seconds.

        Returns:
            True if the key was set, False if it already exists.

        Example:
            >>> success = await client.setnx("unique_key", "value")
            >>> if success:
            ...     print("Key was set")
            ... else:
            ...     print("Key already exists")
        """
        return await self._client.setnx(key, value, ttl)

    async def lock(
        self,
        key: str,
        owner: str,
        ttl: float = 30.0,
    ) -> bool:
        """
        Acquire a distributed lock.

        Args:
            key: The lock key (e.g., "lock:resource").
            owner: Unique identifier for the lock owner (e.g., "worker-123").
            ttl: Lock time-to-live in seconds (default: 30).

        Returns:
            True if lock was acquired, False if already held.

        Example:
            >>> if await client.lock("lock:task:123", "worker-1", ttl=60):
            ...     try:
            ...         # Do work while holding lock
            ...         pass
            ...     finally:
            ...         await client.unlock("lock:task:123", "worker-1")
        """
        return await self._client.lock(key, owner, ttl)

    async def unlock(self, key: str, owner: str) -> bool:
        """
        Release a distributed lock.

        Args:
            key: The lock key.
            owner: The lock owner (must match the owner who acquired it).

        Returns:
            True if lock was released, False if not held or wrong owner.

        Example:
            >>> released = await client.unlock("lock:task:123", "worker-1")
        """
        return await self._client.unlock(key, owner)

    async def extend_lock(self, key: str, owner: str, ttl: float = 30.0) -> bool:
        """
        Extend a lock's TTL.

        Args:
            key: The lock key.
            owner: The lock owner (must match).
            ttl: New TTL in seconds.

        Returns:
            True if extended, False if not held or wrong owner.

        Example:
            >>> # Extend lock while doing long work
            >>> await client.extend_lock("lock:task:123", "worker-1", ttl=60)
        """
        return await self._client.extend_lock(key, owner, ttl)

    async def mget(self, keys: list[str]) -> list[Optional[KvValue]]:
        """
        Get multiple values by keys in a single operation (MGET).

        This is significantly faster than multiple individual GET calls
        due to reduced network round-trips (N round-trips → 1 round-trip).

        Args:
            keys: List of keys to fetch.

        Returns:
            List of values in the same order as keys. None for missing keys.

        Example:
            >>> # Fetch 100 task results in one operation
            >>> task_ids = [f"task:{i}" for i in range(100)]
            >>> results = await client.mget(task_ids)
            >>> for task_id, result in zip(task_ids, results):
            ...     if result is not None:
            ...         print(f"{task_id}: {result}")
        """
        return await self._client.mget(keys)

    async def mset(
        self,
        pairs: list[tuple[str, KvValue]],
        ttl: Optional[float] = None,
    ) -> None:
        """
        Set multiple key-value pairs in a single operation (MSET).

        This is significantly faster than multiple individual SET calls
        due to reduced network round-trips (N round-trips → 1 round-trip).

        Args:
            pairs: List of (key, value) tuples to set.
            ttl: Optional time-to-live in seconds (applies to all keys).

        Example:
            >>> # Store 100 task results in one operation
            >>> pairs = [(f"task:{i}", f"result_{i}") for i in range(100)]
            >>> await client.mset(pairs, ttl=3600)
        """
        await self._client.mset(pairs, ttl)

    async def mdel(self, keys: list[str]) -> int:
        """
        Delete multiple keys in a single operation (MDEL).

        This is significantly faster than multiple individual DELETE calls
        due to reduced network round-trips (N round-trips → 1 round-trip).

        Args:
            keys: List of keys to delete.

        Returns:
            Number of keys that were actually deleted.

        Example:
            >>> # Clean up 100 expired task keys in one operation
            >>> task_ids = [f"task:{i}" for i in range(100)]
            >>> deleted = await client.mdel(task_ids)
            >>> print(f"Deleted {deleted} keys")
        """
        return await self._client.mdel(keys)

    def __repr__(self) -> str:
        return "KvClient(connected)"


class Lock:
    """
    Async context manager for distributed locks.

    Example:
        >>> async with Lock(client, "resource", "worker-1", ttl=30) as acquired:
        ...     if acquired:
        ...         # Do work while holding lock
        ...         pass
    """

    __slots__ = ("_client", "_key", "_owner", "_ttl", "_acquired")

    def __init__(
        self,
        client: KvClient,
        key: str,
        owner: str,
        ttl: float = 30.0,
    ) -> None:
        self._client = client
        self._key = key
        self._owner = owner
        self._ttl = ttl
        self._acquired = False

    async def __aenter__(self) -> bool:
        """Acquire the lock."""
        self._acquired = await self._client.lock(self._key, self._owner, self._ttl)
        return self._acquired

    async def __aexit__(self, *args: Any) -> None:
        """Release the lock if acquired."""
        if self._acquired:
            await self._client.unlock(self._key, self._owner)

    async def extend(self, ttl: Optional[float] = None) -> bool:
        """Extend the lock TTL."""
        if not self._acquired:
            return False
        return await self._client.extend_lock(
            self._key, self._owner, ttl or self._ttl
        )


class PoolConfig:
    """Connection pool configuration."""

    __slots__ = ("_inner",)

    def __init__(
        self,
        addr: str,
        *,
        min_size: int = 2,
        max_size: int = 10,
        idle_timeout: float = 300.0,
        acquire_timeout: float = 5.0,
    ) -> None:
        """
        Create pool configuration.

        Args:
            addr: Server address in format "host:port" or "host:port/namespace"
            min_size: Minimum number of connections to keep alive (default: 2)
            max_size: Maximum number of connections allowed (default: 10)
            idle_timeout: Connection idle timeout in seconds (default: 300)
            acquire_timeout: Timeout for acquiring connection in seconds (default: 5)

        Example:
            >>> config = PoolConfig(
            ...     "127.0.0.1:6380/cache",
            ...     min_size=5,
            ...     max_size=20,
            ...     idle_timeout=600.0
            ... )
        """
        if _PoolConfig is None:
            raise ImportError(
                "KV module not available. "
                "Rebuild with: maturin develop --features kv"
            )
        self._inner = _PoolConfig(addr, min_size, max_size, idle_timeout, acquire_timeout)


class PoolStats:
    """Pool statistics."""

    __slots__ = ("_inner",)

    def __init__(self, inner: Any) -> None:
        """Initialize with Rust stats object."""
        self._inner = inner

    @property
    def idle(self) -> int:
        """Number of idle connections in the pool."""
        return self._inner.idle

    @property
    def active(self) -> int:
        """Number of active (in-use) connections."""
        return self._inner.active

    @property
    def max_size(self) -> int:
        """Maximum pool size."""
        return self._inner.max_size

    def __repr__(self) -> str:
        return f"PoolStats(idle={self.idle}, active={self.active}, max_size={self.max_size})"


class KvPool:
    """
    Connection pool for KV client.

    Provides automatic connection pooling with min/max size limits,
    idle timeout, and automatic connection recycling.

    Attributes:
        _pool: The underlying Rust pool instance.
    """

    __slots__ = ("_pool",)

    def __init__(self, pool: Any) -> None:
        """Initialize with an existing pool instance."""
        self._pool = pool

    @classmethod
    async def connect(cls, config: PoolConfig) -> "KvPool":
        """
        Connect to a KV server with connection pooling.

        Args:
            config: Pool configuration.

        Returns:
            Connected KvPool instance.

        Raises:
            ConnectionError: If connection fails.

        Example:
            >>> config = PoolConfig("127.0.0.1:6380/cache", min_size=5, max_size=20)
            >>> pool = await KvPool.connect(config)
            >>> await pool.set("key", "value")
            >>> stats = await pool.stats()
            >>> print(stats)
            PoolStats(idle=4, active=1, max_size=20)
        """
        if _KvPool is None:
            raise ImportError(
                "KV module not available. "
                "Rebuild with: maturin develop --features kv"
            )
        pool = await _KvPool.connect(config._inner)
        return cls(pool)

    @property
    def namespace(self) -> Optional[str]:
        """
        Get the namespace for this pool, or None if not configured.

        Returns:
            The namespace string if configured, None otherwise.

        Example:
            >>> config = PoolConfig("127.0.0.1:6380/tasks")
            >>> pool = await KvPool.connect(config)
            >>> print(pool.namespace)
            'tasks'
        """
        return self._pool.namespace

    async def stats(self) -> PoolStats:
        """
        Get pool statistics.

        Returns:
            PoolStats object with idle, active, and max_size counts.

        Example:
            >>> stats = await pool.stats()
            >>> print(f"Pool has {stats.idle} idle and {stats.active} active connections")
        """
        stats = await self._pool.stats()
        return PoolStats(stats)

    async def ping(self) -> str:
        """
        Ping the server.

        Returns:
            "PONG" if server is responsive.

        Raises:
            ConnectionError: If server is unreachable.
        """
        return await self._pool.ping()

    async def get(self, key: str) -> Optional[KvValue]:
        """
        Get a value by key.

        Args:
            key: The key to look up (max 256 characters).

        Returns:
            The value if found, None otherwise.

        Example:
            >>> value = await pool.get("mykey")
        """
        return await self._pool.get(key)

    async def set(
        self,
        key: str,
        value: KvValue,
        ttl: Optional[float] = None,
    ) -> None:
        """
        Set a value.

        Args:
            key: The key to set (max 256 characters).
            value: The value to store.
            ttl: Optional time-to-live in seconds.

        Example:
            >>> await pool.set("name", "Alice")
            >>> await pool.set("temp", 123, ttl=60)
        """
        await self._pool.set(key, value, ttl)

    async def delete(self, key: str) -> bool:
        """
        Delete a key.

        Args:
            key: The key to delete.

        Returns:
            True if the key existed, False otherwise.
        """
        return await self._pool.delete(key)

    async def exists(self, key: str) -> bool:
        """
        Check if a key exists.

        Args:
            key: The key to check.

        Returns:
            True if the key exists (and not expired).
        """
        return await self._pool.exists(key)

    async def incr(self, key: str, delta: int = 1) -> int:
        """
        Atomically increment an integer value.

        Args:
            key: The key to increment.
            delta: Amount to add (default: 1).

        Returns:
            The new value after incrementing.
        """
        return await self._pool.incr(key, delta)

    async def decr(self, key: str, delta: int = 1) -> int:
        """
        Atomically decrement an integer value.

        Args:
            key: The key to decrement.
            delta: Amount to subtract (default: 1).

        Returns:
            The new value after decrementing.
        """
        return await self._pool.decr(key, delta)

    async def info(self) -> str:
        """
        Get server information.

        Returns:
            JSON string with server statistics.
        """
        return await self._pool.info()

    async def setnx(
        self,
        key: str,
        value: KvValue,
        ttl: Optional[float] = None,
    ) -> bool:
        """
        Set if not exists (atomic).

        Args:
            key: The key to set.
            value: The value to store.
            ttl: Optional time-to-live in seconds.

        Returns:
            True if the key was set, False if it already exists.
        """
        return await self._pool.setnx(key, value, ttl)

    async def lock(
        self,
        key: str,
        owner: str,
        ttl: float = 30.0,
    ) -> bool:
        """
        Acquire a distributed lock.

        Args:
            key: The lock key.
            owner: Unique identifier for the lock owner.
            ttl: Lock time-to-live in seconds (default: 30).

        Returns:
            True if lock was acquired, False if already held.
        """
        return await self._pool.lock(key, owner, ttl)

    async def unlock(self, key: str, owner: str) -> bool:
        """
        Release a distributed lock.

        Args:
            key: The lock key.
            owner: The lock owner (must match).

        Returns:
            True if lock was released, False if not held or wrong owner.
        """
        return await self._pool.unlock(key, owner)

    async def extend_lock(self, key: str, owner: str, ttl: float = 30.0) -> bool:
        """
        Extend a lock's TTL.

        Args:
            key: The lock key.
            owner: The lock owner (must match).
            ttl: New TTL in seconds.

        Returns:
            True if extended, False if not held or wrong owner.
        """
        return await self._pool.extend_lock(key, owner, ttl)

    async def mget(self, keys: list[str]) -> list[Optional[KvValue]]:
        """
        Get multiple values by keys in a single operation (MGET).

        This is significantly faster than multiple individual GET calls
        due to reduced network round-trips (N round-trips → 1 round-trip).

        Args:
            keys: List of keys to fetch.

        Returns:
            List of values in the same order as keys. None for missing keys.

        Example:
            >>> # Fetch 100 task results in one operation
            >>> task_ids = [f"task:{i}" for i in range(100)]
            >>> results = await pool.mget(task_ids)
        """
        return await self._pool.mget(keys)

    async def mset(
        self,
        pairs: list[tuple[str, KvValue]],
        ttl: Optional[float] = None,
    ) -> None:
        """
        Set multiple key-value pairs in a single operation (MSET).

        This is significantly faster than multiple individual SET calls
        due to reduced network round-trips (N round-trips → 1 round-trip).

        Args:
            pairs: List of (key, value) tuples to set.
            ttl: Optional time-to-live in seconds (applies to all keys).

        Example:
            >>> # Store 100 task results in one operation
            >>> pairs = [(f"task:{i}", f"result_{i}") for i in range(100)]
            >>> await pool.mset(pairs, ttl=3600)
        """
        await self._pool.mset(pairs, ttl)

    async def mdel(self, keys: list[str]) -> int:
        """
        Delete multiple keys in a single operation (MDEL).

        This is significantly faster than multiple individual DELETE calls
        due to reduced network round-trips (N round-trips → 1 round-trip).

        Args:
            keys: List of keys to delete.

        Returns:
            Number of keys that were actually deleted.

        Example:
            >>> # Clean up 100 expired task keys in one operation
            >>> task_ids = [f"task:{i}" for i in range(100)]
            >>> deleted = await pool.mdel(task_ids)
        """
        return await self._pool.mdel(keys)

    def __repr__(self) -> str:
        return f"KvPool(namespace={self.namespace!r})"
