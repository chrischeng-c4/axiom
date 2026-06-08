# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "richcmp"
# dimension = "behavior"
# case = "number_test__test_values"
# subject = "cpython.test_richcmp.NumberTest.test_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_richcmp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_richcmp.py::NumberTest::test_values
"""Auto-ported test: NumberTest::test_values (CPython 3.12 oracle)."""


import unittest
from test import support
import operator


class Number:

    def __init__(self, x):
        self.x = x

    def __lt__(self, other):
        return self.x < other

    def __le__(self, other):
        return self.x <= other

    def __eq__(self, other):
        return self.x == other

    def __ne__(self, other):
        return self.x != other

    def __gt__(self, other):
        return self.x > other

    def __ge__(self, other):
        return self.x >= other

    def __cmp__(self, other):
        raise support.TestFailed('Number.__cmp__() should not be called')

    def __repr__(self):
        return 'Number(%r)' % (self.x,)

class Vector:

    def __init__(self, data):
        self.data = data

    def __len__(self):
        return len(self.data)

    def __getitem__(self, i):
        return self.data[i]

    def __setitem__(self, i, v):
        self.data[i] = v
    __hash__ = None

    def __bool__(self):
        raise TypeError('Vectors cannot be used in Boolean contexts')

    def __cmp__(self, other):
        raise support.TestFailed('Vector.__cmp__() should not be called')

    def __repr__(self):
        return 'Vector(%r)' % (self.data,)

    def __lt__(self, other):
        return Vector([a < b for a, b in zip(self.data, self.__cast(other))])

    def __le__(self, other):
        return Vector([a <= b for a, b in zip(self.data, self.__cast(other))])

    def __eq__(self, other):
        return Vector([a == b for a, b in zip(self.data, self.__cast(other))])

    def __ne__(self, other):
        return Vector([a != b for a, b in zip(self.data, self.__cast(other))])

    def __gt__(self, other):
        return Vector([a > b for a, b in zip(self.data, self.__cast(other))])

    def __ge__(self, other):
        return Vector([a >= b for a, b in zip(self.data, self.__cast(other))])

    def __cast(self, other):
        if isinstance(other, Vector):
            other = other.data
        if len(self.data) != len(other):
            raise ValueError('Cannot compare vectors of different length')
        return other

opmap = {'lt': (lambda a, b: a < b, operator.lt, operator.__lt__), 'le': (lambda a, b: a <= b, operator.le, operator.__le__), 'eq': (lambda a, b: a == b, operator.eq, operator.__eq__), 'ne': (lambda a, b: a != b, operator.ne, operator.__ne__), 'gt': (lambda a, b: a > b, operator.gt, operator.__gt__), 'ge': (lambda a, b: a >= b, operator.ge, operator.__ge__)}


# --- test body ---
def checkvalue(opname, a, b, expres):
    for typea in (int, Number):
        for typeb in (int, Number):
            ta = typea(a)
            tb = typeb(b)
            for op in opmap[opname]:
                realres = op(ta, tb)
                realres = getattr(realres, 'x', realres)

                assert realres is expres
checkvalue('lt', 0, 0, False)
checkvalue('le', 0, 0, True)
checkvalue('eq', 0, 0, True)
checkvalue('ne', 0, 0, False)
checkvalue('gt', 0, 0, False)
checkvalue('ge', 0, 0, True)
checkvalue('lt', 0, 1, True)
checkvalue('le', 0, 1, True)
checkvalue('eq', 0, 1, False)
checkvalue('ne', 0, 1, True)
checkvalue('gt', 0, 1, False)
checkvalue('ge', 0, 1, False)
checkvalue('lt', 1, 0, False)
checkvalue('le', 1, 0, False)
checkvalue('eq', 1, 0, False)
checkvalue('ne', 1, 0, True)
checkvalue('gt', 1, 0, True)
checkvalue('ge', 1, 0, True)
print("NumberTest::test_values: ok")
