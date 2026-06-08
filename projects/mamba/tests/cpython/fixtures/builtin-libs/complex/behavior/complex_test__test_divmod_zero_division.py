# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_divmod_zero_division"
# subject = "cpython.test.test_complex.ComplexTest.test_divmod_zero_division"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_divmod_zero_division
"""Auto-ported test: ComplexTest::test_divmod_zero_division (CPython 3.12 oracle)."""


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
for a, b in ZERO_DIVISION:

    try:
        divmod(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("ComplexTest::test_divmod_zero_division: ok")
