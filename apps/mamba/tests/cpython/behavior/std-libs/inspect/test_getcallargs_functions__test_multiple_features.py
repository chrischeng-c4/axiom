# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_getcallargs_functions__test_multiple_features"
# subject = "cpython.test_inspect.TestGetcallargsFunctions.test_multiple_features"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_inspect.py::TestGetcallargsFunctions::test_multiple_features
"""Auto-ported test: TestGetcallargsFunctions::test_multiple_features (CPython 3.12 oracle)."""


import asyncio
import builtins
import collections
import datetime
import functools
import gc
import importlib
import inspect
import io
import linecache
import os
import dis
from os.path import normcase
import _pickle
import pickle
import shutil
import sys
import types
import textwrap
from typing import Unpack
import unicodedata
import unittest
import unittest.mock
import warnings
import weakref
from test.support import cpython_only
from test.support import MISSING_C_DOCSTRINGS, ALWAYS_EQ
from test.support.import_helper import DirsOnSysPath, ready_to_import
from test.support.os_helper import TESTFN
from test.support.script_helper import assert_python_ok, assert_python_failure
from test import support
from test.test_inspect import inspect_fodder as mod
from test.test_inspect import inspect_fodder2 as mod2
from test.test_inspect import inspect_stock_annotations
from test.test_inspect import inspect_stringized_annotations
from test.test_inspect import inspect_stringized_annotations_2
from test.test_inspect import inspect_stringized_annotations_pep695


try:
    from concurrent.futures import ThreadPoolExecutor
except ImportError:
    ThreadPoolExecutor = None

modfile = mod.__file__

if modfile.endswith(('c', 'o')):
    modfile = modfile[:-1]

modfile = normcase(modfile)

def revise(filename, *args):
    return (normcase(filename),) + args

git = mod.StupidGit()

def tearDownModule():
    if support.has_socket_support:
        asyncio.set_event_loop_policy(None)

def signatures_with_lexicographic_keyword_only_parameters():
    """
    Yields a whole bunch of functions with only keyword-only parameters,
    where those parameters are always in lexicographically sorted order.
    """
    parameters = ['a', 'bar', 'c', 'delta', 'ephraim', 'magical', 'yoyo', 'z']
    for i in range(1, 2 ** len(parameters)):
        p = []
        bit = 1
        for j in range(len(parameters)):
            if i & bit << j:
                p.append(parameters[j])
        fn_text = 'def foo(*, ' + ', '.join(p) + '): pass'
        symbols = {}
        exec(fn_text, symbols, symbols)
        yield symbols['foo']

def unsorted_keyword_only_parameters_fn(*, throw, out, the, baby, with_, the_, bathwater):
    pass

unsorted_keyword_only_parameters = 'throw out the baby with_ the_ bathwater'.split()

def generator_function_example(self):
    for i in range(2):
        yield i

async def async_generator_function_example(self):
    async for i in range(2):
        yield i

async def coroutine_function_example(self):
    return 'spam'

@types.coroutine
def gen_coroutine_function_example(self):
    yield
    return 'spam'

class SlotUser:
    """Docstrings for __slots__"""
    __slots__ = {'power': 'measured in kilowatts', 'distance': 'measured in kilometers'}

class _BrokenDataDescriptor(object):
    """
    A broken data descriptor. See bug #1785.
    """

    def __get__(*args):
        raise AttributeError('broken data descriptor')

    def __set__(*args):
        raise RuntimeError

    def __getattr__(*args):
        raise AttributeError('broken data descriptor')

class _BrokenMethodDescriptor(object):
    """
    A broken method descriptor. See bug #1785.
    """

    def __get__(*args):
        raise AttributeError('broken method descriptor')

    def __getattr__(*args):
        raise AttributeError('broken method descriptor')

def attrs_wo_objs(cls):
    return [t[:3] for t in inspect.classify_class_attrs(cls)]

_global_ref = object()

class MySignature(inspect.Signature):
    pass

class MyParameter(inspect.Parameter):
    pass

class NTimesUnwrappable:

    def __init__(self, n):
        self.n = n
        self._next = None

    @property
    def __wrapped__(self):
        if self.n <= 0:
            raise Exception('Unwrapped too many times')
        if self._next is None:
            self._next = NTimesUnwrappable(self.n - 1)
        return self._next


# --- test body ---
def assertEqualCallArgs(func, call_params_string, locs=None):
    locs = dict(locs or {}, func=func)
    r1 = eval('func(%s)' % call_params_string, None, locs)
    r2 = eval('inspect.getcallargs(func, %s)' % call_params_string, None, locs)

    assert r1 == r2

def assertEqualException(func, call_param_string, locs=None):
    locs = dict(locs or {}, func=func)
    try:
        eval('func(%s)' % call_param_string, None, locs)
    except Exception as e:
        ex1 = e
    else:

        raise AssertionError('Exception not raised')
    try:
        eval('inspect.getcallargs(func, %s)' % call_param_string, None, locs)
    except Exception as e:
        ex2 = e
    else:

        raise AssertionError('Exception not raised')

    assert type(ex1) is type(ex2)

    assert str(ex1) == str(ex2)
    del ex1, ex2

def makeCallable(signature):
    """Create a function that returns its locals()"""
    code = 'lambda %s: locals()'
    return eval(code % signature)
f = makeCallable('a, b=2, *f, **g')
assertEqualCallArgs(f, '2, 3, 7')
assertEqualCallArgs(f, '2, 3, x=8')
assertEqualCallArgs(f, '2, 3, x=8, *[(4,[5,6]), 7]')
assertEqualCallArgs(f, '2, x=8, *[3, (4,[5,6]), 7], y=9')
assertEqualCallArgs(f, 'x=8, *[2, 3, (4,[5,6])], y=9')
assertEqualCallArgs(f, 'x=8, *collections.UserList([2, 3, (4,[5,6])]), **{"y":9, "z":10}')
assertEqualCallArgs(f, '2, x=8, *collections.UserList([3, (4,[5,6])]), **collections.UserDict(y=9, z=10)')
f = makeCallable('a, b=2, *f, x, y=99, **g')
assertEqualCallArgs(f, '2, 3, x=8')
assertEqualCallArgs(f, '2, 3, x=8, *[(4,[5,6]), 7]')
assertEqualCallArgs(f, '2, x=8, *[3, (4,[5,6]), 7], y=9, z=10')
assertEqualCallArgs(f, 'x=8, *[2, 3, (4,[5,6])], y=9, z=10')
assertEqualCallArgs(f, 'x=8, *collections.UserList([2, 3, (4,[5,6])]), q=0, **{"y":9, "z":10}')
assertEqualCallArgs(f, '2, x=8, *collections.UserList([3, (4,[5,6])]), q=0, **collections.UserDict(y=9, z=10)')
print("TestGetcallargsFunctions::test_multiple_features: ok")
