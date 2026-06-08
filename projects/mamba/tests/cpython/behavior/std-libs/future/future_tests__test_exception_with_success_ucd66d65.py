# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future"
# dimension = "behavior"
# case = "future_tests__test_exception_with_success_ucd66d65"
# subject = "cpython.test_future.FutureTests.test_exception_with_success"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_concurrent_futures/test_future.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_concurrent_futures import test_future
_suite = unittest.defaultTestLoader.loadTestsFromName("FutureTests.test_exception_with_success", test_future)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FutureTests.test_exception_with_success did not pass"
print("FutureTests::test_exception_with_success: ok")
