# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_cant"
# subject = "cpython.test_copy.TestCopy.test_copy_cant"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_cant
"""Auto-ported test: TestCopy::test_copy_cant (CPython 3.12 oracle)."""


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

    def __getattribute__(self, name):
        if name.startswith('__reduce'):
            raise AttributeError(name)
        return object.__getattribute__(self, name)
x = C()

try:
    copy.copy(x)
    raise AssertionError('expected copy.Error')
except copy.Error:
    pass
print("TestCopy::test_copy_cant: ok")
