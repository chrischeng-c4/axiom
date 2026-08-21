# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functions"
# dimension = "behavior"
# case = "function_test_case__test_c_char_parm_ucb7f3a8"
# subject = "cpython.test_functions.FunctionTestCase.test_c_char_parm"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_functions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_functions
_suite = unittest.defaultTestLoader.loadTestsFromName("FunctionTestCase.test_c_char_parm", test_functions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FunctionTestCase.test_c_char_parm did not pass"
print("FunctionTestCase::test_c_char_parm: ok")
