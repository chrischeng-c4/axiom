# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numeric_tower"
# dimension = "behavior"
# case = "comparison_test__test_complex"
# subject = "cpython.test_numeric_tower.ComparisonTest.test_complex"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_numeric_tower.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_numeric_tower.py::ComparisonTest::test_complex
"""Auto-ported test: ComparisonTest::test_complex (CPython 3.12 oracle)."""


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
z = 1.0 + 0j
w = -3.14 + 2.7j
for v in (1, 1.0, F(1), D(1), complex(1)):

    assert z == v

    assert v == z
for v in (2, 2.0, F(2), D(2), complex(2)):

    assert z != v

    assert v != z

    assert w != v

    assert v != w
for v in (1, 1.0, F(1), D(1), complex(1), 2, 2.0, F(2), D(2), complex(2), w):
    for op in (operator.le, operator.lt, operator.ge, operator.gt):

        try:
            op(z, v)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            op(v, z)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass
print("ComparisonTest::test_complex: ok")
