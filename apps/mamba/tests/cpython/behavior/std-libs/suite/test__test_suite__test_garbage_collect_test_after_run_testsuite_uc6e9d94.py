# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "suite"
# dimension = "behavior"
# case = "test__test_suite__test_garbage_collect_test_after_run_testsuite_uc6e9d94"
# subject = "cpython.test_suite.Test_TestSuite.test_garbage_collect_test_after_run_TestSuite"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_suite.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_suite
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestSuite.test_garbage_collect_test_after_run_TestSuite", test_suite)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestSuite.test_garbage_collect_test_after_run_TestSuite did not pass"
print("Test_TestSuite::test_garbage_collect_test_after_run_TestSuite: ok")
