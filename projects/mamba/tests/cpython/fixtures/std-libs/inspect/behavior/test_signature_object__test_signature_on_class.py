# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_signature_object__test_signature_on_class"
# subject = "cpython.test_inspect.TestSignatureObject.test_signature_on_class"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_inspect.py::TestSignatureObject::test_signature_on_class
"""Auto-ported test: TestSignatureObject::test_signature_on_class (CPython 3.12 oracle)."""


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

class C:

    def __init__(self, a):
        pass

assert signature(C) == ((('a', ..., ..., 'positional_or_keyword'),), ...)

class CM(type):

    def __call__(cls, a):
        pass

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert signature(C) == ((('a', ..., ..., 'positional_or_keyword'),), ...)

class CM(type):

    @classmethod
    def __call__(cls, a):
        return a

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert C(1) == 1

assert signature(C) == ((('a', ..., ..., 'positional_or_keyword'),), ...)

class CM(type):

    @staticmethod
    def __call__(a):
        return a

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert C(1) == 1

assert signature(C) == ((('a', ..., ..., 'positional_or_keyword'),), ...)

class A:

    def call(self, a):
        return a

class CM(type):
    __call__ = A().call

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert C(1) == 1

assert signature(C) == ((('a', ..., ..., 'positional_or_keyword'),), ...)

class CM(type):
    __call__ = functools.partial(lambda x, a: (x, a), 2)

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert C(1) == (2, 1)

assert signature(C) == ((('a', ..., ..., 'positional_or_keyword'),), ...)

class CM(type):
    __call__ = functools.partialmethod(lambda self, x, a: (x, a), 2)

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert C(1) == (2, 1)

assert signature(C) == ((('a', ..., ..., 'positional_or_keyword'),), ...)

class CM(type):
    __call__ = ':'.join

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert C(['a', 'bc']) == 'a:bc'
try:

    assert signature(C) == signature(''.join)
    raise AssertionError('expected AssertionError')
except AssertionError:
    pass

class CM(type):
    __call__ = 2 .__pow__

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert C(3) == 8

assert C(3, 7) == 1
try:

    assert signature(C) == signature(0 .__pow__)
    raise AssertionError('expected AssertionError')
except AssertionError:
    pass

class CM(type):

    def __new__(mcls, name, bases, dct, *, foo=1):
        return super().__new__(mcls, name, bases, dct)

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert signature(C) == ((('b', ..., ..., 'positional_or_keyword'),), ...)

assert signature(CM) == ((('name', ..., ..., 'positional_or_keyword'), ('bases', ..., ..., 'positional_or_keyword'), ('dct', ..., ..., 'positional_or_keyword'), ('foo', 1, ..., 'keyword_only')), ...)

class CMM(type):

    def __new__(mcls, name, bases, dct, *, foo=1):
        return super().__new__(mcls, name, bases, dct)

    def __call__(cls, nm, bs, dt):
        return type(nm, bs, dt)

class CM(type, metaclass=CMM):

    def __new__(mcls, name, bases, dct, *, bar=2):
        return super().__new__(mcls, name, bases, dct)

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert signature(CMM) == ((('name', ..., ..., 'positional_or_keyword'), ('bases', ..., ..., 'positional_or_keyword'), ('dct', ..., ..., 'positional_or_keyword'), ('foo', 1, ..., 'keyword_only')), ...)

assert signature(CM) == ((('nm', ..., ..., 'positional_or_keyword'), ('bs', ..., ..., 'positional_or_keyword'), ('dt', ..., ..., 'positional_or_keyword')), ...)

assert signature(C) == ((('b', ..., ..., 'positional_or_keyword'),), ...)

class CM(type):

    def __init__(cls, name, bases, dct, *, bar=2):
        return super().__init__(name, bases, dct)

class C(metaclass=CM):

    def __init__(self, b):
        pass

assert signature(CM) == ((('name', ..., ..., 'positional_or_keyword'), ('bases', ..., ..., 'positional_or_keyword'), ('dct', ..., ..., 'positional_or_keyword'), ('bar', 2, ..., 'keyword_only')), ...)
print("TestSignatureObject::test_signature_on_class: ok")
