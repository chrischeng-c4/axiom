# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_predicates__test_excluding_predicates"
# subject = "cpython.test_inspect.TestPredicates.test_excluding_predicates"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_inspect.py::TestPredicates::test_excluding_predicates
"""Auto-ported test: TestPredicates::test_excluding_predicates (CPython 3.12 oracle)."""


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
predicates = set([inspect.isbuiltin, inspect.isclass, inspect.iscode, inspect.isframe, inspect.isfunction, inspect.ismethod, inspect.ismodule, inspect.istraceback, inspect.isgenerator, inspect.isgeneratorfunction, inspect.iscoroutine, inspect.iscoroutinefunction, inspect.isasyncgen, inspect.isasyncgenfunction, inspect.ismethodwrapper])

def istest(predicate, exp):
    obj = eval(exp)

    assert predicate(obj)
    for other in predicates - set([predicate]):
        if (predicate == inspect.isgeneratorfunction or predicate == inspect.isasyncgenfunction or predicate == inspect.iscoroutinefunction) and other == inspect.isfunction:
            continue

        assert not other(obj)
global tb
istest(inspect.isbuiltin, 'sys.exit')
istest(inspect.isbuiltin, '[].append')
istest(inspect.iscode, 'mod.spam.__code__')
try:
    1 / 0
except Exception as e:
    tb = e.__traceback__
    istest(inspect.isframe, 'tb.tb_frame')
    istest(inspect.istraceback, 'tb')
    if hasattr(types, 'GetSetDescriptorType'):
        istest(inspect.isgetsetdescriptor, 'type(tb.tb_frame).f_locals')
    else:

        assert not inspect.isgetsetdescriptor(type(tb.tb_frame).f_locals)
finally:
    tb = None
istest(inspect.isfunction, 'mod.spam')
istest(inspect.isfunction, 'mod.StupidGit.abuse')
istest(inspect.ismethod, 'git.argue')
istest(inspect.ismethod, 'mod.custom_method')
istest(inspect.ismodule, 'mod')
istest(inspect.isdatadescriptor, 'collections.defaultdict.default_factory')
istest(inspect.isgenerator, '(x for x in range(2))')
istest(inspect.isgeneratorfunction, 'generator_function_example')
istest(inspect.isasyncgen, 'async_generator_function_example(1)')
istest(inspect.isasyncgenfunction, 'async_generator_function_example')
with warnings.catch_warnings():
    warnings.simplefilter('ignore')
    istest(inspect.iscoroutine, 'coroutine_function_example(1)')
    istest(inspect.iscoroutinefunction, 'coroutine_function_example')
if hasattr(types, 'MemberDescriptorType'):
    istest(inspect.ismemberdescriptor, 'datetime.timedelta.days')
else:

    assert not inspect.ismemberdescriptor(datetime.timedelta.days)
istest(inspect.ismethodwrapper, 'object().__str__')
istest(inspect.ismethodwrapper, 'object().__eq__')
istest(inspect.ismethodwrapper, 'object().__repr__')

assert not inspect.ismethodwrapper(type)

assert not inspect.ismethodwrapper(int)

assert not inspect.ismethodwrapper(type('AnyClass', (), {}))
print("TestPredicates::test_excluding_predicates: ok")
