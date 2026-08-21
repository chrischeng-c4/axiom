# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_dir"
# subject = "cpython.test_builtin.BuiltinTest.test_dir"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_dir
"""Auto-ported test: BuiltinTest::test_dir (CPython 3.12 oracle)."""


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

try:
    dir(42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
local_var = 1

assert 'local_var' in dir()

assert 'exit' in dir(sys)

class Foo(types.ModuleType):
    __dict__ = 8
f = Foo('foo')

try:
    dir(f)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert 'strip' in dir(str)

assert '__mro__' not in dir(str)

class Foo(object):

    def __init__(self):
        self.x = 7
        self.y = 8
        self.z = 9
f = Foo()

assert 'y' in dir(f)

class Foo(object):
    __slots__ = []
f = Foo()

assert '__repr__' in dir(f)

class Foo(object):
    __slots__ = ['__class__', '__dict__']

    def __init__(self):
        self.bar = 'wow'
f = Foo()

assert '__repr__' not in dir(f)

assert 'bar' in dir(f)

class Foo(object):

    def __dir__(self):
        return ['kan', 'ga', 'roo']
f = Foo()

assert dir(f) == ['ga', 'kan', 'roo']

class Foo(object):

    def __dir__(self):
        return ('b', 'c', 'a')
res = dir(Foo())

assert isinstance(res, list)

assert res == ['a', 'b', 'c']

class Foo(object):

    def __dir__(self):
        return {'b', 'c', 'a'}
res = dir(Foo())

assert isinstance(res, list)

assert sorted(res) == ['a', 'b', 'c']

class Foo(object):

    def __dir__(self):
        return 7
f = Foo()

try:
    dir(f)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    raise IndexError
except IndexError as e:

    assert len(dir(e.__traceback__)) == 4

assert sorted([].__dir__()) == dir([])
print("BuiltinTest::test_dir: ok")
