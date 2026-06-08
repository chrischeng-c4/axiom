# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_map"
# subject = "cpython.test_builtin.BuiltinTest.test_map"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_map
"""Auto-ported test: BuiltinTest::test_map (CPython 3.12 oracle)."""


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

assert list(map(lambda x: x * x, range(1, 4))) == [1, 4, 9]
try:
    from math import sqrt
except ImportError:

    def sqrt(x):
        return pow(x, 0.5)

assert list(map(lambda x: list(map(sqrt, x)), [[16, 4], [81, 9]])) == [[4.0, 2.0], [9.0, 3.0]]

assert list(map(lambda x, y: x + y, [1, 3, 2], [9, 1, 4])) == [10, 4, 6]

def plus(*v):
    accu = 0
    for i in v:
        accu = accu + i
    return accu

assert list(map(plus, [1, 3, 7])) == [1, 3, 7]

assert list(map(plus, [1, 3, 7], [4, 9, 2])) == [1 + 4, 3 + 9, 7 + 2]

assert list(map(plus, [1, 3, 7], [4, 9, 2], [1, 1, 0])) == [1 + 4 + 1, 3 + 9 + 1, 7 + 2 + 0]

assert list(map(int, Squares(10))) == [0, 1, 4, 9, 16, 25, 36, 49, 64, 81]

def Max(a, b):
    if a is None:
        return b
    if b is None:
        return a
    return max(a, b)

assert list(map(Max, Squares(3), Squares(2))) == [0, 1]

try:
    map()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    map(lambda x: x, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class BadSeq:

    def __iter__(self):
        raise ValueError
        yield None

try:
    list(map(lambda x: x, BadSeq()))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

def badfunc(x):
    raise RuntimeError

try:
    list(map(badfunc, range(5)))
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("BuiltinTest::test_map: ok")
