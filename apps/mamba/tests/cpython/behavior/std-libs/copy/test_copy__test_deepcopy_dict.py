# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_dict"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_dict
"""Auto-ported test: TestCopy::test_deepcopy_dict (CPython 3.12 oracle)."""


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
x = {'foo': [1, 2], 'bar': 3}
y = copy.deepcopy(x)

assert y == x

assert x is not y

assert x['foo'] is not y['foo']
print("TestCopy::test_deepcopy_dict: ok")
