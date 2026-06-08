# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "config_parser_test_case_extended_interpolation__test_case_sensitivity_basic_ucb71270"
# subject = "cpython.test_configparser.ConfigParserTestCaseExtendedInterpolation.test_case_sensitivity_basic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_configparser
_suite = unittest.defaultTestLoader.loadTestsFromName("ConfigParserTestCaseExtendedInterpolation.test_case_sensitivity_basic", test_configparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConfigParserTestCaseExtendedInterpolation.test_case_sensitivity_basic did not pass"
print("ConfigParserTestCaseExtendedInterpolation::test_case_sensitivity_basic: ok")
