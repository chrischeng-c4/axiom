# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_non_gen_values"
# subject = "cpython.test_types.CoroutineTests.test_non_gen_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_non_gen_values
"""Auto-ported test: CoroutineTests::test_non_gen_values (CPython 3.12 oracle)."""


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
@types.coroutine
def foo():
    return 'spam'

assert foo() == 'spam'

class Awaitable:

    def __await__(self):
        return ()
aw = Awaitable()

@types.coroutine
def foo():
    return aw

assert aw is foo()
foo = types.coroutine(foo)

assert aw is foo()
print("CoroutineTests::test_non_gen_values: ok")
