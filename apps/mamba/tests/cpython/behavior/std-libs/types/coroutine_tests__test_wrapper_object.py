# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_wrapper_object"
# subject = "cpython.test_types.CoroutineTests.test_wrapper_object"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_wrapper_object
"""Auto-ported test: CoroutineTests::test_wrapper_object (CPython 3.12 oracle)."""


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

@types.coroutine
def coro():
    return gen()
wrapper = coro()

assert 'GeneratorWrapper' in repr(wrapper)

assert repr(wrapper) == str(wrapper)

assert set(dir(wrapper)).issuperset({'__await__', '__iter__', '__next__', 'cr_code', 'cr_running', 'cr_frame', 'gi_code', 'gi_frame', 'gi_running', 'send', 'close', 'throw'})
print("CoroutineTests::test_wrapper_object: ok")
