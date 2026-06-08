# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "test_abstract_async_context_manager__test_structural_subclassing"
# subject = "cpython.test_contextlib_async.TestAbstractAsyncContextManager.test_structural_subclassing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib_async.py::TestAbstractAsyncContextManager::test_structural_subclassing
"""Auto-ported test: TestAbstractAsyncContextManager::test_structural_subclassing (CPython 3.12 oracle)."""


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
class ManagerFromScratch:

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_value, traceback):
        return None

assert issubclass(ManagerFromScratch, AbstractAsyncContextManager)

class DefaultEnter(AbstractAsyncContextManager):

    async def __aexit__(self, *args):
        await super().__aexit__(*args)

assert issubclass(DefaultEnter, AbstractAsyncContextManager)

class NoneAenter(ManagerFromScratch):
    __aenter__ = None

assert not issubclass(NoneAenter, AbstractAsyncContextManager)

class NoneAexit(ManagerFromScratch):
    __aexit__ = None

assert not issubclass(NoneAexit, AbstractAsyncContextManager)
print("TestAbstractAsyncContextManager::test_structural_subclassing: ok")
