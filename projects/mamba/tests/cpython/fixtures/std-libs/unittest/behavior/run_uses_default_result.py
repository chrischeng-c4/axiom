# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "run_uses_default_result"
# subject = "unittest.TestCase.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.run: TestCase.run() with no argument falls back to the object's defaultTestResult()"""
import unittest

default_result = unittest.TestResult()


class WithDefault(unittest.TestCase):
    def test(self):
        pass

    def defaultTestResult(self):
        return default_result


used = WithDefault("test").run()
assert used is default_result
print("run_uses_default_result OK")
