# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "converters_test_case__test_converters_uc370030"
# subject = "cpython.test_configparser.ConvertersTestCase.test_converters"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_configparser
_suite = unittest.defaultTestLoader.loadTestsFromName("ConvertersTestCase.test_converters", test_configparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConvertersTestCase.test_converters did not pass"
print("ConvertersTestCase::test_converters: ok")
