# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "long"
# dimension = "behavior"
# case = "long_test__test_long"
# subject = "cpython.test_long.LongTest.test_long"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_long.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_long.py::LongTest::test_long
"""Auto-ported test: LongTest::test_long (CPython 3.12 oracle)."""


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
LL = [('1' + '0' * 20, 10 ** 20), ('1' + '0' * 100, 10 ** 100)]
for s, v in LL:
    for sign in ('', '+', '-'):
        for prefix in ('', ' ', '\t', '  \t\t  '):
            ss = prefix + sign + s
            vv = v
            if sign == '-' and v is not ValueError:
                vv = -v
            try:

                assert int(ss) == vv
            except ValueError:
                pass

try:
    int('123L')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('123l')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0L')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('-37L')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0x32L', 16)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('1L', 21)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert int('1L', 22) == 43

assert int('000', 0) == 0

assert int('0o123', 0) == 83

assert int('0x123', 0) == 291

assert int('0b100', 0) == 4

assert int(' 0O123   ', 0) == 83

assert int(' 0X123  ', 0) == 291

assert int(' 0B100 ', 0) == 4

assert int('0', 0) == 0

assert int('+0', 0) == 0

assert int('-0', 0) == 0

assert int('00', 0) == 0

try:
    int('08', 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('-012395', 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
invalid_bases = [-909, 2 ** 31 - 1, 2 ** 31, -2 ** 31, -2 ** 31 - 1, 2 ** 63 - 1, 2 ** 63, -2 ** 63, -2 ** 63 - 1, 2 ** 100, -2 ** 100]
for base in invalid_bases:

    try:
        int('42', base)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

try:
    int('こんにちは')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("LongTest::test_long: ok")
