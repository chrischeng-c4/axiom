# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_inplace"
# subject = "cpython.test_operator.COperatorTestCase.test_inplace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_inplace
"""Auto-ported test: COperatorTestCase::test_inplace (CPython 3.12 oracle)."""


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

class C(object):

    def __iadd__(self, other):
        return 'iadd'

    def __iand__(self, other):
        return 'iand'

    def __ifloordiv__(self, other):
        return 'ifloordiv'

    def __ilshift__(self, other):
        return 'ilshift'

    def __imod__(self, other):
        return 'imod'

    def __imul__(self, other):
        return 'imul'

    def __imatmul__(self, other):
        return 'imatmul'

    def __ior__(self, other):
        return 'ior'

    def __ipow__(self, other):
        return 'ipow'

    def __irshift__(self, other):
        return 'irshift'

    def __isub__(self, other):
        return 'isub'

    def __itruediv__(self, other):
        return 'itruediv'

    def __ixor__(self, other):
        return 'ixor'

    def __getitem__(self, other):
        return 5
c = C()

assert operator.iadd(c, 5) == 'iadd'

assert operator.iand(c, 5) == 'iand'

assert operator.ifloordiv(c, 5) == 'ifloordiv'

assert operator.ilshift(c, 5) == 'ilshift'

assert operator.imod(c, 5) == 'imod'

assert operator.imul(c, 5) == 'imul'

assert operator.imatmul(c, 5) == 'imatmul'

assert operator.ior(c, 5) == 'ior'

assert operator.ipow(c, 5) == 'ipow'

assert operator.irshift(c, 5) == 'irshift'

assert operator.isub(c, 5) == 'isub'

assert operator.itruediv(c, 5) == 'itruediv'

assert operator.ixor(c, 5) == 'ixor'

assert operator.iconcat(c, c) == 'iadd'
print("COperatorTestCase::test_inplace: ok")
