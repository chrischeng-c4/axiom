# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "named_ctor_missing_method_raises"
# subject = "unittest.FunctionTestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.FunctionTestCase: constructing a TestCase with a method name that is not an attribute of the class raises ValueError"""
import unittest


class Sample(unittest.TestCase):
    def test_a(self):
        pass

    def runTest(self):
        pass


_raised = False
try:
    Sample("does_not_exist")
except ValueError:
    _raised = True
assert _raised, "named_ctor_missing_method_raises: expected ValueError"
print("named_ctor_missing_method_raises OK")
