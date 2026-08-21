# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "case"
# dimension = "behavior"
# case = "test__test_case__testAssertNoLogsPerLevel"
# subject = "cpython.test_case.Test_TestCase.testAssertNoLogsPerLevel"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_case.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_case
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestCase.testAssertNoLogsPerLevel", test_case)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestCase.testAssertNoLogsPerLevel did not pass"
print("Test_TestCase::testAssertNoLogsPerLevel: ok")
