# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "exception_context_test_case__test_get_basic_interpolation_uc14b477"
# subject = "cpython.test_configparser.ExceptionContextTestCase.test_get_basic_interpolation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_configparser
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionContextTestCase.test_get_basic_interpolation", test_configparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionContextTestCase.test_get_basic_interpolation did not pass"
print("ExceptionContextTestCase::test_get_basic_interpolation: ok")
