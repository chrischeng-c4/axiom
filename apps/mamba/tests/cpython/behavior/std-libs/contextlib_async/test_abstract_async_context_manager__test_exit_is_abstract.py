# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "test_abstract_async_context_manager__test_exit_is_abstract"
# subject = "cpython.test_contextlib_async.TestAbstractAsyncContextManager.test_exit_is_abstract"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib_async.py::TestAbstractAsyncContextManager::test_exit_is_abstract
"""Auto-ported test: TestAbstractAsyncContextManager::test_exit_is_abstract (CPython 3.12 oracle)."""


import asyncio
from contextlib import asynccontextmanager, AbstractAsyncContextManager, AsyncExitStack, nullcontext, aclosing, contextmanager
import functools
from test import support
import unittest
import traceback
from test.test_contextlib import TestBaseExitStack


support.requires_working_socket(module=True)

def _async_test(func):
    """Decorator to turn an async function into a test case."""

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        coro = func(*args, **kwargs)
        asyncio.run(coro)
    return wrapper

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
class MissingAexit(AbstractAsyncContextManager):
    pass
try:
    MissingAexit()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestAbstractAsyncContextManager::test_exit_is_abstract: ok")
