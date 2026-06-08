# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "async_context_manager_test_case__test_contextmanager_non_normalised_ucf3bafc"
# subject = "cpython.test_contextlib_async.AsyncContextManagerTestCase.test_contextmanager_non_normalised"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_contextlib_async
_suite = unittest.defaultTestLoader.loadTestsFromName("AsyncContextManagerTestCase.test_contextmanager_non_normalised", test_contextlib_async)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AsyncContextManagerTestCase.test_contextmanager_non_normalised did not pass"
print("AsyncContextManagerTestCase::test_contextmanager_non_normalised: ok")
