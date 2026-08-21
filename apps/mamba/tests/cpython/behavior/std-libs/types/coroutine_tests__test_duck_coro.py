# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_duck_coro"
# subject = "cpython.test_types.CoroutineTests.test_duck_coro"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_duck_coro
"""Auto-ported test: CoroutineTests::test_duck_coro (CPython 3.12 oracle)."""


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
class CoroLike:

    def send(self):
        pass

    def throw(self):
        pass

    def close(self):
        pass

    def __await__(self):
        return self
coro = CoroLike()

@types.coroutine
def foo():
    return coro

assert foo() is coro

assert foo().__await__() is coro
print("CoroutineTests::test_duck_coro: ok")
