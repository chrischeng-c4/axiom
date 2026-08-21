# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_union_parameter_chaining"
# subject = "cpython.test_types.UnionTests.test_union_parameter_chaining"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_union_parameter_chaining
"""Auto-ported test: UnionTests::test_union_parameter_chaining (CPython 3.12 oracle)."""


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
S = typing.TypeVar('S')

assert (float | list[T])[int] == float | list[int]

assert list[int | list[T]].__parameters__ == (T,)

assert list[int | list[T]][str] == list[int | list[str]]

assert (list[T] | list[S]).__parameters__ == (T, S)

assert (list[T] | list[S])[int, T] == list[int] | list[T]

assert (list[T] | list[S])[int, int] == list[int]
print("UnionTests::test_union_parameter_chaining: ok")
