# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "run_returns_given_result"
# subject = "unittest.TestCase.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.run: TestCase.run(result) returns the same TestResult object it was handed"""
import unittest


class Passing(unittest.TestCase):
    def test(self):
        pass


given = unittest.TestResult()
returned = Passing("test").run(given)
assert returned is given
print("run_returns_given_result OK")
