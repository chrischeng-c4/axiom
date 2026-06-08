# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numeric_tower"
# dimension = "behavior"
# case = "hash_test__test_decimals"
# subject = "cpython.test_numeric_tower.HashTest.test_decimals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_numeric_tower.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_numeric_tower.py::HashTest::test_decimals
"""Auto-ported test: HashTest::test_decimals (CPython 3.12 oracle)."""


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
def check_equal_hash(x, y):

    assert hash(x) == hash(y)

    assert x == y
zeros = ['0', '-0', '0.0', '-0.0e10', '000e-10']
for zero in zeros:
    check_equal_hash(D(zero), D(0))
check_equal_hash(D('1.00'), D(1))
check_equal_hash(D('1.00000'), D(1))
check_equal_hash(D('-1.00'), D(-1))
check_equal_hash(D('-1.00000'), D(-1))
check_equal_hash(D('123e2'), D(12300))
check_equal_hash(D('1230e1'), D(12300))
check_equal_hash(D('12300'), D(12300))
check_equal_hash(D('12300.0'), D(12300))
check_equal_hash(D('12300.00'), D(12300))
check_equal_hash(D('12300.000'), D(12300))
print("HashTest::test_decimals: ok")
