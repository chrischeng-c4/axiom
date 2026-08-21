# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_registry"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_registry"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_registry
"""Auto-ported test: TestCopy::test_deepcopy_registry (CPython 3.12 oracle)."""


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

    def __new__(cls, foo):
        obj = object.__new__(cls)
        obj.foo = foo
        return obj

def pickle_C(obj):
    return (C, (obj.foo,))
x = C(42)

try:
    copy.deepcopy(x)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
copyreg.pickle(C, pickle_C, C)
y = copy.deepcopy(x)

assert x is not y

assert type(y) == C

assert y.foo == x.foo
print("TestCopy::test_deepcopy_registry: ok")
