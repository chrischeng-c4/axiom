# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "assertequal_mismatch_raises"
# subject = "unittest.TestCase.assertEqual"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.assertEqual: calling assertEqual(1, 2) on a TestCase instance raises the failureException (AssertionError) outside a runner"""
import unittest


class Sample(unittest.TestCase):
    def runTest(self):
        pass


tc = Sample()
_raised = False
try:
    tc.assertEqual(1, 2)
except AssertionError:
    _raised = True
assert _raised, "assertequal_mismatch_raises: expected AssertionError"
print("assertequal_mismatch_raises OK")
