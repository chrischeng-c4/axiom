# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "failing_test_reports_addfailure"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: a failing test reports addFailure (not addError) between startTest/stopTest, and a custom failureException is honored for self.fail()"""
import unittest

events = []


class RecordingResult(unittest.TestResult):
    def startTest(self, test):
        events.append("startTest")
        super().startTest(test)

    def addFailure(self, test, err):
        events.append("addFailure")
        super().addFailure(test, err)

    def stopTest(self, test):
        events.append("stopTest")
        super().stopTest(test)


class Failing(unittest.TestCase):
    failureException = RuntimeError

    def test(self):
        self.fail("boom")


assert Failing("test").failureException is RuntimeError
Failing("test").run(RecordingResult())
assert events == ["startTest", "addFailure", "stopTest"]
print("failing_test_reports_addfailure OK")
