# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_genfunc"
# subject = "cpython.test_types.CoroutineTests.test_genfunc"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_genfunc
"""Auto-ported test: CoroutineTests::test_genfunc (CPython 3.12 oracle)."""


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
def gen():
    yield

assert types.coroutine(gen) is gen

assert types.coroutine(types.coroutine(gen)) is gen

assert gen.__code__.co_flags & inspect.CO_ITERABLE_COROUTINE

assert not gen.__code__.co_flags & inspect.CO_COROUTINE
g = gen()

assert g.gi_code.co_flags & inspect.CO_ITERABLE_COROUTINE

assert not g.gi_code.co_flags & inspect.CO_COROUTINE

assert types.coroutine(gen) is gen
print("CoroutineTests::test_genfunc: ok")
