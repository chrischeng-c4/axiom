# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "large_values_test_case__test_bytearray_uc6c126b"
# subject = "cpython.test_marshal.LargeValuesTestCase.test_bytearray"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_marshal
_suite = unittest.defaultTestLoader.loadTestsFromName("LargeValuesTestCase.test_bytearray", test_marshal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LargeValuesTestCase.test_bytearray did not pass"
print("LargeValuesTestCase::test_bytearray: ok")
