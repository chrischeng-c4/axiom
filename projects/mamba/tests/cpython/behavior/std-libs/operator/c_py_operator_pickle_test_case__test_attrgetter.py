# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_py_operator_pickle_test_case__test_attrgetter"
# subject = "cpython.test_operator.CPyOperatorPickleTestCase.test_attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::CPyOperatorPickleTestCase::test_attrgetter
"""Auto-ported test: CPyOperatorPickleTestCase::test_attrgetter (CPython 3.12 oracle)."""


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
module2 = py_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
attrgetter = module.attrgetter

class A:
    pass
a = A()
a.x = 'X'
a.y = 'Y'
a.z = 'Z'
a.t = A()
a.t.u = A()
a.t.u.v = 'V'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = attrgetter('x')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('x', 'y', 'z')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('t.u.v')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("CPyOperatorPickleTestCase::test_attrgetter: ok")
