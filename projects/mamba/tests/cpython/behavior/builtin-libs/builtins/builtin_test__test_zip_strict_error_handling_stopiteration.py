# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_zip_strict_error_handling_stopiteration"
# subject = "cpython.test_builtin.BuiltinTest.test_zip_strict_error_handling_stopiteration"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_zip_strict_error_handling_stopiteration
"""Auto-ported test: BuiltinTest::test_zip_strict_error_handling_stopiteration (CPython 3.12 oracle)."""


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

def check_iter_pickle(it, seq, proto):
    itorg = it
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert type(itorg) == type(it)

    assert list(it) == seq
    it = pickle.loads(d)
    try:
        next(it)
    except StopIteration:
        return
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == seq[1:]

def get_vars_f0():
    return vars()

def get_vars_f2():
    BuiltinTest.get_vars_f0()
    a = 1
    b = 2
    return vars()

def iter_error(iterable, error):
    """Collect `iterable` into a list, catching an expected `error`."""
    items = []
    try:
        for item in iterable:
            items.append(item)
        raise AssertionError('expected error')
    except error:
        pass
    return items

def write_testfile():
    fp = open(TESTFN, 'w', encoding='utf-8')
    pass
    with fp:
        fp.write('1+1\n')
        fp.write('The quick brown fox jumps over the lazy dog')
        fp.write('.\n')
        fp.write('Dear John\n')
        fp.write('XXX' * 100)
        fp.write('YYY' * 100)

class Iter:

    def __init__(self, size):
        self.size = size

    def __iter__(self):
        return self

    def __next__(self):
        self.size -= 1
        if self.size < 0:
            raise StopIteration
        return self.size
l1 = iter_error(zip('AB', Iter(1), strict=True), ValueError)

assert l1 == [('A', 0)]
l2 = iter_error(zip('AB', Iter(2), 'A', strict=True), ValueError)

assert l2 == [('A', 1, 'A')]
l3 = iter_error(zip('AB', Iter(2), 'ABC', strict=True), ValueError)

assert l3 == [('A', 1, 'A'), ('B', 0, 'B')]
l4 = iter_error(zip('AB', Iter(3), strict=True), ValueError)

assert l4 == [('A', 2), ('B', 1)]
l5 = iter_error(zip(Iter(1), 'AB', strict=True), ValueError)

assert l5 == [(0, 'A')]
l6 = iter_error(zip(Iter(2), 'A', strict=True), ValueError)

assert l6 == [(1, 'A')]
l7 = iter_error(zip(Iter(2), 'ABC', strict=True), ValueError)

assert l7 == [(1, 'A'), (0, 'B')]
l8 = iter_error(zip(Iter(3), 'AB', strict=True), ValueError)

assert l8 == [(2, 'A'), (1, 'B')]
print("BuiltinTest::test_zip_strict_error_handling_stopiteration: ok")
