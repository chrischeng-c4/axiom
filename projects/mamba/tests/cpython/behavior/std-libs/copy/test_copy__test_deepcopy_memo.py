# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_memo"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_memo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_memo
"""Auto-ported test: TestCopy::test_deepcopy_memo (CPython 3.12 oracle)."""


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
x = []
x = [x, x]
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y[0] is not x[0]

assert y[0] is y[1]
print("TestCopy::test_deepcopy_memo: ok")
