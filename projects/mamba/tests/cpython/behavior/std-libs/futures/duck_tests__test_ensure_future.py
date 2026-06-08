# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "futures"
# dimension = "behavior"
# case = "duck_tests__test_ensure_future"
# subject = "cpython.test_futures.DuckTests.test_ensure_future"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_futures.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_futures
_suite = unittest.defaultTestLoader.loadTestsFromName("DuckTests.test_ensure_future", test_futures)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DuckTests.test_ensure_future did not pass"
print("DuckTests::test_ensure_future: ok")
