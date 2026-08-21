# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_signature_object__test_signature_on_partial"
# subject = "cpython.test_inspect.TestSignatureObject.test_signature_on_partial"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_inspect.py::TestSignatureObject::test_signature_on_partial
"""Auto-ported test: TestSignatureObject::test_signature_on_partial (CPython 3.12 oracle)."""


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
def signature(func, **kw):
    sig = inspect.signature(func, **kw)
    return (tuple(((param.name, ... if param.default is param.empty else param.default, ... if param.annotation is param.empty else param.annotation, str(param.kind).lower()) for param in sig.parameters.values())), ... if sig.return_annotation is sig.empty else sig.return_annotation)
from functools import partial

def test():
    pass

assert signature(partial(test)) == ((), ...)
try:
    inspect.signature(partial(test, 1))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('has incorrect arguments', str(_aR_e))
try:
    inspect.signature(partial(test, a=1))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('has incorrect arguments', str(_aR_e))

def test(a, b, *, c, d):
    pass

assert signature(partial(test)) == ((('a', ..., ..., 'positional_or_keyword'), ('b', ..., ..., 'positional_or_keyword'), ('c', ..., ..., 'keyword_only'), ('d', ..., ..., 'keyword_only')), ...)

assert signature(partial(test, 1)) == ((('b', ..., ..., 'positional_or_keyword'), ('c', ..., ..., 'keyword_only'), ('d', ..., ..., 'keyword_only')), ...)

assert signature(partial(test, 1, c=2)) == ((('b', ..., ..., 'positional_or_keyword'), ('c', 2, ..., 'keyword_only'), ('d', ..., ..., 'keyword_only')), ...)

assert signature(partial(test, b=1, c=2)) == ((('a', ..., ..., 'positional_or_keyword'), ('b', 1, ..., 'keyword_only'), ('c', 2, ..., 'keyword_only'), ('d', ..., ..., 'keyword_only')), ...)

assert signature(partial(test, 0, b=1, c=2)) == ((('b', 1, ..., 'keyword_only'), ('c', 2, ..., 'keyword_only'), ('d', ..., ..., 'keyword_only')), ...)

assert signature(partial(test, a=1)) == ((('a', 1, ..., 'keyword_only'), ('b', ..., ..., 'keyword_only'), ('c', ..., ..., 'keyword_only'), ('d', ..., ..., 'keyword_only')), ...)

def test(a, *args, b, **kwargs):
    pass

assert signature(partial(test, 1)) == ((('args', ..., ..., 'var_positional'), ('b', ..., ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(test, a=1)) == ((('a', 1, ..., 'keyword_only'), ('b', ..., ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(test, 1, 2, 3)) == ((('args', ..., ..., 'var_positional'), ('b', ..., ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(test, 1, 2, 3, test=True)) == ((('args', ..., ..., 'var_positional'), ('b', ..., ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(test, 1, 2, 3, test=1, b=0)) == ((('args', ..., ..., 'var_positional'), ('b', 0, ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(test, b=0)) == ((('a', ..., ..., 'positional_or_keyword'), ('args', ..., ..., 'var_positional'), ('b', 0, ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(test, b=0, test=1)) == ((('a', ..., ..., 'positional_or_keyword'), ('args', ..., ..., 'var_positional'), ('b', 0, ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)

def test(a, b, c: int) -> 42:
    pass
sig = test.__signature__ = inspect.signature(test)

assert signature(partial(partial(test, 1))) == ((('b', ..., ..., 'positional_or_keyword'), ('c', ..., int, 'positional_or_keyword')), 42)

assert signature(partial(partial(test, 1), 2)) == ((('c', ..., int, 'positional_or_keyword'),), 42)

def foo(a):
    return a
_foo = partial(partial(foo, a=10), a=20)

assert signature(_foo) == ((('a', 20, ..., 'keyword_only'),), ...)

assert _foo() == 20

def foo(a, b, c):
    return (a, b, c)
_foo = partial(partial(foo, 1, b=20), b=30)

assert signature(_foo) == ((('b', 30, ..., 'keyword_only'), ('c', ..., ..., 'keyword_only')), ...)

assert _foo(c=10) == (1, 30, 10)

def foo(a, b, c, *, d):
    return (a, b, c, d)
_foo = partial(partial(foo, d=20, c=20), b=10, d=30)

assert signature(_foo) == ((('a', ..., ..., 'positional_or_keyword'), ('b', 10, ..., 'keyword_only'), ('c', 20, ..., 'keyword_only'), ('d', 30, ..., 'keyword_only')), ...)
ba = inspect.signature(_foo).bind(a=200, b=11)

assert _foo(*ba.args, **ba.kwargs) == (200, 11, 20, 30)

def foo(a=1, b=2, c=3):
    return (a, b, c)
_foo = partial(foo, c=13)
ba = inspect.signature(_foo).bind(a=11)

assert _foo(*ba.args, **ba.kwargs) == (11, 2, 13)
ba = inspect.signature(_foo).bind(11, 12)

assert _foo(*ba.args, **ba.kwargs) == (11, 12, 13)
ba = inspect.signature(_foo).bind(11, b=12)

assert _foo(*ba.args, **ba.kwargs) == (11, 12, 13)
ba = inspect.signature(_foo).bind(b=12)

assert _foo(*ba.args, **ba.kwargs) == (1, 12, 13)
_foo = partial(_foo, b=10, c=20)
ba = inspect.signature(_foo).bind(12)

assert _foo(*ba.args, **ba.kwargs) == (12, 10, 20)

def foo(a, b, /, c, d, **kwargs):
    pass
sig = inspect.signature(foo)

assert str(sig) == '(a, b, /, c, d, **kwargs)'

assert signature(partial(foo, 1)) == ((('b', ..., ..., 'positional_only'), ('c', ..., ..., 'positional_or_keyword'), ('d', ..., ..., 'positional_or_keyword'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(foo, 1, 2)) == ((('c', ..., ..., 'positional_or_keyword'), ('d', ..., ..., 'positional_or_keyword'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(foo, 1, 2, 3)) == ((('d', ..., ..., 'positional_or_keyword'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(foo, 1, 2, c=3)) == ((('c', 3, ..., 'keyword_only'), ('d', ..., ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)

assert signature(partial(foo, 1, c=3)) == ((('b', ..., ..., 'positional_only'), ('c', 3, ..., 'keyword_only'), ('d', ..., ..., 'keyword_only'), ('kwargs', ..., ..., 'var_keyword')), ...)
print("TestSignatureObject::test_signature_on_partial: ok")
