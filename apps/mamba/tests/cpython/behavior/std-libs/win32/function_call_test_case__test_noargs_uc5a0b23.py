# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "win32"
# dimension = "behavior"
# case = "function_call_test_case__test_noargs_uc5a0b23"
# subject = "cpython.test_win32.FunctionCallTestCase.test_noargs"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_win32.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_win32
_suite = unittest.defaultTestLoader.loadTestsFromName("FunctionCallTestCase.test_noargs", test_win32)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FunctionCallTestCase.test_noargs did not pass"
print("FunctionCallTestCase::test_noargs: ok")
