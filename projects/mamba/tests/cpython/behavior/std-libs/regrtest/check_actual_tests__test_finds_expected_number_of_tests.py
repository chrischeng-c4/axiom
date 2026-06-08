# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regrtest"
# dimension = "behavior"
# case = "check_actual_tests__test_finds_expected_number_of_tests"
# subject = "cpython.test_regrtest.CheckActualTests.test_finds_expected_number_of_tests"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_regrtest.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_regrtest
_suite = unittest.defaultTestLoader.loadTestsFromName("CheckActualTests.test_finds_expected_number_of_tests", test_regrtest)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CheckActualTests.test_finds_expected_number_of_tests did not pass"
print("CheckActualTests::test_finds_expected_number_of_tests: ok")
