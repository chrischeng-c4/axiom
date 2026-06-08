# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_function"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_function"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_function
"""Auto-ported test: TestCopy::test_deepcopy_function (CPython 3.12 oracle)."""


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

assert copy.deepcopy(global_foo) == global_foo

def foo(x, y):
    return x + y

assert copy.deepcopy(foo) == foo
bar = lambda: None

assert copy.deepcopy(bar) == bar
print("TestCopy::test_deepcopy_function: ok")
