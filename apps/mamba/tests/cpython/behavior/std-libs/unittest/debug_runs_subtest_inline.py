# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "debug_runs_subtest_inline"
# subject = "unittest.TestCase.debug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.debug: debug() runs the test in-line and a passing subTest does not interrupt control flow"""
import unittest

events = []


class Debuggable(unittest.TestCase):
    def test_a(self):
        events.append("test case")
        with self.subTest():
            events.append("subtest 1")


Debuggable("test_a").debug()
assert events == ["test case", "subtest 1"]
print("debug_runs_subtest_inline OK")
