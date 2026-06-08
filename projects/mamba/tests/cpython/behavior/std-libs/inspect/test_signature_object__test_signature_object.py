# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_signature_object__test_signature_object"
# subject = "cpython.test_inspect.TestSignatureObject.test_signature_object"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_inspect.py::TestSignatureObject::test_signature_object
"""Auto-ported test: TestSignatureObject::test_signature_object (CPython 3.12 oracle)."""


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
S = inspect.Signature
P = inspect.Parameter

assert str(S()) == '()'

assert repr(S().parameters) == 'mappingproxy(OrderedDict())'

def test(po, /, pk, pkd=100, *args, ko, kod=10, **kwargs):
    pass
sig = inspect.signature(test)

assert repr(sig).startswith('<Signature')

assert '(po, /, pk' in repr(sig)

def test2(pod=42, /):
    pass
sig2 = inspect.signature(test2)

assert repr(sig2).startswith('<Signature')

assert '(pod=42, /)' in repr(sig2)
po = sig.parameters['po']
pod = sig2.parameters['pod']
pk = sig.parameters['pk']
pkd = sig.parameters['pkd']
args = sig.parameters['args']
ko = sig.parameters['ko']
kod = sig.parameters['kod']
kwargs = sig.parameters['kwargs']
S((po, pk, args, ko, kwargs))
S((po, pk, ko, kod))
S((po, pod, ko))
S((po, pod, kod))
S((pod, ko, kod))
S((pod, kod))
S((pod, args, kod, kwargs))
S((kod, ko))
S((kod, ko, kwargs))
S((args, kod, ko))
try:
    S((pk, po, args, ko, kwargs))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('wrong parameter order', str(_aR_e))
try:
    S((po, args, pk, ko, kwargs))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('wrong parameter order', str(_aR_e))
try:
    S((args, po, pk, ko, kwargs))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('wrong parameter order', str(_aR_e))
try:
    S((po, pk, args, kwargs, ko))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('wrong parameter order', str(_aR_e))
kwargs2 = kwargs.replace(name='args')
try:
    S((po, pk, args, kwargs2, ko))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('duplicate parameter name', str(_aR_e))
try:
    S((pod, po))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('follows default argument', str(_aR_e))
try:
    S((pod, pk))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('follows default argument', str(_aR_e))
try:
    S((po, pod, pk))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('follows default argument', str(_aR_e))
try:
    S((po, pkd, pk))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('follows default argument', str(_aR_e))
try:
    S((pkd, pk))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('follows default argument', str(_aR_e))
print("TestSignatureObject::test_signature_object: ok")
