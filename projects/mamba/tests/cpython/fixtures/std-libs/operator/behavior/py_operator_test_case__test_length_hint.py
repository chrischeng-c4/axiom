# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_length_hint"
# subject = "cpython.test_operator.PyOperatorTestCase.test_length_hint"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_length_hint
"""Auto-ported test: PyOperatorTestCase::test_length_hint (CPython 3.12 oracle)."""


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

class X(object):

    def __init__(self, value):
        self.value = value

    def __length_hint__(self):
        if type(self.value) is type:
            raise self.value
        else:
            return self.value

assert operator.length_hint([], 2) == 0

assert operator.length_hint(iter([1, 2, 3])) == 3

assert operator.length_hint(X(2)) == 2

assert operator.length_hint(X(NotImplemented), 4) == 4

assert operator.length_hint(X(TypeError), 12) == 12
try:
    operator.length_hint(X('abc'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    operator.length_hint(X(-2))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    operator.length_hint(X(LookupError))
    raise AssertionError('expected LookupError')
except LookupError:
    pass

class Y:
    pass
msg = "'str' object cannot be interpreted as an integer"
try:
    operator.length_hint(X(2), 'abc')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(msg, str(_aR_e))

assert operator.length_hint(Y(), 10) == 10
print("PyOperatorTestCase::test_length_hint: ok")
