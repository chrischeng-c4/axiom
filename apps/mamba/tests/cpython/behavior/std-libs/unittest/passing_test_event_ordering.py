# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "passing_test_event_ordering"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: a passing test run via defaultTestResult() brackets the run with startTestRun/stopTestRun and reports addSuccess: startTestRun, startTest, body, addSuccess, stopTest, stopTestRun"""
import unittest

events = []


class RecordingResult(unittest.TestResult):
    def startTestRun(self):
        events.append("startTestRun")
        super().startTestRun()

    def startTest(self, test):
        events.append("startTest")
        super().startTest(test)

    def addSuccess(self, test):
        events.append("addSuccess")
        super().addSuccess(test)

    def stopTest(self, test):
        events.append("stopTest")
        super().stopTest(test)

    def stopTestRun(self):
        events.append("stopTestRun")
        super().stopTestRun()


default_result = RecordingResult()


class Passing(unittest.TestCase):
    def test(self):
        events.append("body")

    def defaultTestResult(self):
        return default_result


assert Passing("test").run() is default_result
assert events == [
    "startTestRun",
    "startTest",
    "body",
    "addSuccess",
    "stopTest",
    "stopTestRun",
]
print("passing_test_event_ordering OK")
