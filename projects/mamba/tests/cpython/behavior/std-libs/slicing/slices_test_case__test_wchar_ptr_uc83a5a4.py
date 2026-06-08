# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "slicing"
# dimension = "behavior"
# case = "slices_test_case__test_wchar_ptr_uc83a5a4"
# subject = "cpython.test_slicing.SlicesTestCase.test_wchar_ptr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_slicing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_slicing
_suite = unittest.defaultTestLoader.loadTestsFromName("SlicesTestCase.test_wchar_ptr", test_slicing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SlicesTestCase.test_wchar_ptr did not pass"
print("SlicesTestCase::test_wchar_ptr: ok")
