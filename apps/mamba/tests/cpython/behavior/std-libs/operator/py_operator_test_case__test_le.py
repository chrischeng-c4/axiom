# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_le"
# subject = "cpython.test_operator.PyOperatorTestCase.test_le"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_le
"""Auto-ported test: PyOperatorTestCase::test_le (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.le()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.le(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not operator.le(1, 0)

assert not operator.le(1, 0.0)

assert operator.le(1, 1)

assert operator.le(1, 1.0)

assert operator.le(1, 2)

assert operator.le(1, 2.0)
print("PyOperatorTestCase::test_le: ok")
