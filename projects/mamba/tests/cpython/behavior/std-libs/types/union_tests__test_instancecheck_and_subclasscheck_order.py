# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_instancecheck_and_subclasscheck_order"
# subject = "cpython.test_types.UnionTests.test_instancecheck_and_subclasscheck_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_instancecheck_and_subclasscheck_order
"""Auto-ported test: UnionTests::test_instancecheck_and_subclasscheck_order (CPython 3.12 oracle)."""


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
T = typing.TypeVar('T')
will_resolve = (int | T, typing.Union[int, T])
for x in will_resolve:

    assert isinstance(1, x)

    assert issubclass(int, x)
wont_resolve = (T | int, typing.Union[T, int])
for x in wont_resolve:
    try:
        issubclass(int, x)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        isinstance(1, x)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
for x in (*will_resolve, *wont_resolve):
    try:
        issubclass(object, x)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        isinstance(object(), x)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("UnionTests::test_instancecheck_and_subclasscheck_order: ok")
