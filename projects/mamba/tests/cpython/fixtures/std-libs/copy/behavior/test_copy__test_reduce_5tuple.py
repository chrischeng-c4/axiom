# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reduce_5tuple"
# subject = "cpython.test_copy.TestCopy.test_reduce_5tuple"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reduce_5tuple
"""Auto-ported test: TestCopy::test_reduce_5tuple (CPython 3.12 oracle)."""


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
class C(dict):

    def __reduce__(self):
        return (C, (), self.__dict__, None, self.items())

    def __eq__(self, other):
        return dict(self) == dict(other) and self.__dict__ == other.__dict__
x = C([('foo', [1, 2]), ('bar', 3)])
y = copy.copy(x)

assert x == y

assert x is not y

assert x['foo'] is y['foo']
y = copy.deepcopy(x)

assert x == y

assert x is not y

assert x['foo'] is not y['foo']
print("TestCopy::test_reduce_5tuple: ok")
