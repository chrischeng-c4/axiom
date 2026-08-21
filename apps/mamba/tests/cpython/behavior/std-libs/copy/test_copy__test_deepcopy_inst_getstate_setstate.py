# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_inst_getstate_setstate"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_inst_getstate_setstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_inst_getstate_setstate
"""Auto-ported test: TestCopy::test_deepcopy_inst_getstate_setstate (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __getstate__(self):
        return self.foo

    def __setstate__(self, state):
        self.foo = state

    def __eq__(self, other):
        return self.foo == other.foo
x = C([42])
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y.foo is not x.foo
x = C([])
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y.foo is not x.foo
print("TestCopy::test_deepcopy_inst_getstate_setstate: ok")
