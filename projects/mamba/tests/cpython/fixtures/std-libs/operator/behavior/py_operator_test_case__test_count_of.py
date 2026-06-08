# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_count_of"
# subject = "cpython.test_operator.PyOperatorTestCase.test_countOf"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_countOf
"""Auto-ported test: PyOperatorTestCase::test_countOf (CPython 3.12 oracle)."""


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
    operator.countOf()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.countOf(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.countOf(BadIterable(), 1)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert operator.countOf([1, 2, 1, 3, 1, 4], 3) == 1

assert operator.countOf([1, 2, 1, 3, 1, 4], 5) == 0
nan = float('nan')

assert operator.countOf([nan, nan, 21], nan) == 2

assert operator.countOf([{}, 1, {}, 2], {}) == 2
print("PyOperatorTestCase::test_countOf: ok")
