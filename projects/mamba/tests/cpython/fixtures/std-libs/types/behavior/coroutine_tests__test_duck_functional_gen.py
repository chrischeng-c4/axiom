# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_duck_functional_gen"
# subject = "cpython.test_types.CoroutineTests.test_duck_functional_gen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_duck_functional_gen
"""Auto-ported test: CoroutineTests::test_duck_functional_gen (CPython 3.12 oracle)."""


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
class Generator:
    """Emulates the following generator (very clumsy):

              def gen(fut):
                  result = yield fut
                  return result * 2
            """

    def __init__(self, fut):
        self._i = 0
        self._fut = fut

    def __iter__(self):
        return self

    def __next__(self):
        return self.send(None)

    def send(self, v):
        try:
            if self._i == 0:
                assert v is None
                return self._fut
            if self._i == 1:
                raise StopIteration(v * 2)
            if self._i > 1:
                raise StopIteration
        finally:
            self._i += 1

    def throw(self, tp, *exc):
        self._i = 100
        if tp is not GeneratorExit:
            raise tp

    def close(self):
        self.throw(GeneratorExit)

@types.coroutine
def foo():
    return Generator('spam')
wrapper = foo()

assert isinstance(wrapper, types._GeneratorWrapper)

async def corofunc():
    return await foo() + 100
coro = corofunc()

assert coro.send(None) == 'spam'
try:
    coro.send(20)
except StopIteration as ex:

    assert ex.args[0] == 140
else:

    raise AssertionError('StopIteration was expected')
print("CoroutineTests::test_duck_functional_gen: ok")
