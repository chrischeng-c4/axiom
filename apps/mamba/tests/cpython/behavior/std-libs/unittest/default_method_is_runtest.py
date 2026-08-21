# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "default_method_is_runtest"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: with no method name given, the default-selected method is runTest, so id() ends with '.runTest'"""
import unittest


class Sample(unittest.TestCase):
    def test_a(self):
        pass

    def runTest(self):
        pass


assert Sample().id().endswith(".Sample.runTest")
print("default_method_is_runtest OK")
