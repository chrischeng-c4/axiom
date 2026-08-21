# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contains"
# dimension = "behavior"
# case = "test_contains__test_nonreflexive"
# subject = "cpython.test_contains.TestContains.test_nonreflexive"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contains.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contains.py::TestContains::test_nonreflexive
"""Auto-ported test: TestContains::test_nonreflexive (CPython 3.12 oracle)."""


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
values = (float('nan'), 1, None, 'abc', NEVER_EQ)
constructors = (list, tuple, dict.fromkeys, set, frozenset, deque)
for constructor in constructors:
    container = constructor(values)
    for elem in container:

        assert elem in container

    assert container == constructor(values)

    assert container == container
print("TestContains::test_nonreflexive: ok")
