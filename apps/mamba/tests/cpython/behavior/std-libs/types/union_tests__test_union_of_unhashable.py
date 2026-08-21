# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_union_of_unhashable"
# subject = "cpython.test_types.UnionTests.test_union_of_unhashable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_union_of_unhashable
"""Auto-ported test: UnionTests::test_union_of_unhashable (CPython 3.12 oracle)."""


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
class UnhashableMeta(type):
    __hash__ = None

class A(metaclass=UnhashableMeta):
    ...

class B(metaclass=UnhashableMeta):
    ...

assert (A | B).__args__ == (A, B)
union1 = A | B
try:
    hash(union1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
union2 = int | B
try:
    hash(union2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
union3 = A | int
try:
    hash(union3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("UnionTests::test_union_of_unhashable: ok")
