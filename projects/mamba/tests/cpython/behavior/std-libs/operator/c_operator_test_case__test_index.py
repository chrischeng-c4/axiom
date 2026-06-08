# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_index"
# subject = "cpython.test_operator.COperatorTestCase.test_index"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_index
"""Auto-ported test: COperatorTestCase::test_index (CPython 3.12 oracle)."""


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
module = c_operator
operator = module

class X:

    def __index__(self):
        return 1

assert operator.index(X()) == 1

assert operator.index(0) == 0

assert operator.index(1) == 1

assert operator.index(2) == 2
try:
    operator.index(1.5)
    raise AssertionError('expected (AttributeError, TypeError)')
except (AttributeError, TypeError):
    pass
try:
    operator.index(Fraction(3, 7))
    raise AssertionError('expected (AttributeError, TypeError)')
except (AttributeError, TypeError):
    pass
try:
    operator.index(Decimal(1))
    raise AssertionError('expected (AttributeError, TypeError)')
except (AttributeError, TypeError):
    pass
try:
    operator.index(None)
    raise AssertionError('expected (AttributeError, TypeError)')
except (AttributeError, TypeError):
    pass
print("COperatorTestCase::test_index: ok")
