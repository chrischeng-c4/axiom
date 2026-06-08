# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_methodcaller"
# subject = "cpython.test_operator.COperatorTestCase.test_methodcaller"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_methodcaller
"""Auto-ported test: COperatorTestCase::test_methodcaller (CPython 3.12 oracle)."""


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
    operator.methodcaller()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.methodcaller(12)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class A:

    def foo(self, *args, **kwds):
        return args[0] + args[1]

    def bar(self, f=42):
        return f

    def baz(*args, **kwds):
        return (kwds['name'], kwds['self'])
a = A()
f = operator.methodcaller('foo')

try:
    f(a)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
f = operator.methodcaller('foo', 1, 2)

assert f(a) == 3

try:
    f()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    f(a, 3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    f(a, spam=3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
f = operator.methodcaller('bar')

assert f(a) == 42

try:
    f(a, a)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
f = operator.methodcaller('bar', f=5)

assert f(a) == 5
f = operator.methodcaller('baz', name='spam', self='eggs')

assert f(a) == ('spam', 'eggs')
print("COperatorTestCase::test_methodcaller: ok")
