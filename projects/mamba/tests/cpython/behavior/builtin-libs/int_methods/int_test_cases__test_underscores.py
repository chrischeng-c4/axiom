# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_underscores"
# subject = "cpython.test_int.IntTestCases.test_underscores"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_underscores
"""Auto-ported test: IntTestCases::test_underscores (CPython 3.12 oracle)."""


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
for lit in VALID_UNDERSCORE_LITERALS:
    if any((ch in lit for ch in '.eEjJ')):
        continue

    assert int(lit, 0) == eval(lit)

    assert int(lit, 0) == int(lit.replace('_', ''), 0)
for lit in INVALID_UNDERSCORE_LITERALS:
    if any((ch in lit for ch in '.eEjJ')):
        continue

    try:
        int(lit, 0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

assert int('1_00', 3) == 9

assert int('0_100') == 100

assert int(b'1_00') == 100

try:
    int('_100')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('+_100')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('1__00')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('100_')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("IntTestCases::test_underscores: ok")
