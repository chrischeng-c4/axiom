# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "loadtestsfromname_missing_name_yields_failing_test"
# subject = "unittest.TestLoader.loadTestsFromName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestLoader.loadTestsFromName: loadTestsFromName for a non-existent top-level name does not raise; it returns a one-test suite that records an error when run"""
import unittest

loader = unittest.TestLoader()
# CPython 3.12 defers the failure: building the suite does not raise.
suite = loader.loadTestsFromName("no_such_module_xyz")
assert suite.countTestCases() == 1
result = unittest.TestResult()
suite.run(result)
# The deferred failure surfaces as an error when the suite runs.
assert len(result.errors) == 1
assert result.failures == []
print("loadtestsfromname_missing_name_yields_failing_test OK")
