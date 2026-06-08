# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numeric_tower"
# dimension = "behavior"
# case = "hash_test__test_integers"
# subject = "cpython.test_numeric_tower.HashTest.test_integers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_numeric_tower.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_numeric_tower.py::HashTest::test_integers
"""Auto-ported test: HashTest::test_integers (CPython 3.12 oracle)."""


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
for i in range(-1000, 1000):
    check_equal_hash(i, float(i))
    check_equal_hash(i, D(i))
    check_equal_hash(i, F(i))
for i in range(100):
    n = 2 ** i - 1
    if n == int(float(n)):
        check_equal_hash(n, float(n))
        check_equal_hash(-n, -float(n))
    check_equal_hash(n, D(n))
    check_equal_hash(n, F(n))
    check_equal_hash(-n, D(-n))
    check_equal_hash(-n, F(-n))
    n = 2 ** i
    check_equal_hash(n, float(n))
    check_equal_hash(-n, -float(n))
    check_equal_hash(n, D(n))
    check_equal_hash(n, F(n))
    check_equal_hash(-n, D(-n))
    check_equal_hash(-n, F(-n))
for _ in range(1000):
    e = random.randrange(300)
    n = random.randrange(-10 ** e, 10 ** e)
    check_equal_hash(n, D(n))
    check_equal_hash(n, F(n))
    if n == int(float(n)):
        check_equal_hash(n, float(n))
print("HashTest::test_integers: ok")
