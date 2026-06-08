# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_int_base_indexable"
# subject = "cpython.test_int.IntTestCases.test_int_base_indexable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_int_base_indexable
"""Auto-ported test: IntTestCases::test_int_base_indexable (CPython 3.12 oracle)."""


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
class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value
for base in (2 ** 100, -2 ** 100, 1, 37):
    try:
        int('43', base)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

assert int('101', base=MyIndexable(2)) == 5

assert int('101', base=MyIndexable(10)) == 101

assert int('101', base=MyIndexable(36)) == 1 + 36 ** 2
print("IntTestCases::test_int_base_indexable: ok")
