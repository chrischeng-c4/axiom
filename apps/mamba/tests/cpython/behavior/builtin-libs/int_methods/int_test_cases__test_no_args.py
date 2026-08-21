# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_no_args"
# subject = "cpython.test_int.IntTestCases.test_no_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_no_args
"""Auto-ported test: IntTestCases::test_no_args (CPython 3.12 oracle)."""


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

assert int() == 0
print("IntTestCases::test_no_args: ok")
