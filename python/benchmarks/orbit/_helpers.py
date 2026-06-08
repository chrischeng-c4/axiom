"""Shared helpers for orbit benchmark suite."""

from __future__ import annotations

import asyncio
from typing import Any


def has_uvloop() -> bool:
    """Check if uvloop is available."""
    try:
        import uvloop  # noqa: F401

        return True
    except ImportError:
        return False


BACKENDS = ["orbit", "uvloop", "asyncio"] if has_uvloop() else ["orbit", "asyncio"]


def create_loop(backend: str) -> Any:
    """Create an event loop for the given backend.

    Args:
        backend: One of "orbit", "uvloop", or "asyncio".

    Returns:
        An event loop instance.
    """
    if backend == "orbit":
        from cclab.orbit import PyLoop

        return PyLoop()
    elif backend == "uvloop":
        import uvloop

        return uvloop.new_event_loop()
    else:
        return asyncio.new_event_loop()


def close_loop(loop: Any, backend: str) -> None:
    """Close a non-orbit event loop.

    Orbit's PyLoop does not require explicit close, but asyncio and
    uvloop loops do.
    """
    if backend != "orbit":
        loop.close()
