# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_predicates__test_isroutine"
# subject = "cpython.test_inspect.TestPredicates.test_isroutine"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_inspect.py::TestPredicates::test_isroutine
"""Auto-ported test: TestPredicates::test_isroutine (CPython 3.12 oracle)."""


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

class GetSourceBase(unittest.TestCase):
    fodderModule = None

    def setUp(self):
        with open(inspect.getsourcefile(self.fodderModule), encoding='utf-8') as fp:
            self.source = fp.read()

    def sourcerange(self, top, bottom):
        lines = self.source.split('\n')
        return '\n'.join(lines[top - 1:bottom]) + ('\n' if bottom else '')

    def assertSourceEqual(self, obj, top, bottom):
        self.assertEqual(inspect.getsource(obj), self.sourcerange(top, bottom))

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
predicates = set([inspect.isbuiltin, inspect.isclass, inspect.iscode, inspect.isframe, inspect.isfunction, inspect.ismethod, inspect.ismodule, inspect.istraceback, inspect.isgenerator, inspect.isgeneratorfunction, inspect.iscoroutine, inspect.iscoroutinefunction, inspect.isasyncgen, inspect.isasyncgenfunction, inspect.ismethodwrapper])

assert inspect.isroutine(git.argue)

assert inspect.isroutine(mod.custom_method)

assert inspect.isroutine([].count)

assert inspect.isroutine(mod.spam)

assert inspect.isroutine(mod.StupidGit.abuse)

assert inspect.isroutine(object.__init__)

assert inspect.isroutine(object.__str__)

assert inspect.isroutine(object.__lt__)

assert inspect.isroutine(int.__lt__)

assert inspect.isroutine(object().__init__)

assert inspect.isroutine(object().__str__)

assert inspect.isroutine(object().__lt__)

assert inspect.isroutine(42 .__lt__)

assert inspect.isroutine(str.join)

assert inspect.isroutine(list.append)

assert inspect.isroutine(''.join)

assert inspect.isroutine([].append)

assert not inspect.isroutine(object)

assert not inspect.isroutine(object())

assert not inspect.isroutine(str())

assert not inspect.isroutine(mod)

assert not inspect.isroutine(type)

assert not inspect.isroutine(int)

assert not inspect.isroutine(type('some_class', (), {}))
print("TestPredicates::test_isroutine: ok")
