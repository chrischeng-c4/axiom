# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "long"
# dimension = "behavior"
# case = "long_test__test_correctly_rounded_true_division"
# subject = "cpython.test_long.LongTest.test_correctly_rounded_true_division"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_long.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_long.py::LongTest::test_correctly_rounded_true_division
"""Auto-ported test: LongTest::test_correctly_rounded_true_division (CPython 3.12 oracle)."""


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
def check_bitop_identities_1(x):
    eq = self_assertEqual
    eq(x & 0, 0)
    eq(x | 0, x)
    eq(x ^ 0, x)
    eq(x & -1, x)
    eq(x | -1, -1)
    eq(x ^ -1, ~x)
    eq(x, ~~x)
    eq(x & x, x)
    eq(x | x, x)
    eq(x ^ x, 0)
    eq(x & ~x, 0)
    eq(x | ~x, -1)
    eq(x ^ ~x, -1)
    eq(-x, 1 + ~x)
    eq(-x, ~(x - 1))
    for n in range(2 * SHIFT):
        p2 = 2 ** n
        eq(x << n >> n, x)
        eq(x // p2, x >> n)
        eq(x * p2, x << n)
        eq(x & -p2, x >> n << n)
        eq(x & -p2, x & ~(p2 - 1))

def check_bitop_identities_2(x, y):
    eq = self_assertEqual
    eq(x & y, y & x)
    eq(x | y, y | x)
    eq(x ^ y, y ^ x)
    eq(x ^ y ^ x, y)
    eq(x & y, ~(~x | ~y))
    eq(x | y, ~(~x & ~y))
    eq(x ^ y, (x | y) & ~(x & y))
    eq(x ^ y, x & ~y | ~x & y)
    eq(x ^ y, (x | y) & (~x | ~y))

def check_bitop_identities_3(x, y, z):
    eq = self_assertEqual
    eq(x & y & z, x & (y & z))
    eq(x | y | z, x | (y | z))
    eq(x ^ y ^ z, x ^ (y ^ z))
    eq(x & (y | z), x & y | x & z)
    eq(x | y & z, (x | y) & (x | z))

def check_division(x, y):
    eq = self_assertEqual
    q, r = divmod(x, y)
    q2, r2 = (x // y, x % y)
    pab, pba = (x * y, y * x)
    eq(pab, pba, 'multiplication does not commute')
    eq(q, q2, 'divmod returns different quotient than /')
    eq(r, r2, 'divmod returns different mod than %')
    eq(x, q * y + r, 'x != q*y + r after divmod')
    if y > 0:

        assert 0 <= r < y
    else:

        assert y < r <= 0

def check_float_conversion(n):
    try:
        actual = float(n)
    except OverflowError:
        actual = 'overflow'
    try:
        expected = int_to_float(n)
    except OverflowError:
        expected = 'overflow'
    msg = 'Error in conversion of integer {} to float.  Got {}, expected {}.'.format(n, actual, expected)

    assert actual == expected

def check_format_1(x):
    for base, mapper in ((2, bin), (8, oct), (10, str), (10, repr), (16, hex)):
        got = mapper(x)
        expected = slow_format(x, base)

        assert got == expected

        assert int(got, 0) == x

def check_truediv(a, b, skip_small=True):
    """Verify that the result of a/b is correctly rounded, by
        comparing it with a pure Python implementation of correctly
        rounded division.  b should be nonzero."""
    if skip_small and max(abs(a), abs(b)) < 2 ** DBL_MANT_DIG:
        return
    try:
        expected = repr(truediv(a, b))
    except OverflowError:
        expected = 'overflow'
    except ZeroDivisionError:
        expected = 'zerodivision'
    try:
        got = repr(a / b)
    except OverflowError:
        got = 'overflow'
    except ZeroDivisionError:
        got = 'zerodivision'

    assert expected == got

def getran(ndigits):

    assert ndigits > 0
    nbits_hi = ndigits * SHIFT
    nbits_lo = nbits_hi - SHIFT + 1
    answer = 0
    nbits = 0
    r = int(random.random() * (SHIFT * 2)) | 1
    while nbits < nbits_lo:
        bits = (r >> 1) + 1
        bits = min(bits, nbits_hi - nbits)

        assert 1 <= bits <= SHIFT
        nbits = nbits + bits
        answer = answer << bits
        if r & 1:
            answer = answer | (1 << bits) - 1
        r = int(random.random() * (SHIFT * 2))

    assert nbits_lo <= nbits <= nbits_hi
    if random.random() < 0.5:
        answer = -answer
    return answer

def slow_format(x, base):
    digits = []
    sign = 0
    if x < 0:
        sign, x = (1, -x)
    while x:
        x, r = divmod(x, base)
        digits.append(int(r))
    digits.reverse()
    digits = digits or [0]
    return '-'[:sign] + {2: '0b', 8: '0o', 10: '', 16: '0x'}[base] + ''.join(('0123456789abcdef'[i] for i in digits))
check_truediv(123, 0)
check_truediv(-456, 0)
check_truediv(0, 3)
check_truediv(0, -3)
check_truediv(0, 0)
check_truediv(671 * 12345 * 2 ** DBL_MAX_EXP, 12345)
check_truediv(12345, 345678 * 2 ** (DBL_MANT_DIG - DBL_MIN_EXP))
check_truediv(12345 * 2 ** 100, 98765)
check_truediv(12345 * 2 ** 30, 98765 * 7 ** 81)
bases = (0, DBL_MANT_DIG, DBL_MIN_EXP, DBL_MAX_EXP, DBL_MIN_EXP - DBL_MANT_DIG)
for base in bases:
    for exp in range(base - 15, base + 15):
        check_truediv(75312 * 2 ** max(exp, 0), 69187 * 2 ** max(-exp, 0))
        check_truediv(69187 * 2 ** max(exp, 0), 75312 * 2 ** max(-exp, 0))
for m in [1, 2, 7, 17, 12345, 7 ** 100, -1, -2, -5, -23, -67891, -41 ** 50]:
    for n in range(-10, 10):
        check_truediv(m * DBL_MIN_OVERFLOW + n, m)
        check_truediv(m * DBL_MIN_OVERFLOW + n, -m)
for n in range(250):
    check_truediv((2 ** DBL_MANT_DIG + 1) * 12345 * 2 ** 200 + 2 ** n, 2 ** DBL_MANT_DIG * 12345)
check_truediv(1, 2731)
check_truediv(295147931372582273023, 295147932265116303360)
for i in range(1000):
    check_truediv(10 ** (i + 1), 10 ** i)
    check_truediv(10 ** i, 10 ** (i + 1))
for m in [1, 2, 4, 7, 8, 16, 17, 32, 12345, 7 ** 100, -1, -2, -5, -23, -67891, -41 ** 50]:
    for n in range(-10, 10):
        check_truediv(2 ** DBL_MANT_DIG * m + n, m)
for n in range(-20, 20):
    check_truediv(n, 2 ** 1076)
for M in [10 ** 10, 10 ** 100, 10 ** 1000]:
    for i in range(1000):
        a = random.randrange(1, M)
        b = random.randrange(a, 2 * a + 1)
        check_truediv(a, b)
        check_truediv(-a, b)
        check_truediv(a, -b)
        check_truediv(-a, -b)
for _ in range(10000):
    a_bits = random.randrange(1000)
    b_bits = random.randrange(1, 1000)
    x = random.randrange(2 ** a_bits)
    y = random.randrange(1, 2 ** b_bits)
    check_truediv(x, y)
    check_truediv(x, -y)
    check_truediv(-x, y)
    check_truediv(-x, -y)
print("LongTest::test_correctly_rounded_true_division: ok")
