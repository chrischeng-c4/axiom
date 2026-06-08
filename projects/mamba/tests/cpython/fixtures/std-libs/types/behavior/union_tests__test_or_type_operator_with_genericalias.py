# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_or_type_operator_with_genericalias"
# subject = "cpython.test_types.UnionTests.test_or_type_operator_with_genericalias"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_or_type_operator_with_genericalias
"""Auto-ported test: UnionTests::test_or_type_operator_with_genericalias (CPython 3.12 oracle)."""


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
a = list[int]
b = list[str]
c = dict[float, str]

class SubClass(types.GenericAlias):
    ...
d = SubClass(list, float)

assert a | b | c | d == typing.Union[a, b, c, d]

assert a | c | b | b | a | c | d | d == a | b | c | d

assert a | b | d == b | a | d

assert repr(a | b | c | d) == 'list[int] | list[str] | dict[float, str] | list[float]'

class BadType(type):

    def __eq__(self, other):
        return 1 / 0
bt = BadType('bt', (), {})
try:
    list[int] | list[bt]
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass
union_ga = (list[str] | int, collections.abc.Callable[..., str] | int, d | int)
for type_ in union_ga:
    try:
        isinstance(1, type_)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        issubclass(int, type_)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("UnionTests::test_or_type_operator_with_genericalias: ok")
