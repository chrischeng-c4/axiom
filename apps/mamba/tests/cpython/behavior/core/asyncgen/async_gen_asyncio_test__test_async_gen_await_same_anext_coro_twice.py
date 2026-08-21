# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "asyncgen"
# dimension = "behavior"
# case = "async_gen_asyncio_test__test_async_gen_await_same_anext_coro_twice"
# subject = "cpython.test_asyncgen.AsyncGenAsyncioTest.test_async_gen_await_same_anext_coro_twice"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncgen.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_asyncgen
_suite = unittest.defaultTestLoader.loadTestsFromName("AsyncGenAsyncioTest.test_async_gen_await_same_anext_coro_twice", test_asyncgen)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AsyncGenAsyncioTest.test_async_gen_await_same_anext_coro_twice did not pass"
print("AsyncGenAsyncioTest::test_async_gen_await_same_anext_coro_twice: ok")
