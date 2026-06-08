# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_basic"
# subject = "cpython.test_int.IntTestCases.test_basic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_basic
"""Auto-ported test: IntTestCases::test_basic (CPython 3.12 oracle)."""


import sys
import time
import unittest
from unittest import mock
from test import support
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS


try:
    import _pylong
except ImportError:
    _pylong = None

L = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), ('Ȁ', ValueError)]

class IntSubclass(int):
    pass


# --- test body ---

assert int(314) == 314

assert int(3.14) == 3

assert int(-3.14) == -3

assert int(3.9) == 3

assert int(-3.9) == -3

assert int(3.5) == 3

assert int(-3.5) == -3

assert int('-3') == -3

assert int(' -3 ') == -3

assert int('\u2003-3\u2002') == -3

assert int('10', 16) == 16
for s, v in L:
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
s = repr(-1 - sys.maxsize)
x = int(s)

assert x + 1 == -sys.maxsize

assert isinstance(x, int)

assert int(s[1:]) == sys.maxsize + 1
x = int(1e+100)

assert isinstance(x, int)
x = int(-1e+100)

assert isinstance(x, int)
x = -1 - sys.maxsize

assert x >> 1 == x // 2
x = int('1' * 600)

assert isinstance(x, int)

try:
    int(1, 12)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert int('0o123', 0) == 83

assert int('0x123', 16) == 291

try:
    int('0x', 16)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0x', 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0o', 8)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0o', 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0b', 2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0b', 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert int('100000000000000000000000000000000', 2) == 4294967296

assert int('102002022201221111211', 3) == 4294967296

assert int('10000000000000000', 4) == 4294967296

assert int('32244002423141', 5) == 4294967296

assert int('1550104015504', 6) == 4294967296

assert int('211301422354', 7) == 4294967296

assert int('40000000000', 8) == 4294967296

assert int('12068657454', 9) == 4294967296

assert int('4294967296', 10) == 4294967296

assert int('1904440554', 11) == 4294967296

assert int('9ba461594', 12) == 4294967296

assert int('535a79889', 13) == 4294967296

assert int('2ca5b7464', 14) == 4294967296

assert int('1a20dcd81', 15) == 4294967296

assert int('100000000', 16) == 4294967296

assert int('a7ffda91', 17) == 4294967296

assert int('704he7g4', 18) == 4294967296

assert int('4f5aff66', 19) == 4294967296

assert int('3723ai4g', 20) == 4294967296

assert int('281d55i4', 21) == 4294967296

assert int('1fj8b184', 22) == 4294967296

assert int('1606k7ic', 23) == 4294967296

assert int('mb994ag', 24) == 4294967296

assert int('hek2mgl', 25) == 4294967296

assert int('dnchbnm', 26) == 4294967296

assert int('b28jpdm', 27) == 4294967296

assert int('8pfgih4', 28) == 4294967296

assert int('76beigg', 29) == 4294967296

assert int('5qmcpqg', 30) == 4294967296

assert int('4q0jto4', 31) == 4294967296

assert int('4000000', 32) == 4294967296

assert int('3aokq94', 33) == 4294967296

assert int('2qhxjli', 34) == 4294967296

assert int('2br45qb', 35) == 4294967296

assert int('1z141z4', 36) == 4294967296

assert int(' 0o123  ', 0) == 83

assert int(' 0o123  ', 0) == 83

assert int('000', 0) == 0

assert int('0o123', 0) == 83

assert int('0x123', 0) == 291

assert int('0b100', 0) == 4

assert int(' 0O123   ', 0) == 83

assert int(' 0X123  ', 0) == 291

assert int(' 0B100 ', 0) == 4
try:
    int('010', 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert int('0123') == 123

assert int('0123', 10) == 123

assert int('0x123', 16) == 291

assert int('0o123', 8) == 83

assert int('0b100', 2) == 4

assert int('0X123', 16) == 291

assert int('0O123', 8) == 83

assert int('0B100', 2) == 4

try:
    int('0b2', 2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0b02', 2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0B2', 2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0B02', 2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0o8', 8)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0o08', 8)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0O8', 8)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0O08', 8)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0xg', 16)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0x0g', 16)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0Xg', 16)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('0X0g', 16)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert int('100000000000000000000000000000001', 2) == 4294967297

assert int('102002022201221111212', 3) == 4294967297

assert int('10000000000000001', 4) == 4294967297

assert int('32244002423142', 5) == 4294967297

assert int('1550104015505', 6) == 4294967297

assert int('211301422355', 7) == 4294967297

assert int('40000000001', 8) == 4294967297

assert int('12068657455', 9) == 4294967297

assert int('4294967297', 10) == 4294967297

assert int('1904440555', 11) == 4294967297

assert int('9ba461595', 12) == 4294967297

assert int('535a7988a', 13) == 4294967297

assert int('2ca5b7465', 14) == 4294967297

assert int('1a20dcd82', 15) == 4294967297

assert int('100000001', 16) == 4294967297

assert int('a7ffda92', 17) == 4294967297

assert int('704he7g5', 18) == 4294967297

assert int('4f5aff67', 19) == 4294967297

assert int('3723ai4h', 20) == 4294967297

assert int('281d55i5', 21) == 4294967297

assert int('1fj8b185', 22) == 4294967297

assert int('1606k7id', 23) == 4294967297

assert int('mb994ah', 24) == 4294967297

assert int('hek2mgm', 25) == 4294967297

assert int('dnchbnn', 26) == 4294967297

assert int('b28jpdn', 27) == 4294967297

assert int('8pfgih5', 28) == 4294967297

assert int('76beigh', 29) == 4294967297

assert int('5qmcpqh', 30) == 4294967297

assert int('4q0jto5', 31) == 4294967297

assert int('4000001', 32) == 4294967297

assert int('3aokq95', 33) == 4294967297

assert int('2qhxjlj', 34) == 4294967297

assert int('2br45qc', 35) == 4294967297

assert int('1z141z5', 36) == 4294967297
print("IntTestCases::test_basic: ok")
