# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reconstruct_state_setstate"
# subject = "cpython.test_copy.TestCopy.test_reconstruct_state_setstate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reconstruct_state_setstate
"""Auto-ported test: TestCopy::test_reconstruct_state_setstate (CPython 3.12 oracle)."""


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
        return (C, (), self.__dict__)

    def __setstate__(self, state):
        self.__dict__.update(state)

    def __eq__(self, other):
        return self.__dict__ == other.__dict__
x = C()
x.foo = [42]
y = copy.copy(x)

assert y == x
y = copy.deepcopy(x)

assert y == x

assert y.foo is not x.foo
print("TestCopy::test_reconstruct_state_setstate: ok")
