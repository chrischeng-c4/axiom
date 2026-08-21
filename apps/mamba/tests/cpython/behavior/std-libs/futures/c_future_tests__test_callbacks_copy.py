# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "futures"
# dimension = "behavior"
# case = "c_future_tests__test_callbacks_copy"
# subject = "cpython.test_futures.CFutureTests.test_callbacks_copy"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_futures.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_futures
_suite = unittest.defaultTestLoader.loadTestsFromName("CFutureTests.test_callbacks_copy", test_futures)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CFutureTests.test_callbacks_copy did not pass"
print("CFutureTests::test_callbacks_copy: ok")
