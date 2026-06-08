# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_except_star_split_semantics__test_nested_in_loop"
# subject = "cpython.test_except_star.TestExceptStarSplitSemantics.test_nested_in_loop"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_except_star.py::TestExceptStarSplitSemantics::test_nested_in_loop
"""Auto-ported test: TestExceptStarSplitSemantics::test_nested_in_loop (CPython 3.12 oracle)."""


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
for _ in range(2):
    try:
        raise ExceptionGroup('nl', [BlockingIOError('io')])
    except* BlockingIOError:
        pass
    else:

        raise AssertionError('Exception not raised')
print("TestExceptStarSplitSemantics::test_nested_in_loop: ok")
