# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_async_def"
# subject = "cpython.test_types.CoroutineTests.test_async_def"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_async_def
"""Auto-ported test: CoroutineTests::test_async_def (CPython 3.12 oracle)."""


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
async def foo():
    pass
foo_code = foo.__code__
foo_flags = foo.__code__.co_flags
decorated_foo = types.coroutine(foo)

assert foo is decorated_foo

assert foo.__code__.co_flags == foo_flags

assert decorated_foo.__code__ is foo_code
foo_coro = foo()

def bar():
    return foo_coro
for _ in range(2):
    bar = types.coroutine(bar)
    coro = bar()

    assert foo_coro is coro

    assert coro.cr_code.co_flags == foo_flags
    coro.close()
print("CoroutineTests::test_async_def: ok")
