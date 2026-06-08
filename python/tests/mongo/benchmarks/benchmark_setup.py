"""
Automatic setup for MongoDB benchmarks.

This module initializes MongoDB connections when imported, allowing benchmarks
to be auto-discovered and run by dbtest without explicit setup.

Import this module at the top of benchmark files to enable auto-initialization.
"""

import asyncio
import os
from typing import Optional

_setup_complete = False
_motor_client = None


def get_mongodb_uri() -> str:
    """Get MongoDB URI from environment or use default."""
    return os.environ.get(
        "MONGODB_URI",
        "mongodb://localhost:27017/data-bridge-benchmark"
    )


async def _async_setup():
    """Initialize MongoDB connections asynchronously."""
    global _setup_complete, _motor_client

    if _setup_complete:
        return

    from cclab import init, close, is_connected
    from beanie import init_beanie
    from motor.motor_asyncio import AsyncIOMotorClient
    from .models import BEANIE_MODELS

    mongodb_uri = get_mongodb_uri()
    
    # Extract database name from URI if possible
    # e.g., mongodb://localhost:27017/my_db -> my_db
    import urllib.parse
    parsed = urllib.parse.urlparse(mongodb_uri)
    db_name = parsed.path.strip("/") or "data-bridge-benchmark"

    # Initialize data-bridge
    if is_connected():
        await close()
    await init(mongodb_uri)

    # Initialize Beanie
    _motor_client = AsyncIOMotorClient(mongodb_uri)
    await init_beanie(
        database=_motor_client[db_name],
        document_models=BEANIE_MODELS,
    )

    _setup_complete = True


def ensure_setup():
    """
    Ensure MongoDB is initialized (synchronous wrapper).

    This is called automatically when benchmarks are imported.
    When running in an async context, this is a no-op since the CLI
    will call ensure_mongodb_initialized() before loading benchmarks.
    """
    global _setup_complete

    if _setup_complete:
        return

    # Check if we're in an async context
    try:
        asyncio.get_running_loop()
        # If we're in an async context, skip sync setup.
        # The CLI handles async initialization before loading benchmarks.
        return
    except RuntimeError:
        # No event loop running, create one and run setup
        asyncio.run(_async_setup())


async def async_ensure_setup():
    """
    Ensure MongoDB is initialized (async version).

    Call this from benchmark functions if needed.
    """
    await _async_setup()


async def cleanup():
    """Clean up MongoDB connections."""
    global _motor_client, _setup_complete

    if not _setup_complete:
        return

    from cclab import close

    await close()
    if _motor_client:
        _motor_client.close()
        _motor_client = None

    _setup_complete = False


# Auto-initialize when this module is imported (for dbtest auto-discovery)
# This ensures MongoDB is ready when benchmarks are loaded
try:
    ensure_setup()
except Exception as e:
    # If setup fails during import, benchmarks will fail with clear error
    print(f"Warning: Benchmark setup failed: {e}")
    print("MongoDB connection will be attempted when benchmarks run")
