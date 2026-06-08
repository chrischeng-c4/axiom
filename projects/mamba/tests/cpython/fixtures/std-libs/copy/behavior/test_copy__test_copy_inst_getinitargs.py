# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_inst_getinitargs"
# subject = "cpython.test_copy.TestCopy.test_copy_inst_getinitargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_inst_getinitargs
"""Auto-ported test: TestCopy::test_copy_inst_getinitargs (CPython 3.12 oracle)."""


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
class C:

    def __init__(self, foo):
        self.foo = foo

    def __getinitargs__(self):
        return (self.foo,)

    def __eq__(self, other):
        return self.foo == other.foo
x = C(42)

assert copy.copy(x) == x
print("TestCopy::test_copy_inst_getinitargs: ok")
