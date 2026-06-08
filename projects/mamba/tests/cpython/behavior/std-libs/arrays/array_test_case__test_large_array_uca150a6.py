# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_large_array_uca150a6"
# subject = "cpython.test_arrays.ArrayTestCase.test_large_array"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_arrays
_suite = unittest.defaultTestLoader.loadTestsFromName("ArrayTestCase.test_large_array", test_arrays)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ArrayTestCase.test_large_array did not pass"
print("ArrayTestCase::test_large_array: ok")
