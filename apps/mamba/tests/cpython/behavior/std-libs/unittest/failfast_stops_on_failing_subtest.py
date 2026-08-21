# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "failfast_stops_on_failing_subtest"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: under failfast, a failing subTest halts the suite immediately: test_a's subtests run, test_b fails on its second subTest, test_c never runs, and exactly one failure is recorded"""
import unittest

events = []


class FailFast(unittest.TestCase):
    def test_a(self):
        with self.subTest():
            events.append("a1")
        events.append("a2")

    def test_b(self):
        with self.subTest():
            events.append("b1")
        with self.subTest():
            self.fail("failure")
        events.append("b2")

    def test_c(self):
        events.append("c")


result = unittest.TestResult()
result.failfast = True
suite = unittest.TestLoader().loadTestsFromTestCase(FailFast)
suite.run(result)

assert events == ["a1", "a2", "b1"]
assert result.failfast is True
assert len(result.failures) == 1
print("failfast_stops_on_failing_subtest OK")
