# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_richcompare"
# subject = "cpython.test_complex.ComplexTest.test_richcompare"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_richcompare
"""Auto-ported test: ComplexTest::test_richcompare (CPython 3.12 oracle)."""


import unittest
import sys
from test import support
from test.support.testcase import ComplexesAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from random import random
from math import isnan, copysign
import operator


INF = float('inf')

NAN = float('nan')

ZERO_DIVISION = ((1 + 1j, 0 + 0j), (1 + 1j, 0.0), (1 + 1j, 0), (1.0, 0 + 0j), (1, 0 + 0j))

class WithIndex:

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class WithFloat:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class ComplexSubclass(complex):
    pass

class WithComplex:

    def __init__(self, value):
        self.value = value

    def __complex__(self):
        return self.value


# --- test body ---

assert complex.__eq__(1 + 1j, 1 << 10000) is False

assert complex.__lt__(1 + 1j, None) is NotImplemented

assert complex.__eq__(1 + 1j, None) is NotImplemented

assert complex.__eq__(1 + 1j, 1 + 1j) is True

assert complex.__eq__(1 + 1j, 2 + 2j) is False

assert complex.__ne__(1 + 1j, 1 + 1j) is False

assert complex.__ne__(1 + 1j, 2 + 2j) is True
for i in range(1, 100):
    f = i / 100.0

    assert complex.__eq__(f + 0j, f) is True

    assert complex.__ne__(f + 0j, f) is False

    assert complex.__eq__(complex(f, f), f) is False

    assert complex.__ne__(complex(f, f), f) is True

assert complex.__lt__(1 + 1j, 2 + 2j) is NotImplemented

assert complex.__le__(1 + 1j, 2 + 2j) is NotImplemented

assert complex.__gt__(1 + 1j, 2 + 2j) is NotImplemented

assert complex.__ge__(1 + 1j, 2 + 2j) is NotImplemented

try:
    operator.lt(1 + 1j, 2 + 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.le(1 + 1j, 2 + 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.gt(1 + 1j, 2 + 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.ge(1 + 1j, 2 + 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.eq(1 + 1j, 1 + 1j) is True

assert operator.eq(1 + 1j, 2 + 2j) is False

assert operator.ne(1 + 1j, 1 + 1j) is False

assert operator.ne(1 + 1j, 2 + 2j) is True

assert operator.eq(1 + 1j, 2.0) is False
print("ComplexTest::test_richcompare: ok")
