# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_int_base_limits"
# subject = "cpython.test_int.IntTestCases.test_int_base_limits"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_int_base_limits
"""Auto-ported test: IntTestCases::test_int_base_limits (CPython 3.12 oracle)."""


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
"""Testing the supported limits of the int() base parameter."""

assert int('0', 5) == 0
try:
    int('0', 1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    int('0', 37)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    int('0', -909)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    int('0', base=0 - 2 ** 234)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    int('0', base=2 ** 234)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
for base in range(2, 37):

    assert int('0', base=base) == 0
print("IntTestCases::test_int_base_limits: ok")
