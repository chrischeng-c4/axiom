# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "passing_run_records_no_failures"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: after a passing run, the result has testsRun == 1, empty failures/errors, and wasSuccessful() is True"""
import unittest


class Passing(unittest.TestCase):
    def test(self):
        pass


given = unittest.TestResult()
Passing("test").run(given)
assert given.testsRun == 1
assert given.failures == []
assert given.errors == []
assert given.wasSuccessful()
print("passing_run_records_no_failures OK")
