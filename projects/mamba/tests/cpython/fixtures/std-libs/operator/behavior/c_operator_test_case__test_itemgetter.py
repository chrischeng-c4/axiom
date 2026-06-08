# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_itemgetter"
# subject = "cpython.test_operator.COperatorTestCase.test_itemgetter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_itemgetter
"""Auto-ported test: COperatorTestCase::test_itemgetter (CPython 3.12 oracle)."""


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
a = 'ABCDE'
f = operator.itemgetter(2)

assert f(a) == 'C'

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
    f(a, size=3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
f = operator.itemgetter(10)

try:
    f(a)
    raise AssertionError('expected IndexError')
except IndexError:
    pass

class C(object):

    def __getitem__(self, name):
        raise SyntaxError

try:
    operator.itemgetter(42)(C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
f = operator.itemgetter('name')

try:
    f(a)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.itemgetter()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
d = dict(key='val')
f = operator.itemgetter('key')

assert f(d) == 'val'
f = operator.itemgetter('nonkey')

try:
    f(d)
    raise AssertionError('expected KeyError')
except KeyError:
    pass
inventory = [('apple', 3), ('banana', 2), ('pear', 5), ('orange', 1)]
getcount = operator.itemgetter(1)

assert list(map(getcount, inventory)) == [3, 2, 5, 1]

assert sorted(inventory, key=getcount) == [('orange', 1), ('banana', 2), ('apple', 3), ('pear', 5)]
data = list(map(str, range(20)))

assert operator.itemgetter(2, 10, 5)(data) == ('2', '10', '5')

try:
    operator.itemgetter(2, 'x', 5)(data)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
t = tuple('abcde')

assert operator.itemgetter(-1)(t) == 'e'

assert operator.itemgetter(slice(2, 4))(t) == ('c', 'd')

class T(tuple):
    """Tuple subclass"""
    pass

assert operator.itemgetter(0)(T('abc')) == 'a'

assert operator.itemgetter(0)(['a', 'b', 'c']) == 'a'

assert operator.itemgetter(0)(range(100, 200)) == 100
print("COperatorTestCase::test_itemgetter: ok")
