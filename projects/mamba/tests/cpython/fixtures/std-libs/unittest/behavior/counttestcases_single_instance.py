# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "counttestcases_single_instance"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: a single TestCase instance reports countTestCases() == 1"""
import unittest


class Sample(unittest.TestCase):
    def test_a(self):
        pass

    def runTest(self):
        pass


assert Sample("test_a").countTestCases() == 1
print("counttestcases_single_instance OK")
