# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "async_context_manager_test_case__test_contextmanager_attribs"
# subject = "cpython.test_contextlib_async.AsyncContextManagerTestCase.test_contextmanager_attribs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib_async.py::AsyncContextManagerTestCase::test_contextmanager_attribs
"""Auto-ported test: AsyncContextManagerTestCase::test_contextmanager_attribs (CPython 3.12 oracle)."""


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
def _create_contextmanager_attribs():

    def attribs(**kw):

        def decorate(func):
            for k, v in kw.items():
                setattr(func, k, v)
            return func
        return decorate

    @asynccontextmanager
    @attribs(foo='bar')
    async def baz(spam):
        """Whee!"""
        yield
    return baz
baz = _create_contextmanager_attribs()

assert baz.__name__ == 'baz'

assert baz.foo == 'bar'
print("AsyncContextManagerTestCase::test_contextmanager_attribs: ok")
