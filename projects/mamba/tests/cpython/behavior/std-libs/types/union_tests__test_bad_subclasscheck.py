# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_bad_subclasscheck"
# subject = "cpython.test_types.UnionTests.test_bad_subclasscheck"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_bad_subclasscheck
"""Auto-ported test: UnionTests::test_bad_subclasscheck (CPython 3.12 oracle)."""


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
class BadMeta(type):

    def __subclasscheck__(cls, sub):
        1 / 0
x = int | BadMeta('A', (), {})

assert issubclass(int, x)

try:
    issubclass(list, x)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass
print("UnionTests::test_bad_subclasscheck: ok")
