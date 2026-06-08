# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_getnewargs"
# subject = "cpython.test_complex.ComplexTest.test_getnewargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_getnewargs
"""Auto-ported test: ComplexTest::test_getnewargs (CPython 3.12 oracle)."""


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

assert (1 + 2j).__getnewargs__() == (1.0, 2.0)

assert (1 - 2j).__getnewargs__() == (1.0, -2.0)

assert 2j.__getnewargs__() == (0.0, 2.0)

assert (-0j).__getnewargs__() == (0.0, -0.0)

assert complex(0, INF).__getnewargs__() == (0.0, INF)

assert complex(INF, 0).__getnewargs__() == (INF, 0.0)
print("ComplexTest::test_getnewargs: ok")
