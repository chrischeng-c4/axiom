# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "setup_teardown_default_noops"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: the default setUp() and tearDown() are safe no-ops callable directly without error"""
import unittest


class Sample(unittest.TestCase):
    def test_a(self):
        pass

    def runTest(self):
        pass


assert Sample().setUp() is None
assert Sample().tearDown() is None
print("setup_teardown_default_noops OK")
