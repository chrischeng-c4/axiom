# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strtod"
# dimension = "behavior"
# case = "strtod_tests__test_large_exponents"
# subject = "cpython.test_strtod.StrtodTests.test_large_exponents"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strtod.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strtod.py::StrtodTests::test_large_exponents
"""Auto-ported test: StrtodTests::test_large_exponents (CPython 3.12 oracle)."""


import random
import unittest
import re
import sys
import test.support


if getattr(sys, 'float_repr_style', '') != 'short':
    raise unittest.SkipTest('correctly-rounded string->float conversions not available on this system')

strtod_parser = re.compile('    # A numeric string consists of:\n    (?P<sign>[-+])?          # an optional sign, followed by\n    (?=\\d|\\.\\d)              # a number with at least one digit\n    (?P<int>\\d*)             # having a (possibly empty) integer part\n    (?:\\.(?P<frac>\\d*))?     # followed by an optional fractional part\n    (?:E(?P<exp>[-+]?\\d+))?  # and an optional exponent\n    \\Z\n', re.VERBOSE | re.IGNORECASE).match

def strtod(s, mant_dig=53, min_exp=-1021, max_exp=1024):
    """Convert a finite decimal string to a hex string representing an
    IEEE 754 binary64 float.  Return 'inf' or '-inf' on overflow.
    This function makes no use of floating-point arithmetic at any
    stage."""
    m = strtod_parser(s)
    if m is None:
        raise ValueError('invalid numeric string')
    fraction = m.group('frac') or ''
    intpart = int(m.group('int') + fraction)
    exp = int(m.group('exp') or '0') - len(fraction)
    negative = m.group('sign') == '-'
    a, b = (intpart * 10 ** max(exp, 0), 10 ** max(0, -exp))
    if not a:
        return '-0x0.0p+0' if negative else '0x0.0p+0'
    d = a.bit_length() - b.bit_length()
    d += (a >> d if d >= 0 else a << -d) >= b
    e = max(d, min_exp) - mant_dig
    a, b = (a << max(-e, 0), b << max(e, 0))
    q, r = divmod(a, b)
    if 2 * r > b or (2 * r == b and q & 1):
        q += 1
        if q.bit_length() == mant_dig + 1:
            q //= 2
            e += 1
    assert q.bit_length() <= mant_dig and e >= min_exp - mant_dig
    assert q.bit_length() == mant_dig or e == min_exp - mant_dig
    if e + q.bit_length() > max_exp:
        return '-inf' if negative else 'inf'
    if not q:
        return '-0x0.0p+0' if negative else '0x0.0p+0'
    hexdigs = 1 + (mant_dig - 2) // 4
    shift = 3 - (mant_dig - 2) % 4
    q, e = (q << shift, e - shift)
    return '{}0x{:x}.{:0{}x}p{:+d}'.format('-' if negative else '', q // 16 ** hexdigs, q % 16 ** hexdigs, hexdigs, e + 4 * hexdigs)

TEST_SIZE = 10


# --- test body ---
def positive_exp(n):
    """ Long string with value 1.0 and exponent n"""
    return '0.{}1e+{}'.format('0' * (n - 1), n)

def negative_exp(n):
    """ Long string with value 1.0 and exponent -n"""
    return '1{}e-{}'.format('0' * n, n)

assert float(positive_exp(10000)) == 1.0

assert float(positive_exp(20000)) == 1.0

assert float(positive_exp(30000)) == 1.0

assert float(negative_exp(10000)) == 1.0

assert float(negative_exp(20000)) == 1.0

assert float(negative_exp(30000)) == 1.0
print("StrtodTests::test_large_exponents: ok")
