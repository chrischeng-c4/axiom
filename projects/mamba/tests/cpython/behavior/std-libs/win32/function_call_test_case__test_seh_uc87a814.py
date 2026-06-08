# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "win32"
# dimension = "behavior"
# case = "function_call_test_case__test_seh_uc87a814"
# subject = "cpython.test_win32.FunctionCallTestCase.test_SEH"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_win32.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_win32
_suite = unittest.defaultTestLoader.loadTestsFromName("FunctionCallTestCase.test_SEH", test_win32)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FunctionCallTestCase.test_SEH did not pass"
print("FunctionCallTestCase::test_SEH: ok")
