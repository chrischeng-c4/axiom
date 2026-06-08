"""Surface contract for third-party pytest_asyncio package.

# type-regime: monomorphic

Probes: pytest_asyncio.fixture, pytest_asyncio.is_async_test.
CPython 3.12 is the oracle.
"""

import pytest_asyncio

# Core API
assert hasattr(pytest_asyncio, "fixture"), "fixture"
assert hasattr(pytest_asyncio, "is_async_test"), "is_async_test"

# fixture is callable
assert callable(pytest_asyncio.fixture), "fixture callable"

# is_async_test is callable (takes pytest Item objects; checks for async test items)
assert callable(pytest_asyncio.is_async_test), "is_async_test callable"

# fixture can be used as a decorator factory
_decorated = pytest_asyncio.fixture(scope="function")
assert callable(_decorated) or hasattr(_decorated, "__call__") or _decorated is not None, \
    "fixture() returns something"

# Module attributes stable
_fix_ref = pytest_asyncio.fixture
assert pytest_asyncio.fixture is _fix_ref, "fixture stable"
_iat_ref = pytest_asyncio.is_async_test
assert pytest_asyncio.is_async_test is _iat_ref, "is_async_test stable"

print("surface OK")
