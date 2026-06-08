# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_sum"
# subject = "cpython.test_builtin.BuiltinTest.test_sum"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_sum
"""Auto-ported test: BuiltinTest::test_sum (CPython 3.12 oracle)."""


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

assert sum([]) == 0

assert sum(list(range(2, 8))) == 27

assert sum(iter(list(range(2, 8)))) == 27

assert sum(Squares(10)) == 285

assert sum(iter(Squares(10))) == 285

assert sum([[1], [2], [3]], []) == [1, 2, 3]

assert sum(range(10), 1000) == 1045

assert sum(range(10), start=1000) == 1045

assert sum(range(10), 2 ** 31 - 5) == 2 ** 31 + 40

assert sum(range(10), 2 ** 63 - 5) == 2 ** 63 + 40

assert sum((i % 2 != 0 for i in range(10))) == 5

assert sum((i % 2 != 0 for i in range(10)), 2 ** 31 - 3) == 2 ** 31 + 2

assert sum((i % 2 != 0 for i in range(10)), 2 ** 63 - 3) == 2 ** 63 + 2

assert sum([], False) is False

assert sum((i / 2 for i in range(10))) == 22.5

assert sum((i / 2 for i in range(10)), 1000) == 1022.5

assert sum((i / 2 for i in range(10)), 1000.25) == 1022.75

assert sum([0.5, 1]) == 1.5

assert sum([1, 0.5]) == 1.5

assert repr(sum([-0.0])) == '0.0'

assert repr(sum([-0.0], -0.0)) == '-0.0'

assert repr(sum([], -0.0)) == '-0.0'

assert math.isinf(sum([float('inf'), float('inf')]))

assert math.isinf(sum([1e+308, 1e+308]))

try:
    sum()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum(42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum(['a', 'b', 'c'])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum(['a', 'b', 'c'], '')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum([b'a', b'c'], b'')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
values = [bytearray(b'a'), bytearray(b'b')]

try:
    sum(values, bytearray(b''))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum([[1], [2], [3]])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum([{2: 3}])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum([{2: 3}] * 2, {2: 3})
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum([], '')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum([], b'')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sum([], bytearray())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class BadSeq:

    def __getitem__(self, index):
        raise ValueError

try:
    sum(BadSeq())
    raise AssertionError('expected ValueError')
except ValueError:
    pass
empty = []
sum(([x] for x in range(10)), empty)

assert empty == []
print("BuiltinTest::test_sum: ok")
