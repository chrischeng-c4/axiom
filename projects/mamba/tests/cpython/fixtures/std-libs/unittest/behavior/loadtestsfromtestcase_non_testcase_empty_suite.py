# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "loadtestsfromtestcase_non_testcase_empty_suite"
# subject = "unittest.TestLoader.loadTestsFromTestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestLoader.loadTestsFromTestCase: loadTestsFromTestCase on a class that is not a TestCase subclass does not raise in 3.12; it returns an empty TestSuite (no test_* methods to collect)"""
import unittest


class NotATestCase:
    pass


loader = unittest.TestLoader()
suite = loader.loadTestsFromTestCase(NotATestCase)
assert isinstance(suite, unittest.TestSuite)
assert suite.countTestCases() == 0
print("loadtestsfromtestcase_non_testcase_empty_suite OK")
