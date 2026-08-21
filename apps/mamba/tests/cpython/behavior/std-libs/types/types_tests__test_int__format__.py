# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_int__format__"
# subject = "cpython.test_types.TypesTests.test_int__format__"
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

def test(i, format_spec, result):
    assert type(i) is int
    assert type(format_spec) is str
    assert i.__format__(format_spec) == result
test(123456789, 'd', '123456789')
test(123456789, 'd', '123456789')
test(1, 'c', '\x01')
test(1, '-', '1')
test(-1, '-', '-1')
test(1, '-3', '  1')
test(-1, '-3', ' -1')
test(1, '+3', ' +1')
test(-1, '+3', ' -1')
test(1, ' 3', '  1')
test(-1, ' 3', ' -1')
test(1, ' ', ' 1')
test(-1, ' ', '-1')
test(3, 'x', '3')
test(3, 'X', '3')
test(1234, 'x', '4d2')
test(-1234, 'x', '-4d2')
test(1234, '8x', '     4d2')
test(-1234, '8x', '    -4d2')
test(1234, 'x', '4d2')
test(-1234, 'x', '-4d2')
test(-3, 'x', '-3')
test(-3, 'X', '-3')
test(int('be', 16), 'x', 'be')
test(int('be', 16), 'X', 'BE')
test(-int('be', 16), 'x', '-be')
test(-int('be', 16), 'X', '-BE')
test(3, 'o', '3')
test(-3, 'o', '-3')
test(65, 'o', '101')
test(-65, 'o', '-101')
test(1234, 'o', '2322')
test(-1234, 'o', '-2322')
test(1234, '-o', '2322')
test(-1234, '-o', '-2322')
test(1234, ' o', ' 2322')
test(-1234, ' o', '-2322')
test(1234, '+o', '+2322')
test(-1234, '+o', '-2322')
test(3, 'b', '11')
test(-3, 'b', '-11')
test(1234, 'b', '10011010010')
test(-1234, 'b', '-10011010010')
test(1234, '-b', '10011010010')
test(-1234, '-b', '-10011010010')
test(1234, ' b', ' 10011010010')
test(-1234, ' b', '-10011010010')
test(1234, '+b', '+10011010010')
test(-1234, '+b', '-10011010010')
test(0, '#b', '0b0')
test(0, '-#b', '0b0')
test(1, '-#b', '0b1')
test(-1, '-#b', '-0b1')
test(-1, '-#5b', ' -0b1')
test(1, '+#5b', ' +0b1')
test(100, '+#b', '+0b1100100')
test(100, '#012b', '0b0001100100')
test(-100, '#012b', '-0b001100100')
test(0, '#o', '0o0')
test(0, '-#o', '0o0')
test(1, '-#o', '0o1')
test(-1, '-#o', '-0o1')
test(-1, '-#5o', ' -0o1')
test(1, '+#5o', ' +0o1')
test(100, '+#o', '+0o144')
test(100, '#012o', '0o0000000144')
test(-100, '#012o', '-0o000000144')
test(0, '#x', '0x0')
test(0, '-#x', '0x0')
test(1, '-#x', '0x1')
test(-1, '-#x', '-0x1')
test(-1, '-#5x', ' -0x1')
test(1, '+#5x', ' +0x1')
test(100, '+#x', '+0x64')
test(100, '#012x', '0x0000000064')
test(-100, '#012x', '-0x000000064')
test(123456, '#012x', '0x000001e240')
test(-123456, '#012x', '-0x00001e240')
test(0, '#X', '0X0')
test(0, '-#X', '0X0')
test(1, '-#X', '0X1')
test(-1, '-#X', '-0X1')
test(-1, '-#5X', ' -0X1')
test(1, '+#5X', ' +0X1')
test(100, '+#X', '+0X64')
test(100, '#012X', '0X0000000064')
test(-100, '#012X', '-0X000000064')
test(123456, '#012X', '0X000001E240')
test(-123456, '#012X', '-0X00001E240')
test(123, ',', '123')
test(-123, ',', '-123')
test(1234, ',', '1,234')
test(-1234, ',', '-1,234')
test(123456, ',', '123,456')
test(-123456, ',', '-123,456')
test(1234567, ',', '1,234,567')
test(-1234567, ',', '-1,234,567')
test(1234, '010,', '00,001,234')
test(10 ** 100, 'd', '1' + '0' * 100)
test(10 ** 100 + 100, 'd', '1' + '0' * 97 + '100')
try:
    3 .__format__('1.3')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    3 .__format__('+c')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    3 .__format__(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    3 .__format__(0)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    3 .__format__(',n')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    3 .__format__(',c')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    3 .__format__('#c')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
for format_spec in [chr(x) for x in range(ord('a'), ord('z') + 1)] + [chr(x) for x in range(ord('A'), ord('Z') + 1)]:
    if not format_spec in 'bcdoxXeEfFgGn%':
        try:
            0 .__format__(format_spec)
            raise AssertionError('assertRaises: no raise')
        except ValueError:
            pass
        try:
            1 .__format__(format_spec)
            raise AssertionError('assertRaises: no raise')
        except ValueError:
            pass
        try:
            (-1).__format__(format_spec)
            raise AssertionError('assertRaises: no raise')
        except ValueError:
            pass
for format_spec in 'eEfFgG%':
    for value in [0, 1, -1, 100, -100, 1234567890, -1234567890]:
        assert value.__format__(format_spec) == float(value).__format__(format_spec)
test(123456, '0<20', '12345600000000000000')
test(123456, '1<20', '12345611111111111111')
test(123456, '*<20', '123456**************')
test(123456, '0>20', '00000000000000123456')
test(123456, '1>20', '11111111111111123456')
test(123456, '*>20', '**************123456')
test(123456, '0=20', '00000000000000123456')
test(123456, '1=20', '11111111111111123456')
test(123456, '*=20', '**************123456')

print("TypesTests::test_int__format__: ok")
