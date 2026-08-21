# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functiontestcase"
# dimension = "behavior"
# case = "test__function_test_case__test_id_ucfc463c"
# subject = "cpython.test_functiontestcase.Test_FunctionTestCase.test_id"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_functiontestcase.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_functiontestcase
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_FunctionTestCase.test_id", test_functiontestcase)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_FunctionTestCase.test_id did not pass"
print("Test_FunctionTestCase::test_id: ok")
