# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_c_wchar_uc46cfe2"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_c_wchar"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_parameters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_parameters
_suite = unittest.defaultTestLoader.loadTestsFromName("SimpleTypesTestCase.test_c_wchar", test_parameters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimpleTypesTestCase.test_c_wchar did not pass"
print("SimpleTypesTestCase::test_c_wchar: ok")
