# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_max"
# subject = "cpython.test_builtin.BuiltinTest.test_max"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_max
"""Auto-ported test: BuiltinTest::test_max (CPython 3.12 oracle)."""


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

assert max('123123') == '3'

assert max(1, 2, 3) == 3

assert max((1, 2, 3, 1, 2, 3)) == 3

assert max([1, 2, 3, 1, 2, 3]) == 3

assert max(1, 2, 3.0) == 3.0

assert max(1, 2.0, 3) == 3

assert max(1.0, 2, 3) == 3
try:
    max()
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('max expected at least 1 argument, got 0', str(_aR_e))

try:
    max(42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    max(())
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('max\\(\\) iterable argument is empty', str(_aR_e))

class BadSeq:

    def __getitem__(self, index):
        raise ValueError

try:
    max(BadSeq())
    raise AssertionError('expected ValueError')
except ValueError:
    pass
for stmt in ('max(key=int)', 'max(default=None)', 'max(1, 2, default=None)', 'max(default=None, key=int)', 'max(1, key=int)', 'max(1, 2, keystone=int)', 'max(1, 2, key=int, abc=int)', 'max(1, 2, key=1)'):
    try:
        exec(stmt, globals())
    except TypeError:
        pass
    else:

        raise AssertionError(stmt)

assert max((1,), key=neg) == 1

assert max((1, 2), key=neg) == 1

assert max(1, 2, key=neg) == 1

assert max((), default=None) == None

assert max((1,), default=None) == 1

assert max((1, 2), default=None) == 2

assert max((), default=1, key=neg) == 1

assert max((1, 2), default=3, key=neg) == 1

assert max((1, 2), key=None) == 2
data = [random.randrange(200) for i in range(100)]
keys = dict(((elem, random.randrange(50)) for elem in data))
f = keys.__getitem__

assert max(data, key=f) == sorted(reversed(data), key=f)[-1]
print("BuiltinTest::test_max: ok")
