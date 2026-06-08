# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "assertraises_no_exception_raises"
# subject = "unittest.TestCase.assertRaises"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.assertRaises: an assertRaises(ValueError) block whose body raises nothing raises AssertionError on context exit"""
import unittest


class Sample(unittest.TestCase):
    def runTest(self):
        pass


tc = Sample()
_raised = False
try:
    with tc.assertRaises(ValueError):
        pass
except AssertionError:
    _raised = True
assert _raised, "assertraises_no_exception_raises: expected AssertionError"
print("assertraises_no_exception_raises OK")
