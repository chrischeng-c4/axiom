# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int"
# dimension = "behavior"
# case = "int_str_digit_limits_tests__test_max_str_digits_edge_cases_ucd97de2"
# subject = "cpython.test_int.IntStrDigitLimitsTests.test_max_str_digits_edge_cases"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_int
_suite = unittest.defaultTestLoader.loadTestsFromName("IntStrDigitLimitsTests.test_max_str_digits_edge_cases", test_int)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IntStrDigitLimitsTests.test_max_str_digits_edge_cases did not pass"
print("IntStrDigitLimitsTests::test_max_str_digits_edge_cases: ok")
