# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "testcase_introspection_surface"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: a trivial TestCase exposes the documented introspection surface: id() is a str, countTestCases() == 1, defaultTestResult() is a TestResult, shortDescription() is None"""
import unittest


class Probe(unittest.TestCase):
    def runTest(self):
        pass


tc = Probe()
assert isinstance(tc.id(), str)
assert tc.countTestCases() == 1
assert isinstance(tc.defaultTestResult(), unittest.TestResult)
assert tc.shortDescription() is None
print("testcase_introspection_surface OK")
