# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_pow"
# subject = "cpython.test_builtin.BuiltinTest.test_pow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_pow
"""Auto-ported test: BuiltinTest::test_pow (CPython 3.12 oracle)."""


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

assert pow(0, 0) == 1

assert pow(0, 1) == 0

assert pow(1, 0) == 1

assert pow(1, 1) == 1

assert pow(2, 0) == 1

assert pow(2, 10) == 1024

assert pow(2, 20) == 1024 * 1024

assert pow(2, 30) == 1024 * 1024 * 1024

assert pow(-2, 0) == 1

assert pow(-2, 1) == -2

assert pow(-2, 2) == 4

assert pow(-2, 3) == -8

assert abs(pow(0.0, 0) - 1.0) < 1e-07

assert abs(pow(0.0, 1) - 0.0) < 1e-07

assert abs(pow(1.0, 0) - 1.0) < 1e-07

assert abs(pow(1.0, 1) - 1.0) < 1e-07

assert abs(pow(2.0, 0) - 1.0) < 1e-07

assert abs(pow(2.0, 10) - 1024.0) < 1e-07

assert abs(pow(2.0, 20) - 1024.0 * 1024.0) < 1e-07

assert abs(pow(2.0, 30) - 1024.0 * 1024.0 * 1024.0) < 1e-07

assert abs(pow(-2.0, 0) - 1.0) < 1e-07

assert abs(pow(-2.0, 1) - -2.0) < 1e-07

assert abs(pow(-2.0, 2) - 4.0) < 1e-07

assert abs(pow(-2.0, 3) - -8.0) < 1e-07
for x in (2, 2.0):
    for y in (10, 10.0):
        for z in (1000, 1000.0):
            if isinstance(x, float) or isinstance(y, float) or isinstance(z, float):

                try:
                    pow(x, y, z)
                    raise AssertionError('expected TypeError')
                except TypeError:
                    pass
            else:

                assert abs(pow(x, y, z) - 24.0) < 1e-07

assert abs(pow(-1, 0.5) - 1j) < 1e-07

assert abs(pow(-1, 1 / 3) - (0.5 + 0.8660254037844386j)) < 1e-07

assert pow(-1, -2, 3) == 1

try:
    pow(1, 2, 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    pow()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert pow(0, exp=0) == 1

assert pow(base=2, exp=4) == 16

assert pow(base=5, exp=2, mod=14) == 11
twopow = partial(pow, base=2)

assert twopow(exp=5) == 32
fifth_power = partial(pow, exp=5)

assert fifth_power(2) == 32
mod10 = partial(pow, mod=10)

assert mod10(2, 6) == 4

assert mod10(exp=6, base=2) == 4
print("BuiltinTest::test_pow: ok")
