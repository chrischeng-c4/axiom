# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_bound_method"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_bound_method"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_bound_method
"""Auto-ported test: TestCopy::test_deepcopy_bound_method (CPython 3.12 oracle)."""


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
class Foo(object):

    def m(self):
        pass
f = Foo()
f.b = f.m
g = copy.deepcopy(f)

assert g.m == g.b

assert g.b.__self__ is g
g.b()
print("TestCopy::test_deepcopy_bound_method: ok")
