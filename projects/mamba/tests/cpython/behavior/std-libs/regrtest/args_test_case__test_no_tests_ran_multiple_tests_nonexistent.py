# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regrtest"
# dimension = "behavior"
# case = "args_test_case__test_no_tests_ran_multiple_tests_nonexistent"
# subject = "cpython.test_regrtest.ArgsTestCase.test_no_tests_ran_multiple_tests_nonexistent"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_regrtest.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_regrtest
_suite = unittest.defaultTestLoader.loadTestsFromName("ArgsTestCase.test_no_tests_ran_multiple_tests_nonexistent", test_regrtest)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ArgsTestCase.test_no_tests_ran_multiple_tests_nonexistent did not pass"
print("ArgsTestCase::test_no_tests_ran_multiple_tests_nonexistent: ok")
