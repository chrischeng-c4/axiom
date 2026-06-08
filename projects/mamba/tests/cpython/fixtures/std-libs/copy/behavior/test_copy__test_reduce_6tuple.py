# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reduce_6tuple"
# subject = "cpython.test_copy.TestCopy.test_reduce_6tuple"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reduce_6tuple
"""Auto-ported test: TestCopy::test_reduce_6tuple (CPython 3.12 oracle)."""


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
def state_setter(*args, **kwargs):
    self.fail("shouldn't call this")

class C:

    def __reduce__(self):
        return (C, (), self.__dict__, None, None, state_setter)
x = C()
try:
    copy.copy(x)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    copy.deepcopy(x)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestCopy::test_reduce_6tuple: ok")
