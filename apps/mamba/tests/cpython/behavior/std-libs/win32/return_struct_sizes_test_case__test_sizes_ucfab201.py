# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "win32"
# dimension = "behavior"
# case = "return_struct_sizes_test_case__test_sizes_ucfab201"
# subject = "cpython.test_win32.ReturnStructSizesTestCase.test_sizes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_win32.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_win32
_suite = unittest.defaultTestLoader.loadTestsFromName("ReturnStructSizesTestCase.test_sizes", test_win32)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReturnStructSizesTestCase.test_sizes did not pass"
print("ReturnStructSizesTestCase::test_sizes: ok")
