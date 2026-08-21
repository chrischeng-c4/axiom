# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functiontestcase"
# dimension = "behavior"
# case = "test__function_test_case__test_shortdescription__no_docstring_uc5990a9"
# subject = "cpython.test_functiontestcase.Test_FunctionTestCase.test_shortDescription__no_docstring"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_functiontestcase.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_functiontestcase
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_FunctionTestCase.test_shortDescription__no_docstring", test_functiontestcase)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_FunctionTestCase.test_shortDescription__no_docstring did not pass"
print("Test_FunctionTestCase::test_shortDescription__no_docstring: ok")
