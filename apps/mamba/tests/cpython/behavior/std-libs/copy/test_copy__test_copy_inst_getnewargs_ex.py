# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_inst_getnewargs_ex"
# subject = "cpython.test_copy.TestCopy.test_copy_inst_getnewargs_ex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_inst_getnewargs_ex
"""Auto-ported test: TestCopy::test_copy_inst_getnewargs_ex (CPython 3.12 oracle)."""


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
class C(int):

    def __new__(cls, *, foo):
        self = int.__new__(cls)
        self.foo = foo
        return self

    def __getnewargs_ex__(self):
        return ((), {'foo': self.foo})

    def __eq__(self, other):
        return self.foo == other.foo
x = C(foo=42)
y = copy.copy(x)

assert isinstance(y, C)

assert y == x

assert y is not x

assert y.foo == x.foo
print("TestCopy::test_copy_inst_getnewargs_ex: ok")
