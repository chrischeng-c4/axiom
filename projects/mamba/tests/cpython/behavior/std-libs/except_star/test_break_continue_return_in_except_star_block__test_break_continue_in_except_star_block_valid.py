# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_break_continue_return_in_except_star_block__test_break_continue_in_except_star_block_valid"
# subject = "cpython.test_except_star.TestBreakContinueReturnInExceptStarBlock.test_break_continue_in_except_star_block_valid"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_except_star.py::TestBreakContinueReturnInExceptStarBlock::test_break_continue_in_except_star_block_valid
"""Auto-ported test: TestBreakContinueReturnInExceptStarBlock::test_break_continue_in_except_star_block_valid (CPython 3.12 oracle)."""


import sys
import unittest
import textwrap
from test.support.testcase import ExceptionIsLikeMixin


class ExceptStarTest(ExceptionIsLikeMixin, unittest.TestCase):

    def assertMetadataEqual(self, e1, e2):
        if e1 is None or e2 is None:
            self.assertTrue(e1 is None and e2 is None)
        else:
            self.assertEqual(e1.__context__, e2.__context__)
            self.assertEqual(e1.__cause__, e2.__cause__)
            self.assertEqual(e1.__traceback__, e2.__traceback__)

    def assertMetadataNotEqual(self, e1, e2):
        if e1 is None or e2 is None:
            self.assertNotEqual(e1, e2)
        else:
            return not (e1.__context__ == e2.__context__ and e1.__cause__ == e2.__cause__ and (e1.__traceback__ == e2.__traceback__))


# --- test body ---
MSG = "'break', 'continue' and 'return' cannot appear in an except\\* block"
try:
    raise ValueError(42)
except* Exception as e:
    count = 0
    for i in range(5):
        if i == 0:
            continue
        if i == 4:
            break
        count += 1

    assert count == 3

    assert i == 4
    exc = e

assert isinstance(exc, ExceptionGroup)
print("TestBreakContinueReturnInExceptStarBlock::test_break_continue_in_except_star_block_valid: ok")
