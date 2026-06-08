# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random_things"
# dimension = "behavior"
# case = "call_function__test_case__test_uc2d3415"
# subject = "cpython.test_random_things.call_function_TestCase.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_random_things.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_random_things
_suite = unittest.defaultTestLoader.loadTestsFromName("call_function_TestCase.test", test_random_things)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython call_function_TestCase.test did not pass"
print("call_function_TestCase::test: ok")
