# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "asyncgen"
# dimension = "behavior"
# case = "async_gen_asyncio_test__test_async_gen_asyncio_athrow_stopiteration"
# subject = "cpython.test_asyncgen.AsyncGenAsyncioTest.test_async_gen_asyncio_athrow_stopiteration"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncgen.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_asyncgen
_suite = unittest.defaultTestLoader.loadTestsFromName("AsyncGenAsyncioTest.test_async_gen_asyncio_athrow_stopiteration", test_asyncgen)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AsyncGenAsyncioTest.test_async_gen_asyncio_athrow_stopiteration did not pass"
print("AsyncGenAsyncioTest::test_async_gen_asyncio_athrow_stopiteration: ok")
