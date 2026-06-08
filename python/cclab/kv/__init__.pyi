from typing import Any

__all__ = ["KvClient", "_PoolConfig", "_PoolStats", "_KvPool"]

class KvClient:
    """KV Store client for connecting to kv-server"""
    @staticmethod
    def connect(addr: str) -> Any:
        """Connect to a KV server"""
        ...
    def ping(self) -> Any:
        """Ping the server"""
        ...
    def get(self, key: str) -> Any:
        """Get a value by key"""
        ...
    def set(self, key: str, value: Any, ttl: float | None = None) -> Any:
        """Set a value"""
        ...
    def delete(self, key: str) -> Any:
        """Delete a key"""
        ...
    def exists(self, key: str) -> Any:
        """Check if a key exists"""
        ...
    def incr(self, key: str, delta: int = 1) -> Any:
        """Atomically increment an integer value"""
        ...
    def decr(self, key: str, delta: int = 1) -> Any:
        """Atomically decrement an integer value"""
        ...
    def info(self) -> Any:
        """Get server info"""
        ...
    def setnx(self, key: str, value: Any, ttl: float | None = None) -> Any:
        """Set if not exists (atomic)"""
        ...
    def lock(self, key: str, owner: str, ttl: float = 30.0) -> Any:
        """Acquire a distributed lock"""
        ...
    def unlock(self, key: str, owner: str) -> Any:
        """Release a distributed lock"""
        ...
    def extend_lock(self, key: str, owner: str, ttl: float = 30.0) -> Any:
        """Extend lock TTL"""
        ...
    def mget(self, keys: list[str]) -> Any:
        """Get multiple values by keys (MGET)"""
        ...
    def mset(self, pairs: list[tuple[str, Any]], ttl: float | None = None) -> Any:
        """Set multiple key-value pairs (MSET)"""
        ...
    def mdel(self, keys: list[str]) -> Any:
        """Delete multiple keys (MDEL)"""
        ...
    @property
    def namespace(self) -> str | None:
        """Get the namespace for this client"""
        ...
    def __repr__(self) -> str:
        ...

class _PoolConfig:
    """Python pool configuration"""
    def __init__(self, addr: str, min_size: int = 2, max_size: int = 10, idle_timeout: float = 300.0, acquire_timeout: float = 5.0) -> None:
        ...

class _PoolStats:
    """Python pool stats"""
    idle: int
    active: int
    max_size: int

class _KvPool:
    """Python KV pool"""
    @staticmethod
    def connect(config: Any) -> Any:
        """Connect to a KV server with pooling"""
        ...
    @property
    def namespace(self) -> str | None:
        """Get the namespace for this pool"""
        ...
    def stats(self) -> Any:
        """Get pool statistics"""
        ...
    def ping(self) -> Any:
        """Ping the server"""
        ...
    def get(self, key: str) -> Any:
        """Get a value by key"""
        ...
    def set(self, key: str, value: Any, ttl: float | None = None) -> Any:
        """Set a value"""
        ...
    def delete(self, key: str) -> Any:
        """Delete a key"""
        ...
    def exists(self, key: str) -> Any:
        """Check if a key exists"""
        ...
    def incr(self, key: str, delta: int = 1) -> Any:
        """Atomically increment an integer value"""
        ...
    def decr(self, key: str, delta: int = 1) -> Any:
        """Atomically decrement an integer value"""
        ...
    def info(self) -> Any:
        """Get server info"""
        ...
    def setnx(self, key: str, value: Any, ttl: float | None = None) -> Any:
        """Set if not exists (atomic)"""
        ...
    def lock(self, key: str, owner: str, ttl: float = 30.0) -> Any:
        """Acquire a distributed lock"""
        ...
    def unlock(self, key: str, owner: str) -> Any:
        """Release a distributed lock"""
        ...
    def extend_lock(self, key: str, owner: str, ttl: float = 30.0) -> Any:
        """Extend lock TTL"""
        ...
    def __repr__(self) -> str:
        ...

