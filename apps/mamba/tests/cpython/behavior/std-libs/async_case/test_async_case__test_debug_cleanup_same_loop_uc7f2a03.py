# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "async_case"
# dimension = "behavior"
# case = "test_async_case__test_debug_cleanup_same_loop_uc7f2a03"
# subject = "cpython.test_async_case.TestAsyncCase.test_debug_cleanup_same_loop"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_async_case.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_async_case
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAsyncCase.test_debug_cleanup_same_loop", test_async_case)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAsyncCase.test_debug_cleanup_same_loop did not pass"
print("TestAsyncCase::test_debug_cleanup_same_loop: ok")
