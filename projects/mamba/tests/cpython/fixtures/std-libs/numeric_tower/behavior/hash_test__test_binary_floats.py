# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numeric_tower"
# dimension = "behavior"
# case = "hash_test__test_binary_floats"
# subject = "cpython.test_numeric_tower.HashTest.test_binary_floats"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_numeric_tower.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_numeric_tower.py::HashTest::test_binary_floats
"""Auto-ported test: HashTest::test_binary_floats (CPython 3.12 oracle)."""


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
check_equal_hash(0.0, -0.0)
check_equal_hash(0.0, D(0))
check_equal_hash(-0.0, D(0))
check_equal_hash(-0.0, D('-0.0'))
check_equal_hash(0.0, F(0))
check_equal_hash(float('inf'), D('inf'))
check_equal_hash(float('-inf'), D('-inf'))
for _ in range(1000):
    x = random.random() * math.exp(random.random() * 200.0 - 100.0)
    check_equal_hash(x, D.from_float(x))
    check_equal_hash(x, F.from_float(x))
print("HashTest::test_binary_floats: ok")
