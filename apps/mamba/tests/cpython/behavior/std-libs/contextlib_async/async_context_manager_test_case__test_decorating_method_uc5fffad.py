# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "async_context_manager_test_case__test_decorating_method_uc5fffad"
# subject = "cpython.test_contextlib_async.AsyncContextManagerTestCase.test_decorating_method"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_contextlib_async
_suite = unittest.defaultTestLoader.loadTestsFromName("AsyncContextManagerTestCase.test_decorating_method", test_contextlib_async)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AsyncContextManagerTestCase.test_decorating_method did not pass"
print("AsyncContextManagerTestCase::test_decorating_method: ok")
