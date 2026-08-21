# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int"
# dimension = "behavior"
# case = "int_str_digit_limits_tests__test_int_max_str_digits_is_per_interpreter_uc4f96f8"
# subject = "cpython.test_int.IntStrDigitLimitsTests.test_int_max_str_digits_is_per_interpreter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_int
_suite = unittest.defaultTestLoader.loadTestsFromName("IntStrDigitLimitsTests.test_int_max_str_digits_is_per_interpreter", test_int)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IntStrDigitLimitsTests.test_int_max_str_digits_is_per_interpreter did not pass"
print("IntStrDigitLimitsTests::test_int_max_str_digits_is_per_interpreter: ok")
