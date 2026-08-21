# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regrtest"
# dimension = "behavior"
# case = "parse_args_test_case__test_option_with_empty_string_value"
# subject = "cpython.test_regrtest.ParseArgsTestCase.test_option_with_empty_string_value"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_regrtest.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_regrtest
_suite = unittest.defaultTestLoader.loadTestsFromName("ParseArgsTestCase.test_option_with_empty_string_value", test_regrtest)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ParseArgsTestCase.test_option_with_empty_string_value did not pass"
print("ParseArgsTestCase::test_option_with_empty_string_value: ok")
