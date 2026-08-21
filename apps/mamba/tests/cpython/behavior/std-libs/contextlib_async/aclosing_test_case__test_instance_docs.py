# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "aclosing_test_case__test_instance_docs"
# subject = "cpython.test_contextlib_async.AclosingTestCase.test_instance_docs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib_async.py::AclosingTestCase::test_instance_docs
"""Auto-ported test: AclosingTestCase::test_instance_docs (CPython 3.12 oracle)."""


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
cm_docstring = aclosing.__doc__
obj = aclosing(None)

assert obj.__doc__ == cm_docstring
print("AclosingTestCase::test_instance_docs: ok")
