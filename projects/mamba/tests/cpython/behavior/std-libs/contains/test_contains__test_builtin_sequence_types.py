# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contains"
# dimension = "behavior"
# case = "test_contains__test_builtin_sequence_types"
# subject = "cpython.test_contains.TestContains.test_builtin_sequence_types"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contains.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contains.py::TestContains::test_builtin_sequence_types
"""Auto-ported test: TestContains::test_builtin_sequence_types (CPython 3.12 oracle)."""


from collections import deque
import unittest
from test.support import NEVER_EQ


class base_set:

    def __init__(self, el):
        self.el = el

class myset(base_set):

    def __contains__(self, el):
        return self.el == el

class seq(base_set):

    def __getitem__(self, n):
        return [self.el][n]


# --- test body ---
a = range(10)
for i in a:

    assert i in a

assert 16 not in a

assert a not in a
a = tuple(a)
for i in a:

    assert i in a

assert 16 not in a

assert a not in a

class Deviant1:
    """Behaves strangely when compared

            This class is designed to make sure that the contains code
            works when the list is modified during the check.
            """
    aList = list(range(15))

    def __eq__(self, other):
        if other == 12:
            self.aList.remove(12)
            self.aList.remove(13)
            self.aList.remove(14)
        return 0

assert Deviant1() not in Deviant1.aList
print("TestContains::test_builtin_sequence_types: ok")
