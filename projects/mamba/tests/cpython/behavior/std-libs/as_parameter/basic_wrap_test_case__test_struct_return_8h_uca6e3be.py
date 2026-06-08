# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "as_parameter"
# dimension = "behavior"
# case = "basic_wrap_test_case__test_struct_return_8h_uca6e3be"
# subject = "cpython.test_as_parameter.BasicWrapTestCase.test_struct_return_8H"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_as_parameter.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_as_parameter
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicWrapTestCase.test_struct_return_8H", test_as_parameter)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicWrapTestCase.test_struct_return_8H did not pass"
print("BasicWrapTestCase::test_struct_return_8H: ok")
