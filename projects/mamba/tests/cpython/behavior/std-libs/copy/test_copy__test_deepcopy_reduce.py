# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_reduce"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_reduce"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_reduce
"""Auto-ported test: TestCopy::test_deepcopy_reduce (CPython 3.12 oracle)."""


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
        c.append(1)
        return ''
c = []
x = C()
y = copy.deepcopy(x)

assert y is x

assert c == [1]
print("TestCopy::test_deepcopy_reduce: ok")
