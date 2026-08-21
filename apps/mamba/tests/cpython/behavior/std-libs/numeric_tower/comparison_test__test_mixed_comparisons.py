# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numeric_tower"
# dimension = "behavior"
# case = "comparison_test__test_mixed_comparisons"
# subject = "cpython.test_numeric_tower.ComparisonTest.test_mixed_comparisons"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_numeric_tower.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_numeric_tower.py::ComparisonTest::test_mixed_comparisons
"""Auto-ported test: ComparisonTest::test_mixed_comparisons (CPython 3.12 oracle)."""


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
test_values = [float('-inf'), D('-1e425000000'), -1e+308, F(-22, 7), -3.14, -2, 0.0, 1e-320, True, F('1.2'), D('1.3'), float('1.4'), F(275807, 195025), D('1.414213562373095048801688724'), F(114243, 80782), F(473596569, 84615), 7e+200, D('infinity')]
for i, first in enumerate(test_values):
    for second in test_values[i + 1:]:

        assert first < second

        assert first <= second

        assert second > first

        assert second >= first
print("ComparisonTest::test_mixed_comparisons: ok")
