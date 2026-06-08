"""
Transaction support for data-bridge.

IMPORTANT: Transaction support requires Rust backend implementation.
This module provides the API structure for future implementation.

Planned Usage:
    >>> from cclab.mongo import start_session
    >>>
    >>> async with start_session() as session:
    ...     async with session.start_transaction():
    ...         user = User(name="Alice")
    ...         await user.save(session=session)
    ...         order = Order(user_id=user.id)
    ...         await order.save(session=session)
    ...         # Transaction commits on exit
    ...         # Or rolls back on exception
"""

from __future__ import annotations

from typing import Any, Optional, TYPE_CHECKING

if TYPE_CHECKING:
    from .document import Document


class TransactionNotSupportedError(NotImplementedError):
    """Raised when transaction operations are attempted without Rust support."""

    def __init__(self, message: str = None):
        default_msg = (
            "Transaction support requires Rust backend implementation. "
            "This feature is planned for a future release."
        )
        super().__init__(message or default_msg)


class Session:
    """
    MongoDB client session for transactions.

    Sessions provide causal consistency and are required for multi-document
    transactions in MongoDB.

    Example (planned API):
        >>> async with start_session() as session:
        ...     # Use session for operations
        ...     await user.save(session=session)
    """

    def __init__(self):
        raise TransactionNotSupportedError()

    async def __aenter__(self) -> "Session":
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        pass

    def start_transaction(
        self,
        read_concern: Optional[dict] = None,
        write_concern: Optional[dict] = None,
        read_preference: Optional[str] = None,
        max_commit_time_ms: Optional[int] = None,
    ) -> "Transaction":
        """
        Start a transaction in this session.

        Args:
            read_concern: Read concern level
            write_concern: Write concern options
            read_preference: Read preference mode
            max_commit_time_ms: Maximum time to wait for commit

        Returns:
            Transaction context manager

        Example (planned API):
            >>> async with session.start_transaction():
            ...     await doc1.save(session=session)
            ...     await doc2.save(session=session)
        """
        raise TransactionNotSupportedError()

    async def commit_transaction(self) -> None:
        """Commit the current transaction."""
        raise TransactionNotSupportedError()

    async def abort_transaction(self) -> None:
        """Abort the current transaction."""
        raise TransactionNotSupportedError()


class Transaction:
    """
    Transaction context manager.

    Transactions provide ACID guarantees for multi-document operations.

    Example (planned API):
        >>> async with session.start_transaction():
        ...     await user.save(session=session)
        ...     await order.save(session=session)
        ...     # Commits on successful exit
        ...     # Rolls back on exception
    """

    def __init__(self, session: Session, **options: Any):
        raise TransactionNotSupportedError()

    async def __aenter__(self) -> "Transaction":
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        pass


async def start_session() -> Session:
    """
    Start a new client session.

    Sessions are required for transactions and provide causal consistency.

    Returns:
        Session context manager

    Raises:
        TransactionNotSupportedError: Always (not yet implemented)

    Example (planned API):
        >>> async with start_session() as session:
        ...     async with session.start_transaction():
        ...         await doc.save(session=session)
    """
    raise TransactionNotSupportedError(
        "Transaction support requires Rust backend implementation. "
        "For now, use individual operations which are atomic per-document. "
        "Multi-document ACID transactions will be available in a future release."
    )


# Document the required changes for implementation
IMPLEMENTATION_NOTES = """
To implement transaction support, the following Rust changes are needed:

1. Add session management to mongodb.rs:
   - start_session() -> PyObject (session wrapper)
   - Session.start_transaction()
   - Session.commit_transaction()
   - Session.abort_transaction()

2. Update all CRUD operations to accept optional session parameter:
   - insert_one(collection, doc, session=None)
   - update_one(collection, filter, update, session=None)
   - delete_one(collection, filter, session=None)
   - find(collection, filter, session=None)
   - etc.

3. Python layer changes:
   - Add session parameter to Document.save(), delete(), etc.
   - Add session parameter to QueryBuilder.to_list(), first(), etc.
   - Implement Session and Transaction classes properly

4. Testing:
   - Requires MongoDB replica set (transactions don't work on standalone)
   - Test commit/rollback scenarios
   - Test concurrent transactions
"""


__all__ = [
    "Session",
    "Transaction",
    "start_session",
    "TransactionNotSupportedError",
]
