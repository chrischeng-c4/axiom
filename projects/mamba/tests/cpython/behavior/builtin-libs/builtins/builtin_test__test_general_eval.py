# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_general_eval"
# subject = "cpython.test_builtin.BuiltinTest.test_general_eval"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_general_eval
"""Auto-ported test: BuiltinTest::test_general_eval (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

class M:
    """Test mapping interface versus possible calls from eval()."""

    def __getitem__(self, key):
        if key == 'a':
            return 12
        raise KeyError

    def keys(self):
        return list('xyz')
m = M()
g = globals()

assert eval('a', g, m) == 12

try:
    eval('b', g, m)
    raise AssertionError('expected NameError')
except NameError:
    pass

assert eval('dir()', g, m) == list('xyz')

assert eval('globals()', g, m) == g

assert eval('locals()', g, m) == m

try:
    eval('a', m)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class A:
    """Non-mapping"""
    pass
m = A()

try:
    eval('a', g, m)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class D(dict):

    def __getitem__(self, key):
        if key == 'a':
            return 12
        return dict.__getitem__(self, key)

    def keys(self):
        return list('xyz')
d = D()

assert eval('a', g, d) == 12

try:
    eval('b', g, d)
    raise AssertionError('expected NameError')
except NameError:
    pass

assert eval('dir()', g, d) == list('xyz')

assert eval('globals()', g, d) == g

assert eval('locals()', g, d) == d
eval('[locals() for i in (2,3)]', g, d)
eval('[locals() for i in (2,3)]', g, collections.UserDict())

class SpreadSheet:
    """Sample application showing nested, calculated lookups."""
    _cells = {}

    def __setitem__(self, key, formula):
        self._cells[key] = formula

    def __getitem__(self, key):
        return eval(self._cells[key], globals(), self)
ss = SpreadSheet()
ss['a1'] = '5'
ss['a2'] = 'a1*6'
ss['a3'] = 'a2*7'

assert ss['a3'] == 210

class C:

    def __getitem__(self, item):
        raise KeyError(item)

    def keys(self):
        return 1

try:
    eval('dir()', globals(), C())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BuiltinTest::test_general_eval: ok")
