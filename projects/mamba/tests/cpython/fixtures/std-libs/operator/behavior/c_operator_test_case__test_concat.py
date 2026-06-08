# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_concat"
# subject = "cpython.test_operator.COperatorTestCase.test_concat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_concat
"""Auto-ported test: COperatorTestCase::test_concat (CPython 3.12 oracle)."""


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

try:
    operator.concat()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.concat(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.concat('py', 'thon') == 'python'

assert operator.concat([1, 2], [3, 4]) == [1, 2, 3, 4]

assert operator.concat(Seq1([5, 6]), Seq1([7])) == [5, 6, 7]

assert operator.concat(Seq2([5, 6]), Seq2([7])) == [5, 6, 7]

try:
    operator.concat(13, 29)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("COperatorTestCase::test_concat: ok")
