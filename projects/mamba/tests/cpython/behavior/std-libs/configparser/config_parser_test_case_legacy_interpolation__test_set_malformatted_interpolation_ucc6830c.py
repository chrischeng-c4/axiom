# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "config_parser_test_case_legacy_interpolation__test_set_malformatted_interpolation_ucc6830c"
# subject = "cpython.test_configparser.ConfigParserTestCaseLegacyInterpolation.test_set_malformatted_interpolation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_configparser
_suite = unittest.defaultTestLoader.loadTestsFromName("ConfigParserTestCaseLegacyInterpolation.test_set_malformatted_interpolation", test_configparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConfigParserTestCaseLegacyInterpolation.test_set_malformatted_interpolation did not pass"
print("ConfigParserTestCaseLegacyInterpolation::test_set_malformatted_interpolation: ok")
