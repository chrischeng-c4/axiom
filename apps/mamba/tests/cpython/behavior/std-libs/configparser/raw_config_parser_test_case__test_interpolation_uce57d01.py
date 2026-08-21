# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "raw_config_parser_test_case__test_interpolation_uce57d01"
# subject = "cpython.test_configparser.RawConfigParserTestCase.test_interpolation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_configparser
_suite = unittest.defaultTestLoader.loadTestsFromName("RawConfigParserTestCase.test_interpolation", test_configparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RawConfigParserTestCase.test_interpolation did not pass"
print("RawConfigParserTestCase::test_interpolation: ok")
