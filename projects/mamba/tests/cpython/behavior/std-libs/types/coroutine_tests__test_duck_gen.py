# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_duck_gen"
# subject = "cpython.test_types.CoroutineTests.test_duck_gen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_duck_gen
"""Auto-ported test: CoroutineTests::test_duck_gen (CPython 3.12 oracle)."""


from test.support import run_with_locale, cpython_only, iter_builtin_types, iter_slot_wrappers, MISSING_C_DOCSTRINGS
from test.test_import import no_rerun
import collections.abc
from collections import namedtuple
import copy
import gc
import inspect
import pickle
import locale
import sys
import textwrap
import types
import unittest.mock
import weakref
import typing


T = typing.TypeVar('T')

class Example:
    pass

class Forward:
    ...

def clear_typing_caches():
    for f in typing._cleanups:
        f()


# --- test body ---
class GenLike:

    def send(self):
        pass

    def throw(self):
        pass

    def close(self):
        pass

    def __iter__(self):
        pass

    def __next__(self):
        pass
gen = unittest.mock.MagicMock(GenLike)
gen.__iter__ = lambda gen: gen
gen.__name__ = 'gen'
gen.__qualname__ = 'test.gen'

assert isinstance(gen, collections.abc.Generator)

assert gen is iter(gen)

@types.coroutine
def foo():
    return gen
wrapper = foo()

assert isinstance(wrapper, types._GeneratorWrapper)

assert wrapper.__await__() is wrapper

assert iter(wrapper) is wrapper

assert isinstance(wrapper, collections.abc.Coroutine)

assert isinstance(wrapper, collections.abc.Awaitable)

assert wrapper.__qualname__ is gen.__qualname__

assert wrapper.__name__ is gen.__name__
for name in {'gi_running', 'gi_frame', 'gi_code', 'gi_yieldfrom', 'cr_running', 'cr_frame', 'cr_code', 'cr_await'}:
    try:
        getattr(wrapper, name)
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
gen.gi_running = object()
gen.gi_frame = object()
gen.gi_code = object()
gen.gi_yieldfrom = object()

assert wrapper.gi_running is gen.gi_running

assert wrapper.gi_frame is gen.gi_frame

assert wrapper.gi_code is gen.gi_code

assert wrapper.gi_yieldfrom is gen.gi_yieldfrom

assert wrapper.cr_running is gen.gi_running

assert wrapper.cr_frame is gen.gi_frame

assert wrapper.cr_code is gen.gi_code

assert wrapper.cr_await is gen.gi_yieldfrom
wrapper.close()
gen.close.assert_called_once_with()
wrapper.send(1)
gen.send.assert_called_once_with(1)
gen.reset_mock()
next(wrapper)
gen.__next__.assert_called_once_with()
gen.reset_mock()
wrapper.throw(1, 2, 3)
gen.throw.assert_called_once_with(1, 2, 3)
gen.reset_mock()
wrapper.throw(1, 2)
gen.throw.assert_called_once_with(1, 2)
gen.reset_mock()
wrapper.throw(1)
gen.throw.assert_called_once_with(1)
gen.reset_mock()
error = Exception()
gen.throw.side_effect = error
try:
    wrapper.throw(1)
except Exception as ex:

    assert ex is error
else:

    raise AssertionError('wrapper did not propagate an exception')
gen.reset_mock()
try:
    wrapper.throw()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not gen.throw.called
try:
    wrapper.close(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not gen.close.called
try:
    wrapper.send()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not gen.send.called

@types.coroutine
def bar():
    return wrapper

assert wrapper is bar()
ref = weakref.ref(wrapper)

assert ref() is wrapper
print("CoroutineTests::test_duck_gen: ok")
