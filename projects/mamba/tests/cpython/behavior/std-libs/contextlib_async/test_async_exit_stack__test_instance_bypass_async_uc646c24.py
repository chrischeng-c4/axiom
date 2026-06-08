# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "test_async_exit_stack__test_instance_bypass_async_uc646c24"
# subject = "cpython.test_contextlib_async.TestAsyncExitStack.test_instance_bypass_async"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_contextlib_async
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAsyncExitStack.test_instance_bypass_async", test_contextlib_async)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAsyncExitStack.test_instance_bypass_async did not pass"
print("TestAsyncExitStack::test_instance_bypass_async: ok")
