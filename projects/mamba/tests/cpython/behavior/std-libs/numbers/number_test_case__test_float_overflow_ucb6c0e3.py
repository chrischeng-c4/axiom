# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "number_test_case__test_float_overflow_ucb6c0e3"
# subject = "cpython.test_numbers.NumberTestCase.test_float_overflow"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_numbers
_suite = unittest.defaultTestLoader.loadTestsFromName("NumberTestCase.test_float_overflow", test_numbers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NumberTestCase.test_float_overflow did not pass"
print("NumberTestCase::test_float_overflow: ok")
