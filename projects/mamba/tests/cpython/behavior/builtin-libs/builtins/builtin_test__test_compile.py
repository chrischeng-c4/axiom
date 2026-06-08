# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_compile"
# subject = "cpython.test_builtin.BuiltinTest.test_compile"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_compile
"""Auto-ported test: BuiltinTest::test_compile (CPython 3.12 oracle)."""


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
compile('print(1)\n', '', 'exec')
bom = b'\xef\xbb\xbf'
compile(bom + b'print(1)\n', '', 'exec')
compile(source='pass', filename='?', mode='exec')
compile(dont_inherit=False, filename='tmp', source='0', mode='eval')
compile('pass', '?', dont_inherit=True, mode='exec')
compile(memoryview(b'text'), 'name', 'exec')

try:
    compile()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    compile('print(42)\n', '<string>', 'badmode')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    compile('print(42)\n', '<string>', 'single', 255)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    compile('pass', '?', 'exec', mode='eval', source='0', filename='tmp')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
compile('print("å")\n', '', 'exec')

try:
    compile(chr(0), 'f', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    compile(str('a = 1'), 'f', 'bad')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
codestr = 'def f():\n        """doc"""\n        debug_enabled = False\n        if __debug__:\n            debug_enabled = True\n        try:\n            assert False\n        except AssertionError:\n            return (True, f.__doc__, debug_enabled, __debug__)\n        else:\n            return (False, f.__doc__, debug_enabled, __debug__)\n        '

def f():
    """doc"""
values = [(-1, __debug__, f.__doc__, __debug__, __debug__), (0, True, 'doc', True, True), (1, False, 'doc', False, False), (2, False, None, False, False)]
for optval, *expected in values:
    codeobjs = []
    codeobjs.append(compile(codestr, '<test>', 'exec', optimize=optval))
    tree = ast.parse(codestr)
    codeobjs.append(compile(tree, '<test>', 'exec', optimize=optval))
    for code in codeobjs:
        ns = {}
        exec(code, ns)
        rv = ns['f']()

        assert rv == tuple(expected)
print("BuiltinTest::test_compile: ok")
