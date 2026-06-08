# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strtod"
# dimension = "behavior"
# case = "strtod_tests__test_boundaries"
# subject = "cpython.test_strtod.StrtodTests.test_boundaries"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strtod.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strtod.py::StrtodTests::test_boundaries
"""Auto-ported test: StrtodTests::test_boundaries (CPython 3.12 oracle)."""


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
def check_strtod(s):
    """Compare the result of Python's builtin correctly rounded
        string->float conversion (using float) to a pure Python
        correctly rounded string->float implementation.  Fail if the
        two methods give different results."""
    try:
        fs = float(s)
    except OverflowError:
        got = '-inf' if s[0] == '-' else 'inf'
    except MemoryError:
        got = 'memory error'
    else:
        got = fs.hex()
    expected = strtod(s)

    assert expected == got
boundaries = [(10000000000000000000, -19, 1110), (17976931348623159077, 289, 1995), (22250738585072013831, -327, 4941), (0, -327, 4941)]
for n, e, u in boundaries:
    for j in range(1000):
        digits = n + random.randrange(-3 * u, 3 * u)
        exponent = e
        s = '{}e{}'.format(digits, exponent)
        check_strtod(s)
        n *= 10
        u *= 10
        e -= 1
print("StrtodTests::test_boundaries: ok")
