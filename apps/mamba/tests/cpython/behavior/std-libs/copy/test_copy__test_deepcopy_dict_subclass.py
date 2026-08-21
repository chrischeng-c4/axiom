# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_dict_subclass"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_dict_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_dict_subclass
"""Auto-ported test: TestCopy::test_deepcopy_dict_subclass (CPython 3.12 oracle)."""


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

    def __init__(self, d=None):
        if not d:
            d = {}
        self._keys = list(d.keys())
        super().__init__(d)

    def __setitem__(self, key, item):
        super().__setitem__(key, item)
        if key not in self._keys:
            self._keys.append(key)
x = C(d={'foo': 0})
y = copy.deepcopy(x)

assert x == y

assert x._keys == y._keys

assert x is not y
x['bar'] = 1

assert x != y

assert x._keys != y._keys
print("TestCopy::test_deepcopy_dict_subclass: ok")
