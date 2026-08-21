# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int"
# dimension = "behavior"
# case = "int_str_digit_limits_tests__test_denial_of_service_prevented_int_to_str_ucb10254"
# subject = "cpython.test_int.IntStrDigitLimitsTests.test_denial_of_service_prevented_int_to_str"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_int
_suite = unittest.defaultTestLoader.loadTestsFromName("IntStrDigitLimitsTests.test_denial_of_service_prevented_int_to_str", test_int)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IntStrDigitLimitsTests.test_denial_of_service_prevented_int_to_str did not pass"
print("IntStrDigitLimitsTests::test_denial_of_service_prevented_int_to_str: ok")
