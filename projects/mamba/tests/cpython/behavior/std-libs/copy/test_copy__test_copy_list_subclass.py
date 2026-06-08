# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_list_subclass"
# subject = "cpython.test_copy.TestCopy.test_copy_list_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_list_subclass
"""Auto-ported test: TestCopy::test_copy_list_subclass (CPython 3.12 oracle)."""


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
class C(list):
    pass
x = C([[1, 2], 3])
x.foo = [4, 5]
y = copy.copy(x)

assert list(x) == list(y)

assert x.foo == y.foo

assert x[0] is y[0]

assert x.foo is y.foo
print("TestCopy::test_copy_list_subclass: ok")
