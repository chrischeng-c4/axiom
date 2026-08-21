# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "test_abstract_async_context_manager__test_enter_uc9bcd55"
# subject = "cpython.test_contextlib_async.TestAbstractAsyncContextManager.test_enter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_contextlib_async
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAbstractAsyncContextManager.test_enter", test_contextlib_async)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAbstractAsyncContextManager.test_enter did not pass"
print("TestAbstractAsyncContextManager::test_enter: ok")
