# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "suite"
# dimension = "behavior"
# case = "test__test_suite__test_init__testsuite_instances_in_tests_uc2f80cd"
# subject = "cpython.test_suite.Test_TestSuite.test_init__TestSuite_instances_in_tests"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_suite.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_suite
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestSuite.test_init__TestSuite_instances_in_tests", test_suite)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestSuite.test_init__TestSuite_instances_in_tests did not pass"
print("Test_TestSuite::test_init__TestSuite_instances_in_tests: ok")
