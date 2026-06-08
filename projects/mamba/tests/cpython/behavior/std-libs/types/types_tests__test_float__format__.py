# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_float__format__"
# subject = "cpython.test_types.TypesTests.test_float__format__"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import collections.abc
from collections import namedtuple
import copy
import gc
import inspect
import pickle
import locale
import sys
import textwrap
import types
import weakref
import typing

def test(f, format_spec, result):
    assert f.__format__(format_spec) == result
    assert format(f, format_spec) == result
test(0.0, 'f', '0.000000')
test(0.0, '', '0.0')
test(0.01, '', '0.01')
test(0.01, 'g', '0.01')
test(1.23, '1', '1.23')
test(-1.23, '1', '-1.23')
test(1.23, '1g', '1.23')
test(-1.23, '1g', '-1.23')
test(1.0, ' g', ' 1')
test(-1.0, ' g', '-1')
test(1.0, '+g', '+1')
test(-1.0, '+g', '-1')
test(1.1234e+200, 'g', '1.1234e+200')
test(1.1234e+200, 'G', '1.1234E+200')
test(1.0, 'f', '1.000000')
test(-1.0, 'f', '-1.000000')
test(1.0, ' f', ' 1.000000')
test(-1.0, ' f', '-1.000000')
test(1.0, '+f', '+1.000000')
test(-1.0, '+f', '-1.000000')
f = 1.1234e+90
for fmt in ('f', 'F'):
    result = f.__format__(fmt)
    assert len(result) == 98
    assert result[-7] == '.'
    assert result[:12] in ('112340000000', '112339999999')
f = 1.1234e+200
for fmt in ('f', 'F'):
    result = f.__format__(fmt)
    assert len(result) == 208
    assert result[-7] == '.'
    assert result[:12] in ('112340000000', '112339999999')
test(1.0, 'e', '1.000000e+00')
test(-1.0, 'e', '-1.000000e+00')
test(1.0, 'E', '1.000000E+00')
test(-1.0, 'E', '-1.000000E+00')
test(1.1234e+20, 'e', '1.123400e+20')
test(1.1234e+20, 'E', '1.123400E+20')
test(1e+200, '+g', '+1e+200')
test(1e+200, '+', '+1e+200')
test(1.1e+200, '+g', '+1.1e+200')
test(1.1e+200, '+', '+1.1e+200')
test(1234.0, '010f', '1234.000000')
test(1234.0, '011f', '1234.000000')
test(1234.0, '012f', '01234.000000')
test(-1234.0, '011f', '-1234.000000')
test(-1234.0, '012f', '-1234.000000')
test(-1234.0, '013f', '-01234.000000')
test(-1234.12341234, '013f', '-01234.123412')
test(-123456.12341234, '011.2f', '-0123456.12')
test(1.2, '010,.2', '0,000,001.2')
test(1234.0, '011,f', '1,234.000000')
test(1234.0, '012,f', '1,234.000000')
test(1234.0, '013,f', '01,234.000000')
test(-1234.0, '012,f', '-1,234.000000')
test(-1234.0, '013,f', '-1,234.000000')
test(-1234.0, '014,f', '-01,234.000000')
test(-12345.0, '015,f', '-012,345.000000')
test(-123456.0, '016,f', '-0,123,456.000000')
test(-123456.0, '017,f', '-0,123,456.000000')
test(-123456.12341234, '017,f', '-0,123,456.123412')
test(-123456.12341234, '013,.2f', '-0,123,456.12')
test(-1.0, '%', '-100.000000%')
try:
    3.0.__format__(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    3.0.__format__(0)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
for format_spec in 'sbcdoxX':
    try:
        format(0.0, format_spec)
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        format(1.0, format_spec)
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        format(-1.0, format_spec)
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        format(1e+100, format_spec)
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        format(-1e+100, format_spec)
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        format(1e-100, format_spec)
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        format(-1e-100, format_spec)
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
test(1.0, '.0e', '1e+00')
test(1.0, '#.0e', '1.e+00')
test(1.0, '.0f', '1')
test(1.0, '#.0f', '1.')
test(1.1, 'g', '1.1')
test(1.1, '#g', '1.10000')
test(1.0, '.0%', '100%')
test(1.0, '#.0%', '100.%')
test(1.0, '0e', '1.000000e+00')
test(1.0, '#0e', '1.000000e+00')
test(1.0, '0f', '1.000000')
test(1.0, '#0f', '1.000000')
test(1.0, '.1e', '1.0e+00')
test(1.0, '#.1e', '1.0e+00')
test(1.0, '.1f', '1.0')
test(1.0, '#.1f', '1.0')
test(1.0, '.1%', '100.0%')
test(1.0, '#.1%', '100.0%')
test(12345.6, '0<20', '12345.60000000000000')
test(12345.6, '1<20', '12345.61111111111111')
test(12345.6, '*<20', '12345.6*************')
test(12345.6, '0>20', '000000000000012345.6')
test(12345.6, '1>20', '111111111111112345.6')
test(12345.6, '*>20', '*************12345.6')
test(12345.6, '0=20', '000000000000012345.6')
test(12345.6, '1=20', '111111111111112345.6')
test(12345.6, '*=20', '*************12345.6')

print("TypesTests::test_float__format__: ok")
