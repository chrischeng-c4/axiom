# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numeric_tower"
# dimension = "behavior"
# case = "hash_test__test_fractions"
# subject = "cpython.test_numeric_tower.HashTest.test_fractions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_numeric_tower.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_numeric_tower.py::HashTest::test_fractions
"""Auto-ported test: HashTest::test_fractions (CPython 3.12 oracle)."""


import unittest
import random
import math
import sys
import operator
from decimal import Decimal as D
from fractions import Fraction as F


_PyHASH_MODULUS = sys.hash_info.modulus

_PyHASH_INF = sys.hash_info.inf

class DummyIntegral(int):
    """Dummy Integral class to test conversion of the Rational to float."""

    def __mul__(self, other):
        return DummyIntegral(super().__mul__(other))
    __rmul__ = __mul__

    def __truediv__(self, other):
        return NotImplemented
    __rtruediv__ = __truediv__

    @property
    def numerator(self):
        return DummyIntegral(self)

    @property
    def denominator(self):
        return DummyIntegral(1)


# --- test body ---

assert hash(F(1, _PyHASH_MODULUS)) == _PyHASH_INF

assert hash(F(-1, 3 * _PyHASH_MODULUS)) == -_PyHASH_INF

assert hash(F(7 * _PyHASH_MODULUS, 1)) == 0

assert hash(F(-_PyHASH_MODULUS, 1)) == 0
x = F._from_coprime_ints(DummyIntegral(1), DummyIntegral(2))

try:
    (lambda: x.numerator / x.denominator)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert float(x) == 0.5
print("HashTest::test_fractions: ok")
