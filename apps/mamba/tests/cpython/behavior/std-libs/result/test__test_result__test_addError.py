# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "result"
# dimension = "behavior"
# case = "test__test_result__test_addError"
# subject = "cpython.test_result.Test_TestResult.test_addError"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_result.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_result
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestResult.test_addError", test_result)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestResult.test_addError did not pass"
print("Test_TestResult::test_addError: ok")
