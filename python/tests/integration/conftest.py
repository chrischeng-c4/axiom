"""
Integration test fixtures for data-bridge.

Provides MongoDB connection setup for integration tests.
"""

import pytest
from cclab import init, close, is_connected


# MongoDB connection URI for integration tests
# Try localhost without auth first
MONGODB_URI = "mongodb://localhost:27017/data-bridge-test"


@pytest.fixture(scope="function", autouse=True)
async def mongodb_connection():
    """
    Initialize MongoDB connection for integration tests (function-scoped, auto-use).

    This fixture automatically runs before each test function, ensuring
    a fresh database connection and clean collections for each test.
    """
    # Close existing connection if any
    if is_connected():
        await close()

    # Initialize connection for this test
    await init(MONGODB_URI)

    # Clean up test collections before test
    from cclab.nebula import _engine
    try:
        # Drop test collections to ensure clean state
        await _engine._rust.Document.drop_collection("test_conversion")
    except:
        pass  # Collection might not exist yet

    yield

    # Clean up after test
    if is_connected():
        await close()
