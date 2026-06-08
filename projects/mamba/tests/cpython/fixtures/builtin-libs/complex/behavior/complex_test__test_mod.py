# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_mod"
# subject = "cpython.test_complex.ComplexTest.test_mod"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_mod
"""Auto-ported test: ComplexTest::test_mod (CPython 3.12 oracle)."""


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
try:
    (1 + 1j) % (1 + 0j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    (1 + 1j) % 1.0
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    (1 + 1j) % 1
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    1.0 % (1 + 0j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    1 % (1 + 0j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ComplexTest::test_mod: ok")
