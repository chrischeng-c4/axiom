"""Hive/Presto client with connection pooling and zero-copy DataFrame conversion."""

try:
    from cclab._hive import HiveConnection, PrestoConnection, ResultBatch, connect, presto
except ImportError:
    pass

__all__ = [
    "HiveConnection",
    "PrestoConnection",
    "ResultBatch",
    "connect",
    "presto",
]
