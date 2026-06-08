# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reconstruct_nostate"
# subject = "cpython.test_copy.TestCopy.test_reconstruct_nostate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reconstruct_nostate
"""Auto-ported test: TestCopy::test_reconstruct_nostate (CPython 3.12 oracle)."""


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
class C(object):

    def __reduce__(self):
        return (C, ())
x = C()
x.foo = 42
y = copy.copy(x)

assert y.__class__ is x.__class__
y = copy.deepcopy(x)

assert y.__class__ is x.__class__
print("TestCopy::test_reconstruct_nostate: ok")
