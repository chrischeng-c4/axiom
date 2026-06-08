"""Behavior contract for third-party pytest_asyncio package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import pytest_asyncio  # type: ignore[import]
import asyncio
import inspect

# Rule 1: fixture is callable and can wrap an async function
@pytest_asyncio.fixture
async def _sample_fixture():
    yield 42

# The decorated fixture should be callable
assert callable(_sample_fixture), "fixture-decorated func is callable"

# Rule 2: is_async_test is callable (takes pytest Item, not a plain function)
# It is designed for use in pytest hooks; we only verify callability here.
assert callable(pytest_asyncio.is_async_test), "is_async_test callable"

# Rule 3: fixture supports scope parameter
_with_scope = pytest_asyncio.fixture(scope="module")
assert callable(_with_scope), "fixture(scope) returns callable"

# Rule 4: fixture can be used as a plain decorator (no args)
@pytest_asyncio.fixture
async def _no_args_fixture():
    return "hello"

assert callable(_no_args_fixture), "no-args fixture callable"

# Rule 5: Module attributes are identity-stable
_fix_ref = pytest_asyncio.fixture
_iat_ref = pytest_asyncio.is_async_test
for _ in range(5):
    assert pytest_asyncio.fixture is _fix_ref, "fixture stable"
    assert pytest_asyncio.is_async_test is _iat_ref, "is_async_test stable"

print("behavior OK")
