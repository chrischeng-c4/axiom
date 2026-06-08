# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "long"
# dimension = "behavior"
# case = "long_test__test_true_division"
# subject = "cpython.test_long.LongTest.test_true_division"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_long.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_long.py::LongTest::test_true_division
"""Auto-ported test: LongTest::test_true_division (CPython 3.12 oracle)."""


import unittest
from test import support
import sys
import random
import math
import array


SHIFT = sys.int_info.bits_per_digit

BASE = 2 ** SHIFT

MASK = BASE - 1

KARATSUBA_CUTOFF = 70

MAXDIGITS = 15

special = [0, 1, 2, BASE, BASE >> 1, 6148914691236517205, 12297829382473034410]

p2 = 4

for i in range(2 * SHIFT):
    special.append(p2 - 1)
    p2 = p2 << 1

del p2

special += [~x for x in special] + [-x for x in special]

DBL_MAX = sys.float_info.max

DBL_MAX_EXP = sys.float_info.max_exp

DBL_MIN_EXP = sys.float_info.min_exp

DBL_MANT_DIG = sys.float_info.mant_dig

DBL_MIN_OVERFLOW = 2 ** DBL_MAX_EXP - 2 ** (DBL_MAX_EXP - DBL_MANT_DIG - 1)

def int_to_float(n):
    """
    Correctly-rounded integer-to-float conversion.

    """
    PRECISION = sys.float_info.mant_dig + 2
    SHIFT_MAX = sys.float_info.max_exp - PRECISION
    Q_MAX = 1 << PRECISION
    ROUND_HALF_TO_EVEN_CORRECTION = [0, -1, -2, 1, 0, -1, 2, 1]
    if n == 0:
        return 0.0
    elif n < 0:
        return -int_to_float(-n)
    shift = n.bit_length() - PRECISION
    q = n << -shift if shift < 0 else n >> shift | bool(n & ~(-1 << shift))
    q += ROUND_HALF_TO_EVEN_CORRECTION[q & 7]
    if shift + (q == Q_MAX) > SHIFT_MAX:
        raise OverflowError('integer too large to convert to float')
    assert q % 4 == 0 and q // 4 <= 2 ** sys.float_info.mant_dig
    assert q * 2 ** shift <= sys.float_info.max
    return math.ldexp(float(q), shift)

def truediv(a, b):
    """Correctly-rounded true division for integers."""
    negative = a ^ b < 0
    a, b = (abs(a), abs(b))
    if not b:
        raise ZeroDivisionError('division by zero')
    if a >= DBL_MIN_OVERFLOW * b:
        raise OverflowError('int/int too large to represent as a float')
    d = a.bit_length() - b.bit_length()
    if d >= 0 and a >= 2 ** d * b or (d < 0 and a * 2 ** (-d) >= b):
        d += 1
    exp = max(d, DBL_MIN_EXP) - DBL_MANT_DIG
    a, b = (a << max(-exp, 0), b << max(exp, 0))
    q, r = divmod(a, b)
    if 2 * r > b or (2 * r == b and q % 2 == 1):
        q += 1
    result = math.ldexp(q, exp)
    return -result if negative else result


# --- test body ---
huge = 1 << 40000
mhuge = -huge

assert huge / huge == 1.0

assert mhuge / mhuge == 1.0

assert huge / mhuge == -1.0

assert mhuge / huge == -1.0

assert 1 / huge == 0.0

assert 1 / huge == 0.0

assert 1 / mhuge == 0.0

assert 1 / mhuge == 0.0

assert (666 * huge + (huge >> 1)) / huge == 666.5

assert (666 * mhuge + (mhuge >> 1)) / mhuge == 666.5

assert (666 * huge + (huge >> 1)) / mhuge == -666.5

assert (666 * mhuge + (mhuge >> 1)) / huge == -666.5

assert huge / (huge << 1) == 0.5

assert 1000000 * huge / huge == 1000000
namespace = {'huge': huge, 'mhuge': mhuge}
for overflow in ['float(huge)', 'float(mhuge)', 'huge / 1', 'huge / 2', 'huge / -1', 'huge / -2', 'mhuge / 100', 'mhuge / 200']:

    try:
        eval(overflow, namespace)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass
for underflow in ['1 / huge', '2 / huge', '-1 / huge', '-2 / huge', '100 / mhuge', '200 / mhuge']:
    result = eval(underflow, namespace)

    assert result == 0.0
for zero in ['huge / 0', 'mhuge / 0']:

    try:
        eval(zero, namespace)
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass
print("LongTest::test_true_division: ok")
