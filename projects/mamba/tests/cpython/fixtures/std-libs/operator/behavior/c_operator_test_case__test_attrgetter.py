# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_attrgetter"
# subject = "cpython.test_operator.COperatorTestCase.test_attrgetter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_attrgetter
"""Auto-ported test: COperatorTestCase::test_attrgetter (CPython 3.12 oracle)."""


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

class A:
    pass
a = A()
a.name = 'arthur'
f = operator.attrgetter('name')

assert f(a) == 'arthur'

try:
    f()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    f(a, 'dent')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    f(a, surname='dent')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
f = operator.attrgetter('rank')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

try:
    operator.attrgetter(2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.attrgetter()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
record = A()
record.x = 'X'
record.y = 'Y'
record.z = 'Z'

assert operator.attrgetter('x', 'z', 'y')(record) == ('X', 'Z', 'Y')

try:
    operator.attrgetter(('x', (), 'y'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class C(object):

    def __getattr__(self, name):
        raise SyntaxError

try:
    operator.attrgetter('foo')(C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
a = A()
a.name = 'arthur'
a.child = A()
a.child.name = 'thomas'
f = operator.attrgetter('child.name')

assert f(a) == 'thomas'

try:
    f(a.child)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('name', 'child.name')

assert f(a) == ('arthur', 'thomas')
f = operator.attrgetter('name', 'child.name', 'child.child.name')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('child.')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('.child')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
a.child.child = A()
a.child.child.name = 'johnson'
f = operator.attrgetter('child.child.name')

assert f(a) == 'johnson'
f = operator.attrgetter('name', 'child.name', 'child.child.name')

assert f(a) == ('arthur', 'thomas', 'johnson')
print("COperatorTestCase::test_attrgetter: ok")
