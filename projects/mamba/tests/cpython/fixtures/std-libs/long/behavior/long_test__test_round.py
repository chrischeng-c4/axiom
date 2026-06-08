# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "long"
# dimension = "behavior"
# case = "long_test__test_round"
# subject = "cpython.test_long.LongTest.test_round"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_long.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_long.py::LongTest::test_round
"""Auto-ported test: LongTest::test_round (CPython 3.12 oracle)."""


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
test_dict = {0: 0, 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 10, 7: 10, 8: 10, 9: 10, 10: 10, 11: 10, 12: 10, 13: 10, 14: 10, 15: 20, 16: 20, 17: 20, 18: 20, 19: 20}
for offset in range(-520, 520, 20):
    for k, v in test_dict.items():
        got = round(k + offset, -1)
        expected = v + offset

        assert got == expected

        assert type(got) is int

assert round(-150, -2) == -200

assert round(-149, -2) == -100

assert round(-51, -2) == -100

assert round(-50, -2) == 0

assert round(-49, -2) == 0

assert round(-1, -2) == 0

assert round(0, -2) == 0

assert round(1, -2) == 0

assert round(49, -2) == 0

assert round(50, -2) == 0

assert round(51, -2) == 100

assert round(149, -2) == 100

assert round(150, -2) == 200

assert round(250, -2) == 200

assert round(251, -2) == 300

assert round(172500, -3) == 172000

assert round(173500, -3) == 174000

assert round(31415926535, -1) == 31415926540

assert round(31415926535, -2) == 31415926500

assert round(31415926535, -3) == 31415927000

assert round(31415926535, -4) == 31415930000

assert round(31415926535, -5) == 31415900000

assert round(31415926535, -6) == 31416000000

assert round(31415926535, -7) == 31420000000

assert round(31415926535, -8) == 31400000000

assert round(31415926535, -9) == 31000000000

assert round(31415926535, -10) == 30000000000

assert round(31415926535, -11) == 0

assert round(31415926535, -12) == 0

assert round(31415926535, -999) == 0
for k in range(10, 100):
    got = round(10 ** k + 324678, -3)
    expect = 10 ** k + 325000

    assert got == expect

    assert type(got) is int
for n in range(5):
    for i in range(100):
        x = random.randrange(-10000, 10000)
        got = round(x, n)

        assert got == x

        assert type(got) is int
for huge_n in (2 ** 31 - 1, 2 ** 31, 2 ** 63 - 1, 2 ** 63, 2 ** 100, 10 ** 100):

    assert round(8979323, huge_n) == 8979323
for i in range(100):
    x = random.randrange(-10000, 10000)
    got = round(x)

    assert got == x

    assert type(got) is int
bad_exponents = ('brian', 2.0, 0j)
for e in bad_exponents:

    try:
        round(3, e)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("LongTest::test_round: ok")
