# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_keyword_args"
# subject = "cpython.test_int.IntTestCases.test_keyword_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_keyword_args
"""Auto-ported test: IntTestCases::test_keyword_args (CPython 3.12 oracle)."""


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

assert int('100', base=2) == 4
try:
    int(x=1.2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('keyword argument', str(_aR_e))
try:
    int(x='100', base=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('keyword argument', str(_aR_e))

try:
    int(base=10)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    int(base=0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("IntTestCases::test_keyword_args: ok")
